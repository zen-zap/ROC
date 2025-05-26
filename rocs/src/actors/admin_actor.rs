use tokio::sync::{oneshot, mpsc};
use crate::command::Command;

pub type AdminCommandHandler = mpsc::Sender<Command>;

pub fn spawn_admin_actor() -> AdminCommandHandler {

    let (tx, mut rx) = mpsc::channel::<Command>(64);

    tokio::spawn(async move {

        while Some(cmd) = rx.recv().await {
            match cmd {
                Command::Shutdown {
                    respond_to,
                    ..
                } => {
                    // TODO: signal other actors
                    // maybe set some flags if you do later on
                    let _ = respond_to.send(Ok(()));
                }
                Command::Crash {
                    respond_to,
                    ..
                } => {
                    let _ = respond_to.send(Ok(()));
                    panic!("Database crash requested by admin!");
                }
                Command::ClearWal {
                    respond_to,
                    ..
                } => {
                    let _ = respond_to.send(Ok(()));
                    // TODO: clear the wal 
                }
                Command::Snapshot {
                    respond_to,
                    .. 
                } => {
                    let _ = respond_to.send(Ok(()));
                    // TODO: create a snapshot
                }
                other => {
                    #[cfg(debug_assertions)]
                    eprintln!("[admin actor] Ignored non-admin command: {:?}", other);
                }
            }
        }
    });

    tx
} 
