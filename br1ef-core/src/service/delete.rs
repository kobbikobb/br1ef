use anyhow::Result;

use crate::storage::Storage;

pub fn delete_items(storage: &mut dyn Storage) -> Result<usize> {
    let counts = storage.get_item_counts_by_source()?;
    let total: usize = counts.iter().map(|(_, c)| c).sum();
    storage.clear()?;
    Ok(total)
}

#[cfg(test)]
#[path = "delete_test.rs"]
mod tests;
