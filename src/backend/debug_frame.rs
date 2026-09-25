use super::{CellFrame, FrameCell, ImageOutputOptions, PresentedGeometry};
use crate::{
    component::{bridge::element_to_paintspec, Element},
    core::surface::{Attr, Cell, Rgba, Surface},
    error::{ReactiveError, Result},
};
use ::suprtui::{
    ansi,
    buffer::{InitOptions, OptimizedBuffer},
    uni::{
        pool::GraphemePool,
        segments::{grapheme_id_from_char, is_continuation_char, is_grapheme_char},
    },
};
use std::{cell::RefCell, rc::Rc};

pub(super) struct DebugFrame {
    pub surface: Surface,
    pub text: FrameText,
    pub geometry: PresentedGeometry,
}

/// The text of every cell of a presented frame in one buffer, so a frame
/// costs no allocation per cell: a whole grapheme cluster, or an empty
/// string for the second cell of a wide one.
pub(super) struct FrameText {
    text: String,
    /// The end offset in `text` of each cell's text, row-major.
    ends: Vec<u32>,
}

impl FrameText {
    /// Empty the text, keeping its allocation for `cells` more cells.
    fn reuse(mut self, cells: usize) -> Self {
        self.text.clear();
        self.ends.clear();
        self.text.reserve(cells);
        self.ends.reserve(cells);
        self
    }

    fn with_capacity(cells: usize) -> Self {
        Self {
            text: String::with_capacity(cells),
            ends: Vec::with_capacity(cells),
        }
    }

    fn push(&mut self, content: &str) {
        self.text.push_str(content);
        self.ends.push(self.text.len() as u32);
    }

    fn push_char(&mut self, content: char) {
        self.text.push(content);
        self.ends.push(self.text.len() as u32);
    }

    /// The text of the cell at row-major `index`.
    pub fn get(&self, index: usize) -> Option<&str> {
        let end = *self.ends.get(index)? as usize;
        let start = index
            .checked_sub(1)
            .map_or(0, |previous| self.ends[previous] as usize);
        Some(&self.text[start..end])
    }
}

fn color(value: ansi::Rgba) -> Rgba {
    Rgba {
        r: ansi::red_f(value),
        g: ansi::green_f(value),
        b: ansi::blue_f(value),
        a: ansi::alpha_f(value),
    }
}

fn attributes(bits: u32) -> Attr {
    let mut value = Attr::empty();
    for (mask, attribute) in [
        (1, Attr::BOLD),
        (4, Attr::ITALIC),
        (8, Attr::UNDERLINE),
        (32, Attr::REVERSE),
        (128, Attr::STRIKE),
    ] {
        if bits & mask != 0 {
            value |= attribute;
        }
    }
    value
}

/// The buffer, grapheme pool, hit grid and layout cache the debug backend
/// paints with, kept while the frame size holds, as the SuprTUI renderer
/// keeps its own: a frame allocates and first-touches none of them again,
/// and an unchanged layout is reused (PNT-004). A buffer shares its pool
/// through an `Rc`, which a `Send` backend cannot hold, so each painting
/// thread keeps one.
struct Canvas {
    size: (u16, u16),
    pool: Rc<RefCell<GraphemePool<'static>>>,
    buffer: OptimizedBuffer<'static>,
    hits: Vec<u32>,
    cache: crate::layout::paint_tree::suprtui::LayoutCache,
    /// The cells and text of the frame this one replaced on screen, whose
    /// memory the next frame writes into instead of fresh pages.
    spare: Option<(Vec<Cell>, FrameText)>,
}

/// Hand the cells and text of a frame that left the screen back to this
/// thread's canvas, for the next frame painted at its size to reuse.
pub(super) fn recycle(surface: Surface, text: Option<FrameText>) {
    let Some(text) = text else {
        return;
    };
    CANVAS.with_borrow_mut(|slot| {
        if let Some(canvas) = slot {
            let cells = surface.into_cells();
            if cells.len() == usize::from(canvas.size.0) * usize::from(canvas.size.1) {
                canvas.spare = Some((cells, text));
            }
        }
    });
}

thread_local! {
    static CANVAS: RefCell<Option<Canvas>> = const { RefCell::new(None) };
}

/// The last input and output of a conversion, so a run of equal inputs
/// converts once.
struct Memo<I, O> {
    convert: fn(I) -> O,
    last: Option<(I, O)>,
}

impl<I: Copy + PartialEq, O: Copy> Memo<I, O> {
    fn new(convert: fn(I) -> O) -> Self {
        Self {
            convert,
            last: None,
        }
    }

    fn get(&mut self, input: I) -> O {
        match self.last {
            Some((last, output)) if last == input => output,
            _ => {
                let output = (self.convert)(input);
                self.last = Some((input, output));
                output
            }
        }
    }
}

