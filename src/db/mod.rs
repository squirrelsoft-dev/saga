pub mod connection;
pub mod migrations;
pub mod queries;
pub mod schema;

use std::path::Path;

use anyhow::Result;
use rusqlite::Connection;

/// The main database handle wrapping a rusqlite Connection.
pub struct Database {
    pub conn: Connection,
}

impl Database {
    /// Open (or create) a database at the given file path.
    /// Configures PRAGMAs and runs any pending migrations.
    pub fn open(path: &Path) -> Result<Self> {
        let conn = connection::open(path)?;
        Ok(Database { conn })
    }

    /// Open an in-memory database for testing.
    /// Configures PRAGMAs and runs migrations.
    pub fn open_in_memory() -> Result<Self> {
        let conn = connection::open_in_memory()?;
        Ok(Database { conn })
    }

    /// Get a reference to the underlying Connection, for advanced usage.
    pub fn conn(&self) -> &Connection {
        &self.conn
    }
}
