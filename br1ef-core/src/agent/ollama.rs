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

        crate::progress::with_progress(&format!("Digesting {} emails...", items.len()), || {
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

            Ok::<_, anyhow::Error>(resp.response.trim().to_string())
        })
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
            truncate(&item.body, 500),
        );
    }

    let today = chrono::Utc::now().format("%B %-e, %Y");

    format!(
        "Today is {today}. Below are emails from the last week.\n\n\
         {email_list}\
         Only use the information from these emails — do not add anything\n\
         not present in the emails above.\n\n\
         List personal messages and action items concisely. Skip commercial\n\
         emails, newsletters, and LinkedIn notifications. No section headers\n\
         or categories. Under 150 words.",
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

#[cfg(test)]
#[path = "ollama_test.rs"]
mod tests;
