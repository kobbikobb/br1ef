use crate::Item;

use super::{build_prompt, parse_list_response, truncate};

fn make_item(id: &str, from: &str, title: &str, body: &str) -> Item {
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
fn build_prompt_contains_hallucination_guardrail() {
    let item = make_item("1", "a@a.com", "Test", "Body");
    let prompt = build_prompt(&[item]);
    assert!(prompt.contains("Only use the information from these emails"));
    assert!(prompt.contains("do not add anything"));
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
