//! The sidebar (docs/spec/layout-widgets.md, NAV-007 to NAV-010): a tree of
//! items in a bar at the side of the screen, with a toggle on its last row
//! that collapses it to a strip of icons. The item tree and the toggle are
//! two focus stops; the bar is a navigation landmark holding a tree.

mod paint;
mod rows;

use crate::component::{Component, Element, LayoutInfo, Props};
use crate::event::router::EventResult;
use crate::event::types::{KeyEvent, MouseButton, MouseEventKind, Position};
use crate::event::{CustomEvent, Event, MouseEvent};
use crate::keymap::{Action, Keymap};
use crate::widgets::display::pieces::icon::Icon;
use rows::Row;
use std::any::Any;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

/// The callback called with the id of the item the user made active.
pub(crate) type SelectCallback = Arc<dyn Fn(String) + Send + Sync>;

/// The callback called with the collapsed state after the user flips it.
pub(crate) type CollapseCallback = Arc<dyn Fn(bool) + Send + Sync>;

/// The events the sidebar takes for the screen reader's requests
/// (NAV-010): the focus or a click on an item, whose id is the data, and a
/// click on the toggle.
const FOCUS_EVENT: &str = "reactive_tui.sidebar.focus";
const ACTIVATE_EVENT: &str = "reactive_tui.sidebar.activate";
const TOGGLE_EVENT: &str = "reactive_tui.sidebar.toggle";

/// Which focus stop of the sidebar holds the keyboard focus.
const NO_PART: u8 = 0;
const TREE: u8 = 1;
const TOGGLE: u8 = 2;

/// One item of a sidebar. An item with children is a group.
#[derive(Clone, Debug, PartialEq)]
pub struct SidebarItem {
    /// The id the item is known by to the application.
    pub id: String,
    /// The text the user reads.
    pub label: String,
    /// The icon shown before the label, if any.
    pub icon: Option<Icon>,
    /// A short text shown at the end of the row, if any.
    pub badge: Option<String>,
    /// Whether the item ignores the keyboard and the pointer.
    pub disabled: bool,
    /// Whether a group starts open.
    pub open: bool,
    /// The item's children; an item with any is a group.
    pub children: Vec<SidebarItem>,
}

impl SidebarItem {
    /// Create an item with an id and a label, with no icon or badge.
    pub fn new(id: &str, label: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            icon: None,
            badge: None,
            disabled: false,
            open: false,
            children: Vec::new(),
        }
    }

    /// Set the icon shown before the label.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Set the badge text shown at the end of the row.
    pub fn badge(mut self, text: &str) -> Self {
        self.badge = Some(text.to_string());
        self
    }

    /// Set whether the item is disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set whether a group starts open.
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Add a child item, which makes this item a group.
    pub fn child(mut self, item: SidebarItem) -> Self {
        self.children.push(item);
        self
    }
}

/// Properties for the sidebar.
#[derive(Clone, Default, PartialEq)]
pub struct SidebarProps {
    /// The top-level items, in order.
    pub items: Vec<SidebarItem>,
    /// The id of the active item, if any.
    pub active: Option<String>,
    /// Whether the bar is collapsed to its strip of icons.
    pub collapsed: bool,
    /// An element shown above the items while the bar is expanded.
    pub header: Option<Element>,
    /// An element shown below the items while the bar is expanded.
    pub footer: Option<Element>,
    /// The name the screen reader hears for the bar and its toggle.
    pub aria_label: Option<String>,
    /// The bar's width in cells while expanded; none is 28 cells. A
    /// collapsed bar is 3 cells wide.
    pub width: Option<u16>,
}

impl Props for SidebarProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Where the items and the toggle were laid out, for the pointer.
#[derive(Clone, Copy, Default)]
struct Places {
    items: Option<LayoutInfo>,
    toggle: Option<LayoutInfo>,
}

/// What the pointer is over.
#[derive(Clone, Debug, PartialEq)]
enum Hover {
    Item(String),
    Toggle,
}

/// What a pointer event lands on.
enum Target {
    Item(usize),
    Toggle,
}

