// ROC/rocd/src/main.rs

use serde_json::{self, json, Value};
use std::io::{self, prelude::*, BufReader, Write};
use std::net::TcpStream;

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:9879").expect("could not connect to server!");

    loop {
        // we gotta take commands from the user in the terminal ..!
        let mut input = String::new();
        print!("> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();

        // parse the command --- simple parse for now
        let input = input.trim();
        let request = match input.split_whitespace().collect::<Vec<&str>>().as_slice() {
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
            _ => {
                println!("Invalid command!");
                continue;
            }
        };

        // let's send the json request
        serde_json::to_writer(&mut stream, &request).unwrap();
        stream.flush().unwrap();

        let mut reader = BufReader::new(&stream);
        let mut response = String::new();
        reader.read_line(&mut response).unwrap();

        // now thatwe have read the response of the server .. let's parse it
        match serde_json::from_str::<Value>(&response) {
            Ok(res) => println!("Response: {}", res),
            Err(_) => println!("Encountered Error!"),
        }
    }
}
