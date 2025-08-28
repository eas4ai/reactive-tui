//! Terminal writer that consumes RenderOps and outputs ANSI sequences
//! 
//! This module provides the bridge between abstract rendering operations
//! and actual terminal output using ANSI escape sequences.

use super::render_ops::{RenderOp, RenderOps};
use super::surface::{Rgba, Attr};
use std::io::{self, Write, BufWriter};

/// Writer that converts RenderOps to terminal output
pub struct TerminalWriter<W: Write> {
    writer: BufWriter<W>,
    synchronized_output: bool,
}

impl<W: Write> TerminalWriter<W> {
    /// Create a new terminal writer
    pub fn new(writer: W) -> Self {
        Self {
            writer: BufWriter::with_capacity(64 * 1024, writer),
            synchronized_output: false,
        }
    }
    
    /// Execute a collection of render operations
    pub fn execute(&mut self, ops: &RenderOps) -> io::Result<()> {
        for op in ops.ops() {
            self.execute_op(op)?;
        }
        self.writer.flush()
    }
    
    /// Execute a single render operation
    fn execute_op(&mut self, op: &RenderOp) -> io::Result<()> {
        match op {
            RenderOp::MoveTo { x, y } => {
                write!(self.writer, "\x1b[{};{}H", y + 1, x + 1)?;
            }
            
            RenderOp::SetFgColor(color) => {
                let r = (color.r * 255.0) as u8;
                let g = (color.g * 255.0) as u8;
                let b = (color.b * 255.0) as u8;
                write!(self.writer, "\x1b[38;2;{};{};{}m", r, g, b)?;
            }
            
            RenderOp::SetBgColor(color) => {
                let r = (color.r * 255.0) as u8;
                let g = (color.g * 255.0) as u8;
                let b = (color.b * 255.0) as u8;
                write!(self.writer, "\x1b[48;2;{};{};{}m", r, g, b)?;
            }
            
            RenderOp::SetAttributes(attr) => {
                self.apply_attributes(*attr)?;
            }
            
            RenderOp::ResetStyle => {
                write!(self.writer, "\x1b[0m")?;
            }
            
            RenderOp::PrintRun(text) => {
                self.writer.write_all(text.as_bytes())?;
            }
            
            RenderOp::ClearArea { x, y, width, height, bg } => {
                // Save cursor position
                write!(self.writer, "\x1b7")?;
                
                // Apply background color if specified
                if let Some(color) = bg {
                    let r = (color.r * 255.0) as u8;
                    let g = (color.g * 255.0) as u8;
                    let b = (color.b * 255.0) as u8;
                    write!(self.writer, "\x1b[48;2;{};{};{}m", r, g, b)?;
                }
                
                // Clear each line in the area
                for row in 0..*height {
                    write!(self.writer, "\x1b[{};{}H", y + row + 1, x + 1)?;
                    for _ in 0..*width {
                        self.writer.write_all(b" ")?;
                    }
                }
                
                // Restore cursor position
                write!(self.writer, "\x1b8")?;
            }
            
            RenderOp::ClearScreen => {
                write!(self.writer, "\x1b[2J")?;
            }
            
            RenderOp::ClearToEndOfLine => {
                write!(self.writer, "\x1b[K")?;
            }
            
            RenderOp::SetCursorVisible(visible) => {
                if *visible {
                    write!(self.writer, "\x1b[?25h")?;
                } else {
                    write!(self.writer, "\x1b[?25l")?;
                }
            }
            
            RenderOp::SetSynchronizedOutput(enabled) => {
                if *enabled != self.synchronized_output {
                    if *enabled {
                        // Begin synchronized update
                        write!(self.writer, "\x1b[?2026h")?;
                    } else {
                        // End synchronized update
                        write!(self.writer, "\x1b[?2026l")?;
                    }
                    self.synchronized_output = *enabled;
                }
            }
            
            RenderOp::SaveCursorPosition => {
                write!(self.writer, "\x1b7")?;
            }
            
            RenderOp::RestoreCursorPosition => {
                write!(self.writer, "\x1b8")?;
            }
        }
        Ok(())
    }
    
    /// Apply text attributes
    fn apply_attributes(&mut self, attr: Attr) -> io::Result<()> {
        // Reset first to clear any existing attributes
        write!(self.writer, "\x1b[0m")?;
        
        if attr.contains(Attr::BOLD) {
            write!(self.writer, "\x1b[1m")?;
        }
        if attr.contains(Attr::ITALIC) {
            write!(self.writer, "\x1b[3m")?;
        }
        if attr.contains(Attr::UNDERLINE) {
            write!(self.writer, "\x1b[4m")?;
        }
        if attr.contains(Attr::REVERSE) {
            write!(self.writer, "\x1b[7m")?;
        }
        if attr.contains(Attr::STRIKE) {
            write!(self.writer, "\x1b[9m")?;
        }
        
        Ok(())
    }
    
    /// Flush any buffered output
    pub fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

/// Convert RenderOps to ANSI escape sequence string (for testing)
pub fn render_ops_to_ansi(ops: &RenderOps) -> Vec<u8> {
    let mut buffer = Vec::new();
    {
        let mut writer = TerminalWriter::new(&mut buffer);
        writer.execute(ops).expect("Writing to vec should not fail");
    }
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::render_ops::RenderOpsBuilder;
    
    #[test]
    fn test_move_to() {
        let mut builder = RenderOpsBuilder::new();
        builder.move_to(5, 10);
        
        let ops = builder.build();
        let output = render_ops_to_ansi(&ops);
        let output_str = String::from_utf8_lossy(&output);
        
        assert_eq!(output_str, "\x1b[11;6H");
    }
    
    #[test]
    fn test_color_output() {
        let mut builder = RenderOpsBuilder::new();
        let fg = Rgba { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
        let bg = Rgba { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
        
        builder.set_fg(fg).set_bg(bg);
        
        let ops = builder.build();
        let output = render_ops_to_ansi(&ops);
        let output_str = String::from_utf8_lossy(&output);
        
        assert!(output_str.contains("\x1b[38;2;255;0;0m"));
        assert!(output_str.contains("\x1b[48;2;0;255;0m"));
    }
    
    #[test]
    fn test_text_attributes() {
        let mut builder = RenderOpsBuilder::new();
        builder.set_attr(Attr::BOLD | Attr::ITALIC);
        
        let ops = builder.build();
        let output = render_ops_to_ansi(&ops);
        let output_str = String::from_utf8_lossy(&output);
        
        assert!(output_str.contains("\x1b[0m")); // Reset
        assert!(output_str.contains("\x1b[1m")); // Bold
        assert!(output_str.contains("\x1b[3m")); // Italic
    }
    
    #[test]
    fn test_print_run() {
        let mut builder = RenderOpsBuilder::new();
        builder.print("Hello World");
        
        let ops = builder.build();
        let output = render_ops_to_ansi(&ops);
        let output_str = String::from_utf8_lossy(&output);
        
        assert_eq!(output_str, "Hello World");
    }
    
    #[test]
    fn test_synchronized_output() {
        let mut builder = RenderOpsBuilder::new();
        builder.ops.push(RenderOp::SetSynchronizedOutput(true));
        builder.print("Test");
        builder.ops.push(RenderOp::SetSynchronizedOutput(false));
        
        let ops = builder.build();
        let output = render_ops_to_ansi(&ops);
        let output_str = String::from_utf8_lossy(&output);
        
        assert!(output_str.contains("\x1b[?2026h")); // Begin sync
        assert!(output_str.contains("Test"));
        assert!(output_str.contains("\x1b[?2026l")); // End sync
    }
}