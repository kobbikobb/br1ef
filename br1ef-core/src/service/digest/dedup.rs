use crate::Item;

const REPLY_PREFIXES: &[&str] = &[
    "re:", "fwd:", "fw:", "aw:", "vs:", "sv:", "vid:", "antw:", "wg:",
];

fn normalize_subject(subject: &str) -> String {
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

pub(super) fn dedup_threads(items: Vec<Item>) -> Vec<Item> {
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
#[path = "dedup_test.rs"]
mod tests;
