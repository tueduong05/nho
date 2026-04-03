use std::{
    sync::Arc,
    time::{Duration as StdDuration, Instant as StdInstant},
};

use tracing::info;

use bytes::Bytes;
use dashmap::DashMap;
use tokio::time as TokioTime;

#[derive(Clone)]
pub struct Entry {
    value: Bytes,
    expires_at: Option<StdInstant>,
}

pub type Store = Arc<DashMap<Bytes, Entry>>;

pub fn new_store() -> Store {
    Arc::new(DashMap::new())
}

pub fn set(store: &Store, key: Bytes, value: Bytes, ttl: Option<StdDuration>) {
    let expires_at = ttl.map(|duration| StdInstant::now() + duration);
    store.insert(key, Entry { value, expires_at });
}

pub fn get(store: &Store, key: &Bytes) -> Option<Bytes> {
    let entry = store.get(key)?;

    if let Some(expiry) = entry.expires_at {
        if StdInstant::now() > expiry {
            drop(entry);
            store.remove(key);
            return None;
        }
    }

    Some(entry.value().value.clone())
}

pub async fn start_cleanup_worker(store: Store, interval: StdDuration) {
    tokio::spawn(async move {
        let mut ticker = TokioTime::interval(interval);
        loop {
            ticker.tick().await;
            cleanup_expired(&store);
        }
    });
}

pub fn cleanup_expired(store: &Store) {
    let now = StdInstant::now();
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
