pub mod fetcher;
pub mod service;
pub mod storage;

/// A single digest item — an event, email, or notification
/// fetched from a configured source.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub from: String,
    pub body: String,
    pub source: String,
    pub urgent: bool,
}

/// A digest — a processed summary of fetched items.
#[derive(Debug, Clone, PartialEq)]
pub struct Digest {
    pub total_items: usize,
    pub by_source: Vec<(String, usize)>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

/// A source of digest items.
pub trait Source {
    fn id(&self) -> &str;
    fn fetch(&self) -> anyhow::Result<Vec<Item>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_basics() {
        let item = Item {
            id: "1".into(),
            title: "test".into(),
            from: "from".into(),
            body: "body".into(),
            source: "mock".into(),
            urgent: false,
        };
        assert_eq!(item.title, "test");
    }
}
