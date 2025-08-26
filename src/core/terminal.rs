use crossterm::{event, execute, terminal};
use crossterm::event::{Event, EnableBracketedPaste, DisableBracketedPaste, EnableMouseCapture, DisableMouseCapture};
use std::env;
use std::io::{stdout, Write};
use std::io::{Error, Result};

pub struct Terminal {}

impl Terminal {
    pub fn new() -> Result<Self> { Ok(Self {}) }

    pub fn enter_modern_mode(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        execute!(stdout(), terminal::EnterAlternateScreen)?;
        execute!(stdout(), EnableBracketedPaste)?;
        execute!(stdout(), EnableMouseCapture)?;
        Ok(())
    }

    pub fn exit_modern_mode(&mut self) -> Result<()> {
        execute!(stdout(), DisableMouseCapture)?;
        execute!(stdout(), DisableBracketedPaste)?;
        execute!(stdout(), terminal::LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    pub fn begin_sync(&mut self) -> Result<()> {
        execute!(stdout(), terminal::BeginSynchronizedUpdate)?;
        Ok(())
    }

    pub fn end_sync(&mut self) -> Result<()> {
        execute!(stdout(), terminal::EndSynchronizedUpdate)?;
        Ok(())
    }

    pub fn size(&self) -> Result<(u16, u16)> {
        let (cols, rows) = terminal::size()?;
        Ok((cols, rows))
    }

    /// Gate startup on modern terminal assumptions.
    /// We accept terminals that advertise truecolor via COLORTERM or well-known TERM values.
    pub fn capability_gate(&mut self) -> Result<()> {
        let colorterm = env::var("COLORTERM").unwrap_or_default().to_ascii_lowercase();
        let term = env::var("TERM").unwrap_or_default().to_ascii_lowercase();
        let modern_term = term.contains("wezterm") || term.contains("kitty") || term.contains("alacritty") || term.contains("iterm");
        let truecolor = colorterm.contains("truecolor") || colorterm.contains("24bit") || term.contains("direct") || term.contains("24bit");
        if !(modern_term || truecolor) {
            return Err(Error::other(
                "Requires a modern terminal with 24-bit color (wezterm, kitty, alacritty, iTerm2)",
            ));
        }
        Ok(())
    }

    pub fn poll_event(timeout_ms: Option<u64>) -> Result<Option<Event>> {
        if let Some(ms) = timeout_ms {
            if !event::poll(std::time::Duration::from_millis(ms))? { return Ok(None); }
        }
        if event::poll(std::time::Duration::from_millis(0))? {
            Ok(Some(event::read()?))
        } else { Ok(None) }
    }

    pub fn write_all(buf: &[u8]) -> Result<()> {
        stdout().write_all(buf)?;
        Ok(())
    }
}

