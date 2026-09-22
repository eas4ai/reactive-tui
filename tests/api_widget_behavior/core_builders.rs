use super::{app_input, Control};
use reactive_tui::{builder, event::types::KeyCode};

#[test]
fn responsive_grid_changes_columns_after_resizing_across_its_breakpoint() {
    use reactive_tui::event::types::{Event, ResizeEvent};
    for (size, resized) in [((40, 16), (80, 24)), ((80, 24), (40, 16))] {
        let grid = builder::responsive_grid(2)
            .class("gap-0 w-full")
            .child(builder::text("A"))
            .child(builder::text("B"))
            .build();
        let frames = app_input::run(
            Control(grid),
            size,
            vec![
                (
                    1,
                    Some(Event::Resize(ResizeEvent::new(resized.0, resized.1))),
                ),
                (2, None),
            ],
        );
        for (frame, width) in [(&frames[0], size.0), (frames.last().unwrap(), resized.0)] {
            let (x, y) = if width >= 80 { (width / 2, 0) } else { (0, 1) };
            assert_eq!(
                frame.screen.cell(y, x).unwrap().contents(),
                "B",
                "width {width}: {:?}",
                frame.text
            );
        }
    }
}

#[test]
fn responsive_prefixes_use_each_apps_width_and_compose_with_focus() {
    for (width, expected) in [
        (39, vt100::Color::Rgb(239, 68, 68)),
        (40, vt100::Color::Rgb(59, 130, 246)),
        (79, vt100::Color::Rgb(59, 130, 246)),
        (80, vt100::Color::Rgb(34, 197, 94)),
        (119, vt100::Color::Rgb(34, 197, 94)),
        (120, vt100::Color::Rgb(234, 179, 8)),
        (159, vt100::Color::Rgb(234, 179, 8)),
        (160, vt100::Color::Rgb(255, 255, 255)),
        (39, vt100::Color::Rgb(239, 68, 68)),
    ] {
        let element=builder::button().text("X").class("p-0 h-1 w-1 text-red-500 sm:text-blue-500 md:text-green-500 lg:text-yellow-500 xl:text-white no-underline md:focus:underline").on_click(|| {}).build();
        let frames = app_input::run(
            Control(element),
            (width, 8),
            vec![
                (1, app_input::key(KeyCode::Tab)),
                (if width >= 80 { 2 } else { 1 }, None),
            ],
        );
        assert_eq!(
            frames[0].screen.cell(0, 0).unwrap().fgcolor(),
            expected,
            "width {width}"
        );
        assert!(
            !frames[0].screen.cell(0, 0).unwrap().underline(),
            "initial width {width}"
        );
        assert_eq!(
            frames
                .last()
                .unwrap()
                .screen
                .cell(0, 0)
                .unwrap()
                .underline(),
            width >= 80
        );
    }
}

#[test]
fn styled_input_helpers_are_visible_and_editable_at_terminal_sizes() {
    for size in [(40, 16), (80, 24)] {
        for element in [
            builder::styled_input("TYPE HERE").build(),
            builder::search_input("TYPE HERE"),
        ] {
            let frames = app_input::run_when(
                Control(element),
                size,
                vec![
                    ("TYPE HERE", app_input::key(KeyCode::Tab)),
                    ("TYPE HERE", app_input::key(KeyCode::Char('Z'))),
                    ("Z", None),
                ],
            );
            let last = &frames.last().unwrap().text;
            assert!(
                last.contains('Z') && !last.contains("TYPE HERE"),
                "typed text replaces the placeholder at {size:?}:\n{last}"
            );
        }
    }
}

#[test]
fn primary_button_default_style_preserves_visible_text_and_activation() {
    use std::sync::{Arc, Mutex};
    for size in [(40, 16), (80, 24)] {
        let calls = Arc::new(Mutex::new(0));
        let called = calls.clone();
        let element = builder::primary_button("PRIMARY", move || *called.lock().unwrap() += 1);
        app_input::run_when(
            Control(element),
            size,
            vec![
                ("PRIMARY", app_input::key(KeyCode::Tab)),
                ("PRIMARY", app_input::key(KeyCode::Enter)),
                ("PRIMARY", None),
            ],
        );
        assert_eq!(*calls.lock().unwrap(), 1);
    }
}

