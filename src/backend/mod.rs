use crate::error::Result;

use crate::component::Element;
use crate::core::surface::Rgba;
use crate::event::types as rt_event;
use crate::render::reconcile::PatchOp;
use crate::render::tree::{RenderNode, RenderTree};

pub mod cell_frame;
mod debug_frame;
pub mod direct_tty;
pub(crate) mod suprtui;
pub use self::suprtui::{ImageOutputOptions, SuprTuiBackend};
pub use cell_frame::{CellFrame, FrameCell};

/// One node from the last successfully presented Element frame, in paint order.
#[derive(Clone, Debug)]
pub struct PaintedNode {
    /// Index in a preorder traversal of the resolved Element tree.
    pub element_index: usize,
    /// Visible cell bounds after ancestor and viewport clipping.
    pub bounds: crate::event::hit::Bounds,
}

/// A frame laid out but not painted, as [`Backend::layout_frame`] returns it.
#[derive(Clone, Debug, Default)]
pub struct FrameLayout {
    /// Each node's visible bounds, as [`Backend::painted_nodes`] would
    /// report them after presenting the frame.
    pub nodes: Vec<PaintedNode>,
    /// Each component's layout, as [`Backend::component_layouts`] would
    /// report it after presenting the frame.
    pub layouts: Vec<PresentedLayout>,
}

/// Original layout for a node in the presented Element tree.
#[derive(Clone, Copy, Debug)]
pub struct PresentedLayout {
    /// Preorder index, matching PaintedNode::element_index.
    pub element_index: usize,
    /// Original local size, placement transform and ancestor clipping.
    pub layout: crate::component::LayoutInfo,
}

#[derive(Default)]
pub(crate) struct PresentedGeometry {
    pub nodes: Vec<PaintedNode>,
    pub layouts: Vec<PresentedLayout>,
    pub images: Vec<crate::layout::paint_tree::suprtui::images::Plane>,
    pub cursor: Option<::suprtui::render::CursorState>,
    /// Element index plus one at each cell, row-major, from the renderer's
    /// committed hit grid; empty when no frame wrote one (PNT-002).
    pub hits: Vec<u32>,
    /// Whether the frame painted from the previous layout (PNT-004).
    pub layout_reused: bool,
    /// Layouts the painter's cache has computed so far, counted where the
    /// layout engine runs (PNT-004).
    pub layout_runs: u64,
    /// Cells the painter mapped through a node's inverse transform, for its
    /// background or its hit cells. Untransformed, unmasked nodes map cells
    /// by subtraction and add none (PNT-001).
    pub inverse_cells: u64,
}

/// Minimal, patch-driven backend abstraction
pub trait Backend: Send + Sync {
    /// Whether this backend owns an interactive host terminal session.
    /// App enables the Linux screen-reader adapter automatically for such sessions.
    fn is_interactive_terminal(&self) -> bool {
        false
    }

    /// Geometry from the last acknowledged frame. Wrappers should forward this.
    fn painted_nodes(&self) -> Option<&[PaintedNode]> {
        None
    }

