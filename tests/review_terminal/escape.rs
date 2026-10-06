//! Part of tests/review_terminal.rs: the public escape parser.

use reactive_tui::escape::osc::OSCAction;
use reactive_tui::escape::parser::Parser;
use reactive_tui::escape::Action;

fn fed(bytes: &[u8]) -> Vec<Action> {
    Parser::new().feed(bytes)
}

/// PLT-005: the parser decodes UTF-8, whole or split, and a continuation
/// byte never starts a control sequence.
#[test]
fn plt_005_the_public_parser_decodes_utf8() {
    let actions = fed("é".as_bytes());
    assert_eq!(
        actions,
        vec![Action::Print('é')],
        "PLT-005: feeding é (C3 A9) printed {actions:?}"
    );

    let actions = fed("界".as_bytes());
    assert_eq!(
        actions,
        vec![Action::Print('界')],
        "PLT-005: feeding 界 (E7 95 8C) printed {actions:?}"
    );

    let mut parser = Parser::new();
    let mut actions = Vec::new();
    for byte in "界".as_bytes() {
        actions.extend(parser.feed(&[*byte]));
    }
    assert_eq!(
        actions,
        vec![Action::Print('界')],
        "PLT-005: feeding 界 one byte per call printed {actions:?}"
    );

    let actions = fed("ÛX".as_bytes());
    assert_eq!(
        actions,
        vec![Action::Print('Û'), Action::Print('X')],
        "PLT-005: feeding ÛX (C3 9B 58), whose second byte is also the CSI introducer, gave {actions:?}"
    );
}

/// PLT-006: BEL and `ESC \` end an OSC string, `ESC \` ends a DCS string,
/// and the text after them prints.
#[test]
fn plt_006_the_public_parser_ends_its_strings_on_bel_and_st() {
    let title = Action::OSC(OSCAction::SetTitle("t".to_string()));

    let actions = fed(b"\x1b]2;t\x07X");
    assert_eq!(
        actions,
        vec![title.clone(), Action::Print('X')],
        "PLT-006: an OSC title ended by BEL and followed by X gave {actions:?}"
    );

    let actions = fed(b"\x1b]2;t\x1b\\X");
    assert_eq!(
        actions,
        vec![title.clone(), Action::Print('X')],
        "PLT-006: an OSC title ended by ESC backslash and followed by X gave {actions:?}"
    );

    let mut parser = Parser::new();
    let mut actions = parser.feed(b"\x1b]2;t\x1b");
    actions.extend(parser.feed(b"\\X"));
    assert_eq!(
        actions,
        vec![title, Action::Print('X')],
        "PLT-006: the same title with ESC and backslash in separate feeds gave {actions:?}"
    );

    let actions = fed(b"\x1bPqx\x1b\\X");
    assert!(
        matches!(actions.as_slice(), [Action::DCS(payload), Action::Print('X')] if payload.ends_with(b"x")),
        "PLT-006: a DCS string ended by ESC backslash and followed by X gave {actions:?}"
    );
}

/// PLT-005 (the adversary's finding 1): UTF-8 is decoded inside strings too,
/// and a lone byte in 0x80..0x9F is an invalid UTF-8 byte, never a C1
/// control introducer.
#[test]
fn plt_005_utf8_inside_strings_and_no_c1_controls() {
    let actions = fed(b"\x1b]2;\xc3\x9b\x07X");
    assert_eq!(
        actions,
        vec![
            Action::OSC(OSCAction::SetTitle("Û".to_string())),
            Action::Print('X')
        ],
        "PLT-005: a title holding Û (C3 9B) followed by X gave {actions:?}"
    );

    let actions = fed(b"\x9b2A");
    assert_eq!(
        actions,
        vec![
            Action::Print('\u{FFFD}'),
            Action::Print('2'),
            Action::Print('A')
        ],
        "PLT-005: a lone 0x9B followed by 2A gave {actions:?}"
    );

    let actions = fed(b"\x1bPq\xc3\xa9\x1b\\X");
    assert!(
        matches!(actions.as_slice(), [Action::DCS(payload), Action::Print('X')] if payload.ends_with("é".as_bytes())),
        "PLT-005: a DCS payload holding é gave {actions:?}"
    );
}
