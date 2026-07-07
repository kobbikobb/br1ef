use super::*;

#[test]
fn extract_plain_text_body() {
    let raw = b"From: test@example.com\r\nSubject: test\r\n\r\nHello world";
    let parsed = mailparse::parse_mail(raw).unwrap();
    let body = extract_body(&parsed);
    assert_eq!(body, "Hello world");
}

#[test]
fn extract_body_from_multipart() {
    let raw = b"From: test@example.com\r\nSubject: test\r\nContent-Type: multipart/alternative; boundary=foo\r\n\r\n--foo\r\nContent-Type: text/plain\r\n\r\nplain text\r\n--foo\r\nContent-Type: text/html\r\n\r\n<html><body>html</body></html>\r\n--foo--";
    let parsed = mailparse::parse_mail(raw).unwrap();
    let body = extract_body(&parsed);
    assert_eq!(body.trim(), "plain text");
}

#[test]
fn extract_subject_from_header() {
    let raw = b"From: test@example.com\r\nSubject: Hello there\r\n\r\nbody";
    let parsed = mailparse::parse_mail(raw).unwrap();
    let subject = find_header(&parsed, "Subject");
    assert_eq!(subject.as_deref(), Some("Hello there"));
}

#[test]
fn extract_html_only_body() {
    let raw = b"From: test@example.com\r\nSubject: test\r\nContent-Type: text/html\r\n\r\n<html><body>hello</body></html>";
    let parsed = mailparse::parse_mail(raw).unwrap();

    let body = extract_body(&parsed);

    assert_eq!(body, "hello");
}

#[test]
fn extract_body_no_content() {
    let raw = b"From: test@example.com\r\nSubject: test\r\n\r\n";
    let parsed = mailparse::parse_mail(raw).unwrap();

    let body = extract_body(&parsed);

    assert_eq!(body, "");
}

#[test]
fn extract_body_empty_multipart() {
    let raw = b"From: test@example.com\r\nSubject: test\r\nContent-Type: multipart/alternative; boundary=foo\r\n\r\n--foo\r\nContent-Type: text/plain\r\n\r\n\r\n--foo--";
    let parsed = mailparse::parse_mail(raw).unwrap();

    let body = extract_body(&parsed);

    assert_eq!(body.trim(), "");
}

#[test]
fn extract_body_falls_back_to_html_when_no_plain_text() {
    let raw = b"From: test@example.com\r\nSubject: test\r\nContent-Type: multipart/alternative; boundary=foo\r\n\r\n--foo\r\nContent-Type: text/html\r\n\r\n<html><body>fallback</body></html>\r\n--foo--";
    let parsed = mailparse::parse_mail(raw).unwrap();

    let body = extract_body(&parsed);

    assert_eq!(body.trim(), "fallback");
}

#[test]
fn extract_body_nested_multipart() {
    let raw = b"From: test@example.com\r\nSubject: test\r\nContent-Type: multipart/mixed; boundary=outer\r\n\r\n--outer\r\nContent-Type: multipart/alternative; boundary=inner\r\n\r\n--inner\r\nContent-Type: text/plain\r\n\r\nnested plain\r\n--inner--\r\n--outer--";
    let parsed = mailparse::parse_mail(raw).unwrap();

    let body = extract_body(&parsed);

    assert!(body.contains("nested plain"));
}

#[test]
fn find_header_is_case_insensitive() {
    let raw = b"From: test@example.com\r\nMessage-Id: <abc123@example.com>\r\n\r\nbody";
    let parsed = mailparse::parse_mail(raw).unwrap();

    let result = find_header(&parsed, "message-id");

    assert_eq!(result.as_deref(), Some("<abc123@example.com>"));
}

#[test]
fn strip_tags_removes_basic_html() {
    assert_eq!(strip_tags("<p>hello</p>"), "hello");
}

#[test]
fn strip_tags_keeps_text_between_tags() {
    assert_eq!(strip_tags("<div>a</div> <span>b</span>"), "a b");
}

#[test]
fn strip_tags_skips_style_content() {
    assert_eq!(
        strip_tags("<style>.foo { color: red; }</style>text"),
        "text"
    );
}

#[test]
fn strip_tags_skips_script_content() {
    assert_eq!(strip_tags("<script>alert('xss')</script>safe"), "safe");
}

#[test]
fn strip_tags_empty_input() {
    assert_eq!(strip_tags(""), "");
}

#[test]
fn strip_tags_no_html() {
    assert_eq!(strip_tags("hello world"), "hello world");
}

#[test]
fn strip_tags_self_closing_tag() {
    assert_eq!(strip_tags("hello<br/>world"), "helloworld");
}

#[test]
fn decode_entities_basic() {
    assert_eq!(decode_entities("a&amp;b"), "a&b");
    assert_eq!(decode_entities("a&lt;b"), "a<b");
    assert_eq!(decode_entities("a&gt;b"), "a>b");
    assert_eq!(decode_entities("&quot;hello&quot;"), "\"hello\"");
    assert_eq!(decode_entities("a&nbsp;b"), "a b");
}

#[test]
fn decode_entities_no_entities() {
    assert_eq!(decode_entities("plain text"), "plain text");
}

#[test]
fn decode_entities_empty_string() {
    assert_eq!(decode_entities(""), "");
}

#[test]
fn collapse_whitespace_collapses_multiple_spaces() {
    assert_eq!(collapse_whitespace("a    b"), "a b");
}

#[test]
fn collapse_whitespace_replaces_newlines_with_space() {
    assert_eq!(collapse_whitespace("a\nb\nc"), "a b c");
}

#[test]
fn collapse_whitespace_replaces_tabs() {
    assert_eq!(collapse_whitespace("a\t\tb"), "a b");
}

#[test]
fn collapse_whitespace_trims_ends() {
    assert_eq!(collapse_whitespace("  hello  "), "hello");
}

#[test]
fn collapse_whitespace_empty_string() {
    assert_eq!(collapse_whitespace(""), "");
}

#[test]
fn collapse_whitespace_whitespace_only() {
    assert_eq!(collapse_whitespace("   \n  \t  "), "");
}

#[test]
fn strip_html_full_pipeline() {
    let input = "  <html><body><p>hello &amp; goodbye</p></body></html>  ";
    assert_eq!(strip_html(input), "hello & goodbye");
}

#[test]
fn strip_html_with_style_script() {
    let input = "<style>body{margin:0}</style><p>text</p><script>alert(1)</script>";
    assert_eq!(strip_html(input), "text");
}
