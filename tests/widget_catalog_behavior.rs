#[allow(dead_code)]
mod common;
use common::app_input;
#[path = "../examples/widget_catalog/catalog.rs"]
mod catalog;

use catalog::motion::{cube_frame, cube_frame_sized, CubeAnimation, FRAME_INTERVAL};
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
fn page_scroll_view_uses_the_terminal_stage_not_default_dimensions() {
    let mut catalog = Catalog::default();
    for (width, height) in [(60, 24), (144, 50), (200, 60)] {
        catalog.resize(width, height).unwrap();
        let element = catalog.render();
        let props = find_props::<reactive_tui::widgets::layout::ScrollViewProps>(&element).unwrap();
        let sidebar = width >= 80;
        assert_eq!(
            props.viewport_width,
            usize::from(width - if sidebar { 26 } else { 2 })
        );
        assert_eq!(
            props.viewport_height,
            usize::from(height - if sidebar { 8 } else { 11 })
        );
    }
}

#[test]
fn overview_coverage_wraps_and_navigation_entries_do_not_stretch() {
    let mut catalog = Catalog::default();
    catalog.resize(144, 50).unwrap();
    let frames = app_input::run_when(catalog, (144, 50), vec![("Coverage", None)]);
    let frame = &frames.last().unwrap().text;
    let rows: Vec<_> = frame.lines().collect();
    let nav_rows: Vec<_> = (1..=8)
        .map(|number| {
            rows.iter()
                .position(|row| row.contains(&format!("[{number}]")))
                .unwrap()
        })
        .collect();
    assert!(
        nav_rows.windows(2).all(|pair| pair[1] == pair[0] + 1),
        "{frame}"
    );
    assert!(
        rows.iter()
            .position(|row| row.contains("Coverage"))
            .unwrap()
            < 12,
        "{frame}"
    );
    assert!(
        frame.contains("TerminalWidget"),
        "coverage is clipped: {frame}"
    );
}

#[test]
fn layout_page_mounts_a_real_four_column_span_grid() {
    let mut catalog = Catalog::default();
    catalog.set_page(CatalogPage::Layout);
    let page = catalog.demo_element();
    let output = text(&page);
    for label in ["Column spans", "span 4", "span 3", "span 2", "span 1"] {
        assert!(output.contains(label), "missing {label}: {output}");
    }
}

#[test]
fn colored_column_spans_have_native_cell_geometry_at_each_viewport() {
    for size in [(60, 24), (144, 50), (200, 60)] {
        let mut catalog = Catalog::default();
        catalog.set_page(CatalogPage::Layout);
        let frames = app_input::run_when(catalog, size, vec![("span 3", None)]);
        let frame = frames.last().unwrap();
        let rows: Vec<_> = frame.text.lines().collect();
        let color_width = |label: &str| {
            let y = rows.iter().position(|row| row.contains(label)).unwrap();
            let x = rows[y].find(label).unwrap();
            let color = frame.screen.cell(y as u16, x as u16).unwrap().bgcolor();
            assert!(
                matches!(color, vt100::Color::Rgb(..)),
                "{size:?}: {color:?}\n{}",
                frame.text
            );
            (0..size.0)
                .filter(|x| frame.screen.cell(y as u16, *x).unwrap().bgcolor() == color)
                .count()
        };
        let full = color_width("span 4");
        let half = color_width("span 2");
        let three = color_width("span 3");
        assert!(
            full >= usize::from(size.0 - if size.0 >= 80 { 30 } else { 6 }),
            "{size:?}: span 4 width {full}\n{}",
            frame.text
        );
        assert!(
            half.abs_diff(full / 2) <= 2 && three.abs_diff(full * 3 / 4) <= 2,
            "{size:?}: {full}/{half}/{three}\n{}",
            frame.text
        );
    }
}

