mod ollama;

pub use ollama::OllamaAgent;

use crate::Item;

pub trait Agent: Send + Sync {
    fn summarize_items(&self, items: &[Item]) -> anyhow::Result<String>;
}
