use super::{app_input, key, Control};
use reactive_tui::{
    component::Element,
    event::types::{Event, KeyCode, ResizeEvent},
    widgets::{TerminalProps, TerminalWidget},
};

#[test]
#[cfg(windows)]
fn terminal_widget_windows_app_routes_input_and_paints_after_resize() {
    use reactive_tui::event::types::PasteEvent;
    for size in [(32, 8), (44, 12)] {
        let directory = tempfile::tempdir().unwrap();
        let props = TerminalProps {
            shell_command: Some(std::env::var("COMSPEC").expect("Windows command interpreter")),
            working_directory: Some(directory.path().to_str().unwrap().into()),
            env_vars: vec![("PROMPT".into(), "READY$G".into())],
            ..Default::default()
        };
        let resized = (size.0 + 4, size.1 + 2);
        // ConPTY may deliver the number before the erase/cursor operations
        // completing that console update. Wait for the complete result row.
        let final_result = format!("\n53{}│", " ".repeat(usize::from(resized.0 - 3)));
        let frames = app_input::run_when_for(
            Control(Element::typed::<TerminalWidget>(props)),
            size,
            vec![
                (
                    "READY>",
                    Some(Event::Paste(PasteEvent::new("@set /a 41+".into()))),
                ),
                ("READY>", key(KeyCode::Char('1'))),
                ("READY>", key(KeyCode::Enter)),
                (
                    "42",
                    Some(Event::Resize(ResizeEvent::new(resized.0, resized.1))),
                ),
                (
                    "READY>",
                    Some(Event::Paste(PasteEvent::new("@set /a 52+1".into()))),
                ),
                ("READY>", key(KeyCode::Enter)),
                (&final_result, None),
            ],
            // Native cmd startup exceeded the generic three-second App fixture
            // budget; keep the whole two-command workflow bounded to ten seconds.
            std::time::Duration::from_secs(10),
        );
        // The expected answers never occur in the submitted commands. An echoed
        // command alone cannot satisfy these assertions.
        let has_result = |frame: &app_input::Snapshot, expected: &str| {
            // The default widget reserves the final column for its scrollbar.
            let columns = usize::from(frame.screen.size().1.saturating_sub(1));
            frame
                .text
                .lines()
                .any(|line| line.chars().take(columns).collect::<String>().trim() == expected)
        };
        assert!(
            frames.iter().any(|frame| has_result(frame, "42")),
            "{}",
            frames.last().unwrap().text
        );
        let last = frames.last().unwrap();
        assert!(has_result(last, "53"), "{}", last.text);
        assert_eq!(last.screen.size(), (resized.1, resized.0));
        assert!(!last.text.contains("Terminal error"));
    }
}

#[cfg(unix)]
fn shell(body: &str) -> (tempfile::TempDir, TerminalProps) {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("shell");
    std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    let props = TerminalProps {
        shell_command: Some(path.to_str().unwrap().into()),
        working_directory: Some(dir.path().to_str().unwrap().into()),
        env_vars: vec![("WIDGET_VALUE".into(), "configured".into())],
        ..Default::default()
    };
    (dir, props)
}

#[test]
#[cfg(target_os = "linux")]
fn terminal_widget_paints_child_output_routes_input_resizes_and_reaps() {
    for size in [(28, 8), (42, 12)] {
        let (dir, props) = shell(
            r#"
stty -echo
echo $$ > child-pid
printf '\033[31mREADY:%s\033[0m\r\n' "$WIDGET_VALUE"
while IFS= read -r value; do
    printf 'INPUT:%s\r\n' "$value"
    printf 'DIMS:'; stty size
done
"#,
        );
        let next = (size.0 + 3, size.1 + 2);
        let dims = format!("DIMS:{} {}", size.1 - 1, size.0 - 1);
        let resized = format!("DIMS:{} {}", next.1 - 1, next.0 - 1);
        let frames = app_input::run_when(
            Control(Element::typed::<TerminalWidget>(props)),
            size,
            vec![
                ("READY:configured", key(KeyCode::Char('q'))),
                ("READY:configured", key(KeyCode::Enter)),
                (&dims, Some(Event::Resize(ResizeEvent::new(next.0, next.1)))),
                (&dims, key(KeyCode::Char('z'))),
                (&dims, key(KeyCode::Enter)),
                (&resized, None),
            ],
        );
        assert!(frames.iter().any(|f| f.text.contains("INPUT:q")));
        assert!(frames.iter().any(|f| f.text.contains("INPUT:z")));
        let ready = frames
            .iter()
            .find(|f| f.text.contains("READY:configured"))
            .unwrap();
        assert_eq!(ready.screen.cell(1, 0).unwrap().contents(), "R");
        assert_eq!(
            ready.screen.cell(1, 0).unwrap().fgcolor(),
            vt100::Color::Rgb(128, 0, 0)
        );
        let pid = std::fs::read_to_string(dir.path().join("child-pid")).unwrap();
        assert!(
            !std::path::Path::new(&format!("/proc/{}", pid.trim())).exists(),
            "child survived App removal"
        );
    }
}

