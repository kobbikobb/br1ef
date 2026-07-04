use anyhow::{Context, Result};

use crate::fetcher;
use crate::storage::Storage;

fn display_mailbox(name: &str) -> &str {
    if let Some(cat) = name.strip_prefix("@@CATEGORY@@/") {
        return cat;
    }
    name
}

fn normalize_subject(subject: &str) -> String {
    let s = subject.trim();
    let prefixes = ["re:", "fwd:", "fw:", "aw:", "vs:", "sv:", "vid:", "antw:", "wg:"];
    for p in &prefixes {
        if let Some(rest) = s.strip_prefix(p) {
            if rest.starts_with(' ') || rest.starts_with('[') {
                return normalize_subject(rest);
            }
        }
        let upper = p.to_uppercase();
        if let Some(rest) = s.strip_prefix(&upper) {
            if rest.starts_with(' ') || rest.starts_with('[') {
                return normalize_subject(rest);
            }
        }
        let mixed = format!("{}{}", &p[..1].to_uppercase(), &p[1..]);
        if let Some(rest) = s.strip_prefix(&mixed) {
            if rest.starts_with(' ') || rest.starts_with('[') {
                return normalize_subject(rest);
            }
        }
    }
    s.to_string()
}

fn dedup_threads(items: Vec<Item>) -> Vec<Item> {
    let mut seen: std::collections::HashMap<(String, String), Item> = std::collections::HashMap::new();
    let mut order: Vec<(String, String)> = Vec::new();

    for item in items {
        let normalized = normalize_subject(&item.title);
        let key = (item.from.clone(), normalized);
        if !seen.contains_key(&key) {
            order.push(key.clone());
        }
        seen.insert(key, item);
    }

    order.into_iter().filter_map(|k| seen.remove(&k)).collect()
}

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

        let mut new_count = 0;
        for item in items {
            if seen_ids.insert(item.id.clone()) {
                all_items.push(item);
                new_count += 1;
            }
        }
        eprintln!("  {}: {} new item(s)", display_mailbox(mailbox), new_count);
    }

    let before = all_items.len();
    all_items = dedup_threads(all_items);
    let after = all_items.len();
    if after < before {
        eprintln!("  Thread collapse: {} → {} ({} duplicate threads)", before, after, before - after);
    }

    storage.store_items(&all_items)?;
    Ok(all_items)
}

use crate::Item;
