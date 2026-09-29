//! layout mechanism: LAY-001 to LAY-004 (docs/spec/layout.md).
//!
//! On the terminal backend, what was painted is read from the hit grid,
//! which names the element that painted each cell, so a gap is counted in
//! the cells the user sees. The debug backend, where the goldens are
//! checked, keeps no hit grid for a test to read; there the rectangles are
//! the bounds it reports for each painted element.

#[allow(dead_code)]
#[path = "../examples/widget_catalog/catalog.rs"]
mod catalog;
#[allow(dead_code)]
mod common;

use catalog::{Catalog, CatalogPage};
use common::app_input::{click, HANG_GUARD};
use reactive_tui::app::{App, AppWaker, RootComponent};
use reactive_tui::backend::{
    Backend, DebugBackend, FrameLayout, PaintedNode, PresentedLayout, SuprTuiBackend,
};
use reactive_tui::builder::{data_table, div, ElementBuilder};
use reactive_tui::component::{Element, ElementType, LayoutType};
use reactive_tui::error::Result;
use reactive_tui::event::types::Event;
use reactive_tui::layout::css::apply_utility_classes;
use reactive_tui::layout::style::StyleBuilder;
use reactive_tui::render::{reconcile::PatchOp, RenderTree};
use std::collections::VecDeque;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use taffy::style::{LengthPercentage, LengthPercentageAuto, Style};

/// The class that marks the elements a test measures. No utility reads it.
const ITEM: &str = "measured";

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

#[derive(Clone, Copy, Debug)]
enum Painter {
    Debug,
    Terminal,
}
const PAINTERS: [Painter; 2] = [Painter::Debug, Painter::Terminal];

/// A rectangle of cells: its first column and row, and its size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Cells {
    left: usize,
    top: usize,
    width: usize,
    height: usize,
}
impl Cells {
    fn right(self) -> usize {
        self.left + self.width
    }
    fn bottom(self) -> usize {
        self.top + self.height
    }
}

/// What one frame painted.
struct Painted {
    width: usize,
    height: usize,
    /// Element index plus one at each cell, row by row, where the backend
    /// keeps a hit grid.
    hits: Option<Vec<u32>>,
    nodes: Vec<PaintedNode>,
}
impl Painted {
    /// The rectangle `element` painted. It fails when the element painted
    /// nothing or painted cells that do not form one full rectangle.
    fn cells(&self, element: usize, what: &str) -> Cells {
        let Some(hits) = &self.hits else {
            let node = self
                .nodes
                .iter()
                .find(|node| node.element_index == element)
                .unwrap_or_else(|| panic!("{what}: element {element} painted no cell"));
            let bounds = [
                node.bounds.x,
                node.bounds.y,
                node.bounds.width,
                node.bounds.height,
            ];
            assert!(
                bounds
                    .iter()
                    .all(|edge| edge.fract() == 0.0 && *edge >= 0.0),
                "{what}: element {element} has bounds that are no whole cells: {bounds:?}"
            );
            return Cells {
                left: bounds[0] as usize,
                top: bounds[1] as usize,
                width: bounds[2] as usize,
                height: bounds[3] as usize,
            };
        };
        let mine = u32::try_from(element).unwrap() + 1;
        let own: Vec<(usize, usize)> = (0..self.height)
            .flat_map(|y| (0..self.width).map(move |x| (x, y)))
            .filter(|(x, y)| hits[y * self.width + x] == mine)
            .collect();
        assert!(!own.is_empty(), "{what}: element {element} painted no cell");
        let left = own.iter().map(|cell| cell.0).min().unwrap();
        let top = own.iter().map(|cell| cell.1).min().unwrap();
        let right = own.iter().map(|cell| cell.0).max().unwrap() + 1;
        let bottom = own.iter().map(|cell| cell.1).max().unwrap() + 1;
        assert_eq!(
            own.len(),
            (right - left) * (bottom - top),
            "{what}: element {element} did not paint one full rectangle"
        );
        Cells {
            left,
            top,
            width: right - left,
            height: bottom - top,
        }
    }
}

