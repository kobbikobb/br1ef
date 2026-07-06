use super::*;

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

// Email `from` addresses are compared case-sensitively because SMTP allows
// case in the local-part (`User@` != `user@`). Collapsing would risk losing
// replies from a legitimate variant sender.
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
