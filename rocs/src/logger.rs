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
    // TODO: try fiddling with the buffer length here

    writer.flush().expect("Failed to flush into wal.log");
}

pub(crate) fn read_wal() -> io::Result<Vec<Value>> {
    // we gotta return a vector of all the instructions
    use std::convert::TryInto;

    let file_path = Path::new("../logs/wal.log");
    let data = fs::read(file_path)?;

    let mut entries: Vec<Value> = Vec::new();
    let mut offset = 0;

    while offset < data.len() {
        if offset + 4 > data.len() {
            eprintln!("Incomplete header found in the wal.log file");
            break;
        }

        let header_bytes = &data[offset..(offset + 4)];
        let record_len = u32::from_le_bytes(header_bytes.try_into().unwrap()) as usize;

        offset += 4; // adjusted to the end of the offset

        if offset + record_len > data.len() {
            eprintln!(
                "incomplete record found! Expected {} bytes, got {} bytes",
                record_len,
                data.len() - offset
            );
            break;
        }

        let record_bytes = &data[offset..offset + record_len];
        offset += record_len;

        // okay .. let's get the word now
        match bincode::deserialize::<Value>(record_bytes) {
            Ok(record) => {
                entries.push(record);
            }
            Err(e) => {
                eprintln!("Failed to deserialize the bytes:   {}", e);
                break;
            }
        }
    }

    Ok(entries)
}

/// type for health_checkpoints
#[derive(Debug, Serialize, Deserialize)]
struct HealthEntry {
    /// u64
    timestamp: u64,
    /// String
    status: String, // "CLEAN" or "DIRTY"
}

/*
    pub(crate) fn save_checkpoint(msg: String) {
        eprintln!("saving health_checkpoint");

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

        let health_entry = HealthEntry {
            timestamp: timestamp,
            status: msg,
        };

        eprintln!("Successfully saved checkpoint! {:#?}", health_entry);

        let encoded: Vec<u8> =
            bincode::serialize(&health_entry).expect("Failed to encode the health check");

        writer
            .write_all(&encoded)
            .expect("Failed to write checkpoints");

        writer
            .flush()
            .expect("Failed to flush writer for health_checkpoints");
    }
**/

pub(crate) fn save_checkpoint(msg: String) {
    eprintln!("Saving checkpoint: {:?}", msg);
    let file_path = Path::new("../logs/health_checkpoints.log");
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(false)
        .open(file_path)
        .expect("Failed to open the check file!");

    let flag: u8 = if msg.eq_ignore_ascii_case("CLEAN") {
        0
    } else {
        1
    };

    let mut writer = BufWriter::new(file);

    writer
        .write(&[flag])
        .expect("Failed to write the flag into the health_checkpoint.log file");
    eprintln!(
        "Successfully writtern the flag: {:?} for {:?} into the file!",
        flag, msg
    );
    writer.flush().expect("Failed to flush the writer!");
}

pub(crate) fn get_health_checkpoint() -> Option<String> {
    eprintln!("getting health_checkpoint");

    let file_path = Path::new("../logs/health_checkpoints.log");

    let data = match std::fs::read(file_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!(
                "encountered error: {} while reading data from the checkpoint file!",
                e
            );
            return None;
        }
    };

    if data.is_empty() {
        eprintln!("Empty health_checkpoints.log file!");
        return None;
    }

    match data[0] {
        0 => {
            eprintln!("Found CLEAN flag! Non reovery needed!");
            Some("CLEAN".to_string())
        }
        1 => {
            eprintln!("Found DIRTY flag. Recovery needed!");
            Some("DIRTY".to_string())
        }
        other => {
            eprintln!("Invalid flag found!");
            None
        }
    }
}

/*
 *let data = match fs::read(file_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("encountered error : {}", e);
            return None;
        }
    };

    let mut entries: Vec<HealthEntry> = Vec::new();

    let mut slice = &data[..];

    while !slice.is_empty() {
        eprintln!("checking the data in the wal.log");
        match bincode::deserialize::<HealthEntry>(&mut slice) {
            Ok(entry) => {
                eprintln!("Deserialized checkpoint: {:#?}", entry);
                entries.push(entry);
            }
            Err(e) => {
                eprintln!("Failed to get the last checkpoint: {}", e);
                break;
            }
        }
    }

    entries.last().map(|entry| entry.status.clone())
**/
