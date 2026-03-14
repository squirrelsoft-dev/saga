use anyhow::{Context, Result};
use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime};
use colored::Colorize;
use rusqlite::params;

use crate::config::SagaConfig;
use crate::db::Database;
use crate::models::*;

use super::{
    ClientAction, ConfigAction, InvoiceAction, ProjectAction, RateAction, TagAction,
};

// ---------------------------------------------------------------------------
// Time parsing helpers
// ---------------------------------------------------------------------------

/// Parse a time string in either "HH:MM" (assumes today) or "YYYY-MM-DD HH:MM" format.
fn parse_time(input: &str) -> Result<NaiveDateTime> {
    let input = input.trim();

    // Try full datetime first: "YYYY-MM-DD HH:MM"
    if let Ok(dt) = NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M") {
        return Ok(dt);
    }
    // Also accept with seconds
    if let Ok(dt) = NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S") {
        return Ok(dt);
    }

    // Try time only: "HH:MM" -> assume today
    if let Ok(t) = NaiveTime::parse_from_str(input, "%H:%M") {
        let today = Local::now().date_naive();
        return Ok(NaiveDateTime::new(today, t));
    }

    anyhow::bail!(
        "Could not parse time '{}'. Use HH:MM or YYYY-MM-DD HH:MM.",
        input
    );
}

/// Format a duration given as seconds into a human-readable string like "2h 15m 03s".
fn format_duration(total_secs: i64) -> String {
    if total_secs < 0 {
        return "0s".to_string();
    }
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    if hours > 0 {
        format!("{}h {:02}m {:02}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {:02}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

// ---------------------------------------------------------------------------
// Project / client lookup helpers
// ---------------------------------------------------------------------------

/// Find a project by name, or create it if it does not exist.
fn find_or_create_project(db: &Database, name: &str) -> Result<Project> {
    // Try to find existing project by name.
    let existing: Option<Project> = db.conn().query_row(
        "SELECT id, name, client_id, color, status, budget_hours, notes, created_at, updated_at
         FROM projects WHERE name = ?1 COLLATE NOCASE AND status = 'active'",
        params![name],
        |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                client_id: row.get(2)?,
                color: row.get(3)?,
                status: ProjectStatus::from_str(&row.get::<_, String>(4)?).unwrap_or(ProjectStatus::Active),
                budget_hours: row.get(5)?,
                notes: row.get(6)?,
                created_at: row.get::<_, String>(7)?.parse::<NaiveDateTime>().unwrap(),
                updated_at: row.get::<_, String>(8)?.parse::<NaiveDateTime>().unwrap(),
            })
        },
    ).ok();

    if let Some(project) = existing {
        return Ok(project);
    }

    // Create the project.
    db.conn().execute(
        "INSERT INTO projects (name) VALUES (?1)",
        params![name],
    ).context("Failed to create project")?;

    let id = db.conn().last_insert_rowid();
    db.conn().query_row(
        "SELECT id, name, client_id, color, status, budget_hours, notes, created_at, updated_at
         FROM projects WHERE id = ?1",
        params![id],
        |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                client_id: row.get(2)?,
                color: row.get(3)?,
                status: ProjectStatus::from_str(&row.get::<_, String>(4)?).unwrap_or(ProjectStatus::Active),
                budget_hours: row.get(5)?,
                notes: row.get(6)?,
                created_at: row.get::<_, String>(7)?.parse::<NaiveDateTime>().unwrap(),
                updated_at: row.get::<_, String>(8)?.parse::<NaiveDateTime>().unwrap(),
            })
        },
    ).context("Failed to fetch newly created project")
}

/// Find a client by name. Returns None if not found.
fn find_client_by_name(db: &Database, name: &str) -> Result<Option<Client>> {
    let client = db.conn().query_row(
        "SELECT id, name, contact, email, notes, created_at, updated_at
         FROM clients WHERE name = ?1 COLLATE NOCASE",
        params![name],
        |row| {
            Ok(Client {
                id: row.get(0)?,
                name: row.get(1)?,
                contact: row.get(2)?,
                email: row.get(3)?,
                notes: row.get(4)?,
                created_at: row.get::<_, String>(5)?.parse::<NaiveDateTime>().unwrap(),
                updated_at: row.get::<_, String>(6)?.parse::<NaiveDateTime>().unwrap(),
            })
        },
    ).ok();
    Ok(client)
}

/// Look up a project by its id.
fn get_project_by_id(db: &Database, id: i64) -> Result<Project> {
    db.conn().query_row(
        "SELECT id, name, client_id, color, status, budget_hours, notes, created_at, updated_at
         FROM projects WHERE id = ?1",
        params![id],
        |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                client_id: row.get(2)?,
                color: row.get(3)?,
                status: ProjectStatus::from_str(&row.get::<_, String>(4)?).unwrap_or(ProjectStatus::Active),
                budget_hours: row.get(5)?,
                notes: row.get(6)?,
                created_at: row.get::<_, String>(7)?.parse::<NaiveDateTime>().unwrap(),
                updated_at: row.get::<_, String>(8)?.parse::<NaiveDateTime>().unwrap(),
            })
        },
    ).context("Project not found")
}

// ---------------------------------------------------------------------------
// Tag helpers
// ---------------------------------------------------------------------------

