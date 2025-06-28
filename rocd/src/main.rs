// ROC/rocd/src/main.rs
#![allow(unused)]
use anyhow::Result;
use clap::Parser;
use quinn::crypto::rustls::QuicClientConfig;
use quinn::{ClientConfig, Connection, Endpoint};
use rustls::RootCertStore;
use rustls_pki_types::CertificateDer;
use rustls_pki_types::pem::PemObject;
use serde_json::{Value, json};
use std::fs;
use std::io::{self, Write};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as TokioBufReader};
use rocd::{WireCommand, get_user_id};

#[derive(Debug, Parser)]
struct Args {
	/// Server Address
	#[arg(long)]
	server_addr: String,
	/// Path to trusted server certificate PEM file
	#[arg(long)]
	server_cert: String,
    /// enables test-mode
    #[arg(long, default_value_t=false)]
    test_mode: bool,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<()> {
	let args = Args::parse();

	let cert_bytes = fs::read(&args.server_cert)?;
	let cert = CertificateDer::from_pem_slice(&cert_bytes).expect("failed to read from pem bytes");

	let mut roots = RootCertStore::empty();
	roots.add(cert.clone()).expect("failed to add server cert");

	let client_config = ClientConfig::with_root_certificates(Arc::new(roots))?;

	let mut endpoint = Endpoint::client("[::]:0".parse().unwrap())?;
	endpoint.set_default_client_config(client_config);

	let new_conn = endpoint
		.connect(args.server_addr.parse::<SocketAddr>()?, "localhost")
		.expect("Failed to connect")
		.await
		.expect("Connection Failed");

	println!("Connected to ROC server over QUIC!");

    let user_id = get_user_id(&new_conn, args.test_mode).await.expect("failed to get user_id");
    // ID check
    if user_id.trim().is_empty() {
        eprintln!("ERROR: Received empty user_id from server. Exiting.");
        std::process::exit(1);
    }

	repl(new_conn, user_id).await?;

	Ok(())
}

async fn repl(conn: Connection, user_id: String) -> Result<()> {

    // user_id is supposed to properly defined here
    // TODO: maybe add a check if it is a proper UUID

	let stdin = io::stdin();
	let mut stdout = io::stdout();

	loop {
		let mut input = String::new();
		print!("(roc:client)> ");
		io::stdout().flush().unwrap();
		io::stdin().read_line(&mut input).unwrap();

		let input = input.trim();

		if input.is_empty() {
			continue;
		}

		let mut command_tokens: Vec<String> =
			input.split_whitespace().map(|s| s.to_string()).collect();

		command_tokens[0] = command_tokens[0].to_uppercase();

		if (command_tokens[0] == "GET") && command_tokens.len() >= 2 {
			command_tokens[1] = command_tokens[1].to_uppercase();
		}

		let command_str: Vec<&str> = command_tokens.iter().map(|s| s.as_str()).collect();

		let request = match command_str.as_slice() {
			["HI"] => {
				WireCommand::Hi { user_id: Some(user_id.clone()) }
			},
            ["EXIT"] => {
                WireCommand::Exit { user_id: user_id.clone() }
            }
            ["PING"] => {
				WireCommand::Ping { user_id: user_id.clone() }
			},
			["STORE", key, value] => {
                WireCommand::Set {
                    user_id: user_id.clone(),
                    key: key.to_string(),
                    value: {
                        match value.parse::<usize>() {
                            Ok(v) => v,
                            Err(e) => {
                                println!("Failed to parse value into integer");
                                continue;
                            }
                        }
                    },
                }
            },
			["FETCH", key] => {
				WireCommand::Get {
                    user_id: user_id.clone(),
                    key: key.to_string(),
                }
			},
			["LIST"] => {
				WireCommand::List {
                    user_id: user_id.clone(),
                }
			},
			["UPDATE", key, value] => {
				WireCommand::Update {
                    user_id: user_id.clone(),
                    key: key.to_string(),
                    value: {
                        match value.parse::<usize>() {
                            Ok(v) => v,
                            Err(e) => {
                                println!("Failed to parse value into integer");
                                continue;
                            }
                        }
                    },
                }
			},
			["DELETE", key] => {
				WireCommand::Del {
                    user_id: user_id.clone(),
                    key: key.to_string(),
                }
			},
			["GET", "BETWEEN", start, end] => {
				WireCommand::Range {
                    user_id: user_id.clone(),
                    start: start.to_string(),
                    end: end.to_string(),
                }		
            },
			_ => {
				println!("Invalid command!");
				continue;
			},
		};
        
        // each command by the user opens a new bi-directional connection
		let (mut send, mut recv) = conn.open_bi().await?;

		let request_str = serde_json::to_string(&request)? + "\n";
        eprintln!("Sending request: {:?}", request_str);
		send.write_all(request_str.as_bytes()).await?;
		send.finish(); // does this close the send stream? Yes it does! We make entirely new connections for each command sent by the user

		let mut reader = TokioBufReader::new(recv);
		let mut response = String::new();
		reader.read_line(&mut response).await?;

		match serde_json::from_str::<Value>(&response) {
			Ok(res) => {

                println!("Response: {:#?}", res);

                // check if exit was sent -- if so then close the connection with proper message
            },
			Err(_) => println!("Encountered Error!"),
		}

        if command_str.as_slice().first().unwrap().eq_ignore_ascii_case("exit") {
            println!("Closing connection");
            break;
        }
	}

	Ok(())
}
