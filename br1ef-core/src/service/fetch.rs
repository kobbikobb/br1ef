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

        let mut new_count = 0;
        for item in items {
            if seen_ids.insert(item.id.clone()) {
                all_items.push(item);
                new_count += 1;
            }
        }
        eprintln!("  {}: {} new item(s)", display_mailbox(mailbox), new_count);
    }

    storage.store_items(&all_items)?;
    Ok(all_items)
}

#[cfg(test)]
#[path = "fetch_test.rs"]
mod tests;
