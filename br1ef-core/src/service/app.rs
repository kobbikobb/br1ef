use anyhow::{Context, Result};

use crate::Item;
use crate::fetcher;
use crate::storage::Storage;

pub struct App {
    storage: Box<dyn Storage>,
}

impl App {
    pub fn new(storage: Box<dyn Storage>) -> Self {
        Self { storage }
    }

    pub fn fetch_items(&mut self) -> Result<Vec<Item>> {
        let host = std::env::var("IMAP_HOST").context("IMAP_HOST not set")?;
        let port: u16 = std::env::var("IMAP_PORT")
            .unwrap_or_else(|_| "993".into())
            .parse()
            .context("IMAP_PORT must be a number")?;
        let username = std::env::var("IMAP_USERNAME").context("IMAP_USERNAME not set")?;
        let password = std::env::var("IMAP_PASSWORD").context("IMAP_PASSWORD not set")?;

        let items = fetcher::fetch_imap(&host, port, &username, &password)?;
        self.storage.store_items(&items)?;

        Ok(items)
    }

    pub fn get_daily_items(&self) -> Result<Vec<Item>> {
        self.storage.get_items()
    }

    pub fn digest_items(&self, items: &[Item]) -> Result<Vec<Item>> {
        let _ = items;
        anyhow::bail!("digest not implemented yet")
    }

    pub fn configure(&self) -> Result<()> {
        anyhow::bail!("config not implemented yet")
    }
}
