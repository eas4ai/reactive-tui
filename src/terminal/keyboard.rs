use crate::event::types::{KeyCode, KeyEvent, KeyEventKind};

pub(crate) fn encode(event: &KeyEvent, application_cursor: bool) -> Vec<u8> {
    if event.kind == KeyEventKind::Release {
        return Vec::new();
    }
    let modifier = 1
        + u8::from(event.modifiers.shift)
        + 2 * u8::from(event.modifiers.alt)
        + 4 * u8::from(event.modifiers.ctrl);
    let navigation = match event.code {
        KeyCode::Up => Some('A'),
        KeyCode::Down => Some('B'),
        KeyCode::Right => Some('C'),
        KeyCode::Left => Some('D'),
        KeyCode::Home => Some('H'),
        KeyCode::End => Some('F'),
        KeyCode::F(n @ 1..=4) => Some(char::from(b'P' + n - 1)),
        _ => None,
    };
    if let Some(final_byte) = navigation {
        return if modifier != 1 {
            format!("\x1b[1;{modifier}{final_byte}").into_bytes()
        } else if application_cursor || matches!(event.code, KeyCode::F(_)) {
            format!("\x1bO{final_byte}").into_bytes()
        } else {
            format!("\x1b[{final_byte}").into_bytes()
        };
    }
    let numbered = match event.code {
        KeyCode::Insert => Some(2),
        KeyCode::Delete => Some(3),
        KeyCode::PageUp => Some(5),
        KeyCode::PageDown => Some(6),
        KeyCode::F(n @ 5..=12) => Some([15, 17, 18, 19, 20, 21, 23, 24][usize::from(n - 5)]),
        _ => None,
    };
    if let Some(number) = numbered {
        return if modifier == 1 {
            format!("\x1b[{number}~").into_bytes()
        } else {
            format!("\x1b[{number};{modifier}~").into_bytes()
        };
    }
    let mut bytes = match event.code {
        KeyCode::Char(c) if event.modifiers.ctrl => match c {
            'a'..='z' | 'A'..='Z' => vec![c.to_ascii_uppercase() as u8 - b'A' + 1],
            '@' | ' ' | '2' => vec![0],
            '[' | '3' => vec![27],
            '\\' | '4' => vec![28],
            ']' | '5' => vec![29],
            '^' | '6' => vec![30],
            '_' | '7' => vec![31],
            '?' | '8' => vec![127],
            _ => c.to_string().into_bytes(),
        },
        KeyCode::Char(c) => c.to_string().into_bytes(),
        KeyCode::Enter => b"\r".to_vec(),
        KeyCode::Tab if event.modifiers.shift => b"\x1b[Z".to_vec(),
        KeyCode::Tab => b"\t".to_vec(),
        KeyCode::BackTab => b"\x1b[Z".to_vec(),
        KeyCode::Backspace => vec![127],
        KeyCode::Escape => vec![27],
        KeyCode::Space if event.modifiers.ctrl => vec![0],
        KeyCode::Space => vec![b' '],
        KeyCode::Null => vec![0],
        _ => Vec::new(),
    };
    if event.modifiers.alt && !bytes.is_empty() {
        bytes.insert(0, 27);
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::types::KeyModifiers;

    #[test]
    fn modified_navigation_and_function_keys_keep_modifiers_in_csi() {
        let modifiers = KeyModifiers {
            ctrl: true,
            alt: true,
            ..KeyModifiers::empty()
        };
        for (code, expected) in [
            (KeyCode::Left, "\x1b[1;7D"),
            (KeyCode::Home, "\x1b[1;7H"),
            (KeyCode::F(1), "\x1b[1;7P"),
            (KeyCode::F(12), "\x1b[24;7~"),
            (KeyCode::Delete, "\x1b[3;7~"),
            (KeyCode::PageDown, "\x1b[6;7~"),
        ] {
            let event = KeyEvent::new(code).with_modifiers(modifiers);
            for application in [false, true] {
                assert_eq!(encode(&event, application), expected.as_bytes());
            }
        }
        let alt_text = KeyEvent::new(KeyCode::Char('é')).with_modifiers(KeyModifiers {
            alt: true,
            ..KeyModifiers::empty()
        });
        assert_eq!(encode(&alt_text, false), "\x1bé".as_bytes());
        assert!(encode(&alt_text.with_kind(KeyEventKind::Release), false).is_empty());
    }
}
