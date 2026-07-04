use crate::Item;

const REPLY_PREFIXES: &[&str] = &[
    "re:", "fwd:", "fw:", "aw:", "vs:", "sv:", "vid:", "antw:", "wg:",
];

pub fn normalize_subject(subject: &str) -> String {
    let mut s = subject.trim();
    loop {
        let lower = s.to_lowercase();
        let mut found = false;
        for &p in REPLY_PREFIXES {
            if lower.starts_with(p) {
                let after = &s[p.len()..];
                if after.starts_with(' ') || after.starts_with('[') {
                    let trimmed = after.trim_start();
                    if !trimmed.is_empty() {
                        s = trimmed;
                        found = true;
                        break;
                    }
                }
            }
        }
        if !found {
            return s.to_string();
        }
    }
}

pub fn dedup_threads(items: Vec<Item>) -> Vec<Item> {
    let mut seen: std::collections::HashMap<(String, String), Item> =
        std::collections::HashMap::new();
    let mut order: Vec<(String, String)> = Vec::new();

    for item in items {
        let normalized = normalize_subject(&item.title);
        let key = (item.from.clone(), normalized);
        if !seen.contains_key(&key) {
            order.push(key.clone());
        }
        seen.insert(key, item);
    }

    order.into_iter().filter_map(|k| seen.remove(&k)).collect()
}

#[cfg(test)]
mod tests {
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
    fn dedup_single_item() {
        let items = vec![Item {
            id: "1".into(),
            title: "Hello".into(),
            from: "alice@example.com".into(),
            body: "body".into(),
            source: "imap".into(),
            urgent: false,
        }];
        assert_eq!(dedup_threads(items).len(), 1);
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
                urgent: false,
            },
            Item {
                id: "2".into(),
                title: "Re: Hello".into(),
                from: "alice@example.com".into(),
                body: "newest reply".into(),
                source: "imap".into(),
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
                urgent: false,
            },
            Item {
                id: "2".into(),
                title: "Hello".into(),
                from: "bob@example.com".into(),
                body: "bob says".into(),
                source: "imap".into(),
                urgent: false,
            },
        ];
        assert_eq!(dedup_threads(items).len(), 2);
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
                urgent: false,
            },
            Item {
                id: "2".into(),
                title: "Lunch".into(),
                from: "alice@example.com".into(),
                body: "body".into(),
                source: "imap".into(),
                urgent: false,
            },
            Item {
                id: "3".into(),
                title: "Re: Meeting".into(),
                from: "alice@example.com".into(),
                body: "reply".into(),
                source: "imap".into(),
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
                urgent: false,
            },
            Item {
                id: "2".into(),
                title: "Lunch".into(),
                from: "bob@example.com".into(),
                body: "body".into(),
                source: "imap".into(),
                urgent: false,
            },
        ];
        assert_eq!(dedup_threads(items).len(), 2);
    }
}
