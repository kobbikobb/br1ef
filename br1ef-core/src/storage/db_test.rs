use super::SqliteStorage;
use crate::Item;
use crate::storage::Storage;

fn make_item(id: &str) -> Item {
    make_item_with_source(id, "s")
}

fn make_item_with_source(id: &str, source: &str) -> Item {
    Item {
        id: id.into(),
        title: "t".into(),
        from: "f".into(),
        body: "b".into(),
        source: source.into(),
        urgent: false,
    }
}

fn new_store() -> SqliteStorage {
    SqliteStorage::new(":memory:").unwrap()
}

#[test]
fn store_items_accumulates_new_ids() {
    let mut s = new_store();

    s.store_items(&[make_item("a"), make_item("b")]).unwrap();

    let items = s.get_items().unwrap();
    let mut ids: Vec<&str> = items.iter().map(|i| i.id.as_str()).collect();
    ids.sort();
    assert_eq!(ids, vec!["a", "b"]);
}

#[test]
fn store_items_skips_duplicate_ids() {
    let mut s = new_store();
    s.store_items(&[make_item("a")]).unwrap();

    s.store_items(&[make_item("a"), make_item("b")]).unwrap();

    let items = s.get_items().unwrap();
    let mut ids: Vec<&str> = items.iter().map(|i| i.id.as_str()).collect();
    ids.sort();
    assert_eq!(ids, vec!["a", "b"]);
}

#[test]
fn store_items_accumulates_across_multiple_calls() {
    let mut s = new_store();

    s.store_items(&[make_item("a")]).unwrap();
    s.store_items(&[make_item("b")]).unwrap();
    s.store_items(&[make_item("c")]).unwrap();

    let items = s.get_items().unwrap();
    let mut ids: Vec<&str> = items.iter().map(|i| i.id.as_str()).collect();
    ids.sort();
    assert_eq!(ids, vec!["a", "b", "c"]);
}

#[test]
fn store_items_preserves_original_data_on_duplicate() {
    let mut s = new_store();
    let mut item = make_item("x");
    item.title = "original".into();
    s.store_items(&[item]).unwrap();

    let mut duplicate = make_item("x");
    duplicate.title = "overwritten".into();
    s.store_items(&[duplicate]).unwrap();

    let items = s.get_items().unwrap();
    let stored = items.iter().find(|i| i.id == "x").unwrap();
    assert_eq!(
        stored.title, "original",
        "INSERT OR IGNORE should keep original"
    );
}

#[test]
fn store_items_empty_does_not_clear_existing() {
    let mut s = new_store();
    s.store_items(&[make_item("a")]).unwrap();

    s.store_items(&[]).unwrap();

    let items = s.get_items().unwrap();
    assert_eq!(items.len(), 1);
}

#[test]
fn counts_empty_when_no_items() {
    let s = new_store();

    let counts = s.get_item_counts_by_source().unwrap();

    assert!(counts.is_empty());
}

#[test]
fn counts_single_source_single_item() {
    let mut s = new_store();
    s.store_items(&[make_item_with_source("a", "inbox")])
        .unwrap();

    let counts = s.get_item_counts_by_source().unwrap();

    assert_eq!(counts, vec![("inbox".to_string(), 1)]);
}

#[test]
fn counts_single_source_multiple_items() {
    let mut s = new_store();
    s.store_items(&[
        make_item_with_source("a", "inbox"),
        make_item_with_source("b", "inbox"),
        make_item_with_source("c", "inbox"),
    ])
    .unwrap();

    let counts = s.get_item_counts_by_source().unwrap();

    assert_eq!(counts, vec![("inbox".to_string(), 3)]);
}

#[test]
fn counts_multiple_sources() {
    let mut s = new_store();
    s.store_items(&[
        make_item_with_source("a", "social"),
        make_item_with_source("b", "updates"),
        make_item_with_source("c", "social"),
        make_item_with_source("d", "forums"),
        make_item_with_source("e", "updates"),
        make_item_with_source("f", "updates"),
    ])
    .unwrap();

    let counts = s.get_item_counts_by_source().unwrap();

    assert_eq!(
        counts,
        vec![
            ("forums".to_string(), 1),
            ("social".to_string(), 2),
            ("updates".to_string(), 3),
        ]
    );
}

#[test]
fn counts_across_multiple_store_calls() {
    let mut s = new_store();
    s.store_items(&[make_item_with_source("a", "social")])
        .unwrap();
    s.store_items(&[make_item_with_source("b", "updates")])
        .unwrap();
    s.store_items(&[make_item_with_source("c", "social")])
        .unwrap();

    let counts = s.get_item_counts_by_source().unwrap();

    assert_eq!(
        counts,
        vec![("social".to_string(), 2), ("updates".to_string(), 1),]
    );
}

#[test]
fn counts_empty_after_clear() {
    let mut s = new_store();
    s.store_items(&[
        make_item_with_source("a", "social"),
        make_item_with_source("b", "updates"),
    ])
    .unwrap();

    s.clear().unwrap();

    let counts = s.get_item_counts_by_source().unwrap();
    assert!(counts.is_empty());
}

#[test]
fn counts_dedup_does_not_affect_counts() {
    let mut s = new_store();
    let mut item = make_item_with_source("x", "social");
    item.title = "original".into();
    s.store_items(&[item]).unwrap();

    let dup = Item {
        source: "updates".into(),
        ..make_item_with_source("x", "social")
    };
    s.store_items(&[dup]).unwrap();

    let counts = s.get_item_counts_by_source().unwrap();

    assert_eq!(counts, vec![("social".to_string(), 1)]);
}
