use super::*;
use crate::event::types::{FocusEventKind, KeyEventKind, MouseButton, Position};
use crate::widgets::display::overlay::local_rect;

pub(super) fn activate(event: &Event) -> bool {
    matches!(event, Event::Mouse(mouse) if mouse.button == MouseButton::Left && mouse.kind == MouseEventKind::Down)
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
        // The keys mean what the active keymap says (KEY-001).
        let action = crate::keymap::Keymap::active().action(key);
        if !self.data.lock().unwrap().visible
            || action != Some(crate::keymap::Action::Cancel)
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
    /// The keys for what the pointer does to a box that can be dragged or
    /// resized (BAR-003): Alt with an arrow moves it one cell, Alt and
    /// Shift with an arrow make it one cell wider, narrower, taller or
    /// shorter.
    pub(super) fn nudge(&self, event: &Event, props: &ModalProps) -> EventResult {
        let Event::Key(key) = event else {
            return EventResult::Ignored;
        };
        if !key.modifiers.alt
            || key.modifiers.ctrl
            || key.modifiers.meta
            || key.kind == KeyEventKind::Release
            || !props.keyboard_navigation
        {
            return EventResult::Ignored;
        }
        // Alt is the modal's own modifier: the key without it means what the
        // active keymap says (KEY-001), and Shift is read as its variant.
        let mut plain = key.clone();
        plain.modifiers.alt = false;
        let action = crate::keymap::Keymap::active().action_shifted(&plain);
        use crate::keymap::Action;
        let (dx, dy, resize) = match action {
            Some((Action::Left, shifted)) => (-1.0, 0.0, shifted),
            Some((Action::Right, shifted)) => (1.0, 0.0, shifted),
            Some((Action::Up, shifted)) => (0.0, -1.0, shifted),
            Some((Action::Down, shifted)) => (0.0, 1.0, shifted),
            _ => return EventResult::Ignored,
        };
        if (resize && !props.resizable) || (!resize && !props.draggable) {
            return EventResult::Ignored;
        }
        let mut data = self.data.lock().unwrap();
        if !data.visible || data.measurements.body.is_none() {
            return EventResult::Ignored;
        }
        let bounds = data.bounds;
        let mut rect = data.rect;
        if resize {
            rect.right = (rect.right + dx).clamp((rect.left + 1.0).min(bounds.right), bounds.right);
            rect.bottom =
                (rect.bottom + dy).clamp((rect.top + 1.0).min(bounds.bottom), bounds.bottom);
            data.size = Some((
                (rect.right - rect.left) as u16,
                (rect.bottom - rect.top) as u16,
            ));
        } else {
            let (width, height) = (rect.right - rect.left, rect.bottom - rect.top);
            rect.left =
                (rect.left + dx).clamp(bounds.left, (bounds.right - width).max(bounds.left));
            rect.top = (rect.top + dy).clamp(bounds.top, (bounds.bottom - height).max(bounds.top));
        }
        data.position = Some((
            (rect.left - bounds.left).max(0.0) as u16,
            (rect.top - bounds.top).max(0.0) as u16,
        ));
        drop(data);
        self.wake();
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
        // Motion with no button held means the release was lost, as the
        // router reads it too (INP-003).
        if matches!(event, Event::Focus(focus) if focus.kind == FocusEventKind::Lost)
            || matches!(event, Event::Mouse(mouse) if matches!(mouse.kind, MouseEventKind::Up | MouseEventKind::Leave | MouseEventKind::Move))
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
        if mouse.kind != MouseEventKind::Drag {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::types::{KeyCode, KeyEvent};

    /// KEY-001: the modal closes on the key the active keymap binds to
    /// Cancel, and no longer on the key it replaced.
    #[test]
    #[serial_test::serial(keymap)]
    fn key_001_modal_reads_its_keys_through_the_keymap() {
        use crate::keymap::{Action, KeyBinding, Keymap};
        let mut keymap = Keymap::default();
        keymap.rebind(Action::Cancel, [KeyBinding::new(KeyCode::F(2))]);
        let _scope = Keymap::scoped(keymap);
        let reasons = Arc::new(Mutex::new(Vec::new()));
        let heard = reasons.clone();
        let props = ModalProps {
            visible: true,
            on_close: Some(Arc::new(move |reason| heard.lock().unwrap().push(reason))),
            ..Default::default()
        };
        let child = LiveModal::new(LiveProps {
            config: props.clone(),
            seed: ModalState::default(),
            spoken: crate::accessibility::Node::new(crate::accessibility::Role::Dialog),
            motion: None,
            on_presented: None,
            escape_closable: true,
        });
        child.0.sample(&props, Instant::now());
        let old = child
            .0
            .escape(&Event::Key(KeyEvent::new(KeyCode::Escape)), &props, true);
        let closed_by_old = reasons.lock().unwrap().clone();
        let rebound = child
            .0
            .escape(&Event::Key(KeyEvent::new(KeyCode::F(2))), &props, true);
        let closed_by_rebound = reasons.lock().unwrap().clone();
        // The scope above restores the default keymap when it drops.
        assert_eq!(old, EventResult::Ignored);
        assert!(
            closed_by_old.is_empty(),
            "Escape, no longer Cancel, does nothing"
        );
        assert_eq!(rebound, EventResult::Consumed);
        assert_eq!(
            closed_by_rebound,
            [ModalCloseReason::EscapeKey],
            "the new Cancel key closes the modal"
        );
    }
}
