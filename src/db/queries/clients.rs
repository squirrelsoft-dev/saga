use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use rusqlite::params;

use crate::db::Database;
use crate::models::Client;

impl Database {
    /// Create a new client.
    pub fn create_client(
        &self,
        name: &str,
        contact: Option<&str>,
        email: Option<&str>,
        notes: Option<&str>,
    ) -> Result<Client> {
        self.conn.execute(
            "INSERT INTO clients (name, contact, email, notes) VALUES (?1, ?2, ?3, ?4)",
            params![name, contact, email, notes],
        ).context("Failed to insert client")?;

        let id = self.conn.last_insert_rowid();
        self.get_client(id)
    }

    /// Get a client by id.
    pub fn get_client(&self, id: i64) -> Result<Client> {
        let client = self.conn.query_row(
            "SELECT id, name, contact, email, notes, created_at, updated_at
             FROM clients WHERE id = ?1",
            params![id],
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
        ).context("Failed to get client")?;

        Ok(client)
    }

    /// List all clients.
    pub fn list_clients(&self) -> Result<Vec<Client>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, contact, email, notes, created_at, updated_at
             FROM clients ORDER BY name",
        ).context("Failed to prepare list_clients query")?;

        let clients = stmt.query_map([], |row| {
            Ok(Client {
                id: row.get(0)?,
                name: row.get(1)?,
                contact: row.get(2)?,
                email: row.get(3)?,
                notes: row.get(4)?,
                created_at: row.get::<_, String>(5)?.parse::<NaiveDateTime>().unwrap(),
                updated_at: row.get::<_, String>(6)?.parse::<NaiveDateTime>().unwrap(),
            })
        }).context("Failed to execute list_clients query")?;

        let mut result = Vec::new();
        for client in clients {
            result.push(client.context("Failed to read client row")?);
        }
        Ok(result)
    }

    /// Update a client's fields.
    pub fn update_client(
        &self,
        id: i64,
        name: Option<&str>,
        contact: Option<&str>,
        email: Option<&str>,
        notes: Option<&str>,
    ) -> Result<Client> {
        // Get the existing client first so we can preserve unchanged fields.
        let existing = self.get_client(id)?;

        let new_name = name.unwrap_or(&existing.name);
        let new_contact = contact.or(existing.contact.as_deref());
        let new_email = email.or(existing.email.as_deref());
        let new_notes = notes.or(existing.notes.as_deref());

        self.conn.execute(
            "UPDATE clients SET name = ?1, contact = ?2, email = ?3, notes = ?4,
             updated_at = strftime('%Y-%m-%dT%H:%M:%S','now')
             WHERE id = ?5",
            params![new_name, new_contact, new_email, new_notes, id],
        ).context("Failed to update client")?;

        self.get_client(id)
    }

    /// Delete a client by id. Will fail if the client has associated invoices
    /// (due to ON DELETE RESTRICT).
    pub fn delete_client(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM clients WHERE id = ?1", params![id])
            .context("Failed to delete client")?;
        Ok(())
    }
}