#[test]
fn headings_labels_and_text_helpers_paint_inherited_typography() {
    for size in [(40, 16), (80, 24)] {
        for heading in [
            builder::h1(),
            builder::h2(),
            builder::h3(),
            builder::h4(),
            builder::h5(),
            builder::h6(),
        ] {
            let frames = app_input::run(
                Control(heading.text("HEADING").build()),
                size,
                vec![(1, None)],
            );
            assert!(frames[0].text.contains("HEADING"));
            assert!(frames[0].screen.cell(0, 0).unwrap().bold());
        }
        for text in [builder::text("TEXT"), builder::label("TEXT")] {
            let frames = app_input::run(Control(text), size, vec![(1, None)]);
            assert!(frames[0].text.contains("TEXT"));
        }
    }
}

#[test]
fn row_column_and_grid_helpers_place_real_children() {
    for size in [(40, 16), (80, 24)] {
        let children = || vec![builder::text("A"), builder::text("B")];
        let cases = [
            (builder::flex_row("0.5", children()), (3, 0)),
            (builder::flex_col("0.5", children()), (0, 3)),
            (builder::grid_layout(2, "0", children()), (1, 0)),
        ];
        for (element, (x, y)) in cases {
            let frames = app_input::run(Control(element), size, vec![(1, None)]);
            assert_eq!(frames[0].screen.cell(0, 0).unwrap().contents(), "A");
            assert_eq!(
                frames[0].screen.cell(y, x).unwrap().contents(),
                "B",
                "expected B at {x},{y} in {:?}",
                frames[0].text
            );
        }
    }
}

#[test]
fn convenience_containers_show_their_children_with_default_styles() {
    for size in [(40, 16), (80, 24)] {
        for (name, container) in [
            ("div", builder::div()),
            ("span", builder::span()),
            ("paragraph", builder::p()),
            ("section", builder::section()),
            ("article", builder::article()),
            ("header", builder::header()),
            ("footer", builder::footer()),
            ("nav", builder::nav()),
            ("main", builder::main()),
            ("aside", builder::aside()),
            ("grid", builder::grid_builder()),
            ("flex", builder::flex()),
            ("screen", builder::screen()),
            ("container", builder::container()),
            ("card", builder::card_builder()),
            ("sidebar", builder::sidebar()),
            ("content", builder::content()),
            ("responsive grid", builder::responsive_grid(2)),
        ] {
            let frames = app_input::run(
                Control(container.child(builder::text("VISIBLE")).build()),
                size,
                vec![(1, None)],
            );
            assert!(
                frames[0].text.contains("VISIBLE"),
                "{name} at {size:?}: {:?}",
                frames[0].text
            );
        }
        let frames = app_input::run(
            Control(builder::card(vec![builder::text("CARD")])),
            size,
            vec![(1, None)],
        );
        assert!(
            frames[0].text.contains("CARD"),
            "card at {size:?}: {:?}",
            frames[0].text
        );
    }
}