    /// Original component layout from the acknowledged frame. Wrappers should
    /// forward this with painted_nodes so clipped controls retain local coordinates.
    fn component_layouts(&self) -> Option<&[PresentedLayout]> {
        None
    }
    /// The element index plus one painted at each cell of the last presented
    /// frame, row-major at `size().0` columns, when the backend keeps a hit
    /// grid; the event layer prefers it over painted bounds (PNT-002).
    fn hit_cells(&self) -> Option<&[u32]> {
        None
    }
    /// The element index plus one painted at one cell of the last presented
    /// frame, 0 where nothing was painted; `None` when the backend keeps no
    /// hit grid or the cell is outside it (PNT-002).
    fn hit_at(&self, x: u16, y: u16) -> Option<u32> {
        let cells = self.hit_cells()?;
        let width = usize::from(self.size().0);
        if usize::from(x) >= width {
            return None;
        }
        cells.get(usize::from(y) * width + usize::from(x)).copied()
    }
    /// Stage a complete application frame. Return false to use legacy patches.
    fn render_frame(&mut self, _element: &Element) -> Result<bool> {
        Ok(false)
    }
    /// Lay `element` out at the current size without painting or writing
    /// it. The App calls this after a resize, so components and anchored
    /// overlays learn their new geometry before the first frame at that
    /// size is presented. `None` means the backend cannot lay out ahead of
    /// a present; wrappers should forward it.
    fn layout_frame(&mut self, _element: std::sync::Arc<Element>) -> Result<Option<FrameLayout>> {
        Ok(None)
    }
    /// Stage a complete owned cell screen. Unsupported backends fail explicitly.
    fn render_cells(&mut self, _frame: std::sync::Arc<CellFrame>) -> Result<()> {
        Err(crate::error::ReactiveError::invalid_state(
            "this backend does not support cell frames",
        ))
    }
    /// Restore resources on a normal application exit (default: no-op).
    fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
    /// Restore an owned session after a panic and report through its output channel.
    /// Built-in terminal backends replay the message after restoration. Custom
    /// backends default to shutdown and logging; wrappers should forward this method.
    fn shutdown_after_panic(&mut self, message: &str) -> Result<()> {
        let restored = self.shutdown();
        log::error!("Application panicked: {message}");
        restored
    }
    /// Apply patch operations generated by the reconciler
    fn apply_patches(&mut self, patches: &[PatchOp], tree: &RenderTree) -> Result<()>;
    /// Clear the back buffer
    fn clear(&mut self) -> Result<()>;
    /// Present current frame
    fn present(&mut self) -> Result<()>;
    /// Wait until every frame presented so far has been written. A backend
    /// that writes inside `present` has nothing to wait for; one that
    /// writes after `present` returns (PIP-001) waits here and reports a
    /// write failure as the next present would.
    fn sync(&mut self) -> Result<()> {
        Ok(())
    }
    /// Return (cols, rows) if available
    fn size(&self) -> (u16, u16);
    /// Poll next high-level Event (converted from crossterm), with optional timeout in ms
    fn poll_event(&mut self, timeout_ms: Option<u64>) -> Result<Option<rt_event::Event>>;
    /// Wait for input or an App notification. The default adapts legacy backends
    /// with nonblocking input checks at most 10 ms apart and a wakeable sleep.
    fn poll_event_with_wake(
        &mut self,
        timeout: Option<std::time::Duration>,
        wake: &crate::app::AppWaker,
    ) -> Result<Option<rt_event::Event>> {
        let deadline = timeout.and_then(|duration| std::time::Instant::now().checked_add(duration));
        loop {
            if let Some(event) = self.poll_event(Some(0))? {
                return Ok(Some(event));
            }
            if wake.is_pending() || wake.is_closed() {
                return Ok(None);
            }
            let remaining =
                deadline.map(|end| end.saturating_duration_since(std::time::Instant::now()));
            if remaining == Some(std::time::Duration::ZERO) {
                return Ok(None);
            }
            wake.wait(Some(
                remaining
                    .unwrap_or(std::time::Duration::from_millis(10))
                    .min(std::time::Duration::from_millis(10)),
            ));
        }
    }
    /// Enable/disable debug overlay if supported (default: no-op)
    fn set_debug_overlay(&mut self, _enabled: bool) {}
    /// Handle terminal resize (default: no-op)
    fn resize(&mut self, _width: usize, _height: usize) {}
    /// Optional full render path (fallback)
    fn render_full(&mut self, _element: &Element) -> Result<()> {
        Ok(())
    }
}

pub(crate) fn write_panic(writer: &mut impl std::io::Write, message: &str) -> Result<()> {
    let visible: String = message
        .chars()
        .take(8192)
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .collect();
    writeln!(writer, "Framework panicked: {visible}\r")?;
    writer.flush()?;
    Ok(())
}

/// Simple painter that writes text nodes from a RenderNode tree linearly (temporary scaffolding)
pub fn paint_render_node_linear(
    surface: &mut crate::core::surface::Surface,
    node: &dyn RenderNode,
    x: usize,
    y: usize,
) -> usize {
    use crate::component::ElementType;
    use crate::core::surface::{Attr, Rgba};
    let mut cur_y = y;
    if let Some(el) = node.as_element() {
        if let ElementType::Text(s) = &el.element_type {
            let fg = Rgba {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            };
            let bg = Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            };
            for (i, line) in s.lines().enumerate() {
                surface.write_str(x, cur_y + i, line, fg, bg, Attr::empty());
            }
            cur_y += s.lines().count().max(1);
        }
    }
    for child in node.children() {
        cur_y = paint_render_node_linear(surface, child.as_ref(), x, cur_y);
    }
    cur_y
}