/// Ensure tags exist in the database and attach them to an entry.
fn attach_tags(db: &Database, entry_id: i64, tags: &[String]) -> Result<()> {
    for tag_name in tags {
        // Find or create the tag.
        let tag_id: i64 = match db.conn().query_row(
            "SELECT id FROM tags WHERE name = ?1 COLLATE NOCASE",
            params![tag_name],
            |row| row.get(0),
        ) {
            Ok(id) => id,
            Err(_) => {
                db.conn().execute(
                    "INSERT INTO tags (name) VALUES (?1)",
                    params![tag_name],
                ).context("Failed to create tag")?;
                db.conn().last_insert_rowid()
            }
        };

        db.conn().execute(
            "INSERT OR IGNORE INTO time_entry_tags (entry_id, tag_id) VALUES (?1, ?2)",
            params![entry_id, tag_id],
        ).context("Failed to attach tag to entry")?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Command handlers
// ---------------------------------------------------------------------------

/// Start a new timer for the given project.
pub fn handle_start(
    db: &Database,
    project: &str,
    description: Option<&str>,
    tags: &[String],
    no_billable: bool,
) -> Result<()> {
    // Check if there is already an active timer.
    let active: Option<i64> = db.conn().query_row(
        "SELECT id FROM time_entries WHERE end_time IS NULL LIMIT 1",
        [],
        |row| row.get(0),
    ).ok();

    if active.is_some() {
        anyhow::bail!(
            "A timer is already running. Stop or cancel it first."
        );
    }

    let proj = find_or_create_project(db, project)?;
    let now = Local::now().naive_local();
    let uuid_val = uuid::Uuid::new_v4().to_string();
    let desc = description.unwrap_or("");
    let billable = !no_billable;

    db.conn().execute(
        "INSERT INTO time_entries (uuid, project_id, description, start_time, billable)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![uuid_val, proj.id, desc, now.format("%Y-%m-%dT%H:%M:%S").to_string(), billable as i32],
    ).context("Failed to start timer")?;

    let entry_id = db.conn().last_insert_rowid();
    attach_tags(db, entry_id, tags)?;

    println!(
        "{} Timer started for {} at {}",
        ">>>".green().bold(),
        proj.name.cyan().bold(),
        now.format("%H:%M:%S").to_string().white()
    );
    if !desc.is_empty() {
        println!("    {}", desc);
    }

    Ok(())
}

/// Stop the currently active timer.
pub fn handle_stop(db: &Database, description: Option<&str>) -> Result<()> {
    let now = Local::now().naive_local();
    let now_str = now.format("%Y-%m-%dT%H:%M:%S").to_string();

    // Find the active entry.
    let entry: (i64, i64, String, String) = db.conn().query_row(
        "SELECT id, project_id, description, start_time
         FROM time_entries WHERE end_time IS NULL LIMIT 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    ).context("No active timer to stop.")?;

    let (entry_id, project_id, existing_desc, start_str) = entry;
    let start_dt = NaiveDateTime::parse_from_str(&start_str, "%Y-%m-%dT%H:%M:%S")
        .context("Failed to parse start time")?;
    let duration_secs = (now - start_dt).num_seconds();

    let final_desc = description.unwrap_or(&existing_desc);

    db.conn().execute(
        "UPDATE time_entries SET end_time = ?1, duration_secs = ?2, description = ?3,
         updated_at = ?1 WHERE id = ?4",
        params![now_str, duration_secs, final_desc, entry_id],
    ).context("Failed to stop timer")?;

    let project = get_project_by_id(db, project_id)?;

    println!(
        "{} Timer stopped for {}",
        "|||".red().bold(),
        project.name.cyan().bold()
    );
    println!(
        "    Duration: {}",
        format_duration(duration_secs).yellow().bold()
    );
    if !final_desc.is_empty() {
        println!("    {}", final_desc);
    }

    Ok(())
}

/// Show the current timer status.
pub fn handle_status(db: &Database) -> Result<()> {
    let entry: Option<(i64, i64, String, String)> = db.conn().query_row(
        "SELECT id, project_id, description, start_time
         FROM time_entries WHERE end_time IS NULL LIMIT 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    ).ok();

    match entry {
        Some((_id, project_id, desc, start_str)) => {
            let project = get_project_by_id(db, project_id)?;
            let start_dt = NaiveDateTime::parse_from_str(&start_str, "%Y-%m-%dT%H:%M:%S")
                .context("Failed to parse start time")?;
            let now = Local::now().naive_local();
            let elapsed = (now - start_dt).num_seconds();

            println!("{}", "Timer running".green().bold());
            println!("  Project:     {}", project.name.cyan().bold());
            if !desc.is_empty() {
                println!("  Description: {}", desc);
            }
            println!(
                "  Started at:  {}",
                start_dt.format("%H:%M:%S").to_string().white()
            );
            println!(
                "  Elapsed:     {}",
                format_duration(elapsed).yellow().bold()
            );
        }
        None => {
            println!("{}", "No timer running.".dimmed());
        }
    }

    Ok(())
}

/// Cancel (discard) the active timer.
pub fn handle_cancel(db: &Database) -> Result<()> {
    let entry_id: i64 = db.conn().query_row(
        "SELECT id FROM time_entries WHERE end_time IS NULL LIMIT 1",
        [],
        |row| row.get(0),
    ).context("No active timer to cancel.")?;

    // Remove associated tags first.
    db.conn().execute(
        "DELETE FROM time_entry_tags WHERE entry_id = ?1",
        params![entry_id],
    )?;
    db.conn().execute(
        "DELETE FROM time_entries WHERE id = ?1",
        params![entry_id],
    ).context("Failed to cancel timer")?;

    println!(
        "{} Active timer cancelled and discarded.",
        "x".red().bold()
    );

    Ok(())
}

/// Resume the last stopped timer by starting a new one with the same project.
pub fn handle_resume(db: &Database) -> Result<()> {
    // Check no timer is currently running.
    let active: Option<i64> = db.conn().query_row(
        "SELECT id FROM time_entries WHERE end_time IS NULL LIMIT 1",
        [],
        |row| row.get(0),
    ).ok();

    if active.is_some() {
        anyhow::bail!("A timer is already running. Stop or cancel it first.");
    }

    // Get the most recent completed entry.
    let last: (i64, String, bool) = db.conn().query_row(
        "SELECT project_id, description, billable
         FROM time_entries WHERE end_time IS NOT NULL
         ORDER BY end_time DESC LIMIT 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get::<_, i32>(2)? != 0)),
    ).context("No previous timer to resume.")?;

    let (project_id, desc, billable) = last;
    let project = get_project_by_id(db, project_id)?;
    let now = Local::now().naive_local();
    let uuid_val = uuid::Uuid::new_v4().to_string();

    db.conn().execute(
        "INSERT INTO time_entries (uuid, project_id, description, start_time, billable)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            uuid_val,
            project_id,
            desc,
            now.format("%Y-%m-%dT%H:%M:%S").to_string(),
            billable as i32,
        ],
    ).context("Failed to resume timer")?;

    println!(
        "{} Timer resumed for {} at {}",
        ">>>".green().bold(),
        project.name.cyan().bold(),
        now.format("%H:%M:%S").to_string().white()
    );

    Ok(())
}