#[test]
fn both_layout_example_columns_are_visible_and_compact() {
    let mut catalog = Catalog::default();
    catalog.set_page(CatalogPage::Layout);
    let frames = app_input::run_when(catalog, (144, 50), vec![("Stack", None)]);
    let frame = &frames.last().unwrap().text;
    for label in ["Breadcrumb", "Accordion", "Tabs", "ScrollView", "Stack"] {
        assert!(frame.contains(label), "missing {label}:\n{frame}");
    }
    let rows: Vec<_> = frame.lines().collect();
    let breadcrumb = rows
        .iter()
        .position(|row| row.contains("Breadcrumb"))
        .unwrap();
    assert!(
        rows[breadcrumb].contains("Accordion"),
        "not two columns:\n{frame}"
    );
    let tabs = rows.iter().position(|row| row.contains("Tabs")).unwrap();
    assert!(tabs - breadcrumb <= 7, "cards are stretched:\n{frame}");
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
    catalog.try_handle_event(&key(KeyCode::Char('8'))).unwrap();
    assert_eq!(catalog.page(), CatalogPage::Motion);
    catalog.try_handle_event(&key(KeyCode::Char('5'))).unwrap();
    assert_eq!(catalog.page(), CatalogPage::Charts);
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
            CatalogPage::Charts,
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
fn sized_cube_frames_are_bounded_and_change_only_with_time() {
    for (width, height) in [
        (1, 1),
        (60, 15),
        (120, 44),
        (176, 54),
        (usize::MAX, usize::MAX),
    ] {
        let first = cube_frame_sized(Duration::ZERO, width, height);
        assert_eq!(first.lines().count(), height.min(100));
        assert!(first
            .lines()
            .all(|row| row.chars().count() == width.min(240)));
        assert!(first
            .chars()
            .all(|ch| ch == '\n' || ('\u{2800}'..='\u{28ff}').contains(&ch)));
        if width > 1 && height > 1 {
            assert_ne!(first, cube_frame_sized(FRAME_INTERVAL, width, height));
        }
    }
    assert!(cube_frame_sized(Duration::ZERO, 0, 24).is_empty());
    assert!(cube_frame_sized(Duration::ZERO, 60, 0).is_empty());
    let start = Instant::now();
    let mut animation = CubeAnimation::new(start);
    animation.set_viewport(120, 44);
    assert!(!animation.advance(start + FRAME_INTERVAL - Duration::from_nanos(1)));
    assert!(animation.advance(start + FRAME_INTERVAL));
    assert_eq!(animation.frame(), cube_frame_sized(FRAME_INTERVAL, 120, 44));
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
fn default_motion_canvas_tracks_the_terminal_and_uses_subcell_strokes() {
    use reactive_tui::backend::{Backend, DebugBackend};
    let mut catalog = Catalog::default();
    catalog.set_page(CatalogPage::Motion);
    for (width, height) in [(60, 24), (144, 50), (200, 60), (60, 24)] {
        catalog.resize(width, height).unwrap();
        let element = catalog.render();
        let output = text(&element);
        let canvas_width = usize::from(width - if width >= 80 { 24 } else { 0 });
        let canvas_height = usize::from(height - if width >= 80 { 6 } else { 9 });
        let braille_rows: Vec<_> = output
            .lines()
            .filter(|row| row.contains('\u{2800}'))
            .collect();
        assert_eq!(braille_rows.len(), canvas_height, "{output}");
        assert!(
            braille_rows.iter().all(|row| row
                .chars()
                .filter(|ch| ('\u{2800}'..='\u{28ff}').contains(ch))
                .count()
                == canvas_width),
            "{output}"
        );
        assert!(
            output
                .chars()
                .any(|ch| ('\u{2801}'..='\u{28ff}').contains(&ch)),
            "{output}"
        );
        let mut backend = DebugBackend::new(width, height);
        backend.render_full(&element).unwrap();
        let painted = backend.screen_content();
        assert!(
            painted.lines().next().unwrap().contains("Widget Catalog"),
            "{painted}"
        );
        assert!(
            painted.lines().last().unwrap().contains("Ctrl+Q"),
            "{painted}"
        );
        #[cfg(not(feature = "wgpu-graphics"))]
        {
            let mut live = Catalog::default();
            live.set_page(CatalogPage::Motion);
            let frames =
                app_input::run_when(live, (width, height), vec![("Braille subpixels", None)]);
            let painted = &frames.last().unwrap().text;
            assert_eq!(
                painted
                    .chars()
                    .filter(|ch| ('\u{2800}'..='\u{28ff}').contains(ch))
                    .count(),
                canvas_width * canvas_height,
                "{painted}"
            );
        }
    }
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
#[cfg(feature = "wgpu-graphics")]
#[test]
fn compact_graphics_stage_keeps_header_navigation_and_footer_visible() {
    use reactive_tui::backend::{Backend, DebugBackend};
    use reactive_tui::graphics::GraphicsOptions;
    let mut catalog = Catalog::with_graphics(
        GraphicsOptions {
            force_cpu: true,
            fault: None,
            ..Default::default()
        },
        true,
    );
    catalog.resize(60, 24).unwrap();
    catalog.attach_waker(reactive_tui::app::AppWaker::new());
    // A hang guard, not a timing check: generous so a busy machine cannot
    // fail a correct test by running it slowly.
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut backend = DebugBackend::new(60, 24);
    loop {
        catalog.update().unwrap();
        backend.render_full(&catalog.render()).unwrap();
        let output = backend.screen_content();
        if output.contains("CPU fallback") {
            assert!(
                output.lines().next().unwrap().contains("Reactive TUI"),
                "{output}"
            );
            assert!(output.contains("[8]"), "{output}");
            assert!(
                output.lines().last().unwrap().contains("Ctrl+Q quit"),
                "{output}"
            );
            assert_eq!(output.matches('▀').count(), 60 * 15, "{output}");
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(feature = "wgpu-graphics")]
#[test]
fn feature_enabled_motion_fills_the_available_stage() {
    use reactive_tui::backend::Backend;
    use reactive_tui::graphics::GraphicsOptions;
    let mut catalog = Catalog::with_graphics(
        GraphicsOptions {
            force_cpu: true,
            ..Default::default()
        },
        true,
    );
    catalog.resize(144, 50).unwrap();
    catalog.attach_waker(reactive_tui::app::AppWaker::new());
    // A hang guard, not a timing check: generous so a busy machine cannot
    // fail a correct test by running it slowly.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let mut backend = reactive_tui::backend::DebugBackend::new(144, 50);
    loop {
        catalog.update().unwrap();
        backend.render_full(&catalog.render()).unwrap();
        let output = backend.screen_content();
        if output.contains("CPU fallback") {
            assert_eq!(output.matches('▀').count(), 120 * 44);
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "CPU stage never painted: {output}"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
