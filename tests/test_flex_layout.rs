use reactive_tui::core::geometry::Point;
use reactive_tui::core::surface::Surface;
use reactive_tui::layout::paint_tree::{layout_and_paint_with, NodeSpec, PaintOptions};
use std::borrow::Cow;

/// Read a surface region as text rows for structural assertions.
fn region_text(surface: &Surface, x0: usize, y0: usize, x1: usize, y1: usize) -> Vec<String> {
    (y0..y1)
        .map(|y| {
            (x0..x1)
                .map(|x| surface.get_at(Point::new(x, y)).ch)
                .collect::<String>()
        })
        .collect()
}

#[test]
fn test_flex_row_layout() {
    let root = NodeSpec {
        class: Cow::Borrowed("flex flex-row w-40 h-10 gap-2"),
        text: None,
        children: vec![
            NodeSpec {
                class: Cow::Borrowed("w-8 h-3 bg-red-500"),
                text: Some(Cow::Borrowed("Item1")),
                children: vec![],
            },
            NodeSpec {
                class: Cow::Borrowed("w-8 h-3 bg-blue-500"),
                text: Some(Cow::Borrowed("Item2")),
                children: vec![],
            },
            NodeSpec {
                class: Cow::Borrowed("w-8 h-3 bg-green-500"),
                text: Some(Cow::Borrowed("Item3")),
                children: vec![],
            },
        ],
    };

    let mut surface = Surface::new(50, 12);
    let opts = PaintOptions {
        debug_overlay: false,
    };

    layout_and_paint_with(&root, &mut surface, 40, &opts).unwrap();

    // Items share one row, ordered left to right.
    let rows = region_text(&surface, 0, 0, 50, 5);
    let locate = |needle: &str| {
        rows.iter()
            .enumerate()
            .find_map(|(y, row)| row.find(needle).map(|x| (y, x)))
            .unwrap_or_else(|| panic!("{needle} painted"))
    };
    let ((y1, x1), (y2, x2), (y3, x3)) = (locate("Item1"), locate("Item2"), locate("Item3"));
    assert_eq!((y1, y2, y3), (y1, y1, y1), "row items split: {rows:?}");
    assert!(x1 < x2 && x2 < x3, "row items out of order: {rows:?}");
}

#[test]
fn test_flex_column_layout() {
    let root = NodeSpec {
        class: Cow::Borrowed("flex flex-col w-20 h-15 gap-1"),
        text: None,
        children: vec![
            NodeSpec {
                class: Cow::Borrowed("w-full h-3 bg-red-500"),
                text: Some(Cow::Borrowed("Row 1")),
                children: vec![],
            },
            NodeSpec {
                class: Cow::Borrowed("w-full h-3 bg-blue-500"),
                text: Some(Cow::Borrowed("Row 2")),
                children: vec![],
            },
            NodeSpec {
                class: Cow::Borrowed("w-full h-3 bg-green-500"),
                text: Some(Cow::Borrowed("Row 3")),
                children: vec![],
            },
        ],
    };

    let mut surface = Surface::new(25, 15);
    let opts = PaintOptions {
        debug_overlay: false,
    };

    layout_and_paint_with(&root, &mut surface, 20, &opts).unwrap();

    // Rows stack top to bottom in order.
    let rows = region_text(&surface, 0, 0, 25, 15);
    let row_of = |needle: &str| {
        rows.iter()
            .position(|row| row.contains(needle))
            .unwrap_or_else(|| panic!("{needle} painted"))
    };
    let (r1, r2, r3) = (row_of("Row 1"), row_of("Row 2"), row_of("Row 3"));
    assert!(r1 < r2 && r2 < r3, "column items out of order: {rows:?}");
}

#[test]
fn test_justify_content() {
    let root = NodeSpec {
        class: Cow::Borrowed("flex flex-row justify-center w-40 h-5"),
        text: None,
        children: vec![NodeSpec {
            class: Cow::Borrowed("w-6 h-3"),
            text: Some(Cow::Borrowed("CENTER")),
            children: vec![],
        }],
    };

    let mut surface = Surface::new(45, 7);
    let opts = PaintOptions {
        debug_overlay: false,
    };

    layout_and_paint_with(&root, &mut surface, 40, &opts).unwrap();

    // CENTER sits away from the left edge, roughly symmetric.
    let rows = region_text(&surface, 0, 0, 40, 4);
    let row = rows
        .iter()
        .find(|row| row.contains("CENTER"))
        .expect("CENTER painted");
    let start = row.find("CENTER").unwrap();
    let end = start + "CENTER".len();
    assert!(start > 0, "not centered, flush left:\n{row}");
    let (left, right) = (start, 40 - end);
    assert!(
        left.abs_diff(right) <= 1,
        "not centered: left={left} right={right}\n{row}"
    );
}

#[test]
fn test_grid_layout() {
    let root = NodeSpec {
        class: Cow::Borrowed("grid grid-cols-3 gap-1 w-30 h-10"),
        text: None,
        children: vec![
            NodeSpec {
                class: Cow::Borrowed(""),
                text: Some(Cow::Borrowed("A")),
                children: vec![],
            },
            NodeSpec {
                class: Cow::Borrowed(""),
                text: Some(Cow::Borrowed("B")),
                children: vec![],
            },
            NodeSpec {
                class: Cow::Borrowed(""),
                text: Some(Cow::Borrowed("C")),
                children: vec![],
            },
            NodeSpec {
                class: Cow::Borrowed(""),
                text: Some(Cow::Borrowed("D")),
                children: vec![],
            },
            NodeSpec {
                class: Cow::Borrowed(""),
                text: Some(Cow::Borrowed("E")),
                children: vec![],
            },
            NodeSpec {
                class: Cow::Borrowed(""),
                text: Some(Cow::Borrowed("F")),
                children: vec![],
            },
        ],
    };

    let mut surface = Surface::new(35, 12);
    let opts = PaintOptions {
        debug_overlay: false,
    };

    layout_and_paint_with(&root, &mut surface, 30, &opts).unwrap();

    // A B C share the first row in order; D E F sit below in order.
    let rows = region_text(&surface, 0, 0, 35, 12);
    let row_of = |needle: char| {
        rows.iter()
            .position(|row| row.contains(needle))
            .unwrap_or_else(|| panic!("{needle} painted"))
    };
    let (ra, rb, rc) = (row_of('A'), row_of('B'), row_of('C'));
    let (rd, re, rf) = (row_of('D'), row_of('E'), row_of('F'));
    assert_eq!((ra, rb, rc), (ra, ra, ra), "first grid row split: {rows:?}");
    assert_eq!(
        (rd, re, rf),
        (rd, rd, rd),
        "second grid row split: {rows:?}"
    );
    assert!(ra < rd, "grid rows out of order: {rows:?}");
    let col_of = |needle: char| rows[ra].find(needle).unwrap();
    assert!(
        col_of('A') < col_of('B') && col_of('B') < col_of('C'),
        "grid columns out of order: {rows:?}"
    );
}
