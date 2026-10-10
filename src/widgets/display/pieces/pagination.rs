//! The pagination display piece (docs/spec/display-pieces.md), carrying DIS-001, DIS-002, DIS-003, DIS-004, DIS-005, DIS-006.
//!
//! A pagination bar shows the first page, the last page and the pages around
//! the current one, with an ellipsis for each run of hidden pages. Its arrows
//! step one page; a click or Confirm chooses a page and reports it. An
//! ellipsis opens a menu of the pages it hides.

use std::{any::Any, sync::Arc};

use crate::accessibility::{AriaCurrent, Node, Role};
use crate::builder::core::{div, span};
use crate::component::{same_callback, Component, Element, FocusProps, Props};
use crate::event::{
    router::EventResult,
    types::{Event, FocusEventKind},
};
use crate::keymap::{Action, Keymap};
use crate::reactive::ThreadSafeSignal;
use crate::widgets::display::pieces::icon::Icon;
use crate::widgets::menu::{MenuItem, PopupMenu, RelativePlacement};

/// The number of shown pages when the application sets none.
pub const DEFAULT_VISIBLE_PAGES: usize = 5;

/// One entry of the bar: a page, or the run of pages an ellipsis hides.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Entry {
    /// A page, 1-based.
    Page(usize),
    /// An ellipsis for the pages it hides. `trailing` is true when the run
    /// comes after the current page, false when it comes before.
    Hidden { trailing: bool, pages: Vec<usize> },
}

/// Where the keyboard stands on the bar: on the current page, or on the
/// ellipsis before or after it (DIS-003).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Slot {
    #[default]
    Page,
    Leading,
    Trailing,
}

/// The entries of a bar of `pages` pages with `current` (1-based) shown,
/// at most `visible` numbers in all, the first and last page included
/// (DIS-003). The window around the current page keeps its middle pages
/// inside the bar, so the ends never shift.
pub(crate) fn entries(pages: usize, current: usize, visible: usize) -> Vec<Entry> {
    let pages = pages.max(1);
    let current = current.clamp(1, pages);
    let visible = visible.max(3);
    if pages <= visible {
        return (1..=pages).map(Entry::Page).collect();
    }
    let middle = visible - 2;
    let half = (middle - 1) / 2;
    let start = current.saturating_sub(half).clamp(2, pages - middle);
    let end = start + middle - 1;
    let mut out = vec![Entry::Page(1)];
    if start > 2 {
        out.push(Entry::Hidden {
            trailing: false,
            pages: (2..start).collect(),
        });
    }
    out.extend((start..=end).map(Entry::Page));
    if end < pages - 1 {
        out.push(Entry::Hidden {
            trailing: true,
            pages: (end + 1..pages).collect(),
        });
    }
    out.push(Entry::Page(pages));
    out
}

/// The settings of a pagination bar. Callbacks are left out of equality, so a
/// rebuild that changes only the callback keeps the mounted bar (CMP-008).
#[derive(Clone)]
pub struct PaginationProps {
    /// How many pages there are. At least one.
    pub pages: usize,
    /// The current page, 1-based.
    pub current: usize,
    /// How many numbers the bar shows at most, first and last included.
    pub visible_pages: usize,
    /// Show only the arrows and `current / pages` instead of the page numbers.
    pub compact: bool,
    /// The label a screen reader speaks for the bar's landmark.
    pub aria_label: Option<String>,
    /// Classes added to the bar's element, after its own.
    pub class: String,
    /// Called with the page when a click or Confirm chooses it (CMP-009).
    pub on_change: Option<Arc<dyn Fn(usize) + Send + Sync>>,
}

impl Default for PaginationProps {
    fn default() -> Self {
        Self {
            pages: 1,
            current: 1,
            visible_pages: DEFAULT_VISIBLE_PAGES,
            compact: false,
            aria_label: None,
            class: String::new(),
            on_change: None,
        }
    }
}