/// Add a completed time entry with explicit start/end times.
pub fn handle_add(
    db: &Database,
    project: &str,
    start: &str,
    end: &str,
    description: Option<&str>,
    tags: &[String],
) -> Result<()> {
    let start_dt = parse_time(start)?;
    let end_dt = parse_time(end)?;

    if end_dt <= start_dt {
        anyhow::bail!("End time must be after start time.");
    }

    let proj = find_or_create_project(db, project)?;
    let duration_secs = (end_dt - start_dt).num_seconds();
    let uuid_val = uuid::Uuid::new_v4().to_string();
    let desc = description.unwrap_or("");

    db.conn().execute(
        "INSERT INTO time_entries (uuid, project_id, description, start_time, end_time, duration_secs, billable)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1)",
        params![
            uuid_val,
            proj.id,
            desc,
            start_dt.format("%Y-%m-%dT%H:%M:%S").to_string(),
            end_dt.format("%Y-%m-%dT%H:%M:%S").to_string(),
            duration_secs,
        ],
    ).context("Failed to add time entry")?;

    let entry_id = db.conn().last_insert_rowid();
    attach_tags(db, entry_id, tags)?;

    println!(
        "{} Added {} for {}",
        "+".green().bold(),
        format_duration(duration_secs).yellow(),
        proj.name.cyan().bold()
    );

    Ok(())
}

