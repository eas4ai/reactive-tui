use super::{app_input, Control};
use reactive_tui::event::types::KeyCode;
use std::sync::{Arc, Mutex};

#[test]
fn button_macros_preserve_callbacks_and_children() {
    for size in [(40, 16), (80, 24)] {
        for styled in [false, true] {
            let calls = Arc::new(Mutex::new(0));
            let called = calls.clone();
            let handler = move || *called.lock().unwrap() += 1;
            let button = if styled {
                reactive_tui::button![class: "w-12 h-1 p-0", onclick: handler, "GO"]
            } else {
                reactive_tui::button![onclick: handler, "GO"]
            };
            app_input::run_when(
                Control(button.auto_focus()),
                size,
                vec![("GO", app_input::key(KeyCode::Enter)), ("GO", None)],
            );
            assert_eq!(*calls.lock().unwrap(), 1);
        }
    }
}

#[test]
fn input_macros_build_editable_controls() {
    for size in [(40, 16), (80, 24)] {
        for element in [
            reactive_tui::input![],
            reactive_tui::input![placeholder: "TYPE"],
            reactive_tui::input![class: "w-16"],
            reactive_tui::input![class: "w-16", placeholder: "TYPE"],
            reactive_tui::text_input![],
            reactive_tui::text_input![placeholder: "TYPE"],
            reactive_tui::text_input![value: "seed"],
            reactive_tui::text_input![value: "seed", placeholder: "TYPE"],
        ] {
            let frames = app_input::run_when(
                Control(element.auto_focus()),
                size,
                vec![
                    ("", app_input::key(KeyCode::End)),
                    ("", app_input::key(KeyCode::Char('X'))),
                    ("X", None),
                ],
            );
            let last = &frames.last().unwrap().text;
            assert!(
                last.contains('X'),
                "input macro control accepts typed text at {size:?}:\n{last}"
            );
        }
    }
}

#[test]
fn tabs_macro_retains_editable_child_controls() {
    for size in [(40, 16), (80, 24)] {
        let tabs = reactive_tui::tabs!["FIRST" => reactive_tui::text_input![value: "seed"], "SECOND" => reactive_tui::span!["OTHER"]];
        let frames = app_input::run_actions_until_hidden(
            Control(tabs),
            size,
            vec![
                ("seed", app_input::Action::ClickText("SECOND", 1)),
                ("OTHER", app_input::Action::ClickText("FIRST", 1)),
                ("seed", app_input::Action::ClickText("seed", 1)),
                (
                    "seed",
                    app_input::Action::Event(reactive_tui::event::types::Event::Key(
                        reactive_tui::event::types::KeyEvent::new(KeyCode::End),
                    )),
                ),
                (
                    "seed",
                    app_input::Action::Event(reactive_tui::event::types::Event::Key(
                        reactive_tui::event::types::KeyEvent::new(KeyCode::Char('X')),
                    )),
                ),
                (
                    "seedX",
                    app_input::Action::Event(reactive_tui::event::types::Event::Key(
                        reactive_tui::event::types::KeyEvent::new(KeyCode::F(8)),
                    )),
                ),
            ],
            "NEVER",
        );
        assert!(
            frames.last().unwrap().text.contains("seedX"),
            "{:?}",
            frames.iter().map(|frame| &frame.text).collect::<Vec<_>>()
        );
    }
}

#[test]
fn checkbox_and_select_macros_change_selection() {
    for size in [(40, 16), (80, 24)] {
        for element in [
            reactive_tui::checkbox![],
            reactive_tui::checkbox![label: "CHECK"],
            reactive_tui::checkbox![checked: false, label: "CHECK"],
        ] {
            let frames = app_input::run(
                Control(element.auto_focus()),
                size,
                vec![(1, app_input::key(KeyCode::Space)), (2, None)],
            );
            assert_ne!(frames[0].text, frames.last().unwrap().text);
            assert!(
                frames.last().unwrap().text.contains("[✓]"),
                "{}",
                frames.last().unwrap().text
            );
        }
        for element in [
            reactive_tui::select![options: ["a" => "Alpha", "b" => "Beta"]],
            reactive_tui::select![options: ["a" => "Alpha", "b" => "Beta"], selected: "a"],
        ] {
            let frames = app_input::run(
                Control(element.auto_focus()),
                size,
                vec![
                    (1, app_input::key(KeyCode::Enter)),
                    (1, app_input::key(KeyCode::Down)),
                    (1, app_input::key(KeyCode::Enter)),
                    (2, None),
                ],
            );
            assert!(frames.last().unwrap().text.contains("Beta"));
        }
    }
}

#[test]
fn display_macros_paint_actual_values() {
    for size in [(40, 16), (80, 24)] {
        for (element, expected) in [
            (reactive_tui::progress_bar![50.0], "50.0%"),
            (reactive_tui::progress_bar![50.0, label: "WORK"], "WORK"),
            (reactive_tui::progress_bar![50.0, max: 200.0], "25.0%"),
            (reactive_tui::toast![success: "SUCCESS"], "SUCCESS"),
            (reactive_tui::toast![error: "ERROR"], "ERROR"),
            (reactive_tui::toast![warning: "WARNING"], "WARNING"),
            (reactive_tui::toast![info: "INFO"], "INFO"),
            (
                reactive_tui::data_table![columns: ["Name" => "name"], rows: [vec![("name", "Ada")]]],
                "Ada",
            ),
            (
                reactive_tui::data_table![columns: ["Name" => "name"], rows: [vec![("name", "Ada")], vec![("name", "Bea")]], pagination: 1],
                "Ada",
            ),
        ] {
            let frames = app_input::run_when(Control(element), size, vec![(expected, None)]);
            let last = &frames.last().unwrap().text;
            assert!(
                last.contains(expected),
                "display macro paints {expected:?} at {size:?}:\n{last}"
            );
        }
    }
}

#[test]
fn chart_macros_paint_data_geometry() {
    for size in [(40, 16), (80, 24)] {
        for (element, mark) in [
            (reactive_tui::chart![bar: "DATA" => [2.0, 8.0]], '█'),
            (reactive_tui::chart![line: "DATA" => [2.0, 8.0]], '●'),
            (reactive_tui::chart![pie: "DATA" => [2.0, 8.0]], '█'),
            (
                reactive_tui::chart![bar: "DATA" => [2.0, 8.0], title: "TITLE"],
                '█',
            ),
            (
                reactive_tui::chart![line: "DATA" => [2.0, 8.0], title: "TITLE"],
                '●',
            ),
            (
                reactive_tui::chart![pie: "DATA" => [2.0, 8.0], title: "TITLE"],
                '█',
            ),
        ] {
            let frames = app_input::run(Control(element), size, vec![(2, None)]);
            assert!(
                frames.last().unwrap().text.contains(mark),
                "{}",
                frames.last().unwrap().text
            );
        }
    }
}
