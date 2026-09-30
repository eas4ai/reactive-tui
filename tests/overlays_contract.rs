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

#[test]
#[serial_test::serial(theme)]
fn ovl_001_a_popover_box_is_surface_with_a_border_and_its_arrow_a_piece_of_it() {
    let _theme = Active::set(probe());
    let frame = app_input::run_until(
        Control(page(popover_page())),
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
                event: None,
            },
        ],
        WAIT,
    )
    .pop()
    .unwrap();
    let (text, fill) = colors(&frame, "INNER");
    // INNER holds the focus, so its box (with one cell of padding) is in
    // `selection`; the cell before that, the popover's padding, is the box.
    let (column, row) = find(&frame, "INNER").unwrap();
    assert_eq!(
        (text, fill, background(&frame, column - 2, row)),
        (
            Some(role("selection-foreground")),
            Some(role("selection")),
            Some(role("surface"))
        ),
        "OVL-001: the box in `surface`:\n{}",
        frame.text
    );
    let (border, _) = colors(&frame, "┌");
    assert_eq!(
        border,
        Some(role("border")),
        "OVL-001: the border in `border`:\n{}",
        frame.text
    );
    let (arrow, arrow_fill) = colors(&frame, "▲");
    assert_eq!(
        (arrow, arrow_fill),
        (Some(role("border")), Some(role("surface"))),
        "OVL-001: the arrow is a piece of the box, filled in `surface` with its outline in `border`:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn ovl_001_a_retry_button_and_a_button_of_a_look_without_it_take_a_role() {
    use reactive_tui::{
        core::geometry::Rect,
        widgets::dialog::{
            ConfirmationButtons, ConfirmationDialog, ConfirmationDialogOptions, DialogComponent,
            DialogId, DialogTheme,
        },
    };
    let _theme = Active::set(probe());
    // Retry is a warning button, which the default theme's map does not
    // name; a theme whose map is empty names nothing.
    for (name, theme) in [
        ("the default theme", DialogTheme::default()),
        (
            "a theme with no button looks",
            DialogTheme {
                button_styles: Default::default(),
                ..DialogTheme::default()
            },
        ),
    ] {
        let frame = shown(
            ConfirmationDialog::new(
                DialogId::from_u32(4),
                ConfirmationDialogOptions {
                    title: "T".into(),
                    message: "Try again?".into(),
                    buttons: ConfirmationButtons::RetryCancel,
                    default_button: Some("cancel".into()),
                    ..Default::default()
                },
            )
            .render(Rect::default(), &theme),
            (80, 24),
            "Try again?",
        );
        let (text, fill) = colors(&frame, "Retry");
        let (column, row) = find(&frame, "Retry").unwrap();
        assert_eq!(
            (text, fill, background(&frame, column - 1, row)),
            (
                Some(role("warning-foreground")),
                Some(role("warning")),
                Some(role("warning"))
            ),
            "OVL-001: under {name} the Retry button is in `warning` with one cell of padding:\n{}",
            frame.text
        );
    }
}

// ---------------------------------------------------------------- OVL-002

#[test]
#[serial_test::serial(theme)]
fn ovl_002_a_dialog_is_as_wide_as_its_message_with_one_cell_of_padding() {
    let _theme = Active::set(probe());
    // 30 cells of message: the box is 30 + 2 of padding + 2 of border.
    let frame = shown(
        builder::confirmation_dialog()
            .title("T")
            .message("abcdefghij abcdefghij abcdefgh")
            .confirm_text("Y")
            .cancel_text("N")
            .build(),
        (240, 60),
        "abcdefghij",
    );
    let (left, top) = find(&frame, "┌").unwrap();
    let right = (left..240)
        .find(|column| frame.screen.cell(top, *column).unwrap().contents() == "┐")
        .unwrap();
    assert_eq!(
        right - left + 1,
        34,
        "OVL-002: a 30-cell message makes a box of 34 cells:\n{}",
        frame.text
    );
    let (column, row) = find(&frame, "abcdefghij").unwrap();
    assert_eq!(
        (
            column - left,
            frame.screen.cell(row, left + 1).unwrap().contents()
        ),
        (2, " "),
        "OVL-002: one cell of padding between the border and the message:\n{}",
        frame.text
    );
}

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

