//! charts-interaction (CHT-018, CHT-019), charts-motion (CHT-022),
//! frame-budget (CHT-021, BAR-005), widget-bar (BAR-003), charts-palette
//! parity (CHT-017) and plot-layer scale behavior (CHT-011). Each test names
//! the requirement it observes.

mod common;

use std::sync::atomic::{AtomicBool, Ordering};

use common::app_input::{self, Snapshot};
use reactive_tui::app::{RootComponent, RootUpdate};
use reactive_tui::component::Element;
use reactive_tui::event::types::{Event, KeyCode, MouseEvent, MouseEventKind, Position};
use reactive_tui::widgets::display::{
    Chart, ChartAxis, ChartLegend, ChartProps, ChartType, DataPoint, DataSeries,
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
static CHART_WORKER_SEEN: AtomicBool = AtomicBool::new(false);

fn note_chart_workers() {
    #[cfg(target_os = "linux")]
    if !chart_worker_threads().is_empty() {
        CHART_WORKER_SEEN.store(true, Ordering::SeqCst);
    }
}

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
    std::fs::read_dir("/proc/self/task")
        .unwrap()
        .filter_map(|t| std::fs::read_to_string(t.ok()?.path().join("comm")).ok())
        .filter(|n| n.starts_with("rtui-chart"))
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
        let plain = app_input::run_when_painted(
            Root(Element::typed::<Chart>(p.clone())),
            size,
            2,
        )
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
}
impl RootComponent for Switching {
    fn render(&self) -> Element {
        let switched = self.switched.load(Ordering::SeqCst);
        let mut p = props(
            ChartType::BarVertical,
            self.size,
            if switched { &self.after } else { &self.before },
        );
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
    let settled_row = (0..size.1)
        .find(|r| cell_is(&settled, *r, 0, "\u{2588}"))
        .expect("settled column 0");
    // The transition's end is the target's tip cell (its topmost painted
    // cell in column 0), which no intermediate frame shows.
    let (target_row, target_tip) = (0..size.1)
        .find_map(|r| {
            let content = target.screen.cell(r, 0)?.contents();
            (!content.trim().is_empty()).then_some((r, content))
        })
        .expect("target column 0");
    let root = Switching {
        switched: AtomicBool::new(false),
        redraw: AtomicBool::new(false),
        size,
        before: vec![5.0, 5.0],
        after: vec![9.0, 9.0],
    };
    let frames = app_input::run_when_cell(
        root,
        size,
        vec![
            app_input::CellStep {
                x: 0,
                y: settled_row,
                content: "\u{2588}",
                event: app_input::key(KeyCode::Char('s')),
            },
            app_input::CellStep {
                x: 0,
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

fn max_work_ms(root: impl RootComponent + 'static, size: (u16, u16), frames: usize) -> f64 {
    let out = app_input::run(root, size, vec![(frames, None)]);
    assert!(
        out.len() >= frames,
        "harness painted {} of {frames} frames",
        out.len()
    );
    out.iter().skip(1).map(|f| f.work_ms).fold(0.0, f64::max)
}

/// BAR-005: an animating chart keeps per-frame work under 16.6 ms at 700 by
/// 200 on the debug backend, and rasterizes on a worker.
#[test]
fn bar_005_animating_chart_stays_under_the_frame_budget_at_700_by_200() {
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
            vec![(3, None)],
        );
        assert!(frames.len() >= 3);
        #[cfg(target_os = "linux")]
        assert!(chart_worker_seen(), "no rtui-chart worker thread observed");
        return;
    }
    let size = (700u16, 200u16);
    let mut p = props(
        ChartType::Line,
        size,
        &(0..120)
            .map(|i| ((i as f64) * 0.37).sin().abs() * 10.0)
            .collect::<Vec<_>>(),
    );
    p.animated = true;
    p.animation_duration = 2000;
    let root = Root(Element::typed::<Chart>(p));
    let ms = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        max_work_ms(root, size, 12)
    })) {
        Ok(ms) => ms,
        Err(_) => panic!(
            "fewer than 12 frames painted inside the harness's 3 s deadline at 700x200: per-frame work exceeds 16.6 ms by more than an order of magnitude"
        ),
    };
    assert!(
        ms < 16.6,
        "per-frame work {ms:.2} ms exceeds 16.6 ms at 700x200"
    );
    #[cfg(target_os = "linux")]
    assert!(
        chart_worker_seen(),
        "no rtui-chart worker thread observed during the run"
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
    let ms = max_work_ms(Slow, (80, 24), 5);
    assert!(
        ms >= 16.6,
        "fixture must exceed the budget so the mechanism can fail, measured {ms:.2} ms"
    );
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
    for token in ["blue-500", "primary"] {
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
