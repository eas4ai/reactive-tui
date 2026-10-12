//! How a list paints (DAT-006, DAT-007) and what it tells the screen reader
//! (DAT-009): the filter row as a text input's field, one row per line in
//! view, each section's rows in a group named by its title, the loading row
//! as the spinner piece and the empty view as the empty piece.

use super::lines::{matches, needle, Line};
use super::{List, ListProps, FOCUS_EVENT};
use crate::accessibility::{Node, Role};
use crate::component::{Element, ElementType, FocusProps, LayoutType};
use crate::event::CustomEvent;
use crate::layout::style::{Direction, StyleBuilder};
use crate::widgets::display::look;
use crate::widgets::display::pieces::empty::{Empty, EmptyProps};
use crate::widgets::display::pieces::spinner::{Spinner, SpinnerProps};
use crate::widgets::input::look as field_look;
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// An element the screen reader skips: its text is told by the node of
/// the row or the field around it.
fn quiet(element: Element) -> Element {
    let mut node = Node::new(Role::GenericContainer);
    node.set_hidden();
    element.with_accessibility(node)
}

/// A run of text in `class`, on one row.
fn piece(text: &str, class: &str) -> Element {
    quiet(Element::text(text).class(format!("shrink-0 whitespace-pre {class}")))
}

/// A box of `width` by `height` cells at `top` in its parent.
fn placed(top: usize, width: usize, height: usize) -> crate::builder::ElementBuilder {
    crate::builder::ElementBuilder::new(ElementType::Layout(LayoutType::Flex)).styles(
        StyleBuilder::new()
            .position_absolute()
            .inset_left(0.0)
            .inset_top(top as f32)
            .width_px(width as f32)
            .height_px(height as f32)
            .overflow_hidden(),
    )
}

impl List {
    /// The rows of the body the screen shows: those of the view inside the
    /// clip of the list's ancestors, so a list taller than the screen builds
    /// no row the screen cuts away (DAT-007).
    fn rows_on_screen(&self, props: &ListProps) -> Range<usize> {
        let body = self.body(props);
        let Some(layout) = self.viewport else {
            return 0..body;
        };
        let [a, b, c, d, _, y] = layout.transform;
        // A list that is scaled or turned builds every row of its body.
        let upright = a == 1.0 && b == 0.0 && c == 0.0 && d == 1.0;
        if !upright {
            return 0..body;
        }
        let top = y + Self::filter_rows(props) as f32;
        let first = (layout.clip.y - top).floor().max(0.0) as usize;
        let end = (layout.clip.y + layout.clip.height - top).ceil().max(0.0) as usize;
        first.min(body)..end.min(body)
    }

    /// The row of the item `item`, the `place`th shown, by its state
    /// (DAT-006): the cursor's while the list holds the focus, the selected
    /// one, the one under the pointer and a disabled one take their fills
    /// and colors, the others show their label in `foreground`, their
    /// description in `text-muted` and the characters that match the filter
    /// in `primary`.
    fn item_row(
        &self,
        props: &ListProps,
        item: usize,
        place: usize,
        top: usize,
        width: usize,
    ) -> Element {
        let entry = &props.items[item];
        let cursor = self.focused && self.cursor == Some(item);
        let selected = self.selected.as_deref() == Some(entry.id.as_str());
        let hovered = self.hover == Some(item);
        let state = look::row(cursor, selected, hovered, entry.disabled);
        // A filled row takes its fill's text throughout; its fill is drawn
        // round where the terminal takes pixels (PIX-009).
        let filled = !entry.disabled && (cursor || selected || hovered);
        let own = entry.disabled || cursor || selected;
        let mut pieces = vec![piece(" ", "")];
        let chars: Vec<char> = entry.label.chars().collect();
        let mut at = 0;
        let found = if own {
            Vec::new()
        } else {
            matches(&entry.label, &needle(&self.filter))
        };
        for range in found {
            if range.start > at {
                pieces.push(piece(
                    &chars[at..range.start].iter().collect::<String>(),
                    "",
                ));
            }
            pieces.push(piece(
                &chars[range.clone()].iter().collect::<String>(),
                look::MATCH,
            ));
            at = range.end;
        }
        if at < chars.len() {
            pieces.push(piece(&chars[at..].iter().collect::<String>(), ""));
        }
        if let Some(description) = &entry.description {
            pieces.push(piece("  ", ""));
            pieces.push(piece(description, if own { "" } else { look::MUTED }));
        }
        let mut node = Node::new(Role::ListBoxOption);
        node.set_label(entry.label.clone());
        if let Some(description) = &entry.description {
            node.set_description(description.clone());
        }
        node.set_selected(selected);
        node.inner.set_position_in_set(place + 1);
        node.inner.set_size_of_set(self.lines.shown);
        if entry.disabled {
            node.set_disabled();
        }
        let class = if filled {
            format!("flex flex-row {state} {}", look::ROUNDED)
        } else {
            format!("flex flex-row {state}")
        };
        let mut row = placed(top, width, 1)
            .class(&class)
            .children(pieces)
            .build()
            .with_key(format!("item:{item}"))
            .disabled(entry.disabled)
            .with_accessibility(node);
        if !entry.disabled {
            let options = row.metadata.accessibility_options.get_or_insert_default();
            options.focus = cursor;
            options.focus_event =
                Some(CustomEvent::new(FOCUS_EVENT, item.to_string().into_bytes()));
        }
        row
    }

