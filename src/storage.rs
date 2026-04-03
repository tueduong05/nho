use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use tracing::info;

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

pub async fn start_cleanup_worker(store: Store, interval: Duration) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        loop {
            ticker.tick().await;
            cleanup_expired(&store);
        }
    });
}

pub fn cleanup_expired(store: &Store) {
    let now = Instant::now();
    let target_removals = 100;
    let mut removed = 0;

    for entry_ref in store.iter() {
        if let Some(expires_at) = entry_ref.expires_at {
            if now > expires_at {
                let key = entry_ref.key().clone();
                drop(entry_ref);
                store.remove(&key);
                removed += 1;

                if removed >= target_removals {
                    break;
                }
            }
        }
    }

    if removed > 0 {
        info!("Cleaned {} expired entries", removed);
    }
}
