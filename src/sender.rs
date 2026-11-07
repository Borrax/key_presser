//! Sends the key press and release to the OS.
//! On Windows it sends hardware scancodes through `SendInput`, because games using
//! DirectInput or Raw Input ignore virtual-key events. Elsewhere it uses enigo.

use crossterm::event::KeyCode;

type Res = Result<(), Box<dyn std::error::Error>>;

#[cfg(not(windows))]
mod imp {
    use super::*;
    use enigo::{Direction, Enigo, Key, Keyboard, Settings};

    pub fn resolve(code: KeyCode) -> Option<Key> {
        crate::key::to_enigo(code)
    }

    pub struct Sender {
        enigo: Enigo,
        key: Key,
    }

    impl Sender {
        pub fn new(code: KeyCode) -> Result<Self, Box<dyn std::error::Error>> {
            let key = resolve(code).ok_or("Unsupported key")?;
            Ok(Self { enigo: Enigo::new(&Settings::default())?, key })
        }

        pub fn press(&mut self) -> Res {
            Ok(self.enigo.key(self.key, Direction::Press)?)
        }

        pub fn release(&mut self) -> Res {
            Ok(self.enigo.key(self.key, Direction::Release)?)
        }
    }
}

#[cfg(windows)]
mod imp {
    use super::*;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;

    /// A physical key: its scancode and whether it is an extended (0xE0) key.
    #[derive(Clone, Copy, Debug)]
    pub struct Scan {
        code: u16,
        extended: bool,
    }

    fn vk_of(code: KeyCode) -> Option<(u16, bool)> {
        Some(match code {
            KeyCode::Char(c) => {
                let vk = unsafe { VkKeyScanW(c as u16) };
                if vk == -1 {
                    return None;
                }
                ((vk & 0xff) as u16, false)
            }
            KeyCode::Enter => (VK_RETURN, false),
            KeyCode::Esc => (VK_ESCAPE, false),
            KeyCode::Tab => (VK_TAB, false),
            KeyCode::Backspace => (VK_BACK, false),
            KeyCode::Delete => (VK_DELETE, true),
            KeyCode::Home => (VK_HOME, true),
            KeyCode::End => (VK_END, true),
            KeyCode::PageUp => (VK_PRIOR, true),
            KeyCode::PageDown => (VK_NEXT, true),
            KeyCode::Up => (VK_UP, true),
            KeyCode::Down => (VK_DOWN, true),
            KeyCode::Left => (VK_LEFT, true),
            KeyCode::Right => (VK_RIGHT, true),
            KeyCode::F(n @ 1..=12) => (VK_F1 + (n as u16 - 1), false),
            _ => return None,
        })
    }

    pub fn resolve(code: KeyCode) -> Option<Scan> {
        let (vk, extended) = vk_of(code)?;
        let scan = unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC) } as u16;
        if scan == 0 {
            return None;
        }
        Some(Scan { code: scan, extended })
    }

    pub struct Sender {
        scan: Scan,
    }

    impl Sender {
        pub fn new(code: KeyCode) -> Result<Self, Box<dyn std::error::Error>> {
            Ok(Self { scan: resolve(code).ok_or("Unsupported key")? })
        }

        fn send(&self, up: bool) -> Res {
            let mut flags = KEYEVENTF_SCANCODE;
            if self.scan.extended {
                flags |= KEYEVENTF_EXTENDEDKEY;
            }
            if up {
                flags |= KEYEVENTF_KEYUP;
            }
            let input = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT { wVk: 0, wScan: self.scan.code, dwFlags: flags, time: 0, dwExtraInfo: 0 },
                },
            };
            let sent = unsafe { SendInput(1, &input, std::mem::size_of::<INPUT>() as i32) };
            if sent == 1 { Ok(()) } else { Err("SendInput was blocked by Windows".into()) }
        }

        pub fn press(&mut self) -> Res {
            self.send(false)
        }

        pub fn release(&mut self) -> Res {
            self.send(true)
        }
    }
}

pub use imp::Sender;

/// Returns Some if the key can be pressed on this platform.
pub fn resolve(code: KeyCode) -> Option<()> {
    imp::resolve(code).map(|_| ())
}
