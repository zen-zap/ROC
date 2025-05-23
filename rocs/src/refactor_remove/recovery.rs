use crate::command::Command;
use crate::logger;
use crate::store;
use std::io;

pub fn handle_recovery() -> io::Result<()> {
    eprintln!("inside recovery module!");

    if let Err(e) = store::load_store("../snaps/snapshots.json") {
        eprintln!("Failed to load snapshot: {}", e);
    } else {
        eprintln!("Successfully loaded the snapshot");
    }

    let last_checkpoint = logger::get_health_checkpoint();

    match last_checkpoint {
        Some(status) => {
            if status == "CLEAN" {
                eprintln!("No recovery needed!");
            } else {
                eprintln!("DIRTY! There was a crash previously! \n Starting Recovery!");

                let wal_entries = logger::read_wal()?;

                for cmd in wal_entries {
                    match cmd {
                        Command::Store { key, value } => {
                            store::store_values(key, value);
                        }
                        Command::Delete { key } => {
                            let _ = store::delete_val(key);
                        }
                        Command::Update { key, value } => {
                            store::update_val(key, value);
                        }
                        _ => {
                            // pass -- non modifying command
                        }
                    }
                }

                eprintln!("State recovery complete. Exiting recovery mode");
            }
        }
        _ => {
            eprintln!("Could not get status");
            return Err(io::Error::new(io::ErrorKind::Other, "Could not get status"));
        }
    }
    Ok(())
}
