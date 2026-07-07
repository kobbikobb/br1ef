use std::collections::HashMap;

use anyhow::Result;
use chrono::Utc;

use crate::agent::Agent;
use crate::storage::Storage;

use crate::{Digest, Item};

const REPLY_PREFIXES: &[&str] = &[
    "re:", "fwd:", "fw:", "aw:", "vs:", "sv:", "vid:", "antw:", "wg:",
];

fn normalize_subject(subject: &str) -> String {
    let mut s = subject.trim();
    loop {
        let lower = s.to_lowercase();
        let mut found = false;
        for &p in REPLY_PREFIXES {
            if lower.starts_with(p) {
                let after = &s[p.len()..];
                if after.starts_with(' ') || after.starts_with('[') {
                    let trimmed = after.trim_start();
                    if !trimmed.is_empty() {
                        s = trimmed;
                        found = true;
                        break;
                    }
                }
            }
        }
        if !found {
            return s.to_string();
        }
    }
}

fn dedup_threads(items: Vec<Item>) -> Vec<Item> {
    let mut seen: std::collections::HashMap<(String, String), Item> =
        std::collections::HashMap::new();
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

fn is_noise(item: &Item) -> bool {
    let from_lower = item.from.to_lowercase();
    let title_lower = item.title.to_lowercase();

    from_lower.contains("linkedin.com")
        || title_lower.contains("newsletter")
        || from_lower.contains("newsletter@")
        || from_lower.contains("marketing")
        || from_lower.contains("no-reply")
        || from_lower.contains("noreply")
}

fn filter_relevant(items: &[Item]) -> Vec<Item> {
    items.iter().filter(|i| !is_noise(i)).cloned().collect()
}

pub fn build_digest(items: Vec<Item>, agent: &dyn Agent) -> Result<Digest> {
    let items = dedup_threads(items);
    let relevant = filter_relevant(&items);

    let summary = if items.is_empty() {
        "No items to summarize.".to_string()
    } else if relevant.is_empty() {
        "Nothing needs attention today.".to_string()
    } else {
        agent.summarize_items(&relevant)?
    };

    let mut by_source: HashMap<String, usize> = HashMap::new();
    for item in &items {
        *by_source.entry(item.source.clone()).or_default() += 1;
    }

    let mut by_source_vec: Vec<_> = by_source.into_iter().collect();
    by_source_vec.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

    Ok(Digest {
        total_items: items.len(),
        by_source: by_source_vec,
        summary,
        generated_at: Utc::now(),
    })
}

pub fn digest_items(storage: &mut dyn Storage, agent: &dyn Agent) -> Result<()> {
    let raw_items = storage.get_items()?;
    let items = dedup_threads(raw_items);
    let relevant = filter_relevant(&items);

    let has_content = !items.is_empty() && !relevant.is_empty();

    if has_content {
        let bytes: usize = relevant.iter().map(|i| i.body.len()).sum();
        let words: usize = relevant
            .iter()
            .map(|i| i.body.split_whitespace().count())
            .sum();
        let size = if bytes > 1024 * 1024 {
            format!("{:.1} MiB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} KiB", bytes as f64 / 1024.0)
        };
        eprintln!(
            "  📖 Digesting {} item(s) ({size}, {words} words)…",
            relevant.len()
        );
    }

    let start = std::time::Instant::now();
    let digest = build_digest(items, agent)?;
    let elapsed = start.elapsed();
    storage.store_digest(&digest)?;

    if has_content {
        eprintln!("  ✨ Digest ready — {:.1}s.", elapsed.as_secs_f64());
    }

    Ok(())
}

#[cfg(test)]
#[path = "digest_test.rs"]
mod tests;
