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

pub(super) fn paint(element: &Element, size: (u16, u16)) -> Result<DebugFrame> {
    if size.0 == 0
        || size.1 == 0
        || usize::from(size.0) * usize::from(size.1) > CellFrame::MAX_CELLS
    {
        return Err(ReactiveError::invalid_parameter(
            "debug frame dimensions are invalid",
        ));
    }
    let pool = Rc::new(RefCell::new(GraphemePool::new()));
    let mut buffer = OptimizedBuffer::new(
        u32::from(size.0),
        u32::from(size.1),
        InitOptions::new(pool.clone()),
    )
    .map_err(|error| ReactiveError::resource(format!("debug frame allocation: {error:?}")))?;
    let mut hits = vec![0; usize::from(size.0) * usize::from(size.1)];
    let mut cache = crate::layout::paint_tree::suprtui::LayoutCache::default();
    let geometry = crate::layout::paint_tree::suprtui::paint_frame(
        element_to_paintspec(element)?,
        &mut buffer,
        &mut hits,
        &mut cache,
        ImageOutputOptions::default(),
    )?;
    let mut surface = Surface::new(usize::from(size.0), usize::from(size.1));
    let mut text = FrameText::with_capacity(usize::from(size.0) * usize::from(size.1));
    let graphemes = pool.borrow();
    for y in 0..u32::from(size.1) {
        for x in 0..u32::from(size.0) {
            let cell = buffer
                .get(x, y)
                .expect("coordinates are within the allocated frame");
            let first = if is_continuation_char(cell.char) {
                text.push("");
                ' '
            } else if is_grapheme_char(cell.char) {
                let bytes = graphemes
                    .get(grapheme_id_from_char(cell.char))
                    .map_err(|error| {
                        ReactiveError::resource(format!("debug frame grapheme: {error:?}"))
                    })?;
                let content = std::str::from_utf8(bytes).map_err(|error| {
                    ReactiveError::resource(format!("debug frame text: {error}"))
                })?;
                text.push(content);
                content.chars().next().unwrap_or(' ')
            } else {
                let content = char::from_u32(cell.char).unwrap_or(' ');
                text.push_char(content);
                content
            };
            surface.set(
                x as usize,
                y as usize,
                Cell {
                    ch: first,
                    fg: color(cell.fg),
                    bg: color(cell.bg),
                    attr: attributes(cell.attributes),
                    ..Default::default()
                },
            );
        }
    }
    drop(graphemes);
    Ok(DebugFrame {
        surface,
        text,
        geometry,
    })
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
