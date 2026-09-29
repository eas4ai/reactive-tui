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
    /// The rectangle the backend reports for `element`, whatever other
    /// elements painted over it. It fails when the rectangle holds no cell.
    fn bounds(&self, element: usize, what: &str) -> Cells {
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
        assert!(
            bounds[2] > 0.0 && bounds[3] > 0.0,
            "{what}: element {element} painted no cell: {bounds:?}"
        );
        Cells {
            left: bounds[0] as usize,
            top: bounds[1] as usize,
            width: bounds[2] as usize,
            height: bounds[3] as usize,
        }
    }

    /// The rectangle `element` painted. It fails when the element painted
    /// nothing or painted cells that do not form one full rectangle.
    fn cells(&self, element: usize, what: &str) -> Cells {
        let Some(hits) = &self.hits else {
            return self.bounds(element, what);
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

/// `container` at `place` counted down the screen: at its first row, inside
/// a padding of one cell, or below a box that takes a third of the screen's
/// height. The container must take the height it is given.
fn placed_down(place: Place, container: Element) -> Element {
    match place {
        Place::Whole | Place::Padded => placed(place, container),
        Place::AfterAThird => Element::layout(LayoutType::Flex)
            .with_class("flex-col w-full h-full")
            .with_children(vec![
                Element::text("t").with_class("h-1/3 w-1 shrink-0"),
                Element::layout(LayoutType::Flex)
                    .with_class("flex-col flex-1 min-h-0 w-full")
                    .with_children(vec![container]),
            ]),
    }
}

/// The rows a container at `place` has for its content on a screen of
/// `height` cells: its first row and the row after its last.
fn content_down(place: Place, height: usize, frame: &Painted) -> (usize, usize) {
    match place {
        Place::Whole => (0, height),
        Place::Padded => (1, height - 1),
        Place::AfterAThird => {
            let third = frame.cells(1, "the box of a third");
            assert_eq!(third.top, 0, "the box of a third starts at row 0");
            assert!(
                third.height.abs_diff(height / 3) <= 1,
                "the box of a third is {} of {height} cells tall",
                third.height
            );
            (third.bottom(), height)
        }
    }
}

/// The screen heights at which a container of `tracks` tracks with `gap`
/// holds its gaps and one row for each track at every place.
fn heights(tracks: usize, gap: usize) -> std::ops::RangeInclusive<u16> {
    let least = tracks + (tracks - 1) * gap;
    (least * 3 / 2 + 3) as u16..=96
}

/// `cells` with across and down changed for each other, so a column is read
/// as a row.
fn turned(cells: &[Cells]) -> Vec<Cells> {
    cells
        .iter()
        .map(|cells| Cells {
            left: cells.top,
            top: cells.left,
            width: cells.height,
            height: cells.width,
        })
        .collect()
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
fn lay_001_a_grid_keeps_its_gap_at_every_height() {
    for painter in PAINTERS {
        for place in PLACES {
            for rows in 2..=6usize {
                for gap in 1..=2usize {
                    for height in heights(rows, gap) {
                        let items = (0..rows * 2).map(|_| leaf("min-w-0 min-h-0")).collect();
                        let class = format!("w-full h-full grid grid-flow-col grid-rows-{rows}");
                        let root = placed_down(place, spaced(LayoutType::Grid, &class, gap, items));
                        let frame = paint(painter, &root, (24, height));
                        let what = format!(
                            "{painter:?}, {place:?}, {rows} rows, gap {gap}, height {height}"
                        );
                        let cells = rectangles(&frame, &root, &what);
                        let span = content_down(place, height.into(), &frame);
                        let (first, second) = cells.split_at(rows);
                        assert_row(&turned(first), gap, span, &what);
                        assert_row(&turned(second), gap, span, &what);
                        assert_eq!(
                            second[0].left as i64 - first[0].right() as i64,
                            gap as i64,
                            "{what}: the gap between two columns"
                        );
                    }
                }
            }
        }
    }
}

/// Where the tracks of a grid start, read from the bounds the backend
/// reports for its items: an item larger than its track is painted over by
/// the items after it, so its painted cells are no full rectangle.
fn assert_equal_steps(starts: &[usize], first: usize, what: &str) {
    assert_eq!(starts[0], first, "{what}: tracks start at {starts:?}");
    let steps: Vec<usize> = starts.windows(2).map(|pair| pair[1] - pair[0]).collect();
    let longest = steps.iter().max().unwrap();
    let shortest = steps.iter().min().unwrap();
    assert!(
        longest - shortest <= 1,
        "{what}: tracks start at {starts:?}"
    );
}

/// An item that is larger than its share does not widen its track: the
/// tracks stay equal.
#[test]
fn lay_001_tracks_stay_equal_under_an_item_larger_than_its_share() {
    for painter in PAINTERS {
        for place in PLACES {
            for tracks in 2..=6usize {
                for width in [40u16, 41, 66, 100, 161, 240] {
                    let items = (0..tracks)
                        .map(|item| leaf(if item == 1 { "w-100 h-1" } else { "h-1" }))
                        .collect();
                    let class = format!("w-full grid grid-cols-{tracks}");
                    let root = placed(place, spaced(LayoutType::Grid, &class, 1, items));
                    let frame = paint(painter, &root, (width, 6));
                    let what = format!("{painter:?}, {place:?}, {tracks} columns, width {width}");
                    let starts: Vec<usize> = measured(&root)
                        .into_iter()
                        .map(|index| frame.bounds(index, &what).left)
                        .collect();
                    let first = content(place, width.into(), &frame).0;
                    assert_equal_steps(&starts, first, &what);
                }
                for height in [24u16, 25, 33, 60] {
                    let items = (0..tracks)
                        .map(|item| leaf(if item == 1 { "h-40" } else { "" }))
                        .collect();
                    let class = format!("w-full h-full grid grid-rows-{tracks}");
                    let root = placed_down(place, spaced(LayoutType::Grid, &class, 1, items));
                    let frame = paint(painter, &root, (24, height));
                    let what = format!("{painter:?}, {place:?}, {tracks} rows, height {height}");
                    let starts: Vec<usize> = measured(&root)
                        .into_iter()
                        .map(|index| frame.bounds(index, &what).top)
                        .collect();
                    let first = content_down(place, height.into(), &frame).0;
                    assert_equal_steps(&starts, first, &what);
                }
            }
        }
    }
}

/// The rows of spans a grid of `tracks` tracks is filled with: one item
/// over all of them, one over all but the last beside one, and one for
/// each track.
fn spans_of(tracks: usize) -> Vec<Vec<usize>> {
    vec![vec![tracks], vec![tracks - 1, 1], vec![1; tracks]]
}

/// Each item of `painted` starts where its first track starts and ends
/// where its last track ends; the tracks are the last row of `painted`.
fn assert_spans(spans: &[Vec<usize>], painted: &[Vec<Cells>], what: &str) {
    let tracks = painted.last().unwrap();
    for (row, painted) in spans.iter().zip(painted) {
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

#[test]
fn lay_001_an_item_that_spans_tracks_ends_where_its_last_track_ends() {
    for painter in PAINTERS {
        for place in PLACES {
            for tracks in 2..=6usize {
                for gap in 1..=2usize {
                    let spans = spans_of(tracks);
                    for width in 40..=512u16 {
                        let items = spans
                            .iter()
                            .flatten()
                            .map(|span| leaf(&format!("min-w-0 h-1 col-span-{span}")))
                            .collect();
                        let class = format!("w-full grid grid-cols-{tracks}");
                        let root = placed(place, spaced(LayoutType::Grid, &class, gap, items));
                        let frame = paint(painter, &root, (width, 10));
                        let what = format!(
                            "{painter:?}, {place:?}, {tracks} columns, gap {gap}, width {width}"
                        );
                        let mut cells = rectangles(&frame, &root, &what).into_iter();
                        let rows: Vec<Vec<Cells>> = spans
                            .iter()
                            .map(|row| row.iter().map(|_| cells.next().unwrap()).collect())
                            .collect();
                        let span = content(place, width.into(), &frame);
                        assert_row(rows.last().unwrap(), gap, span, &what);
                        assert_spans(&spans, &rows, &what);
                    }
                    for height in heights(tracks, gap) {
                        let items = spans
                            .iter()
                            .flatten()
                            .map(|span| leaf(&format!("min-w-0 min-h-0 row-span-{span}")))
                            .collect();
                        let class = format!("w-full h-full grid grid-flow-col grid-rows-{tracks}");
                        let root = placed_down(place, spaced(LayoutType::Grid, &class, gap, items));
                        let frame = paint(painter, &root, (24, height));
                        let what = format!(
                            "{painter:?}, {place:?}, {tracks} rows, gap {gap}, height {height}"
                        );
                        let mut cells = rectangles(&frame, &root, &what).into_iter();
                        let columns: Vec<Vec<Cells>> = spans
                            .iter()
                            .map(|column| {
                                let column: Vec<Cells> =
                                    column.iter().map(|_| cells.next().unwrap()).collect();
                                turned(&column)
                            })
                            .collect();
                        let span = content_down(place, height.into(), &frame);
                        assert_row(columns.last().unwrap(), gap, span, &what);
                        assert_spans(&spans, &columns, &what);
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
                for place in PLACES {
                    for height in heights(items, gap) {
                        let children = (0..items).map(|_| leaf("flex-1 min-h-0 w-full")).collect();
                        let root = placed_down(
                            place,
                            spaced(LayoutType::Flex, "flex-col w-full h-full", gap, children),
                        );
                        let frame = paint(painter, &root, (20, height));
                        let what = format!(
                            "{painter:?}, {place:?}, {items} items, gap {gap}, height {height}"
                        );
                        let cells = rectangles(&frame, &root, &what);
                        let span = content_down(place, height.into(), &frame);
                        assert_row(&turned(&cells), gap, span, &what);
                    }
                }
            }
        }
    }
}

/// The runs of painted cells in the first row of `surface`: the first
/// column and the width of each run of one background color. A cell that
/// was not painted is black.
fn runs(surface: &reactive_tui::core::surface::Surface, width: usize) -> Vec<(usize, usize)> {
    let color = |x: usize| {
        let background = surface.get(x, 0).bg;
        [background.r, background.g, background.b].map(|channel| (channel * 255.0).round() as u8)
    };
    let mut runs: Vec<(usize, usize)> = Vec::new();
    let mut open: Option<(usize, [u8; 3])> = None;
    for x in 0..=width {
        let here = (x < width)
            .then(|| color(x))
            .filter(|color| *color != [0, 0, 0]);
        if let Some((start, painted)) = open {
            if here != Some(painted) {
                runs.push((start, x - start));
                open = None;
            }
        }
        if open.is_none() {
            open = here.map(|painted| (x, painted));
        }
    }
    runs
}

/// The older painter, which the debug backend uses for patched frames and
/// the screen transitions use, reads its classes from a `NodeSpec`.
#[test]
fn lay_001_the_older_painter_keeps_the_gap_of_a_grid() {
    use reactive_tui::core::surface::Surface;
    use reactive_tui::layout::paint_tree::{layout_and_paint, NodeSpec};
    use std::borrow::Cow;
    let node = |class: String, children: Vec<NodeSpec<'static>>| NodeSpec {
        class: Cow::from(class),
        text: children.is_empty().then(|| Cow::from(" ")),
        children,
    };
    for columns in 2..=6usize {
        for gap in 1..=2usize {
            for width in 40..=512usize {
                let items = (0..columns)
                    .map(|item| {
                        let color = ["bg-blue-500", "bg-green-500"][item % 2];
                        node(format!("min-w-0 h-1 {color}"), Vec::new())
                    })
                    .collect();
                let root = node(
                    "flex flex-row w-full h-full".into(),
                    vec![
                        node("w-1/3 h-1 shrink-0 bg-red-500".into(), Vec::new()),
                        node(
                            "flex flex-col flex-1 min-w-0 h-full".into(),
                            vec![node(
                                format!("w-full grid grid-cols-{columns} gap-{gap}"),
                                items,
                            )],
                        ),
                    ],
                );
                let mut surface = Surface::new(width, 4);
                layout_and_paint(&root, &mut surface, width);
                let what = format!("{columns} columns, gap {gap}, width {width}");
                let painted = runs(&surface, width);
                assert_eq!(painted.len(), columns + 1, "{what}: {painted:?}");
                let third = painted[0];
                assert_eq!(third.0, 0, "{what}: {painted:?}");
                let row: Vec<Cells> = painted[1..]
                    .iter()
                    .map(|(left, width)| Cells {
                        left: *left,
                        top: 0,
                        width: *width,
                        height: 1,
                    })
                    .collect();
                assert_row(&row, gap, (third.0 + third.1, width), &what);
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

        let mut items = vec![leaf("min-w-0 min-h-0 row-span-full")];
        items.extend((0..4).map(|_| leaf("min-w-0 min-h-0")));
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
        assert_eq!(
            (
                cells[0].left,
                cells[0].right(),
                cells[1].left,
                cells[1].right()
            ),
            (0, 10, 11, 21),
            "{what}: the item takes one column and the rows the other: {cells:?}"
        );
    }
}

/// A full span counts the tracks the grid's class names. The rows a grid
/// adds for items beyond its columns are not among them, so in a grid with
/// no `grid-rows-N` a full span down is one row, and it adds none.
#[test]
fn lay_003_a_full_span_counts_the_tracks_the_class_names() {
    for painter in PAINTERS {
        let mut items = vec![leaf("min-w-0 h-1 row-span-full")];
        items.extend((0..5).map(|_| leaf("min-w-0 h-1")));
        let root = spaced(LayoutType::Grid, "w-full grid grid-cols-2", 1, items);
        let what = format!("{painter:?}, row-span-full without grid-rows");
        let cells = rectangles(&paint(painter, &root, (21, 12)), &root, &what);
        assert_eq!((cells[0].top, cells[0].height), (0, 1), "{what}: {cells:?}");
        // Six items in two columns: three rows, a gap of one between them.
        let tops: Vec<usize> = cells.iter().map(|item| item.top).collect();
        assert_eq!(tops, [0, 0, 2, 2, 4, 4], "{what}: {cells:?}");
    }
}

/// Classes are applied in the order they are written, so of two that set
/// the same thing the later one decides.
#[test]
fn lay_003_the_later_of_two_classes_decides() {
    for painter in PAINTERS {
        for (span, wide) in [
            ("col-span-full col-span-2", 2),
            ("col-span-2 col-span-full", 4),
        ] {
            let mut items = vec![leaf(&format!("min-w-0 h-1 {span}"))];
            items.extend((0..4).map(|_| leaf("min-w-0 h-1")));
            let root = spaced(LayoutType::Grid, "w-full grid grid-cols-4", 1, items);
            let what = format!("{painter:?}, {span}");
            let cells = rectangles(&paint(painter, &root, (83, 6)), &root, &what);
            // Four tracks of 20 cells with gaps of one.
            assert_eq!(
                (cells[0].left, cells[0].right()),
                (0, wide * 21 - 1),
                "{what}: {cells:?}"
            );
        }
        for (tracks, columns) in [
            ("grid-cols-auto-fit-20 grid-cols-3", 3),
            ("grid-cols-3 grid-cols-auto-fit-20", 4),
        ] {
            let items = (0..8).map(|_| leaf("min-w-0 h-1")).collect();
            let root = Element::layout(LayoutType::Grid)
                .with_class(format!("w-full grid {tracks}"))
                .with_children(items);
            let what = format!("{painter:?}, {tracks}");
            let cells = rectangles(&paint(painter, &root, (90, 6)), &root, &what);
            let first_row = cells.iter().filter(|item| item.top == cells[0].top).count();
            assert_eq!(first_row, columns, "{what}: {cells:?}");
        }
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

/// Every presented frame's geometry, how many frames were presented when
/// the last click was sent, and the text of the last frame.
type Frames = Arc<Mutex<(Vec<Geometry>, usize, String)>>;

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
        let mut frames = self.frames.lock().unwrap();
        frames.0.push(geometry(nodes));
        frames.2 = self.inner.screen_content();
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
    settle_and_read(root, size, script).0
}

/// `settle`, and the text of the last frame.
fn settle_and_read(
    root: impl RootComponent + 'static,
    size: (u16, u16),
    script: &[&'static str],
) -> (Vec<Geometry>, String) {
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
    let (all, from, text) = Arc::try_unwrap(frames).ok().unwrap().into_inner().unwrap();
    (all[from..].to_vec(), text)
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

fn table() -> Element {
    data_table()
        .column("Widget", "widget")
        .column("State", "state")
        .simple_row(vec![("widget", "Input"), ("state", "Ready")])
        .simple_row(vec![("widget", "Layout"), ("state", "Ready")])
        .build()
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
                    .child(table())
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

/// A data table in a box of a given height.
struct TableOf(u16);
impl RootComponent for TableOf {
    fn render(&self) -> Element {
        div()
            .class("flex-col w-full h-full")
            .child(
                div()
                    .class(&format!("flex-col w-full shrink-0 h-{}", self.0))
                    .child(table())
                    .build(),
            )
            .build()
    }
}

/// The rows of `text` from the one that holds `first` to the one before the
/// one that holds `last`, with the rows between them that hold no text.
fn rows_between(text: &str, first: &str, last: &str) -> (usize, usize) {
    let rows: Vec<&str> = text.lines().collect();
    let from = rows
        .iter()
        .position(|row| row.contains(first))
        .unwrap_or_else(|| panic!("no row holds {first:?}:\n{text}"));
    let to = rows
        .iter()
        .position(|row| row.contains(last))
        .unwrap_or_else(|| panic!("no row holds {last:?}:\n{text}"));
    assert!(from < to, "{first:?} is not above {last:?}:\n{text}");
    let empty = rows[from..to]
        .iter()
        .filter(|row| row.trim().is_empty())
        .count();
    (to - from, empty)
}

#[test]
fn lay_004_a_panel_takes_the_rows_of_its_content() {
    // Two columns give the filter panel five rows: a row of buttons and a
    // field for each, and the button that clears the filters. The table's
    // frame, its header and its two rows come right below.
    let (_, text) = settle_and_read(Table, (100, 60), &["Filters"]);
    assert_eq!(
        rows_between(&text, "Widget: contains", "Clear filters"),
        (4, 0),
        "the filter panel's rows above its last:\n{text}"
    );
    let (rows, empty) = rows_between(&text, "Clear filters", "Input");
    assert_eq!(
        empty, 0,
        "{rows} rows from the panel's last row to the table's first, {empty} of them empty:\n{text}"
    );
    // The column panel has a row for each column.
    let (_, text) = settle_and_read(Table, (100, 60), &["Filters", "Columns"]);
    assert_eq!(
        rows_between(&text, "[x] Widget", "Input").1,
        0,
        "empty rows under the column panel:\n{text}"
    );
    // In a box of eight rows a panel takes half of them at most: four of
    // the filter panel's five rows show, and its last is scrolled to.
    let (_, text) = settle_and_read(TableOf(8), (100, 60), &["Filters"]);
    assert!(
        text.contains("Filter Widget") && !text.contains("Clear filters"),
        "the filter panel is not held to half of eight rows:\n{text}"
    );
}

#[test]
fn lay_003_auto_fit_makes_rows_of_a_least_height() {
    for painter in PAINTERS {
        let items = (0..8).map(|_| leaf("min-w-0 min-h-0")).collect();
        let root = Element::layout(LayoutType::Grid)
            .with_class("w-full h-full grid grid-flow-col grid-rows-auto-fit-3")
            .with_children(items);
        let what = format!("{painter:?}, grid-rows-auto-fit-3");
        let cells = rectangles(&paint(painter, &root, (20, 13)), &root, &what);
        let first_column = cells
            .iter()
            .filter(|item| item.left == cells[0].left)
            .count();
        assert_eq!(first_column, 4, "{what}: rows in 13 cells: {cells:?}");
        assert!(
            cells.iter().all(|item| item.height >= 3),
            "{what}: a row lower than 3 cells: {cells:?}"
        );
    }
}