/// Lay `element` out and paint it at `size` with `painter`.
fn paint(painter: Painter, element: &Element, size: (u16, u16)) -> Painted {
    let (hits, nodes) = match painter {
        Painter::Debug => {
            let mut backend = DebugBackend::new(size.0, size.1);
            assert!(backend.render_frame(element).unwrap());
            backend.present().unwrap();
            (
                None,
                backend.painted_nodes().expect("painted nodes").to_vec(),
            )
        }
        Painter::Terminal => {
            let mut backend = SuprTuiBackend::with_writer(size.0, size.1, Sink::default()).unwrap();
            assert!(backend.render_frame(element).unwrap());
            backend.present().unwrap();
            backend.sync().unwrap();
            (
                Some(backend.hit_cells().expect("a hit grid").to_vec()),
                backend.painted_nodes().expect("painted nodes").to_vec(),
            )
        }
    };
    Painted {
        width: size.0.into(),
        height: size.1.into(),
        hits,
        nodes,
    }
}

/// The measured elements of `root`, by their index in a preorder walk, in
/// the order of the walk.
fn measured(root: &Element) -> Vec<usize> {
    fn walk(element: &Element, next: &mut usize, found: &mut Vec<usize>) {
        let index = *next;
        *next += 1;
        if element
            .class
            .as_deref()
            .is_some_and(|class| class.split_whitespace().any(|token| token == ITEM))
        {
            found.push(index);
        }
        for child in &element.children {
            walk(child, next, found);
        }
    }
    let mut found = Vec::new();
    walk(root, &mut 0, &mut found);
    found
}

/// The rectangles the measured elements of `root` painted.
fn rectangles(frame: &Painted, root: &Element, what: &str) -> Vec<Cells> {
    measured(root)
        .into_iter()
        .map(|index| frame.cells(index, what))
        .collect()
}

/// A measured leaf with `class`.
fn leaf(class: &str) -> Element {
    Element::text("x").with_class(format!("{ITEM} {class}"))
}

/// A layout element of `kind` with `class` and a gap of `gap` cells, set as
/// a style so the check does not rest on what a gap class means (LAY-002).
fn spaced(kind: LayoutType, class: &str, gap: usize, children: Vec<Element>) -> Element {
    ElementBuilder::new(ElementType::Layout(kind))
        .styles(StyleBuilder::new().gap_px(gap as f32, gap as f32))
        .class(class)
        .children(children)
        .build()
}

/// Where a container under test stands on the screen.
#[derive(Clone, Copy, Debug)]
enum Place {
    /// At the screen's first column, as wide as the screen.
    Whole,
    /// Inside a box with a padding of one cell.
    Padded,
    /// Beside a box that takes a third of the screen, so it starts at a
    /// position that is not a whole number of cells at most widths.
    AfterAThird,
}
const PLACES: [Place; 3] = [Place::Whole, Place::Padded, Place::AfterAThird];

/// `container` at `place`. The container must take the width it is given.
fn placed(place: Place, container: Element) -> Element {
    match place {
        Place::Whole => Element::layout(LayoutType::Flex)
            .with_class("flex-col w-full h-full")
            .with_children(vec![container]),
        Place::Padded => Element::layout(LayoutType::Flex)
            .with_class("flex-col w-full h-full")
            .with_children(vec![ElementBuilder::new(ElementType::Layout(
                LayoutType::Flex,
            ))
            .styles(StyleBuilder::new().padding_all_px(1.0))
            .class("flex-col w-full h-full")
            .child(container)
            .build()]),
        Place::AfterAThird => Element::layout(LayoutType::Flex)
            .with_class("flex-row w-full h-full")
            .with_children(vec![
                Element::text("t").with_class("w-1/3 h-1 shrink-0"),
                Element::layout(LayoutType::Flex)
                    .with_class("flex-col flex-1 min-w-0 h-full")
                    .with_children(vec![container]),
            ]),
    }
}

