use super::delete_items;
use crate::Item;
use crate::Source;
use crate::storage::InMemoryStorage;
use crate::storage::Storage;

fn item(id: &str, source: Source) -> Item {
    Item {
        id: id.into(),
        title: "t".into(),
        from: "f".into(),
        body: "b".into(),
        source,
        mailbox: String::new(),
        urgent: false,
    }
}

#[test]
fn delete_items_returns_zero_when_empty() {
    let mut storage = InMemoryStorage::new();

    let n = delete_items(&mut storage).unwrap();

    assert_eq!(n, 0);
    assert!(storage.get_items().unwrap().is_empty());
}

#[test]
fn delete_items_removes_all_items() {
    let mut storage = InMemoryStorage::new();
    storage
        .store_items(&[
            item("1", Source::Imap),
            item("2", Source::Imap),
            item("3", Source::Imap),
        ])
        .unwrap();

    let n = delete_items(&mut storage).unwrap();

    assert_eq!(n, 3);
    assert!(storage.get_items().unwrap().is_empty());
}

#[test]
fn delete_items_removes_items_from_multiple_sources() {
    let mut storage = InMemoryStorage::new();
    storage
        .store_items(&[
            item("a", Source::Imap),
            item("b", Source::Imap),
            item("c", Source::Imap),
        ])
        .unwrap();

    let n = delete_items(&mut storage).unwrap();

    assert_eq!(n, 3);
    assert!(storage.get_items().unwrap().is_empty());
}

#[test]
fn delete_items_second_call_returns_zero() {
    let mut storage = InMemoryStorage::new();
    storage
        .store_items(&[
            item("1", Source::Imap),
            item("2", Source::Imap),
            item("3", Source::Imap),
        ])
        .unwrap();

    let n1 = delete_items(&mut storage).unwrap();
    let n2 = delete_items(&mut storage).unwrap();

    assert_eq!(n1, 3);
    assert_eq!(n2, 0);
    assert!(storage.get_items().unwrap().is_empty());
}
