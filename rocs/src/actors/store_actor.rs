//! src/actors/store_actor.rs
//!
//! Contains the Store Actor implementation for handling all database storage-related commands
//! using the Actor Model pattern. This actor exclusively manages the key-value store state
//! and processes only storage commands. Non-storage commands (like shutdown, logging, etc.)
//! should be routed to their respective actors or handlers elsewhere for clear separation of concerns.

use crate::command::Command;
use std::collections::BTreeMap;
use tokio::sync::mpsc;

/// Type alias for the sender used to communicate with the store actor.
pub type StoreCommandSender = mpsc::Sender<Command>;

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
pub fn spawn_store_actor() -> StoreCommandSender {
    // Buffer size set to 128 for the mpsc channel.
    let (tx, mut rx) = mpsc::channel::<Command>(128);

    tokio::spawn(async move {
        let mut db = BTreeMap::<String, usize>::new();

        while let Some(cmd) = rx.recv().await {
            match cmd {
                // Stores a new key-value pair in the database.
                Command::Set { key, value, respond_to } => {
                    db.insert(key, value);
                    let _ = respond_to.send(Ok(()));
                },
                // Fetches the value associated with a given key.
                Command::Get { key, respond_to } => {
                    let val = db.get(&key).cloned();
                    let _ = respond_to.send(Ok(val));
                },
                // Deletes a key-value pair from the database.
                Command::Del { key, respond_to } => {
                    let deleted = db.remove(&key).map(|v| (key, v));
                    let _ = respond_to.send(Ok(deleted));
                },
                // Updates the value for an existing key.
                Command::Update { key, value, respond_to } => {
                    db.insert(key, value);
                    let _ = respond_to.send(Ok(()));
                },
                // Fetches all key-value pairs within a range of values.
                Command::Range { start, end, respond_to } => {
                    let res = db
                        .iter()
                        .filter(|(_k, &v)| v >= start && v <= end)
                        .map(|(k, &v)| (k.clone(), v))
                        .collect();
                    let _ = respond_to.send(Ok(res));
                },
                // Lists all key-value pairs in the database.
                Command::List { respond_to } => {
                    let res = db.iter().map(|(k, &v)| (k.clone(), v)).collect();
                    let _ = respond_to.send(Ok(res));
                },
                // Any other command variant is considered out of scope for the store actor.
                _ => {
                    // All non-storage commands should be handled by other actors/modules.
                    unreachable!("Received non-storage command in store actor");
                },
            }
        }
    });

    tx // Return the sender for communicating with the store actor.
}
