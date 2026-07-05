use std::collections::HashMap;

use super::*;
use crate::fetcher::Fetcher;
use crate::fetcher::mock::MockFetcher;
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
        mailbox: "".into(),
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
            mailbox: "".into(),
            urgent: false,
        }],
        vec![],
    );

    let result = fetch_items(&mut storage, &fetcher);

    assert!(result.is_ok());
    let items = result.unwrap().items;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "Hello");
}

#[test]
fn fetch_items_auto_selects_categories_when_no_mailboxes_configured() {
    let mut storage = InMemoryStorage::new();
    let fetcher = MailboxMapFetcher(HashMap::from([
        (
            "INBOX".into(),
            vec![item("1", "Inbox Mail")],
        ),
        (
            "@@CATEGORY@@/Social".into(),
            vec![item("2", "Social Post")],
        ),
        (
            "@@CATEGORY@@/Updates".into(),
            vec![item("3", "Update Notice")],
        ),
    ]));

    let result = fetch_items(&mut storage, &fetcher);

    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.items.len(), 3);

    let mut mailbox_names: Vec<&str> =
        result.per_mailbox.iter().map(|m| m.name.as_str()).collect();
    mailbox_names.sort();
    assert_eq!(mailbox_names, vec!["@@CATEGORY@@/Social", "@@CATEGORY@@/Updates", "INBOX"]);
}

#[test]
fn fetch_items_falls_back_to_inbox_when_list_mailboxes_fails() {
    use anyhow::bail;

    struct BrokenListFetcher;

    impl Fetcher for BrokenListFetcher {
        fn fetch_mailbox(&self, _mailbox: &str) -> Result<Vec<Item>> {
            Ok(vec![item("1", "Only Item")])
        }
        fn list_mailboxes(&self) -> Result<Vec<String>> {
            bail!("network unavailable")
        }
    }

    let mut storage = InMemoryStorage::new();

    let result = fetch_items(&mut storage, &BrokenListFetcher);

    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.items.len(), 1);
    assert_eq!(result.per_mailbox.len(), 1);
    assert_eq!(result.per_mailbox[0].name, "INBOX");
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
            mailbox: "".into(),
            urgent: false,
        }],
        vec![],
    );

    let result = fetch_items(&mut storage, &fetcher);

    assert!(result.is_ok());
    assert_eq!(result.unwrap().items.len(), 1);
}

#[test]
fn fetch_items_skips_mailbox_on_fetch_error() {
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

    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.items.is_empty());
    assert!(result.per_mailbox.is_empty());
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
            mailbox: "".into(),
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
        (
            "INBOX".into(),
            vec![item("1", "Inbox Mail"), item("2", "Inbox Note")],
        ),
        ("Work".into(), vec![item("3", "Work Report")]),
    ]));

    let result = fetch_items(&mut storage, &fetcher);

    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.items.len(), 3);
    let mut ids: Vec<&str> = result.items.iter().map(|i| i.id.as_str()).collect();
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
        (
            "INBOX".into(),
            vec![item("1", "Shared"), item("2", "Inbox Only")],
        ),
        (
            "Work".into(),
            vec![item("1", "Shared"), item("3", "Work Only")],
        ),
    ]));

    let result = fetch_items(&mut storage, &fetcher);

    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.items.len(), 3);
    let mut ids: Vec<&str> = result.items.iter().map(|i| i.id.as_str()).collect();
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
    let result = result.unwrap();
    assert!(result.items.is_empty());

    let stored = storage.get_items().unwrap();
    assert!(stored.is_empty());
}

#[test]
fn fetch_items_reports_stats_per_mailbox() {
    let mut storage = InMemoryStorage::new();
    storage
        .set_selected_mailboxes(&["INBOX".into(), "Work".into()])
        .unwrap();

    let fetcher = MailboxMapFetcher(HashMap::from([
        (
            "INBOX".into(),
            vec![item("1", "Shared"), item("2", "Inbox Only")],
        ),
        (
            "Work".into(),
            vec![item("1", "Shared"), item("3", "Work Only")],
        ),
    ]));

    let result = fetch_items(&mut storage, &fetcher);

    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.per_mailbox.len(), 2);
    assert_eq!(result.per_mailbox[0].name, "INBOX");
    assert_eq!(result.per_mailbox[0].total, 2);
    assert_eq!(result.per_mailbox[0].new, 2);
    assert_eq!(result.per_mailbox[1].name, "Work");
    assert_eq!(result.per_mailbox[1].total, 2);
    assert_eq!(result.per_mailbox[1].new, 1);
}

