use crate::logger;
use crate::store;
use std::io;

pub fn handle_recovery() -> io::Result<()> {
    eprintln!("inside recovery module!");
    let last_checkpoint = logger::get_health_checkpoint();

    match last_checkpoint {
        Some(status) => {
            if status == "CLEAN" {
                eprintln!("No recovery needed!");
            } else {
                eprintln!("DIRTY! There was a crash previously! \n Starting Recovery!");

                let wal_entries = logger::read_wal();

                for request in wal_entries.unwrap() {
                    if let Some(cmd) = request["command"].as_str() {
                        match cmd {
                            "STORE" => {
                                if let (Some(key), Some(value)) =
                                    (request["key"].as_str(), request["value"].as_str())
                                {
                                    store::store_values(key.to_string(), value.to_string());
                                } else {
                                    eprintln!("Error while storing values in recovery mode");
                                }
                            }
                            "DELETE" => {
                                if let Some(key) = request["key"].as_str() {
                                    store::delete_val(key.to_string());
                                }
                            }
                            "UPDATE" => {
                                if let (Some(key), Some(val)) =
                                    (request["key"].as_str(), request["value"].as_str())
                                {
                                    store::update_val(key.to_string(), val.to_string());
                                }
                            }
                            _ => {
                                // pass -- non modifying command
                            }
                        }
                    }
                }

                eprintln!("State recovery complete... can proceed with usual operation. Exiting recovery mode");
            }
        }
        _ => {
            eprintln!("Could not get status");
            return Err(io::Error::new(io::ErrorKind::Other, "Could not get status"));
        }
    }
    Ok(())
}
