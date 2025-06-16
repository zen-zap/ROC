//! in src/lib.rs
#![allow(unused)]

use serde::{Deserialize, Serialize};
use quinn::Connection;
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as TokioBufReader};
use std::path::{Path, PathBuf};
use std::fs;
use anyhow::Result;
use directories::UserDirs;

pub type UserId = String;

/// The wire-format for user-accessible commands. Only user commands included.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WireCommand {
    Hi { user_id: Option<UserId> },
    Ping { user_id: UserId },
    Set { user_id: UserId, key: String, value: usize },
    Get { user_id: UserId, key: String },
    Del { user_id: UserId, key: String },
    Update { user_id: UserId, key: String, value: usize },
    Range { user_id: UserId, start: String, end: String },
    List { user_id: UserId },
    Exit { user_id: UserId },
}

async fn hi_handshake(conn: &Connection) -> Result<String> {
    let (mut send, mut recv) = conn.open_bi().await?;
    let hi = serde_json::to_string(&WireCommand::Hi { user_id: None })? + "\n";
    send.write_all(hi.as_bytes()).await?;
    send.finish();

    let mut reader = TokioBufReader::new(recv);
    let mut response = String::new();
    reader.read_line(&mut response).await?;
    let v: Value = serde_json::from_str(&response)?;
    let user_id = v["user_id"].as_str().ok_or_else(|| anyhow::anyhow!("No user_id in response"))?;
    println!("Assigned user_id: {user_id}");
    Ok(user_id.to_string())
}

pub async fn get_user_id(conn: &Connection, test_mode: bool) -> Result<String> {
    if test_mode {
        // Always request a new uuid, don't persist
        hi_handshake(conn).await
    } else {
        // Persistent logic as above
        get_or_create_user_id(conn).await
    }
}

async fn get_or_create_user_id(conn: &Connection) -> Result<String> {
    let path = user_id_file_path()?;

    // Try to read existing user_id
    if path.exists() {
        let id = fs::read_to_string(&path)?;
        let trimmed = id.trim();
        if !trimmed.is_empty() {
            println!("Using stored user_id: {}", trimmed);
            return Ok(trimmed.to_string());
        }
    }

    // File missing or empty, do handshake
    let user_id = hi_handshake(conn).await?;
    write_user_id(&path, &user_id)?; // Save for next time
    println!("Saved new user_id to {}", path.display());
    Ok(user_id)
}

fn user_id_file_path() -> Result<PathBuf> {
    let user_dirs = UserDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let dir = user_dirs.home_dir().join(".roc_client");
    if !dir.exists() {
        fs::create_dir_all(&dir)?; // Recursively create .roc_client
    }
    Ok(dir.join("user_id.crd"))
}

fn write_user_id(path: &Path, user_id: &str) -> Result<()> {
    use std::io::Write;
    let mut file = fs::File::create(path)?;
    file.write_all(user_id.as_bytes())?;
    Ok(())
}
