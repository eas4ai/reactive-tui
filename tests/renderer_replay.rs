//! render-replay mechanism: RAS-004.
//!
//! Every renderer test frame and every charts golden is rendered through the
//! real SuprTUI backend, its byte stream is replayed through the vt100 parser
//! model and, with the `embedded-terminal` feature, Ghostty's parser, and the
//! screens (text plus a digest of every cell's colors) must match the ones
//! recorded in `tests/snapshots/renderer/<name>.screen` before the rasterizer
//! changed. A frame with no recording fails. Run with `REGENERATE=1` only to
//! record screens produced by a rasterizer known to be correct.

mod common;

use common::app_input;
use reactive_tui::app::RootComponent;
use reactive_tui::component::{Element, LayoutType};
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

/// The element shapes tests/suprtui_renderer.rs exercises, one frame each.
fn renderer_frames() -> Vec<(&'static str, (u16, u16), Element)> {
    let styled = |text: &str| {
        Element::layout(LayoutType::Flex)
            .with_class("flex flex-col w-full h-full p-0.5 bg-blue-500")
            .with_children(vec![
                Element::text(text).with_class("w-full h-1 text-red-500 font-bold")
            ])
    };
    vec![
        ("styled_layout", (12, 5), styled("hello")),
        (
            "clusters",
            (8, 2),
            Element::text("e\u{301}界👩‍💻").with_class("w-full h-1"),
        ),
        (
            "nested_clip",
            (12, 8),
            Element::layout(LayoutType::Flex)
                .with_class("flex flex-col w-full h-full p-0.5")
                .with_children(vec![Element::layout(LayoutType::Flex)
                    .with_class("relative w-3 h-1 overflow-hidden")
                    .with_children(vec![
                        Element::text("ab界").with_class("absolute top-0 left-0 w-4 h-1")
                    ])]),
        ),
        (
            "grid",
            (10, 4),
            Element::layout(LayoutType::Grid)
                .with_class("grid grid-cols-2 w-full h-1")
                .with_children(vec![Element::text("A"), Element::text("B")]),
        ),
        (
            "layers",
            (10, 4),
            Element::layout(LayoutType::Flex)
                .with_class("relative w-full h-full")
                .with_children(vec![
                    Element::text("X").with_class("absolute top-0 left-0 w-1 h-1 z-20"),
                    Element::text("lower").with_class("absolute top-0 left-0 w-5 h-1 z-10"),
                ]),
        ),
        (
            "nested_background",
            (20, 10),
            Element::layout(LayoutType::Flex)
                .with_class("flex flex-col w-full h-full p-0.5")
                .with_children(vec![Element::layout(LayoutType::Flex)
                    .with_class("relative flex flex-col w-8 h-3 bg-green-500")
                    .with_children(vec![Element::text("child").with_class("w-full h-1")])]),
        ),
        (
            "black_background",
            (8, 4),
            Element::layout(LayoutType::Flex)
                .with_class("flex flex-col w-full h-full bg-blue-500")
                .with_children(vec![
                    Element::layout(LayoutType::Flex).with_class("w-4 h-2 bg-black")
                ]),
        ),
    ]
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

/// The same props tests/charts_goldens.rs renders its goldens with.
fn chart_props(kind: ChartType, size: (u16, u16)) -> ChartProps {
    let data = [2.0, 8.0, 5.0, 9.0, 3.0, 7.0];
    let candle = kind == ChartType::Candlestick;
    ChartProps {
        chart_type: kind,
        width: size.0,
        height: size.1,
        series: vec![if candle {
            candles(&data)
        } else {
            series(&data)
        }],
        x_axis: ChartAxis {
            show_labels: false,
            show_grid: false,
            ..Default::default()
        },
        y_axis: ChartAxis {
            min: Some(0.0),
            max: Some(if candle { 11.0 } else { 10.0 }),
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

fn chart_frames() -> Vec<(String, (u16, u16), Element)> {
    let mut out = Vec::new();
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
            out.push((
                format!("chart_{name}_{cls}"),
                size,
                Element::typed::<Chart>(chart_props(kind.clone(), size)),
            ));
        }
    }
    out
}

fn snapshots_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots/renderer")
}

/// The vt100 model's screen: text grid then a digest of every cell's
/// colors and attributes.
fn vt100_section(bytes: &[u8], size: (u16, u16)) -> String {
    let mut parser = vt100::Parser::new(size.1, size.0, 0);
    parser.process(bytes);
    let screen = parser.screen();
    let mut hasher = common::digest::Digest::default();
    for r in 0..size.1 {
        for c in 0..size.0 {
            if let Some(cell) = screen.cell(r, c) {
                hasher.field(
                    format!(
                        "{:?}{:?}{}{}{}{}",
                        cell.fgcolor(),
                        cell.bgcolor(),
                        cell.bold(),
                        cell.italic(),
                        cell.underline(),
                        cell.inverse()
                    )
                    .as_bytes(),
                );
            }
        }
    }
    format!(
        "== vt100\n{}\nvt100 colors: {:016x}\n",
        screen.contents(),
        hasher.finish()
    )
}

/// Ghostty's real parser as the second model, only with the crate built.
#[cfg(feature = "embedded-terminal")]
fn ghostty_model(bytes: &[u8], size: (u16, u16)) -> String {
    ghostty_section(bytes, size)
}

#[cfg(not(feature = "embedded-terminal"))]
fn ghostty_model(_: &[u8], _: (u16, u16)) -> String {
    unreachable!("the ghostty model needs the embedded-terminal feature")
}

#[cfg(feature = "embedded-terminal")]
fn ghostty_section(bytes: &[u8], size: (u16, u16)) -> String {
    use libghostty_vt::render::{CellIterator, RowIterator};
    use libghostty_vt::{RenderState, Terminal};

    let mut terminal = Terminal::new(size.0, size.1).expect("terminal init");
    terminal.vt_write(bytes);
    let mut render_state = RenderState::new().expect("render state");
    let snapshot = render_state.update(&terminal).expect("snapshot");
    let mut rows = RowIterator::new().expect("row iterator");
    let mut cells = CellIterator::new().expect("cell iterator");
    let mut row_iter = rows.update(&snapshot).expect("row update");
    let mut hasher = common::digest::Digest::default();
    let mut lines = Vec::new();
    while let Some(row) = row_iter.next() {
        let mut line = String::new();
        let mut cell_iter = cells.update(row).expect("cell update");
        while let Some(cell) = cell_iter.next() {
            let graphemes = cell.graphemes().expect("graphemes");
            if graphemes.is_empty() {
                line.push(' ');
            } else {
                line.extend(graphemes);
            }
            let fg = cell.fg_color().expect("fg").map(|c| (c.r, c.g, c.b));
            let bg = cell.bg_color().expect("bg").map(|c| (c.r, c.g, c.b));
            hasher.field(format!("{fg:?}{bg:?}").as_bytes());
        }
        lines.push(line.trim_end().to_string());
    }
    format!(
        "== ghostty\n{}\nghostty colors: {:016x}\n",
        lines.join("\n"),
        hasher.finish()
    )
}

/// Split a recording into its per-model sections.
fn sections(text: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for line in text.lines() {
        if let Some(name) = line.strip_prefix("== ") {
            out.push((name.to_string(), String::new()));
        } else if let Some((_, body)) = out.last_mut() {
            body.push_str(line);
            body.push('\n');
        }
    }
    out
}

fn check_recording(name: &str, bytes: &[u8], size: (u16, u16)) {
    let mut produced = vec![vt100_section(bytes, size)];
    if cfg!(feature = "embedded-terminal") {
        produced.push(ghostty_model(bytes, size));
    }
    let path = snapshots_dir().join(format!("{name}.screen"));
    if std::env::var("REGENERATE").as_deref() == Ok("1") {
        std::fs::create_dir_all(snapshots_dir()).expect("snapshot dir");
        std::fs::write(&path, produced.concat()).expect("write recording");
        return;
    }
    let recorded = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!(
            "no recording {path:?}; RAS-004 needs a screen recorded before the rasterizer changed"
        )
    });
    let recorded = sections(&recorded);
    for section in sections(&produced.concat()) {
        let expected = recorded
            .iter()
            .find(|(model, _)| *model == section.0)
            .unwrap_or_else(|| panic!("recording {path:?} has no {} section", section.0));
        assert_eq!(
            section.1, expected.1,
            "{name}: the {} screen differs from its recording",
            section.0
        );
    }
}

fn last_bytes(element: Element, size: (u16, u16), painted: bool) -> Vec<u8> {
    let frames = if painted {
        app_input::run_when_painted(Root(element), size, 2)
    } else {
        app_input::run(Root(element), size, vec![(1, None)])
    };
    frames.last().expect("a presented frame").output.clone()
}

/// RAS-004: the renderer test frames replay to their recorded screens.
#[test]
fn ras_004_renderer_frames_replay_to_recorded_screens() {
    for (name, size, element) in renderer_frames() {
        let bytes = last_bytes(element, size, false);
        assert!(!bytes.is_empty(), "{name} emitted no bytes");
        check_recording(name, &bytes, size);
    }
}

/// RAS-004: every charts golden replays to its recorded screen.
#[test]
fn ras_004_chart_goldens_replay_to_recorded_screens() {
    for (name, size, element) in chart_frames() {
        let bytes = last_bytes(element, size, true);
        assert!(!bytes.is_empty(), "{name} emitted no bytes");
        check_recording(&name, &bytes, size);
    }
}
