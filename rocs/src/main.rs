// ROC/rocs/src/main.rs

mod store;

use serde_json::{self, json, Value};
use std::io::{self, BufRead, BufReader, Write};
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

// In the responses .. we're also gonna include a type called: "ALIVE"
//
// "ALIVE" : server can still function ... issue can be resolved by user
// "DEAD"  : server cannot function ... issue cannot be resolved by user

fn handle_client(mut stream: TcpStream) {
    let reader_stream = stream
        .try_clone()
        .expect("failed to clone the stream for the reader -- func handle_client");
    let mut reader = BufReader::new(reader_stream);
    let mut line = String::new();

    // let's setup to continuously read commands from the client
    while reader.read_line(&mut line).unwrap() != 0
    // this one reads into the above line variable
    {
        eprintln!("Raw input line: {:?}", line);
        let command_str = line.trim();
        eprintln!("Command string: {:?}", command_str);

        if !command_str.is_empty() {
            let request: Value = match serde_json::from_str::<Value>(command_str) {
                Ok(req) => req,
                Err(_) => {
                    eprintln!("Invalid json received");
                    let error_response = json!(
                        {"status" : "ERROR",
                        "message" : "Invalid JSON!",
                        "type":"ALIVE"}); // keep type as ALIVE since we can just ask the user to re-enter
                                          // the json request

                    stream.write_all(error_response.to_string().as_bytes()).ok();
                    line.clear();
                    continue;
                }
            };

            eprintln!("Parsed request: {:?}", request);

            let response = match request["command"].as_str() {
                Some("PING") => {
                    json!({"status" : "OK",
                    "message": "Successfully Pinged!"})
                }
                Some("STORE") => {
                    if let (Some(key), Some(value)) =
                        (request["key"].as_str(), request["value"].as_str())
                    {
                        store::store_values(key.to_string(), value.to_string());
                        json!({"status":"OK", "message":"Successfully stored the values", "type":"ALIVE"})
                    } else {
                        json!({"status":"ERROR", "message":"Unable to read key_value pair from the request", "type":"DEAD"})
                        // what should this be?... I mean this should be a critical issue right?
                    }
                }
                Some("FETCH") => {
                    if let Some(key) = request["key"].as_str() {
                        if let Some(value) = store::fetch_values(key.to_string()) {
                            json!({"status":"OK", "value":value, "type":"ALIVE"})
                        } else {
                            json!({"status":"ERROR", "message":"Value not found in storage!", "type":"ALIVE"})
                        }
                    } else {
                        json!({"status":"ERROR", "message":"Unable to get key from request", "type":"DEAD"})
                        // I don't know what this should be ..  but not being able to read the requests would be a critical issue
                    }
                }
                Some("LIST") => {
                    let all_entries = store::list_all();
                    json!({
                        "status":"OK",
                        "data" : all_entries,
                        "type" : "ALIVE"
                    })
                }
                Some("DELETE") => {
                    if let Some(key) = request["key"].as_str() {
                        let del_val = store::delete_val(key.to_string()).unwrap();
                        json!({
                            "status":"OK",
                            "message" : "Successfully deleted the key",
                            "value" : del_val,
                            "type" : "ALIVE"
                        })
                    } else {
                        json!({
                            "status" : "ERROR",
                            "message" : "Error while removing the key",
                            "type" : "ALIVE"
                        })
                    }
                }
                Some("UPDATE") => {
                    if let (Some(key), Some(val)) =
                        (request["key"].as_str(), request["value"].as_str())
                    {
                        store::update_val(key.to_string(), val.to_string());
                        json!({
                            "status" : "OK",
                            "message" : "Value updated",
                            "type" : "ALIVE"
                        })
                    } else {
                        json!({"status":"ERROR",
                            "message" : "Error Updating value",
                            "type" : "ALIVE"})
                    }
                }
                _ => {
                    json!({"status" : "ERROR!"})
                }
            };

            let reponse_str = response.to_string() + "\n";
            stream
                .write_all(reponse_str.as_bytes())
                .expect("Failed to send a response!");
        }

        line.clear();
    }
}
