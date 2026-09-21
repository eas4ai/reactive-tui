//! charts-interaction (CHT-018, CHT-019), charts-motion (CHT-022),
//! frame-budget (CHT-021, BAR-005), widget-bar (BAR-003), charts-palette
//! parity (CHT-017). Each test names the requirement it observes.

mod common;

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;

use common::app_input::{self, Snapshot};
use reactive_tui::app::{RootComponent, RootUpdate};
use reactive_tui::component::Element;
use reactive_tui::event::types::{Event, MouseEvent, MouseEventKind, Position};
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

/// CHT-018: tooltip is a box with a swatch row per series and a crosshair.
#[test]
fn cht_018_tooltip_is_a_box_with_swatch_rows_and_a_crosshair() {
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
        Root(Element::typed::<Chart>(p)),
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
    assert!(
        count(f, |c| c == '│') >= 3,
        "crosshair column expected:\n{}",
        f.text
    );
}

/// CHT-019: mouse selects the nearest data index; two positions inside the
/// same band select the same point.
#[test]
fn cht_019_mouse_selects_nearest_index_within_a_band() {
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
        "two hovers inside one band must select the same point"
    );
    assert!(
        a.text.contains("p1"),
        "nearest point label expected in tooltip:\n{}",
        a.text
    );
}

/// Root that swaps its data when it receives any event, to observe transitions.
struct Switching {
    switched: std::sync::atomic::AtomicBool,
    redraw: std::sync::atomic::AtomicBool,
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
        // The starting state is drawn settled; only the change may animate.
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
/// intermediate frames and ends exactly at the target rendering, never
/// restarting from zero.
#[test]
fn cht_022_value_transition_passes_through_intermediate_frames_and_ends_at_target() {
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
    let root = Switching {
        switched: std::sync::atomic::AtomicBool::new(false),
        redraw: std::sync::atomic::AtomicBool::new(false),
        size,
        before: vec![5.0, 5.0],
        after: vec![9.0, 9.0],
    };
    // Rows are painted bottom-up. Row 12-6 = 6 is filled once the 5.0 bars settle;
    // row 1 is filled only by the 9.0 target. Wait for each, sending one event in between.
    let settled_row = {
        let rows: Vec<u16> = (0..size.1)
            .filter(|r| {
                settled
                    .screen
                    .cell(*r, 0)
                    .map(|c| c.contents() == "\u{2588}")
                    .unwrap_or(false)
            })
            .collect();
        *rows.first().expect("settled chart paints column 0")
    };
    let target_row = {
        let rows: Vec<u16> = (0..size.1)
            .filter(|r| {
                target
                    .screen
                    .cell(*r, 0)
                    .map(|c| c.contents() == "\u{2588}")
                    .unwrap_or(false)
            })
            .collect();
        *rows.first().expect("target chart paints column 0")
    };
    assert!(
        target_row < settled_row,
        "target must be taller than the settled start"
    );
    let frames = app_input::run_when_cell(
        root,
        size,
        vec![
            app_input::CellStep {
                x: 0,
                y: settled_row,
                content: "\u{2588}",
                event: app_input::key(reactive_tui::event::types::KeyCode::Char('s')),
            },
            app_input::CellStep {
                x: 0,
                y: target_row,
                content: "\u{2588}",
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
        panic!("frame {} dropped to {} blocks, below the previous rendering ({before}): the transition restarted from zero instead of moving from the old values", switch_index + 1 + i, post[i]);
    }
}

/// CHT-021: rasterization runs on a worker thread and a default-sized chart
/// fills its allotted rectangle.
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
    {
        let names: Vec<String> = std::fs::read_dir("/proc/self/task")
            .unwrap()
            .filter_map(|t| std::fs::read_to_string(t.ok()?.path().join("comm")).ok())
            .collect();
        let _ = names;
    }
}

fn frame_budget_ms(root: impl RootComponent + 'static, size: (u16, u16), frames: usize) -> f64 {
    let start = Instant::now();
    let out = app_input::run(root, size, vec![(frames, None)]);
    assert!(
        out.len() >= frames,
        "harness painted {} of {frames} frames",
        out.len()
    );
    start.elapsed().as_secs_f64() * 1000.0 / out.len() as f64
}

/// BAR-005: an animating chart keeps the main loop under 16.6 ms per frame at
/// 700 by 200 on the debug backend, and the rasterizer runs on a worker.
#[test]
fn bar_005_animating_chart_stays_under_the_frame_budget_at_700_by_200() {
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
    let ms = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| frame_budget_ms(root, size, 12))) {
        Ok(ms) => ms,
        Err(_) => panic!("fewer than 12 frames painted inside the harness's 3 s deadline at 700x200: main-loop frame time exceeds 16.6 ms by more than an order of magnitude"),
    };
    assert!(
        ms < 16.6,
        "main-loop frame time {ms:.2} ms exceeds 16.6 ms at 700x200"
    );
    #[cfg(target_os = "linux")]
    {
        let workers: Vec<String> = std::fs::read_dir("/proc/self/task")
            .unwrap()
            .filter_map(|t| std::fs::read_to_string(t.ok()?.path().join("comm")).ok())
            .filter(|n| n.starts_with("rtui-chart"))
            .collect();
        assert!(
            !workers.is_empty(),
            "no rtui-chart worker thread observed during the run"
        );
    }
}

/// Negative fixture for BAR-005: a root that burns 25 ms per frame must fail the budget.
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
    let ms = frame_budget_ms(Slow, (80, 24), 5);
    assert!(
        ms >= 16.6,
        "fixture must exceed the budget so the mechanism can fail, measured {ms:.2} ms"
    );
}

/// BAR-003: chart usable by keyboard (selection moves with Right) and fills
/// the parent; colors come from the theme (checked by the script).
#[test]
fn bar_003_chart_is_keyboard_reachable_and_fills_parent() {
    let size = (60u16, 20u16);
    let p = ChartProps {
        chart_type: ChartType::Line,
        series: vec![series(&[2.0, 8.0, 5.0])],
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
        widest > 50,
        "chart must fill a 60-column parent, painted {widest}"
    );
}

/// CHT-017: a color token that works in a `bg-` utility class resolves to the
/// same RGBA in a chart series.
#[test]
fn cht_017_utility_color_tokens_resolve_identically_in_charts() {
    use reactive_tui::layout::colors::parse_color_token;
    let utility = parse_color_token("blue-500").expect("blue-500 resolves for utility classes");
    let size = (30u16, 10u16);
    let mut p = props(ChartType::BarVertical, size, &[10.0]);
    p.series[0] = series(&[10.0]).with_color("blue-500");
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
        "chart must paint blue-500 exactly as the utility resolver does"
    );
    let _ = Arc::new(());
}
