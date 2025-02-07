// ROC/rocd/src/main.rs

use std::io::{self, prelude::*, Read, Write};
use std::net::{self, TcpListener, TcpStream};
use std::thread;

fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:9879").expect("could not connect to server!");

    // let's write and see ..
    // it should echo back
    let test_msg = "Hello server!".as_bytes();
    stream.write_all(test_msg)?;
    eprintln!("Sent message to server: {:#?}", test_msg);

    let mut buffer = [0u8; 1500];
    let bytes_read = stream.read(&mut buffer)?;

    eprintln!(
        "echoed back from the server: {}",
        String::from_utf8_lossy(&buffer[..bytes_read])
    );

    Ok(())
}
