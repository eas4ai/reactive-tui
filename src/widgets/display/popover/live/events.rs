use super::*;
use crate::event::types::{FocusEventKind, KeyEventKind};

impl Popover {
    pub(in super::super) fn escape(&self, event: &Event, props: &PopoverProps) -> EventResult {
        if self.is_visible() {
            let Event::Key(key) = event else {
                return EventResult::Ignored;
            };
            if key.code != KeyCode::Escape || key.kind == KeyEventKind::Release {
                return EventResult::Ignored;
            }
            if props.close_on_escape && key.kind == KeyEventKind::Press && !key.repeat {
                self.hide();
            }
            return EventResult::Consumed;
        }
        EventResult::Ignored
    }

    pub(super) fn observe(
        &self,
        event: &Event,
        props: &PopoverProps,
        trigger: bool,
    ) -> EventResult {
        if trigger && props.trigger == PopoverTrigger::Click && activation(event) {
            if !self.is_visible() {
                self.show();
            } else if props.close_on_trigger_click {
                self.hide();
            }
            return EventResult::Handled;
        }
        if props.trigger == PopoverTrigger::Hover {
            if let Event::Mouse(mouse) = event {
                let hovered = match mouse.kind {
                    MouseEventKind::Enter | MouseEventKind::Move => Some(true),
                    MouseEventKind::Leave => Some(false),
                    _ => None,
                };
                if let Some(hovered) = hovered {
                    let mut data = self.live.data.lock().unwrap();
                    let mut state = self.state.lock().unwrap();
                    // The router preserves shared ancestors when moving between
                    // descendants. A leave with no pointer must also work at (0, 0).
                    let inside = hovered;
                    if inside != state.is_hovered {
                        state.is_hovered = inside;
                        let delay = if inside {
                            props.hover_delay
                        } else {
                            props.hover_leave_delay
                        };
                        data.hover = Instant::now().checked_add(delay).map(|at| (inside, at));
                    }
                    state.last_mouse_pos = Some(mouse.position);
                    drop(state);
                    drop(data);
                    // Enter/Leave notifications can be emitted while the router
                    // refreshes its path, without a handled target event.
                    self.live
                        .changed
                        .update(|revision| *revision = revision.wrapping_add(1));
                    return EventResult::Handled;
                }
            }
        }
        if props.trigger == PopoverTrigger::Focus {
            if let Event::Focus(focus) = event {
                match focus.kind {
                    FocusEventKind::Gained => {
                        let dismissed = self.live.data.lock().unwrap().focus_dismissed;
                        if !dismissed {
                            self.show();
                        }
                    }
                    FocusEventKind::Lost => {
                        self.hide();
                        if trigger {
                            self.live.data.lock().unwrap().focus_dismissed = false;
                        }
                    }
                    _ => return EventResult::Ignored,
                }
                self.state.lock().unwrap().is_focused = focus.kind == FocusEventKind::Gained;
                return EventResult::Handled;
            }
        }
        EventResult::Ignored
    }
}

pub(super) fn activation(event: &Event) -> bool {
    match event {
        Event::Mouse(mouse) => {
            mouse.button == MouseButton::Left
                && matches!(mouse.kind, MouseEventKind::Down | MouseEventKind::Click)
        }
        Event::Key(key) => {
            key.kind == KeyEventKind::Press
                && !key.repeat
                && key.modifiers.is_empty()
                && matches!(
                    key.code,
                    KeyCode::Enter | KeyCode::Space | KeyCode::Char(' ')
                )
        }
        _ => false,
    }
}
