use anyhow::{Context, Result};

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

    let mailboxes = storage.get_selected_mailboxes()?;
    let mailboxes = if mailboxes.is_empty() {
        vec!["INBOX".to_string()]
    } else {
        mailboxes
    };

    let mut all_items = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    for mailbox in &mailboxes {
        let items = fetcher::fetch_imap(&host, port, &username, &password, mailbox)
            .with_context(|| format!("failed to fetch from \"{mailbox}\""))?;

        for item in items {
            if seen_ids.insert(item.id.clone()) {
                all_items.push(item);
            }
        }
    }

    storage.store_items(&all_items)?;
    Ok(all_items)
}

use crate::Item;