impl PartialEq for PaginationProps {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            pages,
            current,
            visible_pages,
            compact,
            aria_label,
            class,
            on_change: _,
        } = self;
        *pages == other.pages
            && *current == other.current
            && *visible_pages == other.visible_pages
            && *compact == other.compact
            && *aria_label == other.aria_label
            && *class == other.class
    }
}

impl Props for PaginationProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Default)]
pub(crate) struct PaginationState {
    focused: bool,
    slot: Slot,
}

/// A pagination bar. The current page lives in a shared signal, so a click
/// on a page, which runs outside the render, can move it too.
pub(crate) struct Pagination {
    page: ThreadSafeSignal<usize>,
    /// Which ellipsis menu is open: `Some(true)` for the trailing run.
    menu: ThreadSafeSignal<Option<bool>>,
    /// The current page the props last carried, so a new current page wins.
    synced: usize,
}

impl Pagination {
    fn clamp_page(props: &PaginationProps, page: usize) -> usize {
        page.clamp(1, props.pages.max(1))
    }

    /// Report `page` to the application.
    fn report(props: &PaginationProps, page: usize) {
        if let Some(callback) = &props.on_change {
            callback(page);
        }
    }
}

impl Component for Pagination {
    type Props = PaginationProps;
    type State = PaginationState;

    fn new(props: Self::Props) -> Self {
        Self {
            page: ThreadSafeSignal::new(Self::clamp_page(&props, props.current)),
            menu: ThreadSafeSignal::new(None),
            synced: props.current,
        }
    }

    fn update(&mut self, props: &Self::Props, _state: &mut Self::State) -> bool {
        if props.current != self.synced {
            self.synced = props.current;
            self.page.set(Self::clamp_page(props, props.current));
        }
        true
    }

    fn adopt_callbacks(
        &self,
        props: &mut Self::Props,
        _state: &mut Self::State,
        supplied: &Self::Props,
    ) -> bool {
        if same_callback(&props.on_change, &supplied.on_change) {
            return false;
        }
        props.on_change = supplied.on_change.clone();
        true
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        let pages = props.pages.max(1);
        let page = Self::clamp_page(props, self.page.get());
        let mut children = Vec::new();

        // One cell of space between the arrows, the numbers and the ellipses.
        children.push(self.arrow(props, page.saturating_sub(1).max(1), Icon::ChevronLeft));
        children.push(gap());
        if props.compact {
            children.push(
                span()
                    .class("text-foreground")
                    .text(&format!("{page} / {pages}"))
                    .build(),
            );
        } else {
            for (index, entry) in entries(pages, page, props.visible_pages)
                .into_iter()
                .enumerate()
            {
                if index > 0 {
                    children.push(gap());
                }
                children.push(self.entry(props, page, pages, entry, state));
            }
        }
        children.push(gap());
        children.push(self.arrow(props, (page + 1).min(pages), Icon::ChevronRight));

        let mut node = Node::new(Role::Navigation);
        if let Some(label) = &props.aria_label {
            node.set_label(label.clone());
        }
        node.inner.set_position_in_set(page);
        node.inner.set_size_of_set(pages);
        let width = if props.class.split_whitespace().any(|t| t.starts_with("w-")) {
            ""
        } else {
            "w-full"
        };
        div()
            .class(&format!(
                "flex flex-row items-center {width} {}",
                props.class
            ))
            .children(children)
            .build()
            .with_accessibility(node)
            .with_focus(FocusProps::input())
    }

    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        if let Event::Focus(focus) = event {
            match focus.kind {
                FocusEventKind::Gained => state.focused = true,
                FocusEventKind::Lost => {
                    state.focused = false;
                    state.slot = Slot::Page;
                }
                _ => return EventResult::Ignored,
            }
            return EventResult::Consumed;
        }
        let Event::Key(key) = event else {
            return EventResult::Ignored;
        };
        // While the menu of hidden pages is open, its keys belong to it.
        if self.menu.get().is_some() {
            return EventResult::Ignored;
        }
        let Some(action) = Keymap::active().action(key) else {
            return EventResult::Ignored;
        };
        let pages = props.pages.max(1);
        let page = Self::clamp_page(props, self.page.get());
        let entries = entries(pages, page, props.visible_pages);
        let has_leading = entries.iter().any(|entry| {
            matches!(
                entry,
                Entry::Hidden {
                    trailing: false,
                    ..
                }
            )
        });
        let has_trailing = entries
            .iter()
            .any(|entry| matches!(entry, Entry::Hidden { trailing: true, .. }));
        match action {
            Action::Left | Action::Right | Action::Home | Action::End
                if state.slot != Slot::Page =>
            {
                state.slot = Slot::Page;
            }
            Action::Left => self.page.set(page.saturating_sub(1).max(1)),
            Action::Right => self.page.set((page + 1).min(pages)),
            Action::Home => self.page.set(1),
            Action::End => self.page.set(pages),
            Action::Up if state.slot == Slot::Page && has_leading => {
                state.slot = Slot::Leading;
            }
            Action::Up if state.slot == Slot::Trailing => state.slot = Slot::Page,
            Action::Down if state.slot == Slot::Page && has_trailing => {
                state.slot = Slot::Trailing;
            }
            Action::Down if state.slot == Slot::Leading => state.slot = Slot::Page,
            Action::Confirm | Action::Activate => match state.slot {
                Slot::Page => Self::report(props, page),
                Slot::Leading => self.menu.set(Some(false)),
                Slot::Trailing => self.menu.set(Some(true)),
            },
            _ => return EventResult::Ignored,
        }
        EventResult::Consumed
    }
}