/// Crossterm terminal/input adapter over the complete SuprTUI frame renderer.
pub struct CrosstermBackend {
    inner: SuprTuiBackend,
}

impl CrosstermBackend {
    /// Enter the host terminal and create its complete-frame renderer.
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: SuprTuiBackend::new()?,
        })
    }

    /// Enable incremental render operations (the default), or disable them to
    /// repaint complete frames. Both modes retain graphemes and frame geometry.
    pub fn set_use_render_ops(&mut self, enabled: bool) {
        self.inner.set_full_redraw(!enabled);
    }

    fn map_ct_key_code(code: crossterm::event::KeyCode) -> rt_event::KeyCode {
        use crossterm::event::KeyCode as C;
        use rt_event::KeyCode as R;
        match code {
            C::Backspace => R::Backspace,
            C::Enter => R::Enter,
            C::Left => R::Left,
            C::Right => R::Right,
            C::Up => R::Up,
            C::Down => R::Down,
            C::Home => R::Home,
            C::End => R::End,
            C::PageUp => R::PageUp,
            C::PageDown => R::PageDown,
            C::Tab => R::Tab,
            C::BackTab => R::BackTab,
            C::Delete => R::Delete,
            C::Insert => R::Insert,
            C::Esc => R::Escape,
            C::Char(c) => R::Char(c),
            C::F(n) => R::F(n),
            _ => R::Unknown,
        }
    }

    fn map_ct_key_mods(m: crossterm::event::KeyModifiers) -> rt_event::KeyModifiers {
        rt_event::KeyModifiers {
            shift: m.contains(crossterm::event::KeyModifiers::SHIFT),
            ctrl: m.contains(crossterm::event::KeyModifiers::CONTROL),
            alt: m.contains(crossterm::event::KeyModifiers::ALT),
            meta: m.contains(crossterm::event::KeyModifiers::SUPER),
        }
    }

    fn map_ct_key_kind(k: crossterm::event::KeyEventKind) -> rt_event::KeyEventKind {
        use crossterm::event::KeyEventKind as C;
        use rt_event::KeyEventKind as R;
        match k {
            C::Press => R::Press,
            C::Release => R::Release,
            C::Repeat => R::Repeat,
        }
    }

    fn map_ct_mouse_button(b: crossterm::event::MouseButton) -> rt_event::MouseButton {
        use crossterm::event::MouseButton as C;
        use rt_event::MouseButton as R;
        match b {
            C::Left => R::Left,
            C::Right => R::Right,
            C::Middle => R::Middle,
        }
    }

    fn map_ct_event(e: crossterm::event::Event) -> Option<rt_event::Event> {
        use crossterm::event::Event as CE;
        match e {
            CE::Key(ke) => {
                let mut ev = rt_event::KeyEvent::new(Self::map_ct_key_code(ke.code))
                    .with_modifiers(Self::map_ct_key_mods(ke.modifiers))
                    .with_kind(Self::map_ct_key_kind(ke.kind));
                ev.repeat = ke.kind == crossterm::event::KeyEventKind::Repeat;
                Some(rt_event::Event::Key(ev))
            }
            CE::Resize(w, h) => Some(rt_event::Event::Resize(rt_event::ResizeEvent::new(w, h))),
            CE::Paste(text) => Some(rt_event::Event::Paste(rt_event::PasteEvent::new(text))),
            CE::FocusGained => Some(rt_event::Event::Focus(rt_event::FocusEvent {
                kind: rt_event::FocusEventKind::Gained,
                timestamp: std::time::Instant::now(),
            })),
            CE::FocusLost => Some(rt_event::Event::Focus(rt_event::FocusEvent {
                kind: rt_event::FocusEventKind::Lost,
                timestamp: std::time::Instant::now(),
            })),
            CE::Mouse(me) => {
                use crossterm::event::MouseEventKind as MK;
                let (x, y) = (me.column, me.row);
                let pos = rt_event::Position::cell(x, y);
                let kind = match me.kind {
                    MK::Down(_) => rt_event::MouseEventKind::Down,
                    MK::Up(_) => rt_event::MouseEventKind::Up,
                    MK::Drag(_) => rt_event::MouseEventKind::Drag,
                    MK::Moved => rt_event::MouseEventKind::Move,
                    MK::ScrollDown | MK::ScrollUp | MK::ScrollLeft | MK::ScrollRight => {
                        rt_event::MouseEventKind::Wheel
                    }
                };
                let btn = match me.kind {
                    MK::Down(b) | MK::Up(b) | MK::Drag(b) => Self::map_ct_mouse_button(b),
                    _ => rt_event::MouseButton::None,
                };
                let modifiers = Self::map_ct_key_mods(me.modifiers);
                let ev = rt_event::MouseEvent::new(kind, pos)
                    .with_button(btn)
                    .with_modifiers(modifiers);
                Some(rt_event::Event::Mouse(ev))
            }
        }
    }
}