#[test]
#[cfg(unix)]
fn terminal_widget_launch_failure_is_visible() {
    let frames = app_input::run_when(
        Control(Element::typed::<TerminalWidget>(TerminalProps {
            shell_command: Some("/nonexistent/reactive-tui-terminal-shell".into()),
            ..Default::default()
        })),
        (60, 6),
        vec![("Terminal error:", None)],
    );
    assert!(frames.last().unwrap().text.contains("Terminal error:"));
}

#[test]
#[cfg(unix)]
fn terminal_widget_paints_extended_colors_unicode_and_hidden_text() {
    for size in [(28, 8), (42, 12)] {
        let (_dir, props) = shell(
            r#"
printf '\033[38;2;12;34;56;48;5;201;1;3;4m界é\033[0m\r\n'
printf '\033[2;38;2;200;100;50mDIM\033[0m\r\n'
printf '\033[8mSECRET\033[0mVISIBLE\r\n'
printf 'READY\r\n'
while IFS= read -r value; do :; done
"#,
        );
        let frames = app_input::run_when(
            Control(Element::typed::<TerminalWidget>(props)),
            size,
            vec![("READY", None)],
        );
        let frame = frames.last().unwrap();
        let wide = frame.screen.cell(1, 0).unwrap();
        assert_eq!(wide.contents(), "界");
        assert!(wide.is_wide());
        assert_eq!(wide.fgcolor(), vt100::Color::Rgb(12, 34, 56));
        assert_eq!(wide.bgcolor(), vt100::Color::Rgb(255, 0, 255));
        assert!(wide.bold() && wide.italic() && wide.underline());
        assert_eq!(frame.screen.cell(1, 2).unwrap().contents(), "é");
        assert_eq!(
            frame.screen.cell(2, 0).unwrap().fgcolor(),
            vt100::Color::Rgb(100, 50, 25)
        );
        assert!(!frame.text.contains("SECRET"));
        assert!(frame.text.contains("VISIBLE"));
        assert_eq!(frame.screen.cell(3, 6).unwrap().contents(), "V");
    }
}

#[test]
#[cfg(unix)]
fn terminal_widget_restores_main_screen_after_alternate_screen() {
    for size in [(28, 8), (42, 12)] {
        let (_dir, props) = shell(
            r#"
stty -echo
printf 'MAIN\033[?1049h\033[?25lALT-READY'
IFS= read -r value
printf '\033[?1049l\033[?25h-RESTORED'
while IFS= read -r value; do :; done
"#,
        );
        let frames = app_input::run_when(
            Control(Element::typed::<TerminalWidget>(props)),
            size,
            vec![("ALT-READY", key(KeyCode::Enter)), ("MAIN-RESTORED", None)],
        );
        let alternate = frames
            .iter()
            .find(|frame| frame.text.contains("ALT-READY"))
            .unwrap();
        assert!(!alternate.text.contains("MAIN"));
        assert!(!frames.last().unwrap().text.contains("ALT-READY"));
    }
}

