//! The log view (docs/spec/layout-widgets.md, NAV-007 to NAV-010): the rows
//! of a [`LogHandle`] in a view that follows the newest row while rows
//! arrive, with a `↓ N new` bar once the user has scrolled away.
//!
//! The rows live in the handle, which any thread may change; each change
//! wakes the App whose latest frame showed a view of the handle, so the
//! view redraws without the application asking for it. The view scrolls by
//! line, as a scroll view does, with a scroll view's bar and keys
//! (NAV-002 to NAV-004), and keeps the rows in view on their lines while
//! older rows are prepended or newer rows arrive under it.

use super::look;
use super::scroll_view::{scrollbar, shifted, thumb_geometry};
use crate::accessibility::{Live, Node, Role};
use crate::component::{
    Component, Element, ElementType, FocusProps, LayoutInfo, LayoutType, Props,
};
use crate::event::router::EventResult;
use crate::event::types::{
    FocusEventKind, KeyEventKind, MouseButton, MouseEvent, MouseEventKind, Position, WheelDelta,
};
use crate::event::{CustomEvent, Event};
use crate::layout::style::{Direction, StyleBuilder};
use crate::widgets::display::pieces::icon::Icon;
use std::any::Any;
use std::sync::{Mutex, MutexGuard};

mod handle;
mod wrap;

pub use handle::LogHandle;
use handle::Rows;

/// The event the `↓ N new` bar takes for the screen reader's Click: it
/// resumes the following, as a click on the bar does.
const FOLLOW_EVENT: &str = "reactive_tui.log_view.follow";

/// The lines one notch of the wheel scrolls, as a scroll view's default.
const WHEEL_LINES: usize = 3;

/// The word the `↓ N new` bar reads when the application gives none.
const NEW_WORD: &str = "new";

/// How a log row is colored.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LogKind {
    /// Plain text in `foreground`.
    #[default]
    Plain,
    /// Information, in the info role's color.
    Info,
    /// A success, in the success role's color.
    Success,
    /// A warning, in the warning role's color.
    Warning,
    /// An error, in the error role's color.
    Error,
}

impl LogKind {
    /// The classes of a row of this kind (NAV-007).
    fn class(self) -> &'static str {
        match self {
            LogKind::Plain => look::TEXT,
            LogKind::Info => look::LOG_INFO,
            LogKind::Success => look::LOG_SUCCESS,
            LogKind::Warning => look::LOG_WARNING,
            LogKind::Error => look::LOG_ERROR,
        }
    }
}

/// One row of a log view: its text and how it is colored.
#[derive(Clone, Debug, PartialEq)]
pub struct LogRow {
    /// The text of the row.
    pub text: String,
    /// How the row is colored.
    pub kind: LogKind,
}

impl LogRow {
    /// Create a plain row with the given text.
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            kind: LogKind::Plain,
        }
    }

    /// Set how the row is colored.
    pub fn kind(mut self, kind: LogKind) -> Self {
        self.kind = kind;
        self
    }
}

/// Properties for the log view.
#[derive(Clone, PartialEq)]
pub struct LogViewProps {
    /// The rows the view shows.
    pub handle: LogHandle,
    /// The word in the `↓ N new` bar; none is `new`.
    pub new_word: Option<String>,
    /// The name the screen reader hears for the view.
    pub aria_label: Option<String>,
    /// The view's width in cells; none takes what the parent allots.
    pub width: Option<u16>,
    /// The view's height in cells; none takes what the parent allots.
    pub height: Option<u16>,
}

impl Props for LogViewProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Where the view stands among its rows.
#[derive(Clone, Copy, Debug, Default)]
struct Place {
    /// Whether the view follows its last row.
    following: bool,
    /// The rows' clear count when the view last stood still.
    epoch: u64,
    /// The row on the view's first line and the line within that row,
    /// while the view does not follow.
    top: (i64, usize),
    /// The id of the last row when the view stopped following: the rows
    /// after it arrived since.
    seen: i64,
}

