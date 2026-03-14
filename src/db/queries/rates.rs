use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use rusqlite::params;

use crate::db::Database;
use crate::models::{Rate, RateType};

impl Database {
    /// Set (insert or update) a rate. For a given rate_type and associated
    /// project/client, only one rate is active. If one already exists for
    /// that combination, it is replaced.
    pub fn set_rate(
        &self,
        rate_type: &RateType,
        project_id: Option<i64>,
        client_id: Option<i64>,
        hourly_rate: f64,
        currency: Option<&str>,
    ) -> Result<Rate> {
        let currency = currency.unwrap_or("USD");
        let rate_type_str = rate_type.to_string();

        // Delete any existing rate for this combination.
        match rate_type {
            RateType::Default => {
                self.conn.execute(
                    "DELETE FROM rates WHERE rate_type = 'default'",
                    [],
                ).context("Failed to clear existing default rate")?;
            }
            RateType::Project => {
                self.conn.execute(
                    "DELETE FROM rates WHERE rate_type = 'project' AND project_id = ?1",
                    params![project_id],
                ).context("Failed to clear existing project rate")?;
            }
            RateType::Client => {
                self.conn.execute(
                    "DELETE FROM rates WHERE rate_type = 'client' AND client_id = ?1",
                    params![client_id],
                ).context("Failed to clear existing client rate")?;
            }
        }

        self.conn.execute(
            "INSERT INTO rates (rate_type, project_id, client_id, hourly_rate, currency)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![rate_type_str, project_id, client_id, hourly_rate, currency],
        ).context("Failed to insert rate")?;

        let id = self.conn.last_insert_rowid();
        self.get_rate(id)
    }

    /// Get a rate by id.
    fn get_rate(&self, id: i64) -> Result<Rate> {
        let rate = self.conn.query_row(
            "SELECT id, rate_type, project_id, client_id, hourly_rate, currency,
                    effective_from, created_at
             FROM rates WHERE id = ?1",
            params![id],
            row_to_rate,
        ).context("Failed to get rate")?;

        Ok(rate)
    }

    /// List all rates.
    pub fn list_rates(&self) -> Result<Vec<Rate>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, rate_type, project_id, client_id, hourly_rate, currency,
                    effective_from, created_at
             FROM rates ORDER BY rate_type, created_at DESC",
        ).context("Failed to prepare list_rates query")?;

        let rates = stmt.query_map([], row_to_rate)
            .context("Failed to execute list_rates query")?;

        let mut result = Vec::new();
        for rate in rates {
            result.push(rate.context("Failed to read rate row")?);
        }
        Ok(result)
    }

    /// Get the effective rate for a given project/client combination using
    /// cascading resolution: project rate > client rate > default rate.
    pub fn get_effective_rate(
        &self,
        project_id: Option<i64>,
        client_id: Option<i64>,
    ) -> Result<Option<Rate>> {
        // 1. Try project-specific rate
        if let Some(pid) = project_id {
            let result = self.conn.query_row(
                "SELECT id, rate_type, project_id, client_id, hourly_rate, currency,
                        effective_from, created_at
                 FROM rates
                 WHERE rate_type = 'project' AND project_id = ?1
                 ORDER BY effective_from DESC LIMIT 1",
                params![pid],
                row_to_rate,
            );
            match result {
                Ok(rate) => return Ok(Some(rate)),
                Err(rusqlite::Error::QueryReturnedNoRows) => {}
                Err(e) => return Err(e).context("Failed to query project rate"),
            }
        }

        // 2. Try client-specific rate
        if let Some(cid) = client_id {
            let result = self.conn.query_row(
                "SELECT id, rate_type, project_id, client_id, hourly_rate, currency,
                        effective_from, created_at
                 FROM rates
                 WHERE rate_type = 'client' AND client_id = ?1
                 ORDER BY effective_from DESC LIMIT 1",
                params![cid],
                row_to_rate,
            );
            match result {
                Ok(rate) => return Ok(Some(rate)),
                Err(rusqlite::Error::QueryReturnedNoRows) => {}
                Err(e) => return Err(e).context("Failed to query client rate"),
            }
        }

        // 3. Try default rate
        let result = self.conn.query_row(
            "SELECT id, rate_type, project_id, client_id, hourly_rate, currency,
                    effective_from, created_at
             FROM rates
             WHERE rate_type = 'default'
             ORDER BY effective_from DESC LIMIT 1",
            [],
            row_to_rate,
        );
        match result {
            Ok(rate) => Ok(Some(rate)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e).context("Failed to query default rate"),
        }
    }
}

/// Map a database row to a Rate struct.
fn row_to_rate(row: &rusqlite::Row) -> rusqlite::Result<Rate> {
    let rate_type_str: String = row.get(1)?;
    let rate_type = RateType::from_str(&rate_type_str)
        .unwrap_or(RateType::Default);

    Ok(Rate {
        id: row.get(0)?,
        rate_type,
        project_id: row.get(2)?,
        client_id: row.get(3)?,
        hourly_rate: row.get(4)?,
        currency: row.get(5)?,
        effective_from: row.get(6)?,
        created_at: row.get::<_, String>(7)?.parse::<NaiveDateTime>().unwrap(),
    })
}