#[test]
fn fetch_items_includes_category_mailboxes_when_inbox_selected() {
    let mut storage = InMemoryStorage::new();
    storage
        .set_selected_mailboxes(&["INBOX".into(), "@@CATEGORY@@/Social".into()])
        .unwrap();

    let fetcher = MailboxMapFetcher(HashMap::from([
        (
            "INBOX".into(),
            vec![item("1", "Inbox Mail"), item("2", "Inbox Note")],
        ),
        (
            "@@CATEGORY@@/Social".into(),
            vec![item("3", "Social Post That Is Not In Inbox")],
        ),
    ]));

    let result = fetch_items(&mut storage, &fetcher);

    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.items.len(), 3);
    assert_eq!(result.per_mailbox.len(), 2);
    assert_eq!(result.per_mailbox[1].name, "@@CATEGORY@@/Social");
    assert_eq!(result.per_mailbox[1].total, 1);
    assert_eq!(result.per_mailbox[1].new, 1);

    let stored = storage.get_items().unwrap();
    assert_eq!(stored.len(), 3);
}

#[test]
fn fetch_items_deduplicates_category_with_inbox() {
    let mut storage = InMemoryStorage::new();
    storage
        .set_selected_mailboxes(&["INBOX".into(), "@@CATEGORY@@/Social".into()])
        .unwrap();

    let fetcher = MailboxMapFetcher(HashMap::from([
        ("INBOX".into(), vec![item("1", "Inbox Also In Social")]),
        (
            "@@CATEGORY@@/Social".into(),
            vec![item("1", "Inbox Also In Social")],
        ),
    ]));

    let result = fetch_items(&mut storage, &fetcher);

    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.items.len(), 1);
    assert_eq!(result.per_mailbox.len(), 2);
    assert_eq!(result.per_mailbox[0].new, 1);
    assert_eq!(result.per_mailbox[1].new, 0);
}

#[test]
fn fetch_items_stamps_mailbox_on_each_item() {
    let mut storage = InMemoryStorage::new();
    storage
        .set_selected_mailboxes(&["INBOX".into(), "Work".into()])
        .unwrap();

    let fetcher = MailboxMapFetcher(HashMap::from([
        (
            "INBOX".into(),
            vec![item("1", "Inbox Mail"), item("2", "Inbox Note")],
        ),
        ("Work".into(), vec![item("3", "Work Report")]),
    ]));

    let result = fetch_items(&mut storage, &fetcher).unwrap();

    for item in &result.items {
        match item.id.as_str() {
            "1" | "2" => assert_eq!(item.mailbox, "INBOX"),
            "3" => assert_eq!(item.mailbox, "Work"),
            _ => panic!("unexpected item id"),
        }
    }

    let stored = storage.get_items().unwrap();
    for item in &stored {
        match item.id.as_str() {
            "1" | "2" => assert_eq!(item.mailbox, "INBOX"),
            "3" => assert_eq!(item.mailbox, "Work"),
            _ => panic!("unexpected stored item id"),
        }
    }
}

#[test]
fn fetch_items_keeps_first_seen_mailbox_on_dedup() {
    let mut storage = InMemoryStorage::new();
    storage
        .set_selected_mailboxes(&["INBOX".into(), "Work".into()])
        .unwrap();

    let fetcher = MailboxMapFetcher(HashMap::from([
        ("INBOX".into(), vec![item("1", "Shared")]),
        ("Work".into(), vec![item("1", "Shared")]),
    ]));

    let result = fetch_items(&mut storage, &fetcher).unwrap();

    assert_eq!(result.items.len(), 1);
    assert_eq!(result.items[0].mailbox, "INBOX");
}
