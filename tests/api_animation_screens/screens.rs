use reactive_tui::{
    backend::SuprTuiBackend,
    builder::core::div,
    component::Element,
    event::types::{
        Event, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind, Position,
        ResizeEvent,
    },
    screen::{ScreenId, ScreenManager},
};
use std::sync::{Arc, Mutex};

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code))
}
fn click(x: u16, y: u16) -> Event {
    Event::Mouse(
        MouseEvent::new(MouseEventKind::Down, Position::cell(x, y)).with_button(MouseButton::Left),
    )
}
fn manager() -> ScreenManager {
    ScreenManager::new(Box::new(
        SuprTuiBackend::with_writer(20, 4, std::io::sink()).unwrap(),
    ))
}
fn button(name: &'static str, calls: &Arc<Mutex<Vec<&'static str>>>) -> Element {
    let calls = calls.clone();
    div()
        .class("flex-1 h-1")
        .text(name)
        .on_click(move || calls.lock().unwrap().push(name))
        .build()
        .key(name)
}
fn pair(reverse: bool, calls: &Arc<Mutex<Vec<&'static str>>>) -> Element {
    let a = button("A", calls).auto_focus();
    let b = button("B", calls);
    div()
        .class("flex flex-row w-full h-full")
        .children(if reverse { vec![b, a] } else { vec![a, b] })
        .build()
}

#[test]
fn keyboard_targets_only_the_active_screen_and_ignores_releases() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut screens = manager();
    screens
        .create_screen("a", "A".into(), button("A", &calls).auto_focus())
        .unwrap();
    screens
        .create_screen("b", "B".into(), button("B", &calls).auto_focus())
        .unwrap();
    screens.process_event(&key(KeyCode::Enter)).unwrap();
    screens.switch_to_immediate(ScreenId::from("b")).unwrap();
    screens.process_event(&key(KeyCode::Enter)).unwrap();
    screens
        .process_event(&Event::Key(
            KeyEvent::new(KeyCode::Enter).with_kind(KeyEventKind::Release),
        ))
        .unwrap();
    assert_eq!(*calls.lock().unwrap(), ["A", "B"]);
}

#[test]
fn keyed_focus_survives_updates_and_mouse_bounds_follow_resize() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut screens = manager();
    screens
        .create_screen("main", "Main".into(), pair(false, &calls))
        .unwrap();
    screens.process_event(&key(KeyCode::Tab)).unwrap();
    screens
        .update_screen(&ScreenId::from("main"), pair(true, &calls))
        .unwrap();
    screens.process_event(&key(KeyCode::Enter)).unwrap();
    screens.process_event(&click(18, 0)).unwrap();
    screens
        .process_event(&Event::Resize(ResizeEvent::new(40, 4)))
        .unwrap();
    screens.process_event(&click(38, 0)).unwrap();
    assert_eq!(screens.size(), (40, 4));
    assert_eq!(*calls.lock().unwrap(), ["B", "A", "A"]);
}

#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<u8>>>);
impl std::io::Write for Capture {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(data);
        Ok(data.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl Capture {
    fn text(&self) -> String {
        let mut parser = vt100::Parser::new(4, 20, 0);
        parser.process(&self.0.lock().unwrap());
        parser.screen().contents()
    }
}

/// The capture once every presented frame has been written (PIP-001).
fn synced<'a>(manager: &mut ScreenManager, output: &'a Capture) -> &'a Capture {
    manager.sync().unwrap();
    output
}

static SCREEN_CLEANUPS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[reactive_tui::component]
fn ScreenCounter(hooks: &reactive_tui::reactive::Hooks) -> Element {
    use reactive_tui::reactive::{use_effect_with_deps, use_signal};
    let value = use_signal(hooks, 0_usize);
    use_effect_with_deps(hooks, (), move || {
        Some(Box::new(move || {
            SCREEN_CLEANUPS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }))
    });
    let label = format!("count:{}", value.get());
    div()
        .class("w-10 h-1")
        .text(&label)
        .on_click(move || value.update(|n| *n += 1))
        .build()
        .auto_focus()
}

#[test]
fn screens_retain_registered_components_and_remove_their_effects_once() {
    use std::sync::{atomic::Ordering, Once};
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        reactive_tui::component::registry::register_component::<ScreenCounter>("ScreenCounter")
            .unwrap()
    });
    let output = Capture::default();
    let backend = SuprTuiBackend::with_writer(20, 4, output.clone()).unwrap();
    let mut screens = ScreenManager::new(Box::new(backend));
    SCREEN_CLEANUPS.store(0, Ordering::SeqCst);
    let element = || ScreenCounter::element().key("counter");
    screens
        .create_screen("counter", "Counter".into(), element())
        .unwrap();
    assert_eq!(synced(&mut screens, &output).text().trim(), "count:0");
    screens.process_event(&key(KeyCode::Enter)).unwrap();
    assert_eq!(synced(&mut screens, &output).text().trim(), "count:1");
    screens
        .update_screen(&ScreenId::from("counter"), element())
        .unwrap();
    assert_eq!(synced(&mut screens, &output).text().trim(), "count:1");
    screens
        .create_screen("other", "Other".into(), Element::text("other"))
        .unwrap();
    screens
        .switch_to_immediate(ScreenId::from("other"))
        .unwrap();
    screens.process_event(&key(KeyCode::Enter)).unwrap();
    screens
        .switch_to_immediate(ScreenId::from("counter"))
        .unwrap();
    screens.process_event(&key(KeyCode::Enter)).unwrap();
    assert_eq!(synced(&mut screens, &output).text().trim(), "count:2");
    assert_eq!(SCREEN_CLEANUPS.load(Ordering::SeqCst), 0);
    screens.remove_screen(&ScreenId::from("counter")).unwrap();
    assert_eq!(SCREEN_CLEANUPS.load(Ordering::SeqCst), 1);
    drop(screens);
    assert_eq!(SCREEN_CLEANUPS.load(Ordering::SeqCst), 1);
}

#[test]
fn relative_opacity_must_start_from_the_presented_target_property() {
    use reactive_tui::animation::{
        api::{animate, AnimateParams, PropertyValue},
        AnimatedValue,
    };
    let mut screens = manager();
    screens
        .create_screen(
            "main",
            "Main".into(),
            div()
                .id("relative-target")
                .class("w-full h-full opacity-50")
                .text("CURRENT")
                .build(),
        )
        .unwrap();
    let mut animation = animate(
        screens
            .animation_target(&ScreenId::from("main"), "relative-target")
            .unwrap(),
        AnimateParams {
            opacity: Some(PropertyValue::Relative("+0.25".into())),
            autoplay: Some(false),
            ..Default::default()
        },
    );
    animation.seek(0.0);
    assert_eq!(
        animation.get_current_values(),
        Some(AnimatedValue::Opacity(0.5))
    );
    animation.seek(1.0);
    assert_eq!(
        animation.get_current_values(),
        Some(AnimatedValue::Opacity(0.75))
    );
}
