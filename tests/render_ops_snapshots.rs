use reactive_tui::core::render_ops::{RenderOp, RenderOps, RenderOpsBuilder};
use reactive_tui::core::styled_text::{styled_line_to_render_ops, StyledLineBuilder};
use reactive_tui::core::surface::{Attr, Rgba};
use reactive_tui::core::writer::render_ops_to_ansi;

/// Goldens live in `tests/snapshots/render_ops/<name>.ansi`, reviewed as a
/// diff like any other source file. Run with `REGENERATE=1` to refresh
/// them after an intentional rendering change — then review the diff
/// before committing.
fn snapshots_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots/render_ops")
}

/// Helper to create a snapshot test
fn assert_snapshot(name: &str, ops: &RenderOps) {
    let output = render_ops_to_ansi(ops);

    // Ensure output is not empty for non-empty ops
    if !ops.is_empty() {
        assert!(
            !output.is_empty(),
            "Output should not be empty for {}",
            name
        );
    }

    let path = snapshots_dir().join(format!("{name}.ansi"));
    if std::env::var("REGENERATE").as_deref() == Ok("1") {
        std::fs::create_dir_all(snapshots_dir()).expect("snapshot dir");
        std::fs::write(&path, &output).expect("write golden");
        return;
    }
    let expected = std::fs::read(&path).unwrap_or_else(|_| {
        panic!("missing golden {path:?}; run with REGENERATE=1 to create it, review the diff, then commit it")
    });
    assert_eq!(
        output, expected,
        "golden mismatch for {name}; if the rendering change is intentional, run with REGENERATE=1, review the diff, and commit it"
    );
}

