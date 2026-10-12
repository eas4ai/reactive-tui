//! charts-goldens mechanism, BAR-004 for the menus: the menu bar, the popup
//! menu, the context menu and the dialog menu, each open, on the debug
//! backend under the dark preset, against checked-in goldens of the text
//! grid plus a color digest at 80 by 24 and at 400 by 100. The grid holds
//! every cell of every row, so the wide golden of the menu bar, which fills
//! the width its parent allots, is 400 columns wide. Goldens live in
//! `tests/snapshots/menus/<menu>_<size>.ansi`, or under
//! `REACTIVE_TUI_SNAPSHOTS` when the check points there; run with
//! `REGENERATE=1` to write them, review the diff, then commit.

mod common;

use common::app_input::{self, Snapshot, Until};
use reactive_tui::{
    app::RootComponent,
    builder,
    component::Element,
    event::types::{Event, KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind, Position},
    theme::{dark_theme, Theme},
    widgets::menu::{
        ContextMenu, ContextMenuProps, DialogMenu, DialogMenuProps, MenuBar, MenuBarProps,
        MenuItem, MenuShortcut, PopupMenu, PopupMenuProps, PopupPlacement,
    },
};

struct Root(Element);
impl RootComponent for Root {
    fn render(&self) -> Element {
        self.0.clone()
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

fn snapshots_dir() -> std::path::PathBuf {
    std::env::var_os("REACTIVE_TUI_SNAPSHOTS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots")
        })
        .join("menus")
}

/// Every cell of every row as text, then a digest of every cell's colors.
fn golden_bytes(frame: &Snapshot) -> Vec<u8> {
    let (rows, columns) = frame.screen.size();
    let mut hasher = common::digest::Digest::default();
    let mut grid = String::new();
    for row in 0..rows {
        for column in 0..columns {
            let Some(cell) = frame.screen.cell(row, column) else {
                continue;
            };
            hasher.field(format!("{:?}{:?}", cell.fgcolor(), cell.bgcolor()).as_bytes());
            if !cell.is_wide_continuation() {
                grid.push_str(if cell.contents().is_empty() {
                    " "
                } else {
                    cell.contents()
                });
            }
        }
        grid.push('\n');
    }
    format!("{grid}colors: {:016x}\n", hasher.finish()).into_bytes()
}

fn key(code: KeyCode) -> Option<Event> {
    Some(Event::Key(KeyEvent::new(code)))
}

fn wait(text: &'static str, event: Option<Event>) -> Until {
    Until {
        text,
        cell: None,
        event,
    }
}

/// A row of every kind, the last with a submenu.
fn items() -> Vec<MenuItem> {
    vec![
        MenuItem::new("new", "New").shortcut(MenuShortcut::new("Ctrl+N", vec!["ctrl+n"])),
        MenuItem::new("open", "Open").icon("*"),
        MenuItem::separator(),
        MenuItem::checkbox("wrap", "Wrap lines", true, |_| {}),
        MenuItem::radio("large", "Large", true, "size", || {}),
        MenuItem::new("locked", "Locked").enabled(false),
        MenuItem::submenu(
            "recent",
            "Recent",
            vec![
                MenuItem::new("first", "First file"),
                MenuItem::new("second", "Second file"),
            ],
        ),
    ]
}

fn page(child: Element) -> Element {
    builder::div()
        .class("relative w-full h-full bg-background text-foreground")
        .child(child)
        .build()
}

/// Each menu by name, with the steps that open it to its last panel.
fn menus() -> Vec<(&'static str, Element, Vec<Until>)> {
    let to_the_submenu = || {
        vec![
            wait("Recent", key(KeyCode::End)),
            wait("Recent", key(KeyCode::Right)),
            wait("Second file", None),
        ]
    };
    let mut open_bar = vec![wait("File", key(KeyCode::Down))];
    open_bar.extend(to_the_submenu());
    vec![
        (
            "menu_bar",
            page(
                Element::typed::<MenuBar>(MenuBarProps {
                    title: Some("Catalog".into()),
                    items: vec![
                        MenuItem::submenu("file", "File", items()),
                        MenuItem::new("edit", "Edit"),
                        MenuItem::new("view", "View").enabled(false),
                    ],
                    ..Default::default()
                })
                .auto_focus(),
            ),
            open_bar,
        ),
        (
            "popup_menu",
            page(
                Element::typed::<PopupMenu>(PopupMenuProps {
                    visible: true,
                    items: items(),
                    placement: PopupPlacement::Position { x: 4, y: 2 },
                    ..Default::default()
                })
                .auto_focus(),
            ),
            to_the_submenu(),
        ),
        (
            "context_menu",
            page(Element::typed::<ContextMenu>(ContextMenuProps {
                items: items(),
                ..Default::default()
            })),
            vec![
                wait(
                    "",
                    Some(Event::Mouse(
                        MouseEvent::new(MouseEventKind::Down, Position::cell(6, 3))
                            .with_button(MouseButton::Right),
                    )),
                ),
                wait("Recent", None),
            ],
        ),
        (
            "dialog_menu",
            page(Element::typed::<DialogMenu>(DialogMenuProps {
                visible: true,
                title: Some("Open a file".into()),
                message: Some("Choose what to open.".into()),
                items: items(),
                ..Default::default()
            })),
            vec![wait("Recent", None)],
        ),
    ]
}

#[test]
#[serial_test::serial(theme)]
fn bar_004_menu_goldens_at_80_by_24_and_400_by_100() {
    let before = Theme::active();
    Theme::set_active(dark_theme());
    let mut mismatches = Vec::new();
    for size in [(80u16, 24u16), (400u16, 100u16)] {
        for (menu, root, steps) in menus() {
            let frame = app_input::run_until_on_debug(
                Root(root),
                size,
                steps,
                std::time::Duration::from_secs(10),
            )
            .pop()
            .expect("a painted frame");
            let name = format!("{menu}_{}x{}", size.0, size.1);
            let path = snapshots_dir().join(format!("{name}.ansi"));
            let bytes = golden_bytes(&frame);
            if std::env::var("REGENERATE").as_deref() == Ok("1") {
                std::fs::create_dir_all(snapshots_dir()).expect("snapshot dir");
                std::fs::write(&path, &bytes).expect("write golden");
                continue;
            }
            match std::fs::read(&path) {
                Ok(expected) if expected == bytes => {}
                Ok(_) => mismatches.push(format!("golden mismatch for {name}")),
                Err(_) => mismatches.push(format!("{name}: no golden at {}", path.display())),
            }
        }
    }
    Theme::set_active((*before).clone());
    assert!(
        mismatches.is_empty(),
        "BAR-004: menu goldens: {mismatches:?}"
    );
}
