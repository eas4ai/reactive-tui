//! Native TTY input/output with the shared complete-frame SuprTUI renderer.

use crate::backend::{
    Backend, CellFrame, FrameLayout, ImageOutputOptions, PaintedNode, PresentedLayout,
    SuprTuiBackend,
};
use crate::component::Element;
use crate::error::{ReactiveError, Result};
use crate::event::types as rt_event;
use crate::platform::{DirectTty, TerminalCapabilities, TerminalEvent};
use crate::render::{PatchOp, RenderTree};
use std::{
    collections::VecDeque,
    io::{self, Write},
    sync::{Arc, Mutex, MutexGuard},
};

#[derive(Clone)]
struct TtyOutput(Arc<Mutex<DirectTty>>);
impl Write for TtyOutput {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0
            .lock()
            .map_err(|_| io::Error::other("direct TTY lock poisoned"))?
            .write(bytes)
            .map_err(|error| io::Error::other(error.to_string()))
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Direct native TTY transport and input, sharing the complete-frame renderer.
pub struct DirectTtyBackend {
    tty: Arc<Mutex<DirectTty>>,
    renderer: SuprTuiBackend,
    capabilities: TerminalCapabilities,
    pending: VecDeque<rt_event::Event>,
    observed_size: (u16, u16),
    active: bool,
}

impl DirectTtyBackend {
    /// Enter one native raw terminal session and start its frame renderer.
    pub fn new() -> Result<Self> {
        let tty = DirectTty::init()?;
        let capabilities = tty.capabilities().clone();
        let size = tty.size()?;
        let tty = Arc::new(Mutex::new(tty));
        let mut images = ImageOutputOptions {
            kitty_graphics: capabilities.kitty_graphics,
            sixel: capabilities.sixel_graphics,
            iterm2_inline: capabilities.iterm2_images,
            ..Default::default()
        };
        images.refresh_cell_pixels();
        let renderer =
            SuprTuiBackend::with_terminal_writer(size.0, size.1, TtyOutput(tty.clone()), images)?;
        let backend = Self {
            tty,
            renderer,
            capabilities,
            pending: VecDeque::new(),
            observed_size: size,
            active: true,
        };
        backend
            .tty()?
            .enable_mouse(backend.capabilities.pixel_mouse)?;
        TtyOutput(backend.tty.clone()).write_all(b"\x1b[?2004h")?;
        Ok(backend)
    }

    fn tty(&self) -> Result<MutexGuard<'_, DirectTty>> {
        self.tty
            .lock()
            .map_err(|_| ReactiveError::invalid_state("direct TTY lock poisoned"))
    }

    /// Write a hyperlink through the retained native transport.
    pub fn write_hyperlink(&mut self, uri: &str, text: &str) -> Result<()> {
        if !self.active {
            return Err(ReactiveError::invalid_state("direct TTY is shut down"));
        }
        self.tty()?.write_hyperlink(uri, text)
    }

    /// Return the capabilities detected for this native terminal.
    pub fn capabilities(&self) -> &TerminalCapabilities {
        &self.capabilities
    }

