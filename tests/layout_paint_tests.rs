use reactive_tui::core::surface::Surface;
use reactive_tui::layout::paint_tree::{layout_and_paint, NodeSpec};

fn find_char(surface: &Surface, ch: char) -> Option<(usize, usize)> {
    let (w, h) = surface.dims();
    for y in 0..h {
        for x in 0..w {
            if surface.get(x, y).ch == ch {
                return Some((x, y));
            }
        }
    }
    None
}

#[test]
fn padding_shifts_text_position() {
    // Root with padding p-8 (8 cells) should place text at (8,8)
    use std::borrow::Cow;
    let root = NodeSpec {
        class: Cow::from("p-8"),
        text: Some(Cow::from("X")),
        children: vec![],
    };
    let mut surf = Surface::new(20, 10);
    layout_and_paint(&root, &mut surf, 20);
    let pos = find_char(&surf, 'X').expect("X painted");
    assert_eq!(pos, (8, 8)); // p-8 is 8 cells
}

#[test]
fn flex_gap_places_second_child_further() {
    // Two children in a row with gap-x-12 (12 cells) and width 1
    // expect B.x = A.x + 1 (width) + 12 (gap)
    use std::borrow::Cow;
    let root = NodeSpec {
        class: Cow::from("flex flex-row gap-x-12"),
        text: None,
        children: vec![
            NodeSpec {
                class: Cow::from("w-1"),
                text: Some(Cow::from("A")),
                children: vec![],
            },
            NodeSpec {
                class: Cow::from("w-1"),
                text: Some(Cow::from("B")),
                children: vec![],
            },
        ],
    };
    let mut surf = Surface::new(40, 5);
    layout_and_paint(&root, &mut surf, 40);
    let a = find_char(&surf, 'A').expect("A");
    let b = find_char(&surf, 'B').expect("B");
    assert_eq!(b.0, a.0 + 1 + 12); // gap-x-12 is 12 cells
}

#[test]
fn child_margin_affects_position() {
    // Parent flex; child with margin-left ml-16 (16 cells) should appear at x == 16
    use std::borrow::Cow;
    let root = NodeSpec {
        class: Cow::from("flex flex-row"),
        text: None,
        children: vec![NodeSpec {
            class: Cow::from("ml-16 w-1 h-1"), // Add explicit size for text node
            text: Some(Cow::from("M")),
            children: vec![],
        }],
    };
    let mut surf = Surface::new(40, 5);
    layout_and_paint(&root, &mut surf, 40);
    let m = find_char(&surf, 'M').expect("M");
    assert_eq!(m.0, 16); // ml-16 is 16 cells
}