impl Backend for CrosstermBackend {
    fn is_interactive_terminal(&self) -> bool {
        self.inner.is_interactive_terminal()
    }
    fn painted_nodes(&self) -> Option<&[PaintedNode]> {
        self.inner.painted_nodes()
    }
    fn component_layouts(&self) -> Option<&[PresentedLayout]> {
        self.inner.component_layouts()
    }
    fn render_frame(&mut self, element: &Element) -> Result<bool> {
        self.inner.render_frame(element)
    }
    fn layout_frame(&mut self, element: std::sync::Arc<Element>) -> Result<Option<FrameLayout>> {
        self.inner.layout_frame(element)
    }
    fn render_cells(&mut self, frame: std::sync::Arc<CellFrame>) -> Result<()> {
        self.inner.render_cells(frame)
    }
    fn apply_patches(&mut self, patches: &[PatchOp], tree: &RenderTree) -> Result<()> {
        if patches.is_empty() {
            return Ok(());
        }
        self.inner.apply_patches(patches, tree)
    }
    fn clear(&mut self) -> Result<()> {
        self.inner.clear()
    }
    fn present(&mut self) -> Result<()> {
        self.inner.present()
    }
    fn size(&self) -> (u16, u16) {
        self.inner.size()
    }
    fn poll_event(&mut self, timeout_ms: Option<u64>) -> Result<Option<rt_event::Event>> {
        self.inner.poll_event(timeout_ms)
    }
    fn poll_event_with_wake(
        &mut self,
        timeout: Option<std::time::Duration>,
        wake: &crate::app::AppWaker,
    ) -> Result<Option<rt_event::Event>> {
        self.inner.poll_event_with_wake(timeout, wake)
    }
    fn set_debug_overlay(&mut self, enabled: bool) {
        self.inner.set_debug_overlay(enabled);
    }
    fn resize(&mut self, width: usize, height: usize) {
        self.inner.resize(width, height);
    }
    fn render_full(&mut self, element: &Element) -> Result<()> {
        self.inner.render_full(element)
    }
    fn shutdown(&mut self) -> Result<()> {
        self.inner.shutdown()
    }
    fn shutdown_after_panic(&mut self, message: &str) -> Result<()> {
        self.inner.shutdown_after_panic(message)
    }
}

/// Debug backend for testing reactive components with virtual screen buffer
pub struct DebugBackend {
    /// Virtual screen buffer for deterministic testing
    virtual_screen: crate::core::surface::Surface,
    /// Size of the virtual screen
    size: (u16, u16),
    /// Event queue for scripted testing
    event_queue: std::collections::VecDeque<rt_event::Event>,
    /// Patch history for debugging
    patch_history: Vec<Vec<PatchOp>>,
    /// Frame counter
    frame_count: usize,
    pending_frame: Option<debug_frame::DebugFrame>,
    graphemes: Option<debug_frame::FrameText>,
    geometry: Option<PresentedGeometry>,
    /// Lays out and paints frames on a stack of its own, as the SuprTUI
    /// renderer does, so a deep tree draws whatever the app thread's stack.
    paint: debug_frame::PaintThread,
}

