use serde::{self, Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    Ping,
    Store { key: String, value: String },
    Fetch { key: String, value: Option<String> },
    Update { key: String, value: String },
    Delete { key: String },
    List { entries: Vec<(String, String)> },
    Shutdown,
    Crash,
    ERR { msg: String },
}
