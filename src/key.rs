use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use std::io;

/// Maps a terminal key code to the key that enigo can press.
#[cfg(not(windows))]
pub fn to_enigo(code: KeyCode) -> Option<enigo::Key> {
    use enigo::Key;
    Some(match code {
        KeyCode::Char(c) => Key::Unicode(c),
        KeyCode::Enter => Key::Return,
        KeyCode::Esc => Key::Escape,
        KeyCode::Tab => Key::Tab,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Delete => Key::Delete,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::Up => Key::UpArrow,
        KeyCode::Down => Key::DownArrow,
        KeyCode::Left => Key::LeftArrow,
        KeyCode::Right => Key::RightArrow,
        KeyCode::F(n) => match n {
            1 => Key::F1,
            2 => Key::F2,
            3 => Key::F3,
            4 => Key::F4,
            5 => Key::F5,
            6 => Key::F6,
            7 => Key::F7,
            8 => Key::F8,
            9 => Key::F9,
            10 => Key::F10,
            11 => Key::F11,
            12 => Key::F12,
            _ => return None,
        },
        _ => return None,
    })
}

/// Whether the key can be pressed on this platform.
pub fn supported(code: KeyCode) -> bool {
    crate::sender::resolve(code).is_some()
}

/// Blocks until the user presses a supported key and returns it with a printable name.
pub fn capture() -> io::Result<(KeyCode, String)> {
    crossterm::terminal::enable_raw_mode()?;
    let result = loop {
        match event::read() {
            Ok(Event::Key(k)) if k.kind == KeyEventKind::Press => {
                if k.code == KeyCode::Char('c') && k.modifiers.contains(KeyModifiers::CONTROL) {
                    crossterm::terminal::disable_raw_mode()?;
                    std::process::exit(0);
                }
                if supported(k.code) {
                    break Ok((k.code, format!("{:?}", k.code)));
                }
            }
            Ok(_) => {}
            Err(e) => break Err(e),
        }
    };
    crossterm::terminal::disable_raw_mode()?;
    result
}

#[cfg(all(test, not(windows)))]
mod tests {
    use super::*;
    use enigo::Key;

    #[test]
    fn maps_characters() {
        assert!(matches!(to_enigo(KeyCode::Char('a')), Some(Key::Unicode('a'))));
    }

    #[test]
    fn rejects_unsupported_keys() {
        assert!(to_enigo(KeyCode::F(30)).is_none());
        assert!(to_enigo(KeyCode::Null).is_none());
    }
}
