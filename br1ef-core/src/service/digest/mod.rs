use std::collections::HashMap;

use anyhow::Result;
use chrono::Utc;

mod dedup;
mod noise;

use self::dedup::dedup_threads;
use crate::agent::Agent;
use crate::storage::Storage;

pub fn build_digest(storage: &mut dyn Storage, agent: &dyn Agent) -> Result<crate::Digest> {
    let items = storage.get_items()?;
    let items = dedup_threads(items);
    let relevant = noise::filter_relevant(&items);

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

    Ok(crate::Digest {
        total_items: items.len(),
        by_source: by_source_vec,
        summary,
        generated_at: Utc::now(),
    })
}

pub fn digest_items(storage: &mut dyn Storage, agent: &dyn Agent) -> Result<()> {
    let items = storage.get_items()?;
    let items = dedup_threads(items);
    let relevant = noise::filter_relevant(&items);

    if !items.is_empty() && !relevant.is_empty() {
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
    let digest = build_digest(storage, agent)?;
    let elapsed = start.elapsed();
    storage.store_digest(&digest)?;

    if !items.is_empty() && !relevant.is_empty() {
        eprintln!("  ✨ Digest ready — {:.1}s.", elapsed.as_secs_f64());
    }

    Ok(())
}

#[cfg(test)]
#[path = "digest_test.rs"]
mod tests;
