use anyhow::{anyhow, Context, Result};
use chrono::{Local, NaiveDateTime};
use rusqlite::params;
use uuid::Uuid;

use crate::db::Database;
use crate::models::{NewTimeEntry, TimeEntry};

impl Database {
    /// Insert a new time entry, generating a UUID automatically.
    /// Also creates/links any tags specified in the NewTimeEntry.
    pub fn insert_entry(&self, new: &NewTimeEntry) -> Result<TimeEntry> {
        let uuid = Uuid::new_v4().to_string();
        let start_str = new.start_time.format("%Y-%m-%dT%H:%M:%S").to_string();
        let end_str = new.end_time.map(|t| t.format("%Y-%m-%dT%H:%M:%S").to_string());

        let duration_secs = match (new.end_time, Some(new.start_time)) {
            (Some(end), Some(start)) => {
                Some((end - start).num_seconds())
            }
            _ => None,
        };

        let billable_int: i32 = if new.billable { 1 } else { 0 };

        self.conn.execute(
            "INSERT INTO time_entries (uuid, project_id, description, start_time, end_time,
             duration_secs, billable)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![uuid, new.project_id, new.description, start_str, end_str,
                    duration_secs, billable_int],
        ).context("Failed to insert time entry")?;

        let entry_id = self.conn.last_insert_rowid();

        // Link tags
        for tag_name in &new.tags {
            let tag = self.get_or_create_tag(tag_name)?;
            self.conn.execute(
                "INSERT OR IGNORE INTO time_entry_tags (entry_id, tag_id) VALUES (?1, ?2)",
                params![entry_id, tag.id],
            ).context("Failed to link tag to entry")?;
        }

