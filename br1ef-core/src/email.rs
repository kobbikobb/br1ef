use mailparse::ParsedMail;

pub const GMAIL_CATEGORY_PREFIX: &str = "@@CATEGORY@@/";

pub(crate) fn find_header(parsed: &ParsedMail, name: &str) -> Option<String> {
    let needle = name.to_lowercase();
    let header = parsed.headers.iter().find(|h| {
        let key = h.get_key();
        key.to_lowercase() == needle
    })?;
    Some(header.get_value())
}

pub(crate) fn extract_body(parsed: &ParsedMail) -> String {
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
    let s = strip_tags(s);
    let s = decode_entities(&s);
    collapse_whitespace(&s)
}

fn strip_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] != '<' {
            out.push(chars[i]);
            i += 1;
            continue;
        }

        i += 1;
        let tag = read_tag_name(&chars, i, len);

        if tag == "style" || tag == "script" {
            i = skip_to_closing_tag(&chars, i, len, &tag);
        } else {
            while i < len && chars[i] != '>' {
                i += 1;
            }
            if i < len {
                i += 1;
            }
        }
    }

    out
}

fn read_tag_name(chars: &[char], start: usize, len: usize) -> String {
    let mut name = String::new();
    let mut j = start;
    if j < len && chars[j] == '/' {
        j += 1;
    }
    while j < len && !chars[j].is_whitespace() && chars[j] != '>' {
        name.push(chars[j].to_ascii_lowercase());
        j += 1;
    }
    name
}

fn skip_to_closing_tag(chars: &[char], mut i: usize, len: usize, tag: &str) -> usize {
    let closer = format!("/{}", tag);
    while i < len {
        if chars[i] == '<' {
            let rest: String = chars[i + 1..].iter().take(closer.len()).collect();
            if rest.to_lowercase() == closer {
                while i < len && chars[i] != '>' {
                    i += 1;
                }
                return if i < len { i + 1 } else { i };
            }
        }
        i += 1;
    }
    i
}

fn decode_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&nbsp;", " ")
}

fn collapse_whitespace(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_was_space = false;
    for c in s.chars() {
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
#[path = "email_test.rs"]
mod tests;
