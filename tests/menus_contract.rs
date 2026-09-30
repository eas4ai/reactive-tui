//! MNU-001 to MNU-004 (docs/spec/menus.md): the colors of the menu bar, the
//! context menu, the popup menu and the dialog menu by role, the size of a
//! panel, where a panel opens, and what it is painted over. And what the
//! widget bar asks of the four menus (BAR-003): every color from the
//! active theme, the width the parent allots, a layout that follows a
//! resize, and a key for what the pointer does. What a row tells the screen
//! reader is a unit test beside the row (src/widgets/menu/view.rs).
//!
//! Every test takes its turn (`serial(theme)`): the color tests set the
//! active theme, which is one for the process, and the others must not
//! paint under it.

mod common;

use common::app_input::{self, Snapshot, Until};
use reactive_tui::{
    app::RootComponent,
    builder::{
        self,
        widgets::menu::{ContextMenuBuilder, MenuBarBuilder, MenuItemBuilder, PopupMenuBuilder},
    },
    component::{Component, Element},
    event::types::{
        Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind, Position,
        ResizeEvent,
    },
    layout::style::StyleBuilder,
    theme::{dark_theme, high_contrast_theme, light_theme, Theme, ThemeVariables},
    widgets::menu::{
        ContextMenu, ContextMenuProps, DialogMenu, DialogMenuBuilder, DialogMenuProps, MenuBar,
        MenuBarProps, MenuBarState, MenuItem, MenuShortcut, MenuTheme, PopupMenu, PopupMenuProps,
        PopupPlacement,
    },
};

type Rgb = [u8; 3];
/// A cell as its glyph, its glyph's color and its background.
type Cell = (String, vt100::Color, vt100::Color);

