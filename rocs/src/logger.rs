// Code/ROC/rocs/src/logger.rs

use bincode;
use serde_json::{Result, Value};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Result, Write};
use std::path::Path;

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

    // Write to file
    writer
        .write_all(&encoded)
        .expect("failed to write into the log file");
    writer.flush().expect("Failed to flush into wal log");
}

pub(crate) fn read_wal() -> io::Result<Vec<Value>, io::Error> {
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

    entries
}