impl DebugBackend {
    const MAX_CELLS: usize = 262_144;
    /// Create a new debug backend with specified dimensions
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            virtual_screen: crate::core::surface::Surface::new(width as usize, height as usize),
            size: (width, height),
            event_queue: std::collections::VecDeque::new(),
            patch_history: Vec::new(),
            frame_count: 0,
            pending_frame: None,
            graphemes: None,
            geometry: None,
            paint: debug_frame::PaintThread::default(),
        }
    }

    /// Lay out and paint `element` into a cleared screen on the paint thread,
    /// for the full-repaint paths.
    fn paint_element(&mut self, element: Element) -> Result<()> {
        let (width, height) = (usize::from(self.size.0), usize::from(self.size.1));
        self.virtual_screen = self.paint.run(move || {
            let mut screen = crate::core::surface::Surface::new(width, height);
            screen.clear(Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            });
            let nodespec = crate::component::bridge::element_to_nodespec(&element);
            crate::layout::paint_tree::layout_and_paint_with(
                &nodespec,
                &mut screen,
                width,
                &crate::layout::paint_tree::PaintOptions::default(),
            )?;
            Ok::<_, crate::error::ReactiveError>(screen)
        })??;
        Ok(())
    }

    /// Add an event to the event queue for testing
    pub fn push_event(&mut self, event: rt_event::Event) {
        self.event_queue.push_back(event);
    }

    /// Get the current virtual screen content as a string for assertions
    pub fn screen_content(&self) -> String {
        let (w, h) = self.virtual_screen.dims();
        let mut content = String::new();
        for y in 0..h {
            for x in 0..w {
                if let Some(text) = &self.graphemes {
                    content.push_str(text.get(y * w + x).unwrap_or_default());
                } else {
                    content.push(self.virtual_screen.get(x, y).ch);
                }
            }
            if y < h - 1 {
                content.push('\n');
            }
        }
        content
    }

    /// The presented cell at position (x, y): its colors and attributes as
    /// the painter wrote them. Its text is [`DebugBackend::cell_text`].
    pub fn cell(&self, x: usize, y: usize) -> Option<crate::core::surface::Cell> {
        let (w, h) = self.virtual_screen.dims();
        (x < w && y < h).then(|| self.virtual_screen.get(x, y))
    }

    /// The text of the presented cell at position (x, y): a whole grapheme
    /// cluster, or an empty string for the second cell of a wide one. `None`
    /// outside the screen or before a frame painted through the SuprTUI
    /// painter has been presented.
    pub fn cell_text(&self, x: usize, y: usize) -> Option<&str> {
        let (w, h) = self.virtual_screen.dims();
        if x >= w || y >= h {
            return None;
        }
        self.graphemes.as_ref().and_then(|text| text.get(y * w + x))
    }

    /// Get a specific character at position (x, y)
    pub fn char_at(&self, x: usize, y: usize) -> Option<char> {
        let (w, h) = self.virtual_screen.dims();
        if x < w && y < h {
            Some(self.virtual_screen.get(x, y).ch)
        } else {
            None
        }
    }

    /// Find the position of a character on the screen
    pub fn find_char(&self, ch: char) -> Option<crate::core::geometry::Point> {
        let (w, h) = self.virtual_screen.dims();
        for y in 0..h {
            for x in 0..w {
                if self.virtual_screen.get(x, y).ch == ch {
                    return Some(crate::core::geometry::Point::new(x, y));
                }
            }
        }
        None
    }

    /// Get the patch history for debugging
    pub fn patch_history(&self) -> &[Vec<PatchOp>] {
        &self.patch_history
    }

    /// Get the current frame count
    pub fn frame_count(&self) -> usize {
        self.frame_count
    }

    /// Clear the virtual screen
    pub fn clear_screen(&mut self) {
        self.pending_frame = None;
        self.graphemes = None;
        self.geometry = None;
        self.virtual_screen.clear(Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        });
    }

    /// Resize the virtual screen without truncating dimensions or making an
    /// accidental unbounded allocation.
    pub fn try_resize(&mut self, width: usize, height: usize) -> Result<()> {
        let cells = width.checked_mul(height).ok_or_else(|| {
            crate::error::ReactiveError::invalid_parameter("debug backend dimensions overflow")
        })?;
        if width > u16::MAX as usize || height > u16::MAX as usize || cells > Self::MAX_CELLS {
            return Err(crate::error::ReactiveError::invalid_parameter(
                "debug backend dimensions exceed 65535 per axis or 262144 cells",
            ));
        }
        let surface = crate::core::surface::Surface::new(width, height);
        self.pending_frame = None;
        self.graphemes = None;
        self.geometry = None;
        self.virtual_screen = surface;
        self.size = (width as u16, height as u16);
        Ok(())
    }
}

