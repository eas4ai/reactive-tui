//! charts-goldens mechanism: CHT-012, CHT-013, CHT-023, CHT-024, CHT-025,
//! CHT-026, CHT-027 and BAR-004.
//!
//! Goldens live in `tests/snapshots/charts/<type>_<class>.ansi` and hold the
//! text grid followed by a digest of every cell's colors, so a diff is
//! readable. Run with `REGENERATE=1` to refresh after an intentional
//! rendering change, review the diff, then commit.

mod common;

use std::hash::{Hash, Hasher};

use common::app_input::{self, Snapshot};
use reactive_tui::app::RootComponent;
use reactive_tui::component::Element;
use reactive_tui::event::types::{Event, ResizeEvent};
use reactive_tui::widgets::display::{
    Chart, ChartAxis, ChartLegend, ChartProps, ChartType, DataPoint, DataSeries,
};

struct Root(Element);
impl RootComponent for Root {
    fn render(&self) -> Element {
        self.0.clone()
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

fn series(values: &[f64]) -> DataSeries {
    DataSeries::new(
        "series",
        values
            .iter()
            .enumerate()
            .map(|(i, v)| DataPoint::with_label(*v, format!("p{i}")))
            .collect(),
    )
}

fn props(kind: ChartType, size: (u16, u16), values: &[f64], max: f64) -> ChartProps {
    ChartProps {
        chart_type: kind,
        width: size.0,
        height: size.1,
        series: vec![series(values)],
        x_axis: ChartAxis {
            show_labels: false,
            show_grid: false,
            ..Default::default()
        },
        y_axis: ChartAxis {
            min: Some(0.0),
            max: Some(max),
            show_labels: false,
            show_grid: false,
            ..Default::default()
        },
        legend: ChartLegend {
            visible: false,
            ..Default::default()
        },
        animated: false,
        ..Default::default()
    }
}

fn last(kind: ChartType, size: (u16, u16), values: &[f64], max: f64) -> Snapshot {
    app_input::run(
        Root(Element::typed::<Chart>(props(kind, size, values, max))),
        size,
        vec![(2, None)],
    )
    .pop()
    .unwrap()
}

fn is_braille(c: char) -> bool {
    ('\u{2800}'..='\u{28FF}').contains(&c)
}
fn count(frame: &Snapshot, f: impl Fn(char) -> bool) -> usize {
    frame.text.chars().filter(|c| f(*c)).count()
}
const EIGHTHS: &[char] = &[
    '▁', '▂', '▃', '▄', '▅', '▆', '▇', '▏', '▎', '▍', '▌', '▋', '▊', '▉',
];

fn snapshots_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots/charts")
}

/// Text grid plus a color digest: readable in a diff, sensitive to color changes.
fn golden_bytes(frame: &Snapshot) -> Vec<u8> {
    let (rows, cols) = frame.screen.size();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for r in 0..rows {
        for c in 0..cols {
            if let Some(cell) = frame.screen.cell(r, c) {
                format!("{:?}{:?}", cell.fgcolor(), cell.bgcolor()).hash(&mut hasher);
            }
        }
    }
    format!("{}\ncolors: {:016x}\n", frame.text, hasher.finish()).into_bytes()
}

fn assert_golden(name: &str, frame: &Snapshot) {
    let path = snapshots_dir().join(format!("{name}.ansi"));
    let bytes = golden_bytes(frame);
    if std::env::var("REGENERATE").as_deref() == Ok("1") {
        std::fs::create_dir_all(snapshots_dir()).expect("snapshot dir");
        std::fs::write(&path, &bytes).expect("write golden");
        return;
    }
    let expected = std::fs::read(&path).unwrap_or_else(|_| {
        panic!("missing golden {path:?}; run with REGENERATE=1, review the diff, then commit it")
    });
    assert_eq!(
        String::from_utf8_lossy(&bytes),
        String::from_utf8_lossy(&expected),
        "golden mismatch for {name}"
    );
}

/// CHT-025: a steep diagonal resolves to braille, a fully covered cell is a
/// full block, and a rectangle edge at 3/8 is an eighth block, not a quarter.
#[test]
fn cht_025_diagonal_uses_braille_full_coverage_is_a_block_and_edges_keep_eighths() {
    let line = last(ChartType::Line, (24, 12), &[0.0, 10.0, 0.0, 10.0], 10.0);
    assert!(
        count(&line, is_braille) > 0,
        "a steep diagonal must produce braille cells, got:\n{}",
        line.text
    );
    let bar = last(ChartType::BarVertical, (24, 12), &[10.0, 10.0], 10.0);
    assert!(
        count(&bar, |c| c == '█') > 0,
        "full coverage must be a full block:\n{}",
        bar.text
    );
    assert_eq!(
        count(&bar, is_braille),
        0,
        "fully covered cells must not be braille:\n{}",
        bar.text
    );
    // 3/8 of one cell: with 8 rows spanning 0..8 a value of 0.375 ends at 3/8 of the bottom cell.
    let three_eighths = last(ChartType::BarVertical, (8, 8), &[0.375], 8.0);
    assert!(
        count(&three_eighths, |c| c == '▃') > 0,
        "a rectangle edge at 3/8 must render as the 3/8 block, got:\n{}",
        three_eighths.text
    );
}

/// CHT-012: no detached dashes on a diagonal; scatter markers on their cells.
#[test]
fn cht_012_diagonal_line_has_no_detached_dashes_and_scatter_marks_cells() {
    let line = last(ChartType::Line, (30, 10), &[0.0, 9.0, 1.0, 10.0, 2.0], 10.0);
    let dashes = count(&line, |c| c == '─');
    let braille = count(&line, is_braille);
    assert!(
        braille > dashes,
        "diagonals must be braille strokes, not stepped dashes (dashes {dashes}, braille {braille}):\n{}",
        line.text
    );
    let scatter = last(
        ChartType::Scatter,
        (30, 10),
        &[0.0, 9.0, 1.0, 10.0, 2.0],
        10.0,
    );
    assert_eq!(
        count(&scatter, |c| c == '•' || is_braille(c)),
        5,
        "one marker per point expected:\n{}",
        scatter.text
    );
}

/// CHT-013: a bar tip resolves to an eighth block, so 3.5 differs from 3 and 4,
/// and the large class labels bars with their values.
#[test]
fn cht_013_bar_tip_resolves_to_an_eighth_block_and_large_bars_carry_value_labels() {
    let three = last(ChartType::BarVertical, (12, 10), &[3.0], 8.0);
    let half = last(ChartType::BarVertical, (12, 10), &[3.5], 8.0);
    let four = last(ChartType::BarVertical, (12, 10), &[4.0], 8.0);
    assert_ne!(
        half.text, three.text,
        "3.5 must not render as 3:\n{}",
        half.text
    );
    assert_ne!(
        half.text, four.text,
        "3.5 must not render as 4:\n{}",
        half.text
    );
    assert!(
        count(&half, |c| EIGHTHS.contains(&c)) > 0,
        "tip must be an eighth block:\n{}",
        half.text
    );
    let large = last(ChartType::BarVertical, (200, 40), &[3.0, 7.5, 5.0], 8.0);
    assert!(
        large.text.contains("7.5"),
        "large class must label bars with their values:\n{}",
        &large.text[..large.text.len().min(1500)]
    );
}

/// CHT-024: size classes chosen from the allotted rectangle, switching on resize.
#[test]
fn cht_024_size_classes_follow_the_rectangle_and_switch_on_resize() {
    let full_axes = |size: (u16, u16)| ChartProps {
        x_axis: ChartAxis::default(),
        y_axis: ChartAxis {
            min: Some(0.0),
            max: Some(5.0),
            ..Default::default()
        },
        legend: ChartLegend::default(),
        ..props(ChartType::Line, size, &[1.0, 3.0, 2.0, 5.0], 5.0)
    };
    let axis_glyphs = |c: char| c == '│' || c == '┼' || c == '■' || c == '└';
    let mini = app_input::run(
        Root(Element::typed::<Chart>(full_axes((39, 10)))),
        (39, 10),
        vec![(2, None)],
    )
    .pop()
    .unwrap();
    assert_eq!(
        count(&mini, axis_glyphs),
        0,
        "39 columns is the mini class: no axis or legend:\n{}",
        mini.text
    );
    let large = app_input::run(
        Root(Element::typed::<Chart>(full_axes((200, 40)))),
        (200, 40),
        vec![(2, None)],
    )
    .pop()
    .unwrap();
    assert!(
        count(&large, |c| c == '·' || c == '┈') > 50,
        "200 by 40 is the large class: grid expected:\n{}",
        &large.text[..large.text.len().min(2000)]
    );
    // Resize across the medium/large boundary must switch class.
    let frames = app_input::run(
        Root(Element::typed::<Chart>(full_axes((80, 24)))),
        (80, 24),
        vec![
            (2, Some(Event::Resize(ResizeEvent::new(200, 40)))),
            (4, None),
        ],
    );
    let after = frames.last().unwrap();
    assert!(
        count(after, |c| c == '·' || c == '┈') > 50,
        "after resizing to 200 by 40 the chart must show the large layout:\n{}",
        &after.text[..after.text.len().min(2000)]
    );
}

/// CHT-023 and BAR-004: goldens at mini, medium and large for each delivered type.
#[test]
fn cht_023_goldens_at_three_size_classes() {
    let data = [2.0, 8.0, 5.0, 9.0, 3.0, 7.0];
    for (kind, name) in [
        (ChartType::Line, "line"),
        (ChartType::Area, "area"),
        (ChartType::Scatter, "scatter"),
        (ChartType::BarVertical, "bar"),
    ] {
        for (cls, size) in [
            ("mini", (20u16, 5u16)),
            ("medium", (80, 24)),
            ("large", (600, 160)),
        ] {
            let frame = last(kind.clone(), size, &data, 10.0);
            assert_golden(&format!("{name}_{cls}"), &frame);
        }
    }
}

/// CHT-026 (Observed): empty and NaN input show a message, paint no shapes, never panic.
#[test]
fn cht_026_empty_and_nan_input_show_a_message_and_no_shapes() {
    let size = (40u16, 12u16);
    let mut empty = props(ChartType::Line, size, &[], 10.0);
    empty.series.clear();
    let e = app_input::run(Root(Element::typed::<Chart>(empty)), size, vec![(2, None)])
        .pop()
        .unwrap();
    assert!(
        e.text.to_lowercase().contains("no data"),
        "empty chart must say so:\n{}",
        e.text
    );
    assert_eq!(count(&e, |c| c == '█' || is_braille(c)), 0);
    // Control: a chart with data must not trip the empty-state predicate.
    let full = last(ChartType::BarVertical, size, &[1.0, 2.0, 3.0], 10.0);
    assert!(
        !full.text.to_lowercase().contains("no data") && count(&full, |c| c == '█') > 0,
        "control chart must paint shapes and no empty message:\n{}",
        full.text
    );
    let nan = last(ChartType::BarVertical, size, &[1.0, f64::NAN, 3.0], 10.0);
    assert!(
        !nan.text.trim().is_empty() && count(&nan, |c| c == '█' || is_braille(c)) == 0,
        "NaN input must render a message and no shapes:\n{}",
        nan.text
    );
}

/// CHT-027: 10,000 points cost at most twice what 1,000 cost, because the
/// plot layer decimates to the column count.
#[test]
fn cht_027_ten_thousand_points_cost_at_most_twice_one_thousand() {
    let size = (200u16, 40u16);
    let work = |n: usize| {
        let values: Vec<f64> = (0..n)
            .map(|i| ((i as f64) * 0.01).sin() * 5.0 + 5.0)
            .collect();
        let run = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            app_input::run(
                Root(Element::typed::<Chart>(props(
                    ChartType::Line,
                    size,
                    &values,
                    10.0,
                ))),
                size,
                vec![(3, None)],
            )
        }));
        match run {
            Ok(frames) => frames.iter().map(|f| f.work_ms).fold(0.0, f64::max),
            Err(_) => panic!("{n} points did not paint three frames inside the harness's 3 s deadline: the chart does not decimate to its column count"),
        }
    };
    let small = work(1_000).max(0.5);
    let big = work(10_000);
    assert!(
        big <= small * 2.0,
        "10,000 points took {big:.1} ms per frame against {small:.1} ms for 1,000: no decimation"
    );
}