/// List recent time entries in a table format.
pub fn handle_log(
    db: &Database,
    today: bool,
    week: bool,
    month: bool,
    project: Option<&str>,
    client: Option<&str>,
) -> Result<()> {
    let now = Local::now().naive_local();

    // Determine the date filter.
    let date_filter = if today {
        let start = now.date().and_hms_opt(0, 0, 0).unwrap();
        Some(start)
    } else if week {
        let days_since_monday = now.date().weekday().num_days_from_monday();
        let monday = now.date() - chrono::Duration::days(days_since_monday as i64);
        Some(monday.and_hms_opt(0, 0, 0).unwrap())
    } else if month {
        let first = NaiveDate::from_ymd_opt(now.date().year(), now.date().month(), 1).unwrap();
        Some(first.and_hms_opt(0, 0, 0).unwrap())
    } else {
        // Default: last 7 days
        let week_ago = now - chrono::Duration::days(7);
        Some(week_ago)
    };

    // Build the query dynamically.
    let mut sql = String::from(
        "SELECT te.start_time, te.end_time, te.duration_secs, te.description, p.name AS project_name
         FROM time_entries te
         JOIN projects p ON te.project_id = p.id"
    );

    let mut conditions: Vec<String> = Vec::new();
    let mut param_values: Vec<String> = Vec::new();

    if let Some(start) = date_filter {
        conditions.push(format!("te.start_time >= ?{}", param_values.len() + 1));
        param_values.push(start.format("%Y-%m-%dT%H:%M:%S").to_string());
    }

    if let Some(proj_name) = project {
        conditions.push(format!("p.name = ?{} COLLATE NOCASE", param_values.len() + 1));
        param_values.push(proj_name.to_string());
    }

    if let Some(client_name) = client {
        sql.push_str(" LEFT JOIN clients c ON p.client_id = c.id");
        conditions.push(format!("c.name = ?{} COLLATE NOCASE", param_values.len() + 1));
        param_values.push(client_name.to_string());
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(" ORDER BY te.start_time DESC LIMIT 50");

    let mut stmt = db.conn().prepare(&sql).context("Failed to prepare log query")?;

    // Build parameter references for rusqlite.
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = param_values
        .iter()
        .map(|v| v as &dyn rusqlite::types::ToSql)
        .collect();

    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        Ok((
            row.get::<_, String>(0)?,          // start_time
            row.get::<_, Option<String>>(1)?,   // end_time
            row.get::<_, Option<i64>>(2)?,      // duration_secs
            row.get::<_, String>(3)?,           // description
            row.get::<_, String>(4)?,           // project_name
        ))
    }).context("Failed to execute log query")?;

    let entries: Vec<(String, Option<String>, Option<i64>, String, String)> = rows
        .filter_map(|r| r.ok())
        .collect();

    if entries.is_empty() {
        println!("{}", "No entries found.".dimmed());
        return Ok(());
    }

    // Print table header.
    println!(
        "{:<12} {:<8} {:<8} {:<12} {:<16} {}",
        "Date".bold().underline(),
        "Start".bold().underline(),
        "End".bold().underline(),
        "Duration".bold().underline(),
        "Project".bold().underline(),
        "Description".bold().underline(),
    );

    for (start_str, end_str, dur_secs, desc, proj_name) in &entries {
        let start_dt = NaiveDateTime::parse_from_str(start_str, "%Y-%m-%dT%H:%M:%S")
            .unwrap_or_default();
        let date_str = start_dt.format("%Y-%m-%d").to_string();
        let start_time = start_dt.format("%H:%M").to_string();

        let end_time = match end_str {
            Some(e) => {
                NaiveDateTime::parse_from_str(e, "%Y-%m-%dT%H:%M:%S")
                    .map(|dt| dt.format("%H:%M").to_string())
                    .unwrap_or_else(|_| "??:??".to_string())
            }
            None => "running".yellow().to_string(),
        };

        let duration = match dur_secs {
            Some(s) => format_duration(*s),
            None => {
                // Still running, compute elapsed.
                let now = Local::now().naive_local();
                let elapsed = (now - start_dt).num_seconds();
                format!("~{}", format_duration(elapsed))
            }
        };

        println!(
            "{:<12} {:<8} {:<8} {:<12} {:<16} {}",
            date_str, start_time, end_time, duration, proj_name.cyan(), desc
        );
    }

    // Print total.
    let total_secs: i64 = entries.iter().filter_map(|(_, _, d, _, _)| *d).sum();
    println!(
        "\n{} {}",
        "Total:".bold(),
        format_duration(total_secs).yellow().bold()
    );

    Ok(())
}

/// Dispatch project subcommands.
pub fn handle_projects(db: &Database, action: ProjectAction) -> Result<()> {
    match action {
        ProjectAction::List { all } => {
            let status_filter = if all { "" } else { " WHERE status = 'active'" };
            let sql = format!(
                "SELECT id, name, client_id, color, status, budget_hours, notes, created_at, updated_at
                 FROM projects{} ORDER BY name",
                status_filter
            );
            let mut stmt = db.conn().prepare(&sql)?;
            let projects: Vec<Project> = stmt
                .query_map([], |row| {
                    Ok(Project {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        client_id: row.get(2)?,
                        color: row.get(3)?,
                        status: ProjectStatus::from_str(&row.get::<_, String>(4)?).unwrap_or(ProjectStatus::Active),
                        budget_hours: row.get(5)?,
                        notes: row.get(6)?,
                        created_at: row.get::<_, String>(7)?.parse::<NaiveDateTime>().unwrap(),
                        updated_at: row.get::<_, String>(8)?.parse::<NaiveDateTime>().unwrap(),
                    })
                })?
                .filter_map(|r| r.ok())
                .collect();

            if projects.is_empty() {
                println!("{}", "No projects found.".dimmed());
                return Ok(());
            }

            println!(
                "{:<20} {:<10} {:<10} {}",
                "Name".bold().underline(),
                "Status".bold().underline(),
                "Color".bold().underline(),
                "Budget".bold().underline(),
            );

            for p in &projects {
                let budget = p.budget_hours.map(|b| format!("{:.1}h", b)).unwrap_or_default();
                let status_str = match p.status {
                    ProjectStatus::Active => "active".green().to_string(),
                    ProjectStatus::Archived => "archived".dimmed().to_string(),
                };
                println!("{:<20} {:<10} {:<10} {}", p.name.cyan(), status_str, p.color, budget);
            }
        }

        ProjectAction::Add { name, client, color, budget } => {
            let client_id: Option<i64> = if let Some(ref client_name) = client {
                let c = find_client_by_name(db, client_name)?
                    .ok_or_else(|| anyhow::anyhow!("Client '{}' not found.", client_name))?;
                Some(c.id)
            } else {
                None
            };

            let color_val = color.unwrap_or_else(|| "#5B9BD5".to_string());
            db.conn().execute(
                "INSERT INTO projects (name, client_id, color, budget_hours) VALUES (?1, ?2, ?3, ?4)",
                params![name, client_id, color_val, budget],
            ).context("Failed to create project")?;

            println!("{} Project '{}' created.", "+".green().bold(), name.cyan());
        }

        ProjectAction::Edit { name, new_name, color, budget } => {
            let project = find_or_create_project(db, &name)?;

            let final_name = new_name.as_deref().unwrap_or(&project.name);
            let final_color = color.as_deref().unwrap_or(&project.color);
            let final_budget = budget.or(project.budget_hours);

            db.conn().execute(
                "UPDATE projects SET name = ?1, color = ?2, budget_hours = ?3,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%S','now')
                 WHERE id = ?4",
                params![final_name, final_color, final_budget, project.id],
            ).context("Failed to update project")?;

            println!("{} Project '{}' updated.", "*".yellow().bold(), final_name.cyan());
        }

        ProjectAction::Archive { name } => {
            db.conn().execute(
                "UPDATE projects SET status = 'archived', updated_at = strftime('%Y-%m-%dT%H:%M:%S','now')
                 WHERE name = ?1 COLLATE NOCASE",
                params![name],
            ).context("Failed to archive project")?;
            println!("{} Project '{}' archived.", "-".dimmed(), name);
        }

        ProjectAction::Activate { name } => {
            db.conn().execute(
                "UPDATE projects SET status = 'active', updated_at = strftime('%Y-%m-%dT%H:%M:%S','now')
                 WHERE name = ?1 COLLATE NOCASE",
                params![name],
            ).context("Failed to activate project")?;
            println!("{} Project '{}' activated.", "+".green().bold(), name.cyan());
        }
    }

    Ok(())
}