/// A sidebar: a tree of items that the user moves through and makes
/// active, and a toggle that collapses it to a strip of icons.
pub struct Sidebar {
    /// Called with the id of an item the user makes active.
    on_select: Option<SelectCallback>,
    /// Called with the collapsed state after the user flips it.
    on_collapse: Option<CollapseCallback>,
    /// The active item: the props' until the user makes another active.
    active: Option<String>,
    /// The props' active item when they were last read.
    given_active: Option<String>,
    /// Whether the bar is collapsed: the props' until the user flips it.
    collapsed: bool,
    /// The props' collapsed state when they were last read.
    given_collapsed: bool,
    /// The groups the user opened or closed, by id.
    opened: HashMap<String, bool>,
    /// The item the keyboard cursor is on, by id.
    cursor: Option<String>,
    /// The first row the items area shows.
    scroll: usize,
    /// The focus stop that holds the keyboard focus, set by its focus
    /// callbacks.
    focus: Arc<AtomicU8>,
    /// What the pointer is over.
    hovered: Option<Hover>,
    /// The bar's box from its last layout.
    viewport: Option<LayoutInfo>,
    /// Where the items area and the toggle were laid out.
    places: Arc<Mutex<Places>>,
}

impl Sidebar {
    /// Create a sidebar from its props, with no callbacks yet.
    pub fn new(props: SidebarProps) -> Self {
        Self {
            on_select: None,
            on_collapse: None,
            active: props.active.clone(),
            given_active: props.active,
            collapsed: props.collapsed,
            given_collapsed: props.collapsed,
            opened: HashMap::new(),
            cursor: None,
            scroll: 0,
            focus: Arc::new(AtomicU8::new(NO_PART)),
            hovered: None,
            viewport: None,
            places: Arc::default(),
        }
    }

    /// Set the callback called with the id of an item made active.
    pub fn with_on_select(mut self, f: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_select = Some(Arc::new(f));
        self
    }

    /// Set the callback called with the collapsed state after a flip.
    pub fn with_on_collapse(mut self, f: impl Fn(bool) + Send + Sync + 'static) -> Self {
        self.on_collapse = Some(Arc::new(f));
        self
    }

    /// Replace the select callback, or remove it with `None`: a builder's
    /// sidebar takes the one its latest render carried (CMP-008, CMP-009).
    pub(crate) fn set_on_select(&mut self, f: Option<SelectCallback>) {
        self.on_select = f;
    }

    /// Replace the collapse callback, or remove it with `None` (CMP-008).
    pub(crate) fn set_on_collapse(&mut self, f: Option<CollapseCallback>) {
        self.on_collapse = f;
    }

    /// Whether the group `item` shows its children.
    fn is_open(&self, item: &SidebarItem) -> bool {
        self.opened.get(&item.id).copied().unwrap_or(item.open)
    }

