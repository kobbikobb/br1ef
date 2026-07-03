use anyhow::{Context, Result};

use crate::Item;
use crate::fetcher;
use crate::storage::Storage;

pub fn fetch_items(storage: &mut dyn Storage) -> Result<Vec<Item>> {
    let host = std::env::var("IMAP_HOST").context("IMAP_HOST not set")?;
    let port: u16 = std::env::var("IMAP_PORT")
        .unwrap_or_else(|_| "993".into())
        .parse()
        .context("IMAP_PORT must be a number")?;
    let username = std::env::var("IMAP_USERNAME").context("IMAP_USERNAME not set")?;
    let password = std::env::var("IMAP_PASSWORD").context("IMAP_PASSWORD not set")?;

    let items = fetcher::fetch_imap(&host, port, &username, &password)?;
    storage.store_items(&items)?;

    Ok(items)
}
