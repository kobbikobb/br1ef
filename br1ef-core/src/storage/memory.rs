use anyhow::Result;

#[cfg(test)]
#[path = "memory_test.rs"]
mod tests;

use crate::storage::{AppConfig, Storage};
use crate::{Digest, Item};

pub struct InMemoryStorage {
    items: Vec<Item>,
    digest: Option<Digest>,
    selected_mailboxes: Vec<String>,
    app_config: AppConfig,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            digest: None,
            selected_mailboxes: Vec::new(),
            app_config: AppConfig::defaults(),
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
        self.digest = None;
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

    fn get_item_counts_by_source(&self) -> Result<Vec<(String, usize)>> {
        let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for item in &self.items {
            *counts.entry(item.source.clone()).or_default() += 1;
        }
        let mut result: Vec<_> = counts.into_iter().collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(result)
    }

    fn get_item_counts_by_mailbox(&self) -> Result<Vec<(String, String, usize)>> {
        let mut counts: std::collections::HashMap<(String, String), usize> =
            std::collections::HashMap::new();
        for item in &self.items {
            let key = (item.source.clone(), item.mailbox.clone());
            *counts.entry(key).or_default() += 1;
        }
        let mut result: Vec<_> = counts
            .into_iter()
            .map(|((source, mailbox), count)| (source, mailbox, count))
            .collect();
        result.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        Ok(result)
    }

    fn get_digest(&self) -> Result<Option<Digest>> {
        Ok(self.digest.clone())
    }

    fn get_app_config(&self) -> Result<AppConfig> {
        Ok(self.app_config.clone())
    }

    fn set_app_config(&mut self, cfg: &AppConfig) -> Result<()> {
        self.app_config = cfg.clone();
        Ok(())
    }
}
