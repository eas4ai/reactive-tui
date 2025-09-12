//! Direct TTY backend for reactive-tui
//!
//! This backend uses direct TTY access instead of crossterm for advanced features

use crate::backend::Backend;
use crate::core::grapheme_cell::GraphemeSurface;
use crate::core::renderer::Renderer;
use crate::core::span_diff::SpanDiffWriter;
use crate::component::Element;

use crate::core::surface::Rgba;
use crate::error::Result;
use crate::event::types as rt_event;
use crate::platform::{DirectTty, TerminalCapabilities, TerminalEvent};
use crate::render::reconcile::PatchOp;
use crate::render::tree::RenderTree;

/// Direct TTY backend with advanced terminal features
pub struct DirectTtyBackend {
    /// Direct TTY interface
    tty: DirectTty,
    /// Renderer for drawing
    renderer: Renderer,
    /// Grapheme-aware surface for proper Unicode support
    grapheme_surface: GraphemeSurface,
    /// Previous frame for diffing
    prev_surface: Option<GraphemeSurface>,
    /// Terminal capabilities
    capabilities: TerminalCapabilities,
}

impl DirectTtyBackend {
    /// Create a new direct TTY backend
    pub fn new() -> Result<Self> {
        // Initialize direct TTY
        let tty = DirectTty::init()?;
        let capabilities = tty.capabilities().clone();

        // Get terminal size
        let (cols, rows) = tty.size()?;

        // Initialize renderer and surfaces
        let renderer = Renderer::new(cols as usize, rows as usize)
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        let grapheme_surface = GraphemeSurface::new(cols as usize, rows as usize);

        // Enable advanced features if supported
        let mut backend = Self {
            tty,
            renderer,
            grapheme_surface,
            prev_surface: None,
            capabilities,
        };

        backend.enable_advanced_features()?;

        Ok(backend)
    }

    /// Enable advanced terminal features
    fn enable_advanced_features(&mut self) -> Result<()> {
        // Enable mouse reporting with pixel coordinates if supported
        self.tty.enable_mouse(self.capabilities.pixel_mouse)?;

        // Enable synchronized output for flicker-free updates
        self.tty.enable_sync_output()?;

        // Enable alternate screen
        self.tty
            .write(crate::platform::sequences::ENABLE_ALT_SCREEN)?;

        // Hide cursor initially
        self.tty.write(crate::platform::sequences::HIDE_CURSOR)?;

        Ok(())
    }

    /// Disable advanced features and restore terminal
    fn disable_advanced_features(&mut self) -> Result<()> {
        // Show cursor
        self.tty.write(crate::platform::sequences::SHOW_CURSOR)?;

        // Disable alternate screen
        self.tty
            .write(crate::platform::sequences::DISABLE_ALT_SCREEN)?;

        // Disable synchronized output
        self.tty.disable_sync_output()?;

        // Disable mouse reporting
        self.tty.disable_mouse()?;

        Ok(())
    }

    /// Present using direct TTY with advanced features
    fn present_with_direct_tty(&mut self) -> Result<()> {
        // Copy from renderer surface to grapheme surface
        let (width, height) = self.renderer.dims();
        let surface = self.renderer.surface_mut();

        // Transfer content to grapheme surface
        self.grapheme_surface.clear(Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        });

        for y in 0..height {
            for x in 0..width {
                let cell = surface.get(x, y);
                use crate::core::grapheme_cell::{CellType, GraphemeCluster};

                // Convert char to grapheme cluster
                let mut buf = [0u8; 4];
                let s = cell.ch.encode_utf8(&mut buf);
                let cluster = GraphemeCluster::new(s);

                self.grapheme_surface.set_cell(
                    x,
                    y,
                    CellType::Glyph {
                        grapheme: cluster,
                        fg: cell.fg,
                        bg: cell.bg,
                        attr: cell.attr,
                    },
                );
            }
        }

        // Generate diff if we have a previous frame
        let mut diff_writer = SpanDiffWriter::new();
        if let Some(ref prev) = self.prev_surface {
            diff_writer.diff(prev, &self.grapheme_surface);
        } else {
            // First frame - render everything
            let empty = GraphemeSurface::new(width, height);
            diff_writer.diff(&empty, &self.grapheme_surface);
        }

        // Output to terminal with synchronized updates
        if self.capabilities.synchronized_output {
            self.tty.enable_sync_output()?;
        }

        // Write the diff
        self.tty.write(diff_writer.output())?;

        if self.capabilities.synchronized_output {
            self.tty.disable_sync_output()?;
        }

        // Save current surface for next frame
        self.prev_surface = Some(self.grapheme_surface.clone());

