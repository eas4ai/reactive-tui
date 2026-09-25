use crate::core::surface::Rgba;
use std::fmt;

/// Represents a change to be applied to the terminal
#[derive(Debug, Clone, PartialEq)]
pub enum Change {
    /// Clear the entire screen
    ClearScreen,

    /// Clear from cursor to end of screen
    ClearToEndOfScreen,

    /// Clear from cursor to end of line
    ClearToEndOfLine,

    /// Clear a specific region
    ClearRegion {
        /// X coordinate of the region
        x: u16,
        /// Y coordinate of the region
        y: u16,
        /// Width of the region to clear
        width: u16,
        /// Height of the region to clear
        height: u16,
    },

    /// Move cursor to position
    MoveTo {
        /// X coordinate to move to
        x: u16,
        /// Y coordinate to move to
        y: u16,
    },

    /// Write text at current cursor position
    WriteText(String),

    /// Write styled text
    WriteStyledText {
        /// Text content to write
        text: String,
        /// Optional foreground color
        fg: Option<Rgba>,
        /// Optional background color
        bg: Option<Rgba>,
        /// Whether text should be bold
        bold: bool,
        /// Whether text should be italic
        italic: bool,
        /// Whether text should be underlined
        underline: bool,
        /// Whether text should have strikethrough
        strike: bool,
        /// Whether colors should be reversed
        reverse: bool,
    },

    /// Set foreground color
    SetForegroundColor(Rgba),

    /// Set background color
    SetBackgroundColor(Rgba),

    /// Reset all attributes
    ResetAttributes,

    /// Enable text attribute
    EnableAttribute(TextAttribute),

    /// Disable text attribute
    DisableAttribute(TextAttribute),

    /// Push a scroll region
    PushScrollRegion {
        /// Top row of the scroll region
        top: u16,
        /// Bottom row of the scroll region
        bottom: u16,
    },

    /// Pop the scroll region
    PopScrollRegion,

    /// Scroll up by n lines in the current region
    ScrollUp(u16),

    /// Scroll down by n lines in the current region
    ScrollDown(u16),

    /// Save cursor position
    SaveCursor,

    /// Restore cursor position
    RestoreCursor,

    /// Hide cursor
    HideCursor,

    /// Show cursor
    ShowCursor,

    /// Begin atomic operation (buffer updates)
    BeginAtomic,

    /// End atomic operation (flush buffer)
    EndAtomic,

    /// Set terminal title
    SetTitle(String),

    /// Ring terminal bell
    Bell,
}

/// Text attributes that can be enabled/disabled
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAttribute {
    /// Bold text weight
    Bold,
    /// Italic text style
    Italic,
    /// Underlined text
    Underline,
    /// Strikethrough text
    Strike,
    /// Reversed foreground/background colors
    Reverse,
    /// Dimmed text intensity
    Dim,
    /// Blinking text (if supported by terminal)
    Blink,
}

/// A batch of changes to be applied together
#[derive(Debug, Default)]
pub struct ChangeBatch {
    changes: Vec<Change>,
    /// Whether to optimize the batch before applying
    optimize: bool,
}

impl ChangeBatch {
    /// Create a new empty batch
    pub fn new() -> Self {
        Self {
            changes: Vec::new(),
            optimize: true,
        }
    }

