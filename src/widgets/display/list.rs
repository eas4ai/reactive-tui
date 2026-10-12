//! The list (docs/spec/data-widgets.md, DAT-006 to DAT-009): a scrolling
//! list of items with optional section titles, a filter row, a loading row
//! and an empty message, with one focus stop.
//!
//! The list builds elements for the rows in view alone, whatever its number
//! of items (DAT-007): its lines are worked out once for its items and its
//! filter, and each frame paints the window of them its height shows.

use crate::component::{Component, Element, LayoutInfo, Props};
use crate::event::router::EventResult;
use crate::event::types::{
    FocusEventKind, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind,
    Position, WheelDelta,
};
use crate::event::Event;
use crate::reactive::scheduler::Scheduler;
use std::any::Any;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

mod lines;
mod paint;
#[cfg(test)]
mod tests;

use lines::{Line, Lines};

/// The event a list's option takes when the screen reader moves the focus
/// to it: the cursor moves there, as a click does.
const FOCUS_EVENT: &str = "reactive_tui.list.focus";

/// How close to the last item, in items, the cursor or the view's last row
/// asks the application for more (DAT-008).
const MORE_WITHIN: usize = 20;

/// The lines one notch of the wheel scrolls, as a scroll view's default.
const WHEEL_LINES: usize = 3;

/// The callback called with an item's id.
pub(crate) type IdCallback = Arc<dyn Fn(String) + Send + Sync>;

/// The callback called with no arguments.
pub(crate) type UnitCallback = Arc<dyn Fn() + Send + Sync>;

/// One item of a list.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ListItem {
    /// The id the item is known by to the application.
    pub id: String,
    /// The text the user reads.
    pub label: String,
    /// A text shown after the label, in the muted color, if any.
    pub description: Option<String>,
    /// The title of the section the item sits under, if any. Consecutive
    /// items with one section share one title row.
    pub section: Option<String>,
    /// Whether the item is skipped by the cursor.
    pub disabled: bool,
}

impl ListItem {
    /// Create an item with an id and a label.
    pub fn new(id: &str, label: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            ..Self::default()
        }
    }

    /// Set the description shown after the label.
    pub fn description(mut self, text: &str) -> Self {
        self.description = Some(text.to_string());
        self
    }

    /// Set the section the item sits under.
    pub fn section(mut self, title: &str) -> Self {
        self.section = Some(title.to_string());
        self
    }

    /// Set whether the item is disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Properties for the list.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ListProps {
    /// The items, in order.
    pub items: Vec<ListItem>,
    /// The id of the selected item, if any.
    pub selected: Option<String>,
    /// Whether a filter row is shown on top of the list.
    pub filterable: bool,
    /// Whether the application has more items to give the list.
    pub has_more: bool,
    /// Whether a loading row is shown at the end.
    pub loading: bool,
    /// The message shown when no item shows, if any.
    pub empty_text: Option<String>,
    /// The name the screen reader hears for the list.
    pub aria_label: Option<String>,
    /// The list's width in cells; none takes what the parent allots.
    pub width: Option<u16>,
    /// The list's height in cells; none takes what the parent allots.
    pub height: Option<u16>,
}

impl Props for ListProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// A list: items the user moves through, selects and confirms.
pub struct List {
    /// Called with the id of the item the cursor moved to.
    pub(crate) on_select: Option<IdCallback>,
    /// Called with the id of the item confirmed with Enter or a double click.
    pub(crate) on_confirm: Option<IdCallback>,
    /// Called when Escape cancels the list.
    pub(crate) on_cancel: Option<UnitCallback>,
    /// Called when the list asks for more items.
    pub(crate) on_load_more: Option<UnitCallback>,
    viewport: Option<LayoutInfo>,
    /// The lines the list shows for its items, its filter and its loading.
    lines: Lines,
    /// What the items were when the lines were made, to tell a change.
    items_print: u64,
    /// The filter's text.
    filter: String,
    /// The props' item the cursor stands on, and its id.
    cursor: Option<usize>,
    cursor_id: Option<String>,
    /// The id of the selected item.
    selected: Option<String>,
    /// The props' selected id last seen, to tell when the application
    /// selects another item.
    props_selected: Option<String>,
    /// The first line in view.
    offset: usize,
    /// The props' item under the pointer.
    hover: Option<usize>,
    focused: bool,
    /// Whether the list asked for more since its items last changed.
    asked: bool,
    /// Whether the view was brought to the cursor at the first layout.
    revealed: bool,
    /// The App's scheduler, which calls back after a render asked for more.
    scheduler: Option<Arc<Scheduler>>,
}

