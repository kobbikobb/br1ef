use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use mailparse::ParsedMail;
use native_tls::TlsConnector;

use crate::Item;

const GMAIL_CATEGORY_PREFIX: &str = "@@CATEGORY@@/";

static GMAIL_CATEGORIES: &[&str] = &["Social", "Updates", "Promotions", "Forums"];

pub fn list_mailboxes(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
) -> Result<Vec<String>> {
    let tls = TlsConnector::builder()
        .build()
        .context("failed to build TLS connector")?;

    let client =
        imap::connect((host, port), host, &tls).context("failed to connect to IMAP server")?;

    let mut session = client
        .login(username, password)
        .map_err(|e| e.0)
        .context("IMAP login failed")?;

    let mailboxes = session
        .list(None, Some("*"))
        .context("failed to list mailboxes")?;

    let mut names: Vec<String> = mailboxes.iter().map(|m| m.name().to_string()).collect();

    session.logout()?;

    if names.iter().any(|n| n.starts_with("[Gmail]")) {
        for cat in GMAIL_CATEGORIES {
            names.push(format!("{GMAIL_CATEGORY_PREFIX}{cat}"));
        }
    }

    Ok(names)
}

pub fn fetch_imap(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    mailbox: &str,
) -> Result<Vec<Item>> {
    let tls = TlsConnector::builder()
        .build()
        .context("failed to build TLS connector")?;

    let client =
        imap::connect((host, port), host, &tls).context("failed to connect to IMAP server")?;

    let mut session = client
        .login(username, password)
        .map_err(|e| e.0)
        .context("IMAP login failed")?;

    let since = Utc::now() - Duration::days(7);
    let since_str = since.format("%d-%b-%Y").to_string();

    let (search_filter, select_mailbox) =
        if let Some(category) = mailbox.strip_prefix(GMAIL_CATEGORY_PREFIX) {
            let label = format!("CATEGORY_{}", category.to_uppercase());
            (
                format!("X-GM-LABELS \"{label}\" SINCE {since_str}"),
                "[Gmail]/All Mail",
            )
        } else {
            (format!("SINCE {since_str}"), mailbox)
        };

    session
        .select(select_mailbox)
        .with_context(|| format!("failed to select mailbox \"{select_mailbox}\""))?;

    let uids = session
        .uid_search(&search_filter)
        .context("IMAP search failed")?;

    if uids.is_empty() {
        session.logout()?;
        return Ok(Vec::new());
    }

    let uid_list: Vec<String> = uids.iter().map(|u| u.to_string()).collect();
    let uid_set = uid_list.join(",");

    let messages = session
        .uid_fetch(&uid_set, "RFC822")
        .context("failed to fetch messages")?;

    let mut items = Vec::with_capacity(messages.len());

    for msg in messages.iter() {
        let body = match msg.body() {
            Some(b) => b,
            None => continue,
        };

        let parsed = match mailparse::parse_mail(body) {
            Ok(p) => p,
            Err(_) => continue,
        };

        let message_id =
            find_header(&parsed, "Message-ID").or_else(|| find_header(&parsed, "Message-Id"));
        let id = message_id.unwrap_or_else(|| {
            let uid = msg.uid.unwrap_or(0).to_string();
            let fallback = format!("{select_mailbox}/{uid}");
            fallback
        });
        let subject = find_header(&parsed, "Subject").unwrap_or_default();
        let from = find_header(&parsed, "From").unwrap_or_default();
        let body_text = extract_body(&parsed);

        items.push(Item {
            id,
            title: subject,
            from,
            body: body_text,
            source: "imap".into(),
            urgent: false,
        });
    }

    session.logout()?;
    Ok(items)
}

fn find_header(parsed: &ParsedMail, name: &str) -> Option<String> {
    let needle = name.to_lowercase();
    let header = parsed.headers.iter().find(|h| {
        let key = h.get_key();
        key.to_lowercase() == needle
    })?;
    Some(header.get_value())
}

fn extract_body(parsed: &ParsedMail) -> String {
    let ct = parsed.ctype.mimetype.as_str();

    if ct == "text/plain" {
        return parsed.get_body().unwrap_or_default();
    }

    if ct.starts_with("multipart/") {
        for part in &parsed.subparts {
            let result = extract_body(part);
            if !result.is_empty() {
                return result;
            }
        }
        for part in &parsed.subparts {
            if part.ctype.mimetype == "text/html" {
                return strip_html(&part.get_body().unwrap_or_default());
            }
        }
        return String::new();
    }

    if ct == "text/html" && parsed.subparts.is_empty() {
        return strip_html(&parsed.get_body().unwrap_or_default());
    }

    String::new()
}

fn strip_html(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == '<' {
            i += 1;

            let mut tag_name = String::new();
            let mut j = i;
            if j < len && chars[j] == '/' {
                j += 1;
            }
            while j < len && !chars[j].is_whitespace() && chars[j] != '>' {
                tag_name.push(chars[j].to_ascii_lowercase());
                j += 1;
            }

            if tag_name == "style" || tag_name == "script" {
                let closing = format!("</{}", tag_name);
                while i < len {
                    if chars[i] == '<' {
                        let rest: String = chars[i + 1..].iter().take(closing.len()).collect();
                        if rest.to_lowercase() == closing {
                            while i < len && chars[i] != '>' {
                                i += 1;
                            }
                            if i < len {
                                i += 1;
                            }
                            break;
                        }
                    }
                    i += 1;
                }
                continue;
            }

            while i < len && chars[i] != '>' {
                i += 1;
            }
            if i < len {
                i += 1;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    let decoded = result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&nbsp;", " ");

    let mut out = String::with_capacity(decoded.len());
    let mut prev_was_space = false;
    for c in decoded.chars() {
        if c.is_whitespace() {
            if !prev_was_space {
                out.push(' ');
            }
            prev_was_space = true;
        } else {
            out.push(c);
            prev_was_space = false;
        }
    }

    out.trim().to_string()
}

#[cfg(test)]
mod tests {
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
}
