use anyhow::Result;

use crate::fetcher::Fetcher;
use crate::storage::Storage;

const CATEGORY_PREFIX: &str = "@@CATEGORY@@/";

fn display_name(raw: &str) -> &str {
    if let Some(cat) = raw.strip_prefix(CATEGORY_PREFIX) {
        return cat;
    }
    raw
}

pub fn configure(storage: &mut dyn Storage, fetcher: &dyn Fetcher) -> Result<()> {
    let all = fetcher.list_mailboxes()?;

    println!("Available mailboxes:\n");
    for (i, name) in all.iter().enumerate() {
        let marker = if name == "INBOX" {
            " (always selected)"
        } else {
            ""
        };
        let suffix = if name.starts_with(CATEGORY_PREFIX) {
            " (category)"
        } else {
            ""
        };
        println!("  {:2}. {}{}{}", i + 1, display_name(name), marker, suffix);
    }

    println!("\nINBOX is always included.");
    println!("Enter numbers of other mailboxes to fetch (comma-separated),");
    println!("'all' for all, or leave empty for just INBOX:");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let input = input.trim();

    let mut selected = vec!["INBOX".to_string()];

    if input.eq_ignore_ascii_case("all") {
        for name in &all {
            if name != "INBOX" && !selected.contains(name) {
                selected.push(name.clone());
            }
        }
    } else if !input.is_empty() {
        for part in input.split(',') {
            let idx: usize = part
                .trim()
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid number — enter comma-separated numbers"))?;
            if idx == 0 || idx > all.len() {
                anyhow::bail!("Number {idx} is out of range (1-{})", all.len());
            }
            let name = &all[idx - 1];
            if !selected.contains(name) {
                selected.push(name.clone());
            }
        }
    }

    storage.set_selected_mailboxes(&selected)?;
    println!("\nSaved! Will fetch from {} mailbox(es).", selected.len());
    Ok(())
}
