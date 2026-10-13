//! What a sidebar paints (NAV-007, NAV-008) and tells the screen reader
//! (NAV-010): the bar on `surface` with its edge, the header, the item
//! rows, the footer and the toggle, and, while collapsed with the focus in
//! its tree, the cursor's label beside it.

use super::rows::{self, Row};
use super::{
    Hover, Places, Sidebar, SidebarProps, ACTIVATE_EVENT, FOCUS_EVENT, TOGGLE, TOGGLE_EVENT, TREE,
};
use crate::accessibility::{Node, Role};
use crate::component::{Element, FocusProps, LayoutInfo, LayoutType};
use crate::event::CustomEvent;
use crate::widgets::display::pieces::badge::{self, BadgeContent, BadgeProps};
use crate::widgets::display::pieces::icon::Icon;
use crate::widgets::display::pieces::separator::{glyph, SeparatorStyle};
use crate::widgets::layout::look;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use unicode_width::UnicodeWidthStr;

/// A sidebar's width while expanded, unless the props set one (NAV-008).
const WIDTH: u16 = 28;

/// A collapsed sidebar's width: one icon with a cell each side of it, the
/// right one the edge (NAV-008).
const COLLAPSED_WIDTH: u16 = 3;

/// The cells a child is indented past its group (NAV-008).
const INDENT: usize = 2;

/// The keyboard cursor's row while the sidebar holds the focus.
const CURSOR: &str = "bg-selection text-selection-foreground rounded";
/// The active item's row.
const ACTIVE: &str = "bg-accent text-accent-foreground rounded";
/// The row under the pointer.
const HOVERED: &str = "bg-hover rounded";

/// How one row is filled, which sets the colors of its glyphs.
#[derive(Clone, Copy, PartialEq)]
enum Fill {
    /// On the bar's `surface`.
    Plain,
    /// On `hover`, under the pointer.
    Hovered,
    /// On a fill whose foreground every glyph of the row takes.
    Strong(&'static str),
}

impl Fill {
    fn classes(self) -> &'static str {
        match self {
            Fill::Plain => "",
            Fill::Hovered => HOVERED,
            Fill::Strong(classes) => classes,
        }
    }

    /// The class of a glyph painted `text-muted` on a plain row.
    fn muted(self) -> &'static str {
        match self {
            Fill::Strong(_) => "",
            _ => look::MUTED,
        }
    }

    /// The class of a label: `foreground` on a plain row, `text-muted`
    /// when disabled.
    fn label(self, disabled: bool) -> &'static str {
        match self {
            Fill::Strong(_) => "",
            _ if disabled => look::DISABLED,
            _ => look::TEXT,
        }
    }
}

/// A node that hides an element's glyphs from the screen reader: the node
/// of the row or the bar they draw speaks for them.
fn hidden() -> Node {
    let mut node = Node::new(Role::Label);
    node.set_hidden();
    node
}

/// `cells` blank cells on one row.
fn blank(cells: usize) -> Element {
    Element::layout(LayoutType::Flex).class(format!("w-{cells} h-1 shrink-0"))
}

/// `text` on one row, kept whole.
fn glyphs(text: impl Into<String>, classes: &str) -> Element {
    Element::text(text).class(format!("shrink-0 h-1 whitespace-pre {classes}"))
}

/// A focus stop's focus callbacks: they record that `part` holds the focus.
fn focus_of(focus: &Arc<AtomicU8>, part: u8, focusable: bool) -> FocusProps {
    let gained = focus.clone();
    let lost = focus.clone();
    FocusProps {
        focusable,
        on_focus: Some(Arc::new(move || gained.store(part, Ordering::Release))),
        on_blur: Some(Arc::new(move || {
            let _ =
                lost.compare_exchange(part, super::NO_PART, Ordering::AcqRel, Ordering::Acquire);
        })),
        ..FocusProps::input()
    }
}

/// A layout callback that records an element's place with `record`, and
/// asks for a render when it moved or changed its size.
fn recorded(
    places: &Arc<Mutex<Places>>,
    record: fn(&mut Places) -> &mut Option<LayoutInfo>,
) -> crate::component::element::LayoutCallback {
    let places = places.clone();
    Arc::new(move |layout| {
        let mut places = places.lock().unwrap_or_else(PoisonError::into_inner);
        let slot = record(&mut places);
        let changed = *slot != Some(layout);
        *slot = Some(layout);
        changed
    })
}

/// The badge an item carries, as the badge piece draws it.
fn badge_of(text: &str) -> BadgeProps {
    BadgeProps {
        content: BadgeContent::Text(text.to_owned()),
        classes: vec!["shrink-0".to_owned()],
        ..BadgeProps::default()
    }
}

