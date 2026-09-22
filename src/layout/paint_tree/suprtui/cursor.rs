//! Native cursor ownership follows the same painted cells as text.
use crate::component::element::TextCursor;
use ::suprtui::{ansi, render::CursorState};

#[derive(Default)]
pub(super) struct Layer {
    pub state: Option<CursorState>,
}
impl Layer {
    pub fn cover(&mut self, x: i32, y: i32, source: ansi::Rgba) {
        let Some(cursor) = self.state.as_mut() else {
            return;
        };
        if cursor.x != x as u32 || cursor.y != y as u32 {
            return;
        }
        if ansi::alpha(source) == 255 {
            self.state = None;
        } else {
            cursor.color = ::suprtui::buffer::draw::blend_colors(source, cursor.color, None);
        }
    }
    pub fn paint(
        &mut self,
        request: Option<TextCursor>,
        local: (i32, i32),
        width: u8,
        screen: (i32, i32),
        color: ansi::Rgba,
    ) {
        let Some(request) = request else { return };
        let column = i32::from(request.column);
        if local.1 != 0 || column < local.0 || column >= local.0 + i32::from(width) {
            return;
        }
        self.state = Some(CursorState {
            x: (screen.0 + column - local.0) as u32,
            y: screen.1 as u32,
            visible: true,
            style: request.style,
            blinking: false,
            color,
        });
    }
}