impl Backend for DebugBackend {
    fn render_frame(&mut self, element: &Element) -> Result<bool> {
        // The one copy per frame, as SuprTuiBackend makes: the paint thread
        // lays out and paints it, and drops it there.
        let (element, size) = (element.clone(), self.size);
        self.pending_frame = Some(
            self.paint
                .run(move || debug_frame::paint(&element, size))??,
        );
        Ok(true)
    }
    fn layout_frame(&mut self, element: std::sync::Arc<Element>) -> Result<Option<FrameLayout>> {
        let size = (u32::from(self.size.0), u32::from(self.size.1));
        self.paint.run(move || {
            crate::layout::paint_tree::suprtui::layout_frame(
                crate::component::bridge::element_to_paintspec(&element)?,
                size,
                &mut crate::layout::paint_tree::suprtui::LayoutCache::default(),
            )
            .map(Some)
        })?
    }
    fn render_cells(&mut self, frame: std::sync::Arc<CellFrame>) -> Result<()> {
        if frame.size() != self.size {
            return Err(crate::error::ReactiveError::invalid_parameter(
                "cell frame and debug backend dimensions differ",
            ));
        }
        self.pending_frame = Some(debug_frame::cells(&frame));
        Ok(())
    }
    fn painted_nodes(&self) -> Option<&[PaintedNode]> {
        self.geometry
            .as_ref()
            .map(|geometry| geometry.nodes.as_slice())
    }
    fn component_layouts(&self) -> Option<&[PresentedLayout]> {
        self.geometry
            .as_ref()
            .map(|geometry| geometry.layouts.as_slice())
    }
    fn apply_patches(&mut self, patches: &[PatchOp], tree: &RenderTree) -> Result<()> {
        self.pending_frame = None;
        self.graphemes = None;
        self.geometry = None;
        // Store patches for debugging
        self.patch_history.push(patches.to_vec());

        // For debug backend, we always do a full repaint for simplicity and determinism
        self.virtual_screen.clear(Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        });

        if let Some(root) = tree.root() {
            // Use proper layout system for debug backend too
            if let Some(element) = tree.root_element() {
                self.paint_element(element)?;
            } else {
                // Fallback to linear painting
                let _end_y = paint_render_node_linear(&mut self.virtual_screen, root, 0, 0);
            }
        }

        #[cfg(feature = "debug_patches")]
        {
            log::debug!(
                "DebugBackend: Applied {} patches in frame {}",
                patches.len(),
                self.frame_count
            );
            for (i, patch) in patches.iter().enumerate() {
                log::debug!("  Patch {i}: {patch:?}");
            }
        }