/// A drag of the scroll bar's thumb, as a scroll view takes one.
struct ThumbDrag {
    /// The row of the pointer when the drag began.
    pointer: f32,
    /// The view's first line when the drag began.
    offset: usize,
    /// The cells the thumb can travel.
    travel: usize,
    /// The greatest first line.
    limit: usize,
}

/// What a view shows of its rows at its size.
struct Geometry {
    /// The view's height in cells.
    height: usize,
    /// The cells a line of text takes: the width less the scroll bar's.
    text_width: usize,
    /// Whether the scroll bar shows: the lines do not fit the height.
    bar: bool,
    /// The lines every row takes at the text width.
    total: usize,
    /// The view's first line, and the greatest it can be.
    top: usize,
    max_top: usize,
    /// Whether the view follows its last row.
    following: bool,
    /// The rows that arrived since the view stopped following.
    fresh: usize,
}

impl Geometry {
    /// Whether the last line is the `↓ N new` bar.
    fn new_bar(&self) -> bool {
        !self.following && self.fresh > 0 && self.height > 0
    }
}

/// A log view: the rows of a handle, following the newest one.
pub struct LogView {
    viewport: Option<LayoutInfo>,
    /// The handle the place was taken in.
    handle: LogHandle,
    /// The lines of the rows at the text width, kept between renders.
    wrapped: Mutex<wrap::Wrapped>,
    place: Place,
    focused: bool,
    drag: Option<ThumbDrag>,
}

impl LogView {
    /// Create a log view from its props.
    pub fn new(props: LogViewProps) -> Self {
        Self {
            viewport: None,
            handle: props.handle,
            wrapped: Mutex::default(),
            place: Place {
                following: true,
                ..Place::default()
            },
            focused: false,
            drag: None,
        }
    }

