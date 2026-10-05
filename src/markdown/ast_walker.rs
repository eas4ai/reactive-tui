//! AST walker for converting comrak AST to StyledRun/StyledLine representation

use super::converter::*;
use crate::core::styled_text::{StyledLine, StyledRun};
use crate::core::surface::{Attr, Rgba};
use comrak::nodes::{AstNode, ListType, NodeTable, NodeValue, TableAlignment};
use std::collections::HashMap;
use unicode_width::UnicodeWidthStr;

/// AST walker that converts comrak AST nodes to StyledRun/StyledLine
pub struct AstWalker {
    enable_syntax_highlighting: bool,
    /// Styled lines for rendering
    pub lines: Vec<StyledLine>,
    current_line: Vec<StyledRun>,
    /// Source position mapping for debugging
    pub sourcepos_map: HashMap<usize, (usize, usize)>,
    list_depth: usize,
}

impl AstWalker {
    /// Create a new AST walker with syntax highlighting option
    pub fn new(enable_syntax_highlighting: bool) -> Self {
        Self {
            enable_syntax_highlighting,
            lines: Vec::new(),
            current_line: Vec::new(),
            sourcepos_map: HashMap::new(),
            list_depth: 0,
        }
    }

    /// Walk the document AST and return styled lines
    pub fn walk_document<'a>(mut self, root: &'a AstNode<'a>) -> Vec<StyledLine> {
        self.walk_node(root);
        self.flush_current_line();
        optimize_styled_lines(self.lines)
    }

    /// Walk the AST and finish processing
    pub fn walk_and_finish<'a>(&mut self, root: &'a AstNode<'a>) {
        self.walk_node(root);
        self.flush_current_line();
    }

    /// Finish processing and return optimized styled lines
    pub fn finish(self) -> Vec<StyledLine> {
        optimize_styled_lines(self.lines)
    }

    /// Take ownership of the source position map
    pub fn take_sourcepos_map(self) -> HashMap<usize, (usize, usize)> {
        self.sourcepos_map
    }

    fn walk_node<'a>(&mut self, node: &'a AstNode<'a>) {
        // Store source position for debugging and selection
        let sourcepos = node.data.borrow().sourcepos;
        let line_index = self.lines.len();
        self.sourcepos_map
            .insert(line_index, (sourcepos.start.line, sourcepos.start.column));

        match &node.data.borrow().value {
            NodeValue::Document => {
                for child in node.children() {
                    self.walk_node(child);
                }
            }

            // Block elements
            NodeValue::Paragraph => {
                self.walk_children(node);
                self.add_line_break();
            }

            NodeValue::Heading(heading) => {
                // Add heading marker
                let marker = "#".repeat(heading.level as usize) + " ";
                let heading_run = create_heading_run(&marker, heading.level as u32);
                self.current_line.push(heading_run);

                // Apply heading style to content
                let start_idx = self.current_line.len();
                self.walk_children(node);

                // Make content bold as well
                for run in &mut self.current_line[start_idx..] {
                    run.attr |= Attr::BOLD;
                    run.fg = match heading.level {
                        1 => Rgba::white(),
                        2 => rgba(200, 200, 200, 255),
                        3 => rgba(150, 150, 150, 255),
                        _ => rgba(128, 128, 128, 255),
                    };
                }

                self.add_line_break();
                self.add_line_break();
            }

            NodeValue::List(list_data) => {
                self.list_depth += 1;
                let mut item_number = list_data.start;

                for child in node.children() {
                    if let NodeValue::Item(_) = &child.data.borrow().value {
                        // Add list item marker
                        let indent = "  ".repeat(self.list_depth.saturating_sub(1));
                        let marker = match list_data.list_type {
                            ListType::Bullet => "• ".to_string(),
                            ListType::Ordered => {
                                let marker = format!("{}. ", item_number);
                                item_number += 1;
                                marker
                            }
                        };

                        self.add_text(&indent);
                        self.add_text(&marker);
                        self.walk_node(child);
                    }
                }

                self.list_depth -= 1;
                if self.list_depth == 0 {
                    self.add_line_break();
                }
            }

            NodeValue::Item(_) => {
                self.walk_children(node);
                if self.list_depth > 0 {
                    self.add_line_break();
                }
            }

            NodeValue::BlockQuote => {
                let quote_run = create_quote_run("│ ");
                self.current_line.push(quote_run);

                let start_idx = self.current_line.len();
                self.walk_children(node);

                // Apply quote styling to content
                for run in &mut self.current_line[start_idx..] {
                    run.fg = rgba(128, 128, 128, 255);
                    run.attr |= Attr::ITALIC;
                }

                self.add_line_break();
            }

            NodeValue::CodeBlock(code_block) => {
                let code = &code_block.literal;

                self.add_line_break();
                let language = code_block.info.split_whitespace().next().unwrap_or("");
                let highlighted = self
                    .enable_syntax_highlighting
                    .then(|| crate::syntax::SyntaxHighlighter::new(language))
                    .flatten()
                    .map(|mut highlighter| highlighter.highlight_text(code));
                if let Some(lines) = highlighted {
                    for line in lines {
                        self.add_text("  ");
                        self.current_line.extend(line.runs);
                        self.add_line_break();
                    }
                } else {
                    for line in code.lines() {
                        self.add_text("  ");
                        self.current_line.push(create_code_run(line));
                        self.add_line_break();
                    }
                }
                self.add_line_break();
            }

            NodeValue::Table(table) => {
                self.walk_table(node, table);
            }

            NodeValue::ThematicBreak => {
                self.add_line_break();
                self.add_text("────────────────────────────");
                self.add_line_break();
                self.add_line_break();
            }

            // Inline elements
            NodeValue::Text(text) => {
                self.add_text(text);
            }

            NodeValue::Strong => {
                let start_idx = self.current_line.len();
                self.walk_children(node);
                // Apply bold to all runs added during children walk
                for run in &mut self.current_line[start_idx..] {
                    run.attr |= Attr::BOLD;
                }
            }

            NodeValue::Emph => {
                let start_idx = self.current_line.len();
                self.walk_children(node);
                // Apply italic to all runs added during children walk
                for run in &mut self.current_line[start_idx..] {
                    run.attr |= Attr::ITALIC;
                }
            }

            NodeValue::Strikethrough => {
                let start_idx = self.current_line.len();
                self.walk_children(node);
                // Apply strikethrough to all runs added during children walk
                for run in &mut self.current_line[start_idx..] {
                    run.attr |= Attr::STRIKE;
                }
            }

            NodeValue::Code(code) => {
                let code_run = create_inline_code_run(&code.literal);
                self.current_line.push(code_run);
            }

            NodeValue::Link(_link) => {
                let start_idx = self.current_line.len();
                self.walk_children(node);
                // Apply link styling to all runs added during children walk
                for run in &mut self.current_line[start_idx..] {
                    run.fg = rgba(100, 150, 255, 255); // Blue
                    run.attr |= Attr::UNDERLINE;
                }
            }

            NodeValue::Image(link) => {
                let img_run = create_link_run("[Image");
                self.current_line.push(img_run);

                if !link.url.is_empty() {
                    let url_run = create_link_run(&format!(": {}", link.url));
                    self.current_line.push(url_run);
                }

                let close_run = create_link_run("]");
                self.current_line.push(close_run);
            }

            NodeValue::LineBreak | NodeValue::SoftBreak => {
                self.add_line_break();
            }

            // Task list items (GFM extension)
            NodeValue::TaskItem(checked) => {
                let checkbox = if checked.symbol.is_some() {
                    "[x] "
                } else {
                    "[ ] "
                };
                self.add_text(checkbox);
                self.walk_children(node);
            }

            _ => {
                // Handle other node types by walking children
                self.walk_children(node);
            }
        }
    }

    fn walk_table<'a>(&mut self, node: &'a AstNode<'a>, table: &NodeTable) {
        self.add_line_break();
        let mut widths = vec![0; table.num_columns];
        let mut rows = Vec::new();

        // TXT-003: buffer styled cells before choosing shared column widths.
        for row in node.children() {
            let data = row.data.borrow();
            let NodeValue::TableRow(header) = data.value else {
                continue;
            };
            let sourcepos = (data.sourcepos.start.line, data.sourcepos.start.column);
            let mut cells = Vec::new();
            for (cell, width) in row.children().zip(&mut widths) {
                self.walk_children(cell);
                let runs = std::mem::take(&mut self.current_line);
                let text: String = runs.iter().map(|run| run.text.as_str()).collect();
                let cell_width = UnicodeWidthStr::width(text.as_str());
                *width = (*width).max(cell_width);
                cells.push((runs, cell_width));
            }
            // A short row still spans every column (TXT-003).
            cells.resize_with(table.num_columns, || (Vec::new(), 0));
            rows.push((header, sourcepos, cells));
        }

        for (header, sourcepos, cells) in rows {
            self.sourcepos_map.insert(self.lines.len(), sourcepos);
            self.add_text("│");
            for (column, (runs, cell_width)) in cells.into_iter().enumerate() {
                let padding = widths[column] - cell_width;
                let left = match table.alignments.get(column) {
                    Some(TableAlignment::Right) => padding,
                    Some(TableAlignment::Center) => padding / 2,
                    _ => 0,
                };
                self.add_text(&" ".repeat(left + 1));
                self.current_line.extend(runs);
                self.add_text(&" ".repeat(padding - left + 1));
                self.add_text("│");
            }
            self.add_line_break();

            if header {
                // TXT-003: the separator uses the same widths as every row.
                self.sourcepos_map.insert(self.lines.len(), sourcepos);
                self.add_text("├");
                for (column, width) in widths.iter().enumerate() {
                    if column > 0 {
                        self.add_text("┼");
                    }
                    self.add_text(&"─".repeat(width + 2));
                }
                self.add_text("┤");
                self.add_line_break();
            }
        }
        self.add_line_break();
    }

    fn walk_children<'a>(&mut self, node: &'a AstNode<'a>) {
        for child in node.children() {
            self.walk_node(child);
        }
    }

    fn add_text(&mut self, text: &str) {
        if !text.is_empty() {
            let run = plain_text_to_styled_run(text);
            self.current_line.push(run);
        }
    }

    fn add_line_break(&mut self) {
        self.flush_current_line();
    }

    fn flush_current_line(&mut self) {
        if !self.current_line.is_empty() || self.lines.is_empty() {
            let runs = std::mem::take(&mut self.current_line);
            let line = create_styled_line(runs);
            self.lines.push(line);
        }
    }
}

