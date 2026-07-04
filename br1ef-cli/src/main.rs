use anyhow::Result;
use br1ef_core::agent::OllamaAgent;
use br1ef_core::service;
use br1ef_core::storage::SqliteStorage;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "br1ef",
    about = "your morning digest",
    disable_help_subcommand = true
)]
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
    let mut storage = SqliteStorage::new("br1ef.db")?;
    let agent = OllamaAgent::new("http://localhost:11434", "llama3.2:1b");
    let cli = Cli::parse();

    match cli.command {
        Commands::Help => {
            print_help();
            Ok(())
        }
        Commands::Fetch => cmd_fetch(&mut storage),
        Commands::Digest => cmd_digest(&mut storage, &agent),
        Commands::Daily => cmd_daily(&storage),
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
    println!("  2. Make sure Ollama is running with llama3.2:1b");
    println!("  3. Run `br1ef fetch`, then `br1ef digest`, then `br1ef daily`");
    println!();
    println!("For more information, visit https://github.com/kobbikobb/br1ef");
}

fn cmd_fetch(storage: &mut dyn br1ef_core::storage::Storage) -> Result<()> {
    dotenvy::dotenv().ok();
    let items = service::fetch_items(storage)?;
    println!("Fetched {} item(s).", items.len());
    Ok(())
}

fn cmd_digest(
    storage: &mut dyn br1ef_core::storage::Storage,
    agent: &dyn br1ef_core::agent::Agent,
) -> Result<()> {
    service::digest_items(storage, agent)?;
    println!("Digest generated.");
    Ok(())
}

fn cmd_daily(storage: &dyn br1ef_core::storage::Storage) -> Result<()> {
    let digest = service::get_daily_items(storage)?;

    match digest {
        None => {
            println!("No digest. Run `br1ef fetch` then `br1ef digest` first.");
        }
        Some(d) => {
            println!("Digest for {}", d.generated_at.format("%B %e, %Y"));
            println!("{}", "═".repeat(40));
            println!("Total items: {}", d.total_items);
            println!();
            println!("By source:");
            for (source, count) in &d.by_source {
                println!("  {source}: {count}");
            }
            println!();
            if !d.summary.is_empty() {
                println!("{}", "─".repeat(40));
                println!("Summary:");
                for line in d.summary.lines() {
                    println!("{line}");
                }
            }
        }
    }

    Ok(())
}

fn cmd_config() -> Result<()> {
    service::configure()?;
    Ok(())
}
