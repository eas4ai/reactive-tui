use super::*;
use crate::{backend::SuprTuiBackend, builder::core::div};
use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
    time::Duration,
};
#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<u8>>>);
impl Write for Capture {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(data);
        Ok(data.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl Capture {
    fn screen(&self) -> vt100::Screen {
        let mut p = vt100::Parser::new(4, 20, 0);
        p.process(&self.0.lock().unwrap());
        p.screen().clone()
    }
}
/// The capture once every presented frame has been written (PIP-001).
fn synced<'a>(manager: &mut ScreenManager, output: &'a Capture) -> &'a Capture {
    manager.sync().unwrap();
    output
}

fn fixture() -> (ScreenManager, Capture) {
    let output = Capture::default();
    let backend = SuprTuiBackend::with_writer(20, 4, output.clone()).unwrap();
    let mut manager = ScreenManager::new(Box::new(backend));
    manager
        .create_screen(
            "from",
            "From".into(),
            div().class("w-full h-full bg-red-500").text("FROM").build(),
        )
        .unwrap();
    manager
        .create_screen(
            "to",
            "To".into(),
            div().class("w-full h-full bg-blue-500").text("TO").build(),
        )
        .unwrap();
    (manager, output)
}

#[test]
fn responsive_screen_styles_follow_resize_without_replacing_content() {
    let output = Capture::default();
    let backend = SuprTuiBackend::with_writer(79, 4, output.clone()).unwrap();
    let mut manager = ScreenManager::new(Box::new(backend));
    manager
        .create_screen(
            "responsive",
            "Responsive".into(),
            div()
                .styles(crate::responsive_css! {
                    base: { background_color: (1.0, 0.0, 0.0, 1.0) },
                    md: { background_color: (0.0, 1.0, 0.0, 1.0) },
                })
                .class("w-full h-full")
                .child(Element::text("same screen"))
                .build(),
        )
        .unwrap();
    for width in [79, 80, 79] {
        manager
            .process_event(&rt_event::Event::Resize(rt_event::ResizeEvent::new(
                width, 4,
            )))
            .unwrap();
        let mut parser = vt100::Parser::new(4, width, 0);
        parser.process(&synced(&mut manager, &output).0.lock().unwrap());
        assert_eq!(
            parser.screen().cell(0, 0).unwrap().bgcolor(),
            if width >= 80 {
                vt100::Color::Rgb(0, 255, 0)
            } else {
                vt100::Color::Rgb(255, 0, 0)
            }
        );
        assert!(parser.screen().contents().contains("same screen"));
    }
}

#[test]
fn responsive_snapshot_rejects_nonfinite_values_even_in_inactive_profiles() {
    for width in [20, 80] {
        for padding in [1.0, f32::NAN, f32::INFINITY] {
            let mut manager =
                ScreenManager::new(Box::new(crate::backend::DebugBackend::new(width, 4)));
            let result = manager.create_screen(
                "profile",
                "Profile".into(),
                div()
                    .styles(crate::responsive_css! {
                        base: { padding: 0.0 },
                        md: { padding: padding },
                    })
                    .child(Element::text("validation"))
                    .build(),
            );
            if padding.is_finite() {
                assert!(result.is_ok(), "{result:?}");
            } else {
                assert!(result.unwrap_err().contains("finite"));
            }
        }
    }
}
#[test]
fn screen_fade_midpoint_blends_presented_colors() {
    let (mut manager, output) = fixture();
    manager
        .switch_to_with_transition(
            ScreenId::from("to"),
            Some(TransitionConfig {
                transition_type: TransitionType::Fade,
                duration: Duration::from_secs(1),
                easing: super::super::EasingFunction::Linear,
                ..Default::default()
            }),
        )
        .unwrap();
    manager.transition_state.progress = 0.5;
    manager.render_transition().unwrap();
    assert_eq!(
        synced(&mut manager, &output)
            .screen()
            .cell(2, 10)
            .unwrap()
            .bgcolor(),
        vt100::Color::Rgb(149, 99, 157)
    );
}

