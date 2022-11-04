use std::sync::Mutex;
use std::{collections::HashMap, hash::Hash};

pub struct Cache<K, D> {
    data: Mutex<HashMap<K, D>>,
}

impl<K: Eq + Hash, D: Clone> Cache<K, D> {
    pub fn new() -> Self {
        Cache {
            data: Mutex::new(HashMap::<K, D>::new()),
        }
    }

    pub fn get(&self, key: &K) -> Option<D> {
        self.data.lock().unwrap().get(&key).cloned()
    }

    pub fn insert(&mut self, key: K, value: D) -> Option<D> {
        self.data.lock().unwrap().insert(key, value)
    }

    pub fn remove(&mut self, key: &K) -> Option<D> {
        self.data.lock().unwrap().remove(key)
    }
}
