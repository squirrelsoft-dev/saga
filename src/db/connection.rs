use std::path::Path;

use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::db::migrations;

/// Open (or create) a SQLite database at the given path, configure PRAGMAs,
/// and run any pending schema migrations.
pub fn open(path: &Path) -> Result<Connection> {
    // Ensure the parent directory exists.
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create database directory: {}", parent.display()))?;
    }

    let conn = Connection::open(path)
        .with_context(|| format!("Failed to open database at {}", path.display()))?;

    configure_pragmas(&conn)?;
    migrations::run_migrations(&conn)?;

    Ok(conn)
}

/// Open an in-memory SQLite database, configure PRAGMAs, and run migrations.
pub fn open_in_memory() -> Result<Connection> {
    let conn = Connection::open_in_memory()
        .context("Failed to open in-memory database")?;

    configure_pragmas(&conn)?;
    migrations::run_migrations(&conn)?;

    Ok(conn)
}

/// Set recommended PRAGMAs for performance and correctness.
fn configure_pragmas(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA foreign_keys = ON;
         PRAGMA busy_timeout = 5000;",
    )
    .context("Failed to set database PRAGMAs")?;
    Ok(())
}
