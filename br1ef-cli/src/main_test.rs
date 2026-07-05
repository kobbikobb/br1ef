use super::*;
use br1ef_core::Item;
use br1ef_core::storage::{AppConfig, InMemoryStorage, Storage};

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

#[test]
fn test_app_config_defaults_incomplete() {
    let cfg = AppConfig::defaults();
    assert!(!cfg.is_complete());
    assert!(cfg.imap_host.is_empty());
    assert_eq!(cfg.imap_port, 993);
    assert_eq!(cfg.ollama_base_url, "http://localhost:11434");
    assert_eq!(cfg.ollama_model, "llama3.2:1b");
}

#[test]
fn test_app_config_is_complete() {
    let cfg = AppConfig {
        imap_host: "imap.example.com".into(),
        imap_port: 993,
        imap_username: "user@example.com".into(),
        imap_password: "secret".into(),
        ..AppConfig::defaults()
    };
    assert!(cfg.is_complete());
}

#[test]
fn test_app_config_missing_host_not_complete() {
    let cfg = AppConfig {
        imap_host: "".into(),
        imap_port: 993,
        imap_username: "user@example.com".into(),
        imap_password: "secret".into(),
        ..AppConfig::defaults()
    };
    assert!(!cfg.is_complete());
}

#[test]
fn test_load_config_errors_on_incomplete() {
    let storage = InMemoryStorage::new();
    let result = load_config(&storage);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Run `br1ef config`"));
}

#[test]
fn test_load_config_ok_on_complete() {
    let mut storage = InMemoryStorage::new();
    let cfg = AppConfig {
        imap_host: "imap.example.com".into(),
        imap_port: 993,
        imap_username: "user".into(),
        imap_password: "pass".into(),
        ..AppConfig::defaults()
    };
    storage.set_app_config(&cfg).unwrap();
    let result = load_config(&storage);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().imap_host, "imap.example.com");
}

#[test]
fn test_app_config_roundtrip_in_memory() {
    let mut storage = InMemoryStorage::new();
    let cfg = AppConfig {
        imap_host: "imap.test.com".into(),
        imap_port: 993,
        imap_username: "test@test.com".into(),
        imap_password: "p@ss|word".into(),
        ollama_base_url: "http://localhost:11434".into(),
        ollama_model: "llama3.2:1b".into(),
    };
    storage.set_app_config(&cfg).unwrap();
    let loaded = storage.get_app_config().unwrap();
    assert_eq!(loaded.imap_host, "imap.test.com");
    assert_eq!(loaded.imap_password, "p@ss|word");
}

#[test]
fn test_br1ef_app_config_roundtrip_sqlite() {
    use br1ef_core::storage::SqliteStorage;
    let mut storage = SqliteStorage::new(":memory:").unwrap();
    let cfg = AppConfig {
        imap_host: "imap.test.com".into(),
        imap_port: 993,
        imap_username: "test@test.com".into(),
        imap_password: "p@ss|word".into(),
        ollama_base_url: "http://localhost:11434".into(),
        ollama_model: "llama3.2:1b".into(),
    };
    storage.set_app_config(&cfg).unwrap();
    let loaded = storage.get_app_config().unwrap();
    assert_eq!(loaded.imap_host, "imap.test.com");
    assert_eq!(loaded.imap_password, "p@ss|word");
    assert_eq!(loaded.imap_port, 993);
}
