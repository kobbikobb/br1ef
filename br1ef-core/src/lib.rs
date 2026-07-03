/// A single digest item — an event, email, or notification
/// fetched from a configured source.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub body: String,
    pub source: String,
    pub urgent: bool,
}

/// A source of digest items.
pub trait Source {
    fn id(&self) -> &str;
    fn fetch(&self) -> Vec<Item>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_basics() {
        let item = Item {
            id: "1".into(),
            title: "test".into(),
            body: "body".into(),
            source: "mock".into(),
            urgent: false,
        };
        assert_eq!(item.title, "test");
    }
}