    /// Map terminal events to reactive-tui events
    fn map_terminal_event(event: TerminalEvent) -> Option<rt_event::Event> {
        match event {
            TerminalEvent::Key {
                code,
                modifiers,
                kind,
            } => {
                let rt_code = match code {
                    crate::platform::KeyCode::Backspace => rt_event::KeyCode::Backspace,
                    crate::platform::KeyCode::Enter => rt_event::KeyCode::Enter,
                    crate::platform::KeyCode::Left => rt_event::KeyCode::Left,
                    crate::platform::KeyCode::Right => rt_event::KeyCode::Right,
                    crate::platform::KeyCode::Up => rt_event::KeyCode::Up,
                    crate::platform::KeyCode::Down => rt_event::KeyCode::Down,
                    crate::platform::KeyCode::Home => rt_event::KeyCode::Home,
                    crate::platform::KeyCode::End => rt_event::KeyCode::End,
                    crate::platform::KeyCode::PageUp => rt_event::KeyCode::PageUp,
                    crate::platform::KeyCode::PageDown => rt_event::KeyCode::PageDown,
                    crate::platform::KeyCode::Tab => rt_event::KeyCode::Tab,
                    crate::platform::KeyCode::BackTab => rt_event::KeyCode::BackTab,
                    crate::platform::KeyCode::Delete => rt_event::KeyCode::Delete,
                    crate::platform::KeyCode::Insert => rt_event::KeyCode::Insert,
                    crate::platform::KeyCode::Escape => rt_event::KeyCode::Escape,
                    crate::platform::KeyCode::Char(c) => rt_event::KeyCode::Char(c),
                    crate::platform::KeyCode::F(n) => rt_event::KeyCode::F(n),
                    crate::platform::KeyCode::Unknown => rt_event::KeyCode::Unknown,
                    // Map all other advanced keys to Unknown for now
                    _ => rt_event::KeyCode::Unknown,
                };

                let rt_modifiers = rt_event::KeyModifiers {
                    shift: modifiers.shift,
                    ctrl: modifiers.ctrl,
                    alt: modifiers.alt,
                    meta: modifiers.meta,
                };

                let rt_kind = match kind {
                    crate::platform::KeyEventKind::Press => rt_event::KeyEventKind::Press,
                    crate::platform::KeyEventKind::Release => rt_event::KeyEventKind::Release,
                    crate::platform::KeyEventKind::Repeat => rt_event::KeyEventKind::Repeat,
                };

                Some(rt_event::Event::Key(rt_event::KeyEvent {
                    code: rt_code,
                    modifiers: rt_modifiers,
                    kind: rt_kind,
                    repeat: kind == crate::platform::KeyEventKind::Repeat,
                    timestamp: std::time::Instant::now(),
                }))
            }

            TerminalEvent::Mouse {
                kind,
                button,
                column,
                row,
                pixel_x,
                pixel_y,
                modifiers,
            } => {
                let (rt_kind, wheel_data) = match kind {
                    crate::platform::MouseEventKind::Down => (rt_event::MouseEventKind::Down, None),
                    crate::platform::MouseEventKind::Up => (rt_event::MouseEventKind::Up, None),
                    crate::platform::MouseEventKind::Drag => (rt_event::MouseEventKind::Drag, None),
                    crate::platform::MouseEventKind::Move => (rt_event::MouseEventKind::Move, None),
                    crate::platform::MouseEventKind::ScrollUp => (
                        rt_event::MouseEventKind::Wheel,
                        Some(rt_event::WheelEvent {
                            delta: rt_event::WheelDelta::Lines { x: 0.0, y: -1.0 },
                            phase: rt_event::WheelPhase::Changed,
                        }),
                    ),
                    crate::platform::MouseEventKind::ScrollDown => (
                        rt_event::MouseEventKind::Wheel,
                        Some(rt_event::WheelEvent {
                            delta: rt_event::WheelDelta::Lines { x: 0.0, y: 1.0 },
                            phase: rt_event::WheelPhase::Changed,
                        }),
                    ),
                    crate::platform::MouseEventKind::ScrollLeft => (
                        rt_event::MouseEventKind::Wheel,
                        Some(rt_event::WheelEvent {
                            delta: rt_event::WheelDelta::Lines { x: -1.0, y: 0.0 },
                            phase: rt_event::WheelPhase::Changed,
                        }),
                    ),
                    crate::platform::MouseEventKind::ScrollRight => (
                        rt_event::MouseEventKind::Wheel,
                        Some(rt_event::WheelEvent {
                            delta: rt_event::WheelDelta::Lines { x: 1.0, y: 0.0 },
                            phase: rt_event::WheelPhase::Changed,
                        }),
                    ),
                };

                let position = if let (Some(x), Some(y)) = (pixel_x, pixel_y) {
                    rt_event::Position::pixel(x.into(), y.into())
                } else {
                    rt_event::Position::cell(column, row)
                };

                let rt_modifiers = rt_event::KeyModifiers {
                    shift: modifiers.shift,
                    ctrl: modifiers.ctrl,
                    alt: modifiers.alt,
                    meta: modifiers.meta,
                };

                let button = match button {
                    Some(crate::platform::MouseButton::Left) => rt_event::MouseButton::Left,
                    Some(crate::platform::MouseButton::Middle) => rt_event::MouseButton::Middle,
                    Some(crate::platform::MouseButton::Right) => rt_event::MouseButton::Right,
                    None => match kind {
                        crate::platform::MouseEventKind::ScrollUp
                        | crate::platform::MouseEventKind::ScrollDown
                        | crate::platform::MouseEventKind::ScrollLeft
                        | crate::platform::MouseEventKind::ScrollRight => {
                            rt_event::MouseButton::Middle
                        }
                        _ => rt_event::MouseButton::Left,
                    },
                };

                Some(rt_event::Event::Mouse(rt_event::MouseEvent {
                    kind: rt_kind,
                    position,
                    button,
                    modifiers: rt_modifiers,
                    timestamp: std::time::Instant::now(),
                    wheel: wheel_data,
                }))
            }

            TerminalEvent::Resize { width, height } => Some(rt_event::Event::Resize(
                rt_event::ResizeEvent::new(width, height),
            )),

            TerminalEvent::FocusGained => Some(rt_event::Event::Focus(rt_event::FocusEvent {
                kind: rt_event::FocusEventKind::Gained,
                timestamp: std::time::Instant::now(),
            })),

            TerminalEvent::FocusLost => Some(rt_event::Event::Focus(rt_event::FocusEvent {
                kind: rt_event::FocusEventKind::Lost,
                timestamp: std::time::Instant::now(),
            })),

            TerminalEvent::Paste(content) => {
                Some(rt_event::Event::Paste(rt_event::PasteEvent::new(content)))
            }

            _ => None, // Ignore other events for now
        }
    }
}

