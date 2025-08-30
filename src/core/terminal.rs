use crossterm::event::Event;
use crossterm::{event, terminal};
use std::io::{BufWriter, Result};
use std::process::Command;
use std::time::Duration;

#[cfg(not(test))]
use std::io::Write;
#[cfg(not(test))]
use std::time::Instant;

use crate::widgets::display::image::ImageCapabilities;

#[cfg(not(test))]
use crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
};
#[cfg(not(test))]
use crossterm::execute;
#[cfg(not(test))]
use std::env;
#[cfg(not(test))]
use std::io::{Error, stdout};

/// Statistics for terminal write operations
#[derive(Debug, Default, Clone)]
pub struct TerminalWriteStats {
    pub bytes_written: u64,
    pub flush_count: u32,
    pub write_time: Duration,
    pub buffer_size: usize,
    pub buffer_utilization: f32,
}

pub struct Terminal {
    #[allow(dead_code)] // Used conditionally based on platform
    writer: Option<BufWriter<std::io::Stdout>>,
    stats: TerminalWriteStats,
    buffer_size: usize,
}

impl Drop for Terminal {
    fn drop(&mut self) {
        // Always restore terminal state, even on panic
        let _ = self.exit_modern_mode();
    }
}

impl Terminal {
    /// Default buffer size - 2MB for optimal performance
    pub const DEFAULT_BUFFER_SIZE: usize = 2 * 1024 * 1024;

    pub fn new() -> Result<Self> {
        Self::with_buffer_size(Self::DEFAULT_BUFFER_SIZE)
    }

    pub fn with_buffer_size(buffer_size: usize) -> Result<Self> {
        let stats = TerminalWriteStats {
            buffer_size,
            ..Default::default()
        };

        Ok(Self {
            writer: None,
            stats,
            buffer_size,
        })
    }

    #[cfg(not(test))]
    pub fn enter_modern_mode(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        execute!(stdout(), terminal::EnterAlternateScreen)?;
        execute!(stdout(), EnableBracketedPaste)?;
        execute!(stdout(), EnableMouseCapture)?;
        Ok(())
    }
    #[cfg(test)]
    pub fn enter_modern_mode(&mut self) -> Result<()> {
        Ok(())
    }

