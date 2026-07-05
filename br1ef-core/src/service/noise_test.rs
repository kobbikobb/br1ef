use crate::Item;

use super::{filter_relevant, is_noise};

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
fn is_noise_linkedin_from() {
    let item = make_item("1", "notifications@linkedin.com", "You have a new message", "body");
    assert!(is_noise(&item));
}

#[test]
fn is_noise_linkedin_from_with_name() {
    let item = make_item("1", "\"LinkedIn\" <invitations@e.linkedin.com>", "Connection request", "body");
    assert!(is_noise(&item));
}

#[test]
fn is_noise_newsletter_in_title() {
    let item = make_item("1", "newsletter@example.com", "Weekly Newsletter", "lots of content");
    assert!(is_noise(&item));
}

#[test]
fn is_noise_newsletter_in_subject() {
    let item = make_item("1", "someone@example.com", "Your Newsletter Issue #42", "body");
    assert!(is_noise(&item));
}

#[test]
fn is_noise_marketing_from() {
    let item = make_item("1", "marketing@company.com", "Big Sale!", "body");
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
fn is_noise_clean_email_not_noise() {
    let item = make_item("1", "mom@family.com", "Dinner tonight?", "Want to come over?");
    assert!(!is_noise(&item));
}

#[test]
fn is_noise_case_insensitive() {
    let item = make_item("1", "nOtIfIcAtIoNs@LiNkEdIn.CoM", "Hello", "body");
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
        make_item("2", "notifications@linkedin.com", "Connection request", "body"),
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
fn filter_relevant_returns_cloned_owned_items() {
    let items = vec![make_item("1", "mom@family.com", "Dinner?", "Tonight?")];
    let relevant = filter_relevant(&items);
    assert_eq!(relevant[0].id, "1");
}
