mod imap;

pub use imap::GMAIL_CATEGORY_PREFIX;

use anyhow::{Context, Result};

use crate::Item;

pub trait Fetcher {
    fn fetch_mailbox(&self, mailbox: &str) -> Result<Vec<Item>>;
    fn list_mailboxes(&self) -> Result<Vec<String>>;
}

pub struct ImapFetcher {
    host: String,
    port: u16,
    username: String,
    password: String,
}

impl ImapFetcher {
    pub fn new(host: &str, port: u16, username: &str, password: &str) -> Self {
        Self {
            host: host.to_string(),
            port,
            username: username.to_string(),
            password: password.to_string(),
        }
    }

    pub fn from_env() -> Result<Self> {
        let host = std::env::var("IMAP_HOST").context("IMAP_HOST not set")?;
        let port: u16 = std::env::var("IMAP_PORT")
            .unwrap_or_else(|_| "993".into())
            .parse()
            .context("IMAP_PORT must be a number")?;
        let username = std::env::var("IMAP_USERNAME").context("IMAP_USERNAME not set")?;
        let password = std::env::var("IMAP_PASSWORD").context("IMAP_PASSWORD not set")?;
        Ok(Self {
            host,
            port,
            username,
            password,
        })
    }
}

impl Fetcher for ImapFetcher {
    fn fetch_mailbox(&self, mailbox: &str) -> Result<Vec<Item>> {
        imap::fetch_imap(
            &self.host,
            self.port,
            &self.username,
            &self.password,
            mailbox,
        )
    }

    fn list_mailboxes(&self) -> Result<Vec<String>> {
        imap::list_mailboxes(&self.host, self.port, &self.username, &self.password)
    }
}

#[cfg(test)]
pub mod mock {
    use super::*;

    pub struct MockFetcher {
        pub items: Vec<Item>,
        pub mailboxes: Vec<String>,
    }

    impl MockFetcher {
        pub fn new(items: Vec<Item>, mailboxes: Vec<String>) -> Self {
            Self { items, mailboxes }
        }
    }

    impl Fetcher for MockFetcher {
        fn fetch_mailbox(&self, _mailbox: &str) -> Result<Vec<Item>> {
            Ok(self.items.clone())
        }

        fn list_mailboxes(&self) -> Result<Vec<String>> {
            Ok(self.mailboxes.clone())
        }
    }
}
