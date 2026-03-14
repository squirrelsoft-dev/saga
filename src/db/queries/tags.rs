use anyhow::{Context, Result};
use rusqlite::params;

use crate::db::Database;
use crate::models::Tag;

impl Database {
    /// Create a new tag.
    pub fn create_tag(&self, name: &str, color: Option<&str>) -> Result<Tag> {
        let color = color.unwrap_or("#808080");

        self.conn.execute(
            "INSERT INTO tags (name, color) VALUES (?1, ?2)",
            params![name, color],
        ).context("Failed to insert tag")?;

        let id = self.conn.last_insert_rowid();
        Ok(Tag {
            id,
            name: name.to_string(),
            color: color.to_string(),
        })
    }

    /// List all tags.
    pub fn list_tags(&self) -> Result<Vec<Tag>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, color FROM tags ORDER BY name",
        ).context("Failed to prepare list_tags query")?;

        let tags = stmt.query_map([], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
            })
        }).context("Failed to execute list_tags query")?;

        let mut result = Vec::new();
        for tag in tags {
            result.push(tag.context("Failed to read tag row")?);
        }
        Ok(result)
    }

    /// Delete a tag by id.
    pub fn delete_tag(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM tags WHERE id = ?1", params![id])
            .context("Failed to delete tag")?;
        Ok(())
    }

    /// Get an existing tag by name, or create it with the default color if it
    /// does not exist.
    pub fn get_or_create_tag(&self, name: &str) -> Result<Tag> {
        let existing = self.conn.query_row(
            "SELECT id, name, color FROM tags WHERE name = ?1",
            params![name],
            |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color: row.get(2)?,
                })
            },
        );

        match existing {
            Ok(tag) => Ok(tag),
            Err(rusqlite::Error::QueryReturnedNoRows) => self.create_tag(name, None),
            Err(e) => Err(e).context("Failed to query tag by name"),
        }
    }
}
