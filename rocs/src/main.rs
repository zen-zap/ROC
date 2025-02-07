// ROC/rocs/src/main.rs

use std::io::prelude::*;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() -> io::Result<()> {
    let mut buffer = [0u8; 1500];

    let mut listener = TcpListener::bind("127.0.0.1:9879")?;

    // to handle the clients connected on the port
    for stream in listener.incoming() {
        match (stream) {
            Ok(stream) => {
                eprintln!("received connection : {:#?}", stream);

                thread::spawn(move || handle_client(stream, &mut buffer));
            }
            Err(e) => {
                eprintln!("Encountered error while receiving connection: {:#?}", e);
            }
        }
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream, buffer: &mut [u8]) {
    // read from the stream and return the number of bytes read
    let bytes_read = stream.read(buffer).unwrap();
    if bytes_read == 0 {
        // no bytes read
        eprintln!("No bytes read");
        // close the connection
        eprintln!("Closing the connection");
        return;
    } else {
        eprintln!("Connected to server at 127.0.0.1:9879");

        eprintln!("Received data: {:?}", &buffer[..bytes_read]);
        // echo back for testing
        stream.write_all(&buffer[..bytes_read]).unwrap();
    }
}
