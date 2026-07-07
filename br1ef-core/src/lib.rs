pub mod agent;
pub mod config;
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
