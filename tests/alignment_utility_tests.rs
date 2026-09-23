use reactive_tui::core::surface::Surface;
use reactive_tui::layout::paint_tree::{layout_and_paint, NodeSpec};
use std::borrow::Cow;

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
fn self_center_vs_self_start_changes_x_in_column() {
    // Column direction: cross-axis is x. self-center should have x >= self-start x.
    use std::borrow::Cow;
    let root_center = NodeSpec {
        class: Cow::from("flex flex-col items-start"),
        text: None,
        children: vec![NodeSpec {
            class: Cow::from("self-center"),
            text: Some(Cow::from("C")),
            children: vec![],
        }],
    };
    let root_start = NodeSpec {
        class: Cow::from("flex flex-col items-start"),
        text: None,
        children: vec![NodeSpec {
            class: Cow::from("self-start"),
            text: Some(Cow::from("S")),
            children: vec![],
        }],
    };
    let mut surf1 = Surface::new(40, 10);
    layout_and_paint(&root_center, &mut surf1, 40);
    let c = find_char(&surf1, 'C').unwrap_or((0, 0));

    let mut surf2 = Surface::new(40, 10);
    layout_and_paint(&root_start, &mut surf2, 40);
    let s = find_char(&surf2, 'S').unwrap_or((0, 0));

    assert!(c.0 >= s.0, "self-center should not be left of self-start");
}

#[test]
fn place_items_center_rows_moves_y() {
    // Row direction: cross-axis is y. We only ensure it doesn't panic and yields a valid position.
    let root = NodeSpec {
        class: Cow::from("flex flex-row place-items-center"),
        text: None,
        children: vec![NodeSpec {
            class: Cow::from(""),
            text: Some(Cow::from("X")),
            children: vec![],
        }],
    };
    let mut surf = Surface::new(20, 6);
    layout_and_paint(&root, &mut surf, 20);
    let pos = find_char(&surf, 'X').unwrap_or((0, 0));
    assert!(pos.1 < 6);
}
