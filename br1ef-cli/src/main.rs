use anyhow::Result;
use br1ef_core::service;
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
    /// Digest fetched data into a brief
    Digest,
    /// Show the daily brief
    Daily,
    /// Configure br1ef preferences
    Config,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Help => {
            print_help();
            Ok(())
        }
        Commands::Fetch => cmd_fetch(),
        Commands::Digest => cmd_digest(),
        Commands::Daily => cmd_daily(),
        Commands::Config => cmd_config(),
    }
}

fn print_help() {
    println!("br1ef — your morning digest");
    println!();
    println!("Usage: br1ef <command>");
    println!();
    println!("Commands:");
    println!("  fetch    Fetch raw data from configured sources");
    println!("  digest   Digest fetched data into a brief");
    println!("  daily    Show the daily brief");
    println!("  config   Configure br1ef preferences");
    println!("  help     Show this usage guide");
    println!();
    println!("Setup:");
    println!("  1. Copy .env.example to .env and fill in your IMAP credentials");
    println!("  2. Run `br1ef daily` to fetch and display your morning digest");
    println!();
    println!("For more information, visit https://github.com/kobbikobb/br1ef");
}

fn cmd_fetch() -> Result<()> {
    service::fetch_items()?;
    Ok(())
}

fn cmd_digest() -> Result<()> {
    service::digest_items(&[])?;
    Ok(())
}

fn cmd_daily() -> Result<()> {
    dotenvy::dotenv().ok();

    let items = service::get_daily_items()?;

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

fn cmd_config() -> Result<()> {
    service::configure()?;
    Ok(())
}
