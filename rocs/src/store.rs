// ROC/rocs/src/store.rs
#![allow(dead_code)]

use crate::logger;

use once_cell::sync::Lazy;
// use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
// can support range queries now ..
use std::sync::RwLock;

static STORE: Lazy<RwLock<BTreeMap<String, usize>>> = Lazy::new(|| RwLock::new(BTreeMap::new()));

pub(crate) fn store_values(key: String, value: usize) {
    let mut db = STORE.write().unwrap();

    db.insert(key, value); // passing the ownership
}

pub(crate) fn fetch_values(key: String) -> Option<usize> {
    let db = STORE.read().unwrap();

    db.get(&key).cloned() // just take the reference and return an OWNED value
}

pub(crate) fn list_all() -> Vec<(String, usize)> {
    let db = STORE.read().unwrap();
    // now we would get them in an sorted order
    db.iter().map(|(k, &v)| (k.clone(), v)).collect()
}

pub(crate) fn delete_val(key: String) -> Option<usize> {
    let mut db = STORE.write().unwrap();

    db.remove(&key)
}

pub(crate) fn update_val(key: String, val: usize) {
    let mut db = STORE.write().unwrap();

    db.insert(key, val);
}

pub(crate) fn get_range(start: usize, end: usize) -> Vec<(String, usize)> {
    let db = STORE.read().unwrap();
    db.iter()
        .filter(|(_k, &v)| v >= start && v <= end)
        .map(|(k, &v)| (k.clone(), v))
        .collect()
}

pub fn save_store<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    let db = STORE.read().unwrap();
    let serialized = serde_json::to_string(&*db).expect("Serialization Failed!");
    fs::write(path, serialized)?;

    // clear the WAL once you make a snapshot!
    let _ = logger::clear_wal();

    Ok(())
}

pub fn load_store<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    if !path.as_ref().exists() {
        return Ok(());
    }

    let data = fs::read_to_string(path)?;
    if data.trim().is_empty() {
        eprintln!("No snapshot to load");
        return Ok(());
    }

    let db: BTreeMap<String, usize> =
        serde_json::from_str(&data).expect("Derserialization into BTreeMap Failed!!");

    let mut store = STORE.write().unwrap();
    *store = db;
    println!("Snapshot Loaded!");
    Ok(())
}
