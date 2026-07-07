use anyhow::Result;

use crate::Item;
use crate::agent::Agent;
use crate::storage::InMemoryStorage;
use crate::storage::Storage;

use super::{
    build_digest, dedup_threads, digest_items, filter_relevant, is_noise, normalize_subject,
};

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

fn make_noise_item(id: &str, from: &str, title: &str, body: &str) -> Item {
    Item {
        id: id.to_string(),
        from: from.to_string(),
        title: title.to_string(),
        body: body.to_string(),
        source: "imap".to_string(),
        mailbox: "".into(),
        urgent: false,
    }
}

// ── normalize_subject tests ──────────────────────────────────────────

#[test]
fn normalize_clean_subject() {
    assert_eq!(normalize_subject("Hello world"), "Hello world");
}

#[test]
fn normalize_re_prefix() {
    assert_eq!(normalize_subject("Re: hello"), "hello");
}

#[test]
fn normalize_re_uppercase() {
    assert_eq!(normalize_subject("RE: hello"), "hello");
}

#[test]
fn normalize_re_lowercase() {
    assert_eq!(normalize_subject("re: hello"), "hello");
}

#[test]
fn normalize_fwd_prefix() {
    assert_eq!(normalize_subject("FWD: hello"), "hello");
}

#[test]
fn normalize_aw_prefix() {
    assert_eq!(normalize_subject("AW: hello"), "hello");
}

#[test]
fn normalize_nested_re() {
    assert_eq!(normalize_subject("Re: Re: hello"), "hello");
}

#[test]
fn normalize_does_not_strip_without_space_or_bracket() {
    assert_eq!(normalize_subject("Re:foo"), "Re:foo");
}

#[test]
fn normalize_strips_with_bracket() {
    assert_eq!(normalize_subject("Re:[PATCH] foo"), "[PATCH] foo");
}

#[test]
fn normalize_empty_subject() {
    assert_eq!(normalize_subject(""), "");
}

#[test]
fn normalize_only_prefix() {
    assert_eq!(normalize_subject("Re:"), "Re:");
}

#[test]
fn normalize_whitespace_only() {
    assert_eq!(normalize_subject("   "), "");
}

#[test]
fn normalize_vs_prefix() {
    assert_eq!(normalize_subject("VS: hello"), "hello");
    assert_eq!(normalize_subject("vs: hello"), "hello");
}

#[test]
fn normalize_sv_prefix() {
    assert_eq!(normalize_subject("SV: hello"), "hello");
}

#[test]
fn normalize_vid_prefix() {
    assert_eq!(normalize_subject("VID: hello"), "hello");
}

#[test]
fn normalize_antw_prefix() {
    assert_eq!(normalize_subject("ANTW: hello"), "hello");
}

#[test]
fn normalize_wg_prefix() {
    assert_eq!(normalize_subject("WG: hello"), "hello");
}

#[test]
fn normalize_mixed_prefixes() {
    assert_eq!(normalize_subject("Re: Fwd: hello"), "hello");
    assert_eq!(normalize_subject("AW: Re: hello"), "hello");
    assert_eq!(normalize_subject("Fwd: Re: FW: hello"), "hello");
}

#[test]
fn normalize_very_deeply_nested() {
    let subject = (0..100).map(|_| "Re: ").collect::<String>() + "hello";

    let result = normalize_subject(&subject);

    assert_eq!(result, "hello");
}

#[test]
fn normalize_double_space_after_prefix() {
    assert_eq!(normalize_subject("Re:  hello"), "hello");
}

#[test]
fn normalize_leading_whitespace() {
    assert_eq!(normalize_subject("  Re: hello"), "hello");
}

#[test]
fn normalize_trailing_whitespace() {
    assert_eq!(normalize_subject("Re: hello  "), "hello");
}

#[test]
fn normalize_unicode_subject() {
    assert_eq!(normalize_subject("Re: こんにちは"), "こんにちは");
}

#[test]
fn normalize_re_only_with_space() {
    assert_eq!(normalize_subject("Re: "), "Re:");
}

#[test]
fn normalize_fw_prefix() {
    assert_eq!(normalize_subject("FW: hello"), "hello");
    assert_eq!(normalize_subject("Fw: hello"), "hello");
    assert_eq!(normalize_subject("fw: hello"), "hello");
}

// ── dedup_threads tests ──────────────────────────────────────────────

#[test]
fn dedup_empty() {
    let result = dedup_threads(vec![]);

    assert!(result.is_empty());
}