/// The columns a container at `place` has for its content on a screen of
/// `width` cells: its first column and the column after its last.
fn content(place: Place, width: usize, frame: &Painted) -> (usize, usize) {
    match place {
        Place::Whole => (0, width),
        Place::Padded => (1, width - 1),
        Place::AfterAThird => {
            // The box beside the container is the root's first child.
            let third = frame.cells(1, "the box of a third");
            assert_eq!(third.left, 0, "the box of a third starts at column 0");
            assert!(
                third.width.abs_diff(width / 3) <= 1,
                "the box of a third is {} of {width} cells wide",
                third.width
            );
            (third.right(), width)
        }
    }
}

/// LAY-001 for one row of items: the gap between neighbours, widths at most
/// one cell apart, and the row filling its container.
fn assert_row(row: &[Cells], gap: usize, span: (usize, usize), what: &str) {
    for pair in row.windows(2) {
        assert_eq!(
            pair[1].left as i64 - pair[0].right() as i64,
            gap as i64,
            "{what}: the gap between two neighbours, asked {gap}: {row:?}"
        );
    }
    let widest = row.iter().map(|cells| cells.width).max().unwrap();
    let narrowest = row.iter().map(|cells| cells.width).min().unwrap();
    assert!(
        widest - narrowest <= 1,
        "{what}: equal tracks differ by {} cells: {row:?}",
        widest - narrowest
    );
    assert_eq!(
        (row[0].left, row[row.len() - 1].right()),
        span,
        "{what}: the row does not fill its container: {row:?}"
    );
}

