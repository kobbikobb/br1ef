use anyhow::Result;

use crate::Item;

pub fn digest_items(items: &[Item]) -> Result<Vec<Item>> {
    let _ = items;
    anyhow::bail!("digest not implemented yet")
}