/// A fingerprint of the items: equal items give equal prints, and a change
/// almost surely another.
fn fingerprint(items: &[ListItem]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    items.len().hash(&mut hasher);
    for item in items {
        item.id.hash(&mut hasher);
        item.label.hash(&mut hasher);
        item.description.hash(&mut hasher);
        item.section.hash(&mut hasher);
        item.disabled.hash(&mut hasher);
    }
    hasher.finish()
}

impl List {
    /// Create a list from its props, with no callbacks yet.
    pub fn new(props: ListProps) -> Self {
        let mut list = Self {
            on_select: None,
            on_confirm: None,
            on_cancel: None,
            on_load_more: None,
            viewport: None,
            lines: Lines::new(&props.items, "", props.loading),
            items_print: fingerprint(&props.items),
            filter: String::new(),
            cursor: None,
            cursor_id: None,
            selected: props.selected.clone(),
            props_selected: props.selected.clone(),
            offset: 0,
            hover: None,
            focused: false,
            asked: false,
            revealed: false,
            scheduler: crate::reactive::component_scope::current().map(|scope| scope.scheduler()),
        };
        let item = list.initial_cursor(&props);
        list.set_cursor(&props, item);
        list
    }

    /// Set the callback called with the id of the item the cursor moved to.
    pub fn with_on_select(mut self, f: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_select = Some(Arc::new(f));
        self
    }

    /// Set the callback called with the id of a confirmed item.
    pub fn with_on_confirm(mut self, f: impl Fn(String) + Send + Sync + 'static) -> Self {
        self.on_confirm = Some(Arc::new(f));
        self
    }

    /// Set the callback called when Escape cancels the list.
    pub fn with_on_cancel(mut self, f: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_cancel = Some(Arc::new(f));
        self
    }

    /// Set the callback called when the list asks for more items.
    pub fn with_on_load_more(mut self, f: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_load_more = Some(Arc::new(f));
        self
    }

    /// The cursor's first place: the selected item when it shows and can
    /// hold the cursor, else the first item that can.
    fn initial_cursor(&self, props: &ListProps) -> Option<usize> {
        let selected = self.selected.as_ref().and_then(|id| {
            let item = props.items.iter().position(|item| &item.id == id)?;
            self.lines
                .line_of(item)
                .and_then(|line| self.lines.reachable(&props.items, line))
        });
        selected.or_else(|| {
            self.lines
                .reachable_from(&props.items, 0)
                .and_then(|line| self.lines.reachable(&props.items, line))
        })
    }

    /// The list's size: what its layout gives it, else what the props set.
    fn size(&self, props: &ListProps) -> (usize, usize) {
        match self.viewport {
            Some(layout) => {
                let (width, height) = layout.content_size();
                (width as usize, height as usize)
            }
            None => (
                props.width.map_or(0, usize::from),
                props.height.map_or(0, usize::from),
            ),
        }
    }

    /// The rows the filter takes on top: one while it shows.
    fn filter_rows(props: &ListProps) -> usize {
        usize::from(props.filterable)
    }

    /// The lines the list's height holds under its filter row.
    fn body(&self, props: &ListProps) -> usize {
        self.size(props).1.saturating_sub(Self::filter_rows(props))
    }

    /// The line the cursor stands on, when its item shows.
    fn cursor_line(&self) -> Option<usize> {
        self.cursor.and_then(|item| self.lines.line_of(item))
    }

