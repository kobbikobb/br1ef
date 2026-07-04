use anyhow::{Context, Result};
use serde::Deserialize;

use crate::Item;
use crate::agent::Agent;

pub struct OllamaAgent {
    base_url: String,
    model: String,
}

impl OllamaAgent {
    pub fn new(base_url: &str, model: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            model: model.to_string(),
        }
    }
}

impl Agent for OllamaAgent {
    fn summarize_items(&self, items: &[Item]) -> Result<String> {
        let prompt = build_prompt(items);

        #[derive(Deserialize)]
        struct GenerateResponse {
            response: String,
        }

        let body = serde_json::json!({
            "model": self.model,
            "prompt": prompt,
            "stream": false,
        });

        let resp: GenerateResponse = ureq::post(&format!("{}/api/generate", self.base_url))
            .send_json(&body)
            .context("failed to call Ollama API")?
            .into_json()
            .context("failed to parse Ollama response")?;

        Ok(resp.response.trim().to_string())
    }
}

fn build_prompt(items: &[Item]) -> String {
    let mut email_list = String::new();
    for (i, item) in items.iter().enumerate() {
        use std::fmt::Write;
        let _ = write!(
            email_list,
            "{}. From: {} | Subject: {}\n   Body: {}\n\n",
            i + 1,
            item.from,
            item.title,
            truncate(&item.body, 200),
        );
    }

    format!(
        "You are a personal assistant reviewing today's email.\n\
         Below are emails from the last week.\n\n\
         {}\n\
         Please provide a brief summary with:\n\
         1. Key highlights — things that need attention\n\
         2. Any events happening today or in the next few days\n\n\
         Keep it concise, under 150 words.",
        email_list
    )
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        return s;
    }
    let cut = s
        .char_indices()
        .take_while(|(i, _)| *i < max)
        .last()
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(0);
    &s[..cut]
}
