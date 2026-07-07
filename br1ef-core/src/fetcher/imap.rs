use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use native_tls::TlsConnector;

use crate::Item;
use crate::email::{GMAIL_CATEGORY_PREFIX, extract_body, find_header};

/// Bare `[Gmail]` is a namespace, not a selectable mailbox.
const GMAIL_NAMESPACE: &str = "[Gmail]";

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

    names.retain(|n| n != GMAIL_NAMESPACE);

    session.logout()?;

    if names.iter().any(|n| n.starts_with(GMAIL_NAMESPACE)) {
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
            source: crate::Source::Imap,
            mailbox: String::new(),
            urgent: false,
        });
    }

    session.logout()?;
    Ok(items)
}
