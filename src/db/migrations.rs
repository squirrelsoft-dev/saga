use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::db::schema;

/// Run all pending migrations on the database.
pub fn run_migrations(conn: &Connection) -> Result<()> {
    // Ensure the schema_version table exists.
    conn.execute_batch(schema::CREATE_SCHEMA_VERSION)
        .context("Failed to create schema_version table")?;

    let current_version = get_current_version(conn)?;

    let migrations: Vec<fn(&Connection) -> Result<()>> = vec![
        migrate_v1,
    ];

    for (i, migration) in migrations.iter().enumerate() {
        let version = (i + 1) as i64;
        if version > current_version {
            migration(conn)
                .with_context(|| format!("Failed to run migration v{}", version))?;
            set_version(conn, version)?;
        }
    }

    Ok(())
}

/// Get the current schema version, or 0 if none is set.
fn get_current_version(conn: &Connection) -> Result<i64> {
    let version: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .context("Failed to read schema version")?;
    Ok(version)
}

/// Set the schema version after a successful migration.
fn set_version(conn: &Connection, version: i64) -> Result<()> {
    conn.execute("DELETE FROM schema_version", [])
        .context("Failed to clear schema_version")?;
    conn.execute("INSERT INTO schema_version (version) VALUES (?1)", [version])
        .context("Failed to set schema version")?;
    Ok(())
}

/// Migration v1: Create the initial schema with all tables.
fn migrate_v1(conn: &Connection) -> Result<()> {
    conn.execute_batch(schema::CREATE_CLIENTS)
        .context("Failed to create clients table")?;
    conn.execute_batch(schema::CREATE_PROJECTS)
        .context("Failed to create projects table")?;
    conn.execute_batch(schema::CREATE_TAGS)
        .context("Failed to create tags table")?;
    conn.execute_batch(schema::CREATE_TIME_ENTRIES)
        .context("Failed to create time_entries table")?;
    conn.execute_batch(schema::CREATE_TIME_ENTRIES_INDEXES)
        .context("Failed to create time_entries indexes")?;
    conn.execute_batch(schema::CREATE_TIME_ENTRY_TAGS)
        .context("Failed to create time_entry_tags table")?;
    conn.execute_batch(schema::CREATE_RATES)
        .context("Failed to create rates table")?;
    conn.execute_batch(schema::CREATE_INVOICES)
        .context("Failed to create invoices table")?;
    Ok(())
}