fn start(manager: &mut ScreenManager, kind: TransitionType) {
    manager
        .switch_to_with_transition(
            ScreenId::from("to"),
            Some(TransitionConfig {
                transition_type: kind,
                duration: Duration::from_secs(60),
                easing: super::super::EasingFunction::Linear,
                ..Default::default()
            }),
        )
        .unwrap();
}
#[test]
fn screen_transitions_have_distinct_intermediate_frames_and_exact_endpoints() {
    for kind in [
        TransitionType::Fade,
        TransitionType::SlideLeft,
        TransitionType::SlideRight,
        TransitionType::SlideUp,
        TransitionType::SlideDown,
        TransitionType::Scale,
        TransitionType::Flip,
        TransitionType::Cube,
        TransitionType::Push,
    ] {
        let (mut manager, output) = fixture();
        let initial = synced(&mut manager, &output).screen().contents_formatted();
        start(&mut manager, kind);
        manager.transition_state.progress = 0.0;
        manager.render_transition().unwrap();
        assert_eq!(
            synced(&mut manager, &output).screen().contents_formatted(),
            initial,
            "{kind:?} start"
        );
        manager.transition_state.progress = 0.25;
        manager.render_transition().unwrap();
        let quarter = synced(&mut manager, &output).screen().contents_formatted();
        manager.transition_state.progress = 0.75;
        manager.render_transition().unwrap();
        let later = synced(&mut manager, &output).screen().contents_formatted();
        assert_ne!(quarter, later, "{kind:?} progress");
        manager.transition_state.progress = 1.0;
        manager.render_transition().unwrap();
        let end = synced(&mut manager, &output).screen().contents_formatted();
        manager.complete_transition().unwrap();
        assert_eq!(
            synced(&mut manager, &output).screen().contents_formatted(),
            end,
            "{kind:?} end"
        );
        assert_eq!(manager.get_active_screen(), Some(&ScreenId::from("to")));
        assert!(!manager.is_transitioning());
    }
}
#[test]
fn screen_slide_has_source_left_and_target_right() {
    let (mut manager, output) = fixture();
    start(&mut manager, TransitionType::SlideLeft);
    manager.transition_state.progress = 0.5;
    manager.render_transition().unwrap();
    let frame = synced(&mut manager, &output).screen();
    assert_eq!(
        frame.cell(2, 5).unwrap().bgcolor(),
        vt100::Color::Rgb(239, 68, 68)
    );
    assert_eq!(
        frame.cell(2, 15).unwrap().bgcolor(),
        vt100::Color::Rgb(59, 130, 246)
    );
}
#[test]
fn screen_transition_clock_handles_submillisecond_duration_and_completion() {
    let mut state = TransitionState::default();
    state.start_transition(
        None,
        ScreenId::from("to"),
        TransitionConfig {
            duration: Duration::from_micros(100),
            easing: super::super::EasingFunction::Linear,
            ..Default::default()
        },
    );
    let start = state.start_time.unwrap();
    assert!(!state.update_at(start + Duration::from_micros(50)));
    assert_eq!(state.progress, 0.5);
    assert!(state.update_at(start + Duration::from_micros(100)));
    assert_eq!(state.progress, 1.0);
    assert!(!state.update_at(start + Duration::from_secs(1)));
}
#[test]
fn screen_transition_input_uses_source_geometry_until_completion() {
    use crate::event::types::{
        Event, KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind, Position,
    };
    let output = Capture::default();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut manager = ScreenManager::new(Box::new(
        SuprTuiBackend::with_writer(20, 4, output).unwrap(),
    ));
    for (id, color) in [("from", "bg-red-500"), ("to", "bg-blue-500")] {
        let calls = calls.clone();
        manager
            .create_screen(
                id,
                id.into(),
                div()
                    .class(&format!("w-full h-full {color}"))
                    .text(id)
                    .on_click(move || calls.lock().unwrap().push(id))
                    .build()
                    .auto_focus(),
            )
            .unwrap();
    }
    start(&mut manager, TransitionType::SlideLeft);
    manager.transition_state.progress = 0.5;
    manager.render_transition().unwrap();
    // Call the active runtime directly to keep the controlled transition clock fixed.
    let runtime = &mut manager
        .screens
        .get_mut(&ScreenId::from("from"))
        .unwrap()
        .runtime;
    runtime.process_event(&Event::Mouse(
        MouseEvent::new(MouseEventKind::Down, Position::cell(15, 2)).with_button(MouseButton::Left),
    ));
    assert!(calls.lock().unwrap().is_empty());
    runtime.process_event(&Event::Mouse(
        MouseEvent::new(MouseEventKind::Down, Position::cell(5, 2)).with_button(MouseButton::Left),
    ));
    runtime.process_event(&Event::Key(KeyEvent::new(KeyCode::Enter)));
    assert_eq!(*calls.lock().unwrap(), ["from", "from"]);
    manager.complete_transition().unwrap();
    manager
        .process_event(&Event::Key(KeyEvent::new(KeyCode::Enter)))
        .unwrap();
    assert_eq!(*calls.lock().unwrap(), ["from", "from", "to"]);
}
#[test]
fn screen_removal_and_immediate_switch_cancel_pending_transition() {
    let (mut manager, output) = fixture();
    start(&mut manager, TransitionType::Fade);
    manager.remove_screen(&ScreenId::from("to")).unwrap();
    assert!(!manager.is_transitioning());
    assert!(synced(&mut manager, &output)
        .screen()
        .contents()
        .contains("FROM"));
    manager.remove_screen(&ScreenId::from("from")).unwrap();
    assert_eq!(manager.get_active_screen(), None);
    assert!(
        synced(&mut manager, &output)
            .screen()
            .contents()
            .trim()
            .is_empty(),
        "{:?}",
        synced(&mut manager, &output).screen().contents()
    );
    let (mut manager, _) = fixture();
    start(&mut manager, TransitionType::Fade);
    manager.switch_to_immediate(ScreenId::from("from")).unwrap();
    manager.update().unwrap();
    assert!(!manager.is_transitioning());
    assert_eq!(manager.get_active_screen(), Some(&ScreenId::from("from")));
}