struct Control(Element);
impl RootComponent for Control {
    fn render(&self) -> Element {
        self.0.clone()
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

/// The roles a menu paints with.
const ROLES: [&str; 10] = [
    "background",
    "surface",
    "foreground",
    "text-muted",
    "border",
    "hover",
    "selection",
    "selection-foreground",
    "primary",
    "primary-foreground",
];

/// The veil and the shadow of the probe theme: a color and how much of it
/// is laid over what is under it.
const OVERLAY: (Rgb, f32) = ([200, 20, 50], 0.5);
const SHADOW: (Rgb, f32) = ([20, 50, 200], 0.4);

/// The color the probe theme gives the role at `index` of `ROLES`.
fn probe_color(index: usize) -> Rgb {
    [
        30 + 9 * index as u8,
        200 - 7 * index as u8,
        90 + 5 * index as u8,
    ]
}

fn role(name: &str) -> Rgb {
    probe_color(
        ROLES
            .iter()
            .position(|role| *role == name)
            .unwrap_or_else(|| panic!("{name} is not a role of the probe theme")),
    )
}

fn hex(color: Rgb, alpha: Option<f32>) -> String {
    let [r, g, b] = color;
    match alpha {
        Some(alpha) => format!(
            "#{r:02x}{g:02x}{b:02x}{:02x}",
            (alpha * 255.0).round() as u8
        ),
        None => format!("#{r:02x}{g:02x}{b:02x}"),
    }
}

/// A theme whose roles all differ, so a cell's color names its role.
fn probe() -> Theme {
    let mut variables = ThemeVariables::new()
        .set("--color-overlay", hex(OVERLAY.0, Some(OVERLAY.1)))
        .set("--color-shadow", hex(SHADOW.0, Some(SHADOW.1)));
    for (index, name) in ROLES.iter().enumerate() {
        variables = variables.set(Theme::color_variable(name), hex(probe_color(index), None));
    }
    Theme::new("probe").with_variables(variables)
}

/// Makes `theme` the active theme until it is dropped, also when the test
/// fails in between.
struct Active(std::sync::Arc<Theme>);
impl Active {
    fn set(theme: Theme) -> Self {
        let before = Theme::active();
        Theme::set_active(theme);
        Self(before)
    }
}
impl Drop for Active {
    fn drop(&mut self) {
        Theme::set_active((*self.0).clone());
    }
}

fn rgb(color: vt100::Color) -> Option<Rgb> {
    match color {
        vt100::Color::Rgb(r, g, b) => Some([r, g, b]),
        _ => None,
    }
}

/// `over` laid over `under` with `alpha` of it.
fn laid_over((over, alpha): (Rgb, f32), under: Rgb) -> Rgb {
    let mut mixed = [0; 3];
    for channel in 0..3 {
        mixed[channel] = (f32::from(over[channel]) * alpha
            + f32::from(under[channel]) * (1.0 - alpha))
            .round() as u8;
    }
    mixed
}

fn near(got: Option<Rgb>, expected: Rgb) -> bool {
    got.is_some_and(|got| got.iter().zip(expected).all(|(g, e)| g.abs_diff(e) <= 2))
}

/// The cell where `needle` starts, as (column, row).
fn find(frame: &Snapshot, needle: &str) -> Option<(u16, u16)> {
    let (rows, columns) = frame.screen.size();
    let glyphs: Vec<String> = needle.chars().map(String::from).collect();
    let length = glyphs.len() as u16;
    (0..rows)
        .flat_map(|row| (0..=columns.saturating_sub(length)).map(move |column| (column, row)))
        .find(|(column, row)| {
            glyphs.iter().enumerate().all(|(offset, glyph)| {
                frame
                    .screen
                    .cell(*row, column + offset as u16)
                    .is_some_and(|cell| cell.contents() == *glyph)
            })
        })
}

/// The glyph color and the background of the cell where `needle` starts.
fn colors(frame: &Snapshot, needle: &str) -> (Option<Rgb>, Option<Rgb>) {
    let (column, row) =
        find(frame, needle).unwrap_or_else(|| panic!("{needle:?} is not painted:\n{}", frame.text));
    let cell = frame.screen.cell(row, column).unwrap();
    (rgb(cell.fgcolor()), rgb(cell.bgcolor()))
}

fn background(frame: &Snapshot, column: u16, row: u16) -> Option<Rgb> {
    frame
        .screen
        .cell(row, column)
        .and_then(|cell| rgb(cell.bgcolor()))
}

/// Every cell of `frame` as its glyph and its two colors.
fn painted(frame: &Snapshot) -> Vec<Cell> {
    let (rows, columns) = frame.screen.size();
    (0..rows)
        .flat_map(|row| (0..columns).map(move |column| (row, column)))
        .map(|(row, column)| {
            let cell = frame.screen.cell(row, column).unwrap();
            (cell.contents().to_owned(), cell.fgcolor(), cell.bgcolor())
        })
        .collect()
}

/// The cells of the first panel in `frame`, from its first corner to its
/// last, row by row, with the panel's width before them.
fn panel_cells(frame: &Snapshot) -> Option<(u16, Vec<Cell>)> {
    let (left, top) = find(frame, "┌")?;
    let (rows, columns) = frame.screen.size();
    let glyph = |column: u16, row: u16| frame.screen.cell(row, column).map(|cell| cell.contents());
    let right = (left..columns).find(|column| glyph(*column, top) == Some("┐"))?;
    let bottom = (top..rows).find(|row| glyph(left, *row) == Some("└"))?;
    let cells = (top..=bottom)
        .flat_map(|row| (left..=right).map(move |column| (row, column)))
        .map(|(row, column)| {
            let cell = frame.screen.cell(row, column).unwrap();
            (cell.contents().to_owned(), cell.fgcolor(), cell.bgcolor())
        })
        .collect();
    Some((right - left + 1, cells))
}

fn key(code: KeyCode) -> Option<Event> {
    Some(Event::Key(KeyEvent::new(code)))
}

fn right_click(column: u16, row: u16) -> Option<Event> {
    Some(Event::Mouse(
        MouseEvent::new(MouseEventKind::Down, Position::cell(column, row))
            .with_button(MouseButton::Right),
    ))
}

/// A page in the theme's background that holds `child`.
fn page(child: Element) -> Element {
    builder::div()
        .class("relative w-full h-full bg-background text-foreground")
        .child(child)
        .build()
}

fn popup(items: Vec<MenuItem>, placement: PopupPlacement) -> Element {
    Element::typed::<PopupMenu>(PopupMenuProps {
        visible: true,
        items,
        placement,
        ..Default::default()
    })
}

fn rows(count: usize) -> Vec<MenuItem> {
    (1..=count)
        .map(|number| MenuItem::new(format!("row{number}"), format!("ROW{number:02}")))
        .collect()
}

// ---------------------------------------------------------------- BAR-003

/// Whether `color` is a role of the probe theme, or its veil or its shadow
/// laid over one, or its shadow laid over its veil over one: a dialog
/// menu's shadow falls on the veil.
fn of_the_theme(color: Option<Rgb>) -> bool {
    let roles = || (0..ROLES.len()).map(probe_color);
    color.is_some_and(|color| roles().any(|role| role == color))
        || roles().any(|role| {
            let veiled = laid_over(OVERLAY, role);
            near(color, veiled)
                || near(color, laid_over(SHADOW, role))
                || near(color, laid_over(SHADOW, veiled))
        })
}

/// The colors in `frame` that the probe theme does not define, each with
/// the first cell that has it.
fn foreign(frame: &Snapshot) -> Vec<String> {
    let (rows, columns) = frame.screen.size();
    let mut found: Vec<(vt100::Color, String)> = Vec::new();
    for (row, column) in (0..rows).flat_map(|row| (0..columns).map(move |column| (row, column))) {
        let cell = frame.screen.cell(row, column).unwrap();
        let mut colors = vec![("background", cell.bgcolor())];
        if !cell.contents().trim().is_empty() {
            colors.push(("glyph", cell.fgcolor()));
        }
        for (part, color) in colors {
            if !of_the_theme(rgb(color)) && !found.iter().any(|(seen, _)| *seen == color) {
                found.push((
                    color,
                    format!(
                        "{color:?} as the {part} of {:?} at ({column}, {row})",
                        cell.contents()
                    ),
                ));
            }
        }
    }
    found.into_iter().map(|(_, place)| place).collect()
}

fn items() -> Vec<MenuItem> {
    vec![
        MenuItem::new("new", "NEW").shortcut(MenuShortcut::new("Ctrl+N", vec!["ctrl+n"])),
        MenuItem::new("open", "OPEN").icon("*"),
        MenuItem::separator(),
        MenuItem::checkbox("wrap", "WRAP", true, |_| {}),
        MenuItem::new("off", "OFF").enabled(false),
        MenuItem::submenu("more", "MORE", rows(2)),
    ]
}

#[test]
#[serial_test::serial(theme)]
fn bar_003_a_menu_takes_every_color_from_the_active_theme() {
    let _theme = Active::set(probe());
    let size = (60, 20);
    let mut wrong = Vec::new();
    let mut check = |name: &str, frames: Vec<Snapshot>| {
        let frame = frames.last().unwrap();
        let foreign = foreign(frame);
        if !foreign.is_empty() {
            wrong.push(format!("{name}: {foreign:#?}\n{}", frame.text));
        }
    };
    check(
        "menu bar",
        app_input::run_until(
            Control(page(
                Element::typed::<MenuBar>(MenuBarProps {
                    title: Some("TITLE".into()),
                    items: vec![
                        MenuItem::submenu("file", "FILE", items()),
                        MenuItem::new("edit", "EDIT"),
                    ],
                    ..Default::default()
                })
                .auto_focus(),
            )),
            size,
            vec![
                Until {
                    text: "FILE",
                    cell: None,
                    event: key(KeyCode::Down),
                },
                Until {
                    text: "MORE",
                    cell: None,
                    event: None,
                },
            ],
            PANEL_WAIT,
        ),
    );
    check(
        "popup menu",
        app_input::run_when(
            Control(page(
                popup(items(), PopupPlacement::Position { x: 4, y: 2 }).auto_focus(),
            )),
            size,
            vec![
                ("MORE", key(KeyCode::End)),
                ("MORE", key(KeyCode::Right)),
                ("ROW02", None),
            ],
        ),
    );
    check(
        "context menu",
        app_input::run_when(
            Control(page(Element::typed::<ContextMenu>(ContextMenuProps {
                items: items(),
                ..Default::default()
            }))),
            size,
            vec![("", right_click(6, 3)), ("MORE", None)],
        ),
    );
    check(
        "dialog menu",
        app_input::run_when(
            Control(page(Element::typed::<DialogMenu>(DialogMenuProps {
                visible: true,
                title: Some("TITLE".into()),
                message: Some("MESSAGE".into()),
                items: items(),
                ..Default::default()
            }))),
            size,
            vec![("MORE", None)],
        ),
    );
    assert!(
        wrong.is_empty(),
        "BAR-003: colors that the active theme does not define: {}",
        wrong.join("\n")
    );
}

#[test]
#[serial_test::serial(theme)]
fn bar_003_a_menu_bar_fills_the_width_its_parent_allots() {
    let _theme = Active::set(probe());
    // A parent that lays its children out in a column, and one that is a
    // plain box.
    for (class, width) in [
        ("flex flex-col w-full", 240),
        ("flex flex-col w-100", 100),
        ("w-full", 240),
        ("w-100", 100),
    ] {
        let tree = builder::div()
            .class("flex flex-col w-full h-full bg-background text-foreground")
            .child(
                builder::div()
                    .class(class)
                    .child(Element::typed::<MenuBar>(MenuBarProps {
                        items: vec![MenuItem::new("file", "FILE")],
                        ..Default::default()
                    }))
                    .build(),
            )
            .build();
        let frames = app_input::run_when(Control(tree), (240, 20), vec![("FILE", None)]);
        let frame = frames.last().unwrap();
        let (_, row) = find(frame, "FILE").unwrap();
        // The bar's own colors: its surface, and the current title's.
        let bar = [role("surface"), role("hover"), role("selection")];
        let painted = (0..240)
            .filter(|column| {
                background(frame, *column, row).is_some_and(|color| bar.contains(&color))
            })
            .count();
        assert_eq!(
            painted, width,
            "BAR-003: the cells of the menu bar's row in the bar's colors, in a parent `{class}` of {width} cells"
        );
    }
}

#[test]
#[serial_test::serial(theme)]
fn bar_003_a_menu_bar_and_its_panel_follow_a_resize() {
    let bar = Element::typed::<MenuBar>(MenuBarProps {
        items: vec![MenuItem::submenu("file", "FILE", rows(2))],
        ..Default::default()
    })
    .auto_focus();
    let tree = builder::div()
        .class("flex flex-col w-full h-full bg-background text-foreground")
        .child(builder::div().class("grow").build())
        .child(bar)
        .build();
    let (old, new) = ((60u16, 20u16), (100u16, 40u16));
    let frames = app_input::run_until(
        Control(tree),
        old,
        vec![
            Until {
                text: "FILE",
                cell: None,
                event: key(KeyCode::Down),
            },
            Until {
                text: "ROW02",
                cell: None,
                event: Some(Event::Resize(ResizeEvent::new(new.0, new.1))),
            },
            // At the new size the title stands on row 38, between the
            // bar's two rows of padding, and the panel's four rows end on
            // the row over it.
            Until {
                text: "ROW02",
                cell: Some((1, 37, "└")),
                event: None,
            },
        ],
        PANEL_WAIT,
    );
    let frame = frames.last().unwrap();
    assert_eq!(
        (
            frame.screen.size(),
            find(frame, "FILE").map(|(_, row)| row),
            find(frame, "ROW01").map(|(_, row)| row),
            find(frame, "ROW02").map(|(_, row)| row),
        ),
        ((new.1, new.0), Some(38), Some(35), Some(36)),
        "BAR-003: the screen's size and the rows of the title and of the panel's two rows after the resize:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn bar_003_the_keyboard_alone_opens_a_context_menu_and_runs_its_action() {
    let calls = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let (first, second) = (calls.clone(), calls.clone());
    // The area the context menu serves is a focus stop of its own: the
    // focus starts on a button before it, and Tab reaches the menu.
    let menu = builder::div()
        .class("flex flex-col w-full h-full")
        .child(builder::button().text("BEFORE").build().auto_focus())
        .child(
            Element::typed::<ContextMenu>(ContextMenuProps {
                items: vec![
                    MenuItem::action("one", "ONE", move || first.lock().unwrap().push("one")),
                    MenuItem::action("two", "TWO", move || second.lock().unwrap().push("two")),
                ],
                ..Default::default()
            })
            .class("w-full h-4"),
        )
        .build();
    let shift_f10 = || {
        Some(Event::Key(KeyEvent::new(KeyCode::F(10)).with_modifiers(
            KeyModifiers {
                shift: true,
                ctrl: false,
                alt: false,
                meta: false,
            },
        )))
    };
    let frames = app_input::run_visibility(
        Control(page(menu)),
        (60, 20),
        vec![
            ("BEFORE", Some("TWO"), key(KeyCode::Tab)),
            ("BEFORE", Some("TWO"), shift_f10()),
            ("TWO", None, key(KeyCode::Down)),
            ("TWO", None, key(KeyCode::Enter)),
            // The action closed the menu; Shift+F10 opens it again and
            // Escape closes it without an action.
            ("BEFORE", Some("TWO"), shift_f10()),
            ("TWO", None, key(KeyCode::Escape)),
            ("BEFORE", Some("TWO"), None),
        ],
    );
    assert_eq!(
        (
            calls.lock().unwrap().clone(),
            frames.last().unwrap().text.contains("ONE")
        ),
        (vec!["two"], false),
        "BAR-003: the actions that keys alone ran, and whether the menu is still open after Escape"
    );
}

// ---------------------------------------------------------------- MNU-001

#[test]
#[serial_test::serial(theme)]
fn mnu_001_a_default_menu_looks_the_same_from_its_props_and_from_its_builder() {
    let _theme = Active::set(probe());
    let size = (60, 20);
    let mut differ = Vec::new();
    let mut compare = |name: &str, one: Vec<Snapshot>, other: Vec<Snapshot>| {
        let (one, other) = (one.last().unwrap(), other.last().unwrap());
        // A popup menu from its builder opens beside its own element and
        // one from its props at its placement, so their panels are compared
        // and not their places; the other menus open at the same cell.
        let cells = if name == "popup menu" {
            match (panel_cells(one), panel_cells(other)) {
                (Some(one), Some(other)) if one.0 == other.0 => {
                    one.1.iter().zip(&other.1).filter(|(a, b)| a != b).count()
                }
                (one, other) => {
                    one.map_or(1, |(_, cells)| cells.len())
                        + other.map_or(1, |(_, cells)| cells.len())
                }
            }
        } else {
            painted(one)
                .iter()
                .zip(painted(other))
                .filter(|(a, b)| **a != *b)
                .count()
        };
        if cells > 0 {
            differ.push(format!(
                "{name}: {cells} cells differ\nfrom its props:\n{}\nfrom its builder:\n{}",
                one.text, other.text
            ));
        }
    };

    let bar = vec![("FILE", key(KeyCode::Down)), ("NEW", None)];
    compare(
        "menu bar",
        app_input::run_when(
            Control(page(
                Element::typed::<MenuBar>(MenuBarProps {
                    items: vec![MenuItem::submenu(
                        "file",
                        "FILE",
                        vec![
                            MenuItem::new("new", "NEW"),
                            MenuItem::new("off", "OFF").enabled(false),
                        ],
                    )],
                    ..Default::default()
                })
                .auto_focus(),
            )),
            size,
            bar.clone(),
        ),
        app_input::run_when(
            Control(page(
                MenuBarBuilder::new()
                    .item(
                        MenuItemBuilder::new("file", "FILE")
                            .submenu(vec![
                                MenuItemBuilder::new("new", "NEW").build(),
                                MenuItemBuilder::new("off", "OFF").enabled(false).build(),
                            ])
                            .build(),
                    )
                    .build()
                    .auto_focus(),
            )),
            size,
            bar,
        ),
    );

    compare(
        "popup menu",
        app_input::run_when(
            Control(page(popup(
                vec![
                    MenuItem::new("new", "NEW"),
                    MenuItem::new("off", "OFF").enabled(false),
                ],
                PopupPlacement::default(),
            ))),
            size,
            vec![("NEW", None)],
        ),
        app_input::run_when(
            Control(page(
                PopupMenuBuilder::new()
                    .item(MenuItemBuilder::new("new", "NEW").build())
                    .item(MenuItemBuilder::new("off", "OFF").enabled(false).build())
                    .build(),
            )),
            size,
            vec![("NEW", None)],
        ),
    );

    let context = vec![("", right_click(6, 3)), ("NEW", None)];
    compare(
        "context menu",
        app_input::run_when(
            Control(page(Element::typed::<ContextMenu>(ContextMenuProps {
                items: vec![
                    MenuItem::new("new", "NEW"),
                    MenuItem::new("off", "OFF").enabled(false),
                ],
                ..Default::default()
            }))),
            size,
            context.clone(),
        ),
        app_input::run_when(
            Control(page(
                ContextMenuBuilder::new()
                    .item(MenuItemBuilder::new("new", "NEW").build())
                    .item(MenuItemBuilder::new("off", "OFF").enabled(false).build())
                    .build(),
            )),
            size,
            context,
        ),
    );

    let mut built = DialogMenuBuilder::selection()
        .title("TITLE")
        .message("MESSAGE")
        .items(vec![
            MenuItem::new("new", "NEW"),
            MenuItem::new("off", "OFF").enabled(false),
        ])
        .build();
    built.visible = true;
    compare(
        "dialog menu",
        app_input::run_when(
            Control(page(Element::typed::<DialogMenu>(DialogMenuProps {
                visible: true,
                title: Some("TITLE".into()),
                message: Some("MESSAGE".into()),
                items: vec![
                    MenuItem::new("new", "NEW"),
                    MenuItem::new("off", "OFF").enabled(false),
                ],
                ..Default::default()
            }))),
            size,
            vec![("NEW", None)],
        ),
        app_input::run_when(
            Control(page(Element::typed::<DialogMenu>(built))),
            size,
            vec![("NEW", None)],
        ),
    );

    assert!(differ.is_empty(), "MNU-001: {}", differ.join("\n"));
}

/// A popup menu with the default style and the focus: a current row, a row
/// with a shortcut, a separator and a disabled row.
fn parts() -> Element {
    page(
        popup(
            vec![
                MenuItem::new("new", "NEW").shortcut(MenuShortcut::new("Ctrl+N", vec!["ctrl+n"])),
                MenuItem::new("open", "OPEN").shortcut(MenuShortcut::new("Ctrl+O", vec!["ctrl+o"])),
                MenuItem::separator(),
                MenuItem::new("off", "OFF").enabled(false),
            ],
            PopupPlacement::Position { x: 4, y: 2 },
        )
        .auto_focus(),
    )
}

#[test]
#[serial_test::serial(theme)]
fn mnu_001_a_default_menu_paints_each_part_in_its_role() {
    let _theme = Active::set(probe());
    let frames = app_input::run_when(Control(parts()), (60, 20), vec![("OFF", None)]);
    let frame = frames.last().unwrap();
    let mut wrong = Vec::new();
    let mut check = |part: &str, got: (Option<Rgb>, Option<Rgb>), glyph: &str, ground: &str| {
        let expected = (Some(role(glyph)), Some(role(ground)));
        if got != expected {
            wrong.push(format!(
                "{part}: painted {got:?}, `{glyph}` on `{ground}` is {expected:?}"
            ));
        }
    };
    check(
        "the current row",
        colors(frame, "NEW"),
        "selection-foreground",
        "selection",
    );
    // The current row is one color from end to end, its shortcut too.
    check(
        "the current row's shortcut",
        colors(frame, "Ctrl+N"),
        "selection-foreground",
        "selection",
    );
    check("a row", colors(frame, "OPEN"), "foreground", "surface");
    check(
        "a shortcut",
        colors(frame, "Ctrl+O"),
        "text-muted",
        "surface",
    );
    check(
        "a disabled row",
        colors(frame, "OFF"),
        "text-muted",
        "surface",
    );
    check("the border", colors(frame, "┌"), "border", "surface");
    let (column, row) = find(frame, "OPEN").unwrap();
    let separator = frame.screen.cell(row + 1, column).unwrap();
    check(
        "the separator",
        (rgb(separator.fgcolor()), rgb(separator.bgcolor())),
        "text-muted",
        "surface",
    );
    assert!(wrong.is_empty(), "MNU-001: {wrong:#?}\n{}", frame.text);
}

#[test]
#[serial_test::serial(theme)]
fn mnu_001_the_current_row_of_a_menu_without_the_focus_is_painted_in_hover() {
    let _theme = Active::set(probe());
    let props = MenuBarProps {
        items: vec![MenuItem::new("file", "FILE"), MenuItem::new("edit", "EDIT")],
        ..Default::default()
    };
    let bar = MenuBar::new(props.clone()).render(
        &props,
        &MenuBarState {
            selected_index: Some(0),
            ..Default::default()
        },
    );
    let tree = builder::div()
        .class("flex flex-col w-full h-full bg-background text-foreground")
        .child(bar)
        .child(builder::button().text("ELSEWHERE").build().auto_focus())
        .build();
    let frames = app_input::run_when(Control(tree), (60, 20), vec![("ELSEWHERE", None)]);
    let frame = frames.last().unwrap();
    assert_eq!(
        (colors(frame, "FILE"), colors(frame, "EDIT")),
        (
            (Some(role("foreground")), Some(role("hover"))),
            (Some(role("foreground")), Some(role("surface")))
        ),
        "MNU-001: the current title and its neighbour, as (glyph, background):\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn mnu_001_the_veil_and_the_shadow_are_their_roles_laid_over_the_page() {
    let _theme = Active::set(probe());
    let dialog = app_input::run_when(
        Control(page(Element::typed::<DialogMenu>(DialogMenuProps {
            visible: true,
            title: Some("TITLE".into()),
            items: vec![MenuItem::new("new", "NEW")],
            ..Default::default()
        }))),
        (60, 20),
        vec![("NEW", None)],
    );
    let veil = background(dialog.last().unwrap(), 0, 0);
    let expected = laid_over(OVERLAY, role("background"));
    assert!(
        near(veil, expected),
        "MNU-001: the veil's first cell is {veil:?}; `overlay` laid over `background` is {expected:?}"
    );

    let menu = app_input::run_when(
        Control(page(popup(
            rows(2),
            PopupPlacement::Position { x: 4, y: 2 },
        ))),
        (60, 20),
        vec![("ROW02", None)],
    );
    let frame = menu.last().unwrap();
    let (column, row) = find(frame, "┐").expect("the panel's border");
    let shadow = background(frame, column + 1, row + 1);
    let expected = laid_over(SHADOW, role("background"));
    assert!(
        near(shadow, expected),
        "MNU-001: the cell right of the panel, one row down, is {shadow:?}; `shadow` laid over `background` is {expected:?}"
    );
}

#[test]
#[serial_test::serial(theme)]
fn mnu_001_a_menu_with_a_named_look_takes_its_colors_from_that_preset() {
    let _theme = Active::set(probe());
    let mut wrong = Vec::new();
    for (look, preset) in [
        (MenuTheme::Dark, dark_theme()),
        (MenuTheme::Light, light_theme()),
        (MenuTheme::HighContrast, high_contrast_theme()),
    ] {
        let frames = app_input::run_when(
            Control(page(
                Element::typed::<PopupMenu>(PopupMenuProps {
                    visible: true,
                    style: look.to_style(),
                    items: vec![MenuItem::new("new", "NEW"), MenuItem::new("open", "OPEN")],
                    placement: PopupPlacement::Position { x: 4, y: 2 },
                    ..Default::default()
                })
                .auto_focus(),
            )),
            (60, 20),
            vec![("OPEN", None)],
        );
        let frame = frames.last().unwrap();
        let of_preset = |name: &str| {
            preset.resolve_variable(name).map(|(r, g, b, _)| {
                let byte = |value: f32| (value * 255.0).round() as u8;
                [byte(r), byte(g), byte(b)]
            })
        };
        let expected = (
            (of_preset("selection-foreground"), of_preset("selection")),
            (of_preset("foreground"), of_preset("surface")),
        );
        let painted = (colors(frame, "NEW"), colors(frame, "OPEN"));
        if expected.0 .1.is_none() || expected.0 .0.is_none() || painted != expected {
            wrong.push(format!(
                "{look:?}: the current row and a row are painted {painted:?}; the {} preset's roles are {expected:?}",
                preset.name
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "MNU-001: as (glyph, background): {wrong:#?}"
    );
}

// ---------------------------------------------------------------- MNU-002

#[test]
#[serial_test::serial(theme)]
fn mnu_002_a_panel_is_as_wide_as_its_widest_row() {
    let label = format!("WIDE{}END", "x".repeat(73));
    assert_eq!(label.len(), 80);
    let frames = app_input::run_when(
        Control(page(popup(
            vec![
                MenuItem::new("wide", label.clone())
                    .shortcut(MenuShortcut::new("Ctrl+W", vec!["ctrl+w"])),
                MenuItem::new("short", "SHORT"),
            ],
            PopupPlacement::Position { x: 4, y: 2 },
        ))),
        (240, 60),
        vec![("SHORT", None)],
    );
    let frame = frames.last().unwrap();
    let (row_of_label, row_of_shortcut) = (
        find(frame, &label).map(|(_, row)| row),
        find(frame, "Ctrl+W").map(|(_, row)| row),
    );
    assert!(
        row_of_label.is_some() && row_of_label == row_of_shortcut,
        "MNU-002: a row of 80 cells and its shortcut are not painted whole on one row of a viewport of 240 columns (label on row {row_of_label:?}, shortcut on row {row_of_shortcut:?}):\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn mnu_002_a_panel_shows_every_row_the_viewport_holds() {
    let frames = app_input::run_when(
        Control(page(popup(
            rows(30),
            PopupPlacement::Position { x: 4, y: 2 },
        ))),
        (240, 60),
        vec![("ROW01", None)],
    );
    let frame = frames.last().unwrap();
    let missing: Vec<String> = (1..=30)
        .map(|number| format!("ROW{number:02}"))
        .filter(|label| find(frame, label).is_none())
        .collect();
    assert!(
        missing.is_empty(),
        "MNU-002: a menu of 30 rows in a viewport of 60 rows leaves {} unpainted: {missing:?}",
        missing.len()
    );
}

#[test]
#[serial_test::serial(theme)]
fn mnu_002_a_panel_taller_than_the_viewport_scrolls_to_its_current_row() {
    let frames = app_input::run_when(
        Control(page(
            popup(rows(30), PopupPlacement::Position { x: 4, y: 0 }).auto_focus(),
        )),
        (60, 20),
        vec![("ROW01", key(KeyCode::End)), ("ROW30", None)],
    );
    let frame = frames.last().unwrap();
    let (top, bottom) = (find(frame, "┌"), find(frame, "└"));
    let shown = (1..=30)
        .filter(|number| find(frame, &format!("ROW{number:02}")).is_some())
        .count();
    assert!(
        top.is_some() && bottom.is_some() && shown == 18,
        "MNU-002: in a viewport of 20 rows the panel's border is at {top:?} and {bottom:?} and it shows {shown} rows, not the 18 that fit between them:\n{}",
        frame.text
    );
}

// ---------------------------------------------------------------- MNU-003

#[test]
#[serial_test::serial(theme)]
fn mnu_003_a_menu_bar_at_the_lower_edge_opens_its_panel_over_its_title() {
    let bar = Element::typed::<MenuBar>(MenuBarProps {
        items: vec![MenuItem::submenu("file", "FILE", rows(5))],
        ..Default::default()
    })
    .auto_focus();
    let tree = builder::div()
        .class("flex flex-col w-full h-full bg-background text-foreground")
        .child(builder::div().class("grow").build())
        .child(bar)
        .build();
    let frames = app_input::run_when(
        Control(tree),
        (60, 40),
        vec![("FILE", key(KeyCode::Down)), ("ROW05", None)],
    );
    let frame = frames.last().unwrap();
    let (title, last) = (find(frame, "FILE"), find(frame, "ROW05"));
    assert!(
        title.is_some_and(|(_, title)| title >= 36)
            && last.zip(title).is_some_and(|((_, last), (_, title))| last < title),
        "MNU-003: the title is at {title:?} and the panel's last row at {last:?}; the panel must stand over the title and leave it painted:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn mnu_003_a_submenu_at_the_right_edge_opens_to_the_left_of_its_parent() {
    let frames = app_input::run_when(
        Control(page(
            popup(
                vec![MenuItem::submenu(
                    "more",
                    "MORE",
                    vec![MenuItem::new("child", "CHILD")],
                )],
                PopupPlacement::Position { x: 46, y: 2 },
            )
            .auto_focus(),
        )),
        (60, 20),
        vec![("MORE", key(KeyCode::Right)), ("CHILD", None)],
    );
    let frame = frames.last().unwrap();
    let (parent, child) = (find(frame, "MORE"), find(frame, "CHILD"));
    assert!(
        parent
            .zip(child)
            .is_some_and(|((parent, _), (child, _))| child + 5 <= parent),
        "MNU-003: the parent row is at {parent:?} and the submenu's row at {child:?}; the submenu must stand left of its parent and leave it painted:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn mnu_003_a_popup_under_an_anchor_at_the_lower_edge_opens_over_it() {
    for (placement, anchor) in [
        (
            PopupPlacement::Widget {
                x: 10,
                y: 19,
                width: 8,
                height: 1,
            },
            19,
        ),
        // `Below` names the row under its anchor: the anchor is the row
        // over it.
        (
            PopupPlacement::Below {
                x: 10,
                y: 19,
                width: 8,
            },
            18,
        ),
    ] {
        let frames = app_input::run_when(
            Control(page(popup(rows(2), placement.clone()))),
            (60, 20),
            vec![("ROW02", None)],
        );
        let frame = frames.last().unwrap();
        let on_anchor: String = (10..18)
            .filter_map(|column| frame.screen.cell(anchor, column))
            .map(|cell| cell.contents().to_owned())
            .collect();
        let (first, last) = (find(frame, "┌"), find(frame, "└"));
        assert!(
            on_anchor.trim().is_empty()
                && first.is_some()
                && last.is_some_and(|(_, row)| row + 1 == anchor),
            "MNU-003: {placement:?}: the anchor's cells on row {anchor} hold {on_anchor:?} and the panel stands from {first:?} to {last:?}; it must stand over the anchor and touch it:\n{}",
            frame.text
        );
    }
}

#[test]
#[serial_test::serial(theme)]
fn mnu_003_a_context_menu_opens_at_the_pointer() {
    let frames = app_input::run_when(
        Control(page(Element::typed::<ContextMenu>(ContextMenuProps {
            items: rows(2),
            ..Default::default()
        }))),
        (60, 20),
        vec![("", right_click(20, 5)), ("ROW02", None)],
    );
    let frame = frames.last().unwrap();
    assert_eq!(
        find(frame, "┌"),
        Some((20, 5)),
        "MNU-003: the panel's first corner after a right click at (20, 5):\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn mnu_003_a_panel_opened_near_the_corner_of_the_viewport_is_painted_whole() {
    let frames = app_input::run_when(
        Control(page(popup(
            rows(2),
            PopupPlacement::Position { x: 57, y: 18 },
        ))),
        (60, 20),
        vec![("ROW02", None)],
    );
    let frame = frames.last().unwrap();
    let first = find(frame, "┌");
    assert_eq!(
        first.map(|first| missing(frame, first)),
        Some(Vec::new()),
        "MNU-003: what is not painted of a panel opened at (57, 18) in a viewport of 60 by 20 cells (its first corner is at {first:?}):\n{}",
        frame.text
    );
}

/// The last step of a run that ends when a panel of two rows stands whole
/// with its first corner at (`column`, `row`): its lower left corner is
/// painted last, three rows under the first.
fn whole_panel_at(column: u16, row: u16) -> Until {
    Until {
        text: "ROW02",
        cell: Some((column, row + 3, "└")),
        event: None,
    }
}

/// How long a test waits for a panel that may never be painted: the App
/// paints a frame in milliseconds, so five seconds tell a missing panel
/// from a slow machine.
const PANEL_WAIT: std::time::Duration = std::time::Duration::from_secs(5);

#[test]
#[serial_test::serial(theme)]
fn mnu_003_shift_f10_opens_the_context_menu_at_the_area_it_serves() {
    let menu = Element::typed::<ContextMenu>(ContextMenuProps {
        items: rows(2),
        ..Default::default()
    })
    .class("absolute left-4 top-2 w-20 h-8")
    .auto_focus();
    let shift_f10 = Event::Key(KeyEvent::new(KeyCode::F(10)).with_modifiers(KeyModifiers {
        shift: true,
        ctrl: false,
        alt: false,
        meta: false,
    }));
    // The run ends at the first frame that shows the panel's last row, and
    // fails after `PANEL_WAIT` when none does.
    let frames = app_input::run_until(
        Control(page(menu)),
        (60, 20),
        vec![
            Until {
                text: "",
                cell: None,
                event: Some(shift_f10),
            },
            whole_panel_at(4, 2),
        ],
        PANEL_WAIT,
    );
    let frame = frames.last().unwrap();
    assert_eq!(
        (find(frame, "┌"), find(frame, "ROW02").is_some()),
        (Some((4, 2)), true),
        "MNU-003: after Shift+F10 the panel's first corner and whether its rows are painted; the context menu serves the area from (4, 2):\n{}",
        frame.text
    );
}

// ---------------------------------------------------------------- MNU-004

/// What is missing of a panel of two rows whose first corner is `first`:
/// its four corners, the top and bottom edges between them, and its rows.
fn missing(frame: &Snapshot, first: (u16, u16)) -> Vec<String> {
    let glyph = |column: u16, row: u16| {
        frame
            .screen
            .cell(row, column)
            .map(|cell| cell.contents().to_owned())
            .unwrap_or_default()
    };
    let (columns, left, top) = (frame.screen.size().1, first.0, first.1);
    let mut missing = Vec::new();
    if glyph(left, top) != "┌" {
        missing.push(format!("the first corner at {first:?}"));
    }
    let right = (left + 1..columns).find(|column| glyph(*column, top) != "─");
    match right {
        Some(right) if glyph(right, top) == "┐" && right > left + 1 => {
            for (what, column, row) in [
                ("the lower left corner", left, top + 3),
                ("the lower right corner", right, top + 3),
            ] {
                if !["└", "┘"].contains(&glyph(column, row).as_str()) {
                    missing.push(format!("{what} at ({column}, {row})"));
                }
            }
            for (label, row) in [("ROW01", top + 1), ("ROW02", top + 2)] {
                let found = find(frame, label);
                if !found.is_some_and(|(column, at)| at == row && column > left && column < right) {
                    missing.push(format!("{label} on row {row} (found at {found:?})"));
                }
            }
        }
        _ => missing.push(format!("the top edge from {first:?} to a right corner")),
    }
    missing
}

/// A body of one line and a popup menu whose panel opens at (40, 10) of
/// the screen: inside the box that holds the body, and past its right and
/// lower edge.
fn body_with_menu() -> Element {
    builder::div()
        .class("w-full h-full")
        .child(Element::text("BODY"))
        .child(popup(rows(2), PopupPlacement::Position { x: 40, y: 10 }))
        .build()
}

#[test]
#[serial_test::serial(theme)]
fn mnu_004_a_menu_opened_inside_a_modal_is_painted_over_it() {
    let modal = builder::modal()
        .title("MODAL")
        .content(body_with_menu())
        .size(30, 8)
        .visible(true)
        .build();
    let frames = app_input::run_until(
        Control(page(modal)),
        (60, 20),
        vec![whole_panel_at(40, 10)],
        PANEL_WAIT,
    );
    let frame = frames.last().unwrap();
    assert_eq!(
        missing(frame, (40, 10)),
        Vec::<String>::new(),
        "MNU-004: what is not painted of a panel at (40, 10) that was opened inside a modal of 30 by 8 cells:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn mnu_004_a_menu_opened_inside_a_popover_is_painted_over_it() {
    let popover = builder::popover()
        .trigger(builder::button().text("OPEN").class("w-6 h-1 p-0").build())
        .content(body_with_menu())
        .build();
    let frames = app_input::run_until(
        Control(page(popover)),
        (60, 20),
        vec![
            Until {
                text: "OPEN",
                cell: None,
                event: app_input::click(1, 0),
            },
            whole_panel_at(40, 10),
        ],
        PANEL_WAIT,
    );
    let frame = frames.last().unwrap();
    assert_eq!(
        missing(frame, (40, 10)),
        Vec::<String>::new(),
        "MNU-004: what is not painted of a panel at (40, 10) that was opened inside a popover:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn mnu_004_a_box_that_clips_its_content_does_not_clip_a_panel() {
    let header = builder::div()
        .class("w-full h-3 overflow-hidden")
        .child(
            Element::typed::<MenuBar>(MenuBarProps {
                items: vec![MenuItem::submenu("file", "FILE", rows(2))],
                ..Default::default()
            })
            .auto_focus(),
        )
        .build();
    let tree = builder::div()
        .class("flex flex-col w-full h-full bg-background text-foreground")
        .child(header)
        .child(Element::text("PAGE"))
        .build();
    let frames = app_input::run_until(
        Control(tree),
        (60, 20),
        vec![
            Until {
                text: "FILE",
                cell: None,
                event: key(KeyCode::Down),
            },
            whole_panel_at(1, 2),
        ],
        PANEL_WAIT,
    );
    let frame = frames.last().unwrap();
    let rows: Vec<bool> = ["ROW01", "ROW02"]
        .iter()
        .map(|label| find(frame, label).is_some())
        .collect();
    assert_eq!(
        rows,
        [true, true],
        "MNU-004: whether each row of a panel is painted when its menu bar stands in a box of three rows that clips its content:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn mnu_004_an_element_no_ancestor_clips_is_painted_and_clicked_past_the_box_that_holds_it() {
    let calls = std::sync::Arc::new(std::sync::Mutex::new(0));
    let tree = |free: bool| {
        let calls = calls.clone();
        let style = StyleBuilder::new()
            .position_absolute()
            .inset_left(0.0)
            .inset_top(0.0)
            .width_px(6.0)
            .height_px(1.0);
        let holder = builder::div()
            .styles(if free { style.unclipped() } else { style })
            .child(
                builder::button()
                    .text("abcdef")
                    .class("w-6 h-1 p-0")
                    .on_click(move || *calls.lock().unwrap() += 1)
                    .build(),
            )
            .build();
        builder::div()
            .class("relative w-full h-full")
            .child(
                builder::div()
                    .class("absolute left-0 top-0 w-3 h-1 overflow-hidden")
                    .child(holder)
                    .build(),
            )
            .build()
    };
    let steps = || vec![(1, app_input::click(5, 0)), (1, None)];
    let clipped = app_input::run(Control(tree(false)), (12, 3), steps());
    assert_eq!(
        (
            clipped.last().unwrap().text.trim().to_owned(),
            *calls.lock().unwrap()
        ),
        ("abc".to_owned(), 0),
        "a box of three cells that clips its content shows three cells of its child and takes no click past them"
    );
    let free = app_input::run(Control(tree(true)), (12, 3), steps());
    assert_eq!(
        (
            free.last().unwrap().text.trim().to_owned(),
            *calls.lock().unwrap()
        ),
        ("abcdef".to_owned(), 1),
        "MNU-004: what is painted of an element that no ancestor clips, and the clicks that reached it at (5, 0)"
    );
}

#[test]
#[serial_test::serial(theme)]
fn mnu_004_a_submenu_is_painted_over_its_parent_panel() {
    // In 24 columns neither side of the parent holds the submenu, so the
    // two panels share cells.
    let frames = app_input::run_when(
        Control(page(
            popup(
                vec![MenuItem::submenu(
                    "more",
                    "MORE-THAN-TWELVE",
                    vec![MenuItem::new("child", "CHILD-OF-MORE")],
                )],
                PopupPlacement::Position { x: 0, y: 2 },
            )
            .auto_focus(),
        )),
        (24, 12),
        vec![("MORE-THAN-TWELVE", key(KeyCode::Right)), ("CHILD", None)],
    );
    let frame = frames.last().unwrap();
    assert!(
        find(frame, "CHILD-OF-MORE").is_some(),
        "MNU-004: the submenu's row is not painted whole over its parent panel:\n{}",
        frame.text
    );
}
