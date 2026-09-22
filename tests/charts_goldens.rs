//! charts-goldens mechanism: CHT-012, CHT-013, CHT-014, CHT-023, CHT-024,
//! CHT-025, CHT-026, CHT-027, CHT-028 and BAR-004.
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
        frame
            .text
            .lines()
            .nth(row)
            .map_or(0, |line| line.chars().filter(|c| !c.is_whitespace()).count())
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
    let pie = last(ChartType::Pie, (40, 20), &[1.0, 1.0], 2.0);
    let shape = |c: char| is_braille(c) || "▁▂▃▄▅▆▇█▏▎▍▌▋▊▉".contains(c);
    let (mut left, mut right) = (0usize, 0usize);
    for line in pie.text.lines() {
        let chars: Vec<char> = line.chars().collect();
        let half = chars.len() / 2;
        left += chars[..half].iter().filter(|c| shape(**c)).count();
        right += chars[half..].iter().filter(|c| shape(**c)).count();
    }
    assert!(
        left > 0 && (left as i64 - right as i64).abs() <= 2,
        "two equal slices must cover mirror halves (left {left}, right {right}):\n{}",
        pie.text
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

/// CHT-023 and BAR-004: goldens at mini, medium and large for each delivered type.
#[test]
fn cht_023_goldens_at_three_size_classes() {
    let data = [2.0, 8.0, 5.0, 9.0, 3.0, 7.0];
    for (kind, name) in [
        (ChartType::Line, "line"),
        (ChartType::Area, "area"),
        (ChartType::Scatter, "scatter"),
        (ChartType::BarVertical, "bar"),
        (ChartType::Candlestick, "candlestick"),
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
            let frame = app_input::run_when_painted(Root(Element::typed::<Chart>(p)), size, 2)
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
}