/// Dispatch client subcommands.
pub fn handle_clients(db: &Database, action: ClientAction) -> Result<()> {
    match action {
        ClientAction::List => {
            let clients = db.list_clients()?;
            if clients.is_empty() {
                println!("{}", "No clients found.".dimmed());
                return Ok(());
            }

            println!(
                "{:<20} {:<25} {}",
                "Name".bold().underline(),
                "Contact".bold().underline(),
                "Email".bold().underline(),
            );

            for c in &clients {
                println!(
                    "{:<20} {:<25} {}",
                    c.name.cyan(),
                    c.contact.as_deref().unwrap_or(""),
                    c.email.as_deref().unwrap_or(""),
                );
            }
        }

        ClientAction::Add { name, contact, email } => {
            db.create_client(&name, contact.as_deref(), email.as_deref(), None)?;
            println!("{} Client '{}' created.", "+".green().bold(), name.cyan());
        }

        ClientAction::Edit { name, new_name, contact, email } => {
            let client = find_client_by_name(db, &name)?
                .ok_or_else(|| anyhow::anyhow!("Client '{}' not found.", name))?;

            db.update_client(
                client.id,
                new_name.as_deref(),
                contact.as_deref(),
                email.as_deref(),
                None,
            )?;

            let display_name = new_name.as_deref().unwrap_or(&name);
            println!("{} Client '{}' updated.", "*".yellow().bold(), display_name.cyan());
        }

        ClientAction::Delete { name } => {
            let client = find_client_by_name(db, &name)?
                .ok_or_else(|| anyhow::anyhow!("Client '{}' not found.", name))?;

            db.delete_client(client.id)?;
            println!("{} Client '{}' deleted.", "x".red().bold(), name);
        }
    }

    Ok(())
}

/// Dispatch tag subcommands.
pub fn handle_tags(db: &Database, action: TagAction) -> Result<()> {
    match action {
        TagAction::List => {
            let mut stmt = db.conn().prepare(
                "SELECT id, name, color FROM tags ORDER BY name",
            )?;
            let tags: Vec<Tag> = stmt
                .query_map([], |row| {
                    Ok(Tag {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        color: row.get(2)?,
                    })
                })?
                .filter_map(|r| r.ok())
                .collect();

            if tags.is_empty() {
                println!("{}", "No tags found.".dimmed());
                return Ok(());
            }

            println!(
                "{:<20} {}",
                "Tag".bold().underline(),
                "Color".bold().underline(),
            );

            for t in &tags {
                println!("{:<20} {}", t.name.cyan(), t.color);
            }
        }

        TagAction::Add { name, color } => {
            let color_val = color.unwrap_or_else(|| "#808080".to_string());
            db.conn().execute(
                "INSERT INTO tags (name, color) VALUES (?1, ?2)",
                params![name, color_val],
            ).context("Failed to add tag")?;
            println!("{} Tag '{}' created.", "+".green().bold(), name.cyan());
        }

        TagAction::Delete { name } => {
            let affected = db.conn().execute(
                "DELETE FROM tags WHERE name = ?1 COLLATE NOCASE",
                params![name],
            ).context("Failed to delete tag")?;

            if affected == 0 {
                anyhow::bail!("Tag '{}' not found.", name);
            }
            println!("{} Tag '{}' deleted.", "x".red().bold(), name);
        }
    }

    Ok(())
}