#[test]
fn dedup_single_item() {
    let items = vec![Item {
        id: "1".into(),
        title: "Hello".into(),
        from: "alice@example.com".into(),
        body: "body".into(),
        source: "imap".into(),
        mailbox: "".into(),
        urgent: false,
    }];

    let result = dedup_threads(items);

    assert_eq!(result.len(), 1);
}

#[test]
fn dedup_collapses_thread_to_newest() {
    let items = vec![
        Item {
            id: "1".into(),
            title: "Re: Hello".into(),
            from: "alice@example.com".into(),
            body: "old reply".into(),
            source: "imap".into(),
            mailbox: "".into(),
            urgent: false,
        },
        Item {
            id: "2".into(),
            title: "Re: Hello".into(),
            from: "alice@example.com".into(),
            body: "newest reply".into(),
            source: "imap".into(),
            mailbox: "".into(),
            urgent: false,
        },
    ];

    let result = dedup_threads(items);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, "2");
    assert_eq!(result[0].body, "newest reply");
}

#[test]
fn dedup_keeps_different_senders() {
    let items = vec![
        Item {
            id: "1".into(),
            title: "Hello".into(),
            from: "alice@example.com".into(),
            body: "alice says".into(),
            source: "imap".into(),
            mailbox: "".into(),
            urgent: false,
        },
        Item {
            id: "2".into(),
            title: "Hello".into(),
            from: "bob@example.com".into(),
            body: "bob says".into(),
            source: "imap".into(),
            mailbox: "".into(),
            urgent: false,
        },
    ];

    let result = dedup_threads(items);

    assert_eq!(result.len(), 2);
}

#[test]
fn dedup_preserves_order() {
    let items = vec![
        Item {
            id: "1".into(),
            title: "Meeting".into(),
            from: "alice@example.com".into(),
            body: "body".into(),
            source: "imap".into(),
            mailbox: "".into(),
            urgent: false,
        },
        Item {
            id: "2".into(),
            title: "Lunch".into(),
            from: "alice@example.com".into(),
            body: "body".into(),
            source: "imap".into(),
            mailbox: "".into(),
            urgent: false,
        },
        Item {
            id: "3".into(),
            title: "Re: Meeting".into(),
            from: "alice@example.com".into(),
            body: "reply".into(),
            source: "imap".into(),
            mailbox: "".into(),
            urgent: false,
        },
    ];

    let result = dedup_threads(items);

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].id, "3");
    assert_eq!(result[1].id, "2");
}

#[test]
fn dedup_all_unique() {
    let items = vec![
        Item {
            id: "1".into(),
            title: "Meeting".into(),
            from: "alice@example.com".into(),
            body: "body".into(),
            source: "imap".into(),
            mailbox: "".into(),
            urgent: false,
        },
        Item {
            id: "2".into(),
            title: "Lunch".into(),
            from: "bob@example.com".into(),
            body: "body".into(),
            source: "imap".into(),
            mailbox: "".into(),
            urgent: false,
        },
    ];

    let result = dedup_threads(items);

    assert_eq!(result.len(), 2);
}

#[test]
fn dedup_many_replies_only_keeps_last() {
    let items: Vec<Item> = (0..10)
        .map(|i| Item {
            id: i.to_string(),
            title: "Re: Hello".into(),
            from: "alice@example.com".into(),
            body: format!("reply {i}"),
            source: "imap".into(),
            mailbox: "".into(),
            urgent: false,
        })
        .collect();

    let result = dedup_threads(items);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, "9");
    assert_eq!(result[0].body, "reply 9");
}

#[test]
fn dedup_different_from_casing() {
    let items = vec![
        Item {
            id: "1".into(),
            title: "Hello".into(),
            from: "Alice@example.com".into(),
            body: "body".into(),
            source: "imap".into(),
            mailbox: "".into(),
            urgent: false,
        },
        Item {
            id: "2".into(),
            title: "Hello".into(),
            from: "alice@example.com".into(),
            body: "body".into(),
            source: "imap".into(),
            mailbox: "".into(),
            urgent: false,
        },
    ];

    let result = dedup_threads(items);

    assert_eq!(result.len(), 2);
}

