// Code/ROC/rocs/src/logger.rs

use bincode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::{self, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH}; // “1970-01-01 00:00:00 UTC”

// Write Ahead Logging [WAL]     --- used for recovery and such
pub(crate) fn store_log(com: &Value) {
    let log_dir = Path::new("../logs");

    // to check if the log directory exists
    if !log_dir.exists() {
        std::fs::create_dir(log_dir).expect("Unable to create the log directory");
    }

    // let's open the log file in append mode since we need to log into the file
    let log_file_path = log_dir.join("wal.log");
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file_path)
        .expect("Failed to open file wal.log");

    let mut writer = BufWriter::new(file);

    // convert the received &Value into a binary format
    let encoded: Vec<u8> =
        bincode::serialize(com).expect("Failed to convert the json into binary ..");

    let len = encoded.len() as u32;
    let len_header = len.to_le_bytes();

    // writing the len_header first
    writer
        .write_all(&len_header)
        .expect("Failed to write the length header into the log file");

    // Write to wal.log
    writer
        .write_all(&encoded)
        .expect("failed to write the reader data into the log file");
    writer.flush().expect("Failed to flush into wal.log");
}

pub(crate) fn read_wal() -> io::Result<Vec<Value>> {
    // we gotta return a vector of all the instructions .. then there will probably some recoverer
    // that applies them to the database
    let file_path = Path::new("../logs/wal.log");
    let data = match fs::read(file_path) {
        Ok(data) => data,
        Err(e) => return Err(e),
    };

    let mut entries: Vec<Value> = Vec::new();

    let mut slice = &data[..];

    while !slice.is_empty() {
        match bincode::deserialize::<Value>(&mut slice) {
            Ok(entry) => entries.push(entry),
            Err(e) => {
                eprintln!("Failed to deserialize, encountered error: {}", e);
                break;
            } // corrupt log -- stop the deserialization
        }
    }

    Ok(entries)
}

#[derive(Debug, Serialize, Deserialize)]
struct HealthEntry {
    timestamp: u64,
    status: String, // "CLEAN" or "DIRTY"
}

pub(crate) fn save_checkpoint(msg: String) {
    let file_path = Path::new("../logs/health_checkpoints.log");
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)
        .expect("Failed to create the health_checkpoints.log file");

    let mut writer = BufWriter::new(file);

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let mut health_entry = HealthEntry {
        timestamp: timestamp,
        status: msg,
    };

    let encoded: Vec<u8> =
        bincode::serialize(&health_entry).expect("Failed to encode the health check");

    writer
        .write_all(&encoded)
        .expect("Failed to write checkpoints");

    writer
        .flush()
        .expect("Failed to flush writer for health_checkpoints");
}

fn health_check() -> Option<String> {
    let file_path = Path::new("../logs/health_checkpoints.log");

    let mut data = match fs::read(file_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("encountered error : {}", e);
            return None;
        }
    };

    let mut last_status: Option<String> = None;

    let mut slice = &data[..];

    while !slice.is_empty() {
        match bincode::deserialize::<HealthEntry>(&mut slice) {
            Ok(entry) => {
                last_status = Some(entry.status);
            }
            Err(e) => {
                eprintln!("Failed to get the last checkpoint");
                break;
            }
        }
    }

    last_status
}