#[test]
fn screen_hotkeys_ignore_release_and_repeat() {
    use crate::event::types::{Event, KeyCode, KeyEvent, KeyEventKind};
    let (mut manager, _) = fixture();
    manager.set_default_transition(TransitionConfig {
        transition_type: TransitionType::None,
        ..Default::default()
    });
    manager.set_hotkey(KeyCode::Char('t'), ScreenId::from("to"));
    for kind in [KeyEventKind::Release, KeyEventKind::Repeat] {
        manager
            .process_event(&Event::Key(
                KeyEvent::new(KeyCode::Char('t')).with_kind(kind),
            ))
            .unwrap();
        assert_eq!(manager.get_active_screen(), Some(&ScreenId::from("from")));
    }
    manager
        .process_event(&Event::Key(KeyEvent::new(KeyCode::Char('t'))))
        .unwrap();
    assert_eq!(manager.get_active_screen(), Some(&ScreenId::from("to")));
}
#[test]
fn screen_fade_preserves_wide_and_combining_text_at_completion() {
    let (mut manager, output) = fixture();
    manager
        .update_screen(
            &ScreenId::from("to"),
            div()
                .class("w-full h-full bg-blue-500")
                .text("界🙂e\u{301}")
                .build(),
        )
        .unwrap();
    start(&mut manager, TransitionType::Fade);
    manager.transition_state.progress = 0.5;
    manager.render_transition().unwrap();
    assert!(synced(&mut manager, &output)
        .screen()
        .contents()
        .contains("界🙂e\u{301}"));
    manager.complete_transition().unwrap();
    assert!(synced(&mut manager, &output)
        .screen()
        .contents()
        .contains("界🙂e\u{301}"));
}
