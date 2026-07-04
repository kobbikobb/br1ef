use std::collections::HashMap;

use anyhow::Result;
use chrono::Utc;

use crate::agent::Agent;
use crate::service::dedup::dedup_threads;
use crate::storage::Storage;

pub fn digest_items(storage: &mut dyn Storage, agent: &dyn Agent) -> Result<()> {
    let items = storage.get_items()?;
    let items = dedup_threads(items);

    let summary = if items.is_empty() {
        "No items to summarize.".to_string()
    } else {
        let n = items.len();
        let bytes: usize = items.iter().map(|i| i.body.len()).sum();
        let words: usize = items
            .iter()
            .map(|i| i.body.split_whitespace().count())
            .sum();
        eprintln!("  Generating digest from {n} item(s) ({bytes} bytes, {words} words)...");

        let start = std::time::Instant::now();
        let summary = agent.summarize_items(&items)?;
        let elapsed = start.elapsed();

        eprintln!("  Digest generated in {:.1}s.", elapsed.as_secs_f64());
        summary
    };

    let mut by_source: HashMap<String, usize> = HashMap::new();
    for item in &items {
        *by_source.entry(item.source.clone()).or_default() += 1;
    }

    let mut by_source_vec: Vec<_> = by_source.into_iter().collect();
    by_source_vec.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

    let digest = crate::Digest {
        total_items: items.len(),
        by_source: by_source_vec,
        summary,
        generated_at: Utc::now(),
    };

    storage.store_digest(&digest)?;
    Ok(())
}

#[cfg(test)]
#[path = "digest_test.rs"]
mod tests;
