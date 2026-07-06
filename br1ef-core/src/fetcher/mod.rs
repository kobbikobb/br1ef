mod imap;

pub use imap::GMAIL_CATEGORY_PREFIX;

use anyhow::Result;

use crate::Item;

pub trait Fetcher {
    fn fetch_mailbox(&self, mailbox: &str) -> Result<Vec<Item>>;
    fn list_mailboxes(&self) -> Result<Vec<String>>;

    fn suggested_mailboxes(&self) -> Result<Vec<String>> {
        Ok(vec!["INBOX".to_string()])
    }
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

    fn suggested_mailboxes(&self) -> Result<Vec<String>> {
        let all = self.list_mailboxes()?;
        let mut mailboxes: Vec<String> = all
            .into_iter()
            .filter(|m| m.starts_with(imap::GMAIL_CATEGORY_PREFIX))
            .collect();
        mailboxes.insert(0, "INBOX".into());
        Ok(mailboxes)
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

        fn suggested_mailboxes(&self) -> Result<Vec<String>> {
            if self.mailboxes.is_empty() {
                Ok(vec!["INBOX".to_string()])
            } else {
                Ok(self.mailboxes.clone())
            }
        }
    }
}
