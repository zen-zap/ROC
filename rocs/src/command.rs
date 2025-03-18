use serde::{self, Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    Ping,
    Store {
        key: String,
        value: usize,
    },
    Fetch {
        key: String,
        value: Option<usize>,
    },
    Update {
        key: String,
        value: usize,
    },
    Delete {
        key: String,
    },
    Range {
        start: usize,
        end: usize,
        result: Vec<(String, usize)>,
    },
    List {
        entries: Vec<(String, usize)>,
    },
    Shutdown,
    Crash,
    ERR {
        msg: String,
    },
}