impl Sidebar {
    /// The whole bar.
    pub(super) fn paint(&self, props: &SidebarProps) -> Element {
        let rows = self.rows(props);
        let cursor = self.cursor_row(&rows);
        let scroll = self.view_scroll(rows.len(), cursor);
        let width = if self.collapsed {
            COLLAPSED_WIDTH
        } else {
            props.width.unwrap_or(WIDTH)
        };
        let height = self
            .viewport
            .map_or(0, |layout| layout.content_size().1.max(0.0) as u16);

        let mut column = Vec::new();
        if !self.collapsed {
            if let Some(header) = &props.header {
                column.push(
                    Element::layout(LayoutType::Flex)
                        .class("flex flex-col shrink-0 w-full")
                        .with_child(header.clone())
                        .with_key("header"),
                );
            }
        }
        column.push(self.tree(props, &rows, cursor, scroll));
        if !self.collapsed {
            if let Some(footer) = &props.footer {
                column.push(
                    Element::layout(LayoutType::Flex)
                        .class("flex flex-col shrink-0 w-full")
                        .with_child(footer.clone())
                        .with_key("footer"),
                );
            }
        }
        column.push(self.toggle(props));

        let line = glyph(SeparatorStyle::Line, true).to_string();
        let edge = Element::layout(LayoutType::Flex)
            .class(format!(
                "flex flex-col shrink-0 w-1 h-full {}",
                look::DIVIDER
            ))
            .with_children(
                (0..height)
                    .map(|_| Element::text(line.clone()).class("shrink-0 h-1 w-1"))
                    .collect(),
            )
            .with_accessibility(hidden())
            .with_key("edge");
        let mut children = vec![
            Element::layout(LayoutType::Flex)
                .class("flex flex-col flex-1 min-w-0 h-full overflow-hidden")
                .with_children(column)
                .with_key("column"),
            edge,
        ];
        if self.collapsed && self.focused_part() == TREE {
            if let Some(cursor) = cursor {
                // Beside the bar as it is laid out: a class may size it
                // other than the collapsed width (NAV-008, NAV-009).
                let edge = self
                    .viewport
                    .map_or(width, |layout| layout.content_size().0.max(0.0) as u16)
                    .max(1);
                children.push(self.beside(&rows[cursor], cursor - scroll.min(cursor), edge));
            }
        }
        let mut node = Node::new(Role::Navigation);
        if let Some(label) = &props.aria_label {
            node.set_label(label.clone());
        }
        // The bar fills the height its parent allots unless a class sizes
        // it (NAV-008).
        Element::layout(LayoutType::Flex)
            .class(format!(
                "relative flex flex-row shrink-0 w-{width} h-full {}",
                look::BAR
            ))
            .with_children(children)
            .with_accessibility(node)
    }

