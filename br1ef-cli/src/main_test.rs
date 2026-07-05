use super::*;
use br1ef_core::Item;
use br1ef_core::storage::InMemoryStorage;
use br1ef_core::storage::Storage;

fn storage_with(items: &[(&str, &str, &str, &str)]) -> InMemoryStorage {
    let mut s = InMemoryStorage::new();
    let stored: Vec<Item> = items
        .iter()
        .map(|&(id, source, title, mailbox)| Item {
            id: id.into(),
            title: title.into(),
            from: "sender".into(),
            body: "body".into(),
            source: source.into(),
            mailbox: mailbox.into(),
            urgent: false,
        })
        .collect();
    s.store_items(&stored).unwrap();
    s
}

fn run<F>(f: F) -> String
where
    F: FnOnce(&mut Vec<u8>) -> Result<()>,
{
    let mut buf = Vec::new();
    f(&mut buf).unwrap();
    String::from_utf8(buf).unwrap()
}

#[test]
fn count_items_empty() {
    let storage = InMemoryStorage::new();

    let output = run(|w| cmd_count_items(&storage, w));

    assert_eq!(output, "No items stored. Run `br1ef fetch` first.\n");
}

#[test]
fn count_items_single_mailbox() {
    let storage = storage_with(&[("a", "imap", "title a", "INBOX")]);

    let output = run(|w| cmd_count_items(&storage, w));

    assert_eq!(
        output,
        "📦 Items by source:\n  imap:\n    INBOX — 1\n  ─────\n  Total: 1\n"
    );
}

#[test]
fn count_items_multiple_mailboxes() {
    let storage = storage_with(&[
        ("a", "imap", "post a", "INBOX"),
        ("b", "imap", "update b", "Updates"),
        ("c", "imap", "post c", "Social"),
        ("d", "imap", "thread d", "Social"),
    ]);

    let output = run(|w| cmd_count_items(&storage, w));

    assert_eq!(
        output,
        "📦 Items by source:\n  imap:\n    INBOX — 1\n    Social — 2\n    Updates — 1\n  ─────\n  Total: 4\n"
    );
}

#[test]
fn count_items_multiple_sources_with_mailboxes() {
    let storage = storage_with(&[
        ("a", "imap", "inbox mail", "INBOX"),
        ("b", "imap", "social post", "Social"),
        ("c", "slack", "channel msg", "general"),
        ("d", "imap", "another social", "Social"),
        ("e", "slack", "announcement", "announcements"),
    ]);

    let output = run(|w| cmd_count_items(&storage, w));

    assert_eq!(
        output,
        "📦 Items by source:\n  imap:\n    INBOX — 1\n    Social — 2\n  slack:\n    announcements — 1\n    general — 1\n  ─────\n  Total: 5\n"
    );
}

#[test]
fn count_items_empty_mailbox_displayed_as_unknown() {
    let storage = storage_with(&[("a", "imap", "title a", "")]);

    let output = run(|w| cmd_count_items(&storage, w));

    assert_eq!(
        output,
        "📦 Items by source:\n  imap:\n    (unknown) — 1\n  ─────\n  Total: 1\n"
    );
}

#[test]
fn count_items_gmail_category_strips_prefix() {
    let storage = storage_with(&[("a", "imap", "social mail", "@@CATEGORY@@/Social")]);

    let output = run(|w| cmd_count_items(&storage, w));

    assert!(
        output.contains("Social — 1"),
        "prefix should be stripped: {output:?}"
    );
    assert!(
        !output.contains("@@CATEGORY@@"),
        "prefix should not appear: {output:?}"
    );
}

#[test]
fn list_items_empty() {
    let storage = InMemoryStorage::new();

    let output = run(|w| cmd_list_items(&storage, w));

    assert_eq!(output, "No items stored. Run `br1ef fetch` first.\n");
}

#[test]
fn list_items_shows_items() {
    let storage = storage_with(&[("a", "inbox", "hello world", "INBOX")]);

    let output = run(|w| cmd_list_items(&storage, w));

    assert_eq!(
        output,
        "📦 Items:\nsender: INBOX\n  hello world\n\n  ─────\n"
    );
}

#[test]
fn list_items_truncates_long_titles() {
    let long_title = "a".repeat(90);
    let storage = storage_with(&[("a", "inbox", &long_title, "INBOX")]);

    let output = run(|w| cmd_list_items(&storage, w));

    let expected_preview: String = long_title.chars().take(50).collect();
    assert!(output.contains(&format!("  {expected_preview}...")));
    assert!(!output.contains(&long_title[..60]));
}

#[test]
fn list_items_short_title_not_truncated() {
    let storage = storage_with(&[("a", "inbox", "short title", "INBOX")]);

    let output = run(|w| cmd_list_items(&storage, w));

    assert!(output.contains("  short title"));
    assert!(!output.contains("..."));
}

#[test]
fn list_items_multiple_items() {
    let storage = storage_with(&[
        ("a", "social", "first item", "INBOX"),
        ("b", "inbox", "second item", "INBOX"),
    ]);

    let output = run(|w| cmd_list_items(&storage, w));

    assert!(output.contains("sender: INBOX"));
    assert!(output.contains("  first item"));
    assert!(output.contains("  second item"));
    assert!(output.contains("  ─────"));
}
