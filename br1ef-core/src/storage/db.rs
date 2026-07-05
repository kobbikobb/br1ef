use std::sync::Mutex;

use anyhow::{Context, Result};

#[cfg(test)]
#[path = "db_test.rs"]
mod tests;
use rusqlite::Connection;

use crate::storage::Storage;
use crate::{Digest, Item};

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
            );
            CREATE TABLE IF NOT EXISTS digests (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                total_items INTEGER NOT NULL,
                generated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS digest_sources (
                digest_id INTEGER NOT NULL,
                source TEXT NOT NULL,
                item_count INTEGER NOT NULL,
                FOREIGN KEY (digest_id) REFERENCES digests(id)
            );
            CREATE TABLE IF NOT EXISTS selected_mailboxes (
                name TEXT PRIMARY KEY
            );",
        )
        .context("failed to create tables")?;

        // migration: add summary column if missing
        let _ =
            conn.execute_batch("ALTER TABLE digests ADD COLUMN summary TEXT NOT NULL DEFAULT ''");

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

impl Storage for SqliteStorage {
    fn store_items(&mut self, items: &[Item]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "INSERT OR IGNORE INTO items (id, title, \"from\", body, source, urgent)
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

    fn store_digest(&mut self, digest: &Digest) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM digest_sources", [])
            .context("failed to clear old digest sources")?;
        conn.execute("DELETE FROM digests", [])
            .context("failed to clear old digests")?;

        conn.execute(
            "INSERT INTO digests (total_items, summary, generated_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![
                digest.total_items as i64,
                digest.summary,
                digest.generated_at.to_rfc3339(),
            ],
        )
        .context("failed to insert digest")?;

        let digest_id = conn.last_insert_rowid();
        let mut stmt = conn
            .prepare(
                "INSERT INTO digest_sources (digest_id, source, item_count)
                 VALUES (?1, ?2, ?3)",
            )
            .context("failed to prepare source insert")?;

        for (source, count) in &digest.by_source {
            stmt.execute(rusqlite::params![digest_id, source, *count as i64])
                .context("failed to insert digest source")?;
        }

        Ok(())
    }

    fn get_selected_mailboxes(&self) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT name FROM selected_mailboxes ORDER BY name")
            .context("failed to prepare mailbox select")?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .context("failed to query mailboxes")?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.context("failed to read mailbox row")?);
        }
        Ok(result)
    }

    fn set_selected_mailboxes(&mut self, mailboxes: &[String]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM selected_mailboxes", [])
            .context("failed to clear selected mailboxes")?;
        let mut stmt = conn
            .prepare("INSERT INTO selected_mailboxes (name) VALUES (?1)")
            .context("failed to prepare mailbox insert")?;
        for m in mailboxes {
            stmt.execute(rusqlite::params![m])
                .context("failed to insert mailbox")?;
        }
        Ok(())
    }

    fn get_item_counts_by_source(&self) -> Result<Vec<(String, usize)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT source, COUNT(*) FROM items GROUP BY source ORDER BY source")
            .context("failed to prepare count query")?;
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
            })
            .context("failed to query item counts")?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.context("failed to read count row")?);
        }
        Ok(result)
    }

    fn get_digest(&self) -> Result<Option<Digest>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, total_items, summary, generated_at
                 FROM digests
                 ORDER BY id DESC
                 LIMIT 1",
            )
            .context("failed to prepare digest select")?;

        let digest_row = stmt
            .query_row([], |row| {
                let id: i64 = row.get(0)?;
                let total_items: i64 = row.get(1)?;
                let summary: String = row.get(2)?;
                let generated_at_str: String = row.get(3)?;
                Ok((id, total_items, summary, generated_at_str))
            })
            .ok();

        let (digest_id, total_items, summary, generated_at_str) = match digest_row {
            Some(r) => r,
            None => return Ok(None),
        };

        let generated_at = chrono::DateTime::parse_from_str(&generated_at_str, "%+")
            .context("failed to parse digest timestamp")?
            .to_utc();

        let mut src_stmt = conn
            .prepare(
                "SELECT source, item_count
                 FROM digest_sources
                 WHERE digest_id = ?1
                 ORDER BY item_count DESC",
            )
            .context("failed to prepare sources select")?;

        let sources = src_stmt
            .query_map(rusqlite::params![digest_id], |row| {
                let source: String = row.get(0)?;
                let count: i64 = row.get(1)?;
                Ok((source, count as usize))
            })
            .context("failed to query digest sources")?;

        let mut by_source = Vec::new();
        for src in sources {
            by_source.push(src.context("failed to read source row")?);
        }

        Ok(Some(Digest {
            total_items: total_items as usize,
            by_source,
            summary,
            generated_at,
        }))
    }
}