    /// Keeps the view inside the lines.
    fn clamp(&mut self, props: &ListProps) {
        let last = self.lines.len().saturating_sub(self.body(props).max(1));
        self.offset = self.offset.min(last);
    }

    /// Scrolls the view so the cursor's item is in view, with its section's
    /// title when the view moves up to it.
    fn reveal(&mut self, props: &ListProps) {
        let body = self.body(props);
        if let Some(line) = self.cursor_line().filter(|_| body > 0) {
            if line < self.offset {
                self.offset = match self.lines.get(line.wrapping_sub(1)) {
                    Some(Line::Title { .. }) => line - 1,
                    _ => line,
                };
            } else if line >= self.offset + body {
                self.offset = line + 1 - body;
            }
        }
        self.clamp(props);
    }

    /// Moves the cursor to the item on `line`, when it can stand there,
    /// keeps it in view and reports the move through `on_select`.
    fn move_to(&mut self, props: &ListProps, line: Option<usize>) -> EventResult {
        let Some(item) = line.and_then(|line| self.lines.reachable(&props.items, line)) else {
            return EventResult::Consumed;
        };
        let moved = self.cursor != Some(item);
        self.set_cursor(props, Some(item));
        self.reveal(props);
        if moved {
            if let Some(callback) = &self.on_select {
                callback(props.items[item].id.clone());
            }
        }
        EventResult::Consumed
    }

    /// Selects the cursor's item and reports it through `on_confirm`.
    fn confirm(&mut self, props: &ListProps) -> EventResult {
        let Some(item) = self
            .cursor
            .filter(|item| self.lines.line_of(*item).is_some())
        else {
            return EventResult::Consumed;
        };
        let id = props.items[item].id.clone();
        self.selected = Some(id.clone());
        if let Some(callback) = &self.on_confirm {
            callback(id);
        }
        EventResult::Consumed
    }

    /// Makes the lines again for the filter's text, with the cursor on the
    /// first item shown and the view at the top (DAT-008).
    fn refilter(&mut self, props: &ListProps) {
        self.lines = Lines::new(&props.items, &self.filter, props.loading);
        self.offset = 0;
        self.hover = None;
        let first = self.lines.reachable_from(&props.items, 0);
        match first {
            Some(_) => {
                self.move_to(props, first);
            }
            None => self.set_cursor(props, None),
        }
    }

    /// Whether the list should ask for more now: the application has more,
    /// the list has not asked since its items changed, and the cursor or
    /// the view's last row is within twenty items of the last item.
    fn wants_more(&self, props: &ListProps) -> bool {
        if !props.has_more || self.asked || self.on_load_more.is_none() {
            return false;
        }
        let Some(last) = self.lines.shown.checked_sub(1) else {
            return self.viewport.is_some();
        };
        let near = |place: usize| last.saturating_sub(place) <= MORE_WITHIN;
        let cursor = self
            .cursor_line()
            .is_some_and(|line| near(self.lines.place_at(line)));
        let body = self.body(props);
        let view = body > 0 && near(self.lines.place_at(self.offset + body - 1).min(last));
        cursor || view
    }

    /// Asks for more when the list should: at once from an event, else on
    /// the App's next turn, outside the frame being made.
    fn ask_more(&mut self, props: &ListProps, at_once: bool) {
        if !self.wants_more(props) {
            return;
        }
        self.asked = true;
        let Some(callback) = self.on_load_more.clone() else {
            return;
        };
        match &self.scheduler {
            Some(scheduler) if !at_once => scheduler.schedule_update(Box::new(move || callback())),
            _ => callback(),
        }
    }

    /// The line of the body under a pointer position, if any.
    fn line_at(&self, props: &ListProps, position: Position) -> Option<usize> {
        let Position::Cell { x, y } = position else {
            return None;
        };
        let (width, height) = self.size(props);
        let (x, y) = (usize::from(x), usize::from(y));
        let row = y.checked_sub(Self::filter_rows(props))?;
        (x < width && y < height).then_some(self.offset + row)
    }

