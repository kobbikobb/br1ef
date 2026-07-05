mod db;
mod memory;

pub use db::SqliteStorage;
pub use memory::InMemoryStorage;

use crate::{Digest, Item};

pub trait Storage: Send + Sync {
    fn store_items(&mut self, items: &[Item]) -> anyhow::Result<()>;
    fn get_items(&self) -> anyhow::Result<Vec<Item>>;
    fn clear(&mut self) -> anyhow::Result<()>;

    fn store_digest(&mut self, digest: &Digest) -> anyhow::Result<()>;
    fn get_digest(&self) -> anyhow::Result<Option<Digest>>;

    fn get_selected_mailboxes(&self) -> anyhow::Result<Vec<String>>;
    fn set_selected_mailboxes(&mut self, mailboxes: &[String]) -> anyhow::Result<()>;

    fn get_item_counts_by_source(&self) -> anyhow::Result<Vec<(String, usize)>>;
}