        Ok(())
    }

    /// Write a hyperlink to the terminal
    pub fn write_hyperlink(&mut self, uri: &str, text: &str) -> Result<()> {
        self.tty.write_hyperlink(uri, text)
    }

    /// Get terminal capabilities
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
                    crate::platform::MouseEventKind::ScrollUp => {
                        (rt_event::MouseEventKind::Wheel, Some(rt_event::WheelEvent {
                            delta: rt_event::WheelDelta::Lines { x: 0.0, y: -1.0 },
                            phase: rt_event::WheelPhase::Changed,
                        }))
                    },
                    crate::platform::MouseEventKind::ScrollDown => {
                        (rt_event::MouseEventKind::Wheel, Some(rt_event::WheelEvent {
                            delta: rt_event::WheelDelta::Lines { x: 0.0, y: 1.0 },
                            phase: rt_event::WheelPhase::Changed,
                        }))
                    },
                    crate::platform::MouseEventKind::ScrollLeft => {
                        (rt_event::MouseEventKind::Wheel, Some(rt_event::WheelEvent {
                            delta: rt_event::WheelDelta::Lines { x: -1.0, y: 0.0 },
                            phase: rt_event::WheelPhase::Changed,
                        }))
                    },
                    crate::platform::MouseEventKind::ScrollRight => {
                        (rt_event::MouseEventKind::Wheel, Some(rt_event::WheelEvent {
                            delta: rt_event::WheelDelta::Lines { x: 1.0, y: 0.0 },
                            phase: rt_event::WheelPhase::Changed,
                        }))
                    },
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

                // Determine button based on event kind and context
                let button = match kind {
                    crate::platform::MouseEventKind::ScrollUp
                    | crate::platform::MouseEventKind::ScrollDown
                    | crate::platform::MouseEventKind::ScrollLeft
                    | crate::platform::MouseEventKind::ScrollRight => rt_event::MouseButton::Middle,
                    _ => {
                        // For regular mouse events, we need to infer the button
                        // This is a limitation of the current platform event structure
                        // In a full implementation, the platform layer would need to be enhanced
                        // to include button information in the TerminalEvent::Mouse
                        if modifiers.ctrl {
                            rt_event::MouseButton::Right // Ctrl+click often simulates right click
                        } else if modifiers.shift {
                            rt_event::MouseButton::Middle // Shift+click often simulates middle click
                        } else {
                            rt_event::MouseButton::Left // Default to left button
                        }
                    }
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
    fn apply_patches(&mut self, patches: &[PatchOp], tree: &RenderTree) -> Result<()> {
        // For now, use the same patch application logic as crossterm backend
        if patches.is_empty() {
            return Ok(());
        }

        // For complex changes, fall back to full repaint
        let should_full_repaint = patches.len() > 10
            || patches
                .iter()
                .any(|p| matches!(p, PatchOp::Replace { .. } | PatchOp::ReorderChildren { .. }));

        if should_full_repaint {
            self.renderer.clear(Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            });
            if let Some(root) = tree.root() {
                // Use proper layout system instead of linear painting
                if let Some(element) = root.as_element() {
                    self.render_full(element)?;
                } else {
                    let surface = self.renderer.surface_mut();
                    let _end_y = crate::backend::paint_render_node_linear(surface, root, 0, 0);
                }
            }
        }

        Ok(())
    }

    fn clear(&mut self) -> Result<()> {
        self.renderer.clear(Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        });
        Ok(())
    }

    fn present(&mut self) -> Result<()> {
        self.present_with_direct_tty()
    }

    fn size(&self) -> (u16, u16) {
        self.tty.size().unwrap_or((80, 24))
    }

    fn poll_event(&mut self, timeout_ms: Option<u64>) -> Result<Option<rt_event::Event>> {
        let timeout = timeout_ms.map(std::time::Duration::from_millis);

        // Use actual direct TTY polling
        let events = self.tty.poll_events(timeout)?;

        // Return the first event, mapped to reactive-tui format
        for event in events {
            if let Some(mapped) = Self::map_terminal_event(event) {
                return Ok(Some(mapped));
            }
        }

        Ok(None)
    }

    fn render_full(&mut self, element: &Element) -> Result<()> {
        // Convert Element tree to NodeSpec and paint using Taffy-based layout
        let nodespec = crate::component::bridge::element_to_nodespec(element);
        // Clear the back buffer surface before painting to avoid stale cells
        self.renderer.clear(Rgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 });
        let (width, _height) = self.renderer.dims();
        let surface = self.renderer.surface_mut();
        let opts = crate::layout::paint_tree::PaintOptions::default();
        crate::layout::paint_tree::layout_and_paint_with(&nodespec, surface, width, &opts)
    }
}

impl Drop for DirectTtyBackend {
    fn drop(&mut self) {
        let _ = self.disable_advanced_features();
    }
}
