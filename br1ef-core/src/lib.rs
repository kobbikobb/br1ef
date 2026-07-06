pub mod agent;
pub mod fetcher;
pub mod progress;
pub mod service;
pub mod storage;

/// A single digest item — an event, email, or notification
/// fetched from a configured source.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub from: String,
    pub body: String,
    pub source: String,
    pub mailbox: String,
    pub urgent: bool,
}

/// A digest — a processed summary of fetched items.
#[derive(Debug, Clone, PartialEq)]
pub struct Digest {
    pub total_items: usize,
    pub by_source: Vec<(String, usize)>,
    pub summary: String,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

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
            imap_host: "imap.gmail.com".into(),
            imap_port: 993,
            imap_username: "".into(),
            imap_password: "".into(),
            ollama_base_url: "http://localhost:11434".into(),
            ollama_model: "qwen2.5-coder:7b".into(),
        }
    }

    pub fn is_complete(&self) -> bool {
        !self.imap_host.is_empty()
            && !self.imap_username.is_empty()
            && !self.imap_password.is_empty()
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::defaults()
    }
}