    /// The rows the bar shows.
    fn rows<'a>(&self, props: &'a SidebarProps) -> Vec<Row<'a>> {
        rows::visible(&props.items, &|item| self.is_open(item))
    }

    /// Which focus stop holds the keyboard focus.
    fn focused_part(&self) -> u8 {
        self.focus.load(Ordering::Acquire)
    }

    /// The row the keyboard cursor stands on: the cursor's item when it
    /// shows and is enabled, else the active item's, else the first enabled
    /// row.
    fn cursor_row(&self, rows: &[Row<'_>]) -> Option<usize> {
        let enabled = |id: &String| {
            rows.iter()
                .position(|row| row.item.id == *id && !row.item.disabled)
        };
        self.cursor
            .as_ref()
            .and_then(enabled)
            .or_else(|| self.active.as_ref().and_then(enabled))
            .or_else(|| rows.iter().position(|row| !row.item.disabled))
    }

    /// The rows the items area holds, once it was laid out.
    fn area_rows(&self) -> Option<usize> {
        self.places
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .items
            .map(|layout| layout.content_size().1.max(0.0) as usize)
    }

    /// The first row the items area shows: the last one kept, moved just
    /// enough that the cursor's row shows.
    fn view_scroll(&self, count: usize, cursor: Option<usize>) -> usize {
        let Some(height) = self.area_rows().filter(|height| *height > 0) else {
            return 0;
        };
        let mut scroll = self.scroll.min(count.saturating_sub(height));
        if let Some(cursor) = cursor {
            if cursor < scroll {
                scroll = cursor;
            } else if cursor >= scroll + height {
                scroll = cursor + 1 - height;
            }
        }
        scroll
    }

    /// Keeps the cursor's row in view after it moved.
    fn keep_in_view(&mut self, props: &SidebarProps) {
        let rows = self.rows(props);
        let cursor = self.cursor_row(&rows);
        self.scroll = self.view_scroll(rows.len(), cursor);
    }

    /// Makes `row`'s item active and reports it through `on_select`, and
    /// opens or closes it when it is a group (NAV-009).
    fn activate(&mut self, row: &Row<'_>) {
        if row.item.disabled {
            return;
        }
        let id = row.item.id.clone();
        if row.is_group() {
            self.opened.insert(id.clone(), !row.open);
        }
        self.cursor = Some(id.clone());
        self.active = Some(id.clone());
        if let Some(callback) = &self.on_select {
            callback(id);
        }
    }

    /// Collapses or expands the bar and reports it through `on_collapse`.
    fn flip(&mut self) -> EventResult {
        self.collapsed = !self.collapsed;
        if let Some(callback) = &self.on_collapse {
            callback(self.collapsed);
        }
        EventResult::Consumed
    }

    /// The tree's keys (NAV-009): Up, Down, Home and End move the cursor
    /// over the enabled rows; Right opens a closed group or moves into an
    /// open one; Left closes an open group or moves to the group that
    /// holds the item; Enter and Space activate the item.
    fn tree_key(&mut self, key: &KeyEvent, props: &SidebarProps) -> EventResult {
        let Some(action) = Keymap::active().action(key) else {
            return EventResult::Ignored;
        };
        let rows = self.rows(props);
        let Some(current) = self.cursor_row(&rows) else {
            return EventResult::Ignored;
        };
        let enabled = |index: &usize| !rows[*index].item.disabled;
        let row = &rows[current];
        let target = match action {
            Action::Up => (0..current).rev().find(enabled),
            Action::Down => (current + 1..rows.len()).find(enabled),
            Action::Home => (0..rows.len()).find(enabled),
            Action::End => (0..rows.len()).rev().find(enabled),
            Action::Right | Action::Expand if row.is_group() && !row.open => {
                self.opened.insert(row.item.id.clone(), true);
                None
            }
            Action::Right if row.open => (current + 1..rows.len())
                .take_while(|index| rows[*index].level > row.level)
                .find(|index| rows[*index].parent == Some(current) && enabled(index)),
            Action::Left | Action::Collapse if row.open => {
                self.opened.insert(row.item.id.clone(), false);
                None
            }
            Action::Left => row.parent.filter(enabled),
            Action::Confirm | Action::Activate => {
                self.activate(row);
                self.keep_in_view(props);
                return EventResult::Consumed;
            }
            Action::Right | Action::Expand | Action::Collapse => None,
            _ => return EventResult::Ignored,
        };
        if let Some(target) = target {
            self.cursor = Some(rows[target].item.id.clone());
        } else if self.cursor.is_none() {
            self.cursor = Some(row.item.id.clone());
        }
        self.keep_in_view(props);
        EventResult::Consumed
    }

    /// The toggle's keys: Enter and Space collapse or expand the bar.
    fn toggle_key(&mut self, key: &KeyEvent) -> EventResult {
        match Keymap::active().action(key) {
            Some(Action::Confirm | Action::Activate) => self.flip(),
            _ => EventResult::Ignored,
        }
    }

    /// What the pointer is over: an item's row, by its index among the
    /// rows, or the toggle.
    fn target(&self, mouse: &MouseEvent, props: &SidebarProps) -> Option<Target> {
        let Position::Cell { x, y } = mouse.position else {
            return None;
        };
        let root = self.viewport?;
        let [a, b, c, d, tx, ty] = root.transform;
        let (x, y) = (f32::from(x), f32::from(y));
        let (x, y) = (a * x + c * y + tx, b * x + d * y + ty);
        let places = *self.places.lock().unwrap_or_else(PoisonError::into_inner);
        if places
            .toggle
            .is_some_and(|toggle| toggle.local_cell(x, y).is_some())
        {
            return Some(Target::Toggle);
        }
        let (_, line) = places.items?.local_cell(x, y)?;
        let rows = self.rows(props);
        let index = self.view_scroll(rows.len(), self.cursor_row(&rows)) + usize::from(line);
        (index < rows.len()).then_some(Target::Item(index))
    }

    /// The pointer hovers an item or the toggle; a press on an item does
    /// what Enter does there, and on the toggle flips the bar (NAV-009).
    fn handle_mouse(&mut self, mouse: &MouseEvent, props: &SidebarProps) -> EventResult {
        match mouse.kind {
            MouseEventKind::Move | MouseEventKind::Enter => {
                let rows = self.rows(props);
                let hovered = match self.target(mouse, props) {
                    Some(Target::Item(index)) => Some(Hover::Item(rows[index].item.id.clone())),
                    Some(Target::Toggle) => Some(Hover::Toggle),
                    None => None,
                };
                if hovered == self.hovered {
                    return EventResult::Ignored;
                }
                self.hovered = hovered;
                EventResult::Handled
            }
            MouseEventKind::Leave => {
                if self.hovered.take().is_some() {
                    EventResult::Handled
                } else {
                    EventResult::Ignored
                }
            }
            MouseEventKind::Down if mouse.button == MouseButton::Left => {
                match self.target(mouse, props) {
                    Some(Target::Toggle) => self.flip(),
                    Some(Target::Item(index)) => {
                        let rows = self.rows(props);
                        if rows[index].item.disabled {
                            return EventResult::Ignored;
                        }
                        self.activate(&rows[index]);
                        self.keep_in_view(props);
                        EventResult::Consumed
                    }
                    None => EventResult::Ignored,
                }
            }
            _ => EventResult::Ignored,
        }
    }

    /// The screen reader's requests (NAV-010): its focus on an item moves
    /// the cursor there; a click on an item activates it, as Enter does;
    /// a click on the toggle flips the bar.
    fn handle_request(&mut self, event: &CustomEvent, props: &SidebarProps) -> EventResult {
        if event.name == TOGGLE_EVENT {
            return self.flip();
        }
        if event.name != FOCUS_EVENT && event.name != ACTIVATE_EVENT {
            return EventResult::Ignored;
        }
        let Ok(id) = std::str::from_utf8(&event.data) else {
            return EventResult::Ignored;
        };
        let rows = self.rows(props);
        let Some(row) = rows
            .iter()
            .find(|row| row.item.id == id && !row.item.disabled)
        else {
            return EventResult::Ignored;
        };
        if event.name == ACTIVATE_EVENT {
            self.activate(row);
        } else {
            self.cursor = Some(row.item.id.clone());
        }
        self.keep_in_view(props);
        EventResult::Consumed
    }
}