/// Dispatch rate subcommands.
pub fn handle_rates(db: &Database, action: RateAction) -> Result<()> {
    match action {
        RateAction::Set { rate, project, client, currency } => {
            let (rate_type, project_id, client_id) = if let Some(ref proj_name) = project {
                let p = find_or_create_project(db, proj_name)?;
                ("project", Some(p.id), None::<i64>)
            } else if let Some(ref client_name) = client {
                let c = find_client_by_name(db, client_name)?
                    .ok_or_else(|| anyhow::anyhow!("Client '{}' not found.", client_name))?;
                ("client", None::<i64>, Some(c.id))
            } else {
                ("default", None::<i64>, None::<i64>)
            };

            db.conn().execute(
                "INSERT INTO rates (rate_type, project_id, client_id, hourly_rate, currency)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![rate_type, project_id, client_id, rate, currency],
            ).context("Failed to set rate")?;

            let scope = project
                .as_deref()
                .or(client.as_deref())
                .unwrap_or("default");

            println!(
                "{} Rate set: {}{}/hr for {}",
                "+".green().bold(),
                currency,
                rate,
                scope.cyan()
            );
        }

        RateAction::List => {
            let mut stmt = db.conn().prepare(
                "SELECT r.id, r.rate_type, r.project_id, r.client_id, r.hourly_rate, r.currency,
                        r.effective_from, r.created_at,
                        p.name AS project_name, c.name AS client_name
                 FROM rates r
                 LEFT JOIN projects p ON r.project_id = p.id
                 LEFT JOIN clients c ON r.client_id = c.id
                 ORDER BY r.rate_type, r.created_at DESC",
            )?;

            let rates: Vec<(String, f64, String, Option<String>, Option<String>)> = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(1)?,   // rate_type
                        row.get::<_, f64>(4)?,      // hourly_rate
                        row.get::<_, String>(5)?,   // currency
                        row.get::<_, Option<String>>(8)?,  // project_name
                        row.get::<_, Option<String>>(9)?,  // client_name
                    ))
                })?
                .filter_map(|r| r.ok())
                .collect();

            if rates.is_empty() {
                println!("{}", "No rates configured.".dimmed());
                return Ok(());
            }

            println!(
                "{:<10} {:<12} {:<10} {}",
                "Type".bold().underline(),
                "Rate".bold().underline(),
                "Currency".bold().underline(),
                "Scope".bold().underline(),
            );

            for (rtype, hourly, currency, proj_name, client_name) in &rates {
                let scope = proj_name
                    .as_deref()
                    .or(client_name.as_deref())
                    .unwrap_or("(global)");
                println!(
                    "{:<10} {:<12.2} {:<10} {}",
                    rtype, hourly, currency, scope.cyan()
                );
            }
        }
    }

    Ok(())
}

/// Handle configuration subcommands.
pub fn handle_config(action: ConfigAction) -> Result<()> {
    match action {
        ConfigAction::Show => {
            let config = SagaConfig::load()?;
            println!("{}", "Current configuration:".bold().underline());
            println!("  default_currency:       {}", config.default_currency);
            println!(
                "  default_hourly_rate:    {}",
                config
                    .default_hourly_rate
                    .map(|r| format!("{:.2}", r))
                    .unwrap_or_else(|| "(not set)".to_string())
            );
            println!("  default_billable:       {}", config.default_billable);
            println!(
                "  daily_goal_hours:       {}",
                config
                    .daily_goal_hours
                    .map(|h| format!("{:.1}", h))
                    .unwrap_or_else(|| "(not set)".to_string())
            );
            println!(
                "  weekly_goal_hours:      {}",
                config
                    .weekly_goal_hours
                    .map(|h| format!("{:.1}", h))
                    .unwrap_or_else(|| "(not set)".to_string())
            );
            println!("  tick_rate_ms:           {}", config.tick_rate_ms);
            println!("  theme:                  {}", config.theme);
            println!("  date_format:            {}", config.date_format);
            println!("  time_format:            {}", config.time_format);
            println!("  reminder_interval_mins: {}", config.reminder_interval_mins);
        }

        ConfigAction::Set { key, value } => {
            let mut config = SagaConfig::load()?;
            match key.as_str() {
                "default_currency" => config.default_currency = value.clone(),
                "default_hourly_rate" => {
                    config.default_hourly_rate = Some(
                        value.parse::<f64>().context("Invalid number for default_hourly_rate")?,
                    );
                }
                "default_billable" => {
                    config.default_billable = value
                        .parse::<bool>()
                        .context("Invalid boolean for default_billable (use true/false)")?;
                }
                "daily_goal_hours" => {
                    config.daily_goal_hours = Some(
                        value.parse::<f64>().context("Invalid number for daily_goal_hours")?,
                    );
                }
                "weekly_goal_hours" => {
                    config.weekly_goal_hours = Some(
                        value.parse::<f64>().context("Invalid number for weekly_goal_hours")?,
                    );
                }
                "tick_rate_ms" => {
                    config.tick_rate_ms = value
                        .parse::<u64>()
                        .context("Invalid number for tick_rate_ms")?;
                }
                "theme" => config.theme = value.clone(),
                "date_format" => config.date_format = value.clone(),
                "time_format" => config.time_format = value.clone(),
                "reminder_interval_mins" => {
                    config.reminder_interval_mins = value
                        .parse::<u64>()
                        .context("Invalid number for reminder_interval_mins")?;
                }
                _ => anyhow::bail!("Unknown config key: '{}'", key),
            }
            config.save()?;
            println!(
                "{} Config '{}' set to '{}'.",
                "*".yellow().bold(),
                key.cyan(),
                value
            );
        }

        ConfigAction::Path => {
            let path = SagaConfig::config_path()?;
            println!("{}", path.display());
        }
    }

    Ok(())
}

