use super::{app_input, key, run, Control};
use reactive_tui::{
    app::RootComponent,
    builder,
    component::Element,
    event::{
        router::EventResult,
        types::{Event, KeyCode, ResizeEvent},
    },
    widgets::display::{ProgressBar, ProgressBarBuilder, ProgressBarOrientation, ProgressBarProps},
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
fn last(element: Element, size: (u16, u16)) -> app_input::Snapshot {
    run(Control(element), size, vec![(2, None)]).pop().unwrap()
}
fn count(frame: &app_input::Snapshot, mark: char) -> usize {
    frame.text.chars().filter(|c| *c == mark).count()
}

#[test]
fn progress_builders_and_macros_paint_actual_bars() {
    for size in [(24, 8), (48, 12)] {
        for element in [
            builder::progress_bar()
                .value(25.0)
                .max_value(50.0)
                .label("Download")
                .width(20)
                .color("#00ff00")
                .build(),
            reactive_tui::progress_bar![50.0,label:"Download"],
            ProgressBarBuilder::new()
                .value(50.0)
                .label("Download")
                .width(20)
                .render(),
        ] {
            let frame = last(element, size);
            assert!(
                frame.text.contains("Download") && frame.text.contains("50.0%"),
                "{}",
                frame.text
            );
            assert!(count(&frame, '█') > 0 && count(&frame, '▒') > 0);
            assert!(!frame.text.contains("Progress:"));
        }
        for element in [
            ProgressBar::new(),
            ProgressBar::with_props(ProgressBarProps::default()),
            Element::typed::<ProgressBar>(ProgressBarProps::default()),
        ] {
            let frame = last(element, size);
            assert!(frame.text.contains("0.0%"));
            assert!(count(&frame, '▒') > 0);
        }
    }
}

#[test]
fn progress_horizontal_and_vertical_bars_use_range_and_real_dimensions() {
    for size in [(24, 10), (48, 16)] {
        let frame = last(
            ProgressBarBuilder::new()
                .range(-10.0, 10.0)
                .value(0.0)
                .width(20)
                .height(2)
                .show_percentage(false)
                .show_value(false)
                .color("#ff0000")
                .background_color("#00ff00")
                .render(),
            size,
        );
        assert_eq!(count(&frame, '█'), 20);
        assert_eq!(count(&frame, '▒'), 20);
        assert_eq!(
            frame.screen.cell(0, 9).unwrap().fgcolor(),
            vt100::Color::Rgb(255, 0, 0)
        );
        assert_eq!(
            frame.screen.cell(1, 10).unwrap().fgcolor(),
            vt100::Color::Rgb(0, 255, 0)
        );
        let frame = last(
            ProgressBarBuilder::new()
                .value(50.0)
                .width(3)
                .height(8)
                .vertical()
                .show_percentage(false)
                .render(),
            size,
        );
        assert_eq!(count(&frame, '█'), 12);
        assert_eq!(count(&frame, '▒'), 12);
        assert_eq!(frame.screen.cell(0, 1).unwrap().contents(), "▒");
        assert_eq!(frame.screen.cell(7, 1).unwrap().contents(), "█");
        for (value, filled) in [(-100.0, 0), (200.0, 20)] {
            let frame = last(
                ProgressBarBuilder::new()
                    .value(value)
                    .width(20)
                    .show_percentage(false)
                    .render(),
                size,
            );
            assert_eq!(count(&frame, '█'), filled);
        }
    }
}

#[test]
fn progress_segments_stripes_and_styles_affect_painted_cells() {
    for size in [(24, 10), (48, 16)] {
        let frame = last(
            ProgressBarBuilder::new()
                .value(50.0)
                .width(20)
                .segments(4)
                .show_percentage(false)
                .render(),
            size,
        );
        assert_eq!(count(&frame, '█'), 8, "{}", frame.text);
        assert_eq!(frame.screen.cell(0, 4).unwrap().contents(), " ");
        let frame = last(
            ProgressBarBuilder::new()
                .value(75.0)
                .width(20)
                .striped(true)
                .label("load")
                .show_value(true)
                .style("bg-black")
                .bar_style("text-red-500")
                .text_style("uppercase text-green-500")
                .render(),
            size,
        );
        assert!(frame.text.contains("LOAD"));
        assert!(count(&frame, '▌') > 0 && count(&frame, '░') > 0);
        assert_eq!(
            frame.screen.cell(1, 0).unwrap().fgcolor(),
            vt100::Color::Rgb(239, 68, 68)
        );
        assert_eq!(
            frame.screen.cell(0, 0).unwrap().fgcolor(),
            vt100::Color::Rgb(34, 197, 94)
        );
        let frame = last(
            ProgressBarBuilder::new()
                .value(50.0)
                .width(8)
                .segments(u16::MAX)
                .show_percentage(false)
                .render(),
            size,
        );
        assert_eq!(count(&frame, '█'), 4);
    }
}

#[test]
fn progress_resize_and_own_padding_keep_content_inside_measured_bounds() {
    let frames = app_input::run_when(
        Control(
            ProgressBarBuilder::new()
                .value(50.0)
                .height(2)
                .style("p-0.5")
                .label("Padded")
                .render(),
        ),
        (32, 12),
        vec![
            ("50.0%", Some(Event::Resize(ResizeEvent::new(24, 10)))),
            ("██████████▒▒▒▒▒▒▒▒▒▒", None),
        ],
    );
    let first = frames.iter().find(|f| f.text.contains("50.0%")).unwrap();
    assert_eq!(count(first, '█'), 28, "{}", first.text);
    let frame = frames.last().unwrap();
    assert_eq!(count(frame, '█'), 20, "{}", frame.text);
    assert!(frame
        .screen
        .cell(0, 0)
        .unwrap()
        .contents()
        .trim()
        .is_empty());
    assert_eq!(frame.screen.cell(3, 2).unwrap().contents(), "█");
}

struct Updates {
    props: Mutex<ProgressBarProps>,
    second: Arc<AtomicUsize>,
}
impl RootComponent for Updates {
    fn render(&self) -> Element {
        Element::typed::<ProgressBar>(self.props.lock().unwrap().clone())
    }
    fn handle_event(&self, event: &Event) -> EventResult {
        let Event::Key(key) = event else {
            return EventResult::Ignored;
        };
        let mut props = self.props.lock().unwrap();
        match key.code {
            KeyCode::Char('c') => props.value = 100.0,
            KeyCode::Char('r') => props.value = 25.0,
            KeyCode::Char('s') => {
                let count = self.second.clone();
                props.on_complete = Some(Arc::new(move || {
                    count.fetch_add(1, Ordering::SeqCst);
                }));
            }
            KeyCode::Char('f') => {
                props.custom_formatter = Some(Arc::new(|value, _, _| format!("New {value}")))
            }
            _ => return EventResult::Ignored,
        }
        EventResult::Consumed
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

#[test]
fn progress_completion_fires_once_per_transition_and_replaced_callbacks_are_used() {
    for size in [(24, 8), (48, 12)] {
        let first = Arc::new(AtomicUsize::new(0));
        let second = Arc::new(AtomicUsize::new(0));
        let count = first.clone();
        let props = ProgressBarBuilder::new()
            .value(25.0)
            .on_complete(Arc::new(move || {
                count.fetch_add(1, Ordering::SeqCst);
            }))
            .build();
        run(
            Updates {
                props: Mutex::new(props),
                second: second.clone(),
            },
            size,
            vec![
                (2, key(KeyCode::Char('c'))),
                (3, key(KeyCode::Char('c'))),
                (4, key(KeyCode::Char('r'))),
                (5, key(KeyCode::Char('s'))),
                (6, key(KeyCode::Char('c'))),
                (7, None),
            ],
        );
        assert_eq!(first.load(Ordering::SeqCst), 1);
        assert_eq!(second.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn progress_custom_formatter_receives_values_and_replacement_updates_without_value_change() {
    for size in [(24, 8), (48, 12)] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let logged = calls.clone();
        let props = ProgressBarBuilder::new()
            .range(10.0, 50.0)
            .value(25.0)
            .custom_formatter(Arc::new(move |value, min, max| {
                logged.lock().unwrap().push((value, min, max));
                "Old format".into()
            }))
            .build();
        let frames = app_input::run_when(
            Updates {
                props: Mutex::new(props),
                second: Arc::default(),
            },
            size,
            vec![("Old format", key(KeyCode::Char('f'))), ("New 25", None)],
        );
        assert!(frames.last().unwrap().text.contains("New 25"));
        assert!(!calls.lock().unwrap().is_empty());
        assert!(calls
            .lock()
            .unwrap()
            .iter()
            .all(|v| *v == (25.0, 10.0, 50.0)));
    }
}

#[test]
fn invalid_progress_and_indeterminate_values_never_report_completion() {
    for size in [(24, 8), (48, 12)] {
        for mode in 0..7 {
            let count = Arc::new(AtomicUsize::new(0));
            let callback = count.clone();
            let mut props = ProgressBarBuilder::new()
                .value(100.0)
                .on_complete(Arc::new(move || {
                    callback.fetch_add(1, Ordering::SeqCst);
                }))
                .build();
            match mode {
                0 => props.value = f64::NAN,
                1 => props.value = f64::INFINITY,
                2 => props.min_value = 100.0,
                3 => props.height = 0,
                4 => props.segments = Some(0),
                5 => props.color = Some("#界".into()),
                _ => {
                    props.indeterminate = true;
                    props.style = Some("reduced-motion".into());
                }
            }
            let frame = last(Element::typed::<ProgressBar>(props), size);
            assert_eq!(count.load(Ordering::SeqCst), 0);
            assert!(
                frame
                    .text
                    .contains(if mode == 6 { "Loading..." } else { "Invalid" }),
                "{}",
                frame.text
            );
        }
    }
}

#[test]
fn progress_indeterminate_pulse_and_smooth_changes_advance_without_input() {
    for size in [(24, 8), (48, 12)] {
        let props = ProgressBarBuilder::new()
            .width(20)
            .indeterminate(true)
            .striped(true)
            .build();
        let frames = run(
            Control(Element::typed::<ProgressBar>(props)),
            size,
            vec![(10, None)],
        );
        assert!(frames.iter().skip(1).any(|f| f.text != frames[1].text));
        assert!(frames.iter().all(|f| !f.text.contains("100.0%")));
        let props = ProgressBarBuilder::new()
            .width(20)
            .value(50.0)
            .pulse(true)
            .color("#ff0000")
            .build();
        let frames = run(
            Control(Element::typed::<ProgressBar>(props)),
            size,
            vec![(8, None)],
        );
        let colors: Vec<_> = frames
            .iter()
            .skip(1)
            .map(|f| f.screen.cell(0, 0).unwrap().fgcolor())
            .collect();
        assert!(colors.iter().any(|c| *c != colors[0]), "{colors:?}");
        let props = ProgressBarBuilder::new()
            .width(20)
            .value(25.0)
            .animated(true)
            .build();
        let frames = run(
            Updates {
                props: Mutex::new(props),
                second: Arc::default(),
            },
            size,
            vec![(2, key(KeyCode::Char('c'))), (10, None)],
        );
        assert!(
            frames
                .iter()
                .any(|f| count(f, '█') > 5 && count(f, '█') < 20),
            "{:?}",
            frames.iter().map(|f| count(f, '█')).collect::<Vec<_>>()
        );
    }
}

#[test]
fn progress_public_helpers_reduced_motion_and_disabled_input_work_through_app() {
    for size in [(24, 10), (48, 16)] {
        let props = ProgressBar::with_value(ProgressBarProps::default(), 75.0);
        let props = ProgressBar::with_range(props, 0.0, 150.0);
        let props = ProgressBar::with_label(props, "Task");
        let props = ProgressBar::with_color(props, "rgb(255,0,0)");
        let props = ProgressBar::animated(props, true);
        let props = ProgressBar::with_orientation(props, ProgressBarOrientation::Horizontal);
        let frame = last(ProgressBar::with_props(props).disabled(true), size);
        assert!(frame.text.contains("Task") && frame.text.contains("50.0%"));
        assert_eq!(
            frame.screen.cell(1, 0).unwrap().fgcolor(),
            vt100::Color::Rgb(255, 0, 0)
        );
        let element = ProgressBarBuilder::new()
            .width(20)
            .value(50.0)
            .render()
            .disabled(true);
        let frames = run(
            Control(element),
            size,
            vec![(2, key(KeyCode::End)), (2, super::click(3, 0)), (2, None)],
        );
        assert!(frames.last().unwrap().text.contains("50.0%"));
        let props = ProgressBarBuilder::new()
            .width(4)
            .height(8)
            .orientation(ProgressBarOrientation::Vertical)
            .indeterminate(true)
            .pulse(true)
            .style("reduced-motion")
            .show_percentage(false)
            .build();
        let frame = last(
            ProgressBar::with_props(ProgressBar::indeterminate(props, true)),
            size,
        );
        assert!(count(&frame, '█') > 0 && count(&frame, '▒') > 0);
    }
}

#[test]
fn progress_initial_completion_is_once_and_invalid_width_is_visible() {
    for size in [(24, 8), (48, 12)] {
        let count = Arc::new(AtomicUsize::new(0));
        let callback = count.clone();
        let element = ProgressBarBuilder::new()
            .value(100.0)
            .on_complete(Arc::new(move || {
                callback.fetch_add(1, Ordering::SeqCst);
            }))
            .render();
        let frame = last(element, size);
        assert!(frame.text.contains("100.0%"));
        assert_eq!(count.load(Ordering::SeqCst), 1);
        let frame = last(ProgressBarBuilder::new().width(0).render(), size);
        assert!(frame.text.contains("Invalid bar dimensions"));
    }
}
