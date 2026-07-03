use std::env;

use anyhow::{Context, Result};
use br1ef_core::{ImapConfig, ImapSource, Source};

fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let host = env::var("IMAP_HOST").context("IMAP_HOST not set")?;
    let port: u16 = env::var("IMAP_PORT")
        .unwrap_or_else(|_| "993".into())
        .parse()
        .context("IMAP_PORT must be a number")?;
    let username = env::var("IMAP_USERNAME").context("IMAP_USERNAME not set")?;
    let password = env::var("IMAP_PASSWORD").context("IMAP_PASSWORD not set")?;

    let config = ImapConfig {
        host,
        port,
        username,
        password,
    };

    let source = ImapSource::new(config);
    let items = source.fetch()?;

    if items.is_empty() {
        println!("No email in the last week.");
        return Ok(());
    }

    for item in &items {
        println!("───");
        println!("From:    {}", item.from);
        println!("Subject: {}", item.title);
        println!();
    }

    println!("───");
    println!("{} email(s) in the last week.", items.len());

    Ok(())
}
