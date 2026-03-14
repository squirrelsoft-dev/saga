use anyhow::{Context, Result};
use rusqlite::params;

use crate::db::Database;
use crate::models::{DailySummary, ProjectBreakdown};

impl Database {
    /// Get a summary of time tracked for a single date (YYYY-MM-DD format).
    pub fn daily_summary(&self, date: &str) -> Result<DailySummary> {
        let summary = self.conn.query_row(
            "SELECT
                ?1 AS date,
                COALESCE(SUM(duration_secs), 0) AS total_seconds,
                COUNT(*) AS entry_count,
                COALESCE(SUM(CASE WHEN billable = 1 THEN duration_secs ELSE 0 END), 0) AS billable_seconds
             FROM time_entries
             WHERE date(start_time) = ?1
               AND end_time IS NOT NULL",
            params![date],
            |row| {
                Ok(DailySummary {
                    date: row.get(0)?,
                    total_seconds: row.get(1)?,
                    entry_count: row.get(2)?,
                    billable_seconds: row.get(3)?,
                })
            },
        ).context("Failed to get daily summary")?;

        Ok(summary)
    }

    /// Get daily summaries for each day in a date range (inclusive).
    /// Dates are in YYYY-MM-DD format.
    pub fn date_range_summary(&self, from: &str, to: &str) -> Result<Vec<DailySummary>> {
        let mut stmt = self.conn.prepare(
            "SELECT
                date(start_time) AS date,
                COALESCE(SUM(duration_secs), 0) AS total_seconds,
                COUNT(*) AS entry_count,
                COALESCE(SUM(CASE WHEN billable = 1 THEN duration_secs ELSE 0 END), 0) AS billable_seconds
             FROM time_entries
             WHERE date(start_time) >= ?1
               AND date(start_time) <= ?2
               AND end_time IS NOT NULL
             GROUP BY date(start_time)
             ORDER BY date(start_time)",
        ).context("Failed to prepare date_range_summary query")?;

        let summaries = stmt.query_map(params![from, to], |row| {
            Ok(DailySummary {
                date: row.get(0)?,
                total_seconds: row.get(1)?,
                entry_count: row.get(2)?,
                billable_seconds: row.get(3)?,
            })
        }).context("Failed to execute date_range_summary query")?;

        let mut result = Vec::new();
        for summary in summaries {
            result.push(summary.context("Failed to read daily summary row")?);
        }
        Ok(result)
    }

    /// Get a breakdown of time tracked per project in a date range.
    /// Includes the effective rate to compute billable amounts where available.
    pub fn project_breakdown(&self, from: &str, to: &str) -> Result<Vec<ProjectBreakdown>> {
        let mut stmt = self.conn.prepare(
            "SELECT
                p.id AS project_id,
                p.name AS project_name,
                p.color AS project_color,
                COALESCE(SUM(te.duration_secs), 0) AS total_seconds,
                COALESCE(SUM(CASE WHEN te.billable = 1 THEN te.duration_secs ELSE 0 END), 0) AS billable_seconds,
                COUNT(te.id) AS entry_count
             FROM time_entries te
             JOIN projects p ON te.project_id = p.id
             WHERE date(te.start_time) >= ?1
               AND date(te.start_time) <= ?2
               AND te.end_time IS NOT NULL
             GROUP BY p.id
             ORDER BY total_seconds DESC",
        ).context("Failed to prepare project_breakdown query")?;

        let rows = stmt.query_map(params![from, to], |row| {
            Ok((
                row.get::<_, i64>(0)?,     // project_id
                row.get::<_, String>(1)?,  // project_name
                row.get::<_, String>(2)?,  // project_color
                row.get::<_, i64>(3)?,     // total_seconds
                row.get::<_, i64>(4)?,     // billable_seconds
                row.get::<_, i64>(5)?,     // entry_count
            ))
        }).context("Failed to execute project_breakdown query")?;

        let mut result = Vec::new();
        for row in rows {
            let (project_id, project_name, project_color, total_seconds, billable_seconds, entry_count) =
                row.context("Failed to read project breakdown row")?;

            // Look up the project to find its client_id for rate resolution.
            let project = self.get_project(project_id)?;
            let amount = match self.get_effective_rate(Some(project_id), project.client_id)? {
                Some(rate) => {
                    let billable_hours = billable_seconds as f64 / 3600.0;
                    Some(billable_hours * rate.hourly_rate)
                }
                None => None,
            };

            result.push(ProjectBreakdown {
                project_id,
                project_name,
                project_color,
                total_seconds,
                billable_seconds,
                entry_count,
                amount,
            });
        }

        Ok(result)
    }
}
