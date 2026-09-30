//! OVL-001 to OVL-004 (docs/spec/overlays.md): the colors of the modal,
//! the popover, the toast and the five dialogs by role, their size in
//! cells, where each is placed and what it is painted over, and where the
//! focus goes. What each tells the screen reader is a unit test beside
//! its code (`cargo test --lib ovl_004_`).
//!
//! Every test takes its turn (`serial(theme)`): the color tests set the
//! active theme, which is one for the process, and the others must not
//! paint under it.

mod common;

use std::time::Duration;

use common::app_input::{self, Snapshot, Until};
use reactive_tui::{
    app::RootComponent,
    builder,
    component::{Element, FocusProps},
    event::types::{Event, KeyCode, KeyEvent},
    theme::{Theme, ThemeVariables},
};

type Rgb = [u8; 3];

/// How long a frame may take to settle before a step gives up.
const WAIT: Duration = Duration::from_secs(5);

struct Control(Element);
impl RootComponent for Control {
    fn render(&self) -> Element {
        self.0.clone()
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

/// The roles an overlay paints with.
const ROLES: [&str; 18] = [
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
    "secondary",
    "secondary-foreground",
    "error",
    "error-foreground",
    "warning",
    "warning-foreground",
    "success",
    "success-foreground",
];

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

fn hex(color: Rgb) -> String {
    let [r, g, b] = color;
    format!("#{r:02x}{g:02x}{b:02x}")
}

/// A theme whose roles all differ, so a cell's color names its role.
fn probe() -> Theme {
    let mut variables = ThemeVariables::new()
        .set("--color-overlay", "#c8143280")
        .set("--color-shadow", "#1432c866");
    for (index, name) in ROLES.iter().enumerate() {
        variables = variables.set(Theme::color_variable(name), hex(probe_color(index)));
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

/// A page in the theme's background that holds `child`.
fn page(child: Element) -> Element {
    builder::div()
        .class("relative w-full h-full bg-background text-foreground")
        .child(child)
        .build()
}

/// The last frame of `root` at `size` once `text` is painted and the App
/// is idle.
fn shown(root: Element, size: (u16, u16), text: &'static str) -> Snapshot {
    app_input::run_until(
        Control(page(root)),
        size,
        vec![Until {
            text,
            cell: None,
            event: None,
        }],
        WAIT,
    )
    .pop()
    .unwrap()
}

/// The widest run of cells on any row whose background is `color`.
fn widest_run(frame: &Snapshot, color: Rgb) -> u16 {
    let (rows, columns) = frame.screen.size();
    (0..rows)
        .map(|row| {
            let mut widest = 0;
            let mut run = 0;
            for column in 0..columns {
                if background(frame, column, row) == Some(color) {
                    run += 1;
                    widest = widest.max(run);
                } else {
                    run = 0;
                }
            }
            widest
        })
        .max()
        .unwrap_or(0)
}

// ---------------------------------------------------------------- OVL-001

#[test]
#[serial_test::serial(theme)]
fn ovl_001_a_confirmation_dialog_paints_its_box_and_buttons_in_the_roles() {
    let _theme = Active::set(probe());
    let frame = shown(
        builder::confirmation_dialog()
            .title("TITLE")
            .message("MESSAGE")
            .confirm_text("YES")
            .cancel_text("NO")
            .build(),
        (60, 20),
        "MESSAGE",
    );
    let (text, box_color) = colors(&frame, "MESSAGE");
    assert_eq!(
        (text, box_color),
        (Some(role("foreground")), Some(role("surface"))),
        "OVL-001: the message in `foreground` on a box in `surface`:\n{}",
        frame.text
    );
    let (_, yes) = colors(&frame, "YES");
    let (_, no) = colors(&frame, "NO");
    assert!(
        yes == Some(role("primary")) || yes == Some(role("selection")),
        "OVL-001: the primary button in `primary`, or `selection` while it holds the focus, not {yes:?}:\n{}",
        frame.text
    );
    assert!(
        no == Some(role("secondary")) || no == Some(role("selection")),
        "OVL-001: the other button in `secondary`, or `selection` while it holds the focus, not {no:?}:\n{}",
        frame.text
    );
}

// ---------------------------------------------------------------- OVL-002

#[test]
#[serial_test::serial(theme)]
fn ovl_002_a_dialog_with_a_long_message_is_at_most_half_the_viewport_wide() {
    let _theme = Active::set(probe());
    let size = (240, 60);
    let words: Vec<String> = (1..=40).map(|number| format!("word{number:02}")).collect();
    // 40 words of 6 cells and 39 spaces: 279 cells on one line.
    let message = words.join(" ");
    let frame = shown(
        builder::confirmation_dialog()
            .title("TITLE")
            .message(&message)
            .build(),
        size,
        "word01",
    );
    let (_, box_color) = colors(&frame, "word01");
    let box_color = box_color.expect("the box has a background");
    let widest = widest_run(&frame, box_color);
    assert!(
        widest <= size.0 / 2,
        "OVL-002: the box is {widest} cells wide, more than half of {} cells:\n{}",
        size.0,
        frame.text
    );
    let missing: Vec<&String> = words
        .iter()
        .filter(|word| find(&frame, word).is_none())
        .collect();
    assert!(
        missing.is_empty(),
        "OVL-002: the message wraps, so every word is painted; missing {missing:?}:\n{}",
        frame.text
    );
}

// ---------------------------------------------------------------- OVL-003

#[test]
#[serial_test::serial(theme)]
fn ovl_003_a_toast_in_a_corner_keeps_one_cell_from_the_edge() {
    let _theme = Active::set(probe());
    let size = (80, 24);
    let frame = shown(
        builder::toast()
            .success("SAVED")
            .position("top-right")
            .duration(60_000)
            .build(),
        size,
        "SAVED",
    );
    let (column, row) = find(&frame, "SAVED").unwrap();
    let box_color = background(&frame, column, row).expect("the toast has a background");
    let last_column = (column..size.0)
        .take_while(|column| background(&frame, *column, row) == Some(box_color))
        .last()
        .unwrap();
    let first_row = (0..=row)
        .rev()
        .take_while(|row| background(&frame, column, *row) == Some(box_color))
        .last()
        .unwrap();
    assert!(
        first_row >= 1 && last_column <= size.0 - 2,
        "OVL-003: a toast at the top right has a cell between it and each edge; it starts on row {first_row} and ends in column {last_column}:\n{}",
        frame.text
    );
}

// ---------------------------------------------------------------- OVL-004

fn key(code: KeyCode) -> Option<Event> {
    Some(Event::Key(KeyEvent::new(code)))
}

/// A focusable box of text that shows in `selection` while it holds the
/// focus.
fn focusable(text: &str) -> Element {
    builder::div()
        .class("px-1 focus:bg-selection focus:text-selection-foreground")
        .child(Element::text(text))
        .build()
        .with_focus(FocusProps::button())
}

#[test]
#[serial_test::serial(theme)]
fn ovl_004_a_popover_opened_by_a_key_takes_the_focus_and_gives_it_back() {
    let _theme = Active::set(probe());
    let frames = app_input::run_until(
        Control(page(
            builder::popover()
                .trigger(focusable("OPEN").auto_focus())
                .content(focusable("INNER"))
                .build(),
        )),
        (60, 20),
        vec![
            Until {
                text: "OPEN",
                cell: None,
                event: key(KeyCode::Enter),
            },
            Until {
                text: "INNER",
                cell: None,
                event: key(KeyCode::Escape),
            },
            Until {
                text: "OPEN",
                cell: None,
                event: None,
            },
        ],
        WAIT,
    );
    let opened = frames
        .iter()
        .rev()
        .find(|frame| find(frame, "INNER").is_some())
        .expect("the popover opened");
    let (_, inner) = colors(opened, "INNER");
    assert_eq!(
        inner,
        Some(role("selection")),
        "OVL-004: after Enter on the trigger the focus is inside the popover:\n{}",
        opened.text
    );
    let closed = frames.last().unwrap();
    let (_, open) = colors(closed, "OPEN");
    assert_eq!(
        open,
        Some(role("selection")),
        "OVL-004: after Escape the focus is back on the trigger:\n{}",
        closed.text
    );
}