#[test]
fn lay_001_a_grid_keeps_its_gap_at_every_width() {
    for painter in PAINTERS {
        for place in PLACES {
            for columns in 2..=6usize {
                for gap in 1..=2usize {
                    for width in 40..=512u16 {
                        let items = (0..columns * 2).map(|_| leaf("min-w-0 h-1")).collect();
                        let class = format!("w-full grid grid-cols-{columns}");
                        let root = placed(place, spaced(LayoutType::Grid, &class, gap, items));
                        let frame = paint(painter, &root, (width, 8));
                        let what = format!(
                            "{painter:?}, {place:?}, {columns} columns, gap {gap}, width {width}"
                        );
                        let cells = rectangles(&frame, &root, &what);
                        let span = content(place, width.into(), &frame);
                        let (first, second) = cells.split_at(columns);
                        assert_row(first, gap, span, &what);
                        assert_row(second, gap, span, &what);
                        assert_eq!(
                            second[0].top as i64 - first[0].bottom() as i64,
                            gap as i64,
                            "{what}: the gap between two rows"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn lay_001_an_item_that_spans_tracks_ends_where_its_last_track_ends() {
    for painter in PAINTERS {
        for place in PLACES {
            for width in 40..=512u16 {
                let spans: [&[usize]; 4] = [&[4], &[2, 1, 1], &[3, 1], &[1, 1, 1, 1]];
                let items = spans
                    .iter()
                    .flat_map(|row| row.iter())
                    .map(|span| leaf(&format!("min-w-0 h-1 col-span-{span}")))
                    .collect();
                let root = placed(
                    place,
                    spaced(LayoutType::Grid, "w-full grid grid-cols-4", 1, items),
                );
                let frame = paint(painter, &root, (width, 10));
                let what = format!("{painter:?}, {place:?}, width {width}");
                let mut cells = rectangles(&frame, &root, &what).into_iter();
                let rows: Vec<Vec<Cells>> = spans
                    .iter()
                    .map(|row| row.iter().map(|_| cells.next().unwrap()).collect())
                    .collect();
                let tracks = &rows[3];
                assert_row(tracks, 1, content(place, width.into(), &frame), &what);
                for (row, painted) in spans.iter().zip(&rows) {
                    let mut track = 0;
                    for (span, item) in row.iter().zip(painted) {
                        assert_eq!(
                            (item.left, item.right()),
                            (tracks[track].left, tracks[track + span - 1].right()),
                            "{what}: an item that spans {span} tracks from track {track}"
                        );
                        track += span;
                    }
                }
            }
        }
    }
}

#[test]
fn lay_001_a_flex_row_and_a_flex_column_keep_their_gap() {
    for painter in PAINTERS {
        for items in 2..=5usize {
            for gap in 1..=2usize {
                for place in PLACES {
                    for width in 40..=512u16 {
                        let children = (0..items).map(|_| leaf("flex-1 min-w-0 h-1")).collect();
                        let root = placed(
                            place,
                            spaced(LayoutType::Flex, "flex-row w-full", gap, children),
                        );
                        let frame = paint(painter, &root, (width, 6));
                        let what = format!(
                            "{painter:?}, {place:?}, {items} items, gap {gap}, width {width}"
                        );
                        let cells = rectangles(&frame, &root, &what);
                        assert_row(&cells, gap, content(place, width.into(), &frame), &what);
                    }
                }
                // The shortest column that holds the gaps and one row for
                // each item.
                let least = (items + (items - 1) * gap) as u16;
                for height in least..=60 {
                    let children = (0..items).map(|_| leaf("flex-1 min-h-0 w-full")).collect();
                    let root = spaced(LayoutType::Flex, "flex-col w-full h-full", gap, children);
                    let frame = paint(painter, &root, (20, height));
                    let what = format!("{painter:?}, {items} items, gap {gap}, height {height}");
                    let cells = rectangles(&frame, &root, &what);
                    for pair in cells.windows(2) {
                        assert_eq!(
                            pair[1].top as i64 - pair[0].bottom() as i64,
                            gap as i64,
                            "{what}: the gap between two neighbours: {cells:?}"
                        );
                    }
                    let tallest = cells.iter().map(|cells| cells.height).max().unwrap();
                    let shortest = cells.iter().map(|cells| cells.height).min().unwrap();
                    assert!(tallest - shortest <= 1, "{what}: {cells:?}");
                    assert_eq!(
                        (cells[0].top, cells[items - 1].bottom()),
                        (0, usize::from(height)),
                        "{what}: the column does not fill its container: {cells:?}"
                    );
                }
            }
        }
    }
}

fn style(class: &str) -> Style {
    apply_utility_classes(class, StyleBuilder::new()).build()
}

/// The lengths `class` sets, of those its prefix names: padding and margin
/// as left, right, top and bottom, a gap as across and down.
fn lengths(prefix: &str, class: &str) -> Vec<f32> {
    let style = style(class);
    let padding = |side: LengthPercentage| {
        let cells = side.into_raw().value();
        assert_eq!(
            side,
            LengthPercentage::length(cells),
            "{class}: not a length"
        );
        cells
    };
    let margin = |side: LengthPercentageAuto| {
        let cells = side.into_raw().value();
        assert_eq!(
            side,
            LengthPercentageAuto::length(cells),
            "{class}: not a length"
        );
        cells
    };
    let (left, right, top, bottom) = match prefix.as_bytes()[0] {
        b'p' => (
            padding(style.padding.left),
            padding(style.padding.right),
            padding(style.padding.top),
            padding(style.padding.bottom),
        ),
        b'm' => (
            margin(style.margin.left),
            margin(style.margin.right),
            margin(style.margin.top),
            margin(style.margin.bottom),
        ),
        _ => {
            let across = padding(style.gap.width);
            let down = padding(style.gap.height);
            return match prefix {
                "gap" => vec![across, down],
                "gap-x" | "space-x" => vec![across],
                _ => vec![down],
            };
        }
    };
    match &prefix[1..] {
        "" => vec![left, right, top, bottom],
        "x" => vec![left, right],
        "y" => vec![top, bottom],
        "l" => vec![left],
        "r" => vec![right],
        "t" => vec![top],
        _ => vec![bottom],
    }
}

const SPACING: [&str; 19] = [
    "p", "px", "py", "pt", "pr", "pb", "pl", "m", "mx", "my", "mt", "mr", "mb", "ml", "gap",
    "gap-x", "gap-y", "space-x", "space-y",
];

#[test]
fn lay_002_a_spacing_class_counts_cells() {
    for prefix in SPACING {
        for cells in 0..=512u16 {
            let class = format!("{prefix}-{cells}");
            for length in lengths(prefix, &class) {
                assert_eq!(length, f32::from(cells), "{class}");
            }
        }
    }
}

#[test]
fn lay_002_a_fraction_counts_as_the_next_whole_cell() {
    for prefix in SPACING {
        for (number, cells) in [("0.25", 1.0), ("0.5", 1.0), ("1.5", 2.0), ("2.5", 3.0)] {
            let class = format!("{prefix}-{number}");
            for length in lengths(prefix, &class) {
                assert_eq!(length, cells, "{class}");
            }
        }
    }
}

#[test]
fn lay_002_padding_and_gap_are_painted_in_cells() {
    for painter in PAINTERS {
        let root = Element::layout(LayoutType::Flex)
            .with_class("flex-col w-full h-full")
            .with_children(vec![Element::layout(LayoutType::Flex)
                .with_class("flex-row w-full h-full p-1 gap-3")
                .with_children(vec![
                    leaf("w-2 h-1 shrink-0"),
                    leaf("w-2 h-1 shrink-0"),
                ])]);
        let frame = paint(painter, &root, (40, 6));
        let what = format!("{painter:?}");
        let cells = rectangles(&frame, &root, &what);
        assert_eq!(
            (cells[0].left, cells[0].top),
            (1, 1),
            "{what}: `p-1` pads one cell"
        );
        assert_eq!(
            cells[1].left as i64 - cells[0].right() as i64,
            3,
            "{what}: `gap-3` leaves three cells"
        );
    }
}

/// Four leaves in a grid of two columns with `class`: the gap across and
/// the gap down that were painted.
fn gaps(painter: Painter, class: &str) -> (i64, i64) {
    let items = (0..4).map(|_| leaf("min-w-0 h-1")).collect();
    let root = Element::layout(LayoutType::Grid)
        .with_class(format!("w-full grid grid-cols-2 {class}"))
        .with_children(items);
    let what = format!("{painter:?}, {class}");
    let cells = rectangles(&paint(painter, &root, (41, 12)), &root, &what);
    (
        cells[1].left as i64 - cells[0].right() as i64,
        cells[2].top as i64 - cells[0].bottom() as i64,
    )
}

#[test]
fn lay_003_a_gap_class_for_one_direction_keeps_the_other() {
    for painter in PAINTERS {
        for (class, across, down) in [
            ("gap-2 gap-x-1", 1, 2),
            ("gap-2 gap-y-1", 2, 1),
            ("gap-x-1 gap-y-2", 1, 2),
        ] {
            assert_eq!(
                gaps(painter, class),
                (across, down),
                "{painter:?}: the gaps of `{class}`, across and down"
            );
        }
    }
}

#[test]
fn lay_003_a_full_span_adds_no_track() {
    for painter in PAINTERS {
        let mut items = vec![leaf("min-w-0 h-1 col-span-full")];
        items.extend((0..4).map(|_| leaf("min-w-0 h-1")));
        let root = spaced(LayoutType::Grid, "w-full grid grid-cols-4", 1, items);
        let what = format!("{painter:?}, col-span-full");
        let cells = rectangles(&paint(painter, &root, (83, 6)), &root, &what);
        assert_row(&cells[1..], 1, (0, 83), &what);
        assert_eq!(
            (cells[0].left, cells[0].right()),
            (0, 83),
            "{what}: the item spans the grid"
        );

        let mut items = vec![leaf("min-h-0 w-full row-span-full")];
        items.extend((0..4).map(|_| leaf("min-h-0 w-full")));
        let class = "w-full h-full grid grid-cols-2 grid-rows-4";
        let root = spaced(LayoutType::Grid, class, 1, items);
        let what = format!("{painter:?}, row-span-full");
        let cells = rectangles(&paint(painter, &root, (21, 19)), &root, &what);
        assert_eq!(
            (cells[0].top, cells[0].bottom()),
            (0, 19),
            "{what}: the item spans the grid"
        );
        for pair in cells[1..].windows(2) {
            assert_eq!(
                pair[1].top as i64 - pair[0].bottom() as i64,
                1,
                "{what}: {cells:?}"
            );
        }
        assert_eq!(
            (cells[1].top, cells[4].bottom()),
            (0, 19),
            "{what}: four rows fill the grid: {cells:?}"
        );
    }
}

#[test]
fn lay_003_auto_fit_and_auto_fill_make_columns_of_a_least_width() {
    for painter in PAINTERS {
        for class in ["grid-cols-auto-fit-20", "grid-cols-auto-fill-20"] {
            let items = (0..8).map(|_| leaf("min-w-0 h-1")).collect();
            let root = Element::layout(LayoutType::Grid)
                .with_class(format!("w-full grid {class}"))
                .with_children(items);
            let what = format!("{painter:?}, {class}");
            let cells = rectangles(&paint(painter, &root, (90, 6)), &root, &what);
            let first_row = cells.iter().filter(|item| item.top == cells[0].top).count();
            assert_eq!(first_row, 4, "{what}: columns in 90 cells: {cells:?}");
            assert!(
                cells.iter().all(|item| item.width >= 20),
                "{what}: a column narrower than 20 cells: {cells:?}"
            );
        }
    }
}

/// One presented frame: where each element was painted, as its index and
/// its left, top, width and height.
type Geometry = Vec<(usize, [i64; 4])>;

fn geometry(nodes: &[PaintedNode]) -> Geometry {
    nodes
        .iter()
        .map(|node| {
            (
                node.element_index,
                [
                    node.bounds.x as i64,
                    node.bounds.y as i64,
                    node.bounds.width as i64,
                    node.bounds.height as i64,
                ],
            )
        })
        .collect()
}

/// Presents after the last input at which a run ends, and turns of the App's
/// loop without a present after which it ends. A layout that feeds on itself
/// presents again and again, so it reaches the first; one that has settled
/// reaches the second.
const PRESENTS: usize = 12;
const IDLE_TURNS: usize = 400;

/// Every presented frame's geometry, and how many frames were presented
/// when the last click was sent.
type Frames = Arc<Mutex<(Vec<Geometry>, usize)>>;

/// A debug backend that clicks each text of a script once the frame shows
/// it, then lets the App run until it has stopped presenting.
struct Settling {
    inner: DebugBackend,
    script: VecDeque<&'static str>,
    frames: Frames,
    seen: usize,
    idle: usize,
    deadline: Instant,
}
impl Settling {
    /// The cell where `text` starts in the presented frame.
    fn find(&self, text: &str) -> Option<(u16, u16)> {
        let (width, height) = self.inner.size();
        let wanted: Vec<String> = text.chars().map(String::from).collect();
        for y in 0..usize::from(height) {
            for x in 0..usize::from(width).saturating_sub(wanted.len() - 1) {
                if wanted.iter().enumerate().all(|(offset, wanted)| {
                    self.inner.cell_text(x + offset, y) == Some(wanted.as_str())
                }) {
                    return Some((x as u16, y as u16));
                }
            }
        }
        None
    }
}
impl Backend for Settling {
    fn painted_nodes(&self) -> Option<&[PaintedNode]> {
        self.inner.painted_nodes()
    }
    fn component_layouts(&self) -> Option<&[PresentedLayout]> {
        self.inner.component_layouts()
    }
    fn hit_cells(&self) -> Option<&[u32]> {
        self.inner.hit_cells()
    }
    fn render_frame(&mut self, element: &Element) -> Result<bool> {
        self.inner.render_frame(element)
    }
    fn layout_frame(&mut self, element: Arc<Element>) -> Result<Option<FrameLayout>> {
        self.inner.layout_frame(element)
    }
    fn apply_patches(&mut self, _: &[PatchOp], _: &RenderTree) -> Result<()> {
        panic!("complete frame required")
    }
    fn clear(&mut self) -> Result<()> {
        self.inner.clear()
    }
    fn size(&self) -> (u16, u16) {
        self.inner.size()
    }
    fn present(&mut self) -> Result<()> {
        self.inner.present()?;
        let nodes = self.inner.painted_nodes().unwrap_or_default();
        self.frames.lock().unwrap().0.push(geometry(nodes));
        Ok(())
    }
    fn resize(&mut self, width: usize, height: usize) {
        self.inner.resize(width, height);
    }
    fn shutdown(&mut self) -> Result<()> {
        self.inner.shutdown()
    }
    fn poll_event(&mut self, _: Option<u64>) -> Result<Option<Event>> {
        panic!("wake-aware input required")
    }
    fn poll_event_with_wake(
        &mut self,
        _: Option<Duration>,
        wake: &AppWaker,
    ) -> Result<Option<Event>> {
        assert!(
            Instant::now() < self.deadline,
            "the App did not show {:?} before the deadline",
            self.script.front()
        );
        let presented = self.frames.lock().unwrap().0.len();
        if let Some(text) = self.script.front() {
            if let Some((x, y)) = self.find(text).filter(|_| presented > 0) {
                self.script.pop_front();
                self.frames.lock().unwrap().1 = presented;
                self.seen = presented;
                self.idle = 0;
                return Ok(click(x, y));
            }
        } else {
            if presented == self.seen {
                self.idle += 1;
            } else {
                self.seen = presented;
                self.idle = 0;
            }
            let since = presented - self.frames.lock().unwrap().1;
            if self.idle >= IDLE_TURNS || since >= PRESENTS {
                wake.request_stop();
            }
        }
        wake.wait(Some(Duration::from_millis(1)));
        Ok(None)
    }
}

/// Run `root` at `size`, click each text of `script` in turn, and return
/// the geometry of every frame presented after the last click, or of every
/// frame when the script is empty.
fn settle(
    root: impl RootComponent + 'static,
    size: (u16, u16),
    script: &[&'static str],
) -> Vec<Geometry> {
    let frames = Frames::default();
    let backend = Settling {
        inner: DebugBackend::new(size.0, size.1),
        script: script.iter().copied().collect(),
        frames: frames.clone(),
        seen: 0,
        idle: 0,
        deadline: Instant::now() + HANG_GUARD,
    };
    App::builder()
        .backend(backend)
        .root(root)
        .build()
        .unwrap()
        .run()
        .unwrap();
    let (all, from) = Arc::try_unwrap(frames).ok().unwrap().into_inner().unwrap();
    all[from..].to_vec()
}

/// LAY-004: every frame after the third has the third's geometry.
fn assert_settled(frames: &[Geometry], what: &str) {
    assert!(!frames.is_empty(), "{what}: nothing was presented");
    let Some(settled) = frames.get(2) else {
        return;
    };
    for (index, frame) in frames.iter().enumerate().skip(3) {
        let moved: Vec<_> = frame
            .iter()
            .zip(settled)
            .filter(|(now, then)| now != then)
            .take(3)
            .collect();
        assert!(
            moved.is_empty() && frame.len() == settled.len(),
            "{what}: present {} differs from present 3; elements as (index, [left, top, width, height]), now and then: {moved:?}",
            index + 1
        );
    }
}

/// A data table in a box whose height follows its content, as a card's does.
struct Table;
impl RootComponent for Table {
    fn render(&self) -> Element {
        div()
            .class("flex-col w-full h-full")
            .child(
                div()
                    .class("flex-col w-full shrink-0")
                    .child(
                        data_table()
                            .column("Widget", "widget")
                            .column("State", "state")
                            .simple_row(vec![("widget", "Input"), ("state", "Ready")])
                            .simple_row(vec![("widget", "Layout"), ("state", "Ready")])
                            .build(),
                    )
                    .build(),
            )
            .build()
    }
}

#[test]
fn lay_004_the_data_table_keeps_its_height_with_its_panels_open() {
    let scripts: [&[&'static str]; 3] = [&["Filters"], &["Columns"], &["Filters", "Columns"]];
    for script in scripts {
        let frames = settle(Table, (100, 60), script);
        assert_settled(
            &frames,
            &format!("the data table after a click on {script:?}"),
        );
    }
}

#[test]
fn lay_004_a_catalog_page_settles() {
    for page in [CatalogPage::Input, CatalogPage::Layout, CatalogPage::Data] {
        for size in [(100, 30), (160, 45), (240, 60)] {
            let mut catalog = Catalog::default();
            catalog.set_page(page);
            let frames = settle(catalog, size, &[]);
            assert_settled(&frames, &format!("the catalog's {page:?} page at {size:?}"));
        }
    }
}
