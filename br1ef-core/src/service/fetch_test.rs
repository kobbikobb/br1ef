use std::collections::HashMap;

use super::*;
use crate::fetcher::mock::MockFetcher;
use crate::fetcher::Fetcher;
use crate::storage::InMemoryStorage;

/// A fetcher that returns different items per mailbox.
/// Useful for testing cross-mailbox collection and dedup.
struct MailboxMapFetcher(HashMap<String, Vec<Item>>);

impl Fetcher for MailboxMapFetcher {
    fn fetch_mailbox(&self, mailbox: &str) -> Result<Vec<Item>> {
        Ok(self.0.get(mailbox).cloned().unwrap_or_default())
    }
    fn list_mailboxes(&self) -> Result<Vec<String>> {
        Ok(self.0.keys().cloned().collect())
    }
}

fn item(id: &str, title: &str) -> Item {
    Item {
        id: id.into(),
        title: title.into(),
        from: "alice@example.com".into(),
        body: "body".into(),
        source: "imap".into(),
        urgent: false,
    }
}

#[test]
fn fetch_items_defaults_to_inbox_when_no_mailboxes_configured() {
    let mut storage = InMemoryStorage::new();
    let fetcher = MockFetcher::new(
        vec![Item {
            id: "1".into(),
            title: "Hello".into(),
            from: "alice@example.com".into(),
            body: "body".into(),
            source: "imap".into(),
            urgent: false,
        }],
        vec![],
    );

    let result = fetch_items(&mut storage, &fetcher);

    assert!(result.is_ok());
    let items = result.unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "Hello");
}

#[test]
fn fetch_items_deduplicates_across_mailboxes() {
    let mut storage = InMemoryStorage::new();
    storage
        .set_selected_mailboxes(&["INBOX".into(), "Work".into()])
        .unwrap();
    let fetcher = MockFetcher::new(
        vec![Item {
            id: "dup".into(),
            title: "Meeting".into(),
            from: "boss@example.com".into(),
            body: "body".into(),
            source: "imap".into(),
            urgent: false,
        }],
        vec![],
    );

    let result = fetch_items(&mut storage, &fetcher);

    assert!(result.is_ok());
    let items = result.unwrap();
    assert_eq!(items.len(), 1);
}

#[test]
fn fetch_items_propagates_fetcher_error() {
    use anyhow::bail;
    struct BrokenFetcher;

    impl Fetcher for BrokenFetcher {
        fn fetch_mailbox(&self, _mailbox: &str) -> Result<Vec<Item>> {
            bail!("network error")
        }
        fn list_mailboxes(&self) -> Result<Vec<String>> {
            Ok(vec![])
        }
    }

    let mut storage = InMemoryStorage::new();
    storage.set_selected_mailboxes(&["INBOX".into()]).unwrap();

    let result = fetch_items(&mut storage, &BrokenFetcher);

    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("failed to fetch from \"INBOX\""),
        "expected context wrapping, got: {err}"
    );
}

#[test]
fn fetch_items_stores_to_storage() {
    let mut storage = InMemoryStorage::new();
    let fetcher = MockFetcher::new(
        vec![Item {
            id: "1".into(),
            title: "Saved".into(),
            from: "alice@example.com".into(),
            body: "body".into(),
            source: "imap".into(),
            urgent: false,
        }],
        vec![],
    );

    fetch_items(&mut storage, &fetcher).unwrap();

    let stored = storage.get_items().unwrap();
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].title, "Saved");
}

#[test]
fn fetch_items_collects_all_unique_items_from_multiple_mailboxes() {
    let mut storage = InMemoryStorage::new();
    storage
        .set_selected_mailboxes(&["INBOX".into(), "Work".into()])
        .unwrap();

    let fetcher = MailboxMapFetcher(HashMap::from([
        ("INBOX".into(), vec![item("1", "Inbox Mail"), item("2", "Inbox Note")]),
        ("Work".into(), vec![item("3", "Work Report")]),
    ]));

    let result = fetch_items(&mut storage, &fetcher);

    assert!(result.is_ok());
    let items = result.unwrap();
    assert_eq!(items.len(), 3);
    let mut ids: Vec<&str> = items.iter().map(|i| i.id.as_str()).collect();
    ids.sort();
    assert_eq!(ids, vec!["1", "2", "3"]);

    let stored = storage.get_items().unwrap();
    assert_eq!(stored.len(), 3);
}

#[test]
fn fetch_items_deduplicates_partial_overlap_across_mailboxes() {
    let mut storage = InMemoryStorage::new();
    storage
        .set_selected_mailboxes(&["INBOX".into(), "Work".into()])
        .unwrap();

    let fetcher = MailboxMapFetcher(HashMap::from([
        ("INBOX".into(), vec![item("1", "Shared"), item("2", "Inbox Only")]),
        ("Work".into(), vec![item("1", "Shared"), item("3", "Work Only")]),
    ]));

    let result = fetch_items(&mut storage, &fetcher);

    assert!(result.is_ok());
    let items = result.unwrap();
    assert_eq!(items.len(), 3);
    let mut ids: Vec<&str> = items.iter().map(|i| i.id.as_str()).collect();
    ids.sort();
    assert_eq!(ids, vec!["1", "2", "3"]);

    let stored = storage.get_items().unwrap();
    assert_eq!(stored.len(), 3);
}

#[test]
fn fetch_items_returns_empty_when_no_mailboxes_have_items() {
    let mut storage = InMemoryStorage::new();
    storage
        .set_selected_mailboxes(&["INBOX".into(), "Empty".into()])
        .unwrap();

    let fetcher = MailboxMapFetcher(HashMap::from([
        ("INBOX".into(), vec![]),
        ("Empty".into(), vec![]),
    ]));

    let result = fetch_items(&mut storage, &fetcher);

    assert!(result.is_ok());
    let items = result.unwrap();
    assert!(items.is_empty());

    let stored = storage.get_items().unwrap();
    assert!(stored.is_empty());
}
