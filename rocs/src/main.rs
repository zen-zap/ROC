use std::io::prelude::*;
use std::io;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::{Read, Write};


fn main() -> io::Result<()> {

    let mut buffer = [0u8; 1500];
    
    let mut incoming = TcpListener::connect(127.0.0.1:9879);

    handle_stream(incoming, buffer);
    
    Ok(())
}

fn handle_stream(incoming: TcpListener, &mut buffer) -> int
{
    // read from the stream and return the number of bytes read
    let bytes_read = incoming.read(&mut buffer);
    if bytes_read == 0
    {
        // no bytes read
        eprintln!("No bytes read");
    }
    else
    {
        // echo back for testing
    }
}
