pub mod agent;
pub mod config;
pub mod fetcher;
pub mod progress;
pub mod service;
pub mod storage;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Source {
    Imap,
}

impl Source {
    pub fn as_str(&self) -> &'static str {
        match self {
            Source::Imap => "imap",
        }
    }
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A single digest item — an event, email, or notification
/// fetched from a configured source.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub from: String,
    pub body: String,
    pub source: Source,
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
