use std::sync::Mutex;

use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::Item;
use crate::storage::Storage;

pub struct SqliteStorage {
    conn: Mutex<Connection>,
}

impl SqliteStorage {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path).context("failed to open SQLite database")?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS items (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                \"from\" TEXT NOT NULL,
                body TEXT NOT NULL,
                source TEXT NOT NULL,
                urgent INTEGER NOT NULL DEFAULT 0
            );",
        )
        .context("failed to create items table")?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

impl Storage for SqliteStorage {
    fn store_items(&mut self, items: &[Item]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM items", [])
            .context("failed to clear items")?;
        let mut stmt = conn
            .prepare(
                "INSERT INTO items (id, title, \"from\", body, source, urgent)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .context("failed to prepare insert")?;
        for item in items {
            stmt.execute(rusqlite::params![
                item.id,
                item.title,
                item.from,
                item.body,
                item.source,
                item.urgent as i32,
            ])
            .context("failed to insert item")?;
        }
        Ok(())
    }

    fn get_items(&self) -> Result<Vec<Item>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, title, \"from\", body, source, urgent FROM items")
            .context("failed to prepare select")?;
        let items = stmt
            .query_map([], |row| {
                Ok(Item {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    from: row.get(2)?,
                    body: row.get(3)?,
                    source: row.get(4)?,
                    urgent: row.get::<_, i32>(5)? != 0,
                })
            })
            .context("failed to query items")?;
        let mut result = Vec::new();
        for item in items {
            result.push(item.context("failed to read item row")?);
        }
        Ok(result)
    }

    fn clear(&mut self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM items", [])
            .context("failed to clear items")?;
        Ok(())
    }
}
