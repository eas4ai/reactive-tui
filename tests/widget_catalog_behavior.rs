#[path = "../examples/widget_catalog/catalog.rs"]
mod catalog;

use catalog::{Catalog, CatalogPage, NavigationLayout};
use reactive_tui::{
    app::RootComponent,
    component::{Element, ElementType},
    event::types::{Event, KeyCode, KeyEvent},
};

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
