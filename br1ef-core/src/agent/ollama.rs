use anyhow::{Context, Result};
use serde::Deserialize;

use crate::Item;
use crate::agent::Agent;

#[derive(Deserialize)]
struct ListResponse {
    models: Vec<ModelEntry>,
}

#[derive(Deserialize)]
struct ModelEntry {
    name: String,
}

pub fn list_ollama_models(base_url: &str) -> Result<Vec<String>> {
    let base = base_url.trim_end_matches('/');
    let resp_body = ureq::get(&format!("{base}/api/tags"))
        .call()
        .context("failed to call Ollama API")?
        .into_string()
        .context("failed to read Ollama response")?;
    parse_list_response(&resp_body)
}

pub fn parse_list_response(json: &str) -> Result<Vec<String>> {
    let resp: ListResponse = serde_json::from_str(json)?;
    let mut names: Vec<String> = resp.models.into_iter().map(|m| m.name).collect();
    names.sort();
    Ok(names)
}

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
        let (system, prompt, count) = build_prompts(items);

        crate::progress::with_progress(&format!("Digesting {count} emails..."), || {
            #[derive(Deserialize)]
            struct GenerateResponse {
                response: String,
            }

            let body = serde_json::json!({
                "model": self.model,
                "system": system,
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

fn build_prompts(items: &[Item]) -> (String, String, usize) {
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

    let system = "\
        You are an email digest assistant. Summarize emails into a concise daily brief. \
        Only use information present in the emails — never add external knowledge. \
        No section headers or categories. Output plain text, no markdown formatting.";

    let user = format!(
        "Today is {today}. Below are emails from the last week.\n\n\
         {email_list}\
         List personal messages and action items concisely. \
         Under 150 words.",
    );

    (system.to_string(), user, items.len())
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
