use reactive_tui::core::surface::Surface;
use reactive_tui::layout::paint_tree::{layout_and_paint_constrained, NodeSpec};

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
fn self_align_in_column_exact_positions() {
    // Container width=10, children width=1; expect start=0, center=4..5, end=9 for x
    use std::borrow::Cow;
    let root = NodeSpec {
        class: Cow::from("w-10 h-5 flex flex-col items-start"),
        text: None,
        children: vec![
            NodeSpec {
                class: Cow::from("self-start w-1"),
                text: Some(Cow::from("A")),
                children: vec![],
            },
            NodeSpec {
                class: Cow::from("self-center w-1"),
                text: Some(Cow::from("B")),
                children: vec![],
            },
            NodeSpec {
                class: Cow::from("self-end w-1"),
                text: Some(Cow::from("C")),
                children: vec![],
            },
        ],
    };
    let mut surf = Surface::new(20, 10);
    layout_and_paint_constrained(&root, &mut surf, 20, 10);

    // Debug: print the surface
    eprintln!("Surface for self_align_in_column_exact_positions:");
    for y in 0..10 {
        let mut line = String::new();
        for x in 0..20 {
            let ch = surf.get(x, y).ch;
            line.push(if ch == ' ' { '.' } else { ch });
        }
        eprintln!("Row {}: {}", y, line);
    }

    let a = find_char(&surf, 'A').expect("A").0;
    let b = find_char(&surf, 'B').expect("B").0;
    let c = find_char(&surf, 'C').expect("C").0;
    assert_eq!(a, 0);
    assert!(
        b == 4 || b == 5,
        "center x should be around middle, got {b}"
    );
    assert_eq!(c, 9);
}

#[test]
fn place_items_center_in_row_exact_y() {
    // Container height=5, child height=1; center y = (5-1)/2 = 2
    use std::borrow::Cow;
    let root = NodeSpec {
        class: Cow::from("w-20 h-5 flex flex-row place-items-center"),
        text: None,
        children: vec![NodeSpec {
            class: Cow::from("w-1 h-1"),
            text: Some(Cow::from("X")),
            children: vec![],
        }],
    };
    let mut surf = Surface::new(40, 10);
    layout_and_paint_constrained(&root, &mut surf, 40, 10);

    // Debug: print the surface
    eprintln!("\nSurface for place_items_center_in_row_exact_y:");
    for y in 0..10 {
        let mut line = String::new();
        for x in 0..40 {
            let ch = surf.get(x, y).ch;
            line.push(if ch == ' ' { '.' } else { ch });
        }
        eprintln!("Row {}: {}", y, line);
    }

    let y = find_char(&surf, 'X').expect("X").1;
    assert_eq!(y, 2);
}
