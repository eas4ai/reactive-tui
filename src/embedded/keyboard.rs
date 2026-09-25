use super::snapshot::native_error;
use crate::error::{ReactiveError, Result};
use crate::event::types::{KeyCode, KeyEvent, KeyEventKind};
use libghostty_vt::{
    key::{self, Key},
    Terminal,
};

/// Encode one host key with the child terminal's current keyboard modes.
pub(super) fn encode(
    terminal: &Terminal<'_, '_>,
    encoder: &mut key::Encoder<'_>,
    key: &KeyEvent,
) -> Result<Vec<u8>> {
    let mut event = key::Event::new().map_err(native_error)?;
    let mut modifiers = key::Mods::empty();
    if key.modifiers.shift || key.code == KeyCode::BackTab {
        modifiers |= key::Mods::SHIFT;
    }
    if key.modifiers.ctrl {
        modifiers |= key::Mods::CTRL;
    }
    if key.modifiers.alt {
        modifiers |= key::Mods::ALT;
    }
    if key.modifiers.meta {
        modifiers |= key::Mods::SUPER;
    }
    event.set_mods(modifiers);
    event.set_action(match key.kind {
        KeyEventKind::Press => key::Action::Press,
        KeyEventKind::Release => key::Action::Release,
        KeyEventKind::Repeat => key::Action::Repeat,
    });
    let physical = match key.code {
        KeyCode::Char(ch) => {
            if ch.is_control() {
                return Err(ReactiveError::invalid_parameter(
                    "use a key code and modifiers for control characters",
                ));
            }
            event
                .set_utf8(Some(ch.to_string()))
                .set_unshifted_codepoint(ch.to_ascii_lowercase());
            character_key(ch)
        }
        KeyCode::Space => {
            event.set_utf8(Some(" "));
            Key::Space
        }
        KeyCode::Enter => Key::Enter,
        KeyCode::Escape => Key::Escape,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Delete => Key::Delete,
        KeyCode::Insert => Key::Insert,
        KeyCode::Tab | KeyCode::BackTab => Key::Tab,
        KeyCode::Up => Key::ArrowUp,
        KeyCode::Down => Key::ArrowDown,
        KeyCode::Left => Key::ArrowLeft,
        KeyCode::Right => Key::ArrowRight,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::F(n @ 1..=24) => {
            const KEYS: [Key; 24] = [
                Key::F1,
                Key::F2,
                Key::F3,
                Key::F4,
                Key::F5,
                Key::F6,
                Key::F7,
                Key::F8,
                Key::F9,
                Key::F10,
                Key::F11,
                Key::F12,
                Key::F13,
                Key::F14,
                Key::F15,
                Key::F16,
                Key::F17,
                Key::F18,
                Key::F19,
                Key::F20,
                Key::F21,
                Key::F22,
                Key::F23,
                Key::F24,
            ];
            KEYS[usize::from(n - 1)]
        }
        _ => {
            return Err(ReactiveError::invalid_parameter(
                "unsupported embedded terminal key",
            ))
        }
    };
    event.set_key(physical);
    encoder.set_options_from_terminal(terminal);
    // A key carries at most one scalar; 256 bytes also fits extended keyboard reports.
    let mut bytes = [0; 256];
    let count = encoder.encode(&event, &mut bytes).map_err(native_error)?;
    Ok(bytes[..count].to_vec())
}

fn character_key(ch: char) -> Key {
    const LETTERS: [Key; 26] = [
        Key::A,
        Key::B,
        Key::C,
        Key::D,
        Key::E,
        Key::F,
        Key::G,
        Key::H,
        Key::I,
        Key::J,
        Key::K,
        Key::L,
        Key::M,
        Key::N,
        Key::O,
        Key::P,
        Key::Q,
        Key::R,
        Key::S,
        Key::T,
        Key::U,
        Key::V,
        Key::W,
        Key::X,
        Key::Y,
        Key::Z,
    ];
    if ch.is_ascii_alphabetic() {
        return LETTERS[(ch.to_ascii_lowercase() as u8 - b'a') as usize];
    }
    match ch {
        '0' | ')' => Key::Digit0,
        '1' | '!' => Key::Digit1,
        '2' | '@' => Key::Digit2,
        '3' | '#' => Key::Digit3,
        '4' | '$' => Key::Digit4,
        '5' | '%' => Key::Digit5,
        '6' | '^' => Key::Digit6,
        '7' | '&' => Key::Digit7,
        '8' | '*' => Key::Digit8,
        '9' | '(' => Key::Digit9,
        '[' | '{' => Key::BracketLeft,
        ']' | '}' => Key::BracketRight,
        ';' | ':' => Key::Semicolon,
        '\'' | '"' => Key::Quote,
        ',' | '<' => Key::Comma,
        '.' | '>' => Key::Period,
        '/' | '?' => Key::Slash,
        '\\' | '|' => Key::Backslash,
        '-' | '_' => Key::Minus,
        '=' | '+' => Key::Equal,
        '`' | '~' => Key::Backquote,
        ' ' => Key::Space,
        _ => Key::Unidentified,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::types::KeyModifiers;
    #[test]
    fn mode_dependent_navigation_and_control_keys() {
        let mut terminal = Terminal::new(10, 4).unwrap();
        let mut encoder = key::Encoder::new().unwrap();
        let up = KeyEvent::new(KeyCode::Up);
        assert_eq!(encode(&terminal, &mut encoder, &up).unwrap(), b"\x1b[A");
        terminal.vt_write(b"\x1b[?1h");
        assert_eq!(encode(&terminal, &mut encoder, &up).unwrap(), b"\x1bOA");
        for (code, bytes) in [
            (KeyCode::Escape, b"\x1b".as_slice()),
            (KeyCode::Enter, b"\r"),
            (KeyCode::Char('é'), "é".as_bytes()),
        ] {
            assert_eq!(
                encode(&terminal, &mut encoder, &KeyEvent::new(code)).unwrap(),
                bytes
            );
        }
        let ctrl_c = KeyEvent::new(KeyCode::Char('c')).with_modifiers(KeyModifiers {
            ctrl: true,
            ..KeyModifiers::empty()
        });
        assert_eq!(encode(&terminal, &mut encoder, &ctrl_c).unwrap(), b"\x03");
    }
}
