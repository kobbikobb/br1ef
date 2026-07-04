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
}
