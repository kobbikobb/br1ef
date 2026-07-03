mod config;
mod daily;
mod digest;
mod fetch;

pub use config::configure;
pub use daily::get_daily_items;
pub use digest::digest_items;
pub use fetch::fetch_items;

use std::sync::{Mutex, OnceLock};

use crate::storage::{InMemoryStorage, Storage};

fn storage() -> &'static Mutex<Box<dyn Storage>> {
    static STORAGE: OnceLock<Mutex<Box<dyn Storage>>> = OnceLock::new();
    STORAGE.get_or_init(|| Mutex::new(Box::new(InMemoryStorage::new())))
}
