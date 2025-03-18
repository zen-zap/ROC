// ROC/rocd/src/main.rs

use serde_json::{self, json, Value};
use std::io::{self, prelude::*, BufReader, Write};
use std::net::TcpStream;

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:9879").expect("could not connect to server!");

    eprintln!("Waiting for command...");
    loop {
        // we gotta take commands from the user in the terminal ..!
        let mut input = String::new();
        print!("(roc)> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();

        // parse the command --- simple parse for nowi
        let input = input.trim();
        eprintln!("user command: {}", input);

        let mut command_tokens: Vec<String> =
            input.split_whitespace().map(|s| s.to_string()).collect();

        if command_tokens.is_empty() {
            continue;
        }

        command_tokens[0] = command_tokens[0].to_uppercase();

        if (command_tokens[0] == "GET") && command_tokens.len() >= 2 {
            command_tokens[1] = command_tokens[1].to_uppercase();
        }

        let command_str: Vec<&str> = command_tokens.iter().map(|s| s.as_str()).collect();

        let request = match command_str.as_slice() {
            // as_slice() returns &[&str] -- like a slice of string slices
            ["PING"] => {
                json!({"command" : "PING"})
            }
            ["STORE", key, value] => {
                json!({"command" : "STORE",
                "key" : key,
                "value" : value})
            }
            ["FETCH", key] => {
                json!({"command" : "FETCH",
                "key" : key})
            }
            ["EXIT"] => {
                break;
            }
            ["LIST"] => {
                json!({"command" : "LIST"})
            }
            ["UPDATE", key, value] => {
                json!({"command" : "UPDATE",
                "key" : key,
                "value" : value})
            }
            ["DELETE", key] => {
                json!({"command" : "DELETE",
                "key" : key})
            }
            ["GET", "BETWEEN", start, end] => {
                json!({
                    "command": "RANGE",
                    "start" : start,
                    "end" : end
                })
            }
            _ => {
                println!("Invalid command!");
                continue;
            }
        };

        // let's send the json request
        // eprintln!("Gonna write this into the stream: {:?}", request);
        serde_json::to_writer(&mut stream, &request).unwrap();
        write!(stream, "\n").unwrap();
        stream.flush().unwrap();

        let mut reader = BufReader::new(&stream);
        let mut response = String::new();
        reader.read_line(&mut response).unwrap();

        // now that we have read the response of the server .. let's parse it and display it ..
        match serde_json::from_str::<Value>(&response) {
            Ok(res) => println!("Response: {:#?}", res),
            Err(_) => println!("Encountered Error!"),
        }
    }
}