/// Helper function to create an rgba color
fn rgba(r: u8, g: u8, b: u8, a: u8) -> Rgba {
    Rgba {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: a as f32 / 255.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::MarkdownRenderer;

    #[test]
    fn txt_003_tables_preserve_styles_and_pad_display_columns() {
        let source =
            "| L | C | R |\n| :--- | :---: | ---: |\n| **界** | `é` | 1234 |\n| x | zz | 1 |";
        let lines = MarkdownRenderer::new().render_to_styled_lines(source);
        let text: Vec<_> = lines.iter().map(StyledLine::text).collect();
        assert_eq!(
            &text[1..],
            &[
                "│ L  │ C  │    R │",
                "├────┼────┼──────┤",
                "│ 界 │ é  │ 1234 │",
                "│ x  │ zz │    1 │",
            ]
        );
        assert!(lines[3]
            .runs
            .iter()
            .any(|run| run.text == "界" && run.attr.contains(Attr::BOLD)));
        assert!(lines[3]
            .runs
            .iter()
            .any(|run| run.text == "é" && run.bg != Rgba::transparent()));
        for line in &text[1..] {
            assert_eq!(UnicodeWidthStr::width(line.as_str()), 18);
        }
    }

    #[test]
    fn txt_003_multiple_tables_keep_widths_and_source_positions_separate() {
        let source = "| A |\n| --- |\n| longest |\n\n| X | Y |\n| --- | --- |\n| z | |";
        let renderer = MarkdownRenderer::new();
        let (lines, positions) = renderer.render_with_sourcepos(source);
        let text: Vec<_> = lines.iter().map(StyledLine::text).collect();
        assert_eq!(
            &text[1..],
            &[
                "│ A       │",
                "├─────────┤",
                "│ longest │",
                "│ X │ Y │",
                "├───┼───┤",
                "│ z │   │",
            ]
        );
        for (output, input) in [(1, 1), (2, 1), (3, 3), (4, 5), (5, 5), (6, 7)] {
            assert_eq!(positions[&output], (input, 1));
        }
        assert_eq!(
            text,
            renderer
                .render_to_styled_lines(source)
                .iter()
                .map(StyledLine::text)
                .collect::<Vec<_>>()
        );
    }
}
