use crate::store;
use std::path::Path;
use std::thread;
use std::time::Duration;

pub fn take_snapshots<P: AsRef<Path> + Send + 'static>(snapshot_path: P, interval_secs: u64) {
    // AsRef helps here to accept different kinda parameters that can be made into a Path variable
    // Send helps us tell that any variables produced within this thread can be safely transferred
    // to any other thread ...

    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(interval_secs));

        store::save_store(&snapshot_path).expect("Failed to save the periodic snapshot");
    });
}
