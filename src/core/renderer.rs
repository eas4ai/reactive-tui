use crate::core::surface::{Surface, Rgba, DiffWriter};
use crate::core::terminal::Terminal;
use std::io::Result;

pub struct Renderer {
    term: Terminal,
    front: Surface,
    back: Surface,
    diff: DiffWriter,
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
        })
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.front = Surface::new(width, height);
        self.back = Surface::new(width, height);
    }

    pub fn clear(&mut self, color: Rgba) { self.back.clear(color); }

    pub fn begin_frame(&mut self) -> Result<()> { self.term.begin_sync() }

    pub fn end_frame(&mut self) -> Result<()> {
        self.diff.diff(&self.front, &self.back, false);
        Terminal::write_all(self.diff.output())?;
        self.front = Surface::new(self.back.dims().0, self.back.dims().1);
        // copy back into front cheaply by rebuilding front from back data next time
        self.front = self.back.clone_into_new();
        self.term.end_sync()
    }

    pub fn shutdown(mut self) -> Result<()> { self.term.exit_modern_mode() }

    pub fn surface_mut(&mut self) -> &mut Surface { &mut self.back }
    pub fn surface(&self) -> &Surface { &self.back }
}

