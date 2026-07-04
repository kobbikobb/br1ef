use std::collections::HashMap;

use anyhow::Result;
use chrono::Utc;

use crate::storage::Storage;

pub fn digest_items(storage: &mut dyn Storage) -> Result<()> {
    let items = storage.get_items()?;

    let mut by_source: HashMap<String, usize> = HashMap::new();
    for item in &items {
        *by_source.entry(item.source.clone()).or_default() += 1;
    }

    let mut by_source_vec: Vec<_> = by_source.into_iter().collect();
    by_source_vec.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

    let digest = crate::Digest {
        total_items: items.len(),
        by_source: by_source_vec,
        generated_at: Utc::now(),
    };

    storage.store_digest(&digest)?;
    Ok(())
}