    /// The item tree: a focus stop holding one row for each item in view.
    fn tree(
        &self,
        props: &SidebarProps,
        rows: &[Row<'_>],
        cursor: Option<usize>,
        scroll: usize,
    ) -> Element {
        let shown = self.area_rows().unwrap_or(rows.len());
        let groups = rows::has_groups(&props.items);
        let focused = self.focused_part() == TREE;
        let elements = rows
            .iter()
            .enumerate()
            .skip(scroll)
            .take(shown)
            .map(|(index, row)| {
                // A disabled item is a disabled item though the application
                // names it active: text-muted, with no fill (NAV-007).
                let fill = if focused && cursor == Some(index) {
                    Fill::Strong(CURSOR)
                } else if !row.item.disabled
                    && self.active.as_deref() == Some(row.item.id.as_str())
                {
                    Fill::Strong(ACTIVE)
                } else if !row.item.disabled
                    && self.hovered == Some(Hover::Item(row.item.id.clone()))
                {
                    Fill::Hovered
                } else {
                    Fill::Plain
                };
                self.row(row, fill, groups, focused && cursor == Some(index))
            })
            .collect();
        let enabled = rows.iter().any(|row| !row.item.disabled);
        let mut tree = Element::layout(LayoutType::Flex)
            .class("flex flex-col flex-1 min-h-0 w-full overflow-hidden")
            .with_children(elements)
            .with_accessibility(Node::new(Role::Tree))
            .with_focus(focus_of(&self.focus, TREE, enabled))
            .with_key("items");
        tree.metadata
            .layout
            .push(recorded(&self.places, |places| &mut places.items));
        tree
    }

    /// One item's row: expanded, its indent, its group's expander, its icon,
    /// its label cut with `…` and its badge at the row's end; collapsed, its
    /// icon or its label's first character. Its node is a tree item with its
    /// place among its siblings (NAV-010).
    fn row(&self, row: &Row<'_>, fill: Fill, groups: bool, cursor: bool) -> Element {
        let item = row.item;
        let badge = item.badge.as_deref().map(badge_of);
        let mut pieces = vec![blank(1)];
        if self.collapsed {
            pieces.push(match item.icon {
                Some(icon) => glyphs(icon.glyph(), fill.muted()),
                None => glyphs(
                    item.label
                        .chars()
                        .next()
                        .map(String::from)
                        .unwrap_or_default(),
                    fill.label(item.disabled),
                ),
            });
        } else {
            if row.level > 0 {
                pieces.push(blank(INDENT * row.level));
            }
            if groups {
                pieces.push(if row.is_group() {
                    let expander = if row.open {
                        Icon::ChevronDown
                    } else {
                        Icon::ChevronRight
                    };
                    glyphs(format!("{} ", expander.glyph()), fill.muted())
                } else {
                    blank(2)
                });
            }
            if let Some(icon) = item.icon {
                pieces.push(glyphs(format!("{} ", icon.glyph()), fill.muted()));
            }
            pieces.push(Element::text(item.label.clone()).class(format!(
                "flex-1 min-w-0 h-1 truncate {}",
                fill.label(item.disabled)
            )));
            pieces.push(blank(1));
            if let Some(badge) = &badge {
                pieces.push(badge::badge(badge));
            }
        }
        let visuals = Element::layout(LayoutType::Flex)
            .class("flex flex-row flex-1 min-w-0 h-1")
            .with_children(pieces)
            .with_accessibility(hidden());

        let mut node = Node::new(Role::TreeItem);
        node.set_label(item.label.clone());
        node.inner.set_level(row.level + 1);
        node.inner.set_position_in_set(row.position);
        node.inner.set_size_of_set(row.count);
        if row.is_group() {
            node.set_expanded(row.open);
        }
        node.set_selected(self.active.as_deref() == Some(item.id.as_str()));
        if let Some(badge) = &badge {
            badge::describe(badge, &mut node);
        }
        if item.disabled {
            node.set_disabled();
        } else {
            node.set_clickable();
        }
        let mut element = Element::layout(LayoutType::Flex)
            .class(format!(
                "flex flex-row h-1 w-full shrink-0 {}",
                fill.classes()
            ))
            .with_child(visuals)
            .with_accessibility(node);
        if !item.disabled {
            let options = element
                .metadata
                .accessibility_options
                .get_or_insert_default();
            // The screen reader's focus follows the keyboard cursor.
            options.focus = cursor;
            options.focus_event = Some(CustomEvent::new(FOCUS_EVENT, item.id.as_bytes().to_vec()));
            options.click_event = Some(CustomEvent::new(
                ACTIVATE_EVENT,
                item.id.as_bytes().to_vec(),
            ));
        }
        element
    }

    /// The toggle on the bar's last row: `‹` while expanded and `›` while
    /// collapsed, in `text-muted`; a button named by the sidebar, expanded
    /// with it (NAV-009, NAV-010).
    fn toggle(&self, props: &SidebarProps) -> Element {
        let fill = if self.focused_part() == TOGGLE {
            Fill::Strong(look::FOCUSED)
        } else if self.hovered == Some(Hover::Toggle) {
            Fill::Hovered
        } else {
            Fill::Plain
        };
        let mark = if self.collapsed {
            Icon::ChevronRight
        } else {
            Icon::ChevronLeft
        };
        let visuals = Element::layout(LayoutType::Flex)
            .class("flex flex-row flex-1 min-w-0 h-1")
            .with_children(vec![blank(1), glyphs(mark.glyph(), fill.muted())])
            .with_accessibility(hidden());
        let mut node = Node::new(Role::Button);
        if let Some(label) = &props.aria_label {
            node.set_label(label.clone());
        }
        node.set_expanded(!self.collapsed);
        node.set_clickable();
        let mut toggle = Element::layout(LayoutType::Flex)
            .class(format!(
                "flex flex-row h-1 w-full shrink-0 {}",
                match fill {
                    Fill::Hovered => look::HOVER,
                    other => other.classes(),
                }
            ))
            .with_child(visuals)
            .with_accessibility(node)
            .with_focus(focus_of(&self.focus, TOGGLE, true))
            .with_key("toggle");
        toggle
            .metadata
            .accessibility_options
            .get_or_insert_default()
            .click_event = Some(CustomEvent::new(TOGGLE_EVENT, Vec::new()));
        toggle
            .metadata
            .layout
            .push(recorded(&self.places, |places| &mut places.toggle));
        toggle
    }

    /// The cursor's label beside a collapsed bar, on its row, painted as
    /// NAV-001 paints a tooltip (NAV-007, NAV-009). `line` is the row's
    /// line in the items area.
    fn beside(&self, row: &Row<'_>, line: usize, width: u16) -> Element {
        let places = *self.places.lock().unwrap_or_else(PoisonError::into_inner);
        let top = match (places.items, self.viewport) {
            (Some(items), Some(root)) => (items.transform[5] - root.transform[5]).max(0.0) as usize,
            _ => 0,
        };
        let text = format!(" {} ", row.item.label);
        // Its own width, as an absolute box is otherwise held to the bar's.
        let cells = UnicodeWidthStr::width(text.as_str());
        Element::text(text)
            .class(format!(
                "absolute z-10 h-1 w-{cells} whitespace-pre left-{width} top-{} {}",
                top + line,
                look::TOOLTIP
            ))
            .with_accessibility(hidden())
            .with_key("label")
    }
}
