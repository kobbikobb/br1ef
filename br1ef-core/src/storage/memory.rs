use anyhow::Result;

#[cfg(test)]
#[path = "memory_test.rs"]
mod tests;

use crate::storage::Storage;
use crate::{Digest, Item};

pub struct InMemoryStorage {
    items: Vec<Item>,
    digest: Option<Digest>,
    selected_mailboxes: Vec<String>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            digest: None,
            selected_mailboxes: Vec::new(),
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
        let existing_ids: std::collections::HashSet<String> =
            self.items.iter().map(|i| i.id.clone()).collect();
        for item in items {
            if existing_ids.contains(&item.id) {
                continue;
            }
            self.items.push(item.clone());
        }
        Ok(())
    }

    fn get_items(&self) -> Result<Vec<Item>> {
        Ok(self.items.clone())
    }

    fn clear(&mut self) -> Result<()> {
        self.items.clear();
        Ok(())
    }

    fn get_selected_mailboxes(&self) -> Result<Vec<String>> {
        Ok(self.selected_mailboxes.clone())
    }

    fn set_selected_mailboxes(&mut self, mailboxes: &[String]) -> Result<()> {
        self.selected_mailboxes = mailboxes.to_vec();
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
