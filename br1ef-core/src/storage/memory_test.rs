use super::InMemoryStorage;
use crate::Item;
use crate::Source;
use crate::storage::Storage;

fn make_item(id: &str) -> Item {
    make_item_with_source(id, Source::Imap)
}

fn make_item_with_source(id: &str, source: Source) -> Item {
    Item {
        id: id.into(),
        title: "t".into(),
        from: "f".into(),
        body: "b".into(),
        source,
        mailbox: "".into(),
        urgent: false,
    }
}

#[test]
fn store_items_accumulates_new_ids() {
    let mut s = InMemoryStorage::new();

    s.store_items(&[make_item("a"), make_item("b")]).unwrap();

    let items = s.get_items().unwrap();
    let ids: Vec<&str> = items.iter().map(|i| i.id.as_str()).collect();
    assert_eq!(ids, vec!["a", "b"]);
}

#[test]
fn store_items_skips_duplicate_ids() {
    let mut s = InMemoryStorage::new();
    s.store_items(&[make_item("a")]).unwrap();

    s.store_items(&[make_item("a"), make_item("b")]).unwrap();

    let items = s.get_items().unwrap();
    let ids: Vec<&str> = items.iter().map(|i| i.id.as_str()).collect();
    assert_eq!(
        ids,
        vec!["a", "b"],
        "should keep first occurrence of 'a', append 'b'"
    );
}

#[test]
fn store_items_accumulates_across_multiple_calls() {
    let mut s = InMemoryStorage::new();

    s.store_items(&[make_item("a")]).unwrap();
    s.store_items(&[make_item("b")]).unwrap();
    s.store_items(&[make_item("c")]).unwrap();

    let items = s.get_items().unwrap();
    let ids: Vec<&str> = items.iter().map(|i| i.id.as_str()).collect();
    assert_eq!(ids, vec!["a", "b", "c"]);
}

#[test]
fn store_items_preserves_original_data_on_duplicate() {
    let mut s = InMemoryStorage::new();
    let mut item = make_item("x");
    item.title = "original".into();
    s.store_items(&[item]).unwrap();

    let mut duplicate = make_item("x");
    duplicate.title = "overwritten".into();
    s.store_items(&[duplicate]).unwrap();

    let items = s.get_items().unwrap();
    assert_eq!(
        items[0].title, "original",
        "should NOT overwrite on duplicate"
    );
}

#[test]
fn store_items_empty_does_not_clear_existing() {
    let mut s = InMemoryStorage::new();
    s.store_items(&[make_item("a")]).unwrap();

    s.store_items(&[]).unwrap();

    let items = s.get_items().unwrap();
    assert_eq!(items.len(), 1);
}

#[test]
fn counts_empty_when_no_items() {
    let s = InMemoryStorage::new();

    let counts = s.get_item_counts_by_source().unwrap();

    assert!(counts.is_empty());
}

#[test]
fn counts_single_source_single_item() {
    let mut s = InMemoryStorage::new();
    s.store_items(&[make_item_with_source("a", Source::Imap)])
        .unwrap();

    let counts = s.get_item_counts_by_source().unwrap();

    assert_eq!(counts, vec![("imap".to_string(), 1)]);
}

#[test]
fn counts_single_source_multiple_items() {
    let mut s = InMemoryStorage::new();
    s.store_items(&[
        make_item_with_source("a", Source::Imap),
        make_item_with_source("b", Source::Imap),
        make_item_with_source("c", Source::Imap),
    ])
    .unwrap();

    let counts = s.get_item_counts_by_source().unwrap();

    assert_eq!(counts, vec![("imap".to_string(), 3)]);
}

#[test]
fn counts_multiple_sources() {
    let mut s = InMemoryStorage::new();
    s.store_items(&[
        make_item_with_source("a", Source::Imap),
        make_item_with_source("b", Source::Imap),
        make_item_with_source("c", Source::Imap),
        make_item_with_source("d", Source::Imap),
        make_item_with_source("e", Source::Imap),
        make_item_with_source("f", Source::Imap),
    ])
    .unwrap();

    let counts = s.get_item_counts_by_source().unwrap();

    assert_eq!(counts, vec![("imap".to_string(), 6)]);
}

#[test]
fn counts_across_multiple_store_calls() {
    let mut s = InMemoryStorage::new();
    s.store_items(&[make_item_with_source("a", Source::Imap)])
        .unwrap();
    s.store_items(&[make_item_with_source("b", Source::Imap)])
        .unwrap();
    s.store_items(&[make_item_with_source("c", Source::Imap)])
        .unwrap();

    let counts = s.get_item_counts_by_source().unwrap();

    assert_eq!(counts, vec![("imap".to_string(), 3)]);
}

#[test]
fn counts_empty_after_clear() {
    let mut s = InMemoryStorage::new();
    s.store_items(&[
        make_item_with_source("a", Source::Imap),
        make_item_with_source("b", Source::Imap),
    ])
    .unwrap();

    s.clear().unwrap();

    let counts = s.get_item_counts_by_source().unwrap();
    assert!(counts.is_empty());
}

#[test]
fn counts_dedup_does_not_affect_counts() {
    let mut s = InMemoryStorage::new();
    let mut item = make_item_with_source("x", Source::Imap);
    item.title = "original".into();
    s.store_items(&[item]).unwrap();

    let dup = Item {
        source: Source::Imap,
        ..make_item_with_source("x", Source::Imap)
    };
    s.store_items(&[dup]).unwrap();

    let counts = s.get_item_counts_by_source().unwrap();

    assert_eq!(counts, vec![("imap".to_string(), 1)]);
}
