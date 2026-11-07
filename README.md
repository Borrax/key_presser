# Key Presser

![rust logo](https://rust-lang.org/logos/rust-logo-512x512.png)

Run it, press a key, and from then on it presses and holds that key for 2 seconds at a random time every 30 to 60 seconds. Works on Windows, Linux and macOS.

### Usage:
```bash
cargo run --release
```
Press the key you want held, then leave it running. Stop it with `Ctrl+C`.

Use `-t <seconds>` for a fixed period instead of the random 30 to 60 seconds. The key is then pressed every `<seconds>` and held for a random time (up to 2 seconds, never longer than the period):
```bash
cargo run --release -- -t 1
```

### Testing
```bash
cargo test
```