impl Component for Sidebar {
    type Props = SidebarProps;
    type State = ();

    fn new(props: Self::Props) -> Self {
        Self::new(props)
    }

    fn update(&mut self, props: &Self::Props, _state: &mut Self::State) -> bool {
        // A change of the props' active item or collapsed state is the
        // application's choice and wins over the user's last one.
        if props.active != self.given_active {
            self.active = props.active.clone();
            self.given_active = props.active.clone();
        }
        if props.collapsed != self.given_collapsed {
            self.collapsed = props.collapsed;
            self.given_collapsed = props.collapsed;
        }
        true
    }

    fn layout(
        &mut self,
        bounds: LayoutInfo,
        _props: &mut Self::Props,
        _state: &mut Self::State,
    ) -> bool {
        let previous = self.viewport.replace(bounds);
        previous.map(|layout| layout.content_size()) != Some(bounds.content_size())
    }

    fn render(&self, props: &Self::Props, _state: &Self::State) -> Element {
        self.paint(props)
    }

    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        _state: &mut Self::State,
    ) -> EventResult {
        match event {
            Event::Key(key) => match self.focused_part() {
                TREE => self.tree_key(key, props),
                TOGGLE => self.toggle_key(key),
                _ => EventResult::Ignored,
            },
            Event::Mouse(mouse) => self.handle_mouse(mouse, props),
            Event::Custom(request) => self.handle_request(request, props),
            _ => EventResult::Ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::types::KeyCode;

    fn items() -> Vec<SidebarItem> {
        vec![
            SidebarItem::new("home", "Home"),
            SidebarItem::new("projects", "Projects")
                .child(SidebarItem::new("alpha", "Alpha"))
                .child(SidebarItem::new("beta", "Beta").disabled(true)),
            SidebarItem::new("archive", "Archive").disabled(true),
            SidebarItem::new("settings", "Settings"),
        ]
    }

    fn press(sidebar: &mut Sidebar, props: &mut SidebarProps, code: KeyCode) {
        sidebar.handle_event(&Event::Key(KeyEvent::new(code)), props, &mut ());
    }

    fn cursor(sidebar: &Sidebar, props: &SidebarProps) -> String {
        let rows = sidebar.rows(props);
        rows[sidebar.cursor_row(&rows).unwrap()].item.id.clone()
    }

    /// NAV-009: the cursor passes over disabled items, Right opens a group
    /// and then moves into it, Left moves back and then closes it, and
    /// Enter reports the item it makes active.
    #[test]
    #[serial_test::serial(keymap)]
    fn nav_009_the_tree_keys_move_open_close_and_activate() {
        let selected: Arc<Mutex<Vec<String>>> = Arc::default();
        let sink = selected.clone();
        let mut props = SidebarProps {
            items: items(),
            ..SidebarProps::default()
        };
        let mut sidebar =
            Sidebar::new(props.clone()).with_on_select(move |id| sink.lock().unwrap().push(id));
        press(&mut sidebar, &mut props, KeyCode::Down);
        assert_eq!(cursor(&sidebar, &props), "home", "keys need the focus");
        sidebar.focus.store(TREE, Ordering::Release);
        press(&mut sidebar, &mut props, KeyCode::Down);
        press(&mut sidebar, &mut props, KeyCode::Down);
        assert_eq!(
            cursor(&sidebar, &props),
            "settings",
            "past disabled Archive"
        );
        press(&mut sidebar, &mut props, KeyCode::Up);
        press(&mut sidebar, &mut props, KeyCode::Right);
        assert_eq!(cursor(&sidebar, &props), "projects");
        press(&mut sidebar, &mut props, KeyCode::Right);
        assert_eq!(cursor(&sidebar, &props), "alpha");
        press(&mut sidebar, &mut props, KeyCode::Down);
        assert_eq!(cursor(&sidebar, &props), "settings", "past disabled Beta");
        press(&mut sidebar, &mut props, KeyCode::Home);
        press(&mut sidebar, &mut props, KeyCode::Down);
        press(&mut sidebar, &mut props, KeyCode::Down);
        press(&mut sidebar, &mut props, KeyCode::Left);
        assert_eq!(cursor(&sidebar, &props), "projects");
        press(&mut sidebar, &mut props, KeyCode::Left);
        assert_eq!(sidebar.rows(&props).len(), 4, "Left closed Projects");
        press(&mut sidebar, &mut props, KeyCode::End);
        press(&mut sidebar, &mut props, KeyCode::Enter);
        assert_eq!(*selected.lock().unwrap(), vec!["settings".to_owned()]);
        assert_eq!(sidebar.active.as_deref(), Some("settings"));
    }

    /// The application's change of the active item or the collapsed state
    /// wins over the user's last one; props that keep them keep the user's.
    #[test]
    fn a_props_change_of_active_or_collapsed_wins() {
        let mut props = SidebarProps {
            items: items(),
            active: Some("home".to_owned()),
            ..SidebarProps::default()
        };
        let mut sidebar = Sidebar::new(props.clone());
        sidebar.active = Some("settings".to_owned());
        sidebar.collapsed = true;
        props.items.pop();
        sidebar.update(&props, &mut ());
        assert_eq!(sidebar.active.as_deref(), Some("settings"));
        assert!(sidebar.collapsed);
        props.active = Some("projects".to_owned());
        props.collapsed = true;
        sidebar.update(&props, &mut ());
        assert_eq!(sidebar.active.as_deref(), Some("projects"));
        props.collapsed = false;
        sidebar.update(&props, &mut ());
        assert!(!sidebar.collapsed);
    }
}