#[test]
#[serial_test::serial(theme)]
fn ovl_002_a_popover_beside_its_trigger_keeps_one_cell_from_it() {
    use reactive_tui::widgets::display::popover::{Popover, PopoverPosition, PopoverProps};
    let _theme = Active::set(probe());
    for (position, text) in [
        (PopoverPosition::Right, "RIGHT"),
        (PopoverPosition::Left, "LEFT"),
        (PopoverPosition::Bottom, "BELOW"),
        (PopoverPosition::Top, "ABOVE"),
    ] {
        let frame = shown(
            builder::div()
                .class("relative w-full h-full")
                .child(
                    Element::typed::<Popover>(PopoverProps {
                        visible: true,
                        position,
                        trigger_element: focusable("OPEN"),
                        content: Element::text(text),
                        ..Default::default()
                    })
                    .class("absolute left-20 top-8"),
                )
                .build(),
            (60, 20),
            text,
        );
        let (open, row) = find(&frame, "OPEN").unwrap();
        // The trigger is " OPEN " with its own padding: cells 19 to 24.
        let (trigger_left, trigger_right) = (open - 1, open + 4);
        let (left, top) = find(&frame, "┌").unwrap();
        let (right, bottom) = find(&frame, "┘").unwrap();
        let (arrow, arrow_row) = find(&frame, "▶")
            .or_else(|| find(&frame, "◀"))
            .or_else(|| find(&frame, "▲"))
            .or_else(|| find(&frame, "▼"))
            .unwrap_or_else(|| panic!("no arrow:\n{}", frame.text));
        let (gap, arrow_in_gap, centered) = match position {
            PopoverPosition::Right => (
                left - trigger_right - 1,
                arrow == trigger_right + 1 && arrow_row == row,
                top <= row && row <= bottom,
            ),
            PopoverPosition::Left => (
                trigger_left - right - 1,
                arrow == trigger_left - 1 && arrow_row == row,
                top <= row && row <= bottom,
            ),
            PopoverPosition::Bottom => (
                top - row - 1,
                arrow_row == row + 1 && left <= arrow && arrow <= right,
                left <= open && open <= right,
            ),
            _ => (
                row - bottom - 1,
                arrow_row == row - 1 && left <= arrow && arrow <= right,
                left <= open && open <= right,
            ),
        };
        assert_eq!(
            (gap, arrow_in_gap, centered),
            (1, true, true),
            "OVL-002: a popover {position:?} its trigger with one cell between, the arrow in it, the box centered on the trigger:\n{}",
            frame.text
        );
    }
}

#[test]
#[serial_test::serial(theme)]
fn ovl_002_a_dialog_with_a_field_is_as_wide_as_its_field_or_its_prompt_needs() {
    use reactive_tui::{
        core::geometry::Rect,
        widgets::dialog::{
            DialogComponent, DialogId, DialogTheme, InputDialog, InputDialogOptions,
        },
    };
    let _theme = Active::set(probe());
    // The field is 36 cells: a short prompt makes a box of 40; a prompt of
    // 64 cells makes a box of 68, wider than the field needs.
    for (prompt, expected) in [
        ("Capture name", 40),
        (
            "A prompt of sixty cells that is wider than the field it asks for",
            68,
        ),
    ] {
        let frame = shown(
            InputDialog::new(
                DialogId::from_u32(7),
                InputDialogOptions {
                    title: "Input".into(),
                    prompt: prompt.into(),
                    ..Default::default()
                },
            )
            .render(Rect::default(), &DialogTheme::default()),
            (240, 60),
            "Input",
        );
        let (left, top) = find(&frame, "┌").unwrap();
        let right = (left..240)
            .find(|column| frame.screen.cell(top, *column).unwrap().contents() == "┐")
            .unwrap();
        assert_eq!(
            right - left + 1,
            expected,
            "OVL-002: the box of an input dialog with the prompt {prompt:?}:\n{}",
            frame.text
        );
    }
}