    /// Create a batch with a specific capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            changes: Vec::with_capacity(capacity),
            optimize: true,
        }
    }

    /// Add a change to the batch
    pub fn push(&mut self, change: Change) {
        self.changes.push(change);
    }

    /// Add multiple changes
    pub fn extend(&mut self, changes: impl IntoIterator<Item = Change>) {
        self.changes.extend(changes);
    }

    /// Clear all changes
    pub fn clear(&mut self) {
        self.changes.clear();
    }

    /// Get the number of changes
    pub fn len(&self) -> usize {
        self.changes.len()
    }

    /// Check if the batch is empty
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Set whether to optimize the batch
    pub fn set_optimize(&mut self, optimize: bool) {
        self.optimize = optimize;
    }

    /// Optimize the batch by removing redundant operations
    pub fn optimize_changes(&mut self) {
        if !self.optimize || self.changes.len() < 2 {
            return;
        }

        let mut optimized = Vec::with_capacity(self.changes.len());
        let mut i = 0;

        while i < self.changes.len() {
            let change = &self.changes[i];

            match change {
                // Consecutive MoveTo commands - keep only the last one
                Change::MoveTo { .. } => {
                    let mut last_move = change.clone();
                    while i + 1 < self.changes.len() {
                        if let Change::MoveTo { .. } = &self.changes[i + 1] {
                            last_move = self.changes[i + 1].clone();
                            i += 1;
                        } else {
                            break;
                        }
                    }
                    optimized.push(last_move);
                }

                // Multiple clears in a row - keep the most encompassing one across the whole clear run
                Change::ClearScreen
                | Change::ClearToEndOfScreen
                | Change::ClearToEndOfLine
                | Change::ClearRegion { .. } => {
                    // Scan forward across consecutive clear operations
                    let mut j = i;
                    let mut saw_clear_screen = matches!(change, Change::ClearScreen);
                    let mut saw_clear_to_end_of_screen =
                        matches!(change, Change::ClearToEndOfScreen);
                    while j + 1 < self.changes.len() {
                        match &self.changes[j + 1] {
                            Change::ClearScreen => {
                                saw_clear_screen = true;
                                j += 1;
                            }
                            Change::ClearToEndOfScreen => {
                                saw_clear_to_end_of_screen = true;
                                j += 1;
                            }
                            Change::ClearToEndOfLine | Change::ClearRegion { .. } => {
                                j += 1;
                            }
                            _ => break,
                        }
                    }
                    if saw_clear_screen {
                        optimized.push(Change::ClearScreen);
                    } else if saw_clear_to_end_of_screen {
                        optimized.push(Change::ClearToEndOfScreen);
                    } else {
                        // Keep the last clear in the run (line or region)
                        optimized.push(self.changes[j].clone());
                    }
                    i = j; // consume the entire run
                }

                // Consecutive text writes at the same position can be combined
                Change::WriteText(text) => {
                    let mut combined = text.clone();
                    while i + 1 < self.changes.len() {
                        if let Change::WriteText(next_text) = &self.changes[i + 1] {
                            combined.push_str(next_text);
                            i += 1;
                        } else {
                            break;
                        }
                    }
                    optimized.push(Change::WriteText(combined));
                }

                // Redundant attribute changes
                Change::ResetAttributes => {
                    optimized.push(change.clone());
                    // Skip following attribute changes until text is written
                    while i + 1 < self.changes.len() {
                        match &self.changes[i + 1] {
                            Change::SetForegroundColor(_)
                            | Change::SetBackgroundColor(_)
                            | Change::EnableAttribute(_)
                            | Change::DisableAttribute(_) => {
                                // Keep looking for text
                                let mut j = i + 2;
                                let mut found_text = false;
                                while j < self.changes.len() && !found_text {
                                    match &self.changes[j] {
                                        Change::WriteText(_) | Change::WriteStyledText { .. } => {
                                            found_text = true;
                                        }
                                        _ => j += 1,
                                    }
                                }
                                if found_text {
                                    break; // Keep the attribute changes
                                } else {
                                    i += 1; // Skip them
                                }
                            }
                            _ => break,
                        }
                    }
                }

                _ => {
                    optimized.push(change.clone());
                }
            }

            i += 1;
        }

        self.changes = optimized;
    }

    /// Apply the batch of changes to a writer
    pub fn apply<W: fmt::Write>(&self, writer: &mut W) -> fmt::Result {
        for change in &self.changes {
            change.apply(writer)?;
        }
        Ok(())
    }

    /// Get an iterator over the changes
    pub fn iter(&self) -> impl Iterator<Item = &Change> {
        self.changes.iter()
    }
}