    #[cfg(not(test))]
    pub fn exit_modern_mode(&mut self) -> Result<()> {
        execute!(stdout(), DisableMouseCapture)?;
        execute!(stdout(), DisableBracketedPaste)?;
        execute!(stdout(), terminal::LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }
    #[cfg(test)]
    pub fn exit_modern_mode(&mut self) -> Result<()> {
        Ok(())
    }

    #[cfg(not(test))]
    pub fn begin_sync(&mut self) -> Result<()> {
        execute!(stdout(), terminal::BeginSynchronizedUpdate)?;
        Ok(())
    }
    #[cfg(test)]
    pub fn begin_sync(&mut self) -> Result<()> {
        Ok(())
    }

    #[cfg(not(test))]
    pub fn end_sync(&mut self) -> Result<()> {
        execute!(stdout(), terminal::EndSynchronizedUpdate)?;
        Ok(())
    }
    #[cfg(test)]
    pub fn end_sync(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn size(&self) -> Result<(u16, u16)> {
        let (cols, rows) = terminal::size()?;
        Ok((cols, rows))
    }

    /// Gate startup on modern terminal assumptions.
    /// We accept terminals that advertise truecolor via COLORTERM or well-known TERM values.
    #[cfg(not(test))]
    pub fn capability_gate(&mut self) -> Result<()> {
        let colorterm = env::var("COLORTERM")
            .unwrap_or_default()
            .to_ascii_lowercase();
        let term = env::var("TERM").unwrap_or_default().to_ascii_lowercase();
        let modern_term = term.contains("wezterm")
            || term.contains("kitty")
            || term.contains("alacritty")
            || term.contains("iterm");
        let truecolor = colorterm.contains("truecolor")
            || colorterm.contains("24bit")
            || term.contains("direct")
            || term.contains("24bit");
        if !(modern_term || truecolor) {
            return Err(Error::other(
                "Requires a modern terminal with 24-bit color (wezterm, kitty, alacritty, iTerm2)",
            ));
        }
        Ok(())
    }
    #[cfg(test)]
    pub fn capability_gate(&mut self) -> Result<()> {
        Ok(())
    }

    #[cfg(test)]
    pub fn enable_buffered_mode(&mut self) -> Result<()> {
        Ok(())
    }

    #[cfg(test)]
    pub fn disable_buffered_mode(&mut self) -> Result<()> {
        Ok(())
    }

    #[cfg(test)]
    pub fn write_all_buffered(&mut self, buf: &[u8]) -> Result<()> {
        self.stats.bytes_written += buf.len() as u64;
        Ok(())
    }

    #[cfg(test)]
    pub fn flush_buffered(&mut self) -> Result<()> {
        self.stats.flush_count += 1;
        Ok(())
    }

    pub fn poll_event(timeout_ms: Option<u64>) -> Result<Option<Event>> {
        let dur = std::time::Duration::from_millis(timeout_ms.unwrap_or(0));
        if !event::poll(dur)? {
            return Ok(None);
        }
        Ok(Some(event::read()?))
    }

    /// Enable buffered writing mode for better performance
    #[cfg(not(test))]
    pub fn enable_buffered_mode(&mut self) -> Result<()> {
        if self.writer.is_none() {
            self.writer = Some(BufWriter::with_capacity(
                self.buffer_size,
                std::io::stdout(),
            ));
        }
        Ok(())
    }

    /// Disable buffered mode and flush any remaining data
    #[cfg(not(test))]
    pub fn disable_buffered_mode(&mut self) -> Result<()> {
        if let Some(mut writer) = self.writer.take() {
            writer.flush()?;
            self.stats.flush_count += 1;
        }
        Ok(())
    }

    /// Get current write statistics
    pub fn write_stats(&self) -> &TerminalWriteStats {
        &self.stats
    }

    /// Reset write statistics
    pub fn reset_write_stats(&mut self) {
        self.stats = TerminalWriteStats {
            buffer_size: self.buffer_size,
            ..TerminalWriteStats::default()
        };
    }

    /// Write with buffering if enabled, otherwise direct write
    #[cfg(not(test))]
    pub fn write_all_buffered(&mut self, buf: &[u8]) -> Result<()> {
        let start = Instant::now();

        if let Some(ref mut writer) = self.writer {
            // Buffered mode
            writer.write_all(buf)?;
            self.stats.bytes_written += buf.len() as u64;

            // Update buffer utilization
            let used = writer.buffer().len();
            self.stats.buffer_utilization = used as f32 / self.buffer_size as f32;

            // Auto-flush if buffer is >75% full
            if self.stats.buffer_utilization > 0.75 {
                writer.flush()?;
                self.stats.flush_count += 1;
            }
        } else {
            // Direct mode (backward compatibility)
            std::io::stdout().write_all(buf)?;
            self.stats.bytes_written += buf.len() as u64;
        }

        self.stats.write_time += start.elapsed();
        Ok(())
    }

    /// Flush buffered output
    #[cfg(not(test))]
    pub fn flush_buffered(&mut self) -> Result<()> {
        if let Some(ref mut writer) = self.writer {
            writer.flush()?;
            self.stats.flush_count += 1;
        }
        Ok(())
    }

    #[cfg(not(test))]
    pub fn write_all(buf: &[u8]) -> Result<()> {
        std::io::stdout().write_all(buf)?;
        Ok(())
    }

    /// Detect available image rendering capabilities
    pub fn detect_image_capabilities() -> ImageCapabilities {
        ImageCapabilities {
            sixel: Self::has_sixel_support(),
            kitty_graphics: Self::has_kitty_graphics(),
            iterm2_inline: Self::has_iterm2_support(),
            chafa_available: Self::has_external_tool("chafa"),
            viu_available: Self::has_external_tool("viu"),
        }
    }

    /// Check if terminal supports sixel graphics
    fn has_sixel_support() -> bool {
        let term = std::env::var("TERM").unwrap_or_default().to_lowercase();
        let term_program = std::env::var("TERM_PROGRAM")
            .unwrap_or_default()
            .to_lowercase();

        // Known terminals with sixel support
        term.contains("xterm")
            || term.contains("wezterm")
            || term.contains("mlterm")
            || term.contains("foot")
            || term.contains("contour")
            || term_program.contains("wezterm")
    }

    /// Check if terminal supports Kitty graphics protocol
    fn has_kitty_graphics() -> bool {
        let term = std::env::var("TERM").unwrap_or_default().to_lowercase();
        let term_program = std::env::var("TERM_PROGRAM")
            .unwrap_or_default()
            .to_lowercase();

        term.contains("kitty") || term_program.contains("kitty")
    }

    /// Check if terminal supports iTerm2 inline images
    fn has_iterm2_support() -> bool {
        let term_program = std::env::var("TERM_PROGRAM")
            .unwrap_or_default()
            .to_lowercase();
        term_program.contains("iterm")
    }

    /// Check if external image rendering tool is available
    fn has_external_tool(tool: &str) -> bool {
        Command::new(tool)
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Get terminal size in characters
    pub fn get_size() -> Result<(u16, u16)> {
        terminal::size()
    }
}

#[cfg(test)]
pub mod test_io_capture {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    static TEST_OUT: OnceLock<Mutex<Vec<u8>>> = OnceLock::new();
    fn out() -> &'static Mutex<Vec<u8>> {
        TEST_OUT.get_or_init(|| Mutex::new(Vec::new()))
    }

    impl Terminal {
        pub fn write_all(buf: &[u8]) -> Result<()> {
            out().lock().unwrap().extend_from_slice(buf);
            Ok(())
        }
    }

    pub fn take_output() -> Vec<u8> {
        let mut guard = out().lock().unwrap();
        std::mem::take(&mut *guard)
    }
}
