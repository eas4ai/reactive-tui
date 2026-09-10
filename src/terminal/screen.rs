//! Virtual screen buffer for terminal emulation

use super::{
    parser::AnsiEvent, ScrollingRegion, TerminalCell, TerminalColor, TerminalCursor, TerminalModes,
};
use std::collections::VecDeque;

mod edit;
mod modes;
mod reflow;
mod text;

#[cfg(test)]
mod reflow_tests;
#[cfg(test)]
mod tests;

/// Virtual screen buffer for terminal emulation
#[derive(Debug)]
pub struct VirtualScreen {
    width: u16,
    height: u16,
    main_buffer: Vec<Vec<TerminalCell>>,
    alt_buffer: Vec<Vec<TerminalCell>>,
    scrollback: VecDeque<Vec<TerminalCell>>,
    max_scrollback: usize,
    cursor: TerminalCursor,
    saved_cursor: Option<TerminalCursor>,
    modes: TerminalModes,
    scrolling_region: ScrollingRegion,
    tab_stops: Vec<bool>,
    #[allow(dead_code)]
    tab_width: u16,
    using_alt_screen: bool,
    title: String,
    last_print: Option<(u16, u16)>,
    working_directory: Option<String>,
}

impl VirtualScreen {
    /// Create a virtual screen with specified dimensions.
    ///
    /// # Panics
    /// Panics for zero dimensions or more than 262144 cells. Use [`Self::try_new`]
    /// when dimensions come from input that may be invalid.
    pub fn new(width: u16, height: u16, max_scrollback: usize) -> Self {
        Self::try_new(width, height, max_scrollback).expect("invalid virtual screen dimensions")
    }

    /// Validate dimensions before allocating either screen buffer.
    pub fn try_new(width: u16, height: u16, max_scrollback: usize) -> super::TerminalResult<Self> {
        Self::validate_size(width, height)?;
        let mut screen = Self {
            width,
            height,
            main_buffer: Self::create_buffer(width, height),
            alt_buffer: Self::create_buffer(width, height),
            scrollback: VecDeque::new(),
            max_scrollback,
            cursor: TerminalCursor::new(),
            saved_cursor: None,
            modes: TerminalModes::default(),
            scrolling_region: ScrollingRegion::full_screen(width, height),
            tab_stops: Self::create_tab_stops(width, 8),
            tab_width: 8,
            using_alt_screen: false,
            title: String::new(),
            last_print: None,
            working_directory: None,
        };

        screen.modes.cursor_visible = true;
        screen.modes.auto_wrap = true;

        Ok(screen)
    }

    pub(crate) fn validate_size(width: u16, height: u16) -> super::TerminalResult<()> {
        if width == 0 || height == 0 || usize::from(width) * usize::from(height) > 262_144 {
            return Err(super::TerminalError::InvalidSize { width, height });
        }
        Ok(())
    }

    fn create_buffer(width: u16, height: u16) -> Vec<Vec<TerminalCell>> {
        (0..height)
            .map(|_| (0..width).map(|_| TerminalCell::new()).collect())
            .collect()
    }

    fn create_tab_stops(width: u16, tab_width: u16) -> Vec<bool> {
        (0..width).map(|i| i % tab_width == 0).collect()
    }

    fn current_buffer(&mut self) -> &mut Vec<Vec<TerminalCell>> {
        if self.using_alt_screen {
            &mut self.alt_buffer
        } else {
            &mut self.main_buffer
        }
    }

    fn current_buffer_ref(&self) -> &Vec<Vec<TerminalCell>> {
        if self.using_alt_screen {
            &self.alt_buffer
        } else {
            &self.main_buffer
        }
    }

    /// Process an ANSI event and update the screen
    pub fn process_event(&mut self, event: AnsiEvent) {
        if !matches!(event, AnsiEvent::Print(_) | AnsiEvent::PrintString(_)) {
            self.last_print = None;
        }
        match event {
            AnsiEvent::Print(ch) => self.print_char(ch),
            AnsiEvent::PrintString(s) => {
                for ch in s.chars() {
                    self.print_char(ch);
                }
            }
            AnsiEvent::Execute(byte) => self.execute_control(byte),
            AnsiEvent::Csi {
                final_byte,
                params,
                intermediates,
                private,
            } => {
                self.process_csi(final_byte, &params, &intermediates, private);
            }
            AnsiEvent::Osc { command, params } => {
                self.process_osc(&command, &params);
            }
            AnsiEvent::Bell => {}
            AnsiEvent::Backspace => self.backspace(),
            AnsiEvent::Tab => self.tab(),
            AnsiEvent::LineFeed => self.line_feed(),
            AnsiEvent::VerticalTab => self.vertical_tab(),
            AnsiEvent::FormFeed => self.form_feed(),
            AnsiEvent::CarriageReturn => self.carriage_return(),
            _ => {}
        }
    }

