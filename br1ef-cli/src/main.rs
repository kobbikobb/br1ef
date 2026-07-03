use anyhow::Result;
use br1ef_core::service::App;
use br1ef_core::storage::InMemoryStorage;
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
    let storage = Box::new(InMemoryStorage::new());
    let mut app = App::new(storage);
    dotenvy::dotenv().ok();

    match Cli::parse().command {
        Commands::Help => print_help(),
        Commands::Fetch => {
            let items = app.fetch_items()?;
            println!("Fetched {} item(s).", items.len());
        }
        Commands::Digest => {
            app.digest_items(&[])?;
        }
        Commands::Daily => {
            let items = app.get_daily_items()?;
            if items.is_empty() {
                println!("No items. Run `br1ef fetch` first.");
            } else {
                for item in &items {
                    println!("───");
                    println!("From:    {}", item.from);
                    println!("Subject: {}", item.title);
                }
                println!("───");
                println!("{} item(s).", items.len());
            }
        }
        Commands::Config => {
            app.configure()?;
        }
    }

    Ok(())
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
    println!("  2. Run `br1ef fetch` then `br1ef daily` to see your digest");
    println!();
    println!("For more information, visit https://github.com/kobbikobb/br1ef");
}
