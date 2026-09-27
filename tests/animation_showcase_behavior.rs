#[path = "../examples/animation_showcase/showcase.rs"]
mod showcase;

use reactive_tui::{
    app::{RootComponent, RootUpdate},
    component::{Element, ElementType},
    event::types::{Event, KeyCode, KeyEvent, KeyModifiers},
};
use showcase::animations::{
    donut_frame, fire_frame, plasma_frame, ripple_frame, warp_frame, Animation, FRAME_INTERVAL,
    MAX_HEIGHT, MAX_WIDTH,
};
use showcase::ShowcasePage;
use std::time::{Duration, Instant};

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code))
}

fn ctrl_key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code).with_modifiers(KeyModifiers {
        ctrl: true,
        ..KeyModifiers::empty()
    }))
}

fn text(element: &Element) -> String {
    let mut output = match &element.element_type {
        ElementType::Text(value) => value.clone(),
        _ => String::new(),
    };
    for child in &element.children {
        output.push_str(&text(child));
    }
    output
}

fn check_frame(frame: &str, width: usize, height: usize) {
    let rows: Vec<&str> = frame.lines().collect();
    assert_eq!(rows.len(), height.min(MAX_HEIGHT), "{frame}");
    assert!(
        rows.iter()
            .all(|row| row.chars().count() == width.min(MAX_WIDTH)),
        "{frame}"
    );
    assert!(
        frame
            .chars()
            .all(|ch| ch == '\n' || ('\u{2800}'..='\u{28ff}').contains(&ch)),
        "{frame}"
    );
}

#[test]
fn every_animation_is_bounded_braille_and_moves() {
    let renderers: [fn(Duration, usize, usize) -> String; 5] = [
        donut_frame,
        plasma_frame,
        warp_frame,
        fire_frame,
        ripple_frame,
    ];
    for render in renderers {
        for (width, height) in [(60, 15), (120, 44), (500, 90)] {
            let first = render(Duration::ZERO, width, height);
            check_frame(&first, width, height);
            let second = render(FRAME_INTERVAL, width, height);
            assert_ne!(first, second, "{width}x{height}");
        }
        assert!(render(Duration::ZERO, 0, 24).is_empty());
        assert!(render(Duration::ZERO, 60, 0).is_empty());
        let oversized = render(Duration::ZERO, usize::MAX, usize::MAX);
        check_frame(&oversized, MAX_WIDTH, MAX_HEIGHT);
    }
}

#[test]
fn driver_centers_small_canvases_and_skips_missed_frames() {
    let start = Instant::now();
    let mut animation = Animation::new(start, donut_frame);
    animation.set_viewport(60, 20);
    assert!(!animation.advance(start + FRAME_INTERVAL - Duration::from_nanos(1)));
    assert!(animation.advance(start + FRAME_INTERVAL));
    assert!(!animation.advance(start + FRAME_INTERVAL));
    let centered = animation.centered(100, 32);
    let rows: Vec<&str> = centered.lines().collect();
    assert_eq!(rows.len(), 32, "{centered}");
    assert!(rows.iter().all(|row| row.chars().count() == 100));
    assert!(rows[16].contains('\u{2800}'));
}

#[test]
fn tab_cycles_every_page_and_wraps() {
    use showcase::Showcase;
    let mut showcase = Showcase::default();
    assert_eq!(showcase.page(), ShowcasePage::Donut);
    for _ in 0..ShowcasePage::ALL.len() {
        showcase.try_handle_event(&key(KeyCode::Tab)).unwrap();
    }
    assert_eq!(showcase.page(), ShowcasePage::Donut);
    showcase.try_handle_event(&key(KeyCode::BackTab)).unwrap();
    assert_eq!(
        showcase.page(),
        ShowcasePage::ALL[ShowcasePage::ALL.len() - 1]
    );
}

#[test]
fn every_page_renders_title_quit_help_and_footer() {
    use showcase::Showcase;
    let mut showcase = Showcase::default();
    for page in ShowcasePage::ALL {
        if let Some(digit) = char::from_digit(
            ShowcasePage::ALL.iter().position(|p| *p == page).unwrap() as u32 + 1,
            10,
        ) {
            showcase
                .try_handle_event(&key(KeyCode::Char(digit)))
                .unwrap();
        }
        assert_eq!(showcase.page(), page);
        showcase.resize(100, 32).unwrap();
        let output = text(&showcase.render());
        assert!(output.contains(page.title()), "{page:?}: {output}");
        assert!(output.contains("Ctrl+Q"), "{page:?}");
    }
}

#[test]
fn ctrl_q_requests_normal_app_exit() {
    use showcase::Showcase;
    let mut showcase = Showcase::default();
    showcase
        .try_handle_event(&ctrl_key(KeyCode::Char('q')))
        .unwrap();
    assert!(matches!(showcase.update().unwrap(), RootUpdate::Exit));
}

#[cfg(feature = "wgpu-graphics")]
#[test]
fn graphics_showcase_reaches_the_shader_page() {
    use reactive_tui::graphics::GraphicsOptions;
    use showcase::{Showcase, ShowcasePage};
    let mut showcase = Showcase::with_graphics(GraphicsOptions::default());
    showcase.try_handle_event(&key(KeyCode::Char('6'))).unwrap();
    assert_eq!(showcase.page(), ShowcasePage::Shader);
    let output = text(&showcase.render());
    assert!(output.contains("Raymarched torus"));
}
