// Code/ROC/rocs/src/logger.rs

use crate::command::Command;
use serde_json;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;
// use std::time::{SystemTime, UNIX_EPOCH}; // “1970-01-01 00:00:00 UTC”

/// Write Ahead Logging [WAL]     
/// append a command entry of type &Command
pub(crate) fn store_log(com: &Command) {
    let log_dir = Path::new("logs");

    if !log_dir.exists() {
        fs::create_dir(log_dir).expect("Unable to create the log directory");
    }

    // let's open the log file in append mode since we need to log into the file
    let log_file_path = log_dir.join("wal.log");
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file_path)
        .expect("Failed to open file wal.log");

    let mut writer = BufWriter::new(file);

    let serialized = serde_json::to_string(com).expect("Serialization failed!");

    writer
        .write_all(serialized.as_bytes())
        .expect("Failed to write to wal.log");

    writer
        .write_all(b"\n")
        .expect("Failed to write newline to wal.log");

    writer.flush().expect("Failed to flush wal.log");
}

/// helper function for reading from the WAL log
pub(crate) fn read_wal() -> io::Result<Vec<Command>> {
    // we gotta return a vector of all the instructions

    let file_path = Path::new("../logs/wal.log");
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut entries = Vec::new();

    for line_res in reader.lines() {
        let line = line_res?;
        let trimmed = line.trim();
        if trimmed.is_empty() || !trimmed.starts_with('{') {
            continue;
        }
        match serde_json::from_str::<Command>(&line) {
            Ok(cmd) => entries.push(cmd),
            Err(e) => eprintln!("Failed to parse command from WAL: {}", e),
        }
    }
    Ok(entries)
}

/// Save a health_checkpoint to file health_checkpoints.log
///
/// "CLEAN" is stored as "0"
/// "DIRTY" is stored as "1"
pub(crate) fn save_checkpoint(msg: String) {
    eprintln!("Saving checkpoint: {:?}", msg);

    let file_path = Path::new("logs/health_checkpoints.log");

    let file = OpenOptions::new()
        .create(true)
        .truncate(true)
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
        .write_all(&[flag])
        .expect("Failed to write the flag into the health_checkpoint.log file");
    eprintln!(
        "Successfully written the flag: {:?} for {:?} into the file!",
        flag, msg
    );
    writer.flush().expect("Failed to flush the writer!");
}

/// Retrieve the last saved checkpoint for recovery checking
///
/// returns Option<String>   
///
/// Some("CLEAN") or Some("DIRTY")
pub(crate) fn get_health_checkpoint() -> Option<String> {
    eprintln!("getting health_checkpoint");

    let file_path = Path::new("logs/health_checkpoints.log");

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
            eprintln!("Found CLEAN flag! No recovery needed!");
            Some("CLEAN".to_string())
        }
        1 => {
            eprintln!("Found DIRTY flag. Recovery needed!");
            Some("DIRTY".to_string())
        }
        _other => {
            eprintln!("Invalid flag found!");
            None
        }
    }
}

pub(crate) fn clear_wal() -> io::Result<()> {
    let file_path = Path::new("logs/wal.log");

    let _file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_path)
        .expect("failed to open the file!");

    Ok(())
}