        Ok(())
    }

    fn clear(&mut self) -> Result<()> {
        self.clear_screen();
        Ok(())
    }

    fn present(&mut self) -> Result<()> {
        if let Some(frame) = self.pending_frame.take() {
            let surface = std::mem::replace(&mut self.virtual_screen, frame.surface);
            let text = self.graphemes.replace(frame.text);
            // The canvas that reuses them lives on the paint thread.
            self.paint.post(move || debug_frame::recycle(surface, text));
            self.geometry = Some(frame.geometry);
        }
        self.frame_count += 1;

        #[cfg(feature = "debug_patches")]
        log::debug!("DebugBackend: Presented frame {}", self.frame_count);

        Ok(())
    }

    fn size(&self) -> (u16, u16) {
        self.size
    }

    fn poll_event(&mut self, _timeout_ms: Option<u64>) -> Result<Option<rt_event::Event>> {
        Ok(self.event_queue.pop_front())
    }

    fn resize(&mut self, width: usize, height: usize) {
        // The legacy trait cannot report errors. Keep the last valid screen;
        // callers that need diagnostics can use `try_resize`.
        let _ = self.try_resize(width, height);
    }

    fn render_full(&mut self, element: &Element) -> Result<()> {
        // For debug backend, convert element to render tree and paint
        self.pending_frame = None;
        self.graphemes = None;
        self.geometry = None;
        use crate::render::tree::{element_to_render_node, RenderTree};

        let mut tree = RenderTree::new();
        let root = element_to_render_node(element.clone());
        tree.set_root(root);

        self.virtual_screen.clear(Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        });

        if let Some(root) = tree.root() {
            // This should not be using paint_render_node_linear anymore
            // Use proper layout system instead
            if let Some(element) = tree.root_element() {
                self.paint_element(element)?;
            } else {
                let _end_y = paint_render_node_linear(&mut self.virtual_screen, root, 0, 0);
            }
        }

        self.frame_count += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::Event as CtEvent;

    #[test]
    fn api019_debug_backend_rejects_oversized_resize_without_truncation_or_allocation() {
        let mut backend = DebugBackend::new(20, 10);
        assert!(backend.try_resize(usize::MAX, 2).is_err());
        assert_eq!(backend.size(), (20, 10));
        assert!(backend.try_resize(65_535, 65_535).is_err());
        assert_eq!(backend.size(), (20, 10));

        Backend::resize(&mut backend, usize::MAX, usize::MAX);
        assert_eq!(backend.size(), (20, 10));
    }

    #[test]
    fn api019_debug_backend_accepts_empty_and_bounded_dimensions() {
        let mut backend = DebugBackend::new(1, 1);
        backend.try_resize(0, 0).unwrap();
        assert_eq!(backend.size(), (0, 0));
        assert_eq!(backend.screen_content(), "");
        backend.try_resize(512, 512).unwrap();
        assert_eq!(backend.size(), (512, 512));
    }

    #[test]
    fn map_paste_event() {
        let ct = CtEvent::Paste("hello".to_string());
        let mapped = CrosstermBackend::map_ct_event(ct).expect("mapped");
        match mapped {
            rt_event::Event::Paste(pe) => assert_eq!(pe.content, "hello"),
            _ => panic!("expected Paste event"),
        }
    }

    #[test]
    fn map_focus_gained_lost() {
        let gained = CrosstermBackend::map_ct_event(CtEvent::FocusGained).expect("mapped");
        match gained {
            rt_event::Event::Focus(f) => {
                assert!(matches!(f.kind, rt_event::FocusEventKind::Gained))
            }
            _ => panic!("expected Focus(Gained)"),
        }
        let lost = CrosstermBackend::map_ct_event(CtEvent::FocusLost).expect("mapped");
        match lost {
            rt_event::Event::Focus(f) => {
                assert!(matches!(f.kind, rt_event::FocusEventKind::Lost))
            }
            _ => panic!("expected Focus(Lost)"),
        }
    }

    #[test]
    fn map_resize_event() {
        let mapped = CrosstermBackend::map_ct_event(CtEvent::Resize(100, 40)).expect("mapped");
        match mapped {
            rt_event::Event::Resize(r) => {
                assert_eq!(r.width, 100);
                assert_eq!(r.height, 40);
            }
            _ => panic!("expected Resize"),
        }
    }

    #[test]
    fn map_key_event_basic() {
        use crossterm::event::KeyEventState as Kes;
        use crossterm::event::{KeyCode as Kc, KeyEvent, KeyEventKind, KeyModifiers as Km};
        let ct = CtEvent::Key(KeyEvent {
            code: Kc::Char('a'),
            modifiers: Km::CONTROL,
            kind: KeyEventKind::Press,
            state: Kes::NONE,
        });
        let mapped = CrosstermBackend::map_ct_event(ct).expect("mapped");
        match mapped {
            rt_event::Event::Key(k) => {
                assert!(matches!(k.code, rt_event::KeyCode::Char('a')));
                assert!(k.modifiers.ctrl);
                assert!(matches!(k.kind, rt_event::KeyEventKind::Press));
            }
            _ => panic!("expected Key"),
        }
    }

    #[test]
    fn map_mouse_event_basic() {
        use crossterm::event::KeyModifiers as Km;
        use crossterm::event::{MouseButton as Mb, MouseEvent, MouseEventKind as Mk};
        let ct = CtEvent::Mouse(MouseEvent {
            kind: Mk::Down(Mb::Left),
            column: 10,
            row: 5,
            modifiers: Km::empty(),
        });
        let mapped = CrosstermBackend::map_ct_event(ct).expect("mapped");
        match mapped {
            rt_event::Event::Mouse(m) => {
                assert!(matches!(m.kind, rt_event::MouseEventKind::Down));
                assert_eq!(m.position, rt_event::Position::cell(10, 5));
                assert!(matches!(m.button, rt_event::MouseButton::Left));
            }
            _ => panic!("expected Mouse"),
        }
    }
}