    /// The view's size: what its layout gives it, else what the props set.
    fn size(&self, props: &LogViewProps) -> (usize, usize) {
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

    fn wrapped(&self) -> MutexGuard<'_, wrap::Wrapped> {
        self.wrapped
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Whether the view follows its last row: it says so, or the row it
    /// stood on was cleared away.
    fn follows(&self, rows: &Rows) -> bool {
        self.place.following || self.place.epoch != rows.epoch || self.place.top.0 < rows.first
    }

    /// What the view shows of `rows` at its size. The scroll bar takes the
    /// last column only while the lines do not fit the height (NAV-002), and
    /// the rows wrap at the width the bar leaves.
    fn measure(&self, rows: &Rows, props: &LogViewProps) -> Geometry {
        let (width, height) = self.size(props);
        let mut wrapped = self.wrapped();
        let narrow = width.saturating_sub(1);
        let bar = width > 1 && height > 0 && {
            wrapped.refresh(rows, narrow);
            // The lines at the full width are no fewer than the rows.
            wrapped.total() > height
                && (rows.rows.len() > height
                    || rows
                        .rows
                        .iter()
                        .map(|row| wrap::count(&row.text, width))
                        .sum::<usize>()
                        > height)
        };
        let text_width = if bar { narrow } else { width };
        wrapped.refresh(rows, text_width);
        let total = wrapped.total();
        let max_top = total.saturating_sub(height);
        let following = self.follows(rows);
        let top = if following {
            max_top
        } else {
            let index = (self.place.top.0 - rows.first) as usize;
            if index >= rows.rows.len() {
                max_top
            } else {
                let within = self
                    .place
                    .top
                    .1
                    .min(wrapped.lines_of(index).saturating_sub(1));
                (wrapped.start(index) + within).min(max_top)
            }
        };
        let fresh = if following {
            0
        } else {
            (rows.last() - self.place.seen).clamp(0, rows.rows.len() as i64) as usize
        };
        Geometry {
            height,
            text_width,
            bar,
            total,
            top,
            max_top,
            following,
            fresh,
        }
    }

    /// Follows the last row again.
    fn follow(&mut self, rows: &Rows) {
        self.place.following = true;
        self.place.epoch = rows.epoch;
    }

    /// Scrolls the view so its first line is `line`: to the last line it
    /// follows the last row again; above it, the view stands on the row of
    /// that line and stops following (NAV-009).
    fn scroll_to(&mut self, rows: &Rows, geometry: &Geometry, line: usize) {
        if line >= geometry.max_top {
            self.follow(rows);
            return;
        }
        if self.follows(rows) {
            self.place.seen = rows.last();
        }
        let located = self.wrapped().locate(line);
        if let Some((index, within)) = located {
            self.place.following = false;
            self.place.epoch = rows.epoch;
            self.place.top = (rows.first + index as i64, within);
        }
    }

    /// Reads the rows and what the view shows of them, then acts on them.
    fn act(
        &mut self,
        props: &LogViewProps,
        act: impl FnOnce(&mut Self, &Rows, &Geometry) -> EventResult,
    ) -> EventResult {
        let handle = props.handle.clone();
        handle.read(|rows| {
            let geometry = self.measure(rows, props);
            act(self, rows, &geometry)
        })
    }

    fn handle_key(
        &mut self,
        key: &crate::event::types::KeyEvent,
        props: &LogViewProps,
    ) -> EventResult {
        use crate::keymap::Action;
        // The keys mean what the active keymap says (KEY-001).
        let Some(action) = crate::keymap::Keymap::active().action(key) else {
            return EventResult::Ignored;
        };
        self.act(props, |view, rows, geometry| {
            let page = geometry.height.max(1);
            let top = geometry.top;
            match action {
                Action::Up => view.scroll_to(rows, geometry, top.saturating_sub(1)),
                Action::Down => view.scroll_to(rows, geometry, top.saturating_add(1)),
                Action::PageUp => view.scroll_to(rows, geometry, top.saturating_sub(page)),
                Action::PageDown => view.scroll_to(rows, geometry, top.saturating_add(page)),
                Action::Home => view.scroll_to(rows, geometry, 0),
                Action::End => view.follow(rows),
                _ => return EventResult::Ignored,
            }
            EventResult::Consumed
        })
    }

    fn handle_wheel(&mut self, mouse: &MouseEvent, props: &LogViewProps) -> EventResult {
        let lines = match mouse.wheel.as_ref().map(|wheel| &wheel.delta) {
            Some(WheelDelta::Lines { y, .. } | WheelDelta::Pixels { y, .. }) => *y as f64,
            None => return EventResult::Ignored,
        };
        self.act(props, |view, rows, geometry| {
            let line = shifted(geometry.top, lines, WHEEL_LINES).min(geometry.max_top);
            // At an edge the wheel passes to an enclosing view (INP-005).
            if line == geometry.top {
                return EventResult::Ignored;
            }
            view.scroll_to(rows, geometry, line);
            EventResult::Consumed
        })
    }

    /// A press on the `↓ N new` bar resumes the following; one on the
    /// scroll bar's thumb starts a drag, one on its track scrolls a page
    /// toward the press (NAV-003, NAV-009).
    fn handle_pointer(&mut self, mouse: &MouseEvent, props: &LogViewProps) -> EventResult {
        let Position::Cell { x, y } = mouse.position else {
            return EventResult::Ignored;
        };
        let (x, y) = (usize::from(x), usize::from(y));
        match mouse.kind {
            MouseEventKind::Up => match self.drag.take() {
                Some(_) => EventResult::Consumed,
                None => EventResult::Ignored,
            },
            MouseEventKind::Drag => {
                let Some(drag) = &self.drag else {
                    return EventResult::Ignored;
                };
                if drag.travel == 0 {
                    return EventResult::Consumed;
                }
                let line = (drag.offset as f64
                    + (y as f64 - f64::from(drag.pointer)) * drag.limit as f64 / drag.travel as f64)
                    .round()
                    .clamp(0.0, drag.limit as f64) as usize;
                self.act(props, |view, rows, geometry| {
                    view.scroll_to(rows, geometry, line);
                    EventResult::Consumed
                })
            }
            MouseEventKind::Down if mouse.button == MouseButton::Left => {
                self.drag = None;
                self.act(props, |view, rows, geometry| {
                    if geometry.new_bar() && y + 1 == geometry.height && x < geometry.text_width {
                        view.follow(rows);
                        return EventResult::Consumed;
                    }
                    if !geometry.bar || x != geometry.text_width || y >= geometry.height {
                        return EventResult::Ignored;
                    }
                    let (start, thumb) =
                        thumb_geometry(geometry.height, geometry.total, geometry.top);
                    if y >= start && y < start + thumb {
                        view.drag = Some(ThumbDrag {
                            pointer: y as f32,
                            offset: geometry.top,
                            travel: geometry.height.saturating_sub(thumb),
                            limit: geometry.max_top,
                        });
                    } else if y < start {
                        view.scroll_to(
                            rows,
                            geometry,
                            geometry.top.saturating_sub(geometry.height),
                        );
                    } else {
                        view.scroll_to(
                            rows,
                            geometry,
                            geometry.top.saturating_add(geometry.height),
                        );
                    }
                    EventResult::Consumed
                })
            }
            _ => EventResult::Ignored,
        }
    }

    /// The lines in view, from `geometry.top`: each row in view as one
    /// element holding its lines, so the screen reader hears a row once,
    /// whole, however many lines it takes.
    fn rows_in_view(&self, rows: &Rows, geometry: &Geometry, lines: usize) -> Vec<Element> {
        let mut children = Vec::new();
        if lines == 0 {
            return children;
        }
        let Some((first, mut within)) = self.wrapped().locate(geometry.top) else {
            return children;
        };
        let mut left = lines;
        for (index, row) in rows.rows.iter().enumerate().skip(first) {
            if left == 0 {
                break;
            }
            let shown: Vec<Element> = wrap::lines(&row.text, geometry.text_width)
                .into_iter()
                .skip(within)
                .take(left)
                .map(|line| {
                    let mut hidden = Node::new(Role::GenericContainer);
                    hidden.set_hidden();
                    Element::text(line)
                        .class("whitespace-pre h-1 shrink-0")
                        .with_accessibility(hidden)
                })
                .collect();
            within = 0;
            left -= shown.len();
            // The row's text is its name and value, as a text's label is:
            // a row appended in view is a node added to a polite log, and
            // the screen reader hears it once.
            let mut node = Node::new(Role::Label);
            node.set_label(row.text.clone());
            node.set_value(row.text.clone());
            children.push(
                Element::layout(LayoutType::Flex)
                    .class(format!("flex flex-col shrink-0 {}", row.kind.class()))
                    .children(shown)
                    .with_key(format!("row:{}:{}", rows.epoch, rows.first + index as i64))
                    .with_accessibility(node),
            );
        }
        children
    }

    /// The `↓ N new` bar, centered on the last line: a button named by its
    /// text, whose Click resumes the following (NAV-010).
    fn new_bar(geometry: &Geometry, word: &str) -> Element {
        let name = format!("{} {} {word}", Icon::ArrowDown.glyph(), geometry.fresh);
        let text = format!(" {name} ");
        let width = unicode_width::UnicodeWidthStr::width(text.as_str());
        let mut node = Node::new(Role::Button);
        node.set_label(name);
        node.set_clickable();
        let mut bar = crate::builder::ElementBuilder::new(ElementType::Text(text))
            .styles(
                StyleBuilder::new()
                    .position_absolute()
                    .inset_left((geometry.text_width.saturating_sub(width) / 2) as f32)
                    .inset_top((geometry.height - 1) as f32)
                    .width_px(width as f32)
                    .height_px(1.0),
            )
            .class(&format!("whitespace-pre {}", look::NEW_BAR))
            .build()
            .with_key("new-bar")
            .with_accessibility(node);
        bar.metadata
            .accessibility_options
            .get_or_insert_default()
            .click_event = Some(CustomEvent::new(FOLLOW_EVENT, Vec::new()));
        bar
    }
}

impl Component for LogView {
    type Props = LogViewProps;
    type State = ();

