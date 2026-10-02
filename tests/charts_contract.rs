//! charts-interaction (CHT-018, CHT-019), charts-motion (CHT-022),
//! frame-budget (CHT-021, BAR-005), widget-bar (BAR-003), charts-palette
//! parity (CHT-017) and plot-layer scale behavior (CHT-011). Each test names
//! the requirement it observes.

mod common;

use std::sync::atomic::{AtomicBool, Ordering};

use common::app_input::{self, Snapshot};
use reactive_tui::app::{RootComponent, RootUpdate};
use reactive_tui::component::Element;
use reactive_tui::event::types::{
    Event, KeyCode, MouseEvent, MouseEventKind, Position, ResizeEvent,
};
use reactive_tui::widgets::display::{
    Chart, ChartAxis, ChartLegend, ChartProps, ChartType, DataPoint, DataSeries,
    SankeyChartBuilder, SankeyLink,
};

struct Root(Element);
impl RootComponent for Root {
    fn render(&self) -> Element {
        self.0.clone()
    }
    fn update(&mut self) -> reactive_tui::error::Result<RootUpdate> {
        note_chart_workers();
        Ok(RootUpdate::Unchanged)
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

/// Whether an `rtui-chart-*` thread was seen alive during any frame of the
/// current test; the worker is joined when the App drops, so the check has
/// to happen while the App runs.
#[cfg(target_os = "linux")]
static CHART_WORKER_SEEN: AtomicBool = AtomicBool::new(false);
/// Whether the image widget's `image-loader` worker, which draws each GIF
/// frame through the blitters, was seen alive during any frame.
#[cfg(target_os = "linux")]
static IMAGE_WORKER_SEEN: AtomicBool = AtomicBool::new(false);

fn note_chart_workers() {
    #[cfg(target_os = "linux")]
    {
        if !chart_worker_threads().is_empty() {
            CHART_WORKER_SEEN.store(true, Ordering::SeqCst);
        }
        if !threads_named("image-loader").is_empty() {
            IMAGE_WORKER_SEEN.store(true, Ordering::SeqCst);
        }
    }
}

#[cfg(target_os = "linux")]
fn chart_worker_seen() -> bool {
    CHART_WORKER_SEEN.load(Ordering::SeqCst)
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

/// Candles whose closes are `values`, each opening at the previous close.
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

fn props(kind: ChartType, size: (u16, u16), values: &[f64]) -> ChartProps {
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
            max: Some(10.0),
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

fn hover(x: u16, y: u16) -> Option<Event> {
    Some(Event::Mouse(MouseEvent::new(
        MouseEventKind::Move,
        Position::cell(x, y),
    )))
}

fn count(frame: &Snapshot, f: impl Fn(char) -> bool) -> usize {
    frame.text.chars().filter(|c| f(*c)).count()
}

fn cell_is(frame: &Snapshot, row: u16, col: u16, glyph: &str) -> bool {
    frame
        .screen
        .cell(row, col)
        .map(|c| c.contents() == glyph)
        .unwrap_or(false)
}

fn column_has(frame: &Snapshot, col: u16, glyph: &str) -> usize {
    let (rows, _) = frame.screen.size();
    (0..rows).filter(|r| cell_is(frame, *r, col, glyph)).count()
}

#[cfg(target_os = "linux")]
fn chart_worker_threads() -> Vec<String> {
    threads_named("rtui-chart")
}

#[cfg(target_os = "linux")]
fn threads_named(prefix: &str) -> Vec<String> {
    std::fs::read_dir("/proc/self/task")
        .unwrap()
        .filter_map(|t| std::fs::read_to_string(t.ok()?.path().join("comm")).ok())
        .filter(|n| n.starts_with(prefix))
        .collect()
}

/// CHT-011: band padding, zero inclusion and zero baseline.
#[test]
fn cht_011_band_padding_zero_inclusion_and_zero_baseline() {
    let size = (40u16, 12u16);
    // Bars must not touch: four separate runs of blocks on the top painted row.
    let bars = app_input::run(
        Root(Element::typed::<Chart>(props(
            ChartType::BarVertical,
            size,
            &[8.0, 8.0, 8.0, 8.0],
        ))),
        size,
        vec![(2, None)],
    )
    .pop()
    .unwrap();
    let (rows, cols) = bars.screen.size();
    let top_row = (0..rows)
        .find(|r| (0..cols).any(|c| cell_is(&bars, *r, c, "█")))
        .expect("bars painted");
    let runs = (0..cols)
        .map(|c| cell_is(&bars, top_row, c, "█"))
        .fold((0usize, false), |(n, prev), on| {
            (if on && !prev { n + 1 } else { n }, on)
        })
        .0;
    assert_eq!(
        runs, 4,
        "four padded bars expected on the top row:\n{}",
        bars.text
    );
    // Zero inclusion: a positive-only series with an automatic axis starts at 0.
    let mut auto = props(ChartType::BarVertical, size, &[5.0, 6.0, 7.0]);
    auto.y_axis = ChartAxis {
        min: None,
        max: None,
        show_labels: true,
        show_grid: false,
        ..Default::default()
    };
    let z = app_input::run(Root(Element::typed::<Chart>(auto)), size, vec![(2, None)])
        .pop()
        .unwrap();
    assert!(
        z.text
            .lines()
            .rev()
            .take(3)
            .any(|l| l.trim_start().starts_with('0')),
        "automatic axis must include zero:\n{}",
        z.text
    );
    // Zero baseline: a negative bar hangs below the positive bar's baseline row.
    let mut signed = props(ChartType::BarVertical, size, &[4.0, -4.0]);
    signed.y_axis = ChartAxis {
        min: Some(-5.0),
        max: Some(5.0),
        show_labels: false,
        show_grid: false,
        ..Default::default()
    };
    let s = app_input::run(Root(Element::typed::<Chart>(signed)), size, vec![(2, None)])
        .pop()
        .unwrap();
    let first_col = (0..cols)
        .find(|c| column_has(&s, *c, "█") > 0)
        .expect("positive bar");
    let last_col = (0..cols)
        .rev()
        .find(|c| column_has(&s, *c, "█") > 0)
        .expect("negative bar");
    let top_of = |col: u16| (0..rows).find(|r| cell_is(&s, *r, col, "█")).unwrap();
    let bottom_of = |col: u16| (0..rows).rev().find(|r| cell_is(&s, *r, col, "█")).unwrap();
    assert!(
        bottom_of(first_col) <= top_of(last_col),
        "the negative bar must hang below the positive bar's baseline:\n{}",
        s.text
    );
}

/// CHT-018: tooltip is a box beside the hovered index with a swatch row per
/// series, a crosshair outside the box, and it flips at the right edge.
#[test]
fn cht_018_tooltip_is_a_box_with_swatch_rows_and_a_crosshair_that_flips_at_the_edge() {
    let size = (40u16, 14u16);
    let mut p = props(ChartType::Line, size, &[2.0, 8.0, 5.0]);
    p.series.push(DataSeries::new(
        "second",
        [1.0, 4.0, 9.0]
            .iter()
            .enumerate()
            .map(|(i, v)| DataPoint::with_label(*v, format!("q{i}")))
            .collect(),
    ));
    let frames = app_input::run(
        Root(Element::typed::<Chart>(p.clone())),
        size,
        vec![(2, hover(20, 7)), (3, None)],
    );
    let f = frames.last().unwrap();
    assert!(
        count(f, |c| "┌╭┏".contains(c)) > 0,
        "tooltip must be a box, got:\n{}",
        f.text
    );
    assert!(
        count(f, |c| c == '■') >= 2,
        "one swatch per series expected:\n{}",
        f.text
    );
    assert!(
        f.text.contains("second"),
        "series name row missing:\n{}",
        f.text
    );
    // Crosshair: the hovered column carries '│' on rows that hold no box border.
    let (rows, _) = f.screen.size();
    let is_box_row = |r: u16| {
        let line = f.text.lines().nth(r as usize).unwrap_or("");
        line.contains('┌') || line.contains('└') || line.contains('╭') || line.contains('╰')
    };
    // The crosshair marks the selected index's column, which is the data
    // column nearest the pointer, so it sits within one cell of the hover.
    let crosshair = (19..=21)
        .map(|col| {
            (0..rows)
                .filter(|r| !is_box_row(*r))
                .filter(|r| cell_is(f, *r, col, "│"))
                .count()
        })
        .max()
        .unwrap_or(0);
    assert!(
        crosshair >= 2,
        "crosshair column expected at the hovered index outside the box:\n{}",
        f.text
    );
    // Edge flip: hovering near the right edge keeps the box inside the chart.
    let edge = app_input::run(
        Root(Element::typed::<Chart>(p)),
        size,
        vec![(2, hover(38, 7)), (3, None)],
    )
    .pop()
    .unwrap();
    let left_border =
        (0..size.0).find(|c| column_has(&edge, *c, "┌") > 0 || column_has(&edge, *c, "╭") > 0);
    assert!(
        matches!(left_border, Some(c) if c < 30),
        "near the right edge the tooltip must flip to the left:\n{}",
        edge.text
    );
}

/// CHT-019: nearest index on x; two hovers inside one band select the same
/// point and leave the frame unchanged; an empty cell beside a scatter point
/// still selects it.
/// CHT-018, CHT-024: a mini chart hovered keeps its shapes and draws no box;
/// the hovered value still reaches the accessibility description.
#[test]
fn cht_018_mini_chart_keeps_its_shapes_and_speaks_the_hovered_value() {
    let is_shape = |c: char| ('\u{2800}'..='\u{28FF}').contains(&c) || "▁▂▃▄▅▆▇█".contains(c);
    for size in [(8u16, 2u16), (20u16, 5u16)] {
        let mut p = props(ChartType::Line, size, &[2.0, 8.0, 5.0, 7.0]);
        p.series.push(DataSeries::new(
            "second",
            [1.0, 4.0, 9.0, 3.0]
                .iter()
                .enumerate()
                .map(|(i, v)| DataPoint::with_label(*v, format!("q{i}")))
                .collect(),
        ));
        let plain = app_input::run_when_painted(Root(Element::typed::<Chart>(p.clone())), size, 2)
            .pop()
            .unwrap();
        let hovered = app_input::run(
            Root(Element::typed::<Chart>(p)),
            size,
            vec![(2, hover(size.0 / 2, size.1 / 2)), (3, None)],
        )
        .pop()
        .unwrap();
        assert_eq!(
            count(&hovered, |c| "┌╭┏└╰┗".contains(c)),
            0,
            "a mini chart draws no tooltip box at {size:?}:\n{}",
            hovered.text
        );
        assert!(
            count(&hovered, is_shape) * 2 >= count(&plain, is_shape),
            "hovering a mini chart must keep its shapes at {size:?}:\nbefore\n{}\nafter\n{}",
            plain.text,
            hovered.text
        );
    }
}

#[test]
fn cht_019_mouse_selects_nearest_index() {
    let size = (40u16, 12u16);
    let p = props(ChartType::BarVertical, size, &[2.0, 8.0, 5.0, 7.0]);
    let a = app_input::run(
        Root(Element::typed::<Chart>(p.clone())),
        size,
        vec![(2, hover(12, 6)), (3, None)],
    )
    .pop()
    .unwrap();
    let b = app_input::run(
        Root(Element::typed::<Chart>(p)),
        size,
        vec![(2, hover(13, 6)), (3, None)],
    )
    .pop()
    .unwrap();
    assert_eq!(
        a.text, b.text,
        "two hovers inside one band must select the same point and leave the frame unchanged"
    );
    assert!(
        a.text.contains("p1"),
        "nearest point label expected in tooltip:\n{}",
        a.text
    );
    let scatter = props(ChartType::Scatter, size, &[2.0, 8.0, 5.0, 7.0]);
    let plain = app_input::run(
        Root(Element::typed::<Chart>(scatter.clone())),
        size,
        vec![(2, None)],
    )
    .pop()
    .unwrap();
    let (row, col) = (0..size.1)
        .flat_map(|r| (0..size.0).map(move |c| (r, c)))
        .find(|(r, c)| cell_is(&plain, *r, *c, "\u{2022}"))
        .expect("scatter paints a point");
    let beside = app_input::run(
        Root(Element::typed::<Chart>(scatter)),
        size,
        vec![(2, hover(col + 1, row + 1)), (3, None)],
    )
    .pop()
    .unwrap();
    assert!(
        beside.text.contains('p'),
        "hovering the empty cell next to a point must select the nearest index, got:\n{}",
        beside.text
    );
}

/// Root that swaps its data when it receives any key, to observe transitions.
struct Switching {
    switched: AtomicBool,
    redraw: AtomicBool,
    size: (u16, u16),
    before: Vec<f64>,
    after: Vec<f64>,
    /// Leave the value range to the data instead of pinning 0 to 10.
    auto_range: bool,
}
impl RootComponent for Switching {
    fn render(&self) -> Element {
        let switched = self.switched.load(Ordering::SeqCst);
        let mut p = props(
            ChartType::BarVertical,
            self.size,
            if switched { &self.after } else { &self.before },
        );
        if self.auto_range {
            p.y_axis.min = None;
            p.y_axis.max = None;
        }
        p.animated = switched;
        p.animation_duration = 200;
        Element::typed::<Chart>(p)
    }
    fn handle_event(&self, _event: &Event) -> reactive_tui::event::router::EventResult {
        self.switched.store(true, Ordering::SeqCst);
        self.redraw.store(true, Ordering::SeqCst);
        reactive_tui::event::router::EventResult::Handled
    }
    fn update(&mut self) -> reactive_tui::error::Result<RootUpdate> {
        Ok(if self.redraw.swap(false, Ordering::SeqCst) {
            RootUpdate::Redraw
        } else {
            RootUpdate::Unchanged
        })
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

/// CHT-022: a data change animates from the previous values through
/// intermediate frames and ends exactly at the target, never dipping below
/// the previous rendering.
#[test]
fn cht_022_value_transition_moves_from_the_old_values_to_the_target() {
    let size = (30u16, 12u16);
    let target = app_input::run(
        Root(Element::typed::<Chart>(props(
            ChartType::BarVertical,
            size,
            &[9.0, 9.0],
        ))),
        size,
        vec![(2, None)],
    )
    .pop()
    .unwrap();
    let settled = app_input::run(
        Root(Element::typed::<Chart>(props(
            ChartType::BarVertical,
            size,
            &[5.0, 5.0],
        ))),
        size,
        vec![(2, None)],
    )
    .pop()
    .unwrap();
    let before = count(&settled, |c| c == '\u{2588}');
    let after = count(&target, |c| c == '\u{2588}');
    assert!(
        before > 0 && after > before,
        "test data must be visible: before {before}, after {after}"
    );
    // The first bar's column: the outer band padding keeps it off column 0.
    let bar_col = (0..size.0)
        .find(|c| column_has(&settled, *c, "\u{2588}") > 0)
        .expect("a bar column");
    let settled_row = (0..size.1)
        .find(|r| cell_is(&settled, *r, bar_col, "\u{2588}"))
        .expect("settled bar column");
    // The transition's end is the target's tip cell (its topmost painted
    // cell in column 0), which no intermediate frame shows.
    let (target_row, target_tip) = (0..size.1)
        .find_map(|r| {
            let content = target.screen.cell(r, bar_col)?.contents();
            (!content.trim().is_empty()).then_some((r, content))
        })
        .expect("target bar column");
    let root = Switching {
        switched: AtomicBool::new(false),
        redraw: AtomicBool::new(false),
        size,
        before: vec![5.0, 5.0],
        after: vec![9.0, 9.0],
        auto_range: false,
    };
    let frames = app_input::run_when_cell(
        root,
        size,
        vec![
            app_input::CellStep {
                x: bar_col,
                y: settled_row,
                content: "\u{2588}",
                event: app_input::key(KeyCode::Char('s')),
            },
            app_input::CellStep {
                x: bar_col,
                y: target_row,
                content: Box::leak(target_tip.to_string().into_boxed_str()),
                event: None,
            },
        ],
    );
    let counts: Vec<usize> = frames
        .iter()
        .map(|f| count(f, |c| c == '\u{2588}'))
        .collect();
    eprintln!("full-block count per frame: {counts:?}");
    assert_eq!(
        frames.last().unwrap().text,
        target.text,
        "transition must end exactly at the target rendering"
    );
    let switch_index = counts
        .iter()
        .position(|c| *c >= before)
        .expect("settled start frame");
    let post: Vec<usize> = counts[switch_index + 1..].to_vec();
    assert!(
        post.iter().any(|c| *c > before && *c < after),
        "a value change must pass through intermediate frames, not jump: {counts:?}"
    );
    if let Some(i) = post.iter().position(|c| *c < before) {
        panic!(
            "frame {} dropped to {} blocks, below the previous rendering ({before}): the transition restarted from zero instead of moving from the old values",
            switch_index + 1 + i,
            post[i]
        );
    }
}

/// CHT-022: with the value range left to the data, a bar growing from 8 to
/// 16 never dips below its previous rendering: the range follows the
/// transition's values instead of jumping to the target's range while the
/// bar is still near 8.
#[test]
fn cht_022_an_automatic_range_never_dips_during_a_transition() {
    let size = (30u16, 12u16);
    let settled = |values: &[f64]| {
        let mut p = props(ChartType::BarVertical, size, values);
        p.y_axis.min = None;
        p.y_axis.max = None;
        app_input::run_when_painted(Root(Element::typed::<Chart>(p)), size, 2)
            .pop()
            .unwrap()
    };
    let before = count(&settled(&[8.0, 8.0]), |c| c == '\u{2588}');
    assert!(before > 0, "test data must be visible");
    let root = Switching {
        switched: AtomicBool::new(false),
        redraw: AtomicBool::new(false),
        size,
        before: vec![8.0, 8.0],
        after: vec![16.0, 16.0],
        auto_range: true,
    };
    let frames = app_input::run(
        root,
        size,
        vec![(2, app_input::key(KeyCode::Char('s'))), (8, None)],
    );
    let counts: Vec<usize> = frames
        .iter()
        .map(|f| count(f, |c| c == '\u{2588}'))
        .collect();
    let switch_index = counts
        .iter()
        .position(|c| *c >= before)
        .expect("settled start frame");
    if let Some(i) = counts[switch_index + 1..].iter().position(|c| *c < before) {
        panic!(
            "frame {} dropped to {} blocks, below the previous rendering ({before}): the automatic range jumped to the target's before the values got there: {counts:?}",
            switch_index + 1 + i,
            counts[switch_index + 1 + i]
        );
    }
}

/// CHT-022, CHT-026: after an invalid (NaN) data frame, the next valid data
/// animates from the last valid rendering, so no bar vanishes mid-transition.
struct Staged {
    stage: std::sync::atomic::AtomicUsize,
    redraw: AtomicBool,
    size: (u16, u16),
    stages: Vec<Vec<f64>>,
}
impl RootComponent for Staged {
    fn render(&self) -> Element {
        let i = self.stage.load(Ordering::SeqCst).min(self.stages.len() - 1);
        let mut p = props(ChartType::BarVertical, self.size, &self.stages[i]);
        p.animated = i > 0;
        p.animation_duration = 200;
        p.transition_duration = 200;
        Element::typed::<Chart>(p)
    }
    fn handle_event(&self, _event: &Event) -> reactive_tui::event::router::EventResult {
        self.stage.fetch_add(1, Ordering::SeqCst);
        self.redraw.store(true, Ordering::SeqCst);
        reactive_tui::event::router::EventResult::Handled
    }
    fn update(&mut self) -> reactive_tui::error::Result<RootUpdate> {
        Ok(if self.redraw.swap(false, Ordering::SeqCst) {
            RootUpdate::Redraw
        } else {
            RootUpdate::Unchanged
        })
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

#[test]
fn cht_022_recovery_from_nan_data_keeps_every_bar_through_the_transition() {
    let size = (30u16, 12u16);
    let settled = app_input::run_when_painted(
        Root(Element::typed::<Chart>(props(
            ChartType::BarVertical,
            size,
            &[5.0, 5.0],
        ))),
        size,
        2,
    )
    .pop()
    .unwrap();
    let block_columns = |f: &Snapshot| {
        (0..size.0)
            .filter(|c| column_has(f, *c, "\u{2588}") > 0)
            .count()
    };
    let settled_columns = block_columns(&settled);
    // The first bar's column: the outer band padding keeps it off column 0.
    let bar_col = (0..size.0)
        .find(|c| column_has(&settled, *c, "\u{2588}") > 0)
        .expect("a bar column");
    let settled_row = (0..size.1)
        .find(|r| cell_is(&settled, *r, bar_col, "\u{2588}"))
        .expect("settled bar column");
    let invalid = app_input::run_when_painted(
        Root(Element::typed::<Chart>(props(
            ChartType::BarVertical,
            size,
            &[f64::NAN, 5.0],
        ))),
        size,
        2,
    )
    .pop()
    .unwrap();
    let (msg_row, msg_col) = (0..size.1)
        .flat_map(|r| (0..size.0).map(move |c| (r, c)))
        .find(|(r, c)| cell_is(&invalid, *r, *c, "C"))
        .expect("the NaN chart shows its message");
    let target = app_input::run_when_painted(
        Root(Element::typed::<Chart>(props(
            ChartType::BarVertical,
            size,
            &[7.0, 7.0],
        ))),
        size,
        2,
    )
    .pop()
    .unwrap();
    let (target_row, target_tip) = (0..size.1)
        .find_map(|r| {
            let content = target.screen.cell(r, bar_col)?.contents();
            (!content.trim().is_empty()).then_some((r, content))
        })
        .expect("target bar column");
    let root = Staged {
        stage: std::sync::atomic::AtomicUsize::new(0),
        redraw: AtomicBool::new(false),
        size,
        stages: vec![vec![5.0, 5.0], vec![f64::NAN, 5.0], vec![7.0, 7.0]],
    };
    let frames = app_input::run_when_cell(
        root,
        size,
        vec![
            app_input::CellStep {
                x: bar_col,
                y: settled_row,
                content: "\u{2588}",
                event: app_input::key(KeyCode::Char('n')),
            },
            app_input::CellStep {
                x: msg_col,
                y: msg_row,
                content: "C",
                event: app_input::key(KeyCode::Char('v')),
            },
            app_input::CellStep {
                x: bar_col,
                y: target_row,
                content: Box::leak(target_tip.to_string().into_boxed_str()),
                event: None,
            },
        ],
    );
    let last_message = frames
        .iter()
        .rposition(|f| f.text.contains("finite"))
        .expect("the NaN frame was shown");
    let counts: Vec<usize> = frames.iter().map(block_columns).collect();
    eprintln!("block columns per frame: {counts:?} (message last shown at {last_message})");
    for (i, f) in frames.iter().enumerate().skip(last_message + 1) {
        assert!(
            block_columns(f) >= settled_columns,
            "frame {i} after the NaN recovery lost a bar (columns {} < {settled_columns}):\n{}",
            block_columns(f),
            f.text
        );
    }
}

/// CHT-021: a chart with unset size fills its parent, and rasterization runs
/// on a named worker thread.
#[test]
fn cht_021_chart_fills_its_parent_and_rasterizes_on_a_worker() {
    let size = (140u16, 40u16);
    let p = ChartProps {
        chart_type: ChartType::BarVertical,
        series: vec![series(&[2.0, 8.0, 5.0, 7.0])],
        animated: false,
        ..Default::default()
    };
    let frame = app_input::run(Root(Element::typed::<Chart>(p)), size, vec![(2, None)])
        .pop()
        .unwrap();
    let widest = frame
        .text
        .lines()
        .map(|l| l.trim_end().chars().count())
        .max()
        .unwrap_or(0);
    assert!(
        widest > 100,
        "a chart with no explicit size must fill a 140-column parent, painted {widest} columns"
    );
    #[cfg(target_os = "linux")]
    assert!(
        chart_worker_seen(),
        "no rtui-chart worker thread existed while rendering: rasterization runs on the main thread"
    );
}

/// The slowest frame's work after the first, and every such frame as
/// "work/present" in milliseconds, so a failure shows whether the App or
/// the backend's present took the time.
fn max_work_ms(
    root: impl RootComponent + 'static,
    size: (u16, u16),
    frames: usize,
) -> (f64, String) {
    // BAR-005 measures on the debug backend. It lays out and paints on its
    // paint thread while the App's thread waits in render_frame, so that
    // work counts in full. Every frame counts, the first included.
    let out = app_input::run_on_debug(root, size, vec![(frames, None)]);
    assert!(
        out.len() >= frames,
        "harness painted {} of {frames} frames",
        out.len()
    );
    let split: Vec<String> = out
        .iter()
        .map(|f| format!("{:.2}/{:.2}", f.work_ms, f.present_ms))
        .collect();
    (
        out.iter().map(|f| f.work_ms).fold(0.0, f64::max),
        split.join(" "),
    )
}

/// The load average, which a failed time bound reports: other work on the
/// host lengthens every frame measured in wall time.
fn load_average() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|text| text.split_whitespace().next().map(str::to_owned))
        .unwrap_or_else(|| "unknown".to_string())
}

/// `max_work_ms` over 12 frames, with a readable failure when the harness
/// cannot even paint that many inside its deadline.
fn frame_budget(root: impl RootComponent + 'static, size: (u16, u16)) -> (f64, String) {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        max_work_ms(root, size, 12)
    })) {
        Ok(measured) => measured,
        Err(_) => panic!(
            "fewer than 12 frames painted inside the harness's {:?} hang guard at {size:?} at load average {}: per-frame work was over {:.0} ms",
            app_input::HANG_GUARD,
            load_average(),
            app_input::HANG_GUARD.as_secs_f64() * 1000.0 / 12.0
        ),
    }
}

/// BAR-005: every cartesian chart type, animating, keeps per-frame work
/// under 16.6 ms at 700 by 200 on the debug backend, and rasterizes on a
/// worker.
#[test]
fn bar_005_animating_charts_stay_under_the_frame_budget_at_700_by_200() {
    if cfg!(debug_assertions) {
        // The App's per-element cost is about ten times higher without
        // optimization, so the budget is only meaningful on the optimized
        // build, which is what the frame-budget mechanism runs.
        eprintln!("SKIP: the frame budget is measured on the optimized build");
        // Still observe the worker at a size a debug build handles.
        let small = props(ChartType::Line, (80, 24), &[1.0, 3.0, 2.0]);
        let frames = app_input::run(
            Root(Element::typed::<Chart>(small)),
            (80, 24),
            vec![(2, None)],
        );
        assert!(frames.len() >= 2);
        #[cfg(target_os = "linux")]
        assert!(chart_worker_seen(), "no rtui-chart worker thread observed");
        return;
    }
    let size = (700u16, 200u16);
    let values: Vec<f64> = (0..120)
        .map(|i| ((i as f64) * 0.37).sin().abs() * 10.0)
        .collect();
    let mut over = Vec::new();
    for kind in [
        ChartType::Line,
        ChartType::Area,
        ChartType::Scatter,
        ChartType::BarVertical,
        ChartType::BarHorizontal,
        ChartType::Candlestick,
    ] {
        let mut p = props(kind.clone(), size, &values);
        if kind == ChartType::Candlestick {
            p.series = vec![candles(&values)];
            p.y_axis.max = Some(11.0);
        }
        p.animated = true;
        p.animation_duration = 2000;
        let (ms, split) = frame_budget(Root(Element::typed::<Chart>(p)), size);
        eprintln!("{kind:?} work/present ms per frame at 700x200: {split}");
        if ms >= 16.6 {
            over.push(format!("{kind:?} {ms:.2} ms ({split})"));
        }
    }
    assert!(
        over.is_empty(),
        "per-frame work exceeds 16.6 ms at 700x200 at load average {}: {}",
        load_average(),
        over.join("; ")
    );
    #[cfg(target_os = "linux")]
    assert!(
        chart_worker_seen(),
        "no rtui-chart worker thread observed during the run"
    );
}

/// An endlessly repeating GIF of `frames` frames, 20 ms each, whose content
/// changes on every frame.
fn gif(width: u32, height: u32, frames: u32) -> Vec<u8> {
    use image::{
        codecs::gif::{GifEncoder, Repeat},
        Delay, Frame,
    };
    let mut bytes = Vec::new();
    {
        let mut encoder = GifEncoder::new(&mut bytes);
        encoder.set_repeat(Repeat::Infinite).unwrap();
        for f in 0..frames {
            let pixels = image::RgbaImage::from_fn(width, height, |x, y| {
                image::Rgba([
                    ((x + f * 40) % 256) as u8,
                    ((y * 2 + f * 20) % 256) as u8,
                    ((f * 60) % 256) as u8,
                    255,
                ])
            });
            encoder
                .encode_frame(Frame::from_parts(
                    pixels,
                    0,
                    0,
                    Delay::from_numer_denom_ms(20, 1),
                ))
                .unwrap();
        }
    }
    bytes
}

/// BAR-005: the image widget playing a GIF that fills 700 by 200 keeps
/// per-frame work under 16.6 ms on the debug backend, and its image-loader
/// worker, not the App's thread, draws each frame through the blitters.
#[test]
fn bar_005_playing_gif_stays_under_the_frame_budget_at_700_by_200() {
    use reactive_tui::widgets::{
        display::{set_image_blitter, Blitter},
        ImageFormat,
    };
    if cfg!(debug_assertions) {
        eprintln!("SKIP: the frame budget is measured on the optimized build");
        assert!(!gif(4, 2, 2).is_empty());
        return;
    }
    // A named blitter wins over an installed chafa or viu (BLT-002), so the
    // picture is drawn the same way on every host.
    set_image_blitter(Some(Blitter::Sextant));
    let size = (700u16, 200u16);
    let picture = reactive_tui::builder::image()
        .source_raw_bytes(gif(280, 120, 4), 0, 0, ImageFormat::GIF)
        .build();
    let root = Root(
        reactive_tui::builder::div()
            .class("w-full h-full")
            .child(picture)
            .build(),
    );
    let (ms, split) = frame_budget(root, size);
    set_image_blitter(None);
    eprintln!("GIF work/present ms per frame at 700x200: {split}");
    assert!(
        ms < 16.6,
        "per-frame work {ms:.2} ms exceeds 16.6 ms at 700x200 while a GIF plays, at load average {} (work/present ms per frame: {split})",
        load_average()
    );
    #[cfg(target_os = "linux")]
    assert!(
        IMAGE_WORKER_SEEN.load(Ordering::SeqCst),
        "no image-loader worker thread observed while the GIF played"
    );
}

/// Negative fixture for BAR-005: a root that burns 25 ms per frame must be rejected.
#[test]
fn bar_005_fixture_slow_root_is_rejected() {
    struct Slow;
    impl RootComponent for Slow {
        fn render(&self) -> Element {
            std::thread::sleep(std::time::Duration::from_millis(25));
            Element::text("slow")
        }
        fn update(&mut self) -> reactive_tui::error::Result<RootUpdate> {
            Ok(RootUpdate::Redraw)
        }
    }
    let (ms, _) = max_work_ms(Slow, (80, 24), 5);
    assert!(
        ms >= 16.6,
        "fixture must exceed the budget so the mechanism can fail, measured {ms:.2} ms"
    );
}

/// The (right, bottom) edge of the painted cells, exclusive; (0, 0) when
/// nothing is painted.
fn ink_extent(frame: &Snapshot) -> (usize, usize) {
    frame
        .text
        .lines()
        .enumerate()
        .flat_map(|(y, line)| {
            line.chars()
                .enumerate()
                .filter(|(_, c)| !c.is_whitespace())
                .map(move |(x, _)| (x + 1, y + 1))
        })
        .fold((0, 0), |(right, bottom), (x, y)| {
            (right.max(x), bottom.max(y))
        })
}

/// BAR-003: after a resize the chart never paints the picture it drew for
/// the old size. Each frame at the new size is busy with an empty chart
/// area while the worker draws, or paints past the old rectangle. At 700 by
/// 200 the worker takes longer than any wait the chart makes.
#[test]
fn bar_003_a_resized_chart_never_paints_the_picture_drawn_for_the_old_size() {
    let old = (40u16, 12u16);
    let new = (700u16, 200u16);
    let mut p = props(ChartType::BarVertical, old, &[2.0, 8.0, 5.0, 7.0]);
    p.width = 0;
    p.height = 0;
    let frames = app_input::run(
        Root(Element::typed::<Chart>(p)),
        old,
        vec![
            (2, Some(Event::Resize(ResizeEvent::new(new.0, new.1)))),
            (3, None),
        ],
    );
    let before = frames
        .iter()
        .rfind(|f| f.screen.size() == (old.1, old.0))
        .expect("a frame at the old size");
    let (right, bottom) = ink_extent(before);
    assert!(
        right > 0 && right <= usize::from(old.0) && bottom <= usize::from(old.1),
        "the chart must paint inside {old:?} before the resize, painted to {right}x{bottom}"
    );
    let resized: Vec<&Snapshot> = frames
        .iter()
        .filter(|f| f.screen.size() == (new.1, new.0))
        .collect();
    let states: Vec<String> = resized
        .iter()
        .map(|f| {
            let (right, bottom) = ink_extent(f);
            format!("busy={} ink={right}x{bottom}", f.busy)
        })
        .collect();
    eprintln!("frames at the new size: {states:?}");
    assert!(
        resized.iter().any(|f| !f.busy),
        "no finished frame at the new size: {states:?}"
    );
    for (i, frame) in resized.iter().enumerate() {
        let (right, bottom) = ink_extent(frame);
        if frame.busy {
            assert_eq!(
                (right, bottom),
                (0, 0),
                "frame {i} at the new size is busy but paints cells: {states:?}"
            );
        } else {
            assert!(
                right > usize::from(old.0) || bottom > usize::from(old.1),
                "frame {i} at the new size paints only inside the old {old:?} rectangle, the picture drawn for the old size: {states:?}"
            );
        }
    }
}

/// BAR-003: every pointer action is reachable by keyboard: two Right presses
/// select the second point, as a hover would. (Fill is CHT-021's check.)
#[test]
fn bar_003_chart_selection_is_keyboard_reachable() {
    let size = (60u16, 20u16);
    let frames = app_input::run(
        Root(
            Element::typed::<Chart>(props(ChartType::BarVertical, size, &[2.0, 8.0, 5.0]))
                .auto_focus(),
        ),
        size,
        vec![
            (2, app_input::key(KeyCode::Right)),
            (3, app_input::key(KeyCode::Right)),
            (4, None),
        ],
    );
    assert!(
        frames.iter().any(|f| f.text.contains("p1")),
        "two Right presses must select the second point by keyboard:\n{}",
        frames.last().unwrap().text
    );
}

/// CHT-017: tokens that resolve for utility classes resolve identically in a chart.
#[test]
fn cht_017_utility_color_tokens_resolve_identically_in_charts() {
    use reactive_tui::layout::colors::parse_color_token;
    let size = (30u16, 10u16);
    for token in ["blue-500", "primary", "rgb(255,0,0)"] {
        let utility = parse_color_token(token)
            .unwrap_or_else(|| panic!("no single resolver handles the token {token}: theme variables and palette names must resolve through one path"));
        let mut p = props(ChartType::BarVertical, size, &[10.0]);
        p.series[0] = series(&[10.0]).with_color(token);
        let frame = app_input::run(Root(Element::typed::<Chart>(p)), size, vec![(2, None)])
            .pop()
            .unwrap();
        let painted = (0..size.1)
            .flat_map(|r| (0..size.0).map(move |c| (r, c)))
            .filter_map(|(r, c)| frame.screen.cell(r, c))
            .find(|cell| cell.contents() == "█")
            .map(|cell| cell.fgcolor());
        let expected = vt100::Color::Rgb(
            (utility.0 * 255.0).round() as u8,
            (utility.1 * 255.0).round() as u8,
            (utility.2 * 255.0).round() as u8,
        );
        assert_eq!(
            painted,
            Some(expected),
            "chart must paint {token} exactly as the utility resolver does"
        );
    }
}

/// Frames of a radial chart after `events`, focused so keys reach it, with
/// the fill blitter fixed at sextant so the frames do not depend on the host.
fn radial_run(
    p: ChartProps,
    size: (u16, u16),
    events: Vec<(usize, Option<Event>)>,
) -> Vec<Snapshot> {
    static CLEARED: std::sync::Once = std::sync::Once::new();
    CLEARED.call_once(|| std::env::remove_var("REACTIVE_TUI_BLITTER"));
    reactive_tui::widgets::display::set_image_blitter(Some(
        reactive_tui::widgets::display::Blitter::Sextant,
    ));
    app_input::run(Root(Element::typed::<Chart>(p).auto_focus()), size, events)
}

/// Whether a cell shows a radial shape: a block, sextant or octant glyph.
fn is_fill(frame: &Snapshot, row: u16, col: u16) -> bool {
    frame.screen.cell(row, col).is_some_and(|cell| {
        cell.contents().chars().next().is_some_and(|c| {
            "█▌▐▀▄▖▗▘▝▙▚▛▜▞▟".contains(c)
                || ('\u{1FB00}'..='\u{1FB3B}').contains(&c)
                || ('\u{1CD00}'..='\u{1CDE5}').contains(&c)
        })
    })
}

/// CHT-031: keys step through every drawn slice of a pie in order, across
/// series and past a point whose zero value draws no slice.
#[test]
fn cht_031_keys_reach_every_drawn_slice_across_series() {
    let size = (40u16, 12u16);
    let named = |name: &str, values: &[f64], prefix: &str| {
        DataSeries::new(
            name,
            values
                .iter()
                .enumerate()
                .map(|(i, v)| DataPoint::with_label(*v, format!("{prefix}{i}")))
                .collect(),
        )
    };
    let mut p = props(ChartType::Pie, size, &[]);
    p.series = vec![
        named("a", &[3.0, 0.0, 2.0], "a"),
        named("b", &[1.0, 2.0], "b"),
    ];
    let selected = |keys: &[KeyCode]| {
        let mut events: Vec<(usize, Option<Event>)> = keys
            .iter()
            .enumerate()
            .map(|(i, k)| (i + 2, app_input::key(k.clone())))
            .collect();
        events.push((keys.len() + 2, None));
        let frame = radial_run(p.clone(), size, events).pop().unwrap();
        ["a0", "a1", "a2", "b0", "b1"]
            .into_iter()
            .filter(|label| frame.text.contains(&format!("{label}:")))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        selected(&[KeyCode::Right]),
        vec!["a0"],
        "Right selects the first slice"
    );
    assert_eq!(
        selected(&[KeyCode::Right, KeyCode::Right]),
        vec!["a2"],
        "Right skips a point whose zero value draws no slice"
    );
    assert_eq!(
        selected(&[KeyCode::Right, KeyCode::Right, KeyCode::Right]),
        vec!["b0"],
        "Right moves on into the next series"
    );
    assert_eq!(
        selected(&[KeyCode::End]),
        vec!["b1"],
        "End selects the last slice of the last series"
    );
}

/// CHT-031: a slice so thin that the pad angle leaves nothing of it drawn
/// is not reachable: the keys step past it, and a pointer in the gap where
/// it would be does not select it.
#[test]
fn cht_031_keys_and_pointer_skip_a_slice_the_pad_leaves_undrawn() {
    let size = (40u16, 12u16);
    let mut p = props(ChartType::Pie, size, &[10.0, 0.05, 10.0]);
    p.radial.pad_angle = 0.1;
    let selected = |events: Vec<(usize, Option<Event>)>| {
        let frame = radial_run(p.clone(), size, events).pop().unwrap();
        (0..3)
            .filter(|i| frame.text.contains(&format!("p{i}:")))
            .collect::<Vec<_>>()
    };
    let key = |code: KeyCode| app_input::key(code);
    assert_eq!(
        selected(vec![
            (2, key(KeyCode::Right)),
            (3, key(KeyCode::Right)),
            (4, None)
        ]),
        vec![2],
        "Right from the first slice steps past the undrawn one to the next drawn slice"
    );
    assert_eq!(
        selected(vec![
            (2, key(KeyCode::End)),
            (3, key(KeyCode::Left)),
            (4, None)
        ]),
        vec![0],
        "Left from the last slice steps past the undrawn one"
    );
    // The undrawn slice would lie at six o'clock, below the center at
    // column 20, row 6.
    for row in 7..12u16 {
        for col in 19..=21u16 {
            assert!(
                !selected(vec![(2, hover(col, row)), (3, None)]).contains(&1),
                "a pointer at ({col}, {row}) must not select the undrawn slice"
            );
        }
    }
    // A pad that hides every slice, or values that draw none, leave the
    // keys nothing to select.
    let mut hidden = props(ChartType::Pie, size, &[1.0, 1.0, 1.0]);
    hidden.radial.pad_angle = 2.5;
    let empty = props(ChartType::Donut, size, &[0.0, 0.0, 0.0]);
    for (name, chart) in [("hidden", hidden), ("empty", empty)] {
        for code in [KeyCode::Right, KeyCode::End] {
            let frame = radial_run(chart.clone(), size, vec![(2, key(code.clone())), (3, None)])
                .pop()
                .unwrap();
            assert!(
                !(0..3).any(|i| frame.text.contains(&format!("p{i}:"))),
                "{name}: {code:?} must select nothing on a chart that draws no slice:\n{}",
                frame.text
            );
        }
    }
}

/// CHT-031 and CHT-018: a pie's tooltip names the selected slice alone with
/// a swatch in that slice's color, and the selection is marked on the chart
/// outside the tooltip box.
#[test]
fn cht_031_the_selected_slice_is_marked_and_its_swatch_takes_its_color() {
    let size = (40u16, 12u16);
    let p = props(ChartType::Pie, size, &[1.0, 1.0, 1.0, 1.0]);
    let plain = radial_run(p.clone(), size, vec![(2, None)]).pop().unwrap();
    let frame = radial_run(p, size, vec![(2, hover(14, 9)), (3, None)])
        .pop()
        .unwrap();
    let (r, g, b, _) = reactive_tui::theme::Theme::active()
        .resolve_color("chart-3")
        .unwrap();
    let rgb = |v: f32| (v * 255.0).round() as u8;
    let swatch = (0..size.1)
        .flat_map(|row| (0..size.0).map(move |col| (row, col)))
        .find(|(row, col)| cell_is(&frame, *row, *col, "■"))
        .expect("the tooltip draws a swatch");
    match frame.screen.cell(swatch.0, swatch.1).unwrap().fgcolor() {
        vt100::Color::Rgb(sr, sg, sb) => assert!(
            sr.abs_diff(rgb(r)) <= 2 && sg.abs_diff(rgb(g)) <= 2 && sb.abs_diff(rgb(b)) <= 2,
            "the swatch must take slice 2's color, chart-3, got ({sr}, {sg}, {sb})"
        ),
        other => panic!("the swatch has no color: {other:?}"),
    }
    assert!(
        !frame.text.contains("p0:") && !frame.text.contains("p1:") && !frame.text.contains("p3:"),
        "the tooltip names only the selected slice:\n{}",
        frame.text
    );
    let marked = |f: &Snapshot| {
        (0..size.1)
            .flat_map(|row| (0..size.0).map(move |col| (row, col)))
            .filter(|(row, col)| cell_is(f, *row, *col, "·"))
            .count()
    };
    assert!(
        marked(&frame) > marked(&plain),
        "a selected slice must be marked on the chart outside the tooltip:\n{}",
        frame.text
    );
}

/// CHT-031: every cell that shows a slice at the circle's rim selects a
/// slice, and a pointer just outside the circle, inside its bounding
/// square, or near the rim of a donut's hole, selects nothing.
#[test]
fn cht_031_rim_cells_select_and_the_bounding_square_and_hole_do_not() {
    let size = (40u16, 12u16);
    let p = props(ChartType::Pie, size, &[1.0, 1.0, 1.0, 1.0]);
    let plain = radial_run(p.clone(), size, vec![(2, None)]).pop().unwrap();
    let rim: Vec<(u16, u16)> = (0..size.1)
        .flat_map(|row| (0..size.0).map(move |col| (row, col)))
        .filter(|(row, col)| is_fill(&plain, *row, *col))
        .filter(|(row, col)| {
            [
                (row.wrapping_sub(1), *col),
                (row + 1, *col),
                (*row, col.wrapping_sub(1)),
                (*row, col + 1),
            ]
            .iter()
            .any(|(r, c)| !is_fill(&plain, *r, *c))
        })
        .collect();
    assert!(rim.len() > 10, "the pie has a rim:\n{}", plain.text);
    for (row, col) in rim {
        let frame = radial_run(p.clone(), size, vec![(2, hover(col, row)), (3, None)])
            .pop()
            .unwrap();
        assert!(
            (0..4).any(|i| frame.text.contains(&format!("p{i}:"))),
            "the painted rim cell ({col}, {row}) must select its slice:\n{}",
            frame.text
        );
    }
    // Inside the circle's bounding square but outside the circle, and near
    // the inner rim of a donut's hole (half the radius).
    for (kind, col, row) in [
        (ChartType::Pie, 9u16, 1u16),
        (ChartType::Pie, 30, 10),
        (ChartType::Donut, 15, 5),
        (ChartType::Donut, 23, 6),
    ] {
        let p = props(kind.clone(), size, &[1.0, 1.0, 1.0, 1.0]);
        let plain = radial_run(p.clone(), size, vec![(2, None)]).pop().unwrap();
        let pointed = radial_run(p, size, vec![(2, hover(col, row)), (3, None)])
            .pop()
            .unwrap();
        assert_eq!(
            plain.text, pointed.text,
            "{kind:?}: a pointer at ({col}, {row}) must select nothing:\n{}",
            pointed.text
        );
    }
}

/// CHT-031: a pointer inside a pie slice selects that slice, clockwise from
/// twelve o'clock: in a 40 by 12 pie of four equal slices the circle is
/// centered on column 20, row 6, with a radius of six rows.
#[test]
fn cht_031_a_pointer_inside_a_slice_selects_that_slice() {
    let size = (40u16, 12u16);
    let p = props(ChartType::Pie, size, &[1.0, 1.0, 1.0, 1.0]);
    // One cell inside each quadrant, halfway out.
    for (slice, (col, row)) in [(0, (26u16, 3u16)), (1, (26, 9)), (2, (14, 9)), (3, (14, 3))] {
        let frame = radial_run(p.clone(), size, vec![(2, hover(col, row)), (3, None)])
            .pop()
            .unwrap();
        let label = format!("p{slice}:");
        assert!(
            frame.text.contains(&label),
            "a pointer at ({col}, {row}) inside slice {slice} must select it:\n{}",
            frame.text
        );
        for other in (0..4).filter(|o| *o != slice) {
            assert!(
                !frame.text.contains(&format!("p{other}:")),
                "a pointer inside slice {slice} must not select slice {other}:\n{}",
                frame.text
            );
        }
    }
}

/// CHT-031: a pointer outside the outer radius, or in a donut's hole,
/// selects nothing and leaves the frame unchanged.
#[test]
fn cht_031_a_pointer_outside_the_radius_or_in_the_hole_selects_nothing() {
    let size = (40u16, 12u16);
    for (kind, col, row) in [
        (ChartType::Pie, 0u16, 0u16),
        (ChartType::Pie, 39, 11),
        (ChartType::Donut, 20, 6),
        (ChartType::Radar, 0, 0),
    ] {
        let p = props(kind.clone(), size, &[3.0, 1.0, 2.0, 4.0]);
        let plain = radial_run(p.clone(), size, vec![(2, None)]).pop().unwrap();
        let pointed = radial_run(p, size, vec![(2, hover(col, row)), (3, None)])
            .pop()
            .unwrap();
        assert_eq!(
            plain.text, pointed.text,
            "{kind:?}: a pointer at ({col}, {row}) must select nothing:\n{}",
            pointed.text
        );
    }
}

/// CHT-031: on a radar the pointer selects the category whose spoke is
/// nearest its angle. Four spokes point up, right, down and left from the
/// center at column 20, row 6.
#[test]
fn cht_031_a_radar_pointer_selects_the_nearest_spoke() {
    let size = (40u16, 12u16);
    let p = props(ChartType::Radar, size, &[4.0, 4.0, 4.0, 4.0]);
    for (spoke, (col, row)) in [(0, (22u16, 2u16)), (1, (27, 5)), (2, (18, 9)), (3, (13, 7))] {
        let frame = radial_run(p.clone(), size, vec![(2, hover(col, row)), (3, None)])
            .pop()
            .unwrap();
        assert!(
            frame.live.iter().any(|t| t.contains(&format!("p{spoke}:"))),
            "a pointer at ({col}, {row}) must select the nearest spoke, {spoke}: announced {:?}\n{}",
            frame.live,
            frame.text
        );
    }
}

/// CHT-031: Left, Right, Home and End move the selection in slice and
/// category order, and the selection shows the tooltip.
#[test]
fn cht_031_keys_move_through_slices_and_categories_in_order() {
    let size = (40u16, 12u16);
    for kind in [ChartType::Pie, ChartType::Donut, ChartType::Radar] {
        let p = props(kind.clone(), size, &[3.0, 1.0, 2.0, 4.0]);
        let selected = |keys: &[KeyCode]| {
            let mut events: Vec<(usize, Option<Event>)> = keys
                .iter()
                .enumerate()
                .map(|(i, k)| (i + 2, app_input::key(k.clone())))
                .collect();
            events.push((keys.len() + 2, None));
            let frame = radial_run(p.clone(), size, events).pop().unwrap();
            // The announcement names the selected point; the radar also
            // writes its category labels at the spokes, so the frame text
            // alone would not tell a selection from a label.
            (0..4)
                .filter(|i| frame.live.iter().any(|t| t.contains(&format!("p{i}:"))))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            selected(&[KeyCode::Right]),
            vec![0],
            "{kind:?}: Right selects the first slice or category"
        );
        assert_eq!(
            selected(&[KeyCode::Right, KeyCode::Right]),
            vec![1],
            "{kind:?}: Right steps to the next"
        );
        assert_eq!(
            selected(&[KeyCode::Right, KeyCode::Right, KeyCode::Left]),
            vec![0],
            "{kind:?}: Left steps back"
        );
        assert_eq!(
            selected(&[KeyCode::End]),
            vec![3],
            "{kind:?}: End selects the last"
        );
        assert_eq!(
            selected(&[KeyCode::End, KeyCode::Home]),
            vec![0],
            "{kind:?}: Home selects the first"
        );
    }
}

/// Two sources feed a hub, which feeds two sinks: the Sankey chart of the
/// selection tests, with each node's throughput.
const SANKEY_NAMES: [&str; 5] = ["north", "south", "hub", "east", "west"];
const SANKEY_THROUGHPUT: [&str; 5] = ["3", "2", "5", "4", "1"];

fn sankey_chart(size: (u16, u16)) -> ChartProps {
    let links = [(0, 2, 3.0), (1, 2, 2.0), (2, 3, 4.0), (2, 4, 1.0)]
        .map(|(s, t, v)| SankeyLink::new(s, t, v));
    SankeyChartBuilder::new(SANKEY_NAMES, links)
        .node_label(|n: &&str| *n)
        .size(size.0, size.1)
        .build()
}

/// The terminal color of palette entry `index` (nodes take the palette in
/// order).
fn node_rgb(index: usize) -> (u8, u8, u8) {
    let (r, g, b, _) = reactive_tui::theme::Theme::active()
        .resolve_color(&format!("chart-{}", index % 5 + 1))
        .expect("palette color");
    let c = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    (c(r), c(g), c(b))
}

/// The cells fully covered by node `index`: a full block in its color.
fn node_cells(frame: &Snapshot, index: usize) -> Vec<(u16, u16)> {
    let rgb = node_rgb(index);
    let (rows, cols) = frame.screen.size();
    (0..rows)
        .flat_map(|r| (0..cols).map(move |c| (r, c)))
        .filter(|(r, c)| {
            frame.screen.cell(*r, *c).is_some_and(|cell| {
                cell.contents() == "█"
                    && matches!(cell.fgcolor(), vt100::Color::Rgb(red, green, blue)
                        if red.abs_diff(rgb.0) <= 2 && green.abs_diff(rgb.1) <= 2 && blue.abs_diff(rgb.2) <= 2)
            })
        })
        .collect()
}

/// A cell's colors, to compare frames.
fn colors_at(frame: &Snapshot, r: u16, c: u16) -> Option<(vt100::Color, vt100::Color)> {
    frame
        .screen
        .cell(r, c)
        .map(|cell| (cell.fgcolor(), cell.bgcolor()))
}

/// The node the frame announces as selected, by its "name / throughput"
/// live-region text.
fn announced(frame: &Snapshot) -> Option<usize> {
    (0..SANKEY_NAMES.len()).find(|i| {
        frame
            .live
            .iter()
            .any(|t| t.contains(&format!("{} / {}", SANKEY_NAMES[*i], SANKEY_THROUGHPUT[*i])))
    })
}

/// CHT-032: a pointer on a node selects that node, and the tooltip and the
/// live region name it with its throughput; a pointer on a ribbon or an
/// empty cell selects nothing and leaves the frame unchanged.
#[test]
fn cht_032_a_pointer_on_a_node_selects_it_and_nothing_else_does() {
    let size = (80u16, 24u16);
    let plain = radial_run(sankey_chart(size), size, vec![(2, None)])
        .pop()
        .unwrap();
    assert_eq!(
        announced(&plain),
        None,
        "nothing is selected before a pointer moves"
    );
    for (index, name) in SANKEY_NAMES.iter().enumerate() {
        let cells = node_cells(&plain, index);
        assert!(!cells.is_empty(), "node {name} is drawn:\n{}", plain.text);
        let (r, c) = cells[cells.len() / 2];
        let frame = radial_run(sankey_chart(size), size, vec![(2, hover(c, r)), (3, None)])
            .pop()
            .unwrap();
        assert_eq!(
            announced(&frame),
            Some(index),
            "a pointer at ({c}, {r}) on {name} must select it; live region {:?}:\n{}",
            frame.live,
            frame.text
        );
        assert!(
            frame.text.contains(SANKEY_THROUGHPUT[index])
                && !plain.text.contains(SANKEY_THROUGHPUT[index]),
            "the tooltip must show {name}'s throughput:\n{}",
            frame.text
        );
    }
    // Ribbon cells: filled, in no node's own color.
    let any_node: Vec<(u16, u16)> = (0..SANKEY_NAMES.len())
        .flat_map(|i| node_cells(&plain, i))
        .collect();
    let ribbons: Vec<(u16, u16)> = (0..size.1)
        .flat_map(|r| (0..size.0).map(move |c| (r, c)))
        .filter(|(r, c)| {
            !any_node.contains(&(*r, *c))
                && plain
                    .screen
                    .cell(*r, *c)
                    .is_some_and(|cell| cell.contents() == "█")
                && [
                    (r.wrapping_sub(1), *c),
                    (r + 1, *c),
                    (*r, c.wrapping_sub(1)),
                    (*r, c + 1),
                ]
                .iter()
                .all(|n| !any_node.contains(n))
        })
        .collect();
    assert!(
        ribbons.len() > 20,
        "the chart draws ribbons:\n{}",
        plain.text
    );
    let empty = (0..size.1)
        .flat_map(|r| (0..size.0).map(move |c| (r, c)))
        .find(|(r, c)| {
            plain
                .screen
                .cell(*r, *c)
                .is_some_and(|cell| cell.contents().trim().is_empty())
        })
        .expect("an empty cell");
    for (r, c) in [
        ribbons[0],
        ribbons[ribbons.len() / 2],
        ribbons[ribbons.len() - 1],
        empty,
    ] {
        let frame = radial_run(sankey_chart(size), size, vec![(2, hover(c, r)), (3, None)])
            .pop()
            .unwrap();
        assert_eq!(
            (announced(&frame), &frame.text),
            (None, &plain.text),
            "a pointer at ({c}, {r}), off every node, must select nothing"
        );
    }
}

/// Whether a cell shows `rgb` as a shape color: its background, or the
/// foreground of a non-blank glyph.
fn shows_rgb(frame: &Snapshot, r: u16, c: u16, rgb: (u8, u8, u8)) -> bool {
    let near = |color: vt100::Color| {
        matches!(color, vt100::Color::Rgb(red, green, blue)
            if red.abs_diff(rgb.0) <= 2 && green.abs_diff(rgb.1) <= 2 && blue.abs_diff(rgb.2) <= 2)
    };
    frame.screen.cell(r, c).is_some_and(|cell| {
        near(cell.bgcolor()) || (!cell.contents().trim().is_empty() && near(cell.fgcolor()))
    })
}

/// CHT-032: every cell that shows a node, the partly covered rows at its
/// top and bottom included, selects that node.
#[test]
fn cht_032_every_drawn_node_cell_selects_its_node() {
    let size = (80u16, 24u16);
    let plain = radial_run(sankey_chart(size), size, vec![(2, None)])
        .pop()
        .unwrap();
    let mut probed = 0;
    for (index, name) in SANKEY_NAMES.iter().enumerate() {
        let rgb = node_rgb(index);
        let cells: Vec<(u16, u16)> = (0..size.1)
            .flat_map(|r| (0..size.0).map(move |c| (r, c)))
            .filter(|(r, c)| shows_rgb(&plain, *r, *c, rgb))
            .collect();
        assert!(!cells.is_empty(), "node {name} is drawn:\n{}", plain.text);
        for (r, c) in cells {
            probed += 1;
            let frame = radial_run(sankey_chart(size), size, vec![(2, hover(c, r)), (3, None)])
                .pop()
                .unwrap();
            assert_eq!(
                announced(&frame),
                Some(index),
                "({c}, {r}) shows {name} and must select it; live region {:?}:\n{}",
                frame.live,
                plain.text
            );
        }
    }
    assert!(probed > 20, "every node's cells are probed ({probed})");
}

/// CHT-032: a Sankey chart built through the untyped ChartsBuilder, whose
/// node points carry no throughput, still names each selected node's
/// throughput in the tooltip and the live region.
#[test]
fn cht_032_the_untyped_builder_announces_the_throughput() {
    use reactive_tui::widgets::display::{ChartsBuilder, SankeyOptions};
    let size = (80u16, 24u16);
    let untyped = || {
        let nodes = DataSeries::new(
            "nodes",
            SANKEY_NAMES
                .iter()
                .map(|name| DataPoint::with_label(0.0, *name))
                .collect(),
        );
        ChartsBuilder::sankey()
            .series(nodes)
            .sankey_options(SankeyOptions {
                links: [(0, 2, 3.0), (1, 2, 2.0), (2, 3, 4.0), (2, 4, 1.0)]
                    .map(|(s, t, v)| SankeyLink::new(s, t, v))
                    .to_vec(),
                ..SankeyOptions::default()
            })
            .size(size.0, size.1)
            .build()
    };
    let plain = radial_run(untyped(), size, vec![(2, None)]).pop().unwrap();
    for (index, name) in SANKEY_NAMES.iter().enumerate() {
        let cells = node_cells(&plain, index);
        let (r, c) = cells[cells.len() / 2];
        let frame = radial_run(untyped(), size, vec![(2, hover(c, r)), (3, None)])
            .pop()
            .unwrap();
        assert_eq!(
            announced(&frame),
            Some(index),
            "{name} must be announced with its throughput {}; live region {:?}",
            SANKEY_THROUGHPUT[index],
            frame.live
        );
    }
}

/// CHT-032: the tooltip sits beside the selected node, right of it or, at
/// the right edge, left of it, and never over the node itself.
#[test]
fn cht_032_the_tooltip_sits_beside_its_node_not_over_it() {
    let size = (80u16, 24u16);
    let plain = radial_run(sankey_chart(size), size, vec![(2, None)])
        .pop()
        .unwrap();
    for (index, name) in SANKEY_NAMES.iter().enumerate() {
        let rgb = node_rgb(index);
        let cells: Vec<(u16, u16)> = (0..size.1)
            .flat_map(|r| (0..size.0).map(move |c| (r, c)))
            .filter(|(r, c)| shows_rgb(&plain, *r, *c, rgb))
            .collect();
        let (r, c) = node_cells(&plain, index)[0];
        let frame = radial_run(sankey_chart(size), size, vec![(2, hover(c, r)), (3, None)])
            .pop()
            .unwrap();
        assert_eq!(announced(&frame), Some(index));
        let covered: Vec<&(u16, u16)> = cells
            .iter()
            .filter(|(r, c)| !shows_rgb(&frame, *r, *c, rgb))
            .collect();
        assert!(
            covered.is_empty(),
            "the tooltip of {name} must not cover its node: {covered:?}\n{}",
            frame.text
        );
    }
}

/// CHT-032: while the reveal has not drawn a node yet, neither a key nor
/// the pointer selects it, and its label does not show.
#[test]
fn cht_032_nodes_the_reveal_has_not_drawn_are_not_selectable() {
    let size = (80u16, 24u16);
    let plain = radial_run(sankey_chart(size), size, vec![(2, None)])
        .pop()
        .unwrap();
    // A ten-minute reveal draws nothing in the frames the test sees.
    let revealing = || {
        let mut p = sankey_chart(size);
        p.animated = true;
        p.animation_duration = 600_000;
        p
    };
    let (r, c) = node_cells(&plain, 3)[0];
    for (how, events) in [
        ("End", vec![(2, app_input::key(KeyCode::End)), (3, None)]),
        ("the pointer", vec![(2, hover(c, r)), (3, None)]),
    ] {
        let frame = radial_run(revealing(), size, events).pop().unwrap();
        assert_eq!(
            announced(&frame),
            None,
            "{how} must not select a node the reveal has not drawn:\n{}",
            frame.text
        );
        assert!(
            !SANKEY_NAMES.iter().any(|name| frame.text.contains(name)),
            "no label shows before its node is drawn:\n{}",
            frame.text
        );
    }
}

/// CHT-032: Left, Right, Home and End step through the nodes column by
/// column and, within a column, from top to bottom.
#[test]
fn cht_032_keys_step_through_the_nodes_by_layer_then_top_to_bottom() {
    let size = (80u16, 24u16);
    let plain = radial_run(sankey_chart(size), size, vec![(2, None)])
        .pop()
        .unwrap();
    let top = |index: usize| {
        node_cells(&plain, index)
            .iter()
            .map(|(r, _)| *r)
            .min()
            .expect("node drawn")
    };
    let mut order = Vec::new();
    for layer in [vec![0, 1], vec![2], vec![3, 4]] {
        let mut layer = layer;
        layer.sort_by_key(|i| top(*i));
        order.extend(layer);
    }
    let selected = |keys: &[KeyCode]| {
        let mut events: Vec<(usize, Option<Event>)> = keys
            .iter()
            .enumerate()
            .map(|(i, k)| (i + 2, app_input::key(k.clone())))
            .collect();
        events.push((keys.len() + 2, None));
        announced(&radial_run(sankey_chart(size), size, events).pop().unwrap())
    };
    for step in 0..order.len() {
        let keys = vec![KeyCode::Right; step + 1];
        assert_eq!(
            selected(&keys),
            Some(order[step]),
            "Right {} times must select node {} of the order {order:?}",
            step + 1,
            SANKEY_NAMES[order[step]]
        );
    }
    assert_eq!(
        selected(&[KeyCode::End]),
        Some(order[4]),
        "End selects the last node"
    );
    assert_eq!(
        selected(&[KeyCode::End, KeyCode::Left]),
        Some(order[3]),
        "Left steps back"
    );
    assert_eq!(
        selected(&[KeyCode::End, KeyCode::Home]),
        Some(order[0]),
        "Home selects the first node"
    );
}

/// CHT-032: a selection keeps the selected node's links at the link
/// opacity and fades every other link, and a second pointer move over the
/// same node leaves the frame unchanged (CHT-019's same-index rule).
#[test]
fn cht_032_a_selection_fades_the_other_links_and_keeps_its_own() {
    let size = (80u16, 24u16);
    let plain = radial_run(sankey_chart(size), size, vec![(2, None)])
        .pop()
        .unwrap();
    let hub = node_cells(&plain, 2);
    let (hub_left, hub_right) = (
        hub.iter().map(|(_, c)| *c).min().unwrap() - 1,
        hub.iter().map(|(_, c)| *c).max().unwrap() + 1,
    );
    let hub_rows: Vec<u16> = {
        let mut rows: Vec<u16> = hub.iter().map(|(r, _)| *r).collect();
        rows.sort_unstable();
        rows.dedup();
        rows
    };
    let changed = |frame: &Snapshot, c: u16| -> Vec<bool> {
        hub_rows
            .iter()
            .map(|r| colors_at(frame, *r, c) != colors_at(&plain, *r, c))
            .collect()
    };
    // North selected: its link into the hub keeps its colors, south's link
    // into the hub and the hub's links out fade.
    let north = node_cells(&plain, 0);
    let (r, c) = north[north.len() / 2];
    let selected = radial_run(sankey_chart(size), size, vec![(2, hover(c, r)), (3, None)])
        .pop()
        .unwrap();
    assert_eq!(announced(&selected), Some(0));
    let left = changed(&selected, hub_left);
    assert!(
        left.iter().any(|c| *c) && left.iter().any(|c| !*c),
        "beside the hub, north's link must keep its colors and south's must fade: {left:?}\nbefore\n{}\nafter\n{}",
        plain.text,
        selected.text
    );
    assert!(
        changed(&selected, hub_right).iter().all(|c| *c),
        "the hub's links out, which north does not touch, must fade:\nbefore\n{}\nafter\n{}",
        plain.text,
        selected.text
    );
    // Hub selected: every link is the hub's, so none fades beside the
    // sources.
    let (r, c) = hub[hub.len() / 2];
    let hub_frame = radial_run(sankey_chart(size), size, vec![(2, hover(c, r)), (3, None)])
        .pop()
        .unwrap();
    assert_eq!(announced(&hub_frame), Some(2));
    for source in [0, 1] {
        let cells = node_cells(&plain, source);
        let right = cells.iter().map(|(_, c)| *c).max().unwrap() + 1;
        for (r, _) in &cells {
            assert_eq!(
                colors_at(&hub_frame, *r, right),
                colors_at(&plain, *r, right),
                "selecting the hub must not fade its own link from {} at ({right}, {r})",
                SANKEY_NAMES[source]
            );
        }
    }
    // A second move over the same node changes nothing.
    let (r2, c2) = hub[0];
    let again = radial_run(
        sankey_chart(size),
        size,
        vec![(2, hover(c, r)), (3, hover(c2, r2)), (4, None)],
    )
    .pop()
    .unwrap();
    assert_eq!(
        again.text, hub_frame.text,
        "a move within the selected node must leave the frame unchanged"
    );
}

/// The theme's color for `token` as 8-bit channels.
fn role_rgb(token: &str) -> (u8, u8, u8) {
    let (r, g, b, _) = reactive_tui::theme::Theme::active()
        .resolve_color(token)
        .expect("theme color");
    let c = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    (c(r), c(g), c(b))
}

/// Whether a terminal color is `rgb`, allowing one rounding step.
fn rgb_near(color: vt100::Color, rgb: (u8, u8, u8)) -> bool {
    match color {
        vt100::Color::Rgb(r, g, b) => {
            r.abs_diff(rgb.0) <= 2 && g.abs_diff(rgb.1) <= 2 && b.abs_diff(rgb.2) <= 2
        }
        _ => false,
    }
}

/// Whether the cell at (`row`, `col`) shows `rgb` as its glyph or background.
fn cell_near(frame: &Snapshot, row: u16, col: u16, rgb: (u8, u8, u8)) -> bool {
    frame
        .screen
        .cell(row, col)
        .is_some_and(|cell| rgb_near(cell.fgcolor(), rgb) || rgb_near(cell.bgcolor(), rgb))
}

/// The cells of `frame` whose glyph satisfies `pick`, as (row, column).
fn cells_with(frame: &Snapshot, pick: impl Fn(&str) -> bool) -> Vec<(u16, u16)> {
    let (rows, cols) = frame.screen.size();
    (0..rows)
        .flat_map(|r| (0..cols).map(move |c| (r, c)))
        .filter(|(r, c)| {
            frame
                .screen
                .cell(*r, *c)
                .is_some_and(|cell| pick(cell.contents()))
        })
        .collect()
}

/// The topmost row of `frame` holding a cell that shows `rgb`.
fn top_row_showing(frame: &Snapshot, rgb: (u8, u8, u8)) -> Option<u16> {
    let (rows, cols) = frame.screen.size();
    (0..rows).find(|r| (0..cols).any(|c| cell_near(frame, *r, c, rgb)))
}

/// CHT-011: the automatic domain covers stacked totals, so the upper series
/// of a stacked bar is drawn above the lower one instead of being clipped
/// by a domain that ends at the single values.
#[test]
fn cht_011_stacked_totals_extend_the_automatic_domain() {
    let size = (20u16, 12u16);
    let mut p = props(ChartType::BarVertical, size, &[8.0]);
    p.series.push(DataSeries::new(
        "upper",
        vec![DataPoint::with_label(8.0, "p0")],
    ));
    p.stacked = true;
    p.y_axis.min = None;
    p.y_axis.max = None;
    let frame = app_input::run_when_painted(Root(Element::typed::<Chart>(p)), size, 2)
        .pop()
        .unwrap();
    let lower = top_row_showing(&frame, role_rgb("chart-1"));
    let upper = top_row_showing(&frame, role_rgb("chart-2"));
    assert!(
        lower.is_some(),
        "the lower series is drawn:\n{}",
        frame.text
    );
    assert!(
        upper.is_some(),
        "the upper series of a stacked bar must be drawn, not clipped away:\n{}",
        frame.text
    );
    assert!(
        upper < lower && lower.unwrap() >= size.1 / 3,
        "with two stacked series of 8 the axis ends at 16: the lower series reaches the middle (its top at row {:?} of {}) and the upper one sits above it (top at row {:?}):\n{}",
        lower,
        size.1,
        upper,
        frame.text
    );
}

/// CHT-034: `tick_margin` thins the category labels on both orientations.
#[test]
fn cht_034_tick_margin_thins_category_labels_on_either_orientation() {
    let size = (40u16, 20u16);
    let values: Vec<f64> = (0..8).map(|i| i as f64 + 1.0).collect();
    let shown = |kind: ChartType| {
        let mut p = props(kind, size, &values);
        p.tick_margin = 2;
        p.x_axis.show_labels = true;
        p.y_axis.show_labels = true;
        let frame = app_input::run_when_painted(Root(Element::typed::<Chart>(p)), size, 2)
            .pop()
            .unwrap();
        let labels = (0..8)
            .filter(|i| frame.text.contains(&format!("p{i}")))
            .count();
        (labels, frame.text)
    };
    let (vertical, text) = shown(ChartType::BarVertical);
    assert!(
        (2..=4).contains(&vertical),
        "a vertical chart with tick_margin 2 shows every second category label, got {vertical} of 8:\n{text}"
    );
    let (horizontal, text) = shown(ChartType::BarHorizontal);
    assert!(
        (2..=4).contains(&horizontal),
        "a horizontal chart with tick_margin 2 shows every second category label, got {horizontal} of 8:\n{text}"
    );
}

/// CHT-018: a chart too short for every series row summarizes the rows that
/// do not fit instead of clipping a series away: six series in an eight-row
/// chart show a title, four rows and "+2 more".
#[test]
fn cht_018_a_short_chart_summarizes_the_tooltip_rows_that_do_not_fit() {
    let size = (80u16, 8u16);
    let mut p = props(ChartType::Line, size, &[2.0, 5.0, 8.0]);
    for i in 1..6 {
        let mut extra = series(&[1.0 + i as f64, 4.0, 7.0]);
        extra.name = format!("s{i}");
        p.series.push(extra);
    }
    let frame = app_input::run(
        Root(Element::typed::<Chart>(p)),
        size,
        vec![(2, hover(40, 4)), (3, None)],
    )
    .pop()
    .unwrap();
    assert!(
        frame.text.contains("+2 more"),
        "the tooltip summarizes the two rows the eight-row chart cannot show:\n{}",
        frame.text
    );
    assert!(
        frame.text.contains("s1") || frame.text.contains("s2"),
        "the rows that fit are shown:\n{}",
        frame.text
    );
}

/// CHT-034: a tick count on the value axis is the number of ticks drawn,
/// grid lines and labels alike: three over a pinned 0 to 8 gives 0, 4 and 8
/// with grid lines at 4 and 8 above the axis, never five ticks, and five over
/// automatic data landing on 10 gives five round labels.
#[test]
fn cht_034_a_tick_count_draws_that_many_grid_lines_and_labels() {
    // The large class, which draws the grid (CHT-024).
    let size = (200u16, 40u16);
    let grid_rows = |frame: &Snapshot| {
        let mut rows: Vec<u16> = cells_with(frame, |glyph| glyph == "·")
            .into_iter()
            .map(|(row, _)| row)
            .collect();
        rows.dedup();
        rows.len()
    };
    let mut pinned = props(ChartType::Line, size, &[2.0, 7.0, 5.0]);
    pinned.y_axis.min = Some(0.0);
    pinned.y_axis.max = Some(8.0);
    pinned.y_axis.tick_count = 3;
    pinned.y_axis.show_labels = true;
    pinned.y_axis.show_grid = true;
    let frame = app_input::run_when_painted(Root(Element::typed::<Chart>(pinned)), size, 2)
        .pop()
        .unwrap();
    // The lowest tick's line is the axis itself at the plot's bottom edge,
    // so the grid holds one row per tick above it: two for three ticks,
    // not four.
    assert_eq!(
        grid_rows(&frame),
        2,
        "a tick count of 3 draws grid lines at two ticks above the axis, not four:\n{}",
        frame.text
    );
    for label in ["0", "4", "8"] {
        assert!(
            cells_with(&frame, |glyph| glyph == label)
                .iter()
                .any(|(_, col)| *col < 4),
            "the tick label {label} is in the gutter:\n{}",
            frame.text
        );
    }
    let mut auto = props(ChartType::Line, size, &[2.0, 10.0, 5.0]);
    auto.y_axis.tick_count = 5;
    auto.y_axis.show_labels = true;
    auto.y_axis.show_grid = true;
    let frame = app_input::run_when_painted(Root(Element::typed::<Chart>(auto)), size, 2)
        .pop()
        .unwrap();
    assert_eq!(
        grid_rows(&frame),
        4,
        "a tick count of 5 draws grid lines at the four ticks above the axis:\n{}",
        frame.text
    );
    for label in ["2.5", "7.5", "10"] {
        assert!(
            frame.text.contains(label),
            "the automatic range lands five ticks on round values ({label}):\n{}",
            frame.text
        );
    }
}

/// CHT-013, CHT-035: a bar series' `fill` is the color of its bars, which
/// the generic route's `.fill()` sets like the typed route's, instead of
/// the palette color.
#[test]
fn cht_013_a_bar_series_fill_colors_its_bars() {
    let size = (30u16, 10u16);
    let mut p = props(ChartType::BarVertical, size, &[4.0, 8.0, 6.0]);
    p.series[0].fill = Some("chart-3".into());
    let frame = app_input::run_when_painted(Root(Element::typed::<Chart>(p)), size, 2)
        .pop()
        .unwrap();
    let fill = role_rgb("chart-3");
    let palette = role_rgb("chart-1");
    let blocks = cells_with(&frame, |glyph| glyph == "█");
    assert!(!blocks.is_empty(), "bars are drawn:\n{}", frame.text);
    assert!(
        blocks
            .iter()
            .all(|(row, col)| cell_near(&frame, *row, *col, fill)),
        "every bar cell takes the series fill chart-3 {fill:?}:\n{}",
        frame.text
    );
    assert!(
        !blocks
            .iter()
            .any(|(row, col)| cell_near(&frame, *row, *col, palette)),
        "no bar cell keeps the palette color chart-1 {palette:?}:\n{}",
        frame.text
    );
}

/// CHT-017: the chart's axes take the `border` role and its tick labels the
/// `text-muted` role, not whatever color the element inherits.
#[test]
fn cht_017_axes_take_the_border_role_and_tick_labels_the_muted_role() {
    let size = (40u16, 14u16);
    let mut p = props(ChartType::Line, size, &[2.0, 8.0, 5.0]);
    p.x_axis.show_labels = true;
    p.y_axis.show_labels = true;
    let frame = app_input::run_when_painted(Root(Element::typed::<Chart>(p)), size, 2)
        .pop()
        .unwrap();
    let border = role_rgb("border");
    let muted = role_rgb("text-muted");
    let axis_cells = cells_with(&frame, |glyph| {
        matches!(
            glyph,
            "─" | "│" | "└" | "┘" | "┌" | "┐" | "├" | "┤" | "┴" | "┬" | "┼"
        )
    });
    assert!(
        !axis_cells.is_empty(),
        "the chart draws axis lines:\n{}",
        frame.text
    );
    let in_border = axis_cells
        .iter()
        .filter(|(r, c)| cell_near(&frame, *r, *c, border))
        .count();
    assert_eq!(
        in_border,
        axis_cells.len(),
        "every axis cell takes the border role {border:?}; {in_border} of {} do:\n{}",
        axis_cells.len(),
        frame.text
    );
    let label_cells = cells_with(&frame, |glyph| glyph == "p");
    assert!(
        !label_cells.is_empty(),
        "tick labels present:\n{}",
        frame.text
    );
    assert!(
        label_cells
            .iter()
            .all(|(r, c)| cell_near(&frame, *r, *c, muted)),
        "tick labels take the text-muted role {muted:?}:\n{}",
        frame.text
    );
}

/// CHT-018: the tooltip box is opaque on the `surface` role, opens with a
/// title row naming the category, and shows a candle as four rows.
#[test]
fn cht_018_tooltip_is_opaque_on_surface_with_a_title_row_and_four_candle_rows() {
    let size = (40u16, 14u16);
    let mut p = props(ChartType::Line, size, &[2.0, 8.0, 5.0]);
    p.series.push(DataSeries::new(
        "second",
        [1.0, 4.0, 9.0]
            .iter()
            .enumerate()
            .map(|(i, v)| DataPoint::with_label(*v, format!("q{i}")))
            .collect(),
    ));
    let frame = app_input::run(
        Root(Element::typed::<Chart>(p)),
        size,
        vec![(2, hover(20, 7)), (3, None)],
    )
    .pop()
    .unwrap();
    let lines: Vec<Vec<char>> = frame.text.lines().map(|l| l.chars().collect()).collect();
    let corner = |row: &Vec<char>, which: &str| row.iter().position(|c| which.contains(*c));
    let top = lines
        .iter()
        .position(|l| corner(l, "╭┌").is_some())
        .expect("tooltip top border");
    let bottom = lines
        .iter()
        .rposition(|l| corner(l, "╰└").is_some())
        .expect("tooltip bottom border");
    let left = corner(&lines[top], "╭┌").unwrap();
    let right = lines[top]
        .iter()
        .rposition(|c| "╮┐".contains(*c))
        .expect("tooltip right corner");
    let surface = role_rgb("surface");
    for row in top..=bottom {
        for col in left..=right {
            let bg = frame
                .screen
                .cell(row as u16, col as u16)
                .map(|c| c.bgcolor());
            assert!(
                bg.is_some_and(|bg| rgb_near(bg, surface)),
                "tooltip cell ({row},{col}) must be opaque on the surface role {surface:?}, found {bg:?}:\n{}",
                frame.text
            );
        }
    }
    let inner: Vec<String> = lines[top + 1..bottom]
        .iter()
        .map(|l| l[left..=right].iter().collect())
        .collect();
    assert!(
        inner[0].contains("p1") && !inner[0].contains('■'),
        "the first row is the category title without a swatch, got {:?}:\n{}",
        inner[0],
        frame.text
    );
    assert!(
        inner[1..]
            .iter()
            .all(|r| !r.contains("p1:") && !r.contains("q1:")),
        "series rows do not repeat the category, got {:?}:\n{}",
        &inner[1..],
        frame.text
    );
    let mut c = props(ChartType::Candlestick, size, &[]);
    c.series = vec![candles(&[3.0, 6.0, 4.0])];
    let frame = app_input::run(
        Root(Element::typed::<Chart>(c)),
        size,
        vec![(2, hover(20, 7)), (3, None)],
    )
    .pop()
    .unwrap();
    let text = frame.text.to_lowercase();
    let rows: Vec<usize> = ["open", "high", "low", "close"]
        .iter()
        .filter_map(|word| text.lines().position(|l| l.contains(word)))
        .collect();
    let mut distinct = rows.clone();
    distinct.sort_unstable();
    distinct.dedup();
    assert_eq!(
        (rows.len(), distinct.len()),
        (4, 4),
        "a candle's tooltip has its own open, high, low and close rows, found rows {rows:?}:\n{}",
        frame.text
    );
}

/// CHT-036: Up and Down choose the series the announcement follows, so the
/// keys reach a scatter's second series.
#[test]
fn cht_036_down_moves_the_selection_to_the_next_series() {
    let size = (40u16, 14u16);
    let named = |name: &str, values: &[f64]| {
        DataSeries::new(
            name,
            values
                .iter()
                .enumerate()
                .map(|(i, v)| DataPoint::with_label(*v, format!("{name}{i}")))
                .collect(),
        )
    };
    let mut p = props(ChartType::Scatter, size, &[]);
    p.series = vec![
        named("first", &[2.0, 8.0, 5.0]),
        named("second", &[1.0, 4.0, 9.0]),
    ];
    // Home after frame 2, then the rest of the keys back to back after frame
    // 3, then the frame they produce. End always moves the index, so a frame
    // follows even while Down is ignored.
    let announced = |then: &[KeyCode]| {
        let mut steps: Vec<(usize, Option<Event>)> = vec![(2, app_input::key(KeyCode::Home))];
        steps.extend(then.iter().map(|k| (3, app_input::key(k.clone()))));
        steps.push((4, None));
        app_input::run(
            Root(Element::typed::<Chart>(p.clone()).auto_focus()),
            size,
            steps,
        )
        .pop()
        .unwrap()
        .live
        .join(" ")
    };
    let first = announced(&[KeyCode::End]);
    assert!(
        first.contains("first"),
        "Home then End selects the first series' last point and announces it: {first:?}"
    );
    let second = announced(&[KeyCode::Down, KeyCode::End]);
    assert!(
        second != first && second.contains("second"),
        "Down must move the selection to the second series, so End then lands on its last point: before {first:?}, after {second:?}"
    );
}
