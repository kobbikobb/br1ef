use anyhow::Result;

use crate::{ImapConfig, ImapSource, Item, Source};

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
