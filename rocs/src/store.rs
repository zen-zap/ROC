// ROC/rocs/src/store.rs

use crate::btree::Node;
use once_cell::sync::Lazy;
use std::collections::HashMap;
// use std::io;
use std::sync::RwLock;

static STORE: Lazy<RwLock<HashMap<String, String>>> = Lazy::new(|| RwLock::new(HashMap::new()));

pub(crate) fn store_values(key: String, value: String) {
    let mut db = STORE.write().unwrap();

    db.insert(key, value); // passing the ownership
}

pub(crate) fn fetch_values(key: String) -> Option<String> {
    let db = STORE.read().unwrap();

    db.get(&key).map(|s| s.clone().to_string()) // just take the reference and return an OWNED value
}

pub(crate) fn list_all() -> Vec<(String, String)> {
    let db = STORE.read().unwrap();

    db.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
}

pub(crate) fn delete_val(key: String) -> Option<String> {
    let mut db = STORE.write().unwrap();

    db.remove(&key)
}

pub(crate) fn update_val(key: String, val: String) {
    let mut db = STORE.write().unwrap();

    db.insert(key, val); // keep it simple for now .. store / update it anyways
}
