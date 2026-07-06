use anyhow::Result;

use crate::Item;
use crate::agent::Agent;
use crate::storage::InMemoryStorage;
use crate::storage::Storage;

use super::{build_digest, digest_items};

struct MockAgent {
    should_fail: bool,
    summary: String,
}

impl Agent for MockAgent {
    fn summarize_items(&self, _items: &[Item]) -> Result<String> {
        if self.should_fail {
            Err(anyhow::anyhow!("agent error"))
        } else {
            Ok(self.summary.clone())
        }
    }
}

fn make_item(id: &str, source: &str, body: &str) -> Item {
    Item {
        id: id.to_string(),
        title: "test".to_string(),
        from: "alice@test.com".to_string(),
        body: body.to_string(),
        source: source.to_string(),
        mailbox: "".into(),
        urgent: false,
    }
}

fn make_item_from(id: &str, from: &str, title: &str) -> Item {
    Item {
        id: id.to_string(),
        title: title.to_string(),
        from: from.to_string(),
        body: "body".to_string(),
        source: "imap".to_string(),
        mailbox: "".into(),
        urgent: false,
    }
}

#[test]
fn digest_items_empty_returns_placeholder_summary() {
    let mut storage = InMemoryStorage::new();
    let agent = MockAgent {
        should_fail: false,
        summary: String::new(),
    };

    digest_items(&mut storage, &agent).unwrap();

    let digest = storage.get_digest().unwrap().unwrap();
    assert_eq!(digest.total_items, 0);
    assert_eq!(digest.summary, "No items to summarize.");
    assert!(digest.by_source.is_empty());
}

#[test]
fn digest_items_with_items_stores_digest_with_correct_summary() {
    let mut storage = InMemoryStorage::new();
    storage
        .store_items(&[
            Item {
                id: "1".into(),
                title: "Meeting".into(),
                from: "alice@a".into(),
                body: "hello world".into(),
                source: "imap".into(),
                mailbox: "".into(),
                urgent: false,
            },
            Item {
                id: "2".into(),
                title: "Lunch".into(),
                from: "bob@b".into(),
                body: "foo bar baz".into(),
                source: "imap".into(),
                mailbox: "".into(),
                urgent: false,
            },
        ])
        .unwrap();
    let agent = MockAgent {
        should_fail: false,
        summary: "Key highlights: none.".to_string(),
    };

    digest_items(&mut storage, &agent).unwrap();

    let digest = storage.get_digest().unwrap().unwrap();
    assert_eq!(digest.total_items, 2);
    assert_eq!(digest.summary, "Key highlights: none.");
    assert_eq!(digest.by_source, vec![("imap".to_string(), 2)]);
}

#[test]
fn digest_items_agent_error_propagates() {
    let mut storage = InMemoryStorage::new();
    storage
        .store_items(&[make_item("1", "imap", "hello")])
        .unwrap();
    let agent = MockAgent {
        should_fail: true,
        summary: String::new(),
    };

    let result = digest_items(&mut storage, &agent);

    assert!(result.is_err());
    assert!(storage.get_digest().unwrap().is_none());
}

#[test]
fn digest_items_by_source_aggregates_multiple_sources() {
    let mut storage = InMemoryStorage::new();
    let title = "unique";
    storage
        .store_items(&[
            Item {
                id: "1".into(),
                title: format!("{title}-a"),
                from: "alice@a".into(),
                body: "a".into(),
                source: "imap".into(),
                mailbox: "".into(),
                urgent: false,
            },
            Item {
                id: "2".into(),
                title: format!("{title}-b"),
                from: "alice@b".into(),
                body: "b".into(),
                source: "slack".into(),
                mailbox: "".into(),
                urgent: false,
            },
            Item {
                id: "3".into(),
                title: format!("{title}-c"),
                from: "alice@c".into(),
                body: "c".into(),
                source: "imap".into(),
                mailbox: "".into(),
                urgent: false,
            },
            Item {
                id: "4".into(),
                title: format!("{title}-d"),
                from: "alice@d".into(),
                body: "d".into(),
                source: "slack".into(),
                mailbox: "".into(),
                urgent: false,
            },
        ])
        .unwrap();
    let agent = MockAgent {
        should_fail: false,
        summary: "summary".to_string(),
    };

    digest_items(&mut storage, &agent).unwrap();

    let digest = storage.get_digest().unwrap().unwrap();
    assert_eq!(digest.total_items, 4);
    let sources: std::collections::HashMap<_, _> = digest.by_source.into_iter().collect();
    assert_eq!(sources.get("imap"), Some(&2));
    assert_eq!(sources.get("slack"), Some(&2));
}