    fn new(props: Self::Props) -> Self {
        Self::new(props)
    }

    fn update(&mut self, props: &Self::Props, _state: &mut Self::State) -> bool {
        // Another handle's rows: the view starts over, following them.
        if self.handle != props.handle {
            *self = Self {
                viewport: self.viewport,
                focused: self.focused,
                ..Self::new(props.clone())
            };
        }
        true
    }

    fn layout(
        &mut self,
        layout: LayoutInfo,
        props: &mut Self::Props,
        _state: &mut Self::State,
    ) -> bool {
        let before = self.size(props);
        let first = self.viewport.is_none();
        self.viewport = Some(layout);
        first || before != self.size(props)
    }

    fn render(&self, props: &Self::Props, _state: &Self::State) -> Element {
        let mut children = Vec::new();
        let geometry = props.handle.read(|rows| {
            let geometry = self.measure(rows, props);
            let lines = geometry.height - usize::from(geometry.new_bar());
            let shown = self.rows_in_view(rows, &geometry, lines);
            children.push(
                crate::builder::ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
                    .styles(
                        StyleBuilder::new()
                            .display_flex()
                            .direction(Direction::Column)
                            .position_absolute()
                            .inset_left(0.0)
                            .inset_top(0.0)
                            .width_px(geometry.text_width as f32)
                            .height_px(lines as f32)
                            .overflow_hidden(),
                    )
                    .children(shown)
                    .build()
                    .with_key("lines"),
            );
            geometry
        });
        if geometry.new_bar() {
            children.push(Self::new_bar(
                &geometry,
                props.new_word.as_deref().unwrap_or(NEW_WORD),
            ));
        }
        if geometry.bar {
            // The bar is drawing, not text: the screen reader is told the
            // offsets instead (NAV-010).
            let mut hidden = Node::new(Role::GenericContainer);
            hidden.set_hidden();
            children.push(
                scrollbar(
                    geometry.height,
                    geometry.total,
                    geometry.top,
                    false,
                    (geometry.text_width as f32, 0.0),
                )
                .with_key("scroll-bar")
                .with_accessibility(hidden),
            );
        }
        // The view fills what its parent allots unless the props set a size
        // (NAV-008); a size class on the element wins over both.
        let mut style = StyleBuilder::new()
            .display_flex()
            .direction(Direction::Column)
            .max_width_percent(100.0)
            .max_height_percent(100.0)
            .overflow_hidden();
        style = match props.width {
            Some(width) => style.width_px(f32::from(width)),
            None => style.width_percent(100.0).min_width_px(0.0),
        };
        style = match props.height {
            Some(height) => style.height_px(f32::from(height)),
            None => style.height_percent(100.0).min_height_px(0.0),
        };
        // A polite log with a scroll view's offsets (NAV-010).
        let mut node = Node::new(Role::Log);
        if let Some(label) = &props.aria_label {
            node.set_label(label.clone());
        }
        node.set_live(Live::Polite);
        node.inner.set_scroll_x(0.0);
        node.inner.set_scroll_x_min(0.0);
        node.inner.set_scroll_x_max(0.0);
        node.inner.set_scroll_y(geometry.top as f64);
        node.inner.set_scroll_y_min(0.0);
        node.inner.set_scroll_y_max(geometry.max_top as f64);
        crate::builder::ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(style)
            .children(children)
            .build()
            .with_accessibility(node)
            .with_focus(FocusProps::input())
    }

    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        _state: &mut Self::State,
    ) -> EventResult {
        match event {
            Event::Focus(focus) => {
                match focus.kind {
                    FocusEventKind::Gained => self.focused = true,
                    FocusEventKind::Lost => {
                        self.focused = false;
                        self.drag = None;
                    }
                    _ => return EventResult::Ignored,
                }
                EventResult::Consumed
            }
            Event::Key(key) if self.focused && key.kind != KeyEventKind::Release => {
                self.handle_key(key, props)
            }
            Event::Mouse(mouse) if mouse.kind == MouseEventKind::Wheel => {
                self.handle_wheel(mouse, props)
            }
            Event::Mouse(mouse) => self.handle_pointer(mouse, props),
            Event::Custom(custom) if custom.name == FOLLOW_EVENT => {
                self.act(props, |view, rows, _| {
                    view.follow(rows);
                    EventResult::Consumed
                })
            }
            _ => EventResult::Ignored,
        }
    }

    fn on_lifecycle(&mut self, event: crate::component::LifecycleEvent, _state: &mut Self::State) {
        if event == crate::component::LifecycleEvent::Unmount {
            self.drag = None;
        }
    }
}

#[cfg(test)]
mod tests;
