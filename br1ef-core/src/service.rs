use anyhow::Result;

use crate::{ImapConfig, ImapSource, Item, Source};

/// Fetch raw items from all configured sources.
pub fn fetch_items() -> Result<Vec<Item>> {
    anyhow::bail!("fetch not implemented yet")
}

/// Classify and rank items into a brief.
pub fn digest_items(items: &[Item]) -> Result<Vec<Item>> {
    let _ = items;
    anyhow::bail!("digest not implemented yet")
}

/// Show the daily brief: fetch + digest, then return items to display.
pub fn get_daily_items() -> Result<Vec<Item>> {
    let host = std::env::var("IMAP_HOST")?;
    let port: u16 = std::env::var("IMAP_PORT")
        .unwrap_or_else(|_| "993".into())
        .parse()?;
    let username = std::env::var("IMAP_USERNAME")?;
    let password = std::env::var("IMAP_PASSWORD")?;

    let config = ImapConfig {
        host,
        port,
        username,
        password,
    };

    let source = ImapSource::new(config);
    source.fetch()
}

/// Open configuration interface.
pub fn configure() -> Result<()> {
    anyhow::bail!("config not implemented yet")
}
