//! src/actors/store_actor.rs
//!
//! Contains the Store Actor implementation for handling all database storage-related commands
//! using the Actor Model pattern. This actor exclusively manages the key-value store state
//! and processes only storage commands. Non-storage commands (like shutdown, logging, etc.)
//! should be routed to their respective actors or handlers elsewhere for clear separation of concerns.

use crate::command::Command;
use tokio::sync::mpsc;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use uuid;
use tokio::time::{self, Duration};

#[derive(Serialize, Deserialize, Default)]
pub struct StoreState {
    pub kv: BTreeMap<(String, String), usize>,
    pub users: BTreeSet<String>,
}
/// Type alias for the sender used to communicate with the store actor.
pub type StoreCommandHandler = mpsc::Sender<Command>;

/// Spawns the store actor as a Tokio task, returning a sender that can be used
/// to send storage-related `Command`s to the actor.
///
/// The store actor owns its state and only responds to commands related to data storage
/// and retrieval (Store, Fetch, Delete, Update, Range, List).
///
/// # Example
/// ```rust
/// let store_sender = spawn_store_actor();
/// // Use store_sender to send storage commands.
/// ```
///
/// # Design Note
/// This actor should **not** handle commands unrelated to storage (such as Shutdown, Crash, etc.).
/// Route such commands to other actors for better modularity and maintainability.
pub fn spawn_store_actor() -> StoreCommandHandler {
    // Buffer size set to 128 for the mpsc channel.
    let (tx, mut rx) = mpsc::channel::<Command>(128);

    let path = store_path();
    let mut state = if let Ok(bytes) = fs::read(&path) {
        bincode::deserialize(&bytes).unwrap_or_default()
    } else {
        StoreState::default()
    };

    tokio::spawn(async move {

        let mut interval = time::interval(Duration::from_secs(10));

        loop {

            tokio::select! {

                maybe_cmd = rx.recv() => {

                    match maybe_cmd {
                        Some(cmd) => {
                            match cmd {
                                Command::Hi { user_id, respond_to } => {
                                    let assigned_id = match user_id {
                                        Some(id) if state.users.contains(&id) => id,
                                        _ => {
                                            let new_id = uuid::Uuid::new_v4().to_string();
                                            state.users.insert(new_id.clone());
                                            new_id
                                        }
                                    };
                                    let _ = respond_to.send(assigned_id);
                                },
                                Command::Set { user_id, key, value, respond_to } => {
                                    state.kv.insert((user_id, key), value);
                                    let _ = respond_to.send(Ok(()));
                                },
                                Command::Get { user_id, key, respond_to } => {
                                    let val = state.kv.get(&(user_id, key)).cloned();
                                    let _ = respond_to.send(Ok(val));
                                },
                                Command::Del { user_id, key, respond_to } => {
                                    let _deleted = state.kv.remove(&(user_id, key.clone())).map(|v| (key, v));
                                    let _ = respond_to.send(Ok(()));
                                },
                                Command::Update { user_id, key, value, respond_to } => {
                                    state.kv.insert((user_id, key), value);
                                    let _ = respond_to.send(Ok(()));
                                },
                                Command::Range { user_id, start, end, respond_to } => {
                                    let res = state.kv
                                        .range((user_id.clone(), start)..=(user_id.clone(), end))
                                        .map(|((u, k), &v)| ((u.clone(), k.clone()), v))
                                        .collect();
                                    let _ = respond_to.send(Ok(res));
                                },
                                Command::List { user_id, respond_to } => {
                                    let res = state.kv
                                        .iter()
                                        .filter(|((u, _), _)| *u == user_id)
                                        .map(|((u, k), &v)| ((u.clone(), k.clone()), v))
                                        .collect();
                                    let _ = respond_to.send(Ok(res));
                                },
                                Command::Persist { respond_to } => {
                                    let res = persist_state(&state).map_err(|e| e.to_string());
                                    let _ = respond_to.send(res);
                                },
                                // Add shutdown or other commands if needed
                                _ => unreachable!("Received non-storage command in store actor"),
                            }
                        }
                        None => break, // channel closed, exit actor
                    }
                },
                _ = interval.tick() => {
                    if let Err(e) = persist_state(&state) {
                        eprintln!("Failed to persist store state: {e}");
                    }
                }
            }
        }
    });

    tx // Return the sender for communicating with the store actor.
}

fn store_path() -> PathBuf {
    let mut home = dirs::home_dir().expect("Could not find home directory");
    home.push(".roc_server");
    fs::create_dir_all(&home).expect("Failed to create roc_server directory");
    home.push("store_state.bin");
    home
}

fn persist_state(state: &StoreState) -> Result<(), Box<dyn std::error::Error>> {
    let path = store_path();
    let tmp_path = path.with_extension("bin.tmp");
    let bytes = bincode::serialize(state)?;
    fs::write(&tmp_path, &bytes)?;
    fs::rename(tmp_path, path)?; // atomic replace
    Ok(())
}