    /// The element of line `line`, on row `top` of its parent.
    fn line_element(
        &self,
        props: &ListProps,
        line: usize,
        top: usize,
        width: usize,
    ) -> Option<Element> {
        Some(match self.lines.get(line)? {
            Line::Title { item } => placed(top, width, 1)
                .class("flex flex-row")
                .children(vec![
                    piece(" ", ""),
                    piece(
                        props.items[item].section.as_deref().unwrap_or_default(),
                        look::MUTED,
                    ),
                ])
                .build()
                .with_key(format!("title:{item}")),
            Line::Item { item, place, .. } => self.item_row(props, item, place, top, width),
            // The loading row is the spinner piece; its status, which has
            // no words, is not told: the list box is busy instead.
            Line::Loading => placed(top, width, 1)
                .class("flex flex-row")
                .children(vec![
                    piece(" ", ""),
                    quiet(Element::typed::<Spinner>(SpinnerProps {
                        label: String::new(),
                        class: String::new(),
                        animated: true,
                    })),
                ])
                .build()
                .with_key("loading"),
        })
    }

    /// The rows of `lines` in view, at their places in the body: a
    /// section's rows sit in a group named by its title.
    fn body_rows(&self, props: &ListProps, lines: Range<usize>, width: usize) -> Vec<Element> {
        let mut rows = Vec::new();
        let mut line = lines.start;
        while line < lines.end {
            let title = match self.lines.get(line) {
                Some(Line::Title { .. }) => Some(line),
                Some(Line::Item { title, .. }) => title,
                _ => None,
            };
            let Some(title) = title else {
                rows.extend(self.line_element(props, line, line - self.offset, width));
                line += 1;
                continue;
            };
            // The lines of this section in view.
            let start = line;
            line += 1;
            while line < lines.end
                && matches!(self.lines.get(line), Some(Line::Item { title: Some(t), .. }) if t == title)
            {
                line += 1;
            }
            let members = (start..line)
                .filter_map(|member| self.line_element(props, member, member - start, width))
                .collect();
            let name = match self.lines.get(title) {
                Some(Line::Title { item }) => props.items[item].section.clone().unwrap_or_default(),
                _ => String::new(),
            };
            let mut group = Node::new(Role::Group);
            group.set_label(name);
            rows.push(
                placed(start - self.offset, width, line - start)
                    .children(members)
                    .build()
                    .with_key(format!("section:{title}"))
                    .with_accessibility(group),
            );
        }
        rows
    }

    /// The filter row, painted as a focused text input's field while the
    /// list holds the focus and as an unfocused one's otherwise (DAT-006),
    /// told to the screen reader as a search input named by the list.
    fn filter_row(&self, props: &ListProps, width: usize) -> Element {
        let field = width.saturating_sub(2);
        let glyphs: Vec<(&str, usize)> = self
            .filter
            .graphemes(true)
            .map(|glyph| (glyph, UnicodeWidthStr::width(glyph)))
            .collect();
        let text: usize = glyphs.iter().map(|(_, cells)| cells).sum();
        // The text scrolls so the cursor, after it, stays in view.
        let cursor = self.focused.then_some(text);
        let scroll = match cursor {
            Some(cursor) if cursor >= field => cursor + 1 - field,
            _ => 0,
        };
        let mut runs: Vec<(String, &'static str)> = Vec::new();
        let mut push = |text: &str, style: &'static str| match runs.last_mut() {
            Some((run, last)) if *last == style => run.push_str(text),
            _ => runs.push((text.to_owned(), style)),
        };
        let pixels = field_look::pixels();
        let (text_style, pad) = if pixels {
            (field_look::FIELD_PICTURE, field_look::FIELD_PAD)
        } else {
            (field_look::FIELD, field_look::FIELD)
        };
        let mut painted = 0;
        let mut column = 0;
        for (glyph, cells) in &glyphs {
            let start = column;
            column += cells;
            if start < scroll {
                continue;
            }
            let x = start - scroll;
            if x + cells > field {
                break;
            }
            if x > painted {
                push(&" ".repeat(x - painted), pad);
            }
            push(glyph, text_style);
            painted = x + cells;
        }
        if let Some(cursor) = cursor
            .map(|cursor| cursor - scroll)
            .filter(|cursor| *cursor < field)
        {
            if cursor > painted {
                push(&" ".repeat(cursor - painted), pad);
            }
            push(" ", field_look::CURSOR);
            painted = cursor + 1;
        }
        push(&" ".repeat(field.saturating_sub(painted)), pad);
        let mut node = Node::new(Role::SearchInput);
        if let Some(label) = &props.aria_label {
            node.set_label(label.clone());
        }
        node.set_value(self.filter.clone());
        let row = if pixels {
            Self::pictured_field(runs, field, self.focused)
        } else {
            let frame = field_look::field_frame(self.focused);
            let mut pieces = vec![piece("[", frame)];
            pieces.extend(runs.iter().map(|(text, style)| piece(text, style)));
            pieces.push(piece("]", frame));
            Element::layout(LayoutType::Flex)
                .class("flex flex-row h-1 shrink-0")
                .children(pieces)
        };
        row.with_key("filter").with_accessibility(node)
    }

