use std::sync::Arc;

use bytes::Bytes;
use dashmap::DashMap;

pub type Store = Arc<DashMap<Bytes, Bytes>>;

pub fn new_store() -> Store {
    Arc::new(DashMap::new())
}

pub fn set(store: &Store, key: Bytes, value: Bytes) {
    store.insert(key, value);
}

pub fn get(store: &Store, key: &Bytes) -> Option<Bytes> {
    store.get(key).map(|entry| entry.value().clone())
}
