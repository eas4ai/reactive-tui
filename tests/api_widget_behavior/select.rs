use super::*;
use reactive_tui::widgets::input::{SelectBuilder, SelectOption};

struct ChangedSelection(std::sync::atomic::AtomicBool);

impl RootComponent for ChangedSelection {
    fn render(&self) -> Element {
        builder::select()
            .option("alpha", "Alpha")
            .option("beta", "Beta")
            .option("gamma", "Gamma")
            .selected(if self.0.load(std::sync::atomic::Ordering::SeqCst) {
                "gamma"
            } else {
                "alpha"
            })
            .multiple(true)
            .build()
            .auto_focus()
    }

    fn handle_event(&self, event: &Event) -> reactive_tui::event::router::EventResult {
        if matches!(event, Event::Key(key) if key.code == KeyCode::F(2)) {
            self.0.store(true, std::sync::atomic::Ordering::SeqCst);
            reactive_tui::event::router::EventResult::Consumed
        } else {
            reactive_tui::event::router::EventResult::Ignored
        }
    }
}

#[test]
fn select_multiple_authored_selection_replaces_retained_choices() {
    for size in [(24, 6), (48, 12)] {
        let frames = app_input::run_visibility(
            ChangedSelection(Default::default()),
            size,
            vec![
                ("[Alpha]", None, key(KeyCode::Enter)),
                ("▼", None, key(KeyCode::Down)),
                ("▼", None, key(KeyCode::Enter)),
                ("[Alpha, Beta]", None, key(KeyCode::Escape)),
                ("[Alpha, Beta]", Some("▼"), key(KeyCode::F(2))),
                ("[Gamma]", None, None),
            ],
        );
        assert!(
            frames
                .iter()
                .any(|frame| frame.text.contains("[Alpha, Beta]")),
            "{:?}",
            frames
                .iter()
                .map(|frame| frame.text.lines().next())
                .collect::<Vec<_>>()
        );
        assert!(
            frames.last().unwrap().text.contains("[Gamma]"),
            "{}",
            frames.last().unwrap().text
        );
        assert!(!frames.last().unwrap().text.contains("Alpha"));
    }
}

#[test]
fn select_builder_search_cycles_and_multiple_choices_survive_reopening() {
    for size in [(24, 6), (48, 12)] {
        let control = builder::select()
            .option("alpha", "Alpha")
            .option("beta", "Beta")
            .option("bravo", "Bravo")
            .multiple(true)
            .build()
            .auto_focus();
        let frames = run(
            Control(control),
            size,
            vec![
                (1, key(KeyCode::Char('b'))),
                (1, key(KeyCode::Char('b'))),
                (1, key(KeyCode::Enter)),
                (1, key(KeyCode::Escape)),
                (1, key(KeyCode::Char('a'))),
                (1, key(KeyCode::Enter)),
                (1, key(KeyCode::Escape)),
                (2, None),
            ],
        );
        assert!(
            frames.last().unwrap().text.contains("[Alpha, Bravo]"),
            "{}",
            frames.last().unwrap().text
        );
        assert!(!frames.last().unwrap().text.contains('▼'));
    }
}

#[test]
fn select_search_opens_and_matches_enabled_unicode_prefixes() {
    for size in [(24, 6), (48, 12)] {
        for (letters, expected) in [(vec!['b', 'r'], "Bravo"), (vec!['É'], "Éclair")] {
            let control = SelectBuilder::new()
                .options(vec![
                    SelectOption::new(0, "Alpha"),
                    SelectOption::new(1, "Brown").disabled(true),
                    SelectOption::new(2, "Beta"),
                    SelectOption::new(3, "Bravo"),
                    SelectOption::new(4, "Éclair"),
                ])
                .selected(0)
                .max_visible_items(2)
                .render()
                .auto_focus();
            let mut events: Vec<_> = letters
                .into_iter()
                .map(|c| (1, key(KeyCode::Char(c))))
                .collect();
            events.extend([(1, key(KeyCode::Enter)), (2, None)]);
            let frames = run(Control(control), size, events);
            assert!(
                frames
                    .last()
                    .unwrap()
                    .text
                    .contains(&format!("[{expected}]")),
                "{}",
                frames.last().unwrap().text
            );
            assert!(!frames.last().unwrap().text.contains("▼"));
        }
    }
}
