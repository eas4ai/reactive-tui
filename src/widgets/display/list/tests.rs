use super::*;
use crate::accessibility::Role;
use crate::event::hit::Bounds;
use crate::event::types::{FocusEvent, KeyModifiers};
use crate::widgets::display::look;
use std::sync::Mutex;

/// `count` items `Item 1`, `Item 2`..., with the ids `i1`, `i2`....
fn numbered(count: usize) -> Vec<ListItem> {
    (1..=count)
        .map(|n| ListItem::new(&format!("i{n}"), &format!("Item {n}")))
        .collect()
}

/// DAT-008's list: Apple, Banana, a section Citrus of Lemon and Lime, and a
/// disabled Mango.
fn fruits() -> Vec<ListItem> {
    vec![
        ListItem::new("apple", "Apple"),
        ListItem::new("banana", "Banana"),
        ListItem::new("lemon", "Lemon").section("Citrus"),
        ListItem::new("lime", "Lime").section("Citrus"),
        ListItem::new("mango", "Mango").disabled(true),
    ]
}

type Log<T> = Arc<Mutex<Vec<T>>>;

/// A focused list of `props` laid out at 100 by `height` cells, recording
/// what it reports through `on_select`.
fn laid_out(mut props: ListProps, height: f32) -> (List, ListProps, Log<String>) {
    let selected: Log<String> = Arc::default();
    let record = selected.clone();
    let mut list =
        List::new(props.clone()).with_on_select(move |id| record.lock().unwrap().push(id));
    list.layout(
        LayoutInfo::from_bounds(Bounds::new(0.0, 0.0, 100.0, height)),
        &mut props,
        &mut (),
    );
    list.handle_event(
        &Event::Focus(FocusEvent {
            kind: FocusEventKind::Gained,
            timestamp: std::time::Instant::now(),
        }),
        &mut props,
        &mut (),
    );
    (list, props, selected)
}

fn press(list: &mut List, props: &mut ListProps, code: KeyCode) -> EventResult {
    list.handle_event(&Event::Key(KeyEvent::new(code)), props, &mut ())
}

fn typed(list: &mut List, props: &mut ListProps, text: &str) {
    for c in text.chars() {
        press(list, props, KeyCode::Char(c));
    }
}

/// The label of the item the cursor stands on.
fn cursor(list: &List, props: &ListProps) -> Option<String> {
    list.cursor.map(|item| props.items[item].label.clone())
}

/// Every node of `element`'s tree with `role`.
fn nodes(element: &Element, role: Role) -> Vec<accesskit::Node> {
    let mut found = Vec::new();
    let mut pending = vec![element];
    while let Some(element) = pending.pop() {
        if let Some(node) = &element.metadata.accessibility {
            if node.role() == role {
                found.push((*node.inner).clone());
            }
        }
        pending.extend(element.children.iter());
    }
    found
}

#[test]
fn pix_009_the_rounded_fills_change_no_cell_without_pixels() {
    use crate::layout::css::apply_utility_classes;
    use crate::layout::style::StyleBuilder;
    for fill in [look::CURSOR, look::SELECTED, look::HOVER] {
        assert_eq!(
            apply_utility_classes(
                &format!("flex flex-row {fill} {}", look::ROUNDED),
                StyleBuilder::new()
            ),
            apply_utility_classes(&format!("flex flex-row {fill}"), StyleBuilder::new()),
            "PIX-009: `rounded` after the fill {fill} changes no cell"
        );
    }
    let bar = crate::widgets::layout::look::NEW_BAR;
    assert_eq!(
        apply_utility_classes(&format!("whitespace-pre {bar}"), StyleBuilder::new()),
        apply_utility_classes(
            &format!("whitespace-pre {}", bar.replace(" rounded-full", "")),
            StyleBuilder::new()
        ),
        "PIX-009: `rounded-full` on the log view's bar changes no cell"
    );
}

#[test]
fn dat_007_a_list_of_100000_items_builds_only_the_rows_in_view() {
    let props = ListProps {
        items: numbered(100_000),
        selected: Some("i70000".to_owned()),
        ..ListProps::default()
    };
    let (list, props, _) = laid_out(props, 20.0);
    let element = list.render(&props, &());
    let options = nodes(&element, Role::ListBoxOption);
    assert_eq!(
        options.len(),
        20,
        "one option for each of the 20 rows in view"
    );
    let selected: Vec<_> = options
        .iter()
        .filter(|node| node.is_selected() == Some(true))
        .collect();
    assert_eq!(
        selected
            .iter()
            .map(|node| (
                node.label().map(str::to_owned),
                node.position_in_set(),
                node.size_of_set()
            ))
            .collect::<Vec<_>>(),
        vec![(Some("Item 70000".to_owned()), Some(70_000), Some(100_000))],
        "DAT-009: the selected item shows, at its place among all the items"
    );
}

