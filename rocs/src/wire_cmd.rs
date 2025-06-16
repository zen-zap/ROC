use serde::{Deserialize, Serialize};
use crate::command::{Command, UserId};
use tokio::sync::oneshot;

/// The wire-format for user-accessible commands. Only user commands included.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WireCommand {
    Hi { user_id: Option<UserId> },
    Ping { user_id: UserId },
    Set { user_id: UserId, key: String, value: usize },
    Get { user_id: UserId, key: String },
    Del { user_id: UserId, key: String },
    Update { user_id: UserId, key: String, value: usize },
    Range { user_id: UserId, start: String, end: String },
    List { user_id: UserId },
    Exit { user_id: UserId },
}

impl WireCommand {
    /// Converts a WireCommand sent by the user into an internal Command, attaching a oneshot responder.
    /// Returns (Command, WireResponseReceiver).
    pub fn into_internal(self) -> (Command, WireResponseReceiver) {

        //eprintln!("INSIDE INTERNAL CONVERSION OF WIRED_COMMAND TO COMMAND");

        match self {
            WireCommand::Hi { user_id } => {
                let (tx, rx) = oneshot::channel();
                (
                    Command::Hi { user_id, respond_to: tx },
                    WireResponseReceiver::UserId(rx),
                )
            }
            WireCommand::Ping { user_id } => {
                let (tx, rx) = oneshot::channel();
                (
                    Command::Ping { user_id, respond_to: tx },
                    WireResponseReceiver::String(rx),
                )
            }
            WireCommand::Set { user_id, key, value } => {
                let (tx, rx) = oneshot::channel();
                (
                    Command::Set { user_id, key, value, respond_to: tx },
                    WireResponseReceiver::ResultUnit(rx),
                )
            }
            WireCommand::Get { user_id, key } => {
                let (tx, rx) = oneshot::channel();
                (
                    Command::Get { user_id, key, respond_to: tx },
                    WireResponseReceiver::ResultOptUsize(rx),
                )
            }
            WireCommand::Del { user_id, key } => {
                let (tx, rx) = oneshot::channel();
                (
                    Command::Del { user_id, key, respond_to: tx },
                    WireResponseReceiver::ResultUnit(rx),
                )
            }
            WireCommand::Update { user_id, key, value } => {
                let (tx, rx) = oneshot::channel();
                (
                    Command::Update { user_id, key, value, respond_to: tx },
                    WireResponseReceiver::ResultUnit(rx),
                )
            }
            WireCommand::Range { user_id, start, end } => {
                let (tx, rx) = oneshot::channel();
                (
                    Command::Range { user_id, start, end, respond_to: tx },
                    WireResponseReceiver::ResultKvVec(rx),
                )
            }
            WireCommand::List { user_id } => {
                let (tx, rx) = oneshot::channel();
                (
                    Command::List { user_id, respond_to: tx },
                    WireResponseReceiver::ResultKvVec(rx),
                )
            }
            WireCommand::Exit { user_id } => {
                let (tx, rx) = oneshot::channel();
                (
                    Command::Exit { user_id, respond_to: tx },
                    WireResponseReceiver::ResultUnit(rx),
                )
            }
        }
    }
}

/// This is the receiver since we hand out the sender to the actor and await their response here
/// Enum for all possible response receiver types.
pub enum WireResponseReceiver {
    UserId(oneshot::Receiver<String>),
    String(oneshot::Receiver<String>),
    ResultUnit(oneshot::Receiver<Result<(), String>>),
    ResultOptUsize(oneshot::Receiver<Result<Option<usize>, String>>),
    ResultKvVec(oneshot::Receiver<Result<Vec<((String, String), usize)>, String>>),
}
