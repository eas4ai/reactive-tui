//! Virtual screen buffer for terminal emulation

use super::{
    parser::AnsiEvent, ScrollingRegion, TerminalCell, TerminalColor, TerminalCursor, TerminalModes,
};
use std::collections::VecDeque;

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
    tab_width: u16,
    using_alt_screen: bool,
    title: String,
    working_directory: Option<String>,
}

impl VirtualScreen {
    pub fn new(width: u16, height: u16, max_scrollback: usize) -> Self {
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
            working_directory: None,
        };

        screen.modes.cursor_visible = true;
        screen.modes.auto_wrap = true;

        screen
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

    pub fn process_event(&mut self, event: AnsiEvent) {
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

    fn print_char(&mut self, ch: char) {
        if self.cursor.pending_wrap {
            let scrolling_region = self.scrolling_region;
            let height = self.height;
            self.cursor.handle_wrap(Some(&scrolling_region), height);
            if self.cursor.row >= height {
                self.scroll_up(1);
                self.cursor.row = height - 1;
            }
        }

        if self.cursor.row >= self.height {
            return;
        }

        let row = self.cursor.row as usize;
        let col = self.cursor.col as usize;
        let cursor_style = self.cursor.style;
        let hyperlink_url = self.cursor.hyperlink_url.clone();
        let hyperlink_id = self.cursor.hyperlink_id.clone();
        let width = self.width;
        let auto_wrap = self.modes.auto_wrap;

        let buffer = self.current_buffer();
        if row < buffer.len() && col < buffer[row].len() {
            let cell = &mut buffer[row][col];
            cell.set_char(ch);
            cell.set_style(cursor_style);

            if let Some(url) = hyperlink_url {
                cell.set_hyperlink(url, hyperlink_id);
            }

            let char_width = cell.width;

            if char_width > 1 {
                for i in 1..char_width {
                    let next_col = col + i as usize;
                    if next_col < buffer[row].len() {
                        let continuation_cell = &mut buffer[row][next_col];
                        continuation_cell.character = String::new();
                        continuation_cell.width = 0;
                        continuation_cell.set_style(cursor_style);
                        continuation_cell.mark_dirty();
                    }
                }
            }

            drop(buffer);
            self.cursor.advance(char_width, width, auto_wrap);
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
            _ => {}
        }
    }

    fn backspace(&mut self) {
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
        }
    }

    fn tab(&mut self) {
        let mut next_tab = self.cursor.col + 1;
        while next_tab < self.width && next_tab < self.tab_stops.len() as u16 {
            if self.tab_stops[next_tab as usize] {
                break;
            }
            next_tab += 1;
        }
        self.cursor.col = next_tab.min(self.width - 1);
    }