#[test]
fn digest_items_all_noise_short_circuits_without_calling_agent() {
    let mut storage = InMemoryStorage::new();
    storage
        .store_items(&[
            make_item_from("1", "notifications@linkedin.com", "New message"),
            make_item_from("2", "newsletter@substack.com", "Weekly Issue"),
        ])
        .unwrap();
    let agent = MockAgent {
        should_fail: false,
        summary: "should not be called".to_string(),
    };

    digest_items(&mut storage, &agent).unwrap();

    let digest = storage.get_digest().unwrap().unwrap();
    assert_eq!(digest.total_items, 2);
    assert_eq!(digest.summary, "Nothing needs attention today.");
    assert_eq!(digest.by_source, vec![("imap".to_string(), 2)]);
}

#[test]
fn digest_items_mixed_noise_and_clean_only_passes_clean_to_agent() {
    let mut storage = InMemoryStorage::new();
    storage
        .store_items(&[
            make_item_from("1", "mom@family.com", "Dinner tonight?"),
            make_item_from("2", "notifications@linkedin.com", "Connection request"),
            make_item_from("3", "dad@family.com", "Call me"),
        ])
        .unwrap();
    let agent = MockAgent {
        should_fail: false,
        summary: "Family matters".to_string(),
    };

    digest_items(&mut storage, &agent).unwrap();

    let digest = storage.get_digest().unwrap().unwrap();
    assert_eq!(digest.total_items, 3);
    assert_eq!(digest.summary, "Family matters");
    assert_eq!(digest.by_source, vec![("imap".to_string(), 3)]);
}

#[test]
fn digest_items_all_noise_still_counts_items_in_stats() {
    let mut storage = InMemoryStorage::new();
    storage
        .store_items(&[
            make_item_from("1", "noreply@updates.co", "Verify your account"),
            make_item_from("2", "marketing@store.com", "Big sale"),
        ])
        .unwrap();
    let agent = MockAgent {
        should_fail: false,
        summary: "irrelevant".to_string(),
    };

    digest_items(&mut storage, &agent).unwrap();

    let digest = storage.get_digest().unwrap().unwrap();
    assert_eq!(digest.total_items, 2);
    assert!(digest.summary.contains("Nothing needs attention"));
}

#[test]
fn build_digest_empty_items() {
    let agent = MockAgent {
        should_fail: false,
        summary: String::new(),
    };

    let digest = build_digest(vec![], &agent).unwrap();

    assert_eq!(digest.total_items, 0);
    assert!(digest.by_source.is_empty());
    assert_eq!(digest.summary, "No items to summarize.");
}

#[test]
fn build_digest_all_noise() {
    let agent = MockAgent {
        should_fail: false,
        summary: String::new(),
    };
    let items = vec![
        make_item_from("1", "notifications@linkedin.com", "New message"),
        make_item_from("2", "newsletter@substack.com", "Weekly Issue"),
    ];

    let digest = build_digest(items, &agent).unwrap();

    assert_eq!(digest.total_items, 2);
    assert_eq!(digest.summary, "Nothing needs attention today.");
}

#[test]
fn build_digest_aggregates_by_source() {
    let agent = MockAgent {
        should_fail: false,
        summary: "summary".to_string(),
    };
    let items = vec![
        Item {
            id: "1".into(),
            title: "Work".into(),
            from: "boss@example.com".into(),
            body: "body".into(),
            source: "imap".into(),
            mailbox: "".into(),
            urgent: false,
        },
        Item {
            id: "2".into(),
            title: "PR".into(),
            from: "github@example.com".into(),
            body: "body".into(),
            source: "imap".into(),
            mailbox: "".into(),
            urgent: false,
        },
        Item {
            id: "3".into(),
            title: "Channel".into(),
            from: "slack@example.com".into(),
            body: "body".into(),
            source: "slack".into(),
            mailbox: "".into(),
            urgent: false,
        },
    ];

    let digest = build_digest(items, &agent).unwrap();

    assert_eq!(digest.total_items, 3);
    assert_eq!(digest.by_source.len(), 2);
    assert!(digest.by_source.contains(&("imap".to_string(), 2)));
    assert!(digest.by_source.contains(&("slack".to_string(), 1)));
}
