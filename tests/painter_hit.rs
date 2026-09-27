//! painter-goldens mechanism: PNT-001, PNT-002, PNT-004 and PNT-005
//! (docs/spec/painter.md).
//!
//! The fast path for untransformed, unmasked nodes must paint the cells the
//! general path painted before it existed: those screens are recorded in
//! tests/snapshots/renderer. The per-cell hit grid must be exact under masks
//! and z-order, and the event layer must dispatch by it. An unchanged spec
//! must paint from the previous layout, and a changed one must keep the
//! layout nodes of the elements that did not change.

mod common;

use std::sync::{Arc, Mutex};

use common::app_input::{self, click};
use reactive_tui::app::RootComponent;
use reactive_tui::backend::{Backend, SuprTuiBackend};
use reactive_tui::builder;
use reactive_tui::component::{Element, LayoutType};
use reactive_tui::vdom::{bridge::vdom_to_element, node::VNode};
use std::io::{self, Write};

struct Root(Element);
impl RootComponent for Root {
    fn render(&self) -> Element {
        self.0.clone()
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

#[derive(Clone, Default)]
struct Sink(Arc<Mutex<Vec<u8>>>);
impl Write for Sink {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// The recorded screens' element shapes, all identity placements, so the
/// fast path paints every one of them (see tests/renderer_replay.rs).
fn recorded_frames() -> Vec<(&'static str, (u16, u16), Element)> {
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
    ]
}

/// The vt100 screen section as tests/renderer_replay.rs records it.
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
        "{}\nvt100 colors: {:016x}\n",
        screen.contents(),
        hasher.finish()
    )
}

fn recorded_vt100(name: &str) -> String {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/snapshots/renderer")
        .join(format!("{name}.screen"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("no recording {path:?}; RAS-004 records the general path"));
    let start = text.find("== vt100\n").expect("vt100 section") + "== vt100\n".len();
    let end = text[start..]
        .find("== ")
        .map_or(text.len(), |offset| start + offset);
    text[start..end].to_string()
}

/// PNT-001: identity nodes painted by the fast path produce the cells the
/// general path recorded before the fast path existed.
#[test]
fn pnt_001_fast_path_paints_the_recorded_cells() {
    for (name, size, element) in recorded_frames() {
        let frames = app_input::run(Root(element), size, vec![(1, None)]);
        let bytes = &frames.last().expect("a frame").output;
        assert_eq!(
            vt100_section(bytes, size),
            recorded_vt100(name),
            "{name}: the fast path's screen differs from the recorded general path"
        );
    }
}

/// PNT-001: identity nodes map no cell through an inverse transform, for
/// their backgrounds or their hit cells; a rotated node does, so the count
/// observes which path each node took.
#[test]
fn pnt_001_identity_nodes_map_no_cell_through_an_inverse_transform() {
    for (name, size, element) in recorded_frames() {
        let mut backend = SuprTuiBackend::with_writer(size.0, size.1, Sink::default()).unwrap();
        assert!(backend.render_frame(&element).unwrap());
        backend.present().unwrap();
        assert_eq!(
            backend.inverse_transformed_cells(),
            0,
            "{name}: an identity node took the per-cell inverse path"
        );
    }
    let rotated = Element::layout(LayoutType::Flex)
        .with_class("relative w-full h-full")
        .with_children(vec![
            Element::text("ABC").with_class("w-3 h-3 rotate-90 bg-green-500")
        ]);
    let mut backend = SuprTuiBackend::with_writer(6, 5, Sink::default()).unwrap();
    assert!(backend.render_frame(&rotated).unwrap());
    backend.present().unwrap();
    assert!(
        backend.inverse_transformed_cells() > 0,
        "a rotated node must take the general path"
    );
}

fn hits_after(size: (u16, u16), element: &Element) -> Vec<u32> {
    let mut backend = SuprTuiBackend::with_writer(size.0, size.1, Sink::default()).unwrap();
    assert!(backend.render_frame(element).unwrap());
    backend.present().unwrap();
    backend.sync().unwrap();
    backend.hit_cells().expect("a hit grid").to_vec()
}

/// PNT-002: the per-cell grid is exact under a mask and under z-order.
#[test]
fn pnt_002_hit_grid_is_exact_under_masks_and_z_order() {
    // Preorder: 0 root, 1 clipping box, 2 wide child, 3 sibling below.
    let masked = Element::layout(LayoutType::Flex)
        .with_class("relative w-full h-full")
        .with_children(vec![
            Element::layout(LayoutType::Flex)
                .with_class("absolute left-0 top-0 w-3 h-1 overflow-hidden")
                .with_children(vec![
                    Element::text("abcdef").with_class("absolute left-0 top-0 w-6 h-1")
                ]),
            Element::text("underneath").with_class("absolute left-0 top-1 w-10 h-1"),
        ]);
    let hits = hits_after((12, 3), &masked);
    let at = |x: usize, y: usize| hits[y * 12 + x];
    let mut backend = SuprTuiBackend::with_writer(12, 3, Sink::default()).unwrap();
    assert!(backend.render_frame(&masked).unwrap());
    backend.present().unwrap();
    backend.sync().unwrap();
    assert_eq!(backend.hit_at(1, 0), Some(3), "the per-cell query agrees");
    assert_eq!(backend.hit_at(5, 0), Some(1));
    assert_eq!(backend.hit_at(12, 0), None, "outside the grid");
    assert_eq!(at(1, 0), 3, "inside the mask the child is hit");
    assert_ne!(
        at(5, 0),
        3,
        "inside the child's rectangle but outside its mask is not the child"
    );
    assert_eq!(at(5, 0), 1, "outside the mask the root is hit");
    assert_eq!(at(2, 1), 4, "the sibling on the next row is hit");

    // Preorder: 0 root, 1 low sibling, 2 high sibling over it.
    let layered = Element::layout(LayoutType::Flex)
        .with_class("relative w-full h-full")
        .with_children(vec![
            Element::text("lower").with_class("absolute top-0 left-0 w-5 h-1 z-10"),
            Element::text("X").with_class("absolute top-0 left-0 w-1 h-1 z-20"),
        ]);
    let hits = hits_after((8, 2), &layered);
    assert_eq!(hits[0], 3, "the higher sibling wins the overlapping cell");
    assert_eq!(hits[1], 2, "the lower sibling keeps its own cells");
}

/// PNT-002: the event layer dispatches by the grid, so a click inside a
/// child's rectangle but outside its mask never reaches the child.
#[test]
fn pnt_002_clicks_dispatch_by_the_painted_cell() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let child = {
        let calls = calls.clone();
        builder::button()
            .text("abcdef")
            .class("absolute left-0 top-0 w-6 h-1 p-0")
            .on_click(move || calls.lock().unwrap().push("child"))
            .build()
    };
    let root = Element::layout(LayoutType::Flex)
        .with_class("relative w-full h-full")
        .with_children(vec![Element::layout(LayoutType::Flex)
            .with_class("absolute left-0 top-0 w-3 h-1 overflow-hidden")
            .with_children(vec![child])]);
    app_input::run(
        Root(root),
        (12, 3),
        vec![(1, click(5, 0)), (1, click(1, 0)), (1, None)],
    );
    let calls = calls.lock().unwrap().clone();
    assert_eq!(
        calls,
        vec!["child"],
        "only the click inside the mask may reach the child"
    );
}

