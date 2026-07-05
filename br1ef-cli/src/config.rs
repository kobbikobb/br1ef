use std::io::Write;

use anyhow::{Context, Result};
use br1ef_core::fetcher::{Fetcher, GMAIL_CATEGORY_PREFIX};
use br1ef_core::storage::{AppConfig, Storage};

pub fn display_name(raw: &str) -> &str {
    if let Some(cat) = raw.strip_prefix(GMAIL_CATEGORY_PREFIX) {
        return cat;
    }
    raw
}

pub fn configure(storage: &mut dyn Storage, fetcher: Option<&dyn Fetcher>) -> Result<()> {
    let mut cfg = storage.get_app_config()?;

    // Step 1: connection settings
    println!("Step 1/3 — Connection Settings");

    cfg.imap_host = ask("IMAP server (hostname)", &cfg.imap_host)
        .with_context(|| "Failed to read IMAP host")?;
    cfg.imap_port = ask_port(&cfg.imap_port)?;
    cfg.imap_username = ask("IMAP username / email", &cfg.imap_username)
        .with_context(|| "Failed to read IMAP username")?;

    {
        let has_pw = !cfg.imap_password.is_empty();
        print!(
            "IMAP password{}",
            if has_pw { " (Enter to keep)" } else { "" }
        );
        print!(": ");
        std::io::Write::flush(&mut std::io::stdout())?;
        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;
        let trimmed = line.trim().to_string();
        if !trimmed.is_empty() {
            cfg.imap_password = trimmed;
        }
    }

    println!();
    cfg.ollama_base_url = ask("Ollama base URL", &cfg.ollama_base_url)
        .with_context(|| "Failed to read Ollama URL")?;

    match br1ef_core::agent::list_ollama_models(&cfg.ollama_base_url) {
        Ok(models) if !models.is_empty() => {
            println!("\nAvailable models:");
            for (i, name) in models.iter().enumerate() {
                let current = if *name == cfg.ollama_model {
                    " (current)"
                } else {
                    ""
                };
                println!("  {:2}. {}{}", i + 1, name, current);
            }
            print!("Select model (number, or Enter for current): ");
            std::io::Write::flush(&mut std::io::stdout())?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if let Ok(idx) = input.trim().parse::<usize>()
                && idx > 0 && idx <= models.len()
            {
                cfg.ollama_model = models[idx - 1].clone();
            }
        }
        _ => {
            cfg.ollama_model = ask("Ollama model name", &cfg.ollama_model)
                .with_context(|| "Failed to read Ollama model")?;
        }
    }

    // Step 2: mailbox selection
    let mailboxes: Vec<String> = if let Some(f) = fetcher {
        f.list_mailboxes().map_err(|e| anyhow::anyhow!("Cannot connect to mail server — can't select mailboxes without connection settings configured: {}", e))?
    } else {
        vec![]
    };

    if mailboxes.is_empty() {
        println!("\nStep 2/3 — Mailbox Selection");
        println!("  (no mailboxes available — connect to IMAP server to see mailbox list)");
        println!("  Defaulting to INBOX.");
        let sel = vec!["INBOX".to_string()];
        storage.set_app_config(&cfg)?;
        storage.set_selected_mailboxes(&sel)?;
        finish(&cfg, storage)?;
        return Ok(());
    }

    println!("\nStep 2/3 — Mailbox Selection");
    for (i, name) in mailboxes.iter().enumerate() {
        let marker = if name == "INBOX" { " (default)" } else { "" };
        let suffix = if name.starts_with(GMAIL_CATEGORY_PREFIX) {
            " (category)"
        } else {
            ""
        };
        println!("  {:2}. {}{}{}", i + 1, display_name(name), marker, suffix);
    }

    println!("\nINBOX always selected.");
    print!(r#"Choose mailboxes (comma-separated numbers, 'all', or Enter for defaults): "#);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    let sel: Vec<String> = if input.trim().is_empty() {
        let cats: Vec<String> = mailboxes
            .iter()
            .filter(|m| m.starts_with(GMAIL_CATEGORY_PREFIX))
            .cloned()
            .collect();
        let mut s = vec!["INBOX".into()];
        s.extend(cats);
        s
    } else if input.trim().eq_ignore_ascii_case("all") {
        mailboxes.clone()
    } else {
        let mut seen = std::collections::HashSet::new();
        seen.insert("INBOX".to_string());
        let mut selected = vec!["INBOX".to_string()];
        for part in input.split(',') {
            if let Ok(idx) = part.trim().parse::<usize>()
                && idx > 0
                && idx <= mailboxes.len()
            {
                let name = mailboxes[idx - 1].clone();
                if seen.insert(name.clone()) {
                    selected.push(name);
                }
            }
        }
        selected
    };

    storage.set_app_config(&cfg)?;
    storage.set_selected_mailboxes(&sel)?;
    finish(&cfg, storage)
}

fn finish(cfg: &AppConfig, storage: &dyn Storage) -> Result<()> {
    println!("\nStep 3/3 — Config Complete!");
    let disp = |val: &str, default: &str| -> String {
        if val.is_empty() || val == default {
            "<default>".into()
        } else {
            val.to_string()
        }
    };
    println!(r#"  IMAP host:     {}"#, disp(&cfg.imap_host, ""));
    println!(
        "  Ollama URL:    {}",
        disp(&cfg.ollama_base_url, "http://localhost:11434")
    );
    println!(
        "  Ollama model:  {}",
        disp(&cfg.ollama_model, "llama3.2:1b")
    );

    let mbx = storage.get_selected_mailboxes()?;
    println!(
        "  Mailboxes:     {} ({} in total)",
        mbx.iter()
            .map(|m| display_name(m))
            .collect::<Vec<_>>()
            .join(", "),
        mbx.len()
    );

    println!("\nRun `br1ef fetch`, then `br1ef digest` when ready.");
    Ok(())
}

fn ask(prompt: &str, current: &str) -> Result<String> {
    let mut input = String::new();

    print!(
        "{prompt} (current: {}) : ",
        if current.is_empty() {
            "(none)"
        } else {
            current
        }
    );
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut input)?;
    Ok(prompt_value(&input, current))
}

pub fn prompt_value(input: &str, current: &str) -> String {
    let trimmed = input.trim().to_string();
    if trimmed.is_empty() {
        current.to_string()
    } else {
        trimmed
    }
}

pub fn parse_port(input: &str, current: u16) -> u16 {
    match input.trim().parse::<u16>() {
        Ok(p) if p > 0 => p,
        _ => current,
    }
}

fn ask_port(current: &u16) -> Result<u16> {
    let mut input = String::new();
    print!("IMAP port (current: {}): ", current);
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut input)?;
    Ok(parse_port(&input, *current))
}
