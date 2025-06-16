//! src/actors/user_actor.rs
//!
//! The user actor acts as the frontend/session proxy for user commands.
//! It receives commands from the dispatcher, forwards them as needed to the store actors,
//! and relays responses back to the user. This actor may also be extended with session logic, access control, etc.

use tokio::sync::{mpsc};
use crate::command::Command;

/// Channel type alias for sending commands to a user actor.
pub type UserCommandHandler = mpsc::Sender<Command>;

/// Channel type alias for sending commands to the store actor.
pub type Sch = mpsc::Sender<Command>;

/// Spawns a user actor as an async task.
///
/// # Arguments
/// * `store_ah` - Sender to communicate with the store actor.
///
/// # Returns
/// * `UserCommandHandler` - The sender to communicate with this user actor.
pub fn spawn_user_actor(store_ah: Sch) -> UserCommandHandler {
    let (tx, mut rx) = mpsc::channel::<Command>(128);

    tokio::spawn(async move {
        while let Some(cmd) = rx.recv().await {
            match cmd {
                // Forward storage commands to the store actor.
                // We don't really care about the fields here since the store actor does the
                // unpacking for us.
                Command::Set {..} => {
                    if let Err(e) = store_ah.send(cmd).await
                    {
                        eprintln!("Failed to send the command to the store actor Set");
                    }
                }
                Command::Del {..} => {
                    if let Err(e) = store_ah.send(cmd).await
                    {
                        eprintln!("Failed to send the command to the store actor Del");
                    }
                }
                Command::List {..} => {
                    if let Err(e) = store_ah.send(cmd).await
                    {
                        eprintln!("Failed to send the command to the store actor List");
                    }
                }
                Command::Range {..} => {
                    if let Err(e) = store_ah.send(cmd).await
                    {
                        eprintln!("Failed to send the command to the store actor Range");
                    }
                }

                // Respond directly to Ping
                Command::Ping {respond_to, ..} => {
                    let _ = respond_to.send("Server Running!".to_string());
                    // TODO: Enhance Ping command to return server's network information.
                    //
                    // - On receiving Ping, respond with the server's current IP address and port.
                    // - If running inside a container (e.g., Docker), fetch these from environment variables or through runtime introspection.
                    // - Investigate using std::env::var for fetching env variables (such as SERVER_IP, SERVER_PORT, etc.).
                    // - Consider supporting multiple interfaces, and fallback to 0.0.0.0 or container's bridge IP if needed.
                    // - Optionally, include other diagnostic info (host, version, uptime, etc.).
                    //
                    // This will help clients automatically discover connection endpoints and improve observability.
                }

                // Handle session exit
                Command::Exit { respond_to, user_id } => {
                    println!("EXITING USER with user_id: {}", user_id);
                    // TODO: Perform session cleanup if needed
                    let _ = respond_to.send(Ok(()));
                }

                // Ignore non-user commands
                other => {
                    eprintln!("user_actor: Skipping non-user command: {:?}", other);
                }
            }
        }
    });

    tx
}