fn layered(with_high: bool) -> Element {
    let mut children = vec![Element::text("X").with_class("absolute top-0 left-0 w-1 h-1 z-10")];
    if with_high {
        children.push(Element::text("X").with_class("absolute top-0 left-0 w-1 h-1 z-20"));
    }
    Element::layout(LayoutType::Flex)
        .with_class("relative w-full h-full")
        .with_children(children)
}

fn present_hits(backend: &mut SuprTuiBackend, element: &Element) -> Vec<u32> {
    assert!(backend.render_frame(element).unwrap());
    backend.present().unwrap();
    backend.sync().unwrap();
    backend.hit_cells().expect("a hit grid").to_vec()
}

/// PNT-002: a frame whose cells equal the previous frame's still commits
/// its own hit grid. Here a higher sibling starts covering a cell with the
/// same glyph, so no cell changes and the renderer skips the frame's bytes;
/// the grid must still name the higher sibling, as a fresh backend does.
#[test]
fn pnt_002_a_frame_with_unchanged_cells_commits_its_hit_grid() {
    let mut fresh = SuprTuiBackend::with_writer(8, 2, Sink::default()).unwrap();
    let expected = present_hits(&mut fresh, &layered(true))[0];
    assert_eq!(
        expected, 3,
        "grid values are preorder index + 1; the higher sibling is 2"
    );

    let out = Sink::default();
    let mut backend = SuprTuiBackend::with_writer(8, 2, out.clone()).unwrap();
    assert_eq!(present_hits(&mut backend, &layered(false))[0], 2);
    let written = out.0.lock().unwrap().len();
    let second = present_hits(&mut backend, &layered(true))[0];
    assert_eq!(
        out.0.lock().unwrap().len(),
        written,
        "the second frame changes no cell, so it writes nothing"
    );
    assert_eq!(
        second, expected,
        "the cell painted by the higher sibling must hit it"
    );
    assert_eq!(backend.hit_at(0, 0), Some(expected), "the query agrees");
}

