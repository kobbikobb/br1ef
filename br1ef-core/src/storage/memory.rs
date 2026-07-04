use anyhow::Result;

use crate::storage::Storage;
use crate::{Digest, Item};

pub struct InMemoryStorage {
    items: Vec<Item>,
    digest: Option<Digest>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            digest: None,
        }
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

    fn store_digest(&mut self, digest: &Digest) -> Result<()> {
        self.digest = Some(digest.clone());
        Ok(())
    }

    fn get_digest(&self) -> Result<Option<Digest>> {
        Ok(self.digest.clone())
    }
}
