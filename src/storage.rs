use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use bytes::Bytes;
use dashmap::DashMap;

#[derive(Clone)]
pub struct Entry {
    value: Bytes,
    expires_at: Option<Instant>,
}

pub type Store = Arc<DashMap<Bytes, Entry>>;

pub fn new_store() -> Store {
    Arc::new(DashMap::new())
}

pub fn set(store: &Store, key: Bytes, value: Bytes, ttl: Option<Duration>) {
    let expires_at = ttl.map(|duration| Instant::now() + duration);
    store.insert(key, Entry { value, expires_at });
}

pub fn get(store: &Store, key: &Bytes) -> Option<Bytes> {
    let entry = store.get(key)?;

    if let Some(expiry) = entry.expires_at {
        if Instant::now() > expiry {
            drop(entry);
            store.remove(key);
            return None;
        }
    }

    Some(entry.value().value.clone())
}