/// A root whose second render inserts two zero-size elements before a
/// button: element indices shift, but no painted cell changes.
struct Shifting {
    renders: std::sync::atomic::AtomicUsize,
    calls: Arc<Mutex<Vec<&'static str>>>,
}
impl RootComponent for Shifting {
    fn render(&self) -> Element {
        let n = self
            .renders
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let calls = self.calls.clone();
        let target = builder::button()
            .text("target")
            .class("absolute left-0 top-1 w-6 h-1 p-0")
            .on_click(move || calls.lock().unwrap().push("target"))
            .build();
        let spacers = if n == 0 {
            Vec::new()
        } else {
            (0..2)
                .map(|_| {
                    Element::layout(LayoutType::Flex).with_class("absolute left-0 top-0 w-0 h-0")
                })
                .collect()
        };
        Element::layout(LayoutType::Flex)
            .with_class("relative w-full h-full")
            .with_children(vec![
                Element::layout(LayoutType::Flex)
                    .with_class("absolute left-0 top-0 w-0 h-0")
                    .with_children(spacers),
                target,
            ])
    }
    fn update(&mut self) -> reactive_tui::error::Result<reactive_tui::app::RootUpdate> {
        Ok(reactive_tui::app::RootUpdate::Redraw)
    }
}

/// PNT-002 at the App: after a frame whose element indices shifted but
/// whose cells did not change, a click on the button reaches the button.
#[test]
fn pnt_002_a_click_after_a_frame_with_unchanged_cells_reaches_its_element() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    app_input::run(
        Shifting {
            renders: std::sync::atomic::AtomicUsize::new(0),
            calls: calls.clone(),
        },
        (12, 3),
        vec![(4, click(2, 1)), (6, None)],
    );
    assert_eq!(
        calls.lock().unwrap().clone(),
        vec!["target"],
        "a click on the button's text must reach it"
    );
}

/// PNT-004: presenting an unchanged spec paints from the previous layout;
/// a changed spec lays out again.
#[test]
fn pnt_004_unchanged_spec_reuses_the_layout() {
    let mut backend = SuprTuiBackend::with_writer(16, 4, Sink::default()).unwrap();
    let frame = |text: &str| {
        Element::layout(LayoutType::Flex)
            .with_class("flex flex-col w-full h-full p-0.5")
            .with_children(vec![Element::text(text).with_class("w-full h-1")])
    };
    // Each present reports whether it reused the layout and how many
    // layouts have run: the count is taken where the layout engine runs,
    // so it observes the work, not only the reuse decision.
    let show = |backend: &mut SuprTuiBackend, element: &Element| {
        assert!(backend.render_frame(element).unwrap());
        backend.present().unwrap();
        backend.sync().unwrap();
        (backend.layout_reused(), backend.layout_runs())
    };
    assert_eq!(
        show(&mut backend, &frame("one")),
        (false, 1),
        "a first frame lays out"
    );
    assert_eq!(
        show(&mut backend, &frame("one")),
        (true, 1),
        "an unchanged spec must reuse the layout and run none"
    );
    assert_eq!(
        show(&mut backend, &frame("two")),
        (false, 2),
        "changed text must lay out again"
    );
    assert_eq!(show(&mut backend, &frame("two")), (true, 2));
    backend.resize(20, 4);
    assert_eq!(
        show(&mut backend, &frame("two")),
        (false, 3),
        "a new size must lay out again"
    );
}

/// A 700 by 200 screen of 199 rows of 25 text elements of class `label`;
/// the first element shows `first`. In spec order the root is element 0,
/// the first row 1 and its labels 2 to 26.
fn label_grid(first: &str, label: &str) -> Element {
    Element::layout(LayoutType::Flex)
        .with_class("flex flex-col w-full h-full")
        .with_children(
            (0..199)
                .map(|row| {
                    Element::layout(LayoutType::Flex)
                        .with_class("flex flex-row w-full h-1")
                        .with_children(
                            (0..25)
                                .map(|column| {
                                    let text = if (row, column) == (0, 0) {
                                        first.to_string()
                                    } else {
                                        format!("r{row}c{column}")
                                    };
                                    Element::text(text).with_class(label)
                                })
                                .collect(),
                        )
                })
                .collect(),
        )
}

