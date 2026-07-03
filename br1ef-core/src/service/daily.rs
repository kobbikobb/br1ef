use anyhow::Result;

use crate::Item;

use super::storage;

pub fn get_daily_items() -> Result<Vec<Item>> {
    storage().lock().unwrap().get_items()
}