impl Pagination {
    /// The previous or next arrow: a button that moves to `target` on a click.
    fn arrow(&self, props: &PaginationProps, target: usize, icon: Icon) -> Element {
        let page = self.page.clone();
        let on_change = props.on_change.clone();
        span()
            .class("text-foreground")
            .text(icon.glyph())
            .on_click(move || {
                page.set(target);
                if let Some(callback) = &on_change {
                    callback(target);
                }
            })
            .build()
            .with_focus(not_focusable())
    }

    /// One entry of the bar: a page or an ellipsis with its menu.
    fn entry(
        &self,
        props: &PaginationProps,
        page: usize,
        pages: usize,
        entry: Entry,
        state: &PaginationState,
    ) -> Element {
        let slot = state.slot;
        let focused = state.focused;
        match entry {
            Entry::Page(number) => {
                let current = number == page;
                let ring = if focused && current && slot == Slot::Page {
                    " ring"
                } else {
                    ""
                };
                let class = if current {
                    format!("bg-primary text-primary-foreground{ring}")
                } else {
                    "text-foreground".to_string()
                };
                let mut node = Node::new(Role::Button);
                node.set_label(format!("Page {number}"));
                node.set_clickable();
                node.inner.set_position_in_set(number);
                node.inner.set_size_of_set(pages);
                if current {
                    node.set_current(AriaCurrent::Page);
                }
                let page_signal = self.page.clone();
                let on_change = props.on_change.clone();
                span()
                    .class(&class)
                    .text(&number.to_string())
                    .on_click(move || {
                        page_signal.set(number);
                        if let Some(callback) = &on_change {
                            callback(number);
                        }
                    })
                    .build()
                    .with_accessibility(node)
                    .with_focus(not_focusable())
            }
            Entry::Hidden { trailing, pages } => {
                let ring = match (trailing, slot) {
                    _ if !focused => "",
                    (true, Slot::Trailing) | (false, Slot::Leading) => " ring",
                    _ => "",
                };
                let mut node = Node::new(Role::Button);
                node.set_label(match pages.len() {
                    1 => "1 hidden page".to_owned(),
                    count => format!("{count} hidden pages"),
                });
                node.set_clickable();
                let open = self.menu.get() == Some(trailing);
                let menu = self.menu.clone();
                node.set_expanded(open);
                let mut children = vec![span()
                    .class(&format!("text-muted{ring}"))
                    .text(Icon::Ellipsis.glyph())
                    .on_click(move || menu.set(Some(trailing)))
                    .build()
                    .with_focus(not_focusable())];
                if open {
                    let items = pages
                        .iter()
                        .map(|number| MenuItem::new(number.to_string(), format!("Page {number}")))
                        .collect();
                    let page_signal = self.page.clone();
                    let menu = self.menu.clone();
                    let on_change = props.on_change.clone();
                    let chosen_menu = self.menu.clone();
                    children.push(PopupMenu::beside(
                        items,
                        RelativePlacement::Below,
                        None,
                        Arc::new(move |id: &str| {
                            let Ok(number) = id.parse::<usize>() else {
                                return;
                            };
                            page_signal.set(number);
                            chosen_menu.set(None);
                            if let Some(callback) = &on_change {
                                callback(number);
                            }
                        }),
                        Arc::new(move || menu.set(None)),
                    ));
                }
                div()
                    .class("flex flex-row shrink-0")
                    .children(children)
                    .build()
                    .with_accessibility(node)
                    .with_focus(not_focusable())
            }
        }
    }
}