// ---------------------------------------------------------------- OVL-003

/// A box of three rows that clips its content, with `child` inside it.
fn clipping_box(child: Element) -> Element {
    builder::div()
        .class("relative w-full h-3 overflow-hidden")
        .child(child)
        .build()
}

#[test]
#[serial_test::serial(theme)]
fn ovl_003_a_modal_inside_a_clipping_box_is_centered_on_the_screen_and_painted_whole() {
    let _theme = Active::set(probe());
    let size = (240, 60);
    let frame = shown(
        clipping_box(
            builder::modal()
                .title("Modal")
                .content(Element::text("CLIPPED?"))
                .visible(true)
                .build(),
        ),
        size,
        "CLIPPED?",
    );
    let (left, top) = find(&frame, "┌").unwrap_or_else(|| panic!("no box:\n{}", frame.text));
    let (right, bottom) = find(&frame, "┘").unwrap();
    // The box is 12 by 4 (border, title, message, border), centered on the
    // 240 by 60 screen: not in the three rows of the box that clips.
    assert_eq!(
        (left, top, right, bottom),
        (114, 28, 125, 31),
        "OVL-003: a modal inside a clipping box is centered on the screen:\n{}",
        frame.text
    );
    let (column, row) = find(&frame, "CLIPPED?").unwrap();
    assert_eq!(
        background(&frame, column, row),
        Some(role("surface")),
        "OVL-003: every cell of the box is painted, also the rows past the clipping box"
    );
}

#[test]
#[serial_test::serial(theme)]
fn ovl_003_a_popover_inside_a_clipping_box_is_painted_whole() {
    let _theme = Active::set(probe());
    let frame = app_input::run_until(
        Control(page(clipping_box(
            builder::popover()
                .trigger(focusable("OPEN").auto_focus())
                .content(
                    builder::div()
                        .class("flex-col")
                        .child(Element::text("ROW ONE"))
                        .child(Element::text("ROW TWO"))
                        .child(Element::text("ROW THREE"))
                        .build(),
                )
                .build(),
        ))),
        (60, 20),
        vec![
            Until {
                text: "OPEN",
                cell: None,
                event: key(KeyCode::Enter),
            },
            Until {
                text: "ROW THREE",
                cell: None,
                event: None,
            },
        ],
        WAIT,
    )
    .pop()
    .unwrap();
    // The trigger is on row 0 of a box of three rows; the popover opens on
    // row 2 with its arrow and paints its five rows (border, three rows,
    // border) past the box's edge.
    let rows: Vec<Option<u16>> = ["ROW ONE", "ROW TWO", "ROW THREE", "└"]
        .iter()
        .map(|text| find(&frame, text).map(|(_, row)| row))
        .collect();
    assert_eq!(
        rows,
        vec![Some(3), Some(4), Some(5), Some(6)],
        "OVL-003: a popover inside a clipping box is painted whole:\n{}",
        frame.text
    );
}

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

/// A page with a popover whose trigger holds the focus and whose content
/// holds a button; both show in `selection` while focused.
fn popover_page() -> Element {
    builder::popover()
        .trigger(focusable("OPEN").auto_focus())
        .content(focusable("INNER"))
        .build()
}

#[test]
#[serial_test::serial(theme)]
fn ovl_004_a_popover_opened_by_a_key_takes_the_focus_and_gives_it_back() {
    let _theme = Active::set(probe());
    let opened = app_input::run_until(
        Control(page(popover_page())),
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
                event: None,
            },
        ],
        WAIT,
    )
    .pop()
    .unwrap();
    let (_, inner) = colors(&opened, "INNER");
    assert_eq!(
        inner,
        Some(role("selection")),
        "OVL-004: after Enter on the trigger the focus is inside the popover:\n{}",
        opened.text
    );
    let closed = app_input::run_until(
        Control(page(popover_page())),
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
            // The box's first corner is gone once the popover has closed.
            Until {
                text: "OPEN",
                cell: Some((0, 2, " ")),
                event: None,
            },
        ],
        WAIT,
    )
    .pop()
    .unwrap();
    let (_, open) = colors(&closed, "OPEN");
    assert_eq!(
        open,
        Some(role("selection")),
        "OVL-004: after Escape the focus is back on the trigger:\n{}",
        closed.text
    );
}