    fn execute_control(&mut self, byte: u8) {
        match byte {
            0x08 => self.backspace(),
            0x09 => self.tab(),
            0x0A => self.line_feed(),
            0x0B => self.vertical_tab(),
            0x0C => self.form_feed(),
            0x0D => self.carriage_return(),
            b'D' => self.line_feed(),
            b'E' => {
                self.line_feed();
                self.carriage_return();
            }
            b'M' => self.reverse_index(),
            b'H' => {
                if let Some(stop) = self.tab_stops.get_mut(usize::from(self.cursor.col)) {
                    *stop = true;
                }
            }
            b'=' => self.modes.application_keypad = true,
            b'>' => self.modes.application_keypad = false,
            b'c' => *self = Self::new(self.width, self.height, self.max_scrollback),
            _ => {}
        }
    }

    fn backspace(&mut self) {
        self.cursor.pending_wrap = false;
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
        }
    }

    fn tab(&mut self) {
        self.tabulate(1, false);
    }

    fn line_feed(&mut self) {
        self.advance_line(None);
    }

    fn advance_line(&mut self, wrap_end: Option<u16>) {
        let row = usize::from(self.cursor.row);
        if let Some(line) = self.current_buffer().get_mut(row) {
            for cell in line.iter_mut() {
                cell.wrapped = false;
            }
            if let Some(end) = wrap_end.filter(|&end| end != 0) {
                line[usize::from(end - 1)].wrapped = true;
            }
        }
        self.cursor.pending_wrap = false;
        if self.height == 0 {
            return;
        }
        if self.cursor.row == self.scrolling_region.bottom {
            self.scroll_up(1);
        } else {
            self.cursor.row = self.cursor.row.saturating_add(1).min(self.height - 1);
        }
    }

    fn vertical_tab(&mut self) {
        self.line_feed();
    }

    fn form_feed(&mut self) {
        self.line_feed();
    }

    fn carriage_return(&mut self) {
        let scrolling_region = self.scrolling_region;
        self.cursor.carriage_return(Some(&scrolling_region));
    }

    fn scroll_up(&mut self, n: u16) {
        let region = self.scrolling_region;
        self.scroll_lines(region.top, region.bottom, n, true, true);
    }

    fn process_csi(
        &mut self,
        final_byte: char,
        params: &[u16],
        intermediates: &[u8],
        private: bool,
    ) {
        if !intermediates.is_empty() {
            if !private && intermediates == b" " && final_byte == 'q' {
                use super::cursor::CursorShape;
                self.cursor.shape = match params.first().copied().unwrap_or(0) {
                    0 | 1 => CursorShape::BlinkingBlock,
                    2 => CursorShape::Block,
                    3 => CursorShape::BlinkingUnderline,
                    4 => CursorShape::Underline,
                    5 => CursorShape::BlinkingBar,
                    6 => CursorShape::Bar,
                    _ => self.cursor.shape,
                };
            }
            return;
        }
        if private {
            if matches!(final_byte, 'h' | 'l') {
                self.set_private_modes(params, final_byte == 'h');
            }
            return;
        }
        let count = params.first().copied().unwrap_or(1).max(1);
        match final_byte {
            'A' => self.cursor_up(count),
            'B' => self.cursor_down(count),
            'C' => self.cursor_right(count),
            'D' => self.cursor_left(count),
            'E' => {
                self.cursor_down(count);
                self.carriage_return();
            }
            'F' => {
                self.cursor_up(count);
                self.carriage_return();
            }
            'G' | '`' => self.set_cursor_position(count - 1, self.cursor.row),
            'd' => self.address_cursor(self.cursor.col, count - 1),
            'a' => self.cursor_right(count),
            'e' => self.cursor_down(count),
            'I' => self.tabulate(count, false),
            'Z' => self.tabulate(count, true),
            'g' => self.clear_tab_stops(params.first().copied().unwrap_or(0)),
            '@' => self.shift_characters(count, true),
            'P' => self.shift_characters(count, false),
            'X' => self.erase_characters(count),
            'L' => self.shift_lines(count, true),
            'M' => self.shift_lines(count, false),
            'S' => {
                self.cursor.pending_wrap = false;
                self.scroll_up(count);
            }
            'T' if params.len() <= 1 => {
                self.cursor.pending_wrap = false;
                let region = self.scrolling_region;
                self.scroll_lines(region.top, region.bottom, count, false, false);
            }
            'h' | 'l' => {
                for &mode in params {
                    if mode == 4 {
                        self.modes.insert_mode = final_byte == 'h';
                    }
                }
            }
            'H' | 'f' => {
                let row = params.first().copied().unwrap_or(1).saturating_sub(1);
                let col = params.get(1).copied().unwrap_or(1).saturating_sub(1);
                self.address_cursor(col, row);
            }
            'r' => self.set_scroll_margins(params),
            'J' => self.erase_display(params.first().copied().unwrap_or(0)),
            'K' => self.erase_line(params.first().copied().unwrap_or(0)),
            'm' => self.set_graphics_rendition(params),
            's' => self.save_cursor(),
            'u' => self.restore_cursor(),
            _ => {}
        }
    }

    fn process_osc(&mut self, command: &str, params: &[String]) {
        match command {
            "0" | "2" => {
                if let Some(title) = params.first() {
                    self.title = title.clone();
                }
            }
            "7" => {
                if let Some(dir) = params.first() {
                    self.working_directory = Some(dir.clone());
                }
            }
            "8" if params.len() >= 2 => {
                let id = if params[0].is_empty() {
                    None
                } else {
                    Some(params[0].clone())
                };
                let url = if params[1].is_empty() {
                    None
                } else {
                    Some(params[1].clone())
                };
                self.cursor.set_hyperlink(url, id);
            }
            _ => {}
        }
    }

    fn cursor_up(&mut self, n: u16) {
        let scrolling_region = self.scrolling_region;
        self.cursor.move_up(n, Some(&scrolling_region));
    }

    fn cursor_down(&mut self, n: u16) {
        let scrolling_region = self.scrolling_region;
        let height = self.height;
        self.cursor.move_down(n, Some(&scrolling_region), height);
    }

    fn cursor_right(&mut self, n: u16) {
        let scrolling_region = self.scrolling_region;
        let width = self.width;
        self.cursor.move_right(n, Some(&scrolling_region), width);
    }

    fn cursor_left(&mut self, n: u16) {
        let scrolling_region = self.scrolling_region;
        self.cursor.move_left(n, Some(&scrolling_region));
    }

    fn set_cursor_position(&mut self, col: u16, row: u16) {
        self.last_print = None;
        let col = col.min(self.width - 1);
        let row = row.min(self.height - 1);
        self.cursor.move_to(col, row);
    }

    fn save_cursor(&mut self) {
        self.cursor.save();
    }

    fn restore_cursor(&mut self) {
        self.cursor.restore();
        self.cursor.constrain_to_screen(self.width, self.height);
    }

    fn erase_display(&mut self, mode: u16) {
        let rows = match mode {
            0 => {
                self.erase_line(0);
                self.cursor.row.saturating_add(1)..self.height
            }
            1 => {
                self.erase_line(1);
                0..self.cursor.row.min(self.height)
            }
            2 => 0..self.height,
            3 => {
                self.scrollback.clear();
                return;
            }
            _ => return,
        };
        for row in rows {
            for col in 0..self.width {
                self.clear_glyph_at(col, row);
            }
        }
    }

    fn erase_line(&mut self, mode: u16) {
        let columns = match mode {
            0 => self.cursor.col..self.width,
            1 => 0..self.cursor.col.saturating_add(1).min(self.width),
            2 => 0..self.width,
            _ => return,
        };
        for col in columns {
            self.clear_glyph_at(col, self.cursor.row);
        }
    }

    fn set_graphics_rendition(&mut self, params: &[u16]) {
        if params.is_empty() {
            self.cursor.style.reset();
            return;
        }

        let mut params = params.iter().copied();
        while let Some(param) = params.next() {
            match param {
                0 => self.cursor.style.reset(),
                1 => self.cursor.style.bold = true,
                2 => self.cursor.style.dim = true,
                3 => self.cursor.style.italic = true,
                4 => self.cursor.style.underline = true,
                5 | 6 => self.cursor.style.blink = true,
                7 => self.cursor.style.reverse = true,
                8 => self.cursor.style.invisible = true,
                9 => self.cursor.style.strikethrough = true,
                22 => {
                    self.cursor.style.bold = false;
                    self.cursor.style.dim = false;
                }
                23 => self.cursor.style.italic = false,
                24 => self.cursor.style.underline = false,
                25 => self.cursor.style.blink = false,
                27 => self.cursor.style.reverse = false,
                28 => self.cursor.style.invisible = false,
                29 => self.cursor.style.strikethrough = false,
                30..=37 => {
                    self.cursor.style.foreground = TerminalColor::Indexed((param - 30) as u8);
                }
                39 => self.cursor.style.foreground = TerminalColor::Default,
                40..=47 => {
                    self.cursor.style.background = TerminalColor::Indexed((param - 40) as u8);
                }
                49 => self.cursor.style.background = TerminalColor::Default,
                90..=97 => {
                    self.cursor.style.foreground = TerminalColor::Indexed((param - 90 + 8) as u8);
                }
                100..=107 => {
                    self.cursor.style.background = TerminalColor::Indexed((param - 100 + 8) as u8);
                }
                38 | 48 => {
                    // Consume color operands even when invalid; they are not style codes.
                    let color = match params.next() {
                        Some(5) => params
                            .next()
                            .and_then(|index| u8::try_from(index).ok().map(TerminalColor::Indexed)),
                        Some(2) => {
                            let components = (params.next(), params.next(), params.next());
                            match components {
                                (Some(r), Some(g), Some(b)) if r <= 255 && g <= 255 && b <= 255 => {
                                    Some(TerminalColor::Rgb(r as u8, g as u8, b as u8))
                                }
                                _ => None,
                            }
                        }
                        _ => None,
                    };
                    if let Some(color) = color {
                        if param == 38 {
                            self.cursor.style.foreground = color;
                        } else {
                            self.cursor.style.background = color;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Get the screen dimensions (width, height)
    pub fn size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    /// Reflow main-screen soft wraps and preserve hard line breaks and scrollback.
    /// The alternate screen remains a fixed grid; clipped wide glyphs are cleared.
    pub fn resize(&mut self, width: u16, height: u16) -> super::TerminalResult<()> {
        Self::validate_size(width, height)?;
        self.reflow_main(width, height);
        reflow::resize_fixed_grid(&mut self.alt_buffer, width, height);
        if self.using_alt_screen {
            reflow::resize_fixed_cursor(&mut self.cursor, width, height);
        }
        self.last_print = None;
        self.width = width;
        self.height = height;
        self.cursor.col = self.cursor.col.min(width - 1);
        self.cursor.row = self.cursor.row.min(height - 1);
        self.scrolling_region = ScrollingRegion::full_screen(width, height);
        // Resize preserves the child's tab configuration. New columns have no
        // stops until the child sets them; cleared defaults must not reappear.
        self.tab_stops.resize(usize::from(width), false);
        Ok(())
    }

    /// Retained history rows available above the visible screen.
    pub fn scrollback_len(&self) -> usize {
        if self.using_alt_screen {
            0
        } else {
            self.scrollback.len()
        }
    }

    /// A cell viewed `offset` rows above the live screen. Offset is clamped.
    pub fn scrolled_cell_at(&self, col: u16, row: u16, offset: usize) -> Option<&TerminalCell> {
        if col >= self.width || row >= self.height {
            return None;
        }
        let history = self.scrollback_len();
        let index = history - offset.min(history) + usize::from(row);
        if index < history {
            self.scrollback.get(index)?.get(usize::from(col))
        } else {
            self.cell_at(col, (index - history) as u16)
        }
    }

    /// Get the current cursor position (column, row)
    pub fn cursor_position(&self) -> (u16, u16) {
        (self.cursor.col, self.cursor.row)
    }

    /// Get the screen title
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Get the current working directory
    pub fn working_directory(&self) -> Option<&str> {
        self.working_directory.as_deref()
    }

    /// Get the cell at the specified position
    pub fn cell_at(&self, col: u16, row: u16) -> Option<&TerminalCell> {
        let buffer = self.current_buffer_ref();
        buffer.get(row as usize)?.get(col as usize)
    }

    /// Check if the cursor is visible
    pub fn cursor_visible(&self) -> bool {
        self.modes.cursor_visible && self.cursor.visible
    }

    /// Get the current cursor shape
    pub fn cursor_shape(&self) -> super::cursor::CursorShape {
        self.cursor.shape
    }

    pub(crate) fn input_modes(&self) -> &TerminalModes {
        &self.modes
    }
}
