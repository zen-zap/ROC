//! rocs/src/main.rs
//!

#![allow(warnings)]
mod actors;
mod command;
mod initializer;
mod router;

use anyhow;
use std::io;
use tokio::{self, io::{AsyncReadExt, AsyncWriteExt}};
use rcgen::{generate_simple_self_signed, CertifiedKey, Certificate};
use quinn::{Endpoint, ServerConfig};
use rocs::{
    network::connections::handle_connection, 
    router::{ActorChannels, route_cmd}, 
    initializer::initialize_system
};
use rustls_pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use std::net::SocketAddr;
use std::fs;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> anyhow::Result<()> {

    let system: ActorChannels = initialize_system().await;

    let alt_domain_names_for_cert = vec!["localhost".to_string(), "hello.azen".to_string()];

    let cert = generate_simple_self_signed(alt_domain_names_for_cert).unwrap();

    let certificate_der = cert.cert.der().clone(); // DER encoded certificate .. 
    let key_pri_der = cert.key_pair.serialize_der(); // DER encoded private key (PKCS#8 format)   -- server authentication

    fs::create_dir_all("data/keys").expect("Failed to create cert directory");
    fs::write("data/keys/server_cert.pem", cert.cert.pem()).expect("Failed to save cert");
    fs::write("data/keys/server_key.pem", cert.key_pair.serialize_pem()).expect("Failed to save private key");

    let certificate_chain = vec![certificate_der];
    let private_key = {
        let pkcs8 = PrivatePkcs8KeyDer::from(key_pri_der);
        PrivateKeyDer::from(pkcs8)
    };

    let server_config = ServerConfig::with_single_cert(certificate_chain, private_key).expect("Failed to create Server Config");

    let addr: SocketAddr = "127.0.0.0:4433".parse().expect("Failed to parse Socker Address for server");
    let endpoint = Endpoint::server(server_config, addr).expect("Failed to create server endpoint");

    println!("Server running at {}  --- transmitting over QUIC", addr);

    while let Some(connecting) = endpoint.accept().await {

        // Each iteration handles one new client

        let system = system.clone(); // you would need the actor handles for each new client

        // we want a clients handshake, streams, lifecycle to be managed independently & concurrently
        // so that if one client is slow it doesn't block the whole thing
        tokio::spawn(async move {

            // handles 1 client connection
            let connection = match connecting.await {
                Ok(conn) => conn,
                Err(e) => {
                    eprintln!("Failed to establish connection : {}", e);
                    return;
                }
            };

            println!("Accepted Connection from: {}", connection.remote_address());

            loop {

                let _ = match connection.accept_bi().await {

                    Ok((mut send, mut recv)) => {

                        let system = system.clone(); 
                        // why do we clone again here?
                        // Because if we use the cloned channels here .. they;ll be moved here ..
                        // we want each stream from the client to use the channels independently

                        tokio::spawn(async move {

                            if let Err(e) = handle_connection(send, recv, system).await {
                                eprintln!("Stream Error: {:?}", e);
                            }
                        });
                    }

                    Err(quinn::ConnectionError::Reset) | Err(quinn::ConnectionError::ApplicationClosed{..}) => {
                        eprintln!("Connection closed by client");
                        break;
                    }

                    Err(e) => {
                        eprintln!("Stream accept error: {:?}", e);
                        break;
                    }

                };
            }

            println!("Connection closed: {:?}", connection.remote_address());
        });

    }
    Ok(())
}
