use anyhow::{Context, Result};

use crate::Item;
use crate::fetcher::Fetcher;
use crate::storage::Storage;

fn display_mailbox(name: &str) -> &str {
    if let Some(cat) = name.strip_prefix("@@CATEGORY@@/") {
        return cat;
    }
    name
}

pub fn fetch_items(storage: &mut dyn Storage, fetcher: &dyn Fetcher) -> Result<Vec<Item>> {
    let mailboxes = storage.get_selected_mailboxes()?;
    let mailboxes = if mailboxes.is_empty() {
        vec!["INBOX".to_string()]
    } else {
        mailboxes
    };

    let mut all_items = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    for mailbox in &mailboxes {
        let items = fetcher
            .fetch_mailbox(mailbox)
            .with_context(|| format!("failed to fetch from \"{mailbox}\""))?;

        let total = items.len();
        let new_count = items
            .into_iter()
            .filter(|item| seen_ids.insert(item.id.clone()))
            .inspect(|item| all_items.push(item.clone()))
            .count();

        println!(
            "  {}: {} / {} new",
            display_mailbox(mailbox),
            new_count,
            total,
        );
    }

    storage.store_items(&all_items)?;
    Ok(all_items)
}

#[cfg(test)]
#[path = "fetch_test.rs"]
mod tests;