// ---------------------------------------------------------------- BAR-003

/// The veil of the probe theme: a color and how much of it is laid over
/// what is under it.
const OVERLAY: (Rgb, f32) = ([200, 20, 50], 0.5);

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

fn near(got: Rgb, expected: Rgb) -> bool {
    got.iter().zip(expected).all(|(g, e)| g.abs_diff(e) <= 2)
}

/// Whether `color` is a role of the probe theme or its veil laid over one.
fn of_the_theme(color: Option<Rgb>) -> bool {
    let roles = || (0..ROLES.len()).map(probe_color);
    color.is_some_and(|color| {
        roles().any(|role| role == color || near(color, laid_over(OVERLAY, role)))
    })
}

/// The colors in `frame` that the probe theme does not define, each with
/// the first cell that has it. The rows `skip` names are left out.
fn foreign(frame: &Snapshot, skip: &[u16]) -> Vec<String> {
    let (rows, columns) = frame.screen.size();
    let mut found: Vec<(vt100::Color, String)> = Vec::new();
    for (row, column) in (0..rows)
        .filter(|row| !skip.contains(row))
        .flat_map(|row| (0..columns).map(move |column| (row, column)))
    {
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

/// Each overlay as the widget catalog builds it, with the text that shows
/// it is open and the text of the row a widget of another family paints
/// (a text field), which the color check leaves out. The progress bar
/// inside the progress dialog is told its roles by the dialog.
fn overlays() -> Vec<(&'static str, Element, &'static str, Option<&'static str>)> {
    use reactive_tui::{
        builder::specialized::WizardStep,
        core::geometry::Rect,
        widgets::{
            dialog::{
                AutocompleteConfig, AutocompleteDialog, AutocompleteDialogOptions, DialogComponent,
                DialogId, DialogTheme, InputDialog, InputDialogOptions,
            },
            display::popover::{Popover, PopoverProps},
        },
    };
    vec![
        (
            "modal",
            builder::modal()
                .title("Modal")
                .content(Element::text("Focused overlay"))
                .visible(true)
                .build(),
            "Focused overlay",
            None,
        ),
        (
            "popover",
            Element::typed::<Popover>(PopoverProps {
                visible: true,
                trigger_element: focusable("OPEN").auto_focus(),
                content: Element::text("Popover content"),
                ..Default::default()
            }),
            "Popover content",
            None,
        ),
        (
            "confirmation dialog",
            builder::confirmation_dialog()
                .title("Confirm")
                .message("Ready to record?")
                .build(),
            "Ready to record?",
            None,
        ),
        (
            "input dialog",
            InputDialog::new(
                DialogId::from_u32(7),
                InputDialogOptions {
                    title: "Input".into(),
                    prompt: "Capture name".into(),
                    ..Default::default()
                },
            )
            .render(Rect::default(), &DialogTheme::default()),
            "Capture name",
            Some("["),
        ),
        (
            "autocomplete dialog",
            AutocompleteDialog::new(
                DialogId::from_u32(8),
                AutocompleteDialogOptions {
                    title: "Autocomplete".into(),
                    prompt: "Find a widget".into(),
                    autocomplete: AutocompleteConfig {
                        min_chars: 0,
                        static_suggestions: vec!["Accordion".into(), "Checkbox".into()],
                        debounce_delay: std::time::Duration::ZERO,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .render(Rect::default(), &DialogTheme::default()),
            "Checkbox",
            Some("["),
        ),
        (
            "progress dialog",
            builder::progress_dialog()
                .title("Progress")
                .message("Rendering")
                .progress(0.64)
                .build(),
            "64.0%",
            None,
        ),
        (
            "toast",
            builder::toast()
                .success("Capture saved")
                .persistent()
                .build(),
            "Capture saved",
            None,
        ),
        (
            "wizard dialog",
            builder::wizard()
                .title("Wizard")
                .step(WizardStep::new("Compose").content(Element::text("Choose widgets")))
                .step(WizardStep::new("Capture").content(Element::text("Record clip")))
                .build(),
            "Choose widgets",
            Some("█"),
        ),
    ]
}

#[test]
#[serial_test::serial(theme)]
fn bar_003_an_overlay_takes_every_color_from_the_active_theme() {
    let _theme = Active::set(probe());
    let mut wrong = Vec::new();
    for (name, root, text, other_family) in overlays() {
        let frame = shown(root, (80, 24), text);
        // A text field inside a dialog is painted by its own family, which
        // a later commitment brings to the widget bar.
        let skip: Vec<u16> = other_family
            .and_then(|glyph| find(&frame, glyph))
            .map(|(_, row)| vec![row])
            .unwrap_or_default();
        let foreign = foreign(&frame, &skip);
        if !foreign.is_empty() {
            wrong.push(format!("{name}: {foreign:#?}\n{}", frame.text));
        }
    }
    assert!(
        wrong.is_empty(),
        "BAR-003: an overlay paints a color the theme does not define:\n{}",
        wrong.join("\n")
    );
}

#[test]
#[serial_test::serial(theme)]
fn bar_003_a_dialog_follows_a_resize() {
    use reactive_tui::event::types::ResizeEvent;
    let _theme = Active::set(probe());
    let frames = app_input::run_until(
        Control(page(
            builder::confirmation_dialog()
                .title("Confirm")
                .message("Ready to record?")
                .build(),
        )),
        (80, 24),
        vec![
            Until {
                text: "Ready to record?",
                cell: None,
                event: Some(Event::Resize(ResizeEvent::new(160, 48))),
            },
            // Centered in 160 by 48: the box, 20 by 5, starts at (70, 21).
            Until {
                text: "Ready to record?",
                cell: Some((70, 21, "┌")),
                event: None,
            },
        ],
        WAIT,
    );
    let frame = frames.last().unwrap();
    assert_eq!(
        frame.screen.size(),
        (48, 160),
        "BAR-003: the frame follows the resize"
    );
    assert!(
        find(frame, "┌") == Some((70, 21)),
        "BAR-003: the box is centered on the resized screen:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn bar_003_every_pointer_action_of_an_overlay_has_a_key() {
    let _theme = Active::set(probe());
    // Escape closes a modal.
    let closed = app_input::run_until(
        Control(page(
            builder::modal()
                .title("Modal")
                .content(Element::text("Focused overlay"))
                .visible(true)
                .build(),
        )),
        (80, 24),
        vec![
            Until {
                text: "Focused overlay",
                cell: None,
                event: key(KeyCode::Escape),
            },
            Until {
                text: "",
                cell: Some((23, 10, " ")),
                event: None,
            },
        ],
        WAIT,
    );
    assert!(
        find(closed.last().unwrap(), "Focused overlay").is_none(),
        "BAR-003: Escape closes the modal:\n{}",
        closed.last().unwrap().text
    );
    // Tab moves between a dialog's buttons and Enter presses the focused
    // one: OK holds the focus first, Tab moves it to Cancel, Enter cancels.
    use reactive_tui::{
        core::geometry::Rect,
        widgets::dialog::{
            ConfirmationDialog, ConfirmationDialogOptions, DialogComponent, DialogId, DialogResult,
            DialogTheme,
        },
    };
    let results = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let noted = results.clone();
    let frames = app_input::run_until(
        Control(page(
            ConfirmationDialog::new(
                DialogId::from_u32(3),
                ConfirmationDialogOptions {
                    title: "Confirm".into(),
                    message: "Ready to record?".into(),
                    on_close: Some(std::sync::Arc::new(move |result| {
                        noted.lock().unwrap().push(result)
                    })),
                    ..Default::default()
                },
            )
            .render(Rect::default(), &DialogTheme::default()),
        )),
        (80, 24),
        vec![
            Until {
                text: "Ready to record?",
                cell: None,
                event: key(KeyCode::Tab),
            },
            Until {
                text: "Ready to record?",
                cell: None,
                event: key(KeyCode::Enter),
            },
            Until {
                text: "",
                cell: Some((30, 10, " ")),
                event: None,
            },
        ],
        WAIT,
    );
    assert!(
        find(frames.last().unwrap(), "Ready to record?").is_none(),
        "BAR-003: Enter presses the focused button and the dialog closes:\n{}",
        frames.last().unwrap().text
    );
    assert!(
        matches!(
            results.lock().unwrap().as_slice(),
            [DialogResult::Cancelled]
        ),
        "BAR-003: Tab moved the focus from OK to Cancel before Enter: {:?}",
        results.lock().unwrap()
    );
    // Escape closes a closable toast.
    let frames = app_input::run_until(
        Control(page(
            builder::toast()
                .success("Capture saved")
                .persistent()
                .closable(true)
                .build()
                .auto_focus(),
        )),
        (80, 24),
        vec![
            Until {
                text: "Capture saved",
                cell: None,
                event: key(KeyCode::Escape),
            },
            Until {
                text: "",
                cell: Some((60, 2, " ")),
                event: None,
            },
        ],
        WAIT,
    );
    assert!(
        find(frames.last().unwrap(), "Capture saved").is_none(),
        "BAR-003: Escape closes a closable toast:\n{}",
        frames.last().unwrap().text
    );
}

// ---------------------------------------------------------------- BAR-005

/// The most work the App did for one of `frames` frames, in ms, and each
/// frame's work and present time.
fn max_work_ms(root: Element, size: (u16, u16), frames: usize) -> (f64, String) {
    let out = app_input::run_on_debug(Control(page(root)), size, vec![(frames, None)]);
    assert!(
        out.len() >= frames,
        "harness painted {} of {frames} frames",
        out.len()
    );
    let split: Vec<String> = out
        .iter()
        .map(|f| format!("{:.2}/{:.2}", f.work_ms, f.present_ms))
        .collect();
    (
        out.iter().map(|f| f.work_ms).fold(0.0, f64::max),
        split.join(" "),
    )
}

/// BAR-005: a dialog fading in and a progress dialog whose bar moves keep
/// the App's work per frame under 16.6 ms at 700 by 200.
#[test]
#[serial_test::serial(theme)]
fn bar_005_an_animating_overlay_stays_under_the_frame_budget_at_700_by_200() {
    use reactive_tui::widgets::dialog::{
        ConfirmationDialogOptions, DialogEngine, DialogEngineConfig,
    };
    if cfg!(debug_assertions) {
        // The App's per-element cost is about ten times higher without
        // optimization, so the budget is only meaningful on the optimized
        // build, which is what the frame-budget mechanism runs.
        eprintln!("SKIP: the frame budget is measured on the optimized build");
        let (_, split) = max_work_ms(
            builder::progress_dialog()
                .title("Progress")
                .message("Rendering")
                .indeterminate(true)
                .build(),
            (80, 24),
            3,
        );
        assert!(!split.is_empty());
        return;
    }
    let size = (700u16, 200u16);
    let mut over = Vec::new();
    // A dialog that fades in over two seconds: every measured frame is a
    // frame of the fade.
    let mut engine = DialogEngine::with_config(DialogEngineConfig {
        animation_duration: std::time::Duration::from_secs(2),
        ..Default::default()
    });
    engine.show_confirmation(ConfirmationDialogOptions {
        title: "Confirm".into(),
        message: "Ready to record?".into(),
        ..Default::default()
    });
    for (name, root) in [
        ("a dialog fading in", engine.render()),
        (
            "a progress dialog whose bar moves",
            builder::progress_dialog()
                .title("Progress")
                .message("Rendering")
                .indeterminate(true)
                .build(),
        ),
    ] {
        let (ms, split) = max_work_ms(root, size, 12);
        eprintln!("{name}: work/present ms per frame at 700x200: {split}");
        if ms >= 16.6 {
            over.push(format!("{name}: {ms:.2} ms ({split})"));
        }
    }
    assert!(
        over.is_empty(),
        "BAR-005: per-frame work exceeds 16.6 ms at 700x200: {}",
        over.join("; ")
    );
}