    /// The filter's field where the terminal takes pixels, as a text input
    /// draws its field (PIX-004): a picture in `input` over the field and
    /// its two frame cells, which hold no glyph; the text and the cursor
    /// stay cells.
    #[cfg(feature = "wgpu-graphics")]
    fn pictured_field(runs: Vec<(String, &'static str)>, field: usize, focused: bool) -> Element {
        let mut children = vec![field_look::blank(1)];
        children.extend(runs.into_iter().map(|(text, style)| {
            if style == field_look::FIELD_PAD {
                field_look::blank(UnicodeWidthStr::width(text.as_str()))
            } else {
                piece(&text, style)
            }
        }));
        children.push(field_look::blank(1));
        let mut element = Element::layout(LayoutType::Flex)
            .class(format!(
                "flex flex-row h-1 shrink-0 w-{} bg-input",
                field + 2
            ))
            .children(children);
        element.metadata.look = Some(std::sync::Arc::new(crate::graphics::look::Look::field(
            focused, false, false,
        )));
        element
    }

    /// Without the pixel looks the field is never a picture.
    #[cfg(not(feature = "wgpu-graphics"))]
    fn pictured_field(runs: Vec<(String, &'static str)>, _field: usize, _focused: bool) -> Element {
        Element::layout(LayoutType::Flex)
            .class("flex flex-row h-1 shrink-0")
            .children(
                runs.iter()
                    .map(|(text, style)| piece(text, style))
                    .collect(),
            )
    }

    /// The list's element: its box takes no fill of its own and fills what
    /// its parent allots unless the props set a size (DAT-006, DAT-007).
    pub(super) fn paint(&self, props: &ListProps) -> Element {
        let (width, height) = self.size(props);
        let filter_rows = Self::filter_rows(props);
        let body = height.saturating_sub(filter_rows);
        let mut children = Vec::new();
        if props.filterable && height > 0 {
            children.push(self.filter_row(props, width));
        }
        let screen = self.rows_on_screen(props);
        let lines = (self.offset + screen.start).min(self.lines.len())
            ..(self.offset + screen.end).min(self.lines.len());
        let mut rows = self.body_rows(props, lines, width);
        if self.lines.shown == 0 {
            // The empty piece under the loading row, if any (DAT-008).
            rows.push(
                placed(
                    self.lines.len().saturating_sub(self.offset),
                    width,
                    body.saturating_sub(self.lines.len()),
                )
                .children(vec![Element::typed::<Empty>(EmptyProps {
                    title: props.empty_text.clone().unwrap_or_default(),
                    ..EmptyProps::default()
                })])
                .build()
                .with_key("empty"),
            );
        }
        let mut rows = placed(filter_rows, width, body)
            .children(rows)
            .build()
            .with_key("rows");
        if props.loading {
            // While the application loads, the screen reader is told the
            // list box is busy, as a skeleton tells it (DIS-004).
            rows = rows.with_busy_parent();
        }
        children.push(rows);
        // The rows are absolute, so this child gives the list its natural
        // height inside a parent that allots none.
        let natural = filter_rows + self.lines.len() + if self.lines.shown == 0 { 2 } else { 0 };
        children.push(
            crate::builder::ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
                .styles(StyleBuilder::new().width_px(0.0).height_px(natural as f32))
                .build()
                .with_key("intrinsic-size"),
        );
        let mut style = StyleBuilder::new()
            .display_flex()
            .direction(Direction::Column)
            .overflow_hidden();
        style = match props.height {
            Some(height) => style.height_px(f32::from(height)),
            None => style
                .height_percent(100.0)
                .min_height_px(0.0)
                .max_height_percent(100.0),
        };
        style = match props.width {
            Some(width) => style.width_px(f32::from(width)),
            None => style.width_percent(100.0),
        };
        let mut node = Node::new(Role::ListBox);
        if let Some(label) = &props.aria_label {
            node.set_label(label.clone());
        }
        crate::builder::ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(style)
            .class("min-w-0")
            .children(children)
            .build()
            .with_focus(FocusProps::input())
            .with_accessibility(node)
    }
}
