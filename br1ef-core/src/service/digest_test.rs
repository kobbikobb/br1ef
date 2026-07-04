use anyhow::Result;

use crate::agent::Agent;
use crate::storage::InMemoryStorage;
use crate::storage::Storage;
use crate::Item;

use super::digest_items;

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
            Item { id: "1".into(), title: "Meeting".into(), from: "alice@a".into(), body: "hello world".into(), source: "imap".into(), urgent: false },
            Item { id: "2".into(), title: "Lunch".into(), from: "bob@b".into(), body: "foo bar baz".into(), source: "imap".into(), urgent: false },
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
            Item { id: "1".into(), title: format!("{title}-a"), from: "alice@a".into(), body: "a".into(), source: "imap".into(), urgent: false },
            Item { id: "2".into(), title: format!("{title}-b"), from: "alice@b".into(), body: "b".into(), source: "slack".into(), urgent: false },
            Item { id: "3".into(), title: format!("{title}-c"), from: "alice@c".into(), body: "c".into(), source: "imap".into(), urgent: false },
            Item { id: "4".into(), title: format!("{title}-d"), from: "alice@d".into(), body: "d".into(), source: "slack".into(), urgent: false },
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