#[test]
#[cfg(unix)]
fn terminal_widget_uses_child_cursor_and_paste_modes() {
    use reactive_tui::event::types::{KeyEvent, KeyModifiers, PasteEvent};
    for size in [(28, 8), (42, 12)] {
        let (_dir, props) = shell(
            r#"
stty raw -echo
bytes() { dd bs=1 count="$1" 2>/dev/null | od -An -tx1 | tr -d ' \n'; }
printf '\033[?1hAPP-READY\r\n'
[ "$(bytes 3)" = 1b4f41 ] || exit 1
printf '\033[?1lNORMAL-READY\r\n'
[ "$(bytes 3)" = 1b5b41 ] || exit 2
printf 'MOD-READY\r\n'
[ "$(bytes 6)" = 1b5b313b3641 ] || exit 3
printf '\033[?2004hPASTE-READY\r\n'
[ "$(bytes 14)" = 1b5b3230307ec3a91b5b3230317e ] || exit 4
printf '\033[?2004lPLAIN-READY\r\n'
[ "$(bytes 2)" = c3a9 ] || exit 5
printf 'INPUT-PASS\r\n'
while IFS= read -r value; do :; done
"#,
        );
        let frames = app_input::run_when(
            Control(Element::typed::<TerminalWidget>(props)),
            size,
            vec![
                ("APP-READY", key(KeyCode::Up)),
                ("NORMAL-READY", key(KeyCode::Up)),
                (
                    "MOD-READY",
                    Some(Event::Key(KeyEvent::new(KeyCode::Up).with_modifiers(
                        KeyModifiers {
                            shift: true,
                            ctrl: true,
                            ..KeyModifiers::empty()
                        },
                    ))),
                ),
                (
                    "PASTE-READY",
                    Some(Event::Paste(PasteEvent::new("é".into()))),
                ),
                (
                    "PLAIN-READY",
                    Some(Event::Paste(PasteEvent::new("é".into()))),
                ),
                ("INPUT-PASS", None),
            ],
        );
        let last = &frames.last().unwrap().text;
        assert!(
            last.contains("INPUT-PASS"),
            "child shell verified every cursor and paste mode at {size:?}:\n{last}"
        );
    }
}

#[cfg(unix)]
struct ChangingTerminal {
    props: TerminalProps,
    removed: bool,
}
#[cfg(unix)]
impl reactive_tui::app::RootComponent for ChangingTerminal {
    fn render(&self) -> Element {
        if self.removed {
            return Element::text("REMOVED");
        }
        Element::typed::<TerminalWidget>(self.props.clone()).with_key("terminal")
    }
    fn try_handle_event(
        &mut self,
        event: &Event,
    ) -> reactive_tui::error::Result<reactive_tui::event::router::EventResult> {
        use reactive_tui::event::router::EventResult;
        if let Event::Custom(event) = event {
            match event.name.as_str() {
                "title" => {
                    self.props.title = "RENAMED".into();
                    self.props.show_scrollbar = false;
                }
                "restart" => self
                    .props
                    .env_vars
                    .push(("GENERATION".into(), "two".into())),
                "remove" => self.removed = true,
                _ => return Ok(EventResult::Ignored),
            }
            return Ok(EventResult::Handled);
        }
        Ok(EventResult::Ignored)
    }
    fn wake_driven(&self) -> bool {
        true
    }
}
#[cfg(unix)]
fn change(name: &str) -> Option<Event> {
    Some(Event::Custom(reactive_tui::event::types::CustomEvent::new(
        name,
        vec![],
    )))
}

#[test]
#[cfg(target_os = "linux")]
fn terminal_widget_display_changes_retain_child_launch_changes_replace_and_removal_reaps() {
    let (dir, props) = shell(
        r#"
echo $$ >> pids
printf 'GEN:%s\r\n' "${GENERATION:-one}"
while IFS= read -r value; do printf 'ACK:%s\r\n' "$value"; done
"#,
    );
    app_input::run_when(
        ChangingTerminal {
            props,
            removed: false,
        },
        (32, 8),
        vec![
            ("GEN:one", change("title")),
            ("RENAMED", key(KeyCode::Char('a'))),
            ("RENAMED", key(KeyCode::Enter)),
            ("ACK:a", change("restart")),
            ("GEN:two", change("remove")),
            ("REMOVED", None),
        ],
    );
    let pids = std::fs::read_to_string(dir.path().join("pids")).unwrap();
    assert_eq!(
        pids.lines().count(),
        2,
        "display props restarted the child: {pids}"
    );
    for pid in pids.lines() {
        assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists());
    }
}

