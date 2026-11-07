use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use rand::RngExt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

pub const HOLD: Duration = Duration::from_secs(2);
pub const MIN_WAIT_SECS: u64 = 30;
pub const MAX_WAIT_SECS: u64 = 60;

/// Picks a random wait between the minimum and maximum interval.
pub fn random_wait() -> Duration {
    Duration::from_secs(rand::rng().random_range(MIN_WAIT_SECS..=MAX_WAIT_SECS))
}

/// Prints a message prefixed with the local time.
fn log(msg: &str) {
    println!("[{}] {msg}", chrono::Local::now().format("%H:%M:%S"));
}

/// Sleeps in short slices so a stop request is noticed quickly.
/// Returns false if the stop flag was raised.
fn interruptible_sleep(total: Duration, stop: &AtomicBool) -> bool {
    let end = Instant::now() + total;
    while Instant::now() < end {
        if stop.load(Ordering::SeqCst) {
            return false;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    !stop.load(Ordering::SeqCst)
}

/// Presses and holds the key at random intervals until the stop flag is raised.
/// The key is always released before returning.
pub fn run(key: Key, name: &str, stop: &AtomicBool) -> Result<(), Box<dyn std::error::Error>> {
    let mut enigo = Enigo::new(&Settings::default())?;
    loop {
        let wait = random_wait();
        log(&format!("Next press of {name} in {}s", wait.as_secs()));
        if !interruptible_sleep(wait, stop) {
            break;
        }
        enigo.key(key, Direction::Press)?;
        log(&format!("Pressed {name}, holding for {}s", HOLD.as_secs()));
        interruptible_sleep(HOLD, stop);
        enigo.key(key, Direction::Release)?;
        log(&format!("Released {name}"));
    }
    log("Stopped");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wait_stays_within_bounds() {
        for _ in 0..1000 {
            let w = random_wait().as_secs();
            assert!((MIN_WAIT_SECS..=MAX_WAIT_SECS).contains(&w));
        }
    }

    #[test]
    fn sleep_stops_when_flagged() {
        let stop = AtomicBool::new(true);
        assert!(!interruptible_sleep(Duration::from_secs(5), &stop));
    }
}
