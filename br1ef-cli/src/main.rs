use std::env;

use anyhow::{Context, Result};
use br1ef_core::{ImapConfig, ImapSource, Source};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "br1ef", about = "your morning digest")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show this help message with detailed usage
    Help,
    /// Fetch raw data from configured sources
    Fetch,
    /// Analyze fetched data and build a brief
    Analyze,
    /// Show the daily brief
    Daily,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Help => {
            print_help();
            Ok(())
        }
        Commands::Fetch => cmd_fetch(),
        Commands::Analyze => cmd_analyze(),
        Commands::Daily => cmd_daily(),
    }
}

fn print_help() {
    println!("br1ef — your morning digest");
    println!();
    println!("Usage: br1ef <command>");
    println!();
    println!("Commands:");
    println!("  help     Show this usage guide");
    println!("  fetch    Fetch raw data from configured sources");
    println!("  analyze  Analyze fetched data and build a brief");
    println!("  daily    Show the daily brief");
    println!();
    println!("Setup:");
    println!("  1. Copy .env.example to .env and fill in your IMAP credentials");
    println!("  2. Run `br1ef daily` to fetch and display your morning digest");
    println!();
    println!("For more information, visit https://github.com/kobbikobb/br1ef");
}

fn cmd_fetch() -> Result<()> {
    anyhow::bail!("not implemented yet")
}

fn cmd_analyze() -> Result<()> {
    anyhow::bail!("not implemented yet")
}

fn cmd_daily() -> Result<()> {
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
    }

    println!("───");
    println!("{} email(s) in the last week.", items.len());

    Ok(())
}
