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

    let mut names: Vec<String> = mailboxes
        .iter()
        .map(|m| m.name().to_string())
        .collect();

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

    let (search_filter, select_mailbox) = if let Some(category) = mailbox.strip_prefix(GMAIL_CATEGORY_PREFIX) {
        let label = format!("CATEGORY_{}", category.to_uppercase());
        (format!("X-GM-LABELS \"{label}\" SINCE {since_str}"), "[Gmail]/All Mail")
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

        let uid = msg.uid.unwrap_or(0).to_string();
        let subject = find_header(&parsed, "Subject").unwrap_or_default();
        let from = find_header(&parsed, "From").unwrap_or_default();
        let body_text = extract_body(&parsed);

        items.push(Item {
            id: format!("{select_mailbox}/{uid}"),
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
                return part.get_body().unwrap_or_default();
            }
        }
        return String::new();
    }

    if ct == "text/html" && parsed.subparts.is_empty() {
        return parsed.get_body().unwrap_or_default();
    }

    String::new()
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
}