/// A one-cell gap between two items of the bar.
fn gap() -> Element {
    span().text(" ").build()
}

/// Focus props that take no focus: a pointer target that is not a stop of the keyboard.
fn not_focusable() -> FocusProps {
    FocusProps {
        focusable: false,
        ..FocusProps::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accessibility::Role;

    fn text_of(entries: &[Entry]) -> String {
        entries
            .iter()
            .map(|entry| match entry {
                Entry::Page(n) => n.to_string(),
                Entry::Hidden { .. } => "…".to_string(),
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn dis_003_twenty_pages_at_five_show_the_ends_and_the_window_around_five() {
        assert_eq!(text_of(&entries(20, 5, 5)), "1 … 4 5 6 … 20");
    }

    #[test]
    fn dis_003_the_window_stays_at_the_ends() {
        assert_eq!(text_of(&entries(20, 1, 5)), "1 2 3 4 … 20");
        assert_eq!(text_of(&entries(20, 20, 5)), "1 … 17 18 19 20");
    }

    #[test]
    fn dis_003_a_bar_with_few_pages_shows_them_all() {
        assert_eq!(text_of(&entries(4, 2, 5)), "1 2 3 4");
    }

    #[test]
    fn dis_003_the_hidden_run_lists_the_pages_it_stands_for() {
        let hidden = entries(20, 5, 5);
        assert_eq!(
            hidden[1],
            Entry::Hidden {
                trailing: false,
                pages: vec![2, 3],
            }
        );
        assert_eq!(hidden[3], Entry::Page(5));
        assert_eq!(
            hidden[5],
            Entry::Hidden {
                trailing: true,
                pages: (7..20).collect(),
            }
        );
    }

    #[test]
    fn dis_004_the_bar_is_a_navigation_landmark_labelled_by_its_aria_label() {
        let props = PaginationProps {
            pages: 20,
            current: 5,
            aria_label: Some("Results pages".into()),
            ..PaginationProps::default()
        };
        let element = Pagination::new(props.clone()).render(&props, &PaginationState::default());
        let node = element.metadata.accessibility.as_ref().expect("a node");
        assert_eq!(node.role(), Role::Navigation);
        assert_eq!(node.inner.label(), Some("Results pages"));
        assert_eq!(node.inner.position_in_set(), Some(5));
        assert_eq!(node.inner.size_of_set(), Some(20));
    }

    #[test]
    fn dis_005_the_bar_ellipsis_and_arrows_come_from_the_catalog() {
        assert_eq!(Icon::Ellipsis.unicode(), "…");
        assert_eq!(Icon::ChevronLeft.unicode(), "‹");
        assert_eq!(Icon::ChevronRight.unicode(), "›");
    }
}