pub(super) fn paint(element: &Element, size: (u16, u16)) -> Result<DebugFrame> {
    if size.0 == 0
        || size.1 == 0
        || usize::from(size.0) * usize::from(size.1) > CellFrame::MAX_CELLS
    {
        return Err(ReactiveError::invalid_parameter(
            "debug frame dimensions are invalid",
        ));
    }
    CANVAS.with_borrow_mut(|slot| {
        if slot.as_ref().is_none_or(|canvas| canvas.size != size) {
            *slot = Some(Canvas::new(size)?);
        }
        let canvas = slot.as_mut().expect("the canvas was just made");
        let frame = canvas.paint(element);
        if frame.is_err() {
            // A failed paint may leave the layout cache half updated.
            *slot = None;
        }
        frame
    })
}

impl Canvas {
    fn new(size: (u16, u16)) -> Result<Self> {
        let pool = Rc::new(RefCell::new(GraphemePool::new()));
        let buffer = OptimizedBuffer::new(
            u32::from(size.0),
            u32::from(size.1),
            InitOptions::new(pool.clone()),
        )
        .map_err(|error| ReactiveError::resource(format!("debug frame allocation: {error:?}")))?;
        Ok(Self {
            size,
            pool,
            buffer,
            hits: vec![0; usize::from(size.0) * usize::from(size.1)],
            cache: crate::layout::paint_tree::suprtui::LayoutCache::default(),
            spare: None,
        })
    }

    fn paint(&mut self, element: &Element) -> Result<DebugFrame> {
        let size = self.size;
        self.hits.fill(0);
        let geometry = crate::layout::paint_tree::suprtui::paint_frame(
            element_to_paintspec(element)?,
            &mut self.buffer,
            &mut self.hits,
            &mut self.cache,
            ImageOutputOptions::default(),
        )?;
        let (width, height) = (usize::from(size.0), usize::from(size.1));
        let (mut cells, mut text) = match self.spare.take() {
            Some((mut cells, text)) => {
                cells.clear();
                (cells, text.reuse(width * height))
            }
            None => (
                Vec::with_capacity(width * height),
                FrameText::with_capacity(width * height),
            ),
        };
        let graphemes = self.pool.borrow();
        // Neighbouring cells usually share colors and attributes, so each is
        // converted when it changes, not once per cell.
        let mut fg = Memo::new(color);
        let mut bg = Memo::new(color);
        let mut attr = Memo::new(attributes);
        for index in 0..width * height {
            let char = self.buffer.char_at(index);
            let first = if is_continuation_char(char) {
                text.push("");
                ' '
            } else if is_grapheme_char(char) {
                let bytes = graphemes
                    .get(grapheme_id_from_char(char))
                    .map_err(|error| {
                        ReactiveError::resource(format!("debug frame grapheme: {error:?}"))
                    })?;
                let content = std::str::from_utf8(bytes).map_err(|error| {
                    ReactiveError::resource(format!("debug frame text: {error}"))
                })?;
                text.push(content);
                content.chars().next().unwrap_or(' ')
            } else {
                let content = char::from_u32(char).unwrap_or(' ');
                text.push_char(content);
                content
            };
            // A control character is stored as U+FFFD, as Surface::set stores it.
            let first = if first.is_control() {
                '\u{fffd}'
            } else {
                first
            };
            cells.push(Cell {
                ch: first,
                fg: fg.get(self.buffer.fg_at(index)),
                bg: bg.get(self.buffer.bg_at(index)),
                attr: attr.get(self.buffer.attributes_at(index)),
                ..Default::default()
            });
        }
        let surface = Surface::from_cells(width, height, cells);
        drop(graphemes);
        Ok(DebugFrame {
            surface,
            text,
            geometry,
        })
    }
}

pub(super) fn cells(frame: &CellFrame) -> DebugFrame {
    let (width, height) = frame.size();
    let mut surface = Surface::new(usize::from(width), usize::from(height));
    let rgb = |[r, g, b]: [u8; 3]| Rgba {
        r: f32::from(r) / 255.0,
        g: f32::from(g) / 255.0,
        b: f32::from(b) / 255.0,
        a: 1.0,
    };
    let mut text = FrameText::with_capacity(frame.cells().len());
    for (
        index,
        FrameCell {
            text: content,
            foreground,
            background,
            attributes: bits,
            ..
        },
    ) in frame.cells().iter().enumerate()
    {
        surface.set(
            index % usize::from(width),
            index / usize::from(width),
            Cell {
                ch: content.chars().next().unwrap_or(' '),
                fg: rgb(*foreground),
                bg: rgb(*background),
                attr: attributes(u32::from(*bits)),
                ..Default::default()
            },
        );
        text.push(content);
    }
    DebugFrame {
        surface,
        text,
        geometry: PresentedGeometry::default(),
    }
}