#[test]
fn dedup_preserves_input_order() {
    let items = vec![
        Item {
            id: "1".into(),
            title: "Alpha".into(),
            from: "a@example.com".into(),
            body: "body".into(),
            source: "imap".into(),
            mailbox: "".into(),
            urgent: false,
        },
        Item {
            id: "2".into(),
            title: "Beta".into(),
            from: "b@example.com".into(),
            body: "body".into(),
            source: "imap".into(),
            mailbox: "".into(),
            urgent: false,
        },
        Item {
            id: "3".into(),
            title: "Gamma".into(),
            from: "c@example.com".into(),
            body: "body".into(),
            source: "imap".into(),
            mailbox: "".into(),
            urgent: false,
        },
    ];

    let result = dedup_threads(items);

    assert_eq!(result.len(), 3);
    assert_eq!(result[0].id, "1");
    assert_eq!(result[1].id, "2");
    assert_eq!(result[2].id, "3");
}

// ── is_noise tests ───────────────────────────────────────────────────

#[test]
fn is_noise_linkedin_from() {
    let item = make_noise_item(
        "1",
        "notifications@linkedin.com",
        "You have a new message",
        "body",
    );
    assert!(is_noise(&item));
}

#[test]
fn is_noise_linkedin_from_with_name() {
    let item = make_noise_item(
        "1",
        "\"LinkedIn\" <invitations@e.linkedin.com>",
        "Connection request",
        "body",
    );
    assert!(is_noise(&item));
}

#[test]
fn is_noise_newsletter_in_title() {
    let item = make_noise_item(
        "1",
        "newsletter@example.com",
        "Weekly Newsletter",
        "lots of content",
    );
    assert!(is_noise(&item));
}

#[test]
fn is_noise_newsletter_in_subject() {
    let item = make_noise_item(
        "1",
        "someone@example.com",
        "Your Newsletter Issue #42",
        "body",
    );
    assert!(is_noise(&item));
}

#[test]
fn is_noise_marketing_from() {
    let item = make_noise_item("1", "marketing@company.com", "Big Sale!", "body");
    assert!(is_noise(&item));
}

#[test]
fn is_noise_marketing_team() {
    let item = make_noise_item("1", "marketing-team@company.com", "Summer Sale", "body");
    assert!(is_noise(&item));
}

#[test]
fn is_noise_no_reply() {
    let item = make_noise_item("1", "no-reply@service.com", "Your receipt", "body");
    assert!(is_noise(&item));
}

#[test]
fn is_noise_noreply() {
    let item = make_noise_item("1", "noreply@updates.co", "Please verify", "body");
    assert!(is_noise(&item));
}

#[test]
fn is_noise_clean_email_not_noise() {
    let item = make_noise_item(
        "1",
        "mom@family.com",
        "Dinner tonight?",
        "Want to come over?",
    );
    assert!(!is_noise(&item));
}

#[test]
fn is_noise_case_insensitive() {
    let item = make_noise_item("1", "nOtIfIcAtIoNs@LiNkEdIn.CoM", "Hello", "body");
    assert!(is_noise(&item));
}

// ── filter_relevant tests ────────────────────────────────────────────

#[test]
fn filter_relevant_keeps_clean_items() {
    let items = vec![
        make_noise_item("1", "mom@family.com", "Dinner?", "Tonight?"),
        make_noise_item("2", "dad@family.com", "Call me", "Please"),
    ];
    let relevant = filter_relevant(&items);
    assert_eq!(relevant.len(), 2);
}

#[test]
fn filter_relevant_removes_all_noise() {
    let items = vec![
        make_noise_item("1", "notifications@linkedin.com", "New message", "body"),
        make_noise_item("2", "marketing@store.com", "Big sale!", "body"),
        make_noise_item("3", "newsletter@substack.com", "Weekly Issue #5", "body"),
    ];
    let relevant = filter_relevant(&items);
    assert!(relevant.is_empty());
}

#[test]
fn filter_relevant_mixed_noise_and_clean() {
    let items = vec![
        make_noise_item("1", "mom@family.com", "Dinner?", "Tonight?"),
        make_noise_item(
            "2",
            "notifications@linkedin.com",
            "Connection request",
            "body",
        ),
        make_noise_item("3", "dad@family.com", "Call me", "Please"),
    ];
    let relevant = filter_relevant(&items);
    assert_eq!(relevant.len(), 2);
    assert!(relevant.iter().any(|i| i.from == "mom@family.com"));
    assert!(relevant.iter().any(|i| i.from == "dad@family.com"));
}

#[test]
fn filter_relevant_empty_input() {
    let relevant = filter_relevant(&[]);
    assert!(relevant.is_empty());
}

#[test]
fn filter_relevant_returns_cloned_owned_items() {
    let items = vec![make_noise_item("1", "mom@family.com", "Dinner?", "Tonight?")];
    let relevant = filter_relevant(&items);
    assert_eq!(relevant[0].id, "1");
}

// ── digest_items + build_digest tests ────────────────────────────────

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
