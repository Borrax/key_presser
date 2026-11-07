use crate::sender::Sender;
use crossterm::event::KeyCode;
use rand::RngExt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

pub const HOLD: Duration = Duration::from_secs(2);
pub const MIN_FIXED_HOLD: Duration = Duration::from_millis(100);
pub const MIN_WAIT_SECS: u64 = 30;
pub const MAX_WAIT_SECS: u64 = 60;

/// How the presses are scheduled.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Schedule {
    /// Hold for 2s after a random 30 to 60 second wait.
    Random,
    /// Press every fixed period (start to start) and hold for a random time within it.
    Fixed(Duration),
}

impl Schedule {
    /// Returns how long to hold the key and how long to wait after releasing it.
    pub fn next(self) -> (Duration, Duration) {
        match self {
            Schedule::Random => (HOLD, random_wait()),
            Schedule::Fixed(period) => {
                let max = period.min(HOLD);
                let min = MIN_FIXED_HOLD.min(max);
                let hold = Duration::from_millis(
                    rand::rng().random_range(min.as_millis() as u64..=max.as_millis() as u64),
                );
                (hold, period - hold)
            }
        }
    }
}

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
pub fn run(key: KeyCode, name: &str, schedule: Schedule, stop: &AtomicBool) -> Result<(), Box<dyn std::error::Error>> {
    let mut sender = Sender::new(key)?;
    loop {
        let (hold, wait) = schedule.next();
        log(&format!("Next press of {name} in {:.1}s", wait.as_secs_f64()));
        if !interruptible_sleep(wait, stop) {
            break;
        }
        sender.press()?;
        log(&format!("Pressed {name}, holding for {:.1}s", hold.as_secs_f64()));
        interruptible_sleep(hold, stop);
        sender.release()?;
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
    fn random_schedule_holds_two_seconds() {
        assert_eq!(Schedule::Random.next().0, HOLD);
    }

    #[test]
    fn fixed_schedule_keeps_the_period() {
        for secs in [0.05, 1.0, 5.0] {
            let period = Duration::from_secs_f64(secs);
            for _ in 0..200 {
                let (hold, wait) = Schedule::Fixed(period).next();
                assert_eq!(hold + wait, period);
                assert!(hold <= HOLD);
            }
        }
    }

    #[test]
    fn sleep_stops_when_flagged() {
        let stop = AtomicBool::new(true);
        assert!(!interruptible_sleep(Duration::from_secs(5), &stop));
    }
}