#[test]
fn snapshot_list_widget() {
    let mut builder = RenderOpsBuilder::new();
    let fg = Rgba {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    let bg = Rgba {
        r: 0.0,
        g: 0.0,
        b: 0.2,
        a: 1.0,
    };
    let selected_bg = Rgba {
        r: 0.0,
        g: 0.2,
        b: 0.4,
        a: 1.0,
    };

    // Simulate a list widget with 3 items, second selected
    builder
        .move_to(0, 0)
        .set_fg(fg)
        .set_bg(bg)
        .print("  Item 1  ")
        .move_to(0, 1)
        .set_bg(selected_bg)
        .set_attr(Attr::BOLD)
        .print("> Item 2  ")
        .move_to(0, 2)
        .set_bg(bg)
        .set_attr(Attr::empty())
        .print("  Item 3  ");

    let ops = builder.build();
    assert_snapshot("list_widget", &ops);

    // Verify specific operations exist
    assert!(ops
        .ops()
        .iter()
        .any(|op| matches!(op, RenderOp::MoveTo { x: 0, y: 0 })));
    assert!(ops
        .ops()
        .iter()
        .any(|op| matches!(op, RenderOp::SetAttributes(Attr::BOLD))));
    assert!(ops
        .ops()
        .iter()
        .any(|op| matches!(op, RenderOp::PrintRun(s) if s.contains("Item 2"))));
}

#[test]
fn snapshot_paragraph_widget() {
    let mut builder = RenderOpsBuilder::new();
    let fg = Rgba {
        r: 0.9,
        g: 0.9,
        b: 0.9,
        a: 1.0,
    };
    let bg = Rgba {
        r: 0.1,
        g: 0.1,
        b: 0.1,
        a: 1.0,
    };

    // Simulate a paragraph with wrapped text
    let lines = [
        "This is a paragraph widget that",
        "demonstrates text wrapping and",
        "proper styling across multiple",
        "lines of content.",
    ];

    for (i, line) in lines.iter().enumerate() {
        builder
            .move_to(2, i as u16 + 1)
            .set_fg(fg)
            .set_bg(bg)
            .print(*line);
    }

    let ops = builder.build();
    assert_snapshot("paragraph_widget", &ops);

    // Verify we have the right number of move operations
    let move_count = ops
        .ops()
        .iter()
        .filter(|op| matches!(op, RenderOp::MoveTo { .. }))
        .count();
    assert_eq!(move_count, 4, "Should have one MoveTo per line");
}

#[test]
fn snapshot_table_widget() {
    let mut builder = RenderOpsBuilder::new();
    let header_fg = Rgba {
        r: 1.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    let header_bg = Rgba {
        r: 0.2,
        g: 0.2,
        b: 0.0,
        a: 1.0,
    };
    let cell_fg = Rgba {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    let cell_bg = Rgba {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    // Table header
    builder
        .move_to(0, 0)
        .set_fg(header_fg)
        .set_bg(header_bg)
        .set_attr(Attr::BOLD)
        .print("│ Name     │ Age │ City      │");

    // Separator
    builder
        .move_to(0, 1)
        .set_fg(cell_fg)
        .set_bg(cell_bg)
        .set_attr(Attr::empty())
        .print("├──────────┼─────┼───────────┤");

    // Data rows
    let data = [
        ("Alice", "25", "New York"),
        ("Bob", "30", "London"),
        ("Charlie", "35", "Tokyo"),
    ];

    for (i, (name, age, city)) in data.iter().enumerate() {
        builder
            .move_to(0, i as u16 + 2)
            .print(format!("│ {:<8} │ {:<3} │ {:<9} │", name, age, city));
    }

    let ops = builder.build();
    assert_snapshot("table_widget", &ops);

    // Verify table structure
    assert!(ops
        .ops()
        .iter()
        .any(|op| matches!(op, RenderOp::SetAttributes(Attr::BOLD))));
    assert!(ops
        .ops()
        .iter()
        .any(|op| matches!(op, RenderOp::PrintRun(s) if s.contains("Name"))));
}

#[test]
fn snapshot_styled_text() {
    let fg_normal = Rgba {
        r: 0.8,
        g: 0.8,
        b: 0.8,
        a: 1.0,
    };
    let fg_keyword = Rgba {
        r: 0.2,
        g: 0.6,
        b: 1.0,
        a: 1.0,
    };
    let bg = Rgba {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    // Simulate syntax highlighted code
    let line = StyledLineBuilder::new()
        .fg(fg_keyword)
        .bg(bg)
        .attr(Attr::BOLD)
        .text("fn")
        .fg(fg_normal)
        .bg(bg)
        .attr(Attr::empty())
        .text(" main() {")
        .build();

    let ops = styled_line_to_render_ops(&line, 4, 10);
    assert_snapshot("styled_code_line", &ops);

    // Verify styled text operations
    assert!(ops
        .ops()
        .iter()
        .any(|op| matches!(op, RenderOp::MoveTo { x: 4, y: 10 })));
    assert!(ops
        .ops()
        .iter()
        .any(|op| matches!(op, RenderOp::SetAttributes(Attr::BOLD))));
    assert!(ops
        .ops()
        .iter()
        .any(|op| matches!(op, RenderOp::PrintRun(s) if s == "fn")));
}

#[test]
fn snapshot_clear_operations() {
    let mut builder = RenderOpsBuilder::new();
    let bg = Rgba {
        r: 0.1,
        g: 0.1,
        b: 0.3,
        a: 1.0,
    };

    builder
        .clear_screen()
        .clear_area(10, 5, 20, 10, Some(bg))
        .move_to(0, 0);

    let ops = builder.build();
    assert_snapshot("clear_operations", &ops);

    assert!(ops
        .ops()
        .iter()
        .any(|op| matches!(op, RenderOp::ClearScreen)));
    assert!(ops.ops().iter().any(|op| matches!(
        op,
        RenderOp::ClearArea {
            x: 10,
            y: 5,
            width: 20,
            height: 10,
            ..
        }
    )));
}

#[test]
fn snapshot_cursor_operations() {
    // Use a RenderOps directly for cursor operations
    let mut ops = RenderOps::new();
    ops.push(RenderOp::SetCursorVisible(false));
    ops.push(RenderOp::SaveCursorPosition);
    ops.push(RenderOp::MoveTo { x: 10, y: 10 });
    ops.push(RenderOp::PrintRun("Temporary text".to_string()));
    ops.push(RenderOp::RestoreCursorPosition);
    ops.push(RenderOp::SetCursorVisible(true));

    assert_snapshot("cursor_operations", &ops);

    assert!(ops
        .ops()
        .iter()
        .any(|op| matches!(op, RenderOp::SetCursorVisible(false))));
    assert!(ops
        .ops()
        .iter()
        .any(|op| matches!(op, RenderOp::SaveCursorPosition)));
    assert!(ops
        .ops()
        .iter()
        .any(|op| matches!(op, RenderOp::RestoreCursorPosition)));
}

#[test]
fn snapshot_synchronized_output() {
    // Create ops directly for synchronized output
    let mut ops = RenderOps::new();

    // Enable synchronized output for flicker-free rendering
    ops.push(RenderOp::SetSynchronizedOutput(true));
    ops.push(RenderOp::MoveTo { x: 0, y: 0 });
    ops.push(RenderOp::PrintRun("Frame content here".to_string()));
    ops.push(RenderOp::MoveTo { x: 0, y: 1 });
    ops.push(RenderOp::PrintRun("More content".to_string()));
    ops.push(RenderOp::SetSynchronizedOutput(false));

    assert_snapshot("synchronized_output", &ops);

    // Verify sync operations are at start and end
    assert!(matches!(
        ops.ops().first(),
        Some(RenderOp::SetSynchronizedOutput(true))
    ));
    assert!(matches!(
        ops.ops().last(),
        Some(RenderOp::SetSynchronizedOutput(false))
    ));
}