#[test]
#[cfg(unix)]
fn terminal_widget_input_uses_focus_and_ignores_release_events() {
    use reactive_tui::event::types::{KeyEvent, KeyEventKind};
    let (_dir, mut props) = shell(
        r#"
stty -echo
printf 'READY\r\n'
while IFS= read -r value; do printf 'ACK:%s\r\n' "$value"; done
"#,
    );
    props.auto_focus = false;
    let element = reactive_tui::builder::div()
        .class("w-full h-full flex-col")
        .child(
            reactive_tui::builder::button()
                .text("OTHER")
                .class("w-8 h-1 p-0")
                .build()
                .auto_focus(),
        )
        .child(Element::typed::<TerminalWidget>(props).with_class("flex-1"))
        .build();
    let frames = app_input::run_when(
        Control(element),
        (28, 8),
        vec![
            ("READY", key(KeyCode::Char('x'))),
            ("READY", key(KeyCode::Tab)),
            (
                "READY",
                Some(Event::Key(
                    KeyEvent::new(KeyCode::Char('q')).with_kind(KeyEventKind::Release),
                )),
            ),
            ("READY", key(KeyCode::Char('a'))),
            ("READY", key(KeyCode::Enter)),
            ("ACK:a", None),
        ],
    );
    assert!(!frames.last().unwrap().text.contains("ACK:xa"));
    assert!(!frames.last().unwrap().text.contains("ACK:qa"));
}

#[test]
#[cfg(unix)]
fn terminal_widget_scrolls_history_using_signed_wheel_at_two_sizes() {
    use reactive_tui::event::types::{MouseEvent, Position, WheelDelta, WheelEvent, WheelPhase};
    for size in [(24, 6), (36, 10)] {
        let (_dir, props) = shell(
            r#"
i=0
while [ "$i" -lt 30 ]; do printf 'LINE%02d\r\n' "$i"; i=$((i+1)); done
while IFS= read -r value; do :; done
"#,
        );
        let wheel = |delta| {
            Some(Event::Mouse(MouseEvent::wheel(
                Position::cell(2, 2),
                WheelEvent {
                    delta: WheelDelta::Lines { x: 0.0, y: delta },
                    phase: WheelPhase::Changed,
                },
            )))
        };
        let frames = app_input::run_visibility(
            Control(Element::typed::<TerminalWidget>(props)),
            size,
            vec![
                ("LINE29", Some("LINE00"), wheel(-100.0)),
                ("LINE00", Some("LINE29"), wheel(100.0)),
                ("LINE29", Some("LINE00"), None),
            ],
        );
        assert!(frames.iter().any(|frame| frame.text.contains("LINE00")));
    }
}

#[test]
#[cfg(unix)]
fn terminal_widget_paints_cursor_on_wide_continuation() {
    let (_dir, props) = shell(
        r#"
printf '界READY\033[1;2H\033[2 q'
while IFS= read -r value; do :; done
"#,
    );
    let frames = app_input::run_when(
        Control(Element::typed::<TerminalWidget>(props)),
        (28, 8),
        vec![("READY", None)],
    );
    let frame = frames.last().unwrap();
    let wide = frame.screen.cell(1, 0).unwrap();
    assert_eq!(wide.contents(), "界");
    assert!(wide.is_wide());
    assert_eq!(wide.bgcolor(), vt100::Color::Rgb(255, 255, 255));
    assert_eq!(wide.fgcolor(), vt100::Color::Rgb(0, 0, 0));
    assert_eq!(
        frame.screen.cell(1, 2).unwrap().bgcolor(),
        vt100::Color::Rgb(0, 0, 0)
    );
}

