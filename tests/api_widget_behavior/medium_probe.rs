//! The medium widgets for the real reader workflow (NAV-011): a resizable
//! split, a sidebar, a list of 70,000 items and a log view, in this focus
//! order with nothing focusable between them. Enter on the list appends a
//! row to the log while the focus stays on the list.
use reactive_tui::{
    builder,
    component::Element,
    widgets::{
        display::ListItem,
        layout::{LogHandle, LogRow, Pane, SidebarItem},
    },
};
use std::sync::OnceLock;

/// The log's rows: one handle for every render of the page, so the row the
/// list's Enter appends reaches the view on the screen.
fn log() -> &'static LogHandle {
    static LOG: OnceLock<LogHandle> = OnceLock::new();
    LOG.get_or_init(|| {
        let log = LogHandle::new();
        log.append_all(vec![
            LogRow::new("Build started"),
            LogRow::new("Fetching crates"),
            LogRow::new("Checking the manifest"),
        ]);
        log
    })
}

/// The list's items, `Item 1` to `Item 70000`, made once.
fn items() -> &'static [ListItem] {
    static ITEMS: OnceLock<Vec<ListItem>> = OnceLock::new();
    ITEMS.get_or_init(|| {
        (1..=70_000)
            .map(|n| ListItem::new(&format!("item-{n}"), &format!("Item {n}")))
            .collect()
    })
}

pub(super) fn page() -> Element {
    let appended = log().clone();
    builder::div()
        .class("w-full h-full flex flex-col gap-1")
        .child(Element::text("Medium widgets"))
        .child(
            builder::div()
                .class("w-full h-5 shrink-0")
                .child(
                    builder::resizable()
                        .pane_with(Pane::new(Element::text("Left pane")).size(40))
                        .pane(Element::text("Right pane"))
                        .aria_label("Panes")
                        .build()
                        .auto_focus(),
                )
                .build(),
        )
        .child(
            builder::div()
                .class("w-full flex-1 min-h-0 flex flex-row gap-1")
                .child(
                    builder::sidebar()
                        .item(SidebarItem::new("home", "Home"))
                        .item(
                            SidebarItem::new("projects", "Projects")
                                .child(SidebarItem::new("alpha", "Alpha"))
                                .child(SidebarItem::new("beta", "Beta")),
                        )
                        .item(SidebarItem::new("settings", "Settings"))
                        .aria_label("Main")
                        .build(),
                )
                .child(
                    builder::div()
                        .class("w-40 h-full shrink-0")
                        .child(
                            builder::list()
                                .items(items().to_vec())
                                .aria_label("Fruits")
                                .on_confirm(move |_| {
                                    appended.append(LogRow::new("Compiling widgets"))
                                })
                                .build(),
                        )
                        .build(),
                )
                .child(
                    builder::div()
                        .class("flex-1 min-w-0 h-full")
                        .child(
                            builder::log_view()
                                .handle(log().clone())
                                .aria_label("Build log")
                                .build(),
                        )
                        .build(),
                )
                .build(),
        )
        .build()
}
