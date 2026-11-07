mod key;
mod presser;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

fn main() {
    println!("Press the key to hold (Ctrl+C to quit)...");
    let (key, name) = match key::capture() {
        Ok(k) => k,
        Err(e) => {
            eprintln!("Could not read the key: {e}");
            std::process::exit(1);
        }
    };
    println!("Holding {name} for 2s every 30-60s");

    let stop = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&stop);
    ctrlc::set_handler(move || flag.store(true, Ordering::SeqCst)).expect("Could not set the Ctrl+C handler");

    if let Err(e) = presser::run(key, &stop) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
