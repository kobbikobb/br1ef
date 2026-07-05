mod ollama;

pub use ollama::{OllamaAgent, list_ollama_models};

use crate::Item;

pub trait Agent: Send + Sync {
    fn summarize_items(&self, items: &[Item]) -> anyhow::Result<String>;
}
