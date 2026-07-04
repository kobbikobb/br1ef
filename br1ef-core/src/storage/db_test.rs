use super::SqliteStorage;
use crate::Item;
use crate::storage::Storage;

fn make_item(id: &str) -> Item {
    Item {
        id: id.into(),
        title: "t".into(),
        from: "f".into(),
        body: "b".into(),
        source: "s".into(),
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