/// PNT-005: changing one element's text keeps every other element's layout
/// node and measures only the texts the change can move: with `flex-1`
/// labels, which share their row's width by their text, the labels of its
/// row; with fixed-width labels, itself alone.
#[test]
fn pnt_005_one_changed_text_keeps_the_other_layout_nodes() {
    for (label, movable) in [("flex-1 h-1", 2..=26), ("w-28 h-1", 2..=2)] {
        let mut backend = SuprTuiBackend::with_writer(700, 200, Sink::default()).unwrap();
        let mut show = |element: &Element| {
            assert!(backend.render_frame(element).unwrap());
            backend.present().unwrap();
            backend.sync().unwrap();
            (
                backend.layout_reused(),
                backend.layout_nodes_built(),
                backend.layout_measured_elements().to_vec(),
            )
        };
        let (_, built, _) = show(&label_grid("first", label));
        assert!(
            built > 5_000,
            "{label}: a first frame lays out every element, but built {built} layout nodes"
        );
        let (reused, built, measured) = show(&label_grid("changed", label));
        assert!(!reused, "{label}: a changed text must lay out again");
        let outside: Vec<usize> = measured
            .iter()
            .copied()
            .filter(|index| !movable.contains(index))
            .collect();
        assert!(
            built <= 1 && outside.is_empty(),
            "{label}: changing one of 4,975 texts built {built} layout nodes and measured {} texts, {} of them elements the change cannot move, such as {:?}",
            measured.len(),
            outside.len(),
            &outside[..outside.len().min(5)]
        );
    }
}

/// PNT-005: frames laid out from the previous frame's layout paint the cells
/// and hit grid that a full layout of the same spec paints, through text,
/// class and child changes.
#[test]
fn pnt_005_edited_frames_paint_what_a_full_layout_paints() {
    let size = (40, 12);
    let plain = "flex flex-row w-full h-1";
    let tall = "flex flex-row w-full h-3 bg-blue-500";
    let row = |texts: &[&str], class: &str| {
        Element::layout(LayoutType::Flex)
            .with_class(class)
            .with_children(
                texts
                    .iter()
                    .map(|text| Element::text(*text).with_class("flex-1 h-1"))
                    .collect(),
            )
    };
    let frame = |rows: Vec<Element>| {
        Element::layout(LayoutType::Flex)
            .with_class("flex flex-col w-full h-full")
            .with_children(rows)
    };
    let long = "a much longer text";
    // A panel shown or hidden by its inline display style.
    let panel = |display: &str| {
        vdom_to_element(
            VNode::element("flex")
                .class("flex flex-row w-full h-2 bg-green-500")
                .style(format!("display: {display}"))
                .child(VNode::text("shown"))
                .build(),
        )
    };
    let frames = [
        frame(vec![row(&["a", "b", "c"], plain), row(&["d", "e"], plain)]),
        // A text changes and grows.
        frame(vec![row(&["a", long, "c"], plain), row(&["d", "e"], plain)]),
        // A class changes: the second row grows and gets a background.
        frame(vec![row(&["a", long, "c"], plain), row(&["d", "e"], tall)]),
        // A child is added in the middle of a row.
        frame(vec![
            row(&["a", "x", long, "c"], plain),
            row(&["d", "e"], tall),
        ]),
        // A row is removed and another is added at the end.
        frame(vec![
            row(&["d", "e"], tall),
            row(&["f"], "flex flex-row w-1/2 h-1 bg-green-500"),
        ]),
        // A hidden panel is shown, then hidden again.
        frame(vec![
            row(&["a", "b"], plain),
            panel("none"),
            row(&["d"], plain),
        ]),
        frame(vec![
            row(&["a", "b"], plain),
            panel("flex"),
            row(&["d"], plain),
        ]),
        frame(vec![
            row(&["a", "b"], plain),
            panel("none"),
            row(&["d"], plain),
        ]),
        // Back to the first frame.
        frame(vec![row(&["a", "b", "c"], plain), row(&["d", "e"], plain)]),
    ];
    let sink = Sink::default();
    let mut edited = SuprTuiBackend::with_writer(size.0, size.1, sink.clone()).unwrap();
    for (index, element) in frames.iter().enumerate() {
        let hits = present_hits(&mut edited, element);
        let screen = vt100_section(&sink.0.lock().unwrap(), size);
        let full_sink = Sink::default();
        let mut full = SuprTuiBackend::with_writer(size.0, size.1, full_sink.clone()).unwrap();
        let full_hits = present_hits(&mut full, element);
        assert_eq!(
            screen,
            vt100_section(&full_sink.0.lock().unwrap(), size),
            "frame {index}: the cells differ from a full layout of the same spec"
        );
        assert_eq!(
            hits, full_hits,
            "frame {index}: the hit grid differs from a full layout of the same spec"
        );
    }
}