impl Backend for DirectTtyBackend {
    fn shutdown_after_panic(&mut self, message: &str) -> Result<()> {
        let restored = self.shutdown();
        let reported = super::write_panic(&mut TtyOutput(self.tty.clone()), message);
        restored.and(reported)
    }
    fn is_interactive_terminal(&self) -> bool {
        self.active
    }
    fn painted_nodes(&self) -> Option<&[PaintedNode]> {
        self.renderer.painted_nodes()
    }
    fn component_layouts(&self) -> Option<&[PresentedLayout]> {
        self.renderer.component_layouts()
    }
    fn render_frame(&mut self, element: &Element) -> Result<bool> {
        self.renderer.render_frame(element)
    }
    fn layout_frame(&mut self, element: Arc<Element>) -> Result<Option<FrameLayout>> {
        self.renderer.layout_frame(element)
    }
    fn render_cells(&mut self, frame: Arc<CellFrame>) -> Result<()> {
        self.renderer.render_cells(frame)
    }
    fn apply_patches(&mut self, patches: &[PatchOp], tree: &RenderTree) -> Result<()> {
        if patches.is_empty() {
            return Ok(());
        }
        self.renderer.apply_patches(patches, tree)
    }
    fn clear(&mut self) -> Result<()> {
        self.renderer.clear()
    }
    fn present(&mut self) -> Result<()> {
        self.renderer.present()
    }
    fn size(&self) -> (u16, u16) {
        self.renderer.size()
    }
    fn resize(&mut self, width: usize, height: usize) {
        self.renderer.resize(width, height);
    }
    fn set_debug_overlay(&mut self, enabled: bool) {
        self.renderer.set_debug_overlay(enabled);
    }
    fn render_full(&mut self, element: &Element) -> Result<()> {
        self.renderer.render_full(element)
    }
    fn poll_event(&mut self, timeout_ms: Option<u64>) -> Result<Option<rt_event::Event>> {
        if !self.active {
            return Err(ReactiveError::invalid_state("direct TTY is shut down"));
        }
        if let Some(event) = self.pending.pop_front() {
            return Ok(Some(event));
        }
        let size = self.tty()?.size()?;
        if size.0 > 0 && size.1 > 0 && size != self.observed_size {
            self.observed_size = size;
            return Ok(Some(rt_event::Event::Resize(rt_event::ResizeEvent::new(
                size.0, size.1,
            ))));
        }
        let events = self
            .tty()?
            .poll_events(timeout_ms.map(std::time::Duration::from_millis))?;
        self.pending
            .extend(events.into_iter().filter_map(Self::map_terminal_event));
        Ok(self.pending.pop_front())
    }
    fn shutdown(&mut self) -> Result<()> {
        if !self.active {
            return Ok(());
        }
        self.active = false;
        let output = self.renderer.shutdown();
        let mouse = self.tty()?.disable_mouse();
        let paste = TtyOutput(self.tty.clone())
            .write_all(b"\x1b[?2004l")
            .map_err(Into::into);
        let raw = self.tty()?.restore();
        output.and(mouse).and(paste).and(raw)
    }
}

impl Drop for DirectTtyBackend {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}
