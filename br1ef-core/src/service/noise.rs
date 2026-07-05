use crate::Item;

pub fn is_noise(item: &Item) -> bool {
    let from_lower = item.from.to_lowercase();
    let title_lower = item.title.to_lowercase();

    from_lower.contains("linkedin.com")
        || title_lower.contains("newsletter")
        || from_lower.contains("newsletter@")
        || from_lower.contains("marketing")
        || from_lower.contains("no-reply")
        || from_lower.contains("noreply")
}

pub fn filter_relevant(items: &[Item]) -> Vec<Item> {
    items
        .iter()
        .filter(|i| !is_noise(i))
        .cloned()
        .collect()
}

#[cfg(test)]
#[path = "noise_test.rs"]
mod tests;
