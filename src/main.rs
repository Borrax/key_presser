mod key;
mod presser;

use presser::Schedule;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Parses the command line: `-t <seconds>` switches to a fixed period.
fn parse_args(args: &[String]) -> Result<Schedule, String> {
    match args {
        [] => Ok(Schedule::Random),
        [flag, value] if flag == "-t" => match value.parse::<f64>() {
            Ok(secs) if secs.is_finite() && secs > 0.0 => {
                Ok(Schedule::Fixed(std::time::Duration::from_secs_f64(secs)))
            }
            _ => Err(format!("Invalid period '{value}', expected seconds greater than 0")),
        },
        _ => Err("Usage: key_presser [-t <seconds>]".to_string()),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let schedule = match parse_args(&args) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };

    println!("Press the key to hold (Ctrl+C to quit)...");
    let (key, name) = match key::capture() {
        Ok(k) => k,
        Err(e) => {
            eprintln!("Could not read the key: {e}");
            std::process::exit(1);
        }
    };
    match schedule {
        Schedule::Random => println!("Holding {name} for 2s every 30-60s"),
        Schedule::Fixed(p) => println!("Pressing {name} every {:.1}s", p.as_secs_f64()),
    }

    let stop = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&stop);
    ctrlc::set_handler(move || flag.store(true, Ordering::SeqCst)).expect("Could not set the Ctrl+C handler");

    if let Err(e) = presser::run(key, &name, schedule, &stop) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn args(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn no_args_is_random() {
        assert_eq!(parse_args(&[]), Ok(Schedule::Random));
    }

    #[test]
    fn t_flag_sets_fixed_period() {
        assert_eq!(parse_args(&args(&["-t", "1"])), Ok(Schedule::Fixed(Duration::from_secs(1))));
    }

    #[test]
    fn rejects_bad_input() {
        assert!(parse_args(&args(&["-t", "0"])).is_err());
        assert!(parse_args(&args(&["-t", "abc"])).is_err());
        assert!(parse_args(&args(&["-t"])).is_err());
        assert!(parse_args(&args(&["-x", "1"])).is_err());
    }
}