    fn handle_key(&mut self, key: &KeyEvent, props: &ListProps) -> EventResult {
        use crate::keymap::Action;
        // A character typed while the list has a filter is text first,
        // whatever the keymap binds it to (KEY-001: typing wins).
        if props.filterable && !key.modifiers.ctrl && !key.modifiers.alt && !key.modifiers.meta {
            let typed = match key.code {
                KeyCode::Char(c) if !c.is_control() => Some(c),
                KeyCode::Space => Some(' '),
                _ => None,
            };
            if let Some(c) = typed {
                self.filter.push(c);
                self.refilter(props);
                return EventResult::Consumed;
            }
            if key.code == KeyCode::Backspace {
                if self.filter.pop().is_some() {
                    self.refilter(props);
                }
                return EventResult::Consumed;
            }
        }
        // Every other key means what the active keymap says.
        let Some(action) = crate::keymap::Keymap::active().action(key) else {
            return EventResult::Ignored;
        };
        let items = &props.items;
        let page = self.body(props).max(1);
        let cursor = self.cursor_line();
        let last = self.lines.len().saturating_sub(1);
        let target = match action {
            Action::Down => cursor.map_or_else(
                || self.lines.reachable_from(items, 0),
                |line| self.lines.next(items, line),
            ),
            Action::Up => cursor.map_or_else(
                || self.lines.reachable_from(items, 0),
                |line| self.lines.previous(items, line),
            ),
            Action::PageDown => self
                .lines
                .reachable_from(items, cursor.map_or(0, |line| line + page)),
            Action::PageUp => self
                .lines
                .reachable_until(items, cursor.map_or(0, |line| line.saturating_sub(page))),
            Action::Home => self.lines.reachable_from(items, 0),
            Action::End => self.lines.reachable_until(items, last),
            Action::Confirm => return self.confirm(props),
            Action::Cancel => {
                if props.filterable && !self.filter.is_empty() {
                    self.filter.clear();
                    self.refilter(props);
                } else if let Some(callback) = &self.on_cancel {
                    callback();
                }
                return EventResult::Consumed;
            }
            _ => return EventResult::Ignored,
        };
        self.move_to(props, target)
    }

    fn handle_mouse(&mut self, mouse: &MouseEvent, props: &ListProps) -> EventResult {
        match mouse.kind {
            MouseEventKind::Wheel => {
                let lines = match mouse.wheel.as_ref().map(|wheel| &wheel.delta) {
                    Some(WheelDelta::Lines { y, .. } | WheelDelta::Pixels { y, .. }) => *y as f64,
                    None => return EventResult::Ignored,
                };
                let before = self.offset;
                self.offset = shifted(self.offset, lines, WHEEL_LINES);
                self.clamp(props);
                // At an edge the wheel passes to an enclosing view (INP-005).
                if self.offset == before {
                    return EventResult::Ignored;
                }
                EventResult::Consumed
            }
            MouseEventKind::Move | MouseEventKind::Enter => {
                let hover = self
                    .line_at(props, mouse.position)
                    .and_then(|line| self.lines.reachable(&props.items, line));
                if hover == self.hover {
                    return EventResult::Ignored;
                }
                self.hover = hover;
                EventResult::Consumed
            }
            MouseEventKind::Leave => {
                if self.hover.take().is_none() {
                    return EventResult::Ignored;
                }
                EventResult::Consumed
            }
            // A press moves the cursor to the item under it; the router
            // gives the list the focus (DAT-008).
            MouseEventKind::Down if mouse.button == MouseButton::Left => {
                match self
                    .line_at(props, mouse.position)
                    .filter(|line| self.lines.reachable(&props.items, *line).is_some())
                {
                    Some(line) => self.move_to(props, Some(line)),
                    None => EventResult::Ignored,
                }
            }
            // A double click confirms the item, as Enter does (DAT-009).
            MouseEventKind::DoubleClick if mouse.button == MouseButton::Left => {
                match self
                    .line_at(props, mouse.position)
                    .and_then(|line| self.lines.reachable(&props.items, line))
                {
                    Some(item) if self.cursor == Some(item) => self.confirm(props),
                    _ => EventResult::Ignored,
                }
            }
            _ => EventResult::Ignored,
        }
    }