#[test]
fn mixed_vdom_children_preserve_click_handlers_through_app() {
    use app_input::Action;
    use reactive_tui::vdom::VNode;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    for size in [(40, 16), (80, 24)] {
        let calls = Arc::new(AtomicUsize::new(0));
        let called = calls.clone();
        let child = VNode::element("flex")
            .class("w-12 h-1")
            .on("click", move |_| {
                called.fetch_add(1, Ordering::SeqCst);
            })
            .child(VNode::text("ACTIVATE"))
            .build();
        let root = builder::mixed_container()
            .class("w-full h-full")
            .child_vdom(child)
            .build();
        app_input::run_actions_until_hidden(
            Control(root),
            size,
            vec![
                ("ACTIVATE", Action::ClickText("ACTIVATE", 1)),
                (
                    "ACTIVATE",
                    Action::Event(app_input::key(KeyCode::F(8)).unwrap()),
                ),
            ],
            "NEVER",
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn core_input_placeholder_is_visible_and_typing_replaces_it() {
    for size in [(32, 12), (60, 20)] {
        let input = builder::core::input()
            .placeholder("TYPE HERE")
            .class("w-20 h-3 p-0")
            .build()
            .auto_focus();
        let frames = app_input::run_when(
            Control(input),
            size,
            vec![
                ("TYPE HERE", app_input::key(KeyCode::Char('界'))),
                ("界", None),
            ],
        );
        assert!(!frames.last().unwrap().text.contains("TYPE HERE"));
    }
}

#[test]
fn core_input_text_is_an_editable_seed() {
    for size in [(32, 12), (60, 20)] {
        let input = builder::core::input()
            .text("seed")
            .class("w-20 h-3 p-0")
            .build()
            .auto_focus();
        let frames = app_input::run_when(
            Control(input),
            size,
            vec![
                ("seed", app_input::key(KeyCode::End)),
                ("seed", app_input::key(KeyCode::Char('X'))),
                ("seedX", None),
            ],
        );
        let last = &frames.last().unwrap().text;
        assert!(
            last.contains("seedX"),
            "seed text accepts an appended character at {size:?}:\n{last}"
        );
    }
}

#[test]
fn core_input_macro_routes_edit_real_controls() {
    for size in [(32, 12), (60, 20)] {
        for (input, initial) in [
            (reactive_tui::input![], "["),
            (reactive_tui::input![placeholder: "PROMPT"], "PROMPT"),
            (reactive_tui::input![class: "w-20 h-3 p-0"], "["),
            (
                reactive_tui::input![class: "w-20 h-3 p-0", placeholder: "PROMPT"],
                "PROMPT",
            ),
        ] {
            let frames = app_input::run_when(
                Control(input.auto_focus()),
                size,
                vec![(initial, app_input::key(KeyCode::Char('Z'))), ("Z", None)],
            );
            assert!(!frames.last().unwrap().text.contains("PROMPT"));
        }
    }
}

#[test]
fn core_input_routes_resized_clicks_and_honors_disabled() {
    use app_input::Action;
    use reactive_tui::event::types::{Event, ResizeEvent};
    use std::sync::{Arc, Mutex};
    for (size, resized) in [((32, 12), (60, 20)), ((60, 20), (32, 12))] {
        for disabled in [false, true] {
            let clicks = Arc::new(Mutex::new(0));
            let clicked = clicks.clone();
            let input = builder::core::input()
                .text("seed")
                .placeholder("PROMPT")
                .class("w-12 h-3 p-0")
                .disabled(disabled)
                .on_click(move || {
                    *clicked.lock().unwrap() += 1;
                })
                .build();
            let root = builder::div()
                .class("w-1/2 h-full justify-end")
                .child(input)
                .build();
            let expected = if disabled { "seed" } else { "seedX" };
            let frames = app_input::run_actions_until_hidden(
                Control(root),
                size,
                vec![
                    (
                        "seed",
                        Action::Event(Event::Resize(ResizeEvent::new(resized.0, resized.1))),
                    ),
                    ("seed", Action::ClickText("seed", 1)),
                    ("seed", Action::Event(app_input::key(KeyCode::End).unwrap())),
                    (
                        "seed",
                        Action::Event(app_input::key(KeyCode::Char('X')).unwrap()),
                    ),
                    (
                        expected,
                        Action::Event(app_input::key(KeyCode::F(8)).unwrap()),
                    ),
                ],
                "PROMPT",
            );
            assert_eq!(*clicks.lock().unwrap(), usize::from(!disabled));
            assert!(frames.last().unwrap().text.contains(expected));
            if disabled {
                assert!(!frames.last().unwrap().text.contains("seedX"));
            }
        }
    }
}
