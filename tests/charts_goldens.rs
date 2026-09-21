//! charts-goldens mechanism: CHT-012, CHT-013, CHT-023, CHT-024, CHT-025.
//!
//! Goldens live in `tests/snapshots/charts/<type>_<class>.ansi`. Run with
//! `REGENERATE=1` to refresh after an intentional rendering change, review
//! the diff, then commit.

mod common;

use common::app_input::{self, Snapshot};
use reactive_tui::app::RootComponent;
use reactive_tui::component::Element;
use reactive_tui::widgets::display::{
    Chart, ChartAxis, ChartProps, ChartType, DataPoint, DataSeries,
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
        legend: reactive_tui::widgets::display::ChartLegend {
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

fn assert_golden(name: &str, frame: &Snapshot) {
    let path = snapshots_dir().join(format!("{name}.ansi"));
    if std::env::var("REGENERATE").as_deref() == Ok("1") {
        std::fs::create_dir_all(snapshots_dir()).expect("snapshot dir");
        std::fs::write(&path, &frame.output).expect("write golden");
        return;
    }
    let expected = std::fs::read(&path).unwrap_or_else(|_| {
        panic!("missing golden {path:?}; run with REGENERATE=1, review the diff, then commit it")
    });
    assert_eq!(frame.output, expected, "golden mismatch for {name}");
}

/// CHT-025 and CHT-012: a steep diagonal line resolves to braille, never to
/// detached horizontal dashes; a fully covered bar cell is a full block.
#[test]
fn cht_025_diagonal_line_uses_braille_and_full_coverage_is_a_full_block() {
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
}

/// CHT-012: no detached dashes on a diagonal; curve styles differ.
#[test]
fn cht_012_diagonal_line_has_no_detached_dashes() {
    let line = last(ChartType::Line, (30, 10), &[0.0, 9.0, 1.0, 10.0, 2.0], 10.0);
    let dashes = count(&line, |c| c == '─');
    let braille = count(&line, is_braille);
    assert!(
        braille > dashes,
        "diagonals must be braille strokes, not stepped dashes (dashes {dashes}, braille {braille}):\n{}",
        line.text
    );
}

/// CHT-013: a bar tip resolves to an eighth block, so 3.5 differs from 3 and 4.
#[test]
fn cht_013_bar_tip_resolves_to_an_eighth_block() {
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
}

/// CHT-024: size classes chosen from the allotted rectangle.
#[test]
fn cht_024_mini_rectangle_draws_values_only_and_large_draws_grid_and_labels() {
    let mut p = props(ChartType::Line, (20, 5), &[1.0, 3.0, 2.0, 5.0], 5.0);
    p.x_axis = ChartAxis::default();
    p.y_axis = ChartAxis {
        min: Some(0.0),
        max: Some(5.0),
        ..Default::default()
    };
    p.legend = reactive_tui::widgets::display::ChartLegend::default();
    let mini = app_input::run(Root(Element::typed::<Chart>(p)), (20, 5), vec![(2, None)])
        .pop()
        .unwrap();
    assert_eq!(
        count(&mini, |c| c == '│' || c == '┼' || c == '■'),
        0,
        "mini class must draw no axis or legend:\n{}",
        mini.text
    );
    let large = last(ChartType::Line, (600, 160), &[1.0, 3.0, 2.0, 5.0], 5.0);
    assert!(
        count(&large, |c| c == '·' || c == '┈') > 50,
        "large class must draw a grid:\n{}",
        &large.text[..large.text.len().min(2000)]
    );
}

/// CHT-023 and BAR-004: goldens at mini, medium and large for each type.
#[test]
fn cht_023_goldens_at_three_size_classes() {
    let data = [2.0, 8.0, 5.0, 9.0, 3.0, 7.0];
    for (kind, name) in [
        (ChartType::Line, "line"),
        (ChartType::Area, "area"),
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
