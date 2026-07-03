use anyhow::Result;

use crate::Item;
use crate::storage::Storage;

pub fn get_daily_items(storage: &dyn Storage) -> Result<Vec<Item>> {
    storage.get_items()
}
