use super::*;
use crate::fetcher::mock::MockFetcher;
use crate::storage::InMemoryStorage;

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
