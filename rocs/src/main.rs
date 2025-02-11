// ROC/rocs/src/main.rs

use serde_json::{self, json, Value};
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() -> io::Result<()> {
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
    let mut buffer = Vec::new();

    if let Err(err) = stream.read_to_end(&mut buffer) {
        eprintln!("failed to read into the buffer, error: {}", err);
        return;
    }

    let request: Value = match serde_json::from_slice(&buffer) {
        Ok(req) => req,
        Err(_) => {
            eprintln!("Invalid json received");
            let error_response = json!({"status" : "ERROR",
            "message" : "Invalid JSON!"});

            stream.write_all(error_response.to_string().as_bytes()).ok();
            return;
        }
    };

    let response = match request["command"].as_str() {
        Some("PING") => {
            json!({"status" : "PONG"})
        }
        Some("STORE") => {
            json!({"status" : "OK"})
        }
        Some("FETCH") => {
            json!({"status" : "OK", "value" : "Dummmy_val"})
        }
        _ => {
            json!({"status" : "ERROR!"})
        }
    };

    stream
        .write_all(response.to_string().as_bytes())
        .expect("Failed to send a response!");
}
