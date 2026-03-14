use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use rusqlite::params;

use crate::db::Database;
use crate::models::{Project, ProjectStatus};

impl Database {
    /// Create a new project.
    pub fn create_project(
        &self,
        name: &str,
        client_id: Option<i64>,
        color: Option<&str>,
        budget_hours: Option<f64>,
        notes: Option<&str>,
    ) -> Result<Project> {
        let color = color.unwrap_or("#5B9BD5");

        self.conn.execute(
            "INSERT INTO projects (name, client_id, color, budget_hours, notes)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![name, client_id, color, budget_hours, notes],
        ).context("Failed to insert project")?;

        let id = self.conn.last_insert_rowid();
        self.get_project(id)
    }

    /// Get a project by id.
    pub fn get_project(&self, id: i64) -> Result<Project> {
        let project = self.conn.query_row(
            "SELECT id, name, client_id, color, status, budget_hours, notes,
                    created_at, updated_at
             FROM projects WHERE id = ?1",
            params![id],
            row_to_project,
        ).context("Failed to get project")?;

        Ok(project)
    }

    /// Find a project by name (case-insensitive).
    pub fn get_project_by_name(&self, name: &str) -> Result<Option<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, client_id, color, status, budget_hours, notes,
                    created_at, updated_at
             FROM projects WHERE LOWER(name) = LOWER(?1)",
        ).context("Failed to prepare get_project_by_name query")?;

        let mut rows = stmt.query_map(params![name], row_to_project)
            .context("Failed to query project by name")?;

        match rows.next() {
            Some(row) => Ok(Some(row.context("Failed to read project row")?)),
            None => Ok(None),
        }
    }

    /// List all projects. If `include_archived` is false, only active projects
    /// are returned.
    pub fn list_projects(&self, include_archived: bool) -> Result<Vec<Project>> {
        let sql = if include_archived {
            "SELECT id, name, client_id, color, status, budget_hours, notes,
                    created_at, updated_at
             FROM projects ORDER BY name"
        } else {
            "SELECT id, name, client_id, color, status, budget_hours, notes,
                    created_at, updated_at
             FROM projects WHERE status = 'active' ORDER BY name"
        };

        let mut stmt = self.conn.prepare(sql)
            .context("Failed to prepare list_projects query")?;

        let projects = stmt.query_map([], row_to_project)
            .context("Failed to execute list_projects query")?;

        let mut result = Vec::new();
        for project in projects {
            result.push(project.context("Failed to read project row")?);
        }
        Ok(result)
    }

    /// Update a project's fields.
    pub fn update_project(
        &self,
        id: i64,
        name: Option<&str>,
        color: Option<&str>,
        budget_hours: Option<f64>,
        notes: Option<&str>,
    ) -> Result<Project> {
        let existing = self.get_project(id)?;

        let new_name = name.unwrap_or(&existing.name);
        let new_color = color.unwrap_or(&existing.color);
        let new_budget = budget_hours.or(existing.budget_hours);
        let new_notes = notes.or(existing.notes.as_deref());

        self.conn.execute(
            "UPDATE projects SET name = ?1, color = ?2, budget_hours = ?3, notes = ?4,
             updated_at = strftime('%Y-%m-%dT%H:%M:%S','now')
             WHERE id = ?5",
            params![new_name, new_color, new_budget, new_notes, id],
        ).context("Failed to update project")?;

        self.get_project(id)
    }

    /// Archive a project by setting its status to 'archived'.
    pub fn archive_project(&self, id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE projects SET status = 'archived',
             updated_at = strftime('%Y-%m-%dT%H:%M:%S','now')
             WHERE id = ?1",
            params![id],
        ).context("Failed to archive project")?;
        Ok(())
    }

    /// Activate a project by setting its status to 'active'.
    pub fn activate_project(&self, id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE projects SET status = 'active',
             updated_at = strftime('%Y-%m-%dT%H:%M:%S','now')
             WHERE id = ?1",
            params![id],
        ).context("Failed to activate project")?;
        Ok(())
    }
}

/// Map a row to a Project struct. Used by multiple query functions.
fn row_to_project(row: &rusqlite::Row) -> rusqlite::Result<Project> {
    let status_str: String = row.get(4)?;
    let status = ProjectStatus::from_str(&status_str)
        .unwrap_or(ProjectStatus::Active);

    Ok(Project {
        id: row.get(0)?,
        name: row.get(1)?,
        client_id: row.get(2)?,
        color: row.get(3)?,
        status,
        budget_hours: row.get(5)?,
        notes: row.get(6)?,
        created_at: row.get::<_, String>(7)?.parse::<NaiveDateTime>().unwrap(),
        updated_at: row.get::<_, String>(8)?.parse::<NaiveDateTime>().unwrap(),
    })
}