#[test]
#[cfg(unix)]
fn terminal_widget_blinks_without_child_output_or_input() {
    let (_dir, props) = shell(
        r#"
printf 'READY\033[1;1H\033[1 q'
while IFS= read -r value; do :; done
"#,
    );
    let frames = app_input::run_when_seen(
        Control(Element::typed::<TerminalWidget>(props)),
        (28, 8),
        &["READY"],
        5,
    );
    let mut phases: Vec<_> = frames
        .iter()
        .filter(|frame| frame.text.contains("READY"))
        .map(|frame| frame.screen.cell(1, 0).unwrap().bgcolor())
        .collect();
    phases.dedup();
    assert!(
        phases.len() >= 3,
        "cursor did not blink off and on: {phases:?}"
    );
    assert_eq!(phases[0], vt100::Color::Rgb(255, 255, 255));
    assert_eq!(phases[1], vt100::Color::Rgb(0, 0, 0));
    assert_eq!(phases[2], vt100::Color::Rgb(255, 255, 255));
}

#[test]
#[cfg(unix)]
fn terminal_widget_preserves_text_under_bar_and_underline_cursors() {
    for shape in [4, 6] {
        let (_dir, mut props) = shell(&format!(
            "stty -echo\nprintf '界READY\\033[1;2H\\033[{shape} q'\nIFS= read -r value"
        ));
        props.title.clear();
        props.show_scrollbar = false;
        let frames = app_input::run_when(
            Control(Element::typed::<TerminalWidget>(props)),
            (20, 5),
            vec![("界READY", None)],
        );
        let frame = frames.last().unwrap();
        assert_eq!(frame.screen.cell(0, 0).unwrap().contents(), "界");
        assert_eq!(
            frame.screen.cell(0, 0).unwrap().bgcolor(),
            vt100::Color::Rgb(0, 0, 0)
        );
        assert!(!frame.screen.hide_cursor(), "host cursor must be visible");
        assert_eq!(frame.screen.cursor_position(), (0, 1));
        assert!(
            frame
                .output
                .windows(5)
                .any(|bytes| bytes == format!("\x1b[{shape} q").as_bytes()),
            "missing native shape {shape}"
        );
    }
}

#[test]
#[cfg(unix)]
fn terminal_widget_blinks_native_shapes_while_child_is_silent() {
    for shape in [3, 5] {
        let (_dir, mut props) = shell(&format!(
            "stty -echo\nprintf 'READY\\033[1;1H\\033[{shape} q'\nIFS= read -r value"
        ));
        props.title.clear();
        props.show_scrollbar = false;
        let frames = app_input::run_when_seen(
            Control(Element::typed::<TerminalWidget>(props)),
            (20, 5),
            &["READY"],
            5,
        );
        let mut phases: Vec<_> = frames
            .iter()
            .filter(|frame| frame.text.contains("READY"))
            .map(|frame| frame.screen.hide_cursor())
            .collect();
        phases.dedup();
        assert!(
            phases.len() >= 3,
            "native shape {shape} did not blink: {phases:?}"
        );
        for frame in frames.iter().filter(|frame| frame.text.contains("READY")) {
            assert_eq!(frame.screen.cell(0, 0).unwrap().contents(), "R");
            assert_eq!(
                frame.screen.cell(0, 0).unwrap().bgcolor(),
                vt100::Color::Rgb(0, 0, 0)
            );
        }
    }
}

#[test]
#[cfg(unix)]
fn terminal_widget_paints_child_character_line_and_scroll_edits() {
    for size in [(20, 6), (30, 8)] {
        let (_dir, mut props) = shell(
            r#"
stty -echo
printf 'abcdef\033[1;3H\033[2@XY\033[1;4H\033[2P\033[1;5H\033[2X'
printf '\033[2;1Hone\033[3;1Htwo\033[4;1Hthree\033[2;4r\033[3;1H\033[LNEW'
printf '\033[2;1H\033[M\033[S\033[T\033[r\033[5;1HREADY\033[?25l'
IFS= read -r value
"#,
        );
        props.title.clear();
        props.show_scrollbar = false;
        let frames = app_input::run_when(
            Control(Element::typed::<TerminalWidget>(props)),
            size,
            vec![("READY", None)],
        );
        let frame = frames.last().unwrap();
        for (row, text) in [
            (0, "abXd  "),
            (1, "      "),
            (2, "two   "),
            (3, "      "),
            (4, "READY "),
        ] {
            for (col, expected) in text.chars().enumerate() {
                assert_eq!(
                    frame.screen.cell(row, col as u16).unwrap().contents(),
                    expected.to_string(),
                    "size {size:?}, cell {col},{row}"
                );
            }
        }
    }
}