#[test]
fn dat_008_the_keys_move_the_cursor_over_items_and_report_each_move() {
    let props = ListProps {
        items: fruits(),
        ..ListProps::default()
    };
    let (mut list, mut props, selected) = laid_out(props, 20.0);
    assert_eq!(cursor(&list, &props).as_deref(), Some("Apple"));
    press(&mut list, &mut props, KeyCode::Down);
    press(&mut list, &mut props, KeyCode::Down);
    assert_eq!(cursor(&list, &props).as_deref(), Some("Lemon"));
    press(&mut list, &mut props, KeyCode::End);
    assert_eq!(cursor(&list, &props).as_deref(), Some("Lime"));
    press(&mut list, &mut props, KeyCode::End);
    assert_eq!(
        *selected.lock().unwrap(),
        vec!["banana", "lemon", "lime"],
        "every move, and no key that does not move the cursor, is reported"
    );

    let props = ListProps {
        items: numbered(1000),
        ..ListProps::default()
    };
    let (mut list, mut props, _) = laid_out(props, 20.0);
    press(&mut list, &mut props, KeyCode::PageDown);
    assert_eq!(cursor(&list, &props).as_deref(), Some("Item 21"));
    assert_eq!(list.offset, 1, "the cursor's item is kept in view");
    press(&mut list, &mut props, KeyCode::End);
    press(&mut list, &mut props, KeyCode::PageUp);
    assert_eq!(cursor(&list, &props).as_deref(), Some("Item 980"));
}

#[test]
fn dat_008_a_filter_takes_typed_characters_and_escape_empties_it_then_cancels() {
    let cancels = Arc::new(Mutex::new(0));
    let count = cancels.clone();
    let props = ListProps {
        items: fruits(),
        filterable: true,
        ..ListProps::default()
    };
    let (list, mut props, _) = laid_out(props, 20.0);
    let mut list = list.with_on_cancel(move || *count.lock().unwrap() += 1);
    press(&mut list, &mut props, KeyCode::End);
    typed(&mut list, &mut props, "LE");
    assert_eq!(list.filter, "LE");
    assert_eq!(list.lines.shown, 2, "Apple and Lemon hold `le`");
    assert_eq!(cursor(&list, &props).as_deref(), Some("Apple"));
    // `s` is the Sort key, yet typed into the filter (KEY-001).
    press(&mut list, &mut props, KeyCode::Backspace);
    typed(&mut list, &mut props, "s");
    assert_eq!(list.filter, "Ls");
    press(&mut list, &mut props, KeyCode::Escape);
    assert_eq!((list.filter.as_str(), *cancels.lock().unwrap()), ("", 0));
    press(&mut list, &mut props, KeyCode::Escape);
    assert_eq!(
        *cancels.lock().unwrap(),
        1,
        "Escape on an empty filter cancels"
    );

    // A control chord is not text.
    let chord = KeyEvent::new(KeyCode::Char('a')).with_modifiers(KeyModifiers::ctrl());
    list.handle_event(&Event::Key(chord), &mut props, &mut ());
    assert_eq!(list.filter, "");
}

#[test]
fn dat_008_a_list_asks_for_more_once_until_its_items_change() {
    let asked = Arc::new(Mutex::new(0));
    let count = asked.clone();
    let props = ListProps {
        items: numbered(100),
        has_more: true,
        ..ListProps::default()
    };
    let (list, mut props, _) = laid_out(props, 20.0);
    let mut list = list.with_on_load_more(move || *count.lock().unwrap() += 1);
    press(&mut list, &mut props, KeyCode::Down);
    assert_eq!(
        *asked.lock().unwrap(),
        0,
        "80 items under the view ask for nothing"
    );
    press(&mut list, &mut props, KeyCode::End);
    press(&mut list, &mut props, KeyCode::Up);
    press(&mut list, &mut props, KeyCode::End);
    assert_eq!(*asked.lock().unwrap(), 1, "the list asks once");
    props.items = numbered(200);
    Component::update(&mut list, &props, &mut ());
    assert_eq!(
        cursor(&list, &props).as_deref(),
        Some("Item 100"),
        "the cursor keeps its item"
    );
    press(&mut list, &mut props, KeyCode::End);
    assert_eq!(
        *asked.lock().unwrap(),
        2,
        "new items let the list ask again"
    );
}
