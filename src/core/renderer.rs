use crate::core::surface::{DiffWriter, Rgba, Surface};
use crate::core::terminal::Terminal;
use std::io::Result;
use std::time::Instant;

#[derive(Debug, Default, Clone, Copy)]
pub struct FrameStats {
    pub bytes_written: usize,
    pub spans_written: usize,
    pub rows_changed: usize,
    pub frame_ms: f32,
}

pub struct Renderer {
    term: Terminal,
    front: Surface,
    back: Surface,
    diff: DiffWriter,
    last_stats: FrameStats,
    frame_start: Option<Instant>,
    debug_overlay: bool,
}

impl Renderer {
    pub fn new(width: usize, height: usize) -> Result<Self> {
        let mut term = Terminal::new()?;
        term.capability_gate()?;
        term.enter_modern_mode()?;
        Ok(Self {
            term,
            front: Surface::new(width, height),
            back: Surface::new(width, height),
            diff: DiffWriter::new(),
            last_stats: FrameStats::default(),
            frame_start: None,
            debug_overlay: false,
        })
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.front.reinit(width, height);
        self.back.reinit(width, height);
    }

    pub fn clear(&mut self, color: Rgba) {
        self.back.clear(color);
    }

    pub fn set_debug_overlay(&mut self, enabled: bool) {
        self.debug_overlay = enabled;
    }

    pub fn begin_frame(&mut self) -> Result<()> {
        self.frame_start = Some(Instant::now());
        self.term.begin_sync()
    }

    pub fn end_frame(&mut self) -> Result<()> {
        // Diff and write
        self.diff.diff(&self.front, &self.back, false);
        let out = self.diff.output();
        Terminal::write_all(out)?;
        // Stats
        self.last_stats.bytes_written = out.len();
        self.last_stats.spans_written = self.diff.last_spans_written();
        self.last_stats.rows_changed = self.diff.last_rows_changed();
        if let Some(start) = self.frame_start.take() {
            self.last_stats.frame_ms = start.elapsed().as_secs_f32() * 1000.0;
        }
        // Optional debug overlay
        if self.debug_overlay {
            let (_w, h) = self.back.dims();
            let overlay = format!(
                "\x1b[{row};1H\x1b[7mframe: {ms:.2}ms | bytes: {b} | spans: {s} | rows: {r}\x1b[0m",
                row = h,
                ms = self.last_stats.frame_ms,
                b = self.last_stats.bytes_written,
                s = self.last_stats.spans_written,
                r = self.last_stats.rows_changed
            );
            Terminal::write_all(overlay.as_bytes())?;
        }
        // Make front reflect the just-rendered back buffer for next diff
        let (fw, fh) = self.front.dims();
        let (bw, bh) = self.back.dims();
        if (fw, fh) == (bw, bh) {
            self.front.copy_from(&self.back);
        } else {
            self.front = self.back.clone_into_new();
        }
        self.term.end_sync()
    }

    pub fn shutdown(mut self) -> Result<()> {
        self.term.exit_modern_mode()
    }

    pub fn frame_stats(&self) -> FrameStats { self.last_stats }

    pub fn surface_mut(&mut self) -> &mut Surface {
        &mut self.back
    }
    pub fn surface(&self) -> &Surface {
        &self.back
    }

    pub fn dims(&self) -> (usize, usize) {
        self.back.dims()
    }
}
