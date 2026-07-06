mod delete;
mod digest;
mod fetch;

pub use delete::delete_items;
pub use digest::digest_items;
pub use fetch::{FetchResult, MailboxStats, fetch_items};
