use anyhow::Result;
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

pub fn fetch_items(storage: &mut dyn Storage, fetcher: &dyn Fetcher) -> Result<FetchResult> {
    let mailboxes = storage.get_selected_mailboxes()?;
    let mailboxes = if mailboxes.is_empty() {
        match fetcher.suggested_mailboxes() {
            Ok(mb) => mb,
            Err(e) => {
                eprintln!("warn: failed to auto-detect mailboxes: {e:#}");
                vec!["INBOX".to_string()]
            }
        }
    } else {
        mailboxes
    };

    let mut all_items = Vec::new();
    let mut seen_ids = HashSet::new();
    let mut per_mailbox = Vec::with_capacity(mailboxes.len());

    for mailbox in &mailboxes {
        let items = match fetcher.fetch_mailbox(mailbox) {
            Ok(items) => items,
            Err(e) => {
                eprintln!("warn: skipping \"{mailbox}\": {e:#}");
                continue;
            }
        };

        let mailbox_items: Vec<Item> = items
            .into_iter()
            .map(|mut item| {
                item.mailbox = mailbox.clone();
                item
            })
            .collect();

        let total = mailbox_items.len();
        let new_count = mailbox_items
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
