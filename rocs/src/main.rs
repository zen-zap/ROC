// ROC/rocs/src/main.rs

mod command;
mod logger;
mod recovery;
mod store;

use crate::command::Command;
use serde_json::{self, json, Value};
use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() -> io::Result<()> {
    // handle the recovery
    match recovery::handle_recovery() {
        Ok(_) => {
            // recovery success!
        }
        Err(_) => {
            // error during recovery
        }
    }

    // let's make an admin thread to control the server
    thread::spawn(move || handle_admin());

    let listener = TcpListener::bind("127.0.0.1:9879")?;

    // to handle the clients connected on the port
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                eprintln!("received connection : {:#?}", stream);

                thread::spawn(move || handle_client(stream));
            }
            Err(e) => {
                eprintln!("Encountered error while receiving connection: {:#?}", e);
            }
        }
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream) {
    let reader_stream = stream
        .try_clone()
        .expect("failed to clone the stream for the reader -- func handle_client");
    let mut reader = BufReader::new(reader_stream);
    let mut line = String::new();

    // let's setup to continuously read commands from the client
    while let Ok(bytes_read) = reader.read_line(&mut line)
    // this one reads into the above line variable
    {
        if bytes_read == 0 {
            break; // connection closed
        }

        let command_str = line.trim(); // eg. STORE ashu 12  -- something like this for now

        if !command_str.is_empty() {
            let request: Value = match serde_json::from_str::<Value>(command_str) {
                Ok(req) => req,
                Err(_) => {
                    eprintln!("Invalid json received");
                    // Command::ERR {msg: "Invalid json received".to_string()}
                    let error_response = json!({"error": "Invalid JSON received!"}).to_string();
                    stream.write_all(error_response.as_bytes()).ok();
                    line.clear();
                    continue;
                }
            };

            let command: Command = match request["command"].as_str() {
                Some("PING") => Command::Ping,
                Some("STORE") => {
                    if let (Some(key), Some(value)) =
                        (request["key"].as_str(), request["value"].as_str())
                    {
                        store::store_values(key.to_string(), value.to_string());
                        Command::Store {
                            key: key.to_string(),
                            value: value.to_string(),
                        }
                    } else {
                        Command::ERR {
                            msg: "Unable to read key_value pair from the request".to_string(),
                        }
                    }
                }
                Some("FETCH") => {
                    if let Some(key) = request["key"].as_str() {
                        if let Some(val) = store::fetch_values(key.to_string()) {
                            Command::Fetch {
                                key: key.to_string(),
                                value: Some(val),
                            }
                        } else {
                            Command::ERR {
                                msg: "Value not found in storage!".to_string(),
                            }
                        }
                    } else {
                        Command::ERR {
                            msg: "Unable to get key from request".to_string(),
                        }
                    }
                }
                Some("LIST") => {
                    let all_entries = store::list_all();
                    Command::List {
                        entries: all_entries,
                    }
                }
                Some("DELETE") => {
                    if let Some(key) = request["key"].as_str() {
                        let _ = store::delete_val(key.to_string()).unwrap();
                        Command::Delete {
                            key: key.to_string(),
                        }
                    } else {
                        Command::ERR {
                            msg: "Error while removing the key".to_string(),
                        }
                    }
                }
                Some("UPDATE") => {
                    if let (Some(key), Some(val)) =
                        (request["key"].as_str(), request["value"].as_str())
                    {
                        store::update_val(key.to_string(), val.to_string());
                        Command::Update {
                            key: key.to_string(),
                            value: val.to_string(),
                        }
                    } else {
                        Command::ERR {
                            msg: "Error updating value".to_string(),
                        }
                    }
                }
                _ => Command::ERR {
                    msg: "unknown command".to_string(),
                },
            };

            logger::store_log(&command);

            let response = serde_json::to_string(&command)
                .unwrap_or_else(|_| "{\"error\": \"Failed to serialize response\"}".to_string());

            let response_str = response + "\n";
            stream
                .write_all(response_str.as_bytes())
                .expect("Failed to send a response!");
        }

        line.clear();
    }
}

fn handle_admin() {
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    // lock the current standard input so that no other process could read from it ..

    let mut admin_line = String::new();

    eprintln!("Admin interface started!");

    loop {
        admin_line.clear();
        print!("roc-admin/~  ");
        io::stdout().flush().ok();

        if reader.read_line(&mut admin_line).is_ok() {
            let admin_cmd = admin_line.trim();
            // eprintln!("Admin command received: {:#?}", admin_cmd);

            if admin_cmd.eq_ignore_ascii_case("SHUTDOWN") {
                logger::save_checkpoint("CLEAN".to_string());
                eprintln!("SHUTDOWN initiated!");
                std::process::exit(0);
            }
            if admin_cmd.eq_ignore_ascii_case("CRASH") {
                logger::save_checkpoint("DIRTY".to_string());
                eprintln!("Simulated CRASH initiated for testing recovery");
                std::process::exit(0);
            }

            if admin_cmd.eq_ignore_ascii_case("clear") {
                let file_path = "../logs/wal.log";
                let file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(false)
                    .open(file_path)
                    .expect("Failed to clear the wal.log file");

                use std::io::BufWriter;
                let mut writer = BufWriter::new(file);

                writer
                    .write(b"")
                    .expect("Failed to write into WAL in admin_cmd");
                writer.flush().expect("Failed to flush! in admin clear WAL");
            }
        } else {
            eprintln!("failed to read command from the admin");
        }
    }
}
