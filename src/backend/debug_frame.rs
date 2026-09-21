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
    pub text: Vec<String>,
    pub geometry: PresentedGeometry,
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
    let geometry = crate::layout::paint_tree::suprtui::paint_frame(
        &element_to_paintspec(element)?,
        &mut buffer,
        ImageOutputOptions::default(),
    )?;
    let mut surface = Surface::new(usize::from(size.0), usize::from(size.1));
    let mut text = Vec::with_capacity(usize::from(size.0) * usize::from(size.1));
    for y in 0..u32::from(size.1) {
        for x in 0..u32::from(size.0) {
            let cell = buffer
                .get(x, y)
                .expect("coordinates are within the allocated frame");
            let content = if is_continuation_char(cell.char) {
                String::new()
            } else if is_grapheme_char(cell.char) {
                let pool = pool.borrow();
                let bytes = pool
                    .get(grapheme_id_from_char(cell.char))
                    .map_err(|error| {
                        ReactiveError::resource(format!("debug frame grapheme: {error:?}"))
                    })?;
                std::str::from_utf8(bytes)
                    .map_err(|error| ReactiveError::resource(format!("debug frame text: {error}")))?
                    .to_owned()
            } else {
                char::from_u32(cell.char).unwrap_or(' ').to_string()
            };
            surface.set(
                x as usize,
                y as usize,
                Cell {
                    ch: content.chars().next().unwrap_or(' '),
                    fg: color(cell.fg),
                    bg: color(cell.bg),
                    attr: attributes(cell.attributes),
                    ..Default::default()
                },
            );
            text.push(content);
        }
    }
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
    let mut text = Vec::with_capacity(frame.cells().len());
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
        text.push(content.clone());
    }
    DebugFrame {
        surface,
        text,
        geometry: PresentedGeometry::default(),
    }
}
