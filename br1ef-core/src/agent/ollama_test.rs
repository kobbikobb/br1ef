use crate::Item;

use super::{build_prompt, filter_relevant, is_noise, truncate};

fn make_item(id: &str, from: &str, title: &str, body: &str) -> Item {
    Item {
        id: id.to_string(),
        from: from.to_string(),
        title: title.to_string(),
        body: body.to_string(),
        source: "imap".to_string(),
        urgent: false,
    }
}

#[test]
fn build_prompt_empty_items() {
    let prompt = build_prompt(&[]);

    assert!(prompt.contains("Today is"));
    assert!(prompt.contains("Below are emails from the last week"));
    assert!(prompt.contains("List personal messages and action items"));
}

#[test]
fn build_prompt_single_item() {
    let item = make_item("1", "alice@example.com", "Hello", "How are you?");
    let prompt = build_prompt(&[item]);

    assert!(prompt.contains("alice@example.com"));
    assert!(prompt.contains("Hello"));
    assert!(prompt.contains("How are you?"));
    assert!(prompt.contains("1. From:"));
}

#[test]
fn build_prompt_multiple_items_numbered() {
    let items = vec![
        make_item("1", "a@a.com", "Subj A", "Body A"),
        make_item("2", "b@b.com", "Subj B", "Body B"),
    ];
    let prompt = build_prompt(&items);

    assert!(prompt.contains("1. From:"));
    assert!(prompt.contains("2. From:"));
    assert!(prompt.contains("Subj A"));
    assert!(prompt.contains("Body B"));
}

#[test]
fn build_prompt_no_section_headers() {
    let item = make_item("1", "a@a.com", "Test", "Body");
    let prompt = build_prompt(&[item]);

    assert!(!prompt.contains("Personal & Action Required"));
    assert!(!prompt.contains("Everything Else"));
    assert!(!prompt.contains("##"));
}

#[test]
fn truncate_short_string() {
    assert_eq!(truncate("hello", 10), "hello");
}

#[test]
fn truncate_exact_length() {
    assert_eq!(truncate("hello", 5), "hello");
}

#[test]
fn truncate_empty() {
    assert_eq!(truncate("", 10), "");
}

#[test]
fn truncate_long_string() {
    let long = "a".repeat(100);
    assert_eq!(truncate(&long, 50).len(), 50);
}

#[test]
fn truncate_unicode_boundary() {
    let s = "a🎉bcdef";
    let t = truncate(s, 5);
    assert_eq!(t, "a🎉");
    assert!(t.len() <= 5);
}

#[test]
fn truncate_unicode_two_chars() {
    let s = "a🎉bcdef";
    let t = truncate(s, 6);
    assert_eq!(t, "a🎉b");
}

#[test]
fn is_noise_linkedin_from() {
    let item = make_item(
        "1",
        "notifications@linkedin.com",
        "You have a new message",
        "body",
    );
    assert!(is_noise(&item));
}

#[test]
fn is_noise_linkedin_from_with_name() {
    let item = make_item(
        "1",
        "\"LinkedIn\" <invitations@e.linkedin.com>",
        "Connection request",
        "body",
    );
    assert!(is_noise(&item));
}

#[test]
fn is_noise_newsletter_in_title() {
    let item = make_item(
        "1",
        "newsletter@example.com",
        "Weekly Newsletter",
        "lots of content",
    );
    assert!(is_noise(&item));
}

#[test]
fn is_noise_newsletter_in_subject() {
    let item = make_item(
        "1",
        "someone@example.com",
        "Your Newsletter Issue #42",
        "body",
    );
    assert!(is_noise(&item));
}

#[test]
fn is_noise_marketing_from() {
    let item = make_item("1", "marketing@company.com", "Big Sale!", "body");
    assert!(is_noise(&item));
}

#[test]
fn is_noise_clean_email_not_noise() {
    let item = make_item(
        "1",
        "mom@family.com",
        "Dinner tonight?",
        "Want to come over?",
    );
    assert!(!is_noise(&item));
}

#[test]
fn is_noise_case_insensitive() {
    let item = make_item("1", "nOtIfIcAtIoNs@LiNkEdIn.CoM", "Hello", "body");
    assert!(is_noise(&item));
}

#[test]
fn is_noise_marketing_team() {
    let item = make_item("1", "marketing-team@company.com", "Summer Sale", "body");
    assert!(is_noise(&item));
}

#[test]
fn is_noise_no_reply() {
    let item = make_item("1", "no-reply@service.com", "Your receipt", "body");
    assert!(is_noise(&item));
}

#[test]
fn is_noise_noreply() {
    let item = make_item("1", "noreply@updates.co", "Please verify", "body");
    assert!(is_noise(&item));
}

#[test]
fn filter_relevant_keeps_clean_items() {
    let items = vec![
        make_item("1", "mom@family.com", "Dinner?", "Tonight?"),
        make_item("2", "dad@family.com", "Call me", "Please"),
    ];
    let relevant = filter_relevant(&items);
    assert_eq!(relevant.len(), 2);
}

#[test]
fn filter_relevant_removes_all_noise() {
    let items = vec![
        make_item("1", "notifications@linkedin.com", "New message", "body"),
        make_item("2", "marketing@store.com", "Big sale!", "body"),
        make_item("3", "newsletter@substack.com", "Weekly Issue #5", "body"),
    ];
    let relevant = filter_relevant(&items);
    assert!(relevant.is_empty());
}

#[test]
fn filter_relevant_mixed_noise_and_clean() {
    let items = vec![
        make_item("1", "mom@family.com", "Dinner?", "Tonight?"),
        make_item(
            "2",
            "notifications@linkedin.com",
            "Connection request",
            "body",
        ),
        make_item("3", "dad@family.com", "Call me", "Please"),
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
fn build_prompt_contains_hallucination_guardrail() {
    let item = make_item("1", "a@a.com", "Test", "Body");
    let prompt = build_prompt(&[item]);
    assert!(prompt.contains("Only use the information from these emails"));
    assert!(prompt.contains("do not add anything"));
}
