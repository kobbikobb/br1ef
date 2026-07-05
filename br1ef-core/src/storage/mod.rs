mod db;
mod memory;

pub use db::SqliteStorage;
pub use memory::InMemoryStorage;

use crate::{Digest, Item};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub imap_host: String,
    pub imap_port: u16,
    pub imap_username: String,
    pub imap_password: String,
    pub ollama_base_url: String,
    pub ollama_model: String,
}

impl AppConfig {
    pub fn defaults() -> Self {
        Self {
            imap_host: "".into(),
            imap_port: 993,
            imap_username: "".into(),
            imap_password: "".into(),
            ollama_base_url: "http://localhost:11434".into(),
            ollama_model: "llama3.2:1b".into(),
        }
    }

    pub fn is_complete(&self) -> bool {
        !self.imap_host.is_empty()
            && !self.imap_username.is_empty()
            && !self.imap_password.is_empty()
    }
}

pub trait Storage: Send + Sync {
    fn store_items(&mut self, items: &[Item]) -> anyhow::Result<()>;
    fn get_items(&self) -> anyhow::Result<Vec<Item>>;
    fn clear(&mut self) -> anyhow::Result<()>;

    fn store_digest(&mut self, digest: &Digest) -> anyhow::Result<()>;
    fn get_digest(&self) -> anyhow::Result<Option<Digest>>;

    fn get_selected_mailboxes(&self) -> anyhow::Result<Vec<String>>;
    fn set_selected_mailboxes(&mut self, mailboxes: &[String]) -> anyhow::Result<()>;

    fn get_item_counts_by_source(&self) -> anyhow::Result<Vec<(String, usize)>>;
    fn get_item_counts_by_mailbox(&self) -> anyhow::Result<Vec<(String, String, usize)>>;

    fn get_app_config(&self) -> anyhow::Result<AppConfig>;
    fn set_app_config(&mut self, cfg: &AppConfig) -> anyhow::Result<()>;
}