        self.get_entry(entry_id)
    }

    /// Get a time entry by id.
    pub fn get_entry(&self, id: i64) -> Result<TimeEntry> {
        let entry = self.conn.query_row(
            "SELECT id, uuid, project_id, description, start_time, end_time,
                    duration_secs, billable, created_at, updated_at
             FROM time_entries WHERE id = ?1",
            params![id],
            row_to_entry,
        ).context("Failed to get time entry")?;

        Ok(entry)
    }

    /// Get the currently active (running) time entry, if any.
    pub fn get_active_entry(&self) -> Result<Option<TimeEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, uuid, project_id, description, start_time, end_time,
                    duration_secs, billable, created_at, updated_at
             FROM time_entries WHERE end_time IS NULL
             ORDER BY start_time DESC LIMIT 1",
        ).context("Failed to prepare get_active_entry query")?;

        let mut rows = stmt.query_map([], row_to_entry)
            .context("Failed to execute get_active_entry query")?;

        match rows.next() {
            Some(row) => Ok(Some(row.context("Failed to read active entry row")?)),
            None => Ok(None),
        }
    }

    /// Stop the currently active time entry by setting end_time to now and
    /// computing duration_secs. Optionally update the description.
    /// Returns the stopped entry, or None if there was no active entry.
    pub fn stop_active_entry(&self, description: Option<&str>) -> Result<Option<TimeEntry>> {
        let active = self.get_active_entry()?;

        let entry = match active {
            Some(entry) => entry,
            None => return Ok(None),
        };

        let now = Local::now().naive_local();
        let now_str = now.format("%Y-%m-%dT%H:%M:%S").to_string();
        let duration = (now - entry.start_time).num_seconds();

        if let Some(desc) = description {
            self.conn.execute(
                "UPDATE time_entries SET end_time = ?1, duration_secs = ?2, description = ?3,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%S','now')
                 WHERE id = ?4",
                params![now_str, duration, desc, entry.id],
            ).context("Failed to stop active entry with description")?;
        } else {
            self.conn.execute(
                "UPDATE time_entries SET end_time = ?1, duration_secs = ?2,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%S','now')
                 WHERE id = ?3",
                params![now_str, duration, entry.id],
            ).context("Failed to stop active entry")?;
        }

        self.get_entry(entry.id).map(Some)
    }

    /// Delete a time entry by id.
    pub fn delete_entry(&self, id: i64) -> Result<()> {
        let affected = self.conn.execute(
            "DELETE FROM time_entries WHERE id = ?1",
            params![id],
        ).context("Failed to delete time entry")?;

        if affected == 0 {
            return Err(anyhow!("No time entry found with id {}", id));
        }
        Ok(())
    }

    /// List time entries with optional filters for project, date range, and limit.
    pub fn list_entries(
        &self,
        project_id: Option<i64>,
        from: Option<NaiveDateTime>,
        to: Option<NaiveDateTime>,
        limit: Option<i64>,
    ) -> Result<Vec<TimeEntry>> {
        let mut sql = String::from(
            "SELECT id, uuid, project_id, description, start_time, end_time,
                    duration_secs, billable, created_at, updated_at
             FROM time_entries WHERE 1=1"
        );
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(pid) = project_id {
            sql.push_str(" AND project_id = ?");
            param_values.push(Box::new(pid));
        }
        if let Some(from_dt) = from {
            let from_str = from_dt.format("%Y-%m-%dT%H:%M:%S").to_string();
            sql.push_str(" AND start_time >= ?");
            param_values.push(Box::new(from_str));
        }
        if let Some(to_dt) = to {
            let to_str = to_dt.format("%Y-%m-%dT%H:%M:%S").to_string();
            sql.push_str(" AND start_time <= ?");
            param_values.push(Box::new(to_str));
        }

        sql.push_str(" ORDER BY start_time DESC");

        if let Some(lim) = limit {
            sql.push_str(" LIMIT ?");
            param_values.push(Box::new(lim));
        }

        let mut stmt = self.conn.prepare(&sql)
            .context("Failed to prepare list_entries query")?;

        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();

        let entries = stmt.query_map(params_refs.as_slice(), row_to_entry)
            .context("Failed to execute list_entries query")?;

        let mut result = Vec::new();
        for entry in entries {
            result.push(entry.context("Failed to read entry row")?);
        }
        Ok(result)
    }

    /// Get all time entries for today.
    pub fn get_today_entries(&self) -> Result<Vec<TimeEntry>> {
        let today = Local::now().format("%Y-%m-%d").to_string();

        let mut stmt = self.conn.prepare(
            "SELECT id, uuid, project_id, description, start_time, end_time,
                    duration_secs, billable, created_at, updated_at
             FROM time_entries
             WHERE date(start_time) = ?1
             ORDER BY start_time DESC",
        ).context("Failed to prepare get_today_entries query")?;

        let entries = stmt.query_map(params![today], row_to_entry)
            .context("Failed to execute get_today_entries query")?;

        let mut result = Vec::new();
        for entry in entries {
            result.push(entry.context("Failed to read today entry row")?);
        }
        Ok(result)
    }

    /// Cancel (delete) the currently active time entry.
    /// Returns true if an entry was cancelled, false if there was no active entry.
    pub fn cancel_active_entry(&self) -> Result<bool> {
        let active = self.get_active_entry()?;
        match active {
            Some(entry) => {
                self.delete_entry(entry.id)?;
                Ok(true)
            }
            None => Ok(false),
        }
    }
}

/// Map a database row to a TimeEntry struct.
fn row_to_entry(row: &rusqlite::Row) -> rusqlite::Result<TimeEntry> {
    let billable_int: i32 = row.get(7)?;

    let start_str: String = row.get(4)?;
    let end_str: Option<String> = row.get(5)?;
    let created_str: String = row.get(8)?;
    let updated_str: String = row.get(9)?;

    Ok(TimeEntry {
        id: row.get(0)?,
        uuid: row.get(1)?,
        project_id: row.get(2)?,
        description: row.get(3)?,
        start_time: start_str.parse::<NaiveDateTime>().unwrap(),
        end_time: end_str.map(|s| s.parse::<NaiveDateTime>().unwrap()),
        duration_secs: row.get(6)?,
        billable: billable_int != 0,
        created_at: created_str.parse::<NaiveDateTime>().unwrap(),
        updated_at: updated_str.parse::<NaiveDateTime>().unwrap(),
    })
}