    /// Puts the cursor on the props' item `item`, keeping its id, so the
    /// cursor finds its item again when the items change.
    fn set_cursor(&mut self, props: &ListProps, item: Option<usize>) {
        self.cursor = item;
        self.cursor_id = item
            .and_then(|item| props.items.get(item))
            .map(|item| item.id.clone());
    }

    /// Whether the props' item `item` shows and can hold the cursor.
    fn holds_cursor(&self, props: &ListProps, item: usize) -> bool {
        self.lines
            .line_of(item)
            .and_then(|line| self.lines.reachable(&props.items, line))
            .is_some()
    }

    /// Takes the props of a render: the lines are made again; new items
    /// keep the cursor on its item where it still shows and let the list
    /// ask for more again; a newly selected item takes the cursor.
    fn take_props(&mut self, props: &ListProps) {
        let print = fingerprint(&props.items);
        if !props.filterable {
            self.filter.clear();
        }
        self.lines = Lines::new(&props.items, &self.filter, props.loading);
        if print != self.items_print {
            self.items_print = print;
            self.asked = false;
            self.hover = None;
            let item = self
                .cursor_id
                .as_ref()
                .and_then(|id| props.items.iter().position(|item| &item.id == id));
            self.set_cursor(props, item);
        }
        if props.selected != self.props_selected {
            self.props_selected = props.selected.clone();
            self.selected = props.selected.clone();
            self.set_cursor(props, None);
            self.revealed = false;
        }
        if !self
            .cursor
            .is_some_and(|item| self.holds_cursor(props, item))
        {
            let item = self.initial_cursor(props);
            self.set_cursor(props, item);
        }
        if !self.revealed && self.viewport.is_some() {
            self.revealed = true;
            self.reveal(props);
        }
        self.clamp(props);
    }
}

/// `value` moved by `delta` notches of `speed` lines each, as a scroll
/// view's wheel moves it.
fn shifted(value: usize, delta: f64, speed: usize) -> usize {
    if !delta.is_finite() {
        return value;
    }
    let amount = (delta.abs() * speed as f64).round() as usize;
    if delta < 0.0 {
        value.saturating_sub(amount)
    } else {
        value.saturating_add(amount)
    }
}

impl Component for List {
    type Props = ListProps;
    type State = ();

    fn new(props: Self::Props) -> Self {
        Self::new(props)
    }

    fn update(&mut self, props: &Self::Props, _state: &mut Self::State) -> bool {
        self.take_props(props);
        self.ask_more(props, false);
        true
    }

    fn layout(
        &mut self,
        layout: LayoutInfo,
        props: &mut Self::Props,
        _state: &mut Self::State,
    ) -> bool {
        let before = (self.viewport, self.offset);
        self.viewport = Some(layout);
        if !self.revealed {
            self.revealed = true;
            self.reveal(props);
        }
        self.clamp(props);
        self.ask_more(props, false);
        before != (self.viewport, self.offset)
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
        let result = match event {
            Event::Focus(focus) => {
                match focus.kind {
                    FocusEventKind::Gained => self.focused = true,
                    FocusEventKind::Lost => self.focused = false,
                    _ => return EventResult::Ignored,
                }
                EventResult::Consumed
            }
            Event::Key(key) if self.focused && key.kind != KeyEventKind::Release => {
                self.handle_key(key, props)
            }
            Event::Mouse(mouse) => self.handle_mouse(mouse, props),
            // The screen reader moved the focus to an option.
            Event::Custom(custom) if custom.name == FOCUS_EVENT => {
                let line = std::str::from_utf8(&custom.data)
                    .ok()
                    .and_then(|item| item.parse::<usize>().ok())
                    .and_then(|item| self.lines.line_of(item));
                match line {
                    Some(line) => self.move_to(props, Some(line)),
                    None => EventResult::Ignored,
                }
            }
            _ => EventResult::Ignored,
        };
        self.ask_more(props, true);
        result
    }
}
