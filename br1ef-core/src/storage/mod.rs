mod db;
mod memory;

pub use db::SqliteStorage;
pub use memory::InMemoryStorage;

use crate::Item;

pub trait Storage: Send + Sync {
    fn store_items(&mut self, items: &[Item]) -> anyhow::Result<()>;
    fn get_items(&self) -> anyhow::Result<Vec<Item>>;
    fn clear(&mut self) -> anyhow::Result<()>;
}
