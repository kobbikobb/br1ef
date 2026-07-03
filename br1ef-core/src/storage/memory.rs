use anyhow::Result;

use crate::Item;
use crate::storage::Storage;

pub struct InMemoryStorage {
    items: Vec<Item>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl Storage for InMemoryStorage {
    fn store_items(&mut self, items: &[Item]) -> Result<()> {
        self.items = items.to_vec();
        Ok(())
    }

    fn get_items(&self) -> Result<Vec<Item>> {
        Ok(self.items.clone())
    }

    fn clear(&mut self) -> Result<()> {
        self.items.clear();
        Ok(())
    }
}
