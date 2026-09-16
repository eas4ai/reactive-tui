#[path = "support/app_input.rs"]
#[allow(dead_code)]
mod app_input;
#[path = "../examples/widget_catalog/catalog.rs"]
mod catalog;

use catalog::motion::{cube_frame, CubeAnimation, FRAME_INTERVAL};
use catalog::{Catalog, CatalogPage, NavigationLayout};
use reactive_tui::{
    app::RootComponent,
    component::{Element, ElementType},
    event::types::{Event, KeyCode, KeyEvent},
};
use std::time::{Duration, Instant};

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code))
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

fn component_names(element: &Element) -> String {
    let mut names = element.component_name().unwrap_or_default().to_string();
    for child in &element.children {
        names.push_str(&component_names(child));
    }
    if let Some(props) = element.props_as::<reactive_tui::widgets::layout::ScrollViewProps>() {
        names.push_str(&component_names(&props.content));
    }
    names
}

fn find_props<T: std::any::Any>(element: &Element) -> Option<&T> {
    element
        .props_as::<T>()
        .or_else(|| element.children.iter().find_map(find_props::<T>))
        .or_else(|| {
            element
                .props_as::<reactive_tui::widgets::layout::ScrollViewProps>()
                .and_then(|props| find_props::<T>(&props.content))
        })
}

#[test]
fn compact_and_wide_navigation_paint_every_shortcut_after_resize() {
    use reactive_tui::backend::{Backend, DebugBackend};
    let mut catalog = Catalog::default();
    let mut backend = DebugBackend::new(100, 32);
    for width in [100, 60, 79, 80, 40] {
        catalog.resize(width, 32).unwrap();
        backend.resize(width as usize, 32);
        backend.render_full(&catalog.render()).unwrap();
        let frame = backend.screen_content();
        for number in 1..=8 {
            assert!(
                frame.contains(&format!("[{number}]")),
                "shortcut {number} clipped at {width}:\n{frame}"
            );
        }
    }
}

#[test]
fn dialogs_are_live_and_separately_reachable() {
    let mut catalog = Catalog::default();
    catalog.set_page(CatalogPage::MenusDialogs);
    let mut names = String::new();
    for _ in 0..12 {
        names.push_str(&component_names(&catalog.demo_element()));
        catalog.try_handle_event(&key(KeyCode::F(2))).unwrap();
    }
    assert!(
        names.contains("LiveInput"),
        "InputDialog is not mounted: {names}"
    );
    assert!(
        names.contains("LiveAutocomplete"),
        "AutocompleteDialog is not mounted: {names}"
    );
}

#[test]
fn live_dialogs_switch_through_real_app_input() {
    for size in [(100, 32), (60, 24)] {
        let mut catalog = Catalog::default();
        catalog.set_page(CatalogPage::MenusDialogs);
        for _ in 0..7 {
            catalog.try_handle_event(&key(KeyCode::F(2))).unwrap();
        }
        let frames = app_input::run_when(
            catalog,
            size,
            vec![
                ("Capture name", Some(key(KeyCode::F(2)))),
                ("Find a widget", None),
            ],
        );
        assert!(frames
            .iter()
            .any(|frame| frame.text.contains("Capture name")));
        assert!(frames
            .iter()
            .any(|frame| frame.text.contains("Find a widget")));
    }
}

#[test]
fn context_and_popup_menus_are_visible_when_selected() {
    for demo in [1, 2] {
        let mut catalog = Catalog::default();
        catalog.set_page(CatalogPage::MenusDialogs);
        for _ in 0..demo {
            catalog.try_handle_event(&key(KeyCode::F(2))).unwrap();
        }
        let frames = app_input::run_when(catalog, (100, 32), vec![("Export clip", None)]);
        assert!(frames
            .iter()
            .any(|frame| frame.text.contains("New capture")));
    }
}

#[test]
fn table_has_representative_rows_and_logo_is_decodable() {
    use reactive_tui::widgets::display::table::TableProps;
    let mut catalog = Catalog::default();
    catalog.set_page(CatalogPage::Data);
    let frame = catalog.demo_element();
    let props = find_props::<TableProps>(&frame).expect("Table props are absent");
    assert!(props.columns.len() >= 2 && props.rows.len() >= 2);
    catalog.set_page(CatalogPage::Media);
    assert!(component_names(&catalog.demo_element()).contains("Image"));
    let logo = ::image::load_from_memory(include_bytes!("../manual/assets/logo.jpg")).unwrap();
    assert!(logo.width() > 0 && logo.height() > 0);
}

