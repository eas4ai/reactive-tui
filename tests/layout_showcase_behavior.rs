#[path = "../examples/layout_showcase/layouts.rs"]
mod layouts;

use layouts::{Showcase, ShowcasePage};
use reactive_tui::{
    app::{RootComponent, RootUpdate},
    component::{Element, ElementType},
    event::types::{Event, KeyCode, KeyEvent, KeyModifiers},
};

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

#[test]
fn every_layout_renders_its_regions() {
    let regions: [(ShowcasePage, &[&str]); 5] = [
        (ShowcasePage::Ide, &["Files", "Editor", "Terminal"]),
        (
            ShowcasePage::Dashboard,
            &["Revenue", "Traffic", "Activity", "Top pages"],
        ),
        (
            ShowcasePage::Mail,
            &["Folders", "Inbox", "Reading", "Compose"],
        ),
        (ShowcasePage::Store, &["Keyboard", "Monitor", "Cart"]),
        (ShowcasePage::Monitor, &["CPU", "MEM", "Processes", "Log"]),
    ];
    for (page, expected) in regions {
        let mut showcase = Showcase::default();
        let number = ShowcasePage::ALL.iter().position(|p| *p == page).unwrap() + 1;
        showcase
            .try_handle_event(&key(KeyCode::Char(
                char::from_digit(number as u32, 10).unwrap(),
            )))
            .unwrap();
        let output = text(&showcase.render());
        assert!(output.contains(page.title()), "{page:?}");
        for region in expected {
            assert!(output.contains(region), "{page:?} missing {region}");
        }
    }
}

#[test]
fn tab_cycles_forward_and_wraps() {
    let mut showcase = Showcase::default();
    assert_eq!(showcase.page(), ShowcasePage::Ide);
    for expected in [
        ShowcasePage::Dashboard,
        ShowcasePage::Mail,
        ShowcasePage::Store,
        ShowcasePage::Monitor,
        ShowcasePage::Ide,
    ] {
        showcase.try_handle_event(&key(KeyCode::Tab)).unwrap();
        assert_eq!(showcase.page(), expected);
    }
}

#[test]
fn shift_tab_and_arrows_cycle_backward() {
    let mut showcase = Showcase::default();
    showcase.try_handle_event(&key(KeyCode::BackTab)).unwrap();
    assert_eq!(showcase.page(), ShowcasePage::Monitor);
    showcase.try_handle_event(&key(KeyCode::Left)).unwrap();
    assert_eq!(showcase.page(), ShowcasePage::Store);
    showcase.try_handle_event(&key(KeyCode::Up)).unwrap();
    assert_eq!(showcase.page(), ShowcasePage::Mail);
}

#[test]
fn number_keys_jump_and_footer_names_tab_and_quit() {
    let mut showcase = Showcase::default();
    showcase.try_handle_event(&key(KeyCode::Char('4'))).unwrap();
    assert_eq!(showcase.page(), ShowcasePage::Store);
    for (width, footer) in [(100, "Tab next layout"), (60, "Tab next · 1–5")] {
        showcase.resize(width, 24).unwrap();
        let output = text(&showcase.render());
        assert!(output.contains(footer), "{width}: {output}");
        assert!(output.contains("Ctrl+Q"), "{width}: {output}");
    }
}

fn has_class(element: &Element, needle: &str) -> bool {
    if element
        .class
        .as_deref()
        .is_some_and(|class| class.contains(needle))
    {
        return true;
    }
    element
        .children
        .iter()
        .any(|child| has_class(child, needle))
}

#[test]
fn product_grid_grows_with_terminal_width() {
    for (width, cols) in [(40, 1), (60, 2), (100, 3), (150, 5), (220, 7), (520, 8)] {
        let mut showcase = Showcase::default();
        showcase.resize(width, 32).unwrap();
        showcase.try_handle_event(&key(KeyCode::Char('4'))).unwrap();
        assert_eq!(showcase.page(), ShowcasePage::Store);
        assert!(
            has_class(&showcase.render(), &format!("grid-cols-{cols}")),
            "{width}"
        );
    }
}

#[test]
fn ctrl_q_requests_normal_app_exit() {
    let mut showcase = Showcase::default();
    showcase
        .try_handle_event(&ctrl_key(KeyCode::Char('q')))
        .unwrap();
    assert!(matches!(showcase.update().unwrap(), RootUpdate::Exit));
    let mut plain = Showcase::default();
    plain.try_handle_event(&key(KeyCode::Char('q'))).unwrap();
    assert!(matches!(plain.update().unwrap(), RootUpdate::Unchanged));
}
