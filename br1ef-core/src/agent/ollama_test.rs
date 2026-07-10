use crate::Item;
use crate::Source;

use super::{build_prompts, parse_list_response, truncate};

fn make_item(id: &str, from: &str, title: &str, body: &str) -> Item {
    Item {
        id: id.to_string(),
        from: from.to_string(),
        title: title.to_string(),
        body: body.to_string(),
        source: Source::Imap,
        mailbox: "".into(),
        urgent: false,
    }
}

#[test]
fn build_prompts_empty_items() {
    let (system, user) = build_prompts(&[]);

    assert!(!system.is_empty());
    assert!(user.contains("Today is"));
    assert!(user.contains("Below are emails from the last week"));
    assert!(user.contains("List personal messages and action items"));
}

#[test]
fn build_prompts_single_item() {
    let item = make_item("1", "alice@example.com", "Hello", "How are you?");
    let (_system, user) = build_prompts(&[item]);

    assert!(user.contains("alice@example.com"));
    assert!(user.contains("Hello"));
    assert!(user.contains("How are you?"));
    assert!(user.contains("1. From:"));
}

#[test]
fn build_prompts_multiple_items_numbered() {
    let items = vec![
        make_item("1", "a@a.com", "Subj A", "Body A"),
        make_item("2", "b@b.com", "Subj B", "Body B"),
    ];
    let (_system, user) = build_prompts(&items);

    assert!(user.contains("1. From:"));
    assert!(user.contains("2. From:"));
    assert!(user.contains("Subj A"));
    assert!(user.contains("Body B"));
}

#[test]
fn build_prompts_no_section_headers() {
    let item = make_item("1", "a@a.com", "Test", "Body");
    let (_system, user) = build_prompts(&[item]);

    assert!(!user.contains("Personal & Action Required"));
    assert!(!user.contains("Everything Else"));
    assert!(!user.contains("##"));
}

#[test]
fn build_prompts_contains_hallucination_guardrail() {
    let item = make_item("1", "a@a.com", "Test", "Body");
    let (system, _user) = build_prompts(&[item]);
    assert!(system.contains("Only use information present in the emails"));
    assert!(system.contains("never add external knowledge"));
}

#[test]
fn build_prompts_system_separates_role_from_content() {
    let item = make_item("1", "a@a.com", "Test", "Body");
    let (system, user) = build_prompts(&[item]);

    assert!(system.contains("email digest assistant"));
    assert!(user.contains("a@a.com"));
    assert!(!user.contains("email digest assistant"));
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
fn parse_list_response_empty() {
    let names = parse_list_response(r#"{"models":[]}"#).unwrap();
    assert!(names.is_empty());
}

#[test]
fn parse_list_response_single() {
    let names = parse_list_response(r#"{"models":[{"name":"llama3.2:1b"}]}"#).unwrap();
    assert_eq!(names, vec!["llama3.2:1b"]);
}

#[test]
fn parse_list_response_multiple() {
    let json = r#"{
        "models": [
            {"name": "qwen2.5-coder:7b"},
            {"name": "llama3.2:1b"},
            {"name": "mistral:latest"}
        ]
    }"#;
    let names = parse_list_response(json).unwrap();
    assert_eq!(
        names,
        vec!["llama3.2:1b", "mistral:latest", "qwen2.5-coder:7b"]
    );
}

#[test]
fn parse_list_response_invalid_json() {
    let result = parse_list_response("not json");
    assert!(result.is_err());
}

#[test]
fn parse_list_response_missing_models_field() {
    let result = parse_list_response(r#"{"other":true}"#);
    assert!(result.is_err());
}