impl Change {
    /// Apply this change to a writer using ANSI escape sequences
    pub fn apply<W: fmt::Write>(&self, writer: &mut W) -> fmt::Result {
        match self {
            Change::ClearScreen => write!(writer, "\x1b[2J\x1b[H"),
            Change::ClearToEndOfScreen => write!(writer, "\x1b[J"),
            Change::ClearToEndOfLine => write!(writer, "\x1b[K"),
            Change::ClearRegion {
                x,
                y,
                width,
                height,
            } => {
                // Save cursor, clear region line by line, restore cursor
                write!(writer, "\x1b7")?;
                for row in 0..*height {
                    write!(writer, "\x1b[{};{}H", y + row + 1, x + 1)?;
                    for _ in 0..*width {
                        write!(writer, " ")?;
                    }
                }
                write!(writer, "\x1b8")
            }
            Change::MoveTo { x, y } => write!(writer, "\x1b[{};{}H", y + 1, x + 1),
            Change::WriteText(text) => write!(writer, "{text}"),
            Change::WriteStyledText {
                text,
                fg,
                bg,
                bold,
                italic,
                underline,
                strike,
                reverse,
            } => {
                // Build SGR sequence
                let mut sgr = String::from("\x1b[");
                let mut params = Vec::new();

                if *bold {
                    params.push("1".to_string());
                }
                if *italic {
                    params.push("3".to_string());
                }
                if *underline {
                    params.push("4".to_string());
                }
                if *reverse {
                    params.push("7".to_string());
                }
                if *strike {
                    params.push("9".to_string());
                }

                if let Some(fg) = fg {
                    params.push(format!(
                        "38;2;{};{};{}",
                        (fg.r * 255.0) as u8,
                        (fg.g * 255.0) as u8,
                        (fg.b * 255.0) as u8
                    ));
                }

                if let Some(bg) = bg {
                    params.push(format!(
                        "48;2;{};{};{}",
                        (bg.r * 255.0) as u8,
                        (bg.g * 255.0) as u8,
                        (bg.b * 255.0) as u8
                    ));
                }

                if params.is_empty() {
                    write!(writer, "{text}")
                } else {
                    for (i, param) in params.iter().enumerate() {
                        if i > 0 {
                            sgr.push(';');
                        }
                        sgr.push_str(param);
                    }
                    sgr.push('m');
                    write!(writer, "{sgr}{text}\x1b[0m")
                }
            }
            Change::SetForegroundColor(color) => {
                write!(
                    writer,
                    "\x1b[38;2;{};{};{}m",
                    (color.r * 255.0) as u8,
                    (color.g * 255.0) as u8,
                    (color.b * 255.0) as u8
                )
            }
            Change::SetBackgroundColor(color) => {
                write!(
                    writer,
                    "\x1b[48;2;{};{};{}m",
                    (color.r * 255.0) as u8,
                    (color.g * 255.0) as u8,
                    (color.b * 255.0) as u8
                )
            }
            Change::ResetAttributes => write!(writer, "\x1b[0m"),
            Change::EnableAttribute(attr) => {
                let code = match attr {
                    TextAttribute::Bold => "1",
                    TextAttribute::Italic => "3",
                    TextAttribute::Underline => "4",
                    TextAttribute::Strike => "9",
                    TextAttribute::Reverse => "7",
                    TextAttribute::Dim => "2",
                    TextAttribute::Blink => "5",
                };
                write!(writer, "\x1b[{code}m")
            }
            Change::DisableAttribute(attr) => {
                let code = match attr {
                    TextAttribute::Bold => "22",
                    TextAttribute::Italic => "23",
                    TextAttribute::Underline => "24",
                    TextAttribute::Strike => "29",
                    TextAttribute::Reverse => "27",
                    TextAttribute::Dim => "22",
                    TextAttribute::Blink => "25",
                };
                write!(writer, "\x1b[{code}m")
            }
            Change::PushScrollRegion { top, bottom } => {
                write!(writer, "\x1b[{};{}r", top + 1, bottom + 1)
            }
            Change::PopScrollRegion => write!(writer, "\x1b[r"),
            Change::ScrollUp(n) => write!(writer, "\x1b[{n}S"),
            Change::ScrollDown(n) => write!(writer, "\x1b[{n}T"),
            Change::SaveCursor => write!(writer, "\x1b7"),
            Change::RestoreCursor => write!(writer, "\x1b8"),
            Change::HideCursor => write!(writer, "\x1b[?25l"),
            Change::ShowCursor => write!(writer, "\x1b[?25h"),
            Change::BeginAtomic => Ok(()), // No-op for now, could use synchronized updates
            Change::EndAtomic => Ok(()),   // No-op for now
            Change::SetTitle(title) => write!(writer, "\x1b]0;{title}\x07"),
            Change::Bell => write!(writer, "\x07"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_batch_optimization() {
        let mut batch = ChangeBatch::new();

        // Add redundant moves
        batch.push(Change::MoveTo { x: 0, y: 0 });
        batch.push(Change::MoveTo { x: 10, y: 10 });
        batch.push(Change::WriteText("Hello".to_string()));

        batch.optimize_changes();
        assert_eq!(batch.len(), 2); // Only last move and text

        // Test text combining
        let mut batch2 = ChangeBatch::new();
        batch2.push(Change::WriteText("Hello".to_string()));
        batch2.push(Change::WriteText(" ".to_string()));
        batch2.push(Change::WriteText("World".to_string()));

        batch2.optimize_changes();
        assert_eq!(batch2.len(), 1);
        if let Change::WriteText(text) = &batch2.changes[0] {
            assert_eq!(text, "Hello World");
        }
    }

    #[test]
    fn test_clear_optimization() {
        let mut batch = ChangeBatch::new();

        batch.push(Change::ClearToEndOfLine);
        batch.push(Change::ClearRegion {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
        });
        batch.push(Change::ClearScreen);
        batch.push(Change::ClearToEndOfScreen);

        batch.optimize_changes();
        assert_eq!(batch.len(), 1);
        assert_eq!(batch.changes[0], Change::ClearScreen);
    }

    #[test]
    fn test_change_apply() {
        let mut output = String::new();

        let change = Change::WriteText("Hello".to_string());
        change
            .apply(&mut output)
            .expect("Should be able to apply WriteText change");
        assert_eq!(output, "Hello");

        output.clear();
        let change = Change::MoveTo { x: 5, y: 10 };
        change
            .apply(&mut output)
            .expect("Should be able to apply MoveTo change");
        assert_eq!(output, "\x1b[11;6H");
    }
}