#[test]
#[cfg(unix)]
fn terminal_widget_recovers_invalid_initial_size_before_launching_measured_child() {
    for size in [(20, 6), (30, 8)] {
        let (_dir, mut props) = shell("stty -echo\nprintf 'READY:'; stty size\nIFS= read -r value");
        props.config.size = (0, 0);
        props.title.clear();
        props.show_scrollbar = false;
        let expected = format!("READY:{} {}", size.1, size.0);
        let frames = app_input::run_when(
            Control(Element::typed::<TerminalWidget>(props)),
            size,
            vec![(&expected, None)],
        );
        assert!(frames.last().unwrap().text.contains(&expected));
        assert!(!frames.last().unwrap().text.contains("Terminal error"));
    }
}

#[test]
#[cfg(unix)]
fn terminal_widget_scrollbar_click_and_drag_use_measured_padded_bounds() {
    use reactive_tui::{
        builder::ElementBuilder,
        component::{ElementType, LayoutType},
        event::types::{MouseButton, MouseEvent, MouseEventKind, Position},
    };
    let pointer = |kind, x, y| {
        Some(Event::Mouse(
            MouseEvent::new(kind, Position::cell(x, y)).with_button(MouseButton::Left),
        ))
    };
    for (width, height) in [(24, 10), (32, 12)] {
        let (_dir, mut props) = shell("stty -echo\ni=0; while [ $i -lt 40 ]; do printf 'ROW%02d\\r\\n' $i; i=$((i+1)); done\nprintf READY\nIFS= read -r value");
        props.title = "PTY".into();
        let mut terminal = Element::typed::<TerminalWidget>(props).with_class(format!(
            "absolute left-2 top-1 w-{} h-{}",
            width - 4,
            height - 2
        ));
        terminal.metadata.styles = Some(std::sync::Arc::new(
            reactive_tui::layout::style::StyleBuilder::new()
                .padding_all_px(1.0)
                .snapshot(),
        ));
        let root = ElementBuilder::new(ElementType::Layout(LayoutType::Absolute))
            .class("w-full h-full")
            .child(terminal)
            .build();
        let frames = app_input::run_when(
            Control(root),
            (width, height),
            vec![
                ("READY", pointer(MouseEventKind::Down, width - 4, 3)),
                ("ROW00", pointer(MouseEventKind::Drag, 3, height - 3)),
                ("READY", pointer(MouseEventKind::Up, 3, height - 3)),
                ("READY", pointer(MouseEventKind::Drag, width - 4, 3)),
                ("READY", None),
            ],
        );
        assert!(frames.iter().any(|frame| frame.text.contains("ROW00")));
        assert!(frames.last().unwrap().text.contains("READY"));
    }
}

#[test]
#[cfg(unix)]
fn terminal_widget_keeps_old_history_text_readable_after_narrowing() {
    use reactive_tui::event::types::{MouseEvent, Position, WheelDelta, WheelEvent, WheelPhase};
    for (size, narrower, suffix) in [
        ((24, 6), (12, 6), "LMNOPQRST"),
        ((36, 10), (16, 10), "PQRST"),
    ] {
        let (_directory, props) = shell("stty -echo\nprintf 'ABCDEFGHIJKLMNOPQRST\\r\\n'\ni=0; while [ $i -lt 30 ]; do printf 'ROW%02d\\r\\n' $i; i=$((i+1)); done\nprintf READY\nIFS= read -r value");
        let frames = app_input::run_when(
            Control(Element::typed::<TerminalWidget>(props)),
            size,
            vec![
                (
                    "READY",
                    Some(Event::Resize(ResizeEvent::new(narrower.0, narrower.1))),
                ),
                (
                    "READY",
                    Some(Event::Mouse(MouseEvent::wheel(
                        Position::cell(2, 2),
                        WheelEvent {
                            delta: WheelDelta::Lines { x: 0.0, y: -100.0 },
                            phase: WheelPhase::Changed,
                        },
                    ))),
                ),
                (suffix, None),
            ],
        );
        assert!(frames.last().unwrap().text.contains(suffix));
        assert_eq!(
            frames.last().unwrap().screen.size(),
            (narrower.1, narrower.0)
        );
    }
}
