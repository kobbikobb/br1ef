use anyhow::{Context, Result};
use std::collections::HashSet;

use crate::Item;
use crate::fetcher::Fetcher;
use crate::storage::Storage;

#[derive(Debug)]
pub struct MailboxStats {
    pub name: String,
    pub total: usize,
    pub new: usize,
}

#[derive(Debug)]
pub struct FetchResult {
    pub items: Vec<Item>,
    pub per_mailbox: Vec<MailboxStats>,
}

const CATEGORY_PREFIX: &str = "@@CATEGORY@@/";

pub fn fetch_items(storage: &mut dyn Storage, fetcher: &dyn Fetcher) -> Result<FetchResult> {
    let mailboxes = storage.get_selected_mailboxes()?;
    let mailboxes = if mailboxes.is_empty() {
        vec!["INBOX".to_string()]
    } else {
        mailboxes
    };

    let has_inbox = mailboxes.iter().any(|m| m == "INBOX");
    let mut all_items = Vec::new();
    let mut seen_ids = HashSet::new();
    let mut per_mailbox = Vec::with_capacity(mailboxes.len());

    for mailbox in &mailboxes {
        if has_inbox && mailbox.starts_with(CATEGORY_PREFIX) {
            let cat = &mailbox[CATEGORY_PREFIX.len()..];
            eprintln!("  ⏭ {cat} — already in INBOX");
            continue;
        }

        let items = fetcher
            .fetch_mailbox(mailbox)
            .with_context(|| format!("failed to fetch from \"{mailbox}\""))?;

        let total = items.len();
        let new_count = items
            .into_iter()
            .filter(|item| seen_ids.insert(item.id.clone()))
            .inspect(|item| all_items.push(item.clone()))
            .count();

        per_mailbox.push(MailboxStats {
            name: mailbox.clone(),
            total,
            new: new_count,
        });
    }

    storage.store_items(&all_items)?;
    Ok(FetchResult {
        items: all_items,
        per_mailbox,
    })
}

#[cfg(test)]
#[path = "fetch_test.rs"]
mod tests;