#[test]
fn page_navigation_wraps_in_both_directions() {
    let mut catalog = Catalog::default();
    assert_eq!(catalog.page(), CatalogPage::Overview);

    catalog.try_handle_event(&key(KeyCode::Up)).unwrap();
    assert_eq!(catalog.page(), CatalogPage::System);

    catalog.try_handle_event(&key(KeyCode::Down)).unwrap();
    assert_eq!(catalog.page(), CatalogPage::Overview);
}

#[test]
fn number_keys_jump_to_the_displayed_page() {
    let mut catalog = Catalog::default();
    catalog.try_handle_event(&key(KeyCode::Char('7'))).unwrap();
    assert_eq!(catalog.page(), CatalogPage::Motion);
}

#[test]
fn navigation_changes_at_the_eighty_column_breakpoint() {
    let mut catalog = Catalog::default();
    catalog.resize(79, 24).unwrap();
    assert_eq!(catalog.navigation_layout(), NavigationLayout::Compact);
    catalog.resize(80, 24).unwrap();
    assert_eq!(catalog.navigation_layout(), NavigationLayout::Sidebar);
}

#[test]
fn every_page_renders_a_focused_stage() {
    let mut catalog = Catalog::default();
    for page in CatalogPage::ALL {
        catalog.set_page(page);
        let output = text(&catalog.render());
        assert!(
            output.contains(page.title()),
            "missing title for {page:?}: {output}"
        );
        assert!(output.contains("Ctrl+Q"), "quit help is absent on {page:?}");
    }
}

#[test]
fn catalog_exposes_the_complete_page_order() {
    assert_eq!(
        CatalogPage::ALL,
        [
            CatalogPage::Overview,
            CatalogPage::Input,
            CatalogPage::Layout,
            CatalogPage::Data,
            CatalogPage::MenusDialogs,
            CatalogPage::Media,
            CatalogPage::Motion,
            CatalogPage::System,
        ]
    );
}

#[test]
fn cube_frames_are_distinct_and_bounded() {
    let first = cube_frame(Duration::ZERO);
    assert_ne!(first, cube_frame(FRAME_INTERVAL));
    for second in 0..100 {
        let frame = cube_frame(Duration::from_millis(second * 80));
        assert_eq!(frame.lines().count(), 16);
        assert!(frame.lines().all(|line| line.chars().count() == 40));
    }
}

#[test]
fn animation_waits_for_its_deadline_and_skips_missed_frames() {
    let start = Instant::now();
    let mut animation = CubeAnimation::new(start);
    assert!(!animation.advance(start + FRAME_INTERVAL - Duration::from_nanos(1)));
    assert!(animation.advance(start + FRAME_INTERVAL));
    assert!(!animation.advance(start + FRAME_INTERVAL));
    let late = start + Duration::from_secs(10);
    assert!(animation.advance(late));
    assert!(!animation.advance(late + Duration::from_millis(1)));
}

#[test]
fn advertised_quit_keys_request_normal_app_exit() {
    use reactive_tui::{app::RootUpdate, event::types::KeyModifiers};
    for code in [KeyCode::Char('q'), KeyCode::Char('c'), KeyCode::Escape] {
        let mut catalog = Catalog::default();
        let event = Event::Key(KeyEvent::new(code).with_modifiers(KeyModifiers {
            ctrl: true,
            ..KeyModifiers::empty()
        }));
        catalog.try_handle_event(&event).unwrap();
        assert!(matches!(catalog.update().unwrap(), RootUpdate::Exit));
    }
}

#[test]
fn released_quit_keys_and_plain_letters_do_not_exit() {
    use reactive_tui::{
        app::RootUpdate,
        event::types::{KeyEventKind, KeyModifiers},
    };
    for event in [
        key(KeyCode::Char('q')),
        key(KeyCode::Char('c')),
        Event::Key(
            KeyEvent::new(KeyCode::Char('q'))
                .with_modifiers(KeyModifiers {
                    ctrl: true,
                    ..KeyModifiers::empty()
                })
                .with_kind(KeyEventKind::Release),
        ),
    ] {
        let mut catalog = Catalog::default();
        catalog.try_handle_event(&event).unwrap();
        assert!(matches!(catalog.update().unwrap(), RootUpdate::Unchanged));
    }
}