/// Generate reports (table, csv, or pdf).
pub fn handle_report(
    db: &Database,
    period: &str,
    format: &str,
    output: Option<&str>,
) -> Result<()> {
    let now = Local::now().naive_local();

    // Determine date range based on period.
    let (start_date, end_date) = match period {
        "daily" => {
            let start = now.date().and_hms_opt(0, 0, 0).unwrap();
            let end = now;
            (start, end)
        }
        "weekly" => {
            let days_since_monday = now.date().weekday().num_days_from_monday();
            let monday = now.date() - chrono::Duration::days(days_since_monday as i64);
            let start = monday.and_hms_opt(0, 0, 0).unwrap();
            (start, now)
        }
        "monthly" => {
            let first = NaiveDate::from_ymd_opt(now.date().year(), now.date().month(), 1).unwrap();
            let start = first.and_hms_opt(0, 0, 0).unwrap();
            (start, now)
        }
        _ => anyhow::bail!("Unknown period '{}'. Use daily, weekly, or monthly.", period),
    };

    let start_str = start_date.format("%Y-%m-%dT%H:%M:%S").to_string();
    let end_str = end_date.format("%Y-%m-%dT%H:%M:%S").to_string();

    // Query entries grouped by project.
    let mut stmt = db.conn().prepare(
        "SELECT p.name, SUM(te.duration_secs), COUNT(*),
                SUM(CASE WHEN te.billable THEN te.duration_secs ELSE 0 END)
         FROM time_entries te
         JOIN projects p ON te.project_id = p.id
         WHERE te.start_time >= ?1 AND te.start_time <= ?2 AND te.end_time IS NOT NULL
         GROUP BY p.name
         ORDER BY SUM(te.duration_secs) DESC",
    )?;

    let rows: Vec<(String, i64, i64, i64)> = stmt
        .query_map(params![start_str, end_str], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();

    if rows.is_empty() {
        println!("{}", "No entries found for the selected period.".dimmed());
        return Ok(());
    }

    match format {
        "table" => {
            println!(
                "{} report ({} to {})\n",
                period.bold(),
                start_date.format("%Y-%m-%d").to_string().white(),
                end_date.format("%Y-%m-%d").to_string().white(),
            );
            println!(
                "{:<20} {:<12} {:<8} {}",
                "Project".bold().underline(),
                "Total".bold().underline(),
                "Entries".bold().underline(),
                "Billable".bold().underline(),
            );

            let mut grand_total: i64 = 0;
            let mut grand_billable: i64 = 0;

            for (project_name, total_secs, count, billable_secs) in &rows {
                grand_total += total_secs;
                grand_billable += billable_secs;
                println!(
                    "{:<20} {:<12} {:<8} {}",
                    project_name.cyan(),
                    format_duration(*total_secs).yellow(),
                    count,
                    format_duration(*billable_secs),
                );
            }

            println!(
                "\n{:<20} {:<12} {:<8} {}",
                "TOTAL".bold(),
                format_duration(grand_total).yellow().bold(),
                "",
                format_duration(grand_billable).bold(),
            );
        }

        "csv" => {
            let mut csv_output = String::new();
            csv_output.push_str("Project,Total Seconds,Total Duration,Entries,Billable Seconds\n");
            for (project_name, total_secs, count, billable_secs) in &rows {
                csv_output.push_str(&format!(
                    "{},{},{},{},{}\n",
                    project_name, total_secs, format_duration(*total_secs), count, billable_secs
                ));
            }

            if let Some(path) = output {
                std::fs::write(path, &csv_output).context("Failed to write CSV file")?;
                println!("{} Report saved to {}", "+".green().bold(), path.cyan());
            } else {
                print!("{}", csv_output);
            }
        }

        "pdf" => {
            println!(
                "{}",
                "PDF report generation requires the export module. Use --format csv or table."
                    .yellow()
            );
        }

        _ => {
            anyhow::bail!("Unknown format '{}'. Use table, csv, or pdf.", format);
        }
    }

    Ok(())
}

pub fn handle_invoice(db: &Database, action: InvoiceAction) -> Result<()> {
    match action {
        InvoiceAction::Generate { client, from, to, output } => {
            let client_record = find_client_by_name(db, &client)?
                .ok_or_else(|| anyhow::anyhow!("Client '{}' not found.", client))?;

            // Get entries for the client's projects in the date range
            let mut stmt = db.conn().prepare(
                "SELECT te.id, te.uuid, te.project_id, te.description, te.start_time,
                        te.end_time, te.duration_secs, te.billable, te.created_at, te.updated_at,
                        p.id, p.name, p.client_id, p.color, p.status, p.budget_hours, p.notes,
                        p.created_at, p.updated_at
                 FROM time_entries te
                 JOIN projects p ON te.project_id = p.id
                 WHERE p.client_id = ?1
                   AND te.start_time >= ?2
                   AND te.start_time <= ?3
                   AND te.end_time IS NOT NULL
                 ORDER BY te.start_time",
            )?;

            let entries: Vec<(TimeEntry, Project)> = stmt
                .query_map(
                    params![client_record.id, format!("{}T00:00:00", from), format!("{}T23:59:59", to)],
                    |row| {
                        let entry = TimeEntry {
                            id: row.get(0)?,
                            uuid: row.get(1)?,
                            project_id: row.get(2)?,
                            description: row.get(3)?,
                            start_time: row.get::<_, String>(4)?.parse::<NaiveDateTime>().unwrap(),
                            end_time: row.get::<_, Option<String>>(5)?.map(|s| s.parse::<NaiveDateTime>().unwrap()),
                            duration_secs: row.get(6)?,
                            billable: row.get::<_, i32>(7)? != 0,
                            created_at: row.get::<_, String>(8)?.parse::<NaiveDateTime>().unwrap(),
                            updated_at: row.get::<_, String>(9)?.parse::<NaiveDateTime>().unwrap(),
                        };
                        let project = Project {
                            id: row.get(10)?,
                            name: row.get(11)?,
                            client_id: row.get(12)?,
                            color: row.get(13)?,
                            status: ProjectStatus::from_str(&row.get::<_, String>(14)?).unwrap_or(ProjectStatus::Active),
                            budget_hours: row.get(15)?,
                            notes: row.get(16)?,
                            created_at: row.get::<_, String>(17)?.parse::<NaiveDateTime>().unwrap(),
                            updated_at: row.get::<_, String>(18)?.parse::<NaiveDateTime>().unwrap(),
                        };
                        Ok((entry, project))
                    },
                )?
                .filter_map(|r| r.ok())
                .collect();

            if entries.is_empty() {
                anyhow::bail!("No billable entries found for client '{}' in the specified period.", client);
            }

            // Get the effective rate
            let rate = db.conn().query_row(
                "SELECT hourly_rate, currency FROM rates
                 WHERE (rate_type = 'client' AND client_id = ?1)
                    OR rate_type = 'default'
                 ORDER BY CASE rate_type WHEN 'client' THEN 0 ELSE 1 END
                 LIMIT 1",
                params![client_record.id],
                |row| Ok((row.get::<_, f64>(0)?, row.get::<_, String>(1)?)),
            ).unwrap_or((0.0, "USD".to_string()));

            // Generate invoice number
            let invoice_count: i64 = db.conn().query_row(
                "SELECT COUNT(*) FROM invoices",
                [],
                |row| row.get(0),
            ).unwrap_or(0);
            let invoice_number = format!("INV-{:04}", invoice_count + 1);

            let total_hours: f64 = entries.iter()
                .filter(|(e, _)| e.billable)
                .map(|(e, _)| e.duration_secs.unwrap_or(0) as f64 / 3600.0)
                .sum();
            let total_amount = total_hours * rate.0;

            // Save invoice record
            db.conn().execute(
                "INSERT INTO invoices (invoice_number, client_id, period_start, period_end,
                                       total_hours, total_amount, currency)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![invoice_number, client_record.id, from, to, total_hours, total_amount, rate.1],
            )?;

            let invoice_data = crate::export::invoice::InvoiceData {
                invoice_number: invoice_number.clone(),
                client: client_record,
                entries,
                hourly_rate: rate.0,
                currency: rate.1,
                period_start: from,
                period_end: to,
                notes: None,
            };

            match crate::export::invoice::generate_invoice(&invoice_data) {
                Ok(filename) => {
                    let final_path = if let Some(out) = output {
                        std::fs::rename(&filename, &out).ok();
                        out.to_string()
                    } else {
                        filename
                    };
                    println!(
                        "{} Invoice {} generated: {}",
                        "+".green().bold(),
                        invoice_number.cyan(),
                        final_path
                    );
                }
                Err(e) => {
                    println!(
                        "{} Invoice {} recorded but PDF generation failed: {}",
                        "!".yellow().bold(),
                        invoice_number.cyan(),
                        e
                    );
                }
            }
        }

        InvoiceAction::List => {
            let mut stmt = db.conn().prepare(
                "SELECT i.invoice_number, c.name, i.period_start, i.period_end,
                        i.total_hours, i.total_amount, i.currency, i.status
                 FROM invoices i
                 JOIN clients c ON i.client_id = c.id
                 ORDER BY i.generated_at DESC",
            )?;

            let invoices: Vec<(String, String, String, String, f64, f64, String, String)> = stmt
                .query_map([], |row| {
                    Ok((
                        row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?,
                        row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?,
                    ))
                })?
                .filter_map(|r| r.ok())
                .collect();

            if invoices.is_empty() {
                println!("{}", "No invoices found.".dimmed());
                return Ok(());
            }

            println!(
                "{:<12} {:<16} {:<24} {:<12} {:<10}",
                "Invoice #".bold().underline(),
                "Client".bold().underline(),
                "Period".bold().underline(),
                "Amount".bold().underline(),
                "Status".bold().underline(),
            );

            for (num, client_name, start, end, _hours, amount, currency, status) in &invoices {
                println!(
                    "{:<12} {:<16} {:<24} {:<12} {:<10}",
                    num.cyan(),
                    client_name,
                    format!("{} - {}", start, end),
                    format!("{}{:.2}", currency, amount),
                    status,
                );
            }
        }
    }

    Ok(())
}