    fn line_feed(&mut self) {
        let scrolling_region = self.scrolling_region;
        let height = self.height;
        self.cursor.line_feed(Some(&scrolling_region), height);
        if self.cursor.row >= height {
            self.scroll_up(1);
            self.cursor.row = height - 1;
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
        let using_alt_screen = self.using_alt_screen;
        let max_scrollback = self.max_scrollback;

        for _ in 0..n {
            if !using_alt_screen && region.top == 0 {
                let line = {
                    let buffer = self.current_buffer();
                    buffer[region.top as usize].clone()
                };
                self.scrollback.push_back(line);

                while self.scrollback.len() > max_scrollback {
                    self.scrollback.pop_front();
                }
            }

            let buffer = self.current_buffer();
            for row in region.top..region.bottom {
                let src_row = (row + 1) as usize;
                let dst_row = row as usize;
                if src_row < buffer.len() && dst_row < buffer.len() {
                    let src_line = buffer[src_row].clone();
                    buffer[dst_row] = src_line;
                }
            }

            let bottom_row = region.bottom as usize;
            if bottom_row < buffer.len() {
                for col in region.left..=region.right {
                    if (col as usize) < buffer[bottom_row].len() {
                        buffer[bottom_row][col as usize].clear(TerminalColor::Default);
                    }
                }
            }
        }
    }

    fn process_csi(
        &mut self,
        final_byte: char,
        params: &[u16],
        _intermediates: &[u8],
        _private: bool,
    ) {
        match final_byte {
            'A' => self.cursor_up(params.get(0).copied().unwrap_or(1)),
            'B' => self.cursor_down(params.get(0).copied().unwrap_or(1)),
            'C' => self.cursor_right(params.get(0).copied().unwrap_or(1)),
            'D' => self.cursor_left(params.get(0).copied().unwrap_or(1)),
            'H' | 'f' => {
                let row = params.get(0).copied().unwrap_or(1).saturating_sub(1);
                let col = params.get(1).copied().unwrap_or(1).saturating_sub(1);
                self.set_cursor_position(col, row);
            }
            'J' => self.erase_display(params.get(0).copied().unwrap_or(0)),
            'K' => self.erase_line(params.get(0).copied().unwrap_or(0)),
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
            "8" => {
                if params.len() >= 2 {
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
        let col = col.min(self.width - 1);
        let row = row.min(self.height - 1);
        self.cursor.move_to(col, row);
    }

    fn save_cursor(&mut self) {
        self.cursor.save();
    }

    fn restore_cursor(&mut self) {
        self.cursor.restore();
    }

    fn erase_display(&mut self, mode: u16) {
        match mode {
            0 => {
                self.erase_line(0);
                let cursor_row = self.cursor.row;
                let height = self.height;
                let width = self.width;
                let buffer = self.current_buffer();
                for row in (cursor_row + 1)..height {
                    for col in 0..width {
                        if let Some(cell) = buffer
                            .get_mut(row as usize)
                            .and_then(|r| r.get_mut(col as usize))
                        {
                            cell.clear(TerminalColor::Default);
                        }
                    }
                }
            }
            2 => {
                let height = self.height;
                let width = self.width;
                let buffer = self.current_buffer();
                for row in 0..height {
                    for col in 0..width {
                        if let Some(cell) = buffer
                            .get_mut(row as usize)
                            .and_then(|r| r.get_mut(col as usize))
                        {
                            cell.clear(TerminalColor::Default);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn erase_line(&mut self, mode: u16) {
        let row = self.cursor.row as usize;
        let cursor_col = self.cursor.col;
        let width = self.width;

        let buffer = self.current_buffer();
        if let Some(line) = buffer.get_mut(row) {
            match mode {
                0 => {
                    for col in cursor_col..width {
                        if let Some(cell) = line.get_mut(col as usize) {
                            cell.clear(TerminalColor::Default);
                        }
                    }
                }
                2 => {
                    for cell in line.iter_mut() {
                        cell.clear(TerminalColor::Default);
                    }
                }
                _ => {}
            }
        }
    }

    fn set_graphics_rendition(&mut self, params: &[u16]) {
        if params.is_empty() {
            self.cursor.style.reset();
            return;
        }

        for &param in params {
            match param {
                0 => self.cursor.style.reset(),
                1 => self.cursor.style.bold = true,
                3 => self.cursor.style.italic = true,
                4 => self.cursor.style.underline = true,
                7 => self.cursor.style.reverse = true,
                22 => self.cursor.style.bold = false,
                23 => self.cursor.style.italic = false,
                24 => self.cursor.style.underline = false,
                27 => self.cursor.style.reverse = false,
                30..=37 => {
                    self.cursor.style.foreground = TerminalColor::Indexed((param - 30) as u8);
                }
                39 => self.cursor.style.foreground = TerminalColor::Default,
                40..=47 => {
                    self.cursor.style.background = TerminalColor::Indexed((param - 40) as u8);
                }
                49 => self.cursor.style.background = TerminalColor::Default,
                _ => {}
            }
        }
    }

    pub fn size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    pub fn cursor_position(&self) -> (u16, u16) {
        (self.cursor.col, self.cursor.row)
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn working_directory(&self) -> Option<&str> {
        self.working_directory.as_deref()
    }

    pub fn cell_at(&self, col: u16, row: u16) -> Option<&TerminalCell> {
        let buffer = self.current_buffer_ref();
        buffer.get(row as usize)?.get(col as usize)
    }

    pub fn cursor_visible(&self) -> bool {
        self.modes.cursor_visible && self.cursor.visible
    }

    pub fn cursor_shape(&self) -> super::cursor::CursorShape {
        self.cursor.shape
    }
}
