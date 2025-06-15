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

#[derive(Debug, Parser)]
struct Args {
	/// Server Address
	#[arg(long)]
	server_addr: String,
	/// Path to trusted server certificate PEM file
	#[arg(long)]
	server_cert: String,
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

	repl(new_conn).await?;

	Ok(())
}

async fn repl(conn: Connection) -> Result<()> {
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

		if input.eq_ignore_ascii_case("EXIT") {
			break;
		}

		let mut command_tokens: Vec<String> =
			input.split_whitespace().map(|s| s.to_string()).collect();

		command_tokens[0] = command_tokens[0].to_uppercase();

		if (command_tokens[0] == "GET") && command_tokens.len() >= 2 {
			command_tokens[1] = command_tokens[1].to_uppercase();
		}

		let command_str: Vec<&str> = command_tokens.iter().map(|s| s.as_str()).collect();

		let request = match command_str.as_slice() {
			["PING"] => {
				json!({"command" : "PING"})
			},
			["STORE", key, value] => {
				json!({"command" : "STORE",
                "key" : key,
                "value" : value})
			},
			["FETCH", key] => {
				json!({"command" : "FETCH",
                "key" : key})
			},
			["LIST"] => {
				json!({"command" : "LIST"})
			},
			["UPDATE", key, value] => {
				json!({"command" : "UPDATE",
                "key" : key,
                "value" : value})
			},
			["DELETE", key] => {
				json!({"command" : "DELETE",
                "key" : key})
			},
			["GET", "BETWEEN", start, end] => {
				json!({
					"command": "RANGE",
					"start" : start,
					"end" : end
				})
			},
			_ => {
				println!("Invalid command!");
				continue;
			},
		};

		let (mut send, mut recv) = conn.open_bi().await?;

		let request_str = serde_json::to_string(&request)? + "\n";
		send.write_all(request_str.as_bytes()).await?;
		send.finish(); // does this close the send stream?

		let mut reader = TokioBufReader::new(recv);
		let mut response = String::new();
		reader.read_line(&mut response).await?;

		match serde_json::from_str::<Value>(&response) {
			Ok(res) => println!("Response: {:#?}", res),
			Err(_) => println!("Encountered Error!"),
		}
	}

	Ok(())
}
