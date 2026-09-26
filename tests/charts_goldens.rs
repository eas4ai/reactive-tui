//! charts-goldens mechanism: CHT-012, CHT-013, CHT-014, CHT-015, CHT-016,
//! CHT-023, CHT-024, CHT-025, CHT-026, CHT-027, CHT-028, CHT-030 and BAR-004.
//!
//! Goldens live in `tests/snapshots/charts/<type>_<class>.ansi` and hold the
//! text grid followed by a digest of every cell's colors, so a diff is
//! readable. Run with `REGENERATE=1` to refresh after an intentional
//! rendering change, review the diff, then commit.

mod common;

use common::app_input::{self, Snapshot};
use reactive_tui::app::RootComponent;
use reactive_tui::component::Element;
use reactive_tui::event::types::{Event, ResizeEvent};
use reactive_tui::widgets::display::{
    Chart, ChartAxis, ChartLegend, ChartProps, ChartType, DataPoint, DataSeries, LegendPosition,
    SankeyChartBuilder, SankeyLabel, SankeyLink, SizeClass,
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

/// A candlestick series whose closes are `values`, each candle opening at
/// the previous close with a wick one unit past the body on both sides.
fn candles(values: &[f64]) -> DataSeries {
    let mut previous = values.first().copied().unwrap_or(0.0);
    DataSeries::new(
        "candles",
        values
            .iter()
            .enumerate()
            .map(|(i, close)| {
                let open = previous;
                previous = *close;
                let (low, high) = (open.min(*close) - 1.0, open.max(*close) + 1.0);
                let mut point = DataPoint::candle(open, high, low, *close);
                point.label = Some(format!("p{i}"));
                point
            })
            .collect(),
    )
}

fn last(kind: ChartType, size: (u16, u16), values: &[f64], max: f64) -> Snapshot {
    app_input::run_when_painted(
        Root(Element::typed::<Chart>(props(kind, size, values, max))),
        size,
        2,
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

/// `tests/snapshots/charts`, or `charts` under `REACTIVE_TUI_SNAPSHOTS`: the
/// charts-goldens check points that at a copy with one golden changed, and
/// this test must then fail.
fn snapshots_dir() -> std::path::PathBuf {
    std::env::var_os("REACTIVE_TUI_SNAPSHOTS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots")
        })
        .join("charts")
}

/// Text grid plus a color digest: readable in a diff, sensitive to color changes.
fn golden_bytes(frame: &Snapshot) -> Vec<u8> {
    let (rows, cols) = frame.screen.size();
    let mut hasher = common::digest::Digest::default();
    for r in 0..rows {
        for c in 0..cols {
            if let Some(cell) = frame.screen.cell(r, c) {
                hasher.field(format!("{:?}{:?}", cell.fgcolor(), cell.bgcolor()).as_bytes());
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

/// CHT-012: a stacked area's upper series starts at the lower series' top at
/// every column, decimated (1000 points in 80 columns) or not.
#[test]
fn cht_012_stacked_area_stays_on_the_lower_series_under_decimation() {
    let stacked = |n: usize| {
        let lower: Vec<f64> = (0..n)
            .map(|i| match i % 5 {
                1 => 4.0,
                2 => 6.0,
                _ => 5.0,
            })
            .collect();
        let upper = vec![3.0; n];
        let mut p = props(ChartType::Area, (80, 24), &lower, 10.0);
        p.series.push(series(&upper));
        p.stacked = true;
        app_input::run_when_painted(Root(Element::typed::<Chart>(p)), (80, 24), 2)
            .pop()
            .unwrap()
    };
    let painted_columns = |frame: &Snapshot, row: usize| {
        frame.text.lines().nth(row).map_or(0, |line| {
            line.chars().filter(|c| !c.is_whitespace()).count()
        })
    };
    let plain = stacked(100);
    let decimated = stacked(1000);
    // Row 8 lies inside the upper series' fill (values 7 to 9 of 10 over 24 rows).
    let (a, b) = (painted_columns(&plain, 8), painted_columns(&decimated, 8));
    assert_eq!(
        a, 80,
        "the undecimated stacked area must fill every column of row 8:\n{}",
        plain.text
    );
    assert_eq!(
        b, a,
        "the decimated stacked area must fill the same columns as the undecimated one:\n{}",
        decimated.text
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

/// CHT-013: the tallest bar keeps its value label when the automatic axis puts
/// its tip on the plot's top row.
#[test]
fn cht_013_tallest_bar_on_the_top_row_keeps_its_value_label() {
    let mut p = props(ChartType::BarVertical, (200, 40), &[3.0, 7.5, 5.0], 8.0);
    p.y_axis.max = None;
    let large = app_input::run_when_painted(Root(Element::typed::<Chart>(p)), (200, 40), 2)
        .pop()
        .unwrap();
    for label in ["3", "7.5", "5"] {
        assert!(
            large.text.contains(label),
            "large class must label every bar, {label} missing:\n{}",
            &large.text[..large.text.len().min(1500)]
        );
    }
}

/// CHT-013, CHT-025: every bar tip is a block or an eighth block, never a
/// braille cap, whatever fraction of a cell the value lands on.
#[test]
fn cht_013_every_bar_tip_is_an_eighth_block_not_a_braille_cap() {
    let bars = last(
        ChartType::BarVertical,
        (40, 24),
        &[2.0, 8.0, 5.0, 9.0, 3.0, 7.0, 4.3, 6.7],
        10.0,
    );
    assert_eq!(
        count(&bars, is_braille),
        0,
        "bar tips must resolve to eighth blocks, not braille:\n{}",
        bars.text
    );
    assert!(
        count(&bars, |c| EIGHTHS.contains(&c)) > 0,
        "fractional tips must show an eighth block:\n{}",
        bars.text
    );
}

/// CHT-020, CHT-013: a typed bar builder's `.label(..)` text is what the
/// large class draws beside each bar.
#[test]
fn cht_020_typed_bar_label_accessor_is_drawn() {
    use reactive_tui::widgets::display::charts::typed::BarChartBuilder;
    let rows = vec![("a", 3.0), ("b", 7.0), ("c", 5.0)];
    let mut p = BarChartBuilder::new(rows)
        .band(|r| r.0)
        .value(|r| r.1)
        .label(|r| format!("L{}", r.0))
        .size(200, 40)
        .build();
    p.animated = false;
    let large = app_input::run_when_painted(Root(Element::typed::<Chart>(p)), (200, 40), 2)
        .pop()
        .unwrap();
    for label in ["La", "Lb", "Lc"] {
        assert!(
            large.text.contains(label),
            "the builder label {label} must be drawn:\n{}",
            &large.text[..large.text.len().min(1500)]
        );
    }
}

/// CHT-024: the medium class draws axes and ticks but no grid; the grid is
/// the large class's, and an axis can still turn it off there.
#[test]
fn cht_024_medium_charts_draw_no_grid_and_large_charts_can_turn_theirs_off() {
    let with_grid = |size: (u16, u16), grid: bool| {
        let mut p = props(ChartType::Line, size, &[2.0, 8.0, 5.0, 7.0], 10.0);
        p.x_axis = ChartAxis {
            show_grid: grid,
            ..Default::default()
        };
        p.y_axis = ChartAxis {
            min: Some(0.0),
            max: Some(10.0),
            show_grid: grid,
            ..Default::default()
        };
        app_input::run_when_painted(Root(Element::typed::<Chart>(p)), size, 2)
            .pop()
            .unwrap()
    };
    let grid_dots = |f: &Snapshot| count(f, |c| c == '·' || c == '┈');
    let medium = with_grid((80, 24), true);
    assert_eq!(
        grid_dots(&medium),
        0,
        "80 by 24 is the medium class: no grid:\n{}",
        medium.text
    );
    let large = with_grid((200, 40), true);
    assert!(
        grid_dots(&large) > 50,
        "200 by 40 is the large class: grid expected:\n{}",
        &large.text[..large.text.len().min(2000)]
    );
    let large_off = with_grid((200, 40), false);
    assert_eq!(
        grid_dots(&large_off),
        0,
        "an axis with show_grid false draws no grid at the large class:\n{}",
        &large_off.text[..large_off.text.len().min(2000)]
    );
}

/// CHT-010: pie slices take their angles from the plot layer's linear scale,
/// so two equal values split the circle into mirror halves.
#[test]
fn cht_010_pie_slices_split_the_circle_through_the_linear_scale() {
    let size = (40u16, 20u16);
    let pie = radial_frame(radial_props(ChartType::Pie, size, &[1.0, 1.0]), size);
    let (mut left, mut right) = (0usize, 0usize);
    for (_, c) in cells_showing(&pie, &palette(2)) {
        if c < size.0 / 2 {
            left += 1;
        } else {
            right += 1;
        }
    }
    assert!(
        left > 0 && (left as i64 - right as i64).abs() <= 2,
        "two equal slices must cover mirror halves (left {left}, right {right}):\n{}",
        pie.text
    );
}

/// CHT-024: a radial chart forced to the large class differs from the
/// medium one at the same size: the radar gains its grid and the pie its
/// value labels.
#[test]
fn cht_024_radial_charts_follow_their_forced_size_class() {
    use reactive_tui::widgets::display::SizeClass;
    let size = (80u16, 24u16);
    for kind in [ChartType::Pie, ChartType::Donut, ChartType::Radar] {
        let forced = |class: SizeClass| {
            let mut p = radial_props(kind.clone(), size, &[3.0, 1.0, 2.0, 4.0]);
            p.size_class = Some(class);
            radial_frame(p, size).text
        };
        let (medium, large) = (forced(SizeClass::Medium), forced(SizeClass::Large));
        assert_ne!(
            medium, large,
            "{kind:?}: forcing the large class must change the layout"
        );
        if kind == ChartType::Radar {
            assert!(
                !medium.contains('·'),
                "a medium radar draws no grid:\n{medium}"
            );
            assert!(
                large.contains('·'),
                "a large radar draws its grid:\n{large}"
            );
        }
    }
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
    // Resize across the medium/large boundary must switch class; the chart
    // fills its terminal (size unset) so the resize reaches it.
    let mut filling = full_axes((80, 24));
    filling.width = 0;
    filling.height = 0;
    let frames = app_input::run(
        Root(Element::typed::<Chart>(filling)),
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

/// CHT-023 and BAR-004: goldens at mini, medium and large for each delivered
/// type, rendered on the debug backend.
#[test]
fn cht_023_goldens_at_three_size_classes() {
    let data = [2.0, 8.0, 5.0, 9.0, 3.0, 7.0];
    // Radial fills draw through the image blitter; the goldens fix the tier.
    fix_blitter();
    for (kind, name) in [
        (ChartType::Line, "line"),
        (ChartType::Area, "area"),
        (ChartType::Scatter, "scatter"),
        (ChartType::BarVertical, "bar"),
        (ChartType::Candlestick, "candlestick"),
        (ChartType::Pie, "pie"),
        (ChartType::Donut, "donut"),
        (ChartType::Radar, "radar"),
        (ChartType::Sankey, "sankey"),
    ] {
        for (cls, size) in [
            ("mini", (20u16, 5u16)),
            ("medium", (80, 24)),
            ("large", (600, 160)),
        ] {
            let mut p = props(kind.clone(), size, &data, 10.0);
            if kind == ChartType::Candlestick {
                p.series = vec![candles(&data)];
                p.y_axis.min = Some(0.0);
                p.y_axis.max = Some(11.0);
            }
            if kind == ChartType::Sankey {
                p = sankey_builder(size).build();
            }
            // Goldens render on the debug backend (CHT-023, BAR-004).
            let frame =
                app_input::run_when_painted_on_debug(Root(Element::typed::<Chart>(p)), size, 2)
                    .pop()
                    .unwrap();
            assert!(
                !frame.text.trim().is_empty(),
                "{name} at {cls} must paint something"
            );
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
    // The radial types: no series, an empty series, and a NaN or infinite
    // value each show a message and paint no slice or polygon.
    let mut bad_fill = radial_props(ChartType::Radar, size, &[1.0, 2.0, 3.0]);
    bad_fill.radial.fills = vec![Some("not-a-color".into())];
    let bad_fill = radial_frame(bad_fill, size);
    assert!(
        bad_fill.text.to_lowercase().contains("fill")
            && cells_showing(&bad_fill, &palette(5)).is_empty(),
        "an invalid radar fill token must show a message, not fall back:\n{}",
        bad_fill.text
    );
    for kind in [ChartType::Pie, ChartType::Donut, ChartType::Radar] {
        let mut none = radial_props(kind.clone(), size, &[]);
        none.series.clear();
        for (case, p) in [
            ("no series", none),
            ("an empty series", radial_props(kind.clone(), size, &[])),
            (
                "a NaN value",
                radial_props(kind.clone(), size, &[1.0, f64::NAN, 3.0]),
            ),
            (
                "an infinite value",
                radial_props(kind.clone(), size, &[1.0, f64::INFINITY]),
            ),
        ] {
            let frame = radial_frame(p, size);
            assert!(
                !frame.text.trim().is_empty() && cells_showing(&frame, &palette(5)).is_empty(),
                "{kind:?} with {case} must show a message and no shapes:\n{}",
                frame.text
            );
        }
        // Values whose total overflows, and non-finite options.
        let mut radius = radial_props(kind.clone(), size, &[1.0, 2.0]);
        radius.radial.outer_radius = f64::NAN;
        for (case, p) in [
            (
                "an overflowing total",
                radial_props(kind.clone(), size, &[1e308, 1e308]),
            ),
            ("a NaN outer radius", radius),
        ] {
            if case == "an overflowing total" && kind == ChartType::Radar {
                continue;
            }
            let frame = radial_frame(p, size);
            assert!(
                frame.text.contains("finite") && cells_showing(&frame, &palette(5)).is_empty(),
                "{kind:?} with {case} must say the values must be finite and draw no shapes:\n{}",
                frame.text
            );
        }
        let drawn = radial_frame(radial_props(kind.clone(), size, &[1.0, 2.0, 3.0]), size);
        assert!(
            !cells_showing(&drawn, &palette(5)).is_empty(),
            "control: {kind:?} with data draws shapes"
        );
    }
}

/// CHT-027: 10,000 points cost at most twice what 1,000 cost, because the
/// plot layer decimates to the column count, and the tooltip on a decimated
/// chart still reports the original index.
#[test]
fn cht_027_ten_thousand_points_cost_at_most_twice_one_thousand() {
    use reactive_tui::event::types::{Event, MouseEvent, MouseEventKind, Position};
    let size = (200u16, 40u16);
    // Hovering the middle column of a 10,000-point chart names a point in
    // the thousands, not a column index under 200.
    let many: Vec<f64> = (0..10_000)
        .map(|i| ((i as f64) * 0.01).sin() * 5.0 + 5.0)
        .collect();
    let hovered = app_input::run(
        Root(Element::typed::<Chart>(props(
            ChartType::Line,
            size,
            &many,
            10.0,
        ))),
        size,
        vec![
            (
                2,
                Some(Event::Mouse(MouseEvent::new(
                    MouseEventKind::Move,
                    Position::cell(100, 20),
                ))),
            ),
            (3, None),
        ],
    )
    .pop()
    .unwrap();
    let label = hovered
        .text
        .split('p')
        .filter_map(|s| {
            s.chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
                .parse::<usize>()
                .ok()
        })
        .max()
        .unwrap_or(0);
    assert!(
        label >= 1_000,
        "tooltip must name the original index, found p{label}:\n{}",
        hovered.text
    );
    if cfg!(debug_assertions) {
        // Comparing 10,000 props per frame costs more than the chart in a
        // debug build; the cost ratio is measured on the optimized build,
        // which is what the charts-goldens mechanism runs for this check.
        eprintln!("SKIP: the decimation cost ratio is measured on the optimized build");
        return;
    }
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
            // The quietest frame after the first is the cost of the chart
            // itself; the maximum would measure whatever else the machine
            // was doing at that moment.
            Ok(frames) => frames
                .iter()
                .skip(1)
                .map(|f| f.work_ms)
                .fold(f64::INFINITY, f64::min),
            Err(_) => panic!(
                "{n} points did not paint three frames inside the harness's {:?} hang guard: the chart does not decimate to its column count",
                app_input::HANG_GUARD
            ),
        }
    };
    let small = work(1_000).max(0.5);
    let big = work(10_000);
    assert!(
        big <= small * 2.0,
        "10,000 points took {big:.1} ms per frame against {small:.1} ms for 1,000: no decimation"
    );
}

/// CHT-014: a candle that closes below its open uses the bearish color, one
/// that closes above uses the bullish color, and the wick spans low to high.
#[test]
fn cht_014_candles_take_theme_colors_by_direction_and_wicks_span_low_to_high() {
    use reactive_tui::layout::colors::parse_color_token;
    let size = (40u16, 12u16);
    let mut p = props(ChartType::Candlestick, size, &[], 10.0);
    // Bullish: opens at 2, closes at 8; bearish: opens at 8, closes at 2.
    p.series = vec![DataSeries::new(
        "candles",
        vec![
            DataPoint::candle(2.0, 9.5, 0.5, 8.0),
            DataPoint::candle(8.0, 9.5, 0.5, 2.0),
        ],
    )];
    p.y_axis.min = Some(0.0);
    p.y_axis.max = Some(10.0);
    let frame = app_input::run(Root(Element::typed::<Chart>(p)), size, vec![(2, None)])
        .pop()
        .unwrap();
    let rgb = |token: &str| {
        let (r, g, b, _) = parse_color_token(token).expect("theme defines the chart colors");
        vt100::Color::Rgb(
            (r * 255.0).round() as u8,
            (g * 255.0).round() as u8,
            (b * 255.0).round() as u8,
        )
    };
    let (bullish, bearish) = (rgb("chart-bullish"), rgb("chart-bearish"));
    assert_ne!(
        bullish, bearish,
        "presets must distinguish bullish from bearish"
    );
    let painted = |col: u16| -> Vec<(u16, vt100::Color)> {
        (0..size.1)
            .filter_map(|r| {
                let cell = frame.screen.cell(r, col)?;
                (!cell.contents().trim().is_empty()).then(|| (r, cell.fgcolor()))
            })
            .collect()
    };
    let column_of = |half: u16| {
        (half..half + size.0 / 2)
            .max_by_key(|c| painted(*c).len())
            .expect("a painted column in each half")
    };
    let (left, right) = (column_of(0), column_of(size.0 / 2));
    let first = painted(left);
    let second = painted(right);
    assert!(
        first.iter().all(|(_, c)| *c == bullish),
        "the candle that closes above its open must use the bullish color:\n{}",
        frame.text
    );
    assert!(
        second.iter().all(|(_, c)| *c == bearish),
        "the candle that closes below its open must use the bearish color:\n{}",
        frame.text
    );
    // The wick from 0.5 to 9.5 of 10 spans at least ten of the twelve rows.
    let span = |cells: &[(u16, vt100::Color)]| {
        cells.iter().map(|(r, _)| *r).max().unwrap_or(0) + 1
            - cells.iter().map(|(r, _)| *r).min().unwrap_or(0)
    };
    assert!(
        span(&first) >= 10 && span(&second) >= 10,
        "wicks must span low to high: {} and {} rows\n{}",
        span(&first),
        span(&second),
        frame.text
    );
}

/// CHT-028: with ASCII forced, the same geometry renders with `#`, `|`, `-`
/// and `.` and never a braille or block glyph.
#[test]
fn cht_028_ascii_fallback_keeps_the_geometry_without_unicode_glyphs() {
    let size = (40u16, 12u16);
    let unicode = last(ChartType::BarVertical, size, &[3.5, 8.0, 5.0], 10.0);
    let mut p = props(ChartType::BarVertical, size, &[3.5, 8.0, 5.0], 10.0);
    p.ascii = true;
    let ascii = app_input::run(Root(Element::typed::<Chart>(p)), size, vec![(2, None)])
        .pop()
        .unwrap();
    let is_block = |c: char| c == '█' || EIGHTHS.contains(&c) || is_braille(c);
    assert!(count(&unicode, is_block) > 0, "control chart paints blocks");
    assert_eq!(
        count(&ascii, is_block),
        0,
        "ASCII mode must not emit braille or block glyphs:\n{}",
        ascii.text
    );
    assert!(
        count(&ascii, |c| c == '#') > 0,
        "ASCII bars use #:\n{}",
        ascii.text
    );
    // Same geometry: every cell painted in unicode is painted in ASCII.
    let painted = |f: &Snapshot| {
        (0..size.1)
            .flat_map(|r| (0..size.0).map(move |c| (r, c)))
            .filter(|(r, c)| {
                f.screen
                    .cell(*r, *c)
                    .is_some_and(|cell| !cell.contents().trim().is_empty())
            })
            .count()
    };
    assert_eq!(
        painted(&unicode),
        painted(&ascii),
        "ASCII keeps the same cells painted"
    );
    let mut line = props(ChartType::Line, size, &[0.0, 9.0, 1.0, 10.0, 2.0], 10.0);
    line.ascii = true;
    let line = app_input::run(Root(Element::typed::<Chart>(line)), size, vec![(2, None)])
        .pop()
        .unwrap();
    assert_eq!(
        count(&line, is_braille),
        0,
        "ASCII lines use no braille:\n{}",
        line.text
    );
    assert!(
        count(&line, |c| c == '.'
            || c == '|'
            || c == '-'
            || c == '#'
            || c == 'o')
            > 0,
        "ASCII lines use the ASCII set:\n{}",
        line.text
    );
    // The radial types keep their geometry in ASCII: no braille, block,
    // sextant or octant glyph, the fill drawn with `#`, and the same cells
    // painted as with unicode glyphs.
    let is_unicode_shape = |c: char| {
        c == '█'
            || EIGHTHS.contains(&c)
            || is_braille(c)
            || ('\u{1FB00}'..='\u{1FB3B}').contains(&c)
            || ('\u{1CD00}'..='\u{1CDE5}').contains(&c)
            || "▀▄▌▐▖▗▘▙▚▛▜▝▞▟".contains(c)
    };
    for kind in [ChartType::Pie, ChartType::Donut, ChartType::Radar] {
        let unicode = radial_frame(
            radial_props(kind.clone(), size, &[3.0, 5.0, 2.0, 4.0]),
            size,
        );
        let mut p = radial_props(kind.clone(), size, &[3.0, 5.0, 2.0, 4.0]);
        p.ascii = true;
        let ascii = radial_frame(p, size);
        assert_eq!(
            count(&ascii, is_unicode_shape),
            0,
            "{kind:?} in ASCII must emit no block, braille, sextant or octant glyph:\n{}",
            ascii.text
        );
        assert!(
            count(&ascii, |c| c == '#') > 0,
            "{kind:?} in ASCII fills with #:\n{}",
            ascii.text
        );
        let shaped = |f: &Snapshot| cells_showing(f, &palette(4)).len();
        assert!(
            shaped(&unicode).abs_diff(shaped(&ascii)) <= shaped(&unicode) / 10 + 2,
            "{kind:?} in ASCII keeps its geometry ({} unicode cells, {} ASCII cells)",
            shaped(&unicode),
            shaped(&ascii)
        );
    }
}

/// Radial charts draw their fills through the image blitter; these tests fix
/// the tier at sextant so the frames do not depend on the host. The
/// `REACTIVE_TUI_BLITTER` override outranks the application's choice, so it
/// is cleared first.
fn fix_blitter() {
    static CLEARED: std::sync::Once = std::sync::Once::new();
    CLEARED.call_once(|| std::env::remove_var("REACTIVE_TUI_BLITTER"));
    reactive_tui::widgets::display::set_image_blitter(Some(
        reactive_tui::widgets::display::Blitter::Sextant,
    ));
}

fn radial_frame(p: ChartProps, size: (u16, u16)) -> Snapshot {
    fix_blitter();
    app_input::run_when_painted_on_debug(Root(Element::typed::<Chart>(p)), size, 2)
        .pop()
        .unwrap()
}

/// A theme token as the terminal color it renders to.
fn theme_color(token: &str) -> (u8, u8, u8) {
    let (r, g, b, _) = reactive_tui::theme::Theme::active()
        .resolve_color(token)
        .expect("theme color");
    let c = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    (c(r), c(g), c(b))
}

/// Whether a terminal color is `rgb`, allowing one rounding step.
fn near(color: vt100::Color, rgb: (u8, u8, u8)) -> bool {
    match color {
        vt100::Color::Rgb(r, g, b) => {
            r.abs_diff(rgb.0) <= 2 && g.abs_diff(rgb.1) <= 2 && b.abs_diff(rgb.2) <= 2
        }
        _ => false,
    }
}

/// Whether the cell at (`r`, `c`) shows `rgb` as a shape: as its background,
/// or as the color of a shape glyph. Label text in a slice's color is not a
/// shape.
fn shows(frame: &Snapshot, r: u16, c: u16, rgb: (u8, u8, u8)) -> bool {
    frame.screen.cell(r, c).is_some_and(|cell| {
        let text = cell.contents();
        let glyph = !text.trim().is_empty() && !text.chars().any(char::is_alphanumeric);
        near(cell.bgcolor(), rgb) || (near(cell.fgcolor(), rgb) && glyph)
    })
}

/// The cells showing any of `colors`, as (row, column).
fn cells_showing(frame: &Snapshot, colors: &[(u8, u8, u8)]) -> Vec<(u16, u16)> {
    let (rows, cols) = frame.screen.size();
    (0..rows)
        .flat_map(|r| (0..cols).map(move |c| (r, c)))
        .filter(|(r, c)| colors.iter().any(|rgb| shows(frame, *r, *c, *rgb)))
        .collect()
}

/// The slice cell, as (row, column), that the leader beside the label on
/// row `r`, columns `c` to `c + len`, reaches through box-drawing glyphs,
/// each joined to the next: the first block or sextant glyph past its end.
/// `None` when the leader breaks before it reaches one.
fn leader_end(grid: &[Vec<char>], r: usize, c: usize, len: usize) -> Option<(u16, u16)> {
    let ends = |g: char| -> &'static [(isize, isize)] {
        match g {
            '─' => &[(-1, 0), (1, 0)],
            '│' => &[(0, -1), (0, 1)],
            '╮' => &[(-1, 0), (0, 1)],
            '╯' => &[(-1, 0), (0, -1)],
            '╭' => &[(1, 0), (0, 1)],
            '╰' => &[(1, 0), (0, -1)],
            _ => &[],
        }
    };
    let glyph = |x: isize, y: isize| -> Option<char> {
        let (x, y) = (usize::try_from(x).ok()?, usize::try_from(y).ok()?);
        grid.get(y).and_then(|l| l.get(x)).copied()
    };
    let slice = |x: isize, y: isize| {
        glyph(x, y).is_some_and(|g| {
            "█▌▐▀▄▖▗▘▝▙▚▛▜▞▟".contains(g) || ('\u{1FB00}'..='\u{1FB3B}').contains(&g)
        })
    };
    // Start beside the label on either side, entering from the label.
    [(c as isize - 1, -1isize), ((c + len) as isize, 1)]
        .into_iter()
        .find_map(|(x0, step)| {
            let (mut x, mut y, mut from) = (x0, r as isize, (-step, 0));
            for _ in 0..400 {
                let g = glyph(x, y)?;
                let ends = ends(g);
                if !ends.contains(&from) {
                    return None;
                }
                let &(dx, dy) = ends.iter().find(|d| **d != from)?;
                if slice(x + dx, y + dy) {
                    return Some((u16::try_from(y + dy).ok()?, u16::try_from(x + dx).ok()?));
                }
                (x, y, from) = (x + dx, y + dy, (-dx, -dy));
            }
            None
        })
}

fn palette(n: usize) -> Vec<(u8, u8, u8)> {
    (1..=n)
        .map(|i| theme_color(&format!("chart-{i}")))
        .collect()
}

fn radial_props(kind: ChartType, size: (u16, u16), values: &[f64]) -> ChartProps {
    let mut p = props(kind, size, values, 10.0);
    p.legend.visible = false;
    p
}

/// CHT-025: a fill-only cell resolves to the chosen blitter's glyph with
/// the two-color split `blit_block` gives for its samples; an uncovered
/// sample leaves the background transparent; a stroke over fill keeps
/// braille; and a pie boundary cell between two slices shows both colors.
#[test]
fn cht_025_fill_only_cells_take_the_blitters_two_color_split() {
    use reactive_tui::widgets::display::charts::mask::{GlyphSet, MaskCanvas};
    use reactive_tui::widgets::display::Blitter;
    let (red, blue) = ((0.8, 0.1, 0.1, 1.0), (0.1, 0.2, 0.9, 1.0));
    let px = |(r, g, b, _): (f32, f32, f32, f32)| {
        let c = |v: f32| (v * 255.0).round() as u8;
        [c(r), c(g), c(b), 255]
    };
    for blitter in [Blitter::Sextant, Blitter::Octant, Blitter::Quadrant] {
        let mut canvas = MaskCanvas::new(3, 1);
        canvas.set_fill_blitter(blitter);
        // Cell 0: left column red, right column blue.
        canvas.fill_polygon(
            &[(0.0, 0.0), (1.0, 0.0), (1.0, 4.0), (0.0, 4.0)],
            Some(red),
            None,
        );
        canvas.fill_polygon(
            &[(1.0, 0.0), (2.0, 0.0), (2.0, 4.0), (1.0, 4.0)],
            Some(blue),
            None,
        );
        // Cell 1: left column red, right column uncovered.
        canvas.fill_polygon(
            &[(2.0, 0.0), (3.0, 0.0), (3.0, 4.0), (2.0, 4.0)],
            Some(red),
            None,
        );
        // Cell 2: filled red, with one stroke dot on top.
        canvas.fill_polygon(
            &[(4.0, 0.0), (6.0, 0.0), (6.0, 4.0), (4.0, 4.0)],
            Some(red),
            None,
        );
        canvas.dot(4, 0, Some(blue), None);
        let (cols, rows) = blitter.cell_pixels();
        let pixels: Vec<[u8; 4]> = (0..cols * rows)
            .map(|i| if i % cols == 0 { px(red) } else { px(blue) })
            .collect();
        let expected = suprtui::blit::blit_block(blitter, &pixels);
        let two = canvas.resolve(0, 0, GlyphSet::Unicode);
        assert_eq!(
            two.glyph.and_then(|g| g.chars().next()),
            Some(expected.glyph),
            "{blitter:?}: a two-color fill cell must take blit_block's glyph"
        );
        let to_u8 = |c: Option<(f32, f32, f32, f32)>| {
            c.map(|c| {
                let p = px(c);
                [p[0], p[1], p[2]]
            })
        };
        assert_eq!(
            (to_u8(two.color), to_u8(two.background)),
            (expected.fg, expected.bg),
            "{blitter:?}: the cell's colors must be blit_block's split"
        );
        let mut shown = [to_u8(two.color), to_u8(two.background)];
        shown.sort();
        let mut wanted = [
            Some([px(red)[0], px(red)[1], px(red)[2]]),
            Some([px(blue)[0], px(blue)[1], px(blue)[2]]),
        ];
        wanted.sort();
        assert_eq!(
            shown, wanted,
            "{blitter:?}: a cell holding two slice colors shows both"
        );
        let half = canvas.resolve(1, 0, GlyphSet::Unicode);
        assert!(
            half.background.is_none(),
            "{blitter:?}: an uncovered sample keeps the background transparent"
        );
        assert_eq!(
            to_u8(half.color),
            Some([px(red)[0], px(red)[1], px(red)[2]])
        );
        let stroked = canvas.resolve(2, 0, GlyphSet::Unicode);
        assert!(
            stroked
                .glyph
                .is_some_and(|g| g.chars().all(|c| ('\u{2800}'..='\u{28FF}').contains(&c))),
            "{blitter:?}: a stroke over fill resolves to braille, got {:?}",
            stroked.glyph
        );
    }
    // The ASCII tier draws one pixel per cell as a space on its color, so a
    // filled cell keeps its background there and stays visible.
    let mut ascii = MaskCanvas::new(1, 1);
    ascii.set_fill_blitter(Blitter::Ascii);
    ascii.fill_polygon(
        &[(0.0, 0.0), (2.0, 0.0), (2.0, 4.0), (0.0, 4.0)],
        Some(red),
        None,
    );
    let cell = ascii.resolve(0, 0, GlyphSet::Unicode);
    assert_eq!(cell.glyph, Some(" "), "the ASCII tier draws a space");
    assert!(
        cell.background.is_some(),
        "an ASCII-tier fill cell must keep its color as the background, or the fill disappears"
    );
    // A pie of two equal slices, centered on an odd width: the column on
    // the vertical boundary shows both slice colors in one cell.
    let size = (81u16, 25u16);
    let frame = radial_frame(radial_props(ChartType::Pie, size, &[1.0, 1.0]), size);
    let colors = palette(2);
    let (rows, _) = frame.screen.size();
    let boundary = (0..rows)
        .filter(|r| shows(&frame, *r, 40, colors[0]) && shows(&frame, *r, 40, colors[1]))
        .count();
    assert!(
        boundary >= 8,
        "the boundary column of a two-slice pie must show both colors in its cells ({boundary} rows):\n{}",
        frame.text
    );
}

/// CHT-015: every sample inside a slice takes that slice's color, a full
/// pie is twice as wide in columns as it is tall in rows, a pad angle
/// leaves a gap, labels sit beside the circle joined by leader lines and
/// never overlap, and a mini pie draws no labels.
#[test]
fn cht_015_pie_slices_aspect_pad_and_labels() {
    // Four equal slices, clockwise from twelve o'clock.
    let size = (81u16, 25u16);
    let frame = radial_frame(
        radial_props(ChartType::Pie, size, &[1.0, 1.0, 1.0, 1.0]),
        size,
    );
    let colors = palette(4);
    let painted = cells_showing(&frame, &colors);
    let (top, bottom) = (
        painted.iter().map(|p| p.0).min().unwrap(),
        painted.iter().map(|p| p.0).max().unwrap(),
    );
    let (left, right) = (
        painted.iter().map(|p| p.1).min().unwrap(),
        painted.iter().map(|p| p.1).max().unwrap(),
    );
    let (height, width) = (i32::from(bottom - top) + 1, i32::from(right - left) + 1);
    assert!(
        (width - 2 * height).abs() <= 1,
        "a full pie must be twice as wide as tall: {width} columns by {height} rows\n{}",
        frame.text
    );
    let (cr, cc) = ((top + bottom) / 2, (left + right) / 2);
    let quarter = (height / 4) as u16;
    for (k, (r, c)) in [
        (cr - quarter, cc + 2 * quarter),
        (cr + quarter, cc + 2 * quarter),
        (cr + quarter, cc - 2 * quarter),
        (cr - quarter, cc - 2 * quarter),
    ]
    .into_iter()
    .enumerate()
    {
        assert!(
            shows(&frame, r, c, colors[k]) && !colors.iter().enumerate().any(|(j, rgb)| j != k && shows(&frame, r, c, *rgb)),
            "slice {k} (clockwise from twelve) must paint its interior cell ({r}, {c}) in its own color alone\n{}",
            frame.text
        );
    }
    // A pad angle leaves an unpainted gap on the boundary above the center.
    let two = palette(2);
    let size = (81u16, 25u16);
    let joined = radial_frame(radial_props(ChartType::Pie, size, &[1.0, 1.0]), size);
    let mut padded = radial_props(ChartType::Pie, size, &[1.0, 1.0]);
    padded.radial.pad_angle = 0.4;
    let padded = radial_frame(padded, size);
    let above: Vec<u16> = (3..10).collect();
    assert!(
        above
            .iter()
            .all(|r| shows(&joined, *r, 40, two[0]) || shows(&joined, *r, 40, two[1])),
        "control: without a pad angle the boundary above the center is painted\n{}",
        joined.text
    );
    assert!(
        above
            .iter()
            .all(|r| !shows(&padded, *r, 40, two[0]) && !shows(&padded, *r, 40, two[1])),
        "a pad angle must leave an unpainted gap between the slices\n{}",
        padded.text
    );
    // Labels beside the circle, each joined by a leader, none overlapping.
    let names = [
        "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta",
    ];
    let size = (80u16, 24u16);
    let mut labelled = radial_props(
        ChartType::Pie,
        size,
        &[3.0, 1.0, 2.0, 1.0, 4.0, 1.0, 2.0, 1.0],
    );
    for (point, name) in labelled.series[0].data.iter_mut().zip(names) {
        point.label = Some(name.to_string());
    }
    let frame = radial_frame(labelled, size);
    let lines: Vec<String> = frame.text.lines().map(str::to_string).collect();
    let mut placed = 0;
    for name in names {
        let found: Vec<(usize, usize)> = lines
            .iter()
            .enumerate()
            .flat_map(|(r, line)| {
                let chars: Vec<char> = line.chars().collect();
                let target: Vec<char> = name.chars().collect();
                (0..chars.len().saturating_sub(target.len() - 1))
                    .filter(move |c| {
                        chars[*c..*c + target.len()] == target[..]
                            && !chars
                                .get(c + target.len())
                                .is_some_and(|n| n.is_alphabetic())
                            && !(*c > 0 && chars[c - 1].is_alphabetic())
                    })
                    .map(move |c| (r, c))
                    .collect::<Vec<_>>()
            })
            .collect();
        assert!(
            found.len() <= 1,
            "label {name} drawn more than once:\n{}",
            frame.text
        );
        let Some((r, c)) = found.first().copied() else {
            continue;
        };
        placed += 1;
        let chars: Vec<char> = lines[r].chars().collect();
        let before = c.checked_sub(1).map(|i| chars[i]);
        let after = chars.get(c + name.chars().count()).copied();
        assert!(
            [before, after]
                .iter()
                .any(|g| g.is_some_and(|g| "─╮╯╭╰│".contains(g))),
            "label {name} must be joined to its slice by a leader line:\n{}",
            frame.text
        );
    }
    assert!(
        placed >= 4,
        "most labels must be placed ({placed} of 8):\n{}",
        frame.text
    );
    // Leaders stay off the slices: every cell a slice paints without labels
    // still shows a slice color with them, and each leader touches a slice.
    // Empty labels with a wider gap keep the circle the same size.
    let mut bare = radial_props(
        ChartType::Pie,
        size,
        &[3.0, 1.0, 2.0, 1.0, 4.0, 1.0, 2.0, 1.0],
    );
    for point in &mut bare.series[0].data {
        point.label = Some(String::new());
    }
    bare.radial.label_gap = 2 + "epsilon".len() as u16;
    let bare = radial_frame(bare, size);
    let colors = palette(5);
    let covered: Vec<(u16, u16)> = cells_showing(&bare, &colors)
        .into_iter()
        .filter(|(r, c)| !colors.iter().any(|rgb| shows(&frame, *r, *c, *rgb)))
        .collect();
    assert!(
        covered.is_empty(),
        "leader lines must not draw over slice cells: {covered:?}\n{}",
        frame.text
    );
    let glyph_at = |r: usize, c: usize| lines.get(r).and_then(|l| l.chars().nth(c));
    let touching = (0..lines.len())
        .flat_map(|r| (0..lines[r].chars().count()).map(move |c| (r, c)))
        .filter(|(r, c)| glyph_at(*r, *c).is_some_and(|g| "─╮╯╭╰".contains(g)))
        .filter(|(r, c)| {
            [c.wrapping_sub(1), c + 1].iter().any(|n| {
                colors
                    .iter()
                    .any(|rgb| shows(&frame, *r as u16, *n as u16, *rgb))
            })
        })
        .count();
    assert!(
        touching >= placed,
        "each placed label's leader must reach its slice ({touching} of {placed}):\n{}",
        frame.text
    );
    // At a narrow gap, where a leader has no room to bend outside the
    // circle, leaders still stay off the slices and off their own labels.
    let values = [2.0, 8.0, 5.0, 9.0, 3.0, 7.0];
    // Six slices take the five palette colors, the last wrapping around.
    let six = palette(5);
    for gap in [0u16, 1] {
        let mut narrow = radial_props(ChartType::Pie, size, &values);
        narrow.radial.label_gap = gap;
        let narrow = radial_frame(narrow, size);
        let mut bare = radial_props(ChartType::Pie, size, &values);
        for point in &mut bare.series[0].data {
            point.label = Some(String::new());
        }
        bare.radial.label_gap = gap + "p0".len() as u16;
        let bare = radial_frame(bare, size);
        let covered: Vec<(u16, u16)> = cells_showing(&bare, &six)
            .into_iter()
            .filter(|(r, c)| !six.iter().any(|rgb| shows(&narrow, *r, *c, *rgb)))
            .collect();
        assert!(
            covered.is_empty(),
            "at label gap {gap} leader lines must not draw over slice cells: {covered:?}\n{}",
            narrow.text
        );
        let cut: Vec<String> = narrow
            .text
            .lines()
            .filter(|line| {
                let chars: Vec<char> = line.chars().collect();
                (0..chars.len())
                    .any(|i| chars[i] == 'p' && !chars.get(i + 1).is_some_and(char::is_ascii_digit))
            })
            .map(str::to_string)
            .collect();
        assert!(
            cut.is_empty(),
            "at label gap {gap} a leader must not overwrite its label: {cut:?}\n{}",
            narrow.text
        );
        assert!(
            (0..values.len())
                .filter(|i| narrow.text.contains(&format!("p{i}")))
                .count()
                >= 3,
            "at label gap {gap} labels are still placed:\n{}",
            narrow.text
        );
    }
    // Every placed label's leader runs unbroken to its own slice, on a
    // crowded side, at the medium and large classes and at every gap: no
    // leader breaks another, no label sits against its slice without one,
    // and no leader ends on a neighbouring slice. Each slice takes its own
    // color, so the cell at a leader's end names the slice it reaches.
    let crowded = [1.0, 1.0, 1.0, 1.0, 1.0, 20.0, 1.0, 1.0];
    let crowded_left = [20.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
    let crowded_bottom = [10.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 10.0];
    let spread = [3.0, 1.0, 2.0, 1.0, 4.0, 1.0, 2.0, 1.0];
    let own = [
        "#e6194b", "#3cb44b", "#ffe119", "#4363d8", "#f58231", "#911eb4", "#46f0f0", "#f032e6",
    ];
    for (kind, values) in [
        (ChartType::Pie, &crowded[..]),
        (ChartType::Donut, &crowded[..]),
        (ChartType::Pie, &crowded_left[..]),
        (ChartType::Donut, &crowded_bottom[..]),
        (ChartType::Pie, &values[..]),
        (ChartType::Pie, &spread[..]),
    ] {
        for (size, large) in [
            ((80u16, 24u16), false),
            ((120, 30), false),
            ((200, 40), true),
        ] {
            for gap in 0u16..=3 {
                let mut chart = radial_props(kind.clone(), size, values);
                chart.radial.label_gap = gap;
                for (point, color) in chart.series[0].data.iter_mut().zip(own) {
                    point.color = Some(color.to_string());
                }
                let frame = radial_frame(chart, size);
                let grid: Vec<Vec<char>> =
                    frame.text.lines().map(|l| l.chars().collect()).collect();
                let mut placed = 0;
                for (i, value) in values.iter().enumerate() {
                    // The large class shows the value after the name.
                    let label = if large {
                        format!("p{i} {value}")
                    } else {
                        format!("p{i}")
                    };
                    let name: Vec<char> = label.chars().collect();
                    for (r, line) in grid.iter().enumerate() {
                        for c in 0..line.len().saturating_sub(name.len() - 1) {
                            if line[c..c + name.len()] != name[..]
                                || line.get(c + name.len()).is_some_and(char::is_ascii_digit)
                            {
                                continue;
                            }
                            placed += 1;
                            let Some((er, ec)) = leader_end(&grid, r, c, name.len()) else {
                                panic!(
                                    "{kind:?} {size:?} at label gap {gap}: the leader of p{i} must run unbroken to a slice:\n{}",
                                    frame.text
                                );
                            };
                            assert!(
                                shows(&frame, er, ec, theme_color(own[i])),
                                "{kind:?} {size:?} at label gap {gap}: the leader of p{i} ends at ({er}, {ec}), which does not show p{i}'s own slice:\n{}",
                                frame.text
                            );
                        }
                    }
                }
                assert!(
                    placed >= 3,
                    "{kind:?} {size:?} at label gap {gap}: labels are still placed ({placed}):\n{}",
                    frame.text
                );
            }
        }
    }
    // A label too long for the room beside the circle is cut, not dropped.
    let mut long = radial_props(ChartType::Pie, size, &[3.0, 1.0, 2.0]);
    long.legend.visible = true;
    for point in &mut long.series[0].data {
        point.label = Some(format!(
            "{}-a-rather-long-name",
            point.label.clone().unwrap()
        ));
    }
    let long = radial_frame(long, size);
    assert!(
        long.text.contains('…'),
        "a long label must be cut to fit, not dropped:\n{}",
        long.text
    );
    // The large class shows each label in full with its value.
    let size = (200u16, 40u16);
    let large = radial_frame(radial_props(ChartType::Pie, size, &[3.0, 1.0, 2.0]), size);
    for label in ["p0 3", "p1 1", "p2 2"] {
        assert!(
            large.text.contains(label),
            "a large pie labels each slice with its value ({label}):\n{}",
            large.text
        );
    }
    // A mini pie draws shapes only.
    let size = (20u16, 5u16);
    let mut mini = radial_props(ChartType::Pie, size, &[1.0, 2.0]);
    mini.series[0].data[0].label = Some("alpha".into());
    let mini = radial_frame(mini, size);
    assert!(
        !mini.text.contains("alpha"),
        "a mini pie draws no labels:\n{}",
        mini.text
    );
    assert!(
        !cells_showing(&mini, &palette(2)).is_empty(),
        "a mini pie still draws its slices"
    );
}

/// CHT-015: a label left out keeps its slice in the legend, also when the
/// legend has no room for every slice: each slice is named by its label on
/// the chart or by its legend entry. Seven long names do not fit one legend
/// row, and the thin last slice, next to twelve o'clock, loses its label.
#[test]
fn cht_015_a_left_out_label_keeps_its_slice_in_the_legend() {
    let names = [
        "amber-region",
        "birch-region",
        "cedar-region",
        "delta-region",
        "ember-region",
        "frost-region",
        "grove-region",
    ];
    let values = [30.0, 20.0, 15.0, 12.0, 10.0, 10.0, 3.0];
    let mut full = 0;
    for (kind, size, position) in [
        (ChartType::Pie, (80u16, 24u16), LegendPosition::Top),
        (ChartType::Pie, (80, 24), LegendPosition::Bottom),
        (ChartType::Donut, (80, 24), LegendPosition::Top),
        (ChartType::Pie, (100, 30), LegendPosition::Bottom),
        (ChartType::Pie, (120, 30), LegendPosition::Bottom),
    ] {
        let mut chart = radial_props(kind.clone(), size, &values);
        chart.legend.visible = true;
        chart.legend.position = position.clone();
        for (point, name) in chart.series[0].data.iter_mut().zip(names) {
            point.label = Some(name.to_string());
        }
        let frame = radial_frame(chart, size);
        let lines: Vec<&str> = frame.text.lines().collect();
        let legend_row = if position == LegendPosition::Top {
            lines[0]
        } else {
            lines[lines.len() - 1]
        };
        let face: Vec<&str> = if position == LegendPosition::Top {
            lines[1..].to_vec()
        } else {
            lines[..lines.len() - 1].to_vec()
        };
        let in_legend = |name: &str| legend_row.contains(name);
        let on_face = |name: &str| face.iter().any(|line| line.contains(name));
        // The case the fix is for: the legend is full and a label is out.
        if names.iter().any(|n| !in_legend(n)) && names.iter().any(|n| !on_face(n)) {
            full += 1;
        }
        let unnamed: Vec<&str> = names
            .iter()
            .copied()
            .filter(|n| !in_legend(n) && !on_face(n))
            .collect();
        assert!(
            unnamed.is_empty(),
            "{kind:?} {size:?} {position:?}: every slice must be named on the chart or in the legend; named nowhere: {unnamed:?}\n{}",
            frame.text
        );
    }
    assert!(
        full >= 2,
        "control: at least two charts must have a full legend and a label left out ({full})"
    );
}

/// CHT-016: each vertex lies on its spoke at its value's distance, the grid
/// shows the configured number of levels, and a later series lies over an
/// earlier one.
#[test]
fn cht_016_radar_vertices_grid_levels_and_series_order() {
    let size = (81u16, 25u16);
    let mut p = radial_props(ChartType::Radar, size, &[8.0, 4.0, 8.0, 4.0]);
    p.dots = true;
    p.radial.grid = false;
    p.radial.fills = vec![Some("none".into())];
    let frame = radial_frame(p, size);
    let lines: Vec<Vec<char>> = frame.text.lines().map(|l| l.chars().collect()).collect();
    let markers: Vec<(i32, i32)> = lines
        .iter()
        .enumerate()
        .flat_map(|(r, l)| {
            l.iter()
                .enumerate()
                .filter(|(_, c)| **c == '•')
                .map(move |(c, _)| (r as i32, c as i32))
        })
        .collect();
    assert_eq!(
        markers.len(),
        4,
        "one vertex dot per category:\n{}",
        frame.text
    );
    let top = *markers.iter().min_by_key(|m| m.0).unwrap();
    let bottom = *markers.iter().max_by_key(|m| m.0).unwrap();
    let left = *markers.iter().min_by_key(|m| m.1).unwrap();
    let right = *markers.iter().max_by_key(|m| m.1).unwrap();
    let center = ((top.0 + bottom.0) / 2, top.1);
    assert!(
        (top.1 - bottom.1).abs() <= 1 && (left.0 - center.0).abs() <= 1 && (right.0 - center.0).abs() <= 1,
        "vertices must lie on their spokes: top {top:?} right {right:?} bottom {bottom:?} left {left:?}\n{}",
        frame.text
    );
    // In dots a row is four and a column two: the value-8 vertex is twice
    // as far out as the value-4 one.
    let (up, across) = ((center.0 - top.0) * 4, (right.1 - center.1) * 2);
    assert!(
        (up - 2 * across).abs() <= 4,
        "vertex distances must follow the values (8 at {up} dots, 4 at {across} dots)\n{}",
        frame.text
    );
    // Grid levels, drawn at the large class: rings crossed walking right
    // along the row two above the center, which misses the spokes.
    let rings = |levels: usize| {
        let size = (201u16, 41u16);
        let mut p = radial_props(ChartType::Radar, size, &[0.01, 0.01, 0.01, 0.01]);
        p.radial.max_value = Some(10.0);
        p.radial.grid_levels = levels;
        let grid = radial_frame(p, size);
        let lines: Vec<Vec<char>> = grid.text.lines().map(|l| l.chars().collect()).collect();
        let row = &lines[(size.1 / 2 - 2) as usize];
        let (mut rings, mut on) = (0, false);
        for glyph in &row[(size.0 / 2 + 1) as usize..] {
            let dot = *glyph == '·';
            if dot && !on {
                rings += 1;
            }
            on = dot;
        }
        (rings, grid.text)
    };
    let (three, text) = rings(3);
    assert_eq!(three, 3, "the grid must show three levels:\n{text}");
    let (none, text) = rings(0);
    assert_eq!(none, 0, "zero grid levels draw no level polygon:\n{text}");
    // An unfilled later series lies over nothing: the earlier outline stays.
    let mut p = radial_props(ChartType::Radar, size, &[4.0, 4.0, 4.0, 4.0]);
    let mut outer = series(&[8.0, 8.0, 8.0, 8.0]);
    outer.name = "outer".into();
    p.series.push(outer);
    p.radial.fills = vec![Some("none".into()), Some("none".into())];
    let unfilled = radial_frame(p, size);
    assert!(
        !cells_showing(&unfilled, &palette(1)).is_empty(),
        "an unfilled later series must leave the earlier outline:\n{}",
        unfilled.text
    );
    // A later, larger series covers an earlier one: none of the earlier
    // series' color shows.
    let mut p = radial_props(ChartType::Radar, size, &[4.0, 4.0, 4.0, 4.0]);
    let mut outer = series(&[8.0, 8.0, 8.0, 8.0]);
    outer.name = "outer".into();
    p.series.push(outer);
    let layered = radial_frame(p, size);
    let colors = palette(2);
    assert!(
        cells_showing(&layered, &colors[..1]).is_empty(),
        "the earlier series must lie under the later one:\n{}",
        layered.text
    );
    assert!(
        !cells_showing(&layered, &colors[1..]).is_empty(),
        "the later series draws"
    );
}

/// Energy flows for the Sankey tests and goldens: three sources, a middle
/// node and three sinks, with one link that skips the middle column.
const SANKEY_NODES: [&str; 7] = ["coal", "gas", "solar", "power", "industry", "homes", "loss"];
const SANKEY_LINKS: [(usize, usize, f64); 7] = [
    (0, 3, 4.0),
    (1, 3, 3.0),
    (2, 3, 2.0),
    (1, 4, 2.0),
    (3, 4, 3.0),
    (3, 5, 5.0),
    (3, 6, 1.0),
];

fn sankey_links(links: &[(usize, usize, f64)]) -> Vec<SankeyLink> {
    links
        .iter()
        .map(|(s, t, v)| SankeyLink::new(*s, *t, *v))
        .collect()
}

fn sankey_builder(size: (u16, u16)) -> SankeyChartBuilder<&'static str> {
    SankeyChartBuilder::new(SANKEY_NODES, sankey_links(&SANKEY_LINKS))
        .node_label(|n: &&str| *n)
        .size(size.0, size.1)
}

fn sankey_frame(p: ChartProps, size: (u16, u16)) -> Snapshot {
    fix_blitter();
    app_input::run_when_painted_on_debug(Root(Element::typed::<Chart>(p)), size, 2)
        .pop()
        .unwrap()
}

/// Whether a cell shows a filled shape: a block or sextant glyph.
fn is_fill_glyph(c: char) -> bool {
    "█▌▐▀▄▖▗▘▝▙▚▛▜▞▟".contains(c) || ('\u{1FB00}'..='\u{1FB3B}').contains(&c)
}

/// A node's palette color: the palette in node order.
fn node_rgb(index: usize) -> (u8, u8, u8) {
    theme_color(&format!("chart-{}", index % 5 + 1))
}

/// `rgb` blended toward the background: `opacity` of the color.
fn blended(rgb: (u8, u8, u8), opacity: f32) -> (u8, u8, u8) {
    let bg = theme_color("background");
    let mix = |c: u8, b: u8| {
        let (c, b) = (f32::from(c) / 255.0, f32::from(b) / 255.0);
        ((b + (c - b) * opacity).clamp(0.0, 1.0) * 255.0).round() as u8
    };
    (mix(rgb.0, bg.0), mix(rgb.1, bg.1), mix(rgb.2, bg.2))
}

/// Whether the cell's shape color is within `tolerance` of `rgb`.
fn shows_near(frame: &Snapshot, r: u16, c: u16, rgb: (u8, u8, u8), tolerance: u8) -> bool {
    let near = |color: vt100::Color| match color {
        vt100::Color::Rgb(red, green, blue) => {
            red.abs_diff(rgb.0) <= tolerance
                && green.abs_diff(rgb.1) <= tolerance
                && blue.abs_diff(rgb.2) <= tolerance
        }
        _ => false,
    };
    frame.screen.cell(r, c).is_some_and(|cell| {
        let glyph = cell.contents().chars().next().is_some_and(is_fill_glyph);
        near(cell.bgcolor()) || (glyph && near(cell.fgcolor()))
    })
}

/// CHT-030 and CHT-025: a filled box covers exactly the samples whose
/// centers lie inside it, like the same rectangle filled as a polygon, so a
/// node's edge inside a cell leaves that cell partly covered.
#[test]
fn cht_030_a_filled_box_covers_only_its_own_pixels() {
    use reactive_tui::widgets::display::charts::mask::{GlyphSet, MaskCanvas};
    use reactive_tui::widgets::display::Blitter;
    let red = Some((0.8, 0.1, 0.1, 1.0));
    // Dots 1 to 5 of a two-row column: two of row 0's three sextant pixel
    // rows and one of row 1's.
    let (x0, y0, x1, y1) = (0.0, 1.0, 2.0, 5.0);
    let mut boxed = MaskCanvas::new(1, 2);
    boxed.set_fill_blitter(Blitter::Sextant);
    boxed.fill_where((x0, y0, x1, y1), red, None, |_, _| true);
    let mut polygon = MaskCanvas::new(1, 2);
    polygon.set_fill_blitter(Blitter::Sextant);
    polygon.fill_polygon(&[(x0, y0), (x1, y0), (x1, y1), (x0, y1)], red, None);
    for row in 0..2 {
        let (got, want) = (
            boxed.resolve(0, row, GlyphSet::Unicode),
            polygon.resolve(0, row, GlyphSet::Unicode),
        );
        assert_eq!(
            got, want,
            "row {row}: the box must cover the polygon's pixels"
        );
        assert_ne!(
            got.glyph,
            Some("\u{2588}"),
            "row {row} is only partly covered"
        );
    }
}

/// CHT-030: the layout gives each node the larger of its totals and each
/// link its value under the chosen scale, links stack at a node without
/// overlap, and the drawn nodes and ribbon ends lie where the layout puts
/// them, within one fill pixel, the ribbon ends in their nodes' colors
/// blended by the link opacity.
#[test]
fn cht_030_nodes_and_ribbons_take_their_values_under_the_scale() {
    use reactive_tui::widgets::display::plot::{Sankey, SankeyGraph, SankeyValueScale};
    let links = sankey_links(&SANKEY_LINKS);
    let layout = |scale: SankeyValueScale| -> SankeyGraph {
        Sankey::new()
            .value_scale(scale)
            .node_width(4.0)
            .node_padding(4.0)
            .snap_x(2.0)
            .extent(0.0, 0.0, 160.0, 96.0)
            .layout(SANKEY_NODES.len(), &links)
            .unwrap()
    };
    for scale in [SankeyValueScale::Linear, SankeyValueScale::Sqrt] {
        let graph = layout(scale);
        let ky = graph.links[0].width / graph.links[0].value;
        for (ribbon, link) in graph.links.iter().zip(&links) {
            assert!(
                (ribbon.value - scale.apply(link.value)).abs() < 1e-12
                    && (ribbon.width - ribbon.value * ky).abs() < 1e-9,
                "{scale:?}: link {} must be its value under the scale wide",
                ribbon.index
            );
        }
        for node in &graph.nodes {
            let total = |side: &[usize]| side.iter().map(|l| graph.links[*l].value).sum::<f64>();
            let larger = total(&node.source_links).max(total(&node.target_links));
            assert!(
                (node.value - larger).abs() < 1e-9
                    && (node.y1 - node.y0 - larger * ky).abs() < 1e-6,
                "{scale:?}: node {} must be its larger total tall",
                node.index
            );
            for (side, source) in [(&node.source_links, true), (&node.target_links, false)] {
                let mut spans: Vec<(f64, f64)> = side
                    .iter()
                    .map(|l| {
                        let r = &graph.links[*l];
                        let center = if source { r.y0 } else { r.y1 };
                        (center - r.width / 2.0, center + r.width / 2.0)
                    })
                    .collect();
                spans.sort_by(|a, b| a.0.total_cmp(&b.0));
                assert!(
                    spans.windows(2).all(|w| w[0].1 <= w[1].0 + 1e-9)
                        && spans
                            .iter()
                            .all(|s| s.0 >= node.y0 - 1e-9 && s.1 <= node.y1 + 1e-9),
                    "{scale:?}: the links at node {} must stack inside it without overlap: {spans:?}",
                    node.index
                );
            }
        }
    }
    // The drawn chart, with empty labels so the plot is the whole 80 by 24
    // area (160 by 96 dots), matches the linear layout.
    let size = (80u16, 24u16);
    let frame = sankey_frame(
        sankey_builder(size).node_label(|_| String::new()).build(),
        size,
    );
    let graph = layout(SankeyValueScale::Linear);
    let pixel = 4.0 / 3.0;
    for node in &graph.nodes {
        let rgb = node_rgb(node.index);
        let cols = (node.x0 / 2.0) as u16..(node.x1 / 2.0) as u16;
        for r in 0..size.1 {
            let (top, bottom) = (f64::from(r) * 4.0, f64::from(r) * 4.0 + 4.0);
            for c in cols.clone() {
                if top >= node.y0 && bottom <= node.y1 {
                    assert!(
                        shows(&frame, r, c, rgb),
                        "node {} must fill ({r}, {c}) inside its height:\n{}",
                        node.index,
                        frame.text
                    );
                } else if top >= node.y1 + pixel || bottom <= node.y0 - pixel {
                    assert!(
                        !shows(&frame, r, c, rgb),
                        "node {} must not reach ({r}, {c}), a pixel past its height:\n{}",
                        node.index,
                        frame.text
                    );
                }
            }
        }
    }
    // Per pixel, on a graph whose column totals put node edges inside
    // cells: a row that a node covers only in part is not a full block in
    // the node's color.
    let uneven_links = sankey_links(&[(0, 2, 3.0), (1, 2, 4.0), (2, 3, 5.0), (2, 4, 2.0)]);
    let uneven = sankey_frame(
        SankeyChartBuilder::new(["a", "b", "c", "d", "e"], uneven_links.clone())
            .node_label(|_| String::new())
            .size(size.0, size.1)
            .build(),
        size,
    );
    let uneven_graph = Sankey::new()
        .node_width(4.0)
        .node_padding(4.0)
        .snap_x(2.0)
        .extent(0.0, 0.0, 160.0, 96.0)
        .layout(5, &uneven_links)
        .unwrap();
    let mut partial = 0;
    for node in &uneven_graph.nodes {
        let rgb = node_rgb(node.index);
        let bottom = node.y1.max(node.y0 + pixel);
        for r in 0..size.1 {
            let covered = (0..3)
                .map(|k| f64::from(r) * 4.0 + (f64::from(k) + 0.5) * pixel)
                .filter(|y| *y >= node.y0 && *y < bottom)
                .count();
            if covered == 0 || covered == 3 {
                continue;
            }
            for c in (node.x0 / 2.0) as u16..(node.x1 / 2.0) as u16 {
                partial += 1;
                let full = uneven
                    .screen
                    .cell(r, c)
                    .is_some_and(|cell| cell.contents() == "\u{2588}" && shows(&uneven, r, c, rgb));
                assert!(
                    !full,
                    "node {} covers {covered} of row {r}'s 3 pixel rows but fills ({r}, {c}) whole:\n{}",
                    node.index,
                    uneven.text
                );
            }
        }
    }
    assert!(partial > 0, "the layout puts node edges inside cells");
    let mut ends = 0;
    for ribbon in &graph.links {
        let (source, target) = (&graph.nodes[ribbon.source], &graph.nodes[ribbon.target]);
        for (col, center, rgb) in [
            ((source.x1 / 2.0) as u16, ribbon.y0, node_rgb(ribbon.source)),
            (
                (target.x0 / 2.0) as u16 - 1,
                ribbon.y1,
                node_rgb(ribbon.target),
            ),
        ] {
            let (top, bottom) = (
                center - ribbon.width / 2.0 + 1.0,
                center + ribbon.width / 2.0 - 1.0,
            );
            for r in 0..size.1 {
                if f64::from(r) * 4.0 >= top && f64::from(r) * 4.0 + 4.0 <= bottom {
                    ends += 1;
                    assert!(
                        shows_near(&frame, r, col, blended(rgb, 0.3), 8),
                        "link {} must end at ({r}, {col}) in its node's color blended by the opacity:\n{}",
                        ribbon.index,
                        frame.text
                    );
                }
            }
        }
    }
    assert!(ends >= 8, "the ribbon ends are measured ({ends} cells)");
}

/// CHT-030: the node alignment and the value scale each change the layout
/// of a graph where they apply.
#[test]
fn cht_030_alignment_and_value_scale_change_the_layout() {
    use reactive_tui::widgets::display::plot::{Sankey, SankeyAlign, SankeyValueScale};
    // D is a sink one step from the source: Justify puts it last, Left at
    // its depth.
    let links = sankey_links(&[(0, 1, 2.0), (1, 2, 2.0), (0, 3, 1.0)]);
    let layer = |align| {
        Sankey::new()
            .node_align(align)
            .extent(0.0, 0.0, 100.0, 100.0)
            .layout(4, &links)
            .unwrap()
            .nodes[3]
            .layer
    };
    assert_eq!(
        (layer(SankeyAlign::Justify), layer(SankeyAlign::Left)),
        (2, 1)
    );
    let size = (80u16, 24u16);
    let drawn = |align, scale| {
        sankey_frame(
            SankeyChartBuilder::new(["A", "B", "C", "D"], links.clone())
                .node_label(|n: &&str| *n)
                .node_align(align)
                .value_scale(scale)
                .size(size.0, size.1)
                .build(),
            size,
        )
    };
    let justify = drawn(SankeyAlign::Justify, SankeyValueScale::Linear);
    assert_ne!(
        justify.text,
        drawn(SankeyAlign::Left, SankeyValueScale::Linear).text,
        "Left must move the sink off the last column:\n{}",
        justify.text
    );
    let lopsided = sankey_links(&[(0, 1, 9.0), (0, 2, 1.0)]);
    let scaled = |scale| {
        sankey_frame(
            SankeyChartBuilder::new(["A", "B", "C"], lopsided.clone())
                .node_label(|n: &&str| *n)
                .value_scale(scale)
                .size(size.0, size.1)
                .build(),
            size,
        )
    };
    let linear = scaled(SankeyValueScale::Linear);
    assert_ne!(
        linear.text,
        scaled(SankeyValueScale::Sqrt).text,
        "the square-root scale must change the node heights:\n{}",
        linear.text
    );
}

/// CHT-030: at the medium class each node's label sits beside it, a
/// first-layer node's on its left, a last-layer node's on its right and the
/// middle node's above it; the large class adds the throughput; the mini
/// class draws no label.
#[test]
fn cht_030_labels_sit_beside_their_nodes() {
    let size = (80u16, 24u16);
    let frame = sankey_frame(sankey_builder(size).build(), size);
    let grid: Vec<Vec<char>> = frame.text.lines().map(|l| l.chars().collect()).collect();
    let find = |name: &str| -> (usize, usize) {
        let target: Vec<char> = name.chars().collect();
        grid.iter()
            .enumerate()
            .find_map(|(r, line)| {
                (0..line.len().saturating_sub(target.len() - 1))
                    .find(|c| line[*c..*c + target.len()] == target[..])
                    .map(|c| (r, c))
            })
            .unwrap_or_else(|| panic!("label {name} must be placed:\n{}", frame.text))
    };
    let fill_at = |r: usize, c: usize| {
        grid.get(r)
            .and_then(|l| l.get(c))
            .is_some_and(|g| is_fill_glyph(*g))
    };
    for name in ["coal", "gas", "solar"] {
        let (r, c) = find(name);
        let end = c + name.len();
        assert!(
            !(0..c).any(|x| fill_at(r, x)) && (end..end + 4).any(|x| fill_at(r, x)),
            "{name} must sit left of its first-layer node:\n{}",
            frame.text
        );
    }
    for name in ["industry", "homes", "loss"] {
        let (r, c) = find(name);
        assert!(
            !(c..grid[r].len()).any(|x| fill_at(r, x))
                && (c.saturating_sub(4)..c).any(|x| fill_at(r, x)),
            "{name} must sit right of its last-layer node:\n{}",
            frame.text
        );
    }
    let (r, c) = find("power");
    let power = node_rgb(3);
    assert!(
        (c..c + 5).any(|x| shows(&frame, r as u16 + 1, x as u16, power)),
        "power must sit above its node:\n{}",
        frame.text
    );
    // A middle node whose top the layout leaves a rounding error above the
    // plot keeps its label, and a two-line label keeps both lines.
    let chain = sankey_links(&[(0, 1, 4.0), (1, 2, 1.0)]);
    let chained = sankey_frame(
        SankeyChartBuilder::new(["a", "b", "c"], chain.clone())
            .node_label(|n: &&str| *n)
            .size(size.0, size.1)
            .build(),
        size,
    );
    let rows: Vec<&str> = chained.text.lines().collect();
    assert!(
        rows.iter()
            .any(|row| row.split_whitespace().any(|word| word == "b")),
        "the middle node b must be labeled:\n{}",
        chained.text
    );
    let lined = sankey_frame(
        SankeyChartBuilder::new(["a", "b", "c"], chain)
            .labels(|n: &&str, v| {
                vec![
                    SankeyLabel::new(format!("{n}-name")),
                    SankeyLabel::new(format!("{n}-flow {v}")),
                ]
            })
            .size(size.0, size.1)
            .build(),
        size,
    );
    for line in ["b-name", "b-flow 4"] {
        assert!(
            lined.text.contains(line),
            "a middle node's two-line label keeps its line {line}:\n{}",
            lined.text
        );
    }
    let size = (200u16, 40u16);
    let large = sankey_frame(sankey_builder(size).build(), size);
    for label in ["coal 4", "power 9", "industry 5", "loss 1"] {
        assert!(
            large.text.contains(label),
            "a large Sankey chart labels each node with its throughput ({label}):\n{}",
            large.text
        );
    }
    let size = (20u16, 5u16);
    let mini = sankey_frame(sankey_builder(size).build(), size);
    assert!(
        !mini.text.chars().any(char::is_alphabetic) && mini.text.chars().any(is_fill_glyph),
        "a mini Sankey chart draws shapes only:\n{}",
        mini.text
    );
}

/// CHT-030: through the untyped ChartsBuilder, whose node points carry no
/// throughput, the large class still labels each node with the larger of its
/// incoming and outgoing totals.
#[test]
fn cht_030_the_untyped_builder_shows_each_nodes_throughput() {
    use reactive_tui::widgets::display::{ChartsBuilder, SankeyOptions};
    let size = (200u16, 40u16);
    let nodes = DataSeries::new(
        "nodes",
        SANKEY_NODES
            .iter()
            .map(|name| DataPoint::with_label(0.0, *name))
            .collect(),
    );
    let props = ChartsBuilder::sankey()
        .series(nodes)
        .sankey_options(SankeyOptions {
            links: sankey_links(&SANKEY_LINKS),
            ..SankeyOptions::default()
        })
        .size(size.0, size.1)
        .build();
    let frame = sankey_frame(props, size);
    for label in ["coal 4", "power 9", "industry 5", "loss 1"] {
        assert!(
            frame.text.contains(label),
            "the untyped chart labels each node with its throughput ({label}):\n{}",
            frame.text
        );
    }
}

/// CHT-030: a link naming a missing node, or links forming a cycle, show a
/// message instead of shapes.
#[test]
fn cht_030_a_missing_node_or_a_cycle_shows_an_error() {
    let size = (60u16, 12u16);
    for (links, message) in [
        (vec![(0, 5, 1.0)], "missing node"),
        (vec![(0, 1, 1.0), (1, 0, 1.0)], "cycle"),
    ] {
        let frame = sankey_frame(
            SankeyChartBuilder::new(["a", "b"], sankey_links(&links))
                .size(size.0, size.1)
                .build(),
            size,
        );
        assert!(
            frame.text.contains(message) && !frame.text.chars().any(is_fill_glyph),
            "a Sankey chart with {message} must say so and draw no shape:\n{}",
            frame.text
        );
    }
}

/// CHT-026: a Sankey chart with a non-finite link value, or with nothing to
/// draw, shows a message and no shapes.
#[test]
fn cht_026_sankey_bad_or_empty_input_shows_a_message() {
    let size = (60u16, 12u16);
    for (nodes, links, message) in [
        (vec!["a", "b"], vec![(0, 1, f64::NAN)], "finite"),
        (vec!["a", "b"], vec![(0, 1, f64::INFINITY)], "finite"),
        (vec![], vec![], "No data"),
        (vec!["a", "b"], vec![], "No data"),
    ] {
        let frame = sankey_frame(
            SankeyChartBuilder::new(nodes, sankey_links(&links))
                .size(size.0, size.1)
                .build(),
            size,
        );
        assert!(
            frame.text.contains(message) && !frame.text.chars().any(is_fill_glyph),
            "expected '{message}' and no shape:\n{}",
            frame.text
        );
    }
    // Nodes too wide for the chart say so instead of leaving it blank.
    let wide = sankey_frame(sankey_builder(size).node_width(40).build(), size);
    assert!(
        wide.text.contains("do not fit") && !wide.text.chars().any(is_fill_glyph),
        "a Sankey chart whose nodes do not fit must say so:\n{}",
        wide.text
    );
}

/// CHT-024: a Sankey chart follows its forced size class: the large class
/// adds throughputs to the labels and the mini class drops them.
#[test]
fn cht_024_sankey_follows_its_forced_size_class() {
    let size = (80u16, 24u16);
    let at = |class| sankey_frame(sankey_builder(size).size_class(class).build(), size);
    let (medium, large, mini) = (
        at(SizeClass::Medium),
        at(SizeClass::Large),
        at(SizeClass::Mini),
    );
    assert!(
        medium.text.contains("power") && !medium.text.contains("power 9"),
        "{}",
        medium.text
    );
    assert!(large.text.contains("power 9"), "{}", large.text);
    assert!(!mini.text.contains("power"), "{}", mini.text);
}

/// CHT-028: a Sankey chart in ASCII mode draws no block or sextant glyph and
/// keeps its shapes.
#[test]
fn cht_028_sankey_resolves_with_ascii() {
    let size = (80u16, 24u16);
    let frame = sankey_frame(sankey_builder(size).ascii(true).build(), size);
    assert!(
        !frame
            .text
            .chars()
            .any(|c| is_fill_glyph(c) || is_braille(c))
            && frame.text.chars().any(|c| "#|-.".contains(c)),
        "ASCII mode must keep the shapes without Unicode glyphs:\n{}",
        frame.text
    );
}
