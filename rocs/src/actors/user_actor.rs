//! src/actors/user_actor.rs
//!
//! The user actor acts as the frontend/session proxy for user commands.
//! It receives commands from the dispatcher, forwards them as needed to the store or workspace actors,
//! and relays responses back to the user. This actor may also be extended with session logic, access control, etc.

use tokio::sync::{mpsc, oneshot};
use crate::command::Command;

/// Channel type alias for sending commands to a user actor.
pub type UserCommandHandler = mpsc::Sender<Command>;

/// Channel type alias for sending commands to the store actor.
pub type Sch = mpsc::Sender<Command>;

/// Channel type alias for sending commands to the workspace actor.
pub type Wch = mpsc::Sender<Command>;

/// Spawns a user actor as an async task.
///
/// # Arguments
/// * `store_ah` - Sender to communicate with the store actor.
/// * `workspace_ah` - Sender to communicate with the workspace actor.
///
/// # Returns
/// * `UserCommandHandler` - The sender to communicate with this user actor.
pub fn spawn_user_actor(store_ah: Sch, workspace_ah: Wch) -> UserCommandHandler {
    let (tx, mut rx) = mpsc::channel::<Command>(128);

    tokio::spawn(async move {
        while let Some(cmd) = rx.recv().await {
            match cmd {
                // Forward storage commands to the store actor.
                Command::Set { key, value, respond_to } => {
                    // TODO: Forward to store_ah
                }
                Command::Get { key, respond_to } => {
                    // TODO: Forward to store_ah
                }
                Command::Del { key, respond_to } => {
                    // TODO: Forward to store_ah
                }
                Command::List { respond_to } => {
                    // TODO: Forward to store_ah
                }
                Command::Range { start, end, respond_to } => {
                    // TODO: Forward to store_ah
                }

                // Workspace management commands
                Command::CreateWorkspace { name, respond_to } => {
                    // TODO: Forward to workspace_ah
                }
                Command::ListWorkspaces { respond_to } => {
                    // TODO: Forward to workspace_ah
                }
                Command::SwitchWorkspace { name, respond_to } => {
                    // TODO: Forward to workspace_ah
                }
                Command::DropWorkspace { name, respond_to } => {
                    // You might want to restrict this to admin only.
                    // TODO: Forward or restrict as needed.
                }

                // Respond directly to Ping
                Command::Ping { respond_to } => {
                    let _ = respond_to.send("Server Running at ip:port".to_string());
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
                Command::Exit { respond_to } => {
                    // TODO: Perform session cleanup if needed
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


