use anyhow::Result;

use crate::storage::Storage;

pub fn get_daily_items(storage: &dyn Storage) -> Result<Option<crate::Digest>> {
    storage.get_digest()
}
