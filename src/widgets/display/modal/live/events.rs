use super::*;
use crate::event::types::{FocusEventKind, KeyEventKind, MouseButton, Position};
use crate::widgets::display::overlay::local_rect;

pub(super) fn activate(event: &Event) -> bool {
    matches!(event, Event::Mouse(mouse) if mouse.button == MouseButton::Left && matches!(mouse.kind, MouseEventKind::Down | MouseEventKind::Click))
}

impl Runtime {
    pub(super) fn escape(
        &self,
        event: &Event,
        props: &ModalProps,
        escape_closable: bool,
    ) -> EventResult {
        let Event::Key(key) = event else {
            return EventResult::Ignored;
        };
        if !self.data.lock().unwrap().visible
            || key.code != KeyCode::Escape
            || key.kind == KeyEventKind::Release
        {
            return EventResult::Ignored;
        }
        if props.closable
            && escape_closable
            && props.keyboard_navigation
            && key.kind == KeyEventKind::Press
            && !key.repeat
        {
            self.close(props, ModalCloseReason::EscapeKey);
        }
        EventResult::Consumed
    }
    pub(super) fn begin_drag(&self, event: &Event, handle: Option<ResizeHandle>) -> EventResult {
        let Event::Mouse(mouse) = event else {
            return EventResult::Ignored;
        };
        if mouse.kind != MouseEventKind::Down || mouse.button != MouseButton::Left {
            return EventResult::Ignored;
        }
        let Position::Cell { x, y } = mouse.position else {
            return EventResult::Ignored;
        };
        let mut data = self.data.lock().unwrap();
        if !data.visible {
            return EventResult::Ignored;
        }
        let Some(root) = data.measurements.root else {
            return EventResult::Ignored;
        };
        let point = local_rect(
            root,
            Rect {
                left: f32::from(x),
                right: f32::from(x),
                top: f32::from(y),
                bottom: f32::from(y),
            },
        );
        data.observed.dragging = handle.is_none();
        data.observed.resizing = handle.is_some();
        data.observed.resize_handle = handle.clone();
        data.drag = Some(Drag {
            handle,
            start: (point.left, point.top),
            rect: data.rect,
        });
        EventResult::Consumed
    }
    pub(super) fn drag(&self, event: &Event) -> EventResult {
        let mut data = self.data.lock().unwrap();
        if data.drag.is_none() {
            return EventResult::Ignored;
        }
        if matches!(event, Event::Focus(focus) if focus.kind == FocusEventKind::Lost)
            || matches!(event, Event::Mouse(mouse) if matches!(mouse.kind, MouseEventKind::Up | MouseEventKind::Leave))
        {
            data.drag = None;
            data.observed.dragging = false;
            data.observed.resizing = false;
            data.observed.resize_handle = None;
            return EventResult::Consumed;
        }
        let Event::Mouse(mouse) = event else {
            return EventResult::Ignored;
        };
        if !matches!(mouse.kind, MouseEventKind::Move | MouseEventKind::Drag) {
            return EventResult::Ignored;
        }
        let Position::Cell { x, y } = mouse.position else {
            return EventResult::Ignored;
        };
        let Some(root) = data.measurements.root else {
            return EventResult::Ignored;
        };
        let point = local_rect(
            root,
            Rect {
                left: f32::from(x),
                right: f32::from(x),
                top: f32::from(y),
                bottom: f32::from(y),
            },
        );
        let drag = data.drag.as_ref().unwrap();
        let (dx, dy) = (point.left - drag.start.0, point.top - drag.start.1);
        let mut rect = drag.rect;
        let bounds = data.bounds;
        match &drag.handle {
            None => {
                let (width, height) = (rect.right - rect.left, rect.bottom - rect.top);
                rect.left =
                    (rect.left + dx).clamp(bounds.left, (bounds.right - width).max(bounds.left));
                rect.top =
                    (rect.top + dy).clamp(bounds.top, (bounds.bottom - height).max(bounds.top));
                rect.right = rect.left + width;
                rect.bottom = rect.top + height;
            }
            Some(handle) => {
                use ResizeHandle::*;
                if matches!(handle, Left | TopLeft | BottomLeft) {
                    rect.left =
                        (rect.left + dx).clamp(bounds.left, (rect.right - 1.0).max(bounds.left));
                }
                if matches!(handle, Right | TopRight | BottomRight) {
                    rect.right =
                        (rect.right + dx).clamp((rect.left + 1.0).min(bounds.right), bounds.right);
                }
                if matches!(handle, Top | TopLeft | TopRight) {
                    rect.top =
                        (rect.top + dy).clamp(bounds.top, (rect.bottom - 1.0).max(bounds.top));
                }
                if matches!(handle, Bottom | BottomLeft | BottomRight) {
                    rect.bottom = (rect.bottom + dy)
                        .clamp((rect.top + 1.0).min(bounds.bottom), bounds.bottom);
                }
                data.size = Some((
                    (rect.right - rect.left) as u16,
                    (rect.bottom - rect.top) as u16,
                ));
            }
        }
        data.position = Some((
            (rect.left - bounds.left).max(0.0) as u16,
            (rect.top - bounds.top).max(0.0) as u16,
        ));
        drop(data);
        self.wake();
        EventResult::Consumed
    }
}
