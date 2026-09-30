//! The input widgets' contract (docs/spec/input-widgets.md, CTL-001 to
//! CTL-004): the text input, the checkbox, the radio button, the select,
//! the slider and the button under a theme whose roles all differ, so a
//! cell's color names its role; the width a field, a row and a track fill;
//! where a select's list opens and what it is painted over; and the keys
//! behind every pointer action. What each control tells the screen reader
//! is read by the `ctl_004_` unit tests beside the widgets.

mod common;

use std::time::Duration;

use common::app_input::{self, Snapshot, Until};
use reactive_tui::{
    app::RootComponent,
    builder,
    component::Element,
    event::types::{Event, KeyCode},
    theme::{Theme, ThemeVariables},
    widgets::input::{Checkbox, CheckboxProps, Slider, SliderProps},
};

type Rgb = [u8; 3];

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

const ROLES: [&str; 25] = [
    "background",
    "surface",
    "foreground",
    "text-muted",
    "border",
    "input",
    "ring",
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
    "info",
    "info-foreground",
    "accent",
    "accent-foreground",
    "overlay",
];

fn probe_color(index: usize) -> Rgb {
    [
        30 + 8 * index as u8,
        200 - 6 * index as u8,
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
    let mut variables = ThemeVariables::new();
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

fn at(frame: &Snapshot, needle: &str) -> (u16, u16) {
    find(frame, needle).unwrap_or_else(|| panic!("{needle:?} is not painted:\n{}", frame.text))
}

/// The glyph color and the background of the cell at (column, row).
fn cell_colors(frame: &Snapshot, column: u16, row: u16) -> (Option<Rgb>, Option<Rgb>) {
    let cell = frame.screen.cell(row, column).unwrap();
    (rgb(cell.fgcolor()), rgb(cell.bgcolor()))
}

/// The glyph color and the background of the cell where `needle` starts.
fn colors(frame: &Snapshot, needle: &str) -> (Option<Rgb>, Option<Rgb>) {
    let (column, row) = at(frame, needle);
    cell_colors(frame, column, row)
}

/// The glyph in the cell at (column, row).
fn glyph(frame: &Snapshot, column: u16, row: u16) -> String {
    frame
        .screen
        .cell(row, column)
        .map(|cell| cell.contents().to_string())
        .unwrap_or_default()
}

/// The last column of `row` that holds a glyph other than a space.
fn last_glyph_column(frame: &Snapshot, row: u16) -> Option<u16> {
    let (_, columns) = frame.screen.size();
    (0..columns)
        .rev()
        .find(|column| !glyph(frame, *column, row).trim().is_empty())
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

/// Every frame `root` presents at `size` through `steps`: each step waits
/// until its text is painted and the App is idle, then sends its event;
/// the frames are picked by content.
fn shown_after(
    root: Element,
    size: (u16, u16),
    steps: Vec<(&'static str, Option<Event>)>,
) -> Vec<Snapshot> {
    let until = steps
        .into_iter()
        .map(|(text, event)| Until {
            text,
            cell: None,
            event,
        })
        .collect();
    app_input::run_until(Control(page(root)), size, until, WAIT)
}

/// The first frame that paints `needle`.
fn first_with<'a>(frames: &'a [Snapshot], needle: &str) -> &'a Snapshot {
    frames
        .iter()
        .find(|frame| find(frame, needle).is_some())
        .unwrap_or_else(|| panic!("no frame paints {needle:?}"))
}

fn last(frames: &[Snapshot]) -> &Snapshot {
    frames.last().expect("a frame")
}

fn key(code: KeyCode) -> Option<Event> {
    app_input::key(code)
}

/// A box of `width` cells and `height` rows on the page.
fn boxed(width: u16, height: u16, child: Element) -> Element {
    builder::div()
        .class(&format!("flex-col w-{width} h-{height}"))
        .child(child)
        .build()
}

// ---------------------------------------------------------------- CTL-001

#[test]
#[serial_test::serial(theme)]
fn ctl_001_a_focused_checkbox_paints_its_frame_in_ring_and_its_mark_in_primary() {
    let _theme = Active::set(probe());
    let frame = shown(
        builder::checkbox()
            .label("Capture-ready")
            .checked(true)
            .build()
            .auto_focus(),
        (80, 6),
        "Capture-ready",
    );
    let (column, row) = at(&frame, "Capture-ready");
    let (frame_color, _) = colors(&frame, "[");
    assert_eq!(
        frame_color,
        Some(role("ring")),
        "the frame of a focused box is ring:\n{}",
        frame.text
    );
    let (mark, _) = colors(&frame, "✓");
    assert_eq!(
        mark,
        Some(role("primary")),
        "the mark is primary:\n{}",
        frame.text
    );
    assert_eq!(
        cell_colors(&frame, column, row).0,
        Some(role("foreground")),
        "the label is foreground:\n{}",
        frame.text
    );
    assert!(
        find(&frame, "▶").is_none(),
        "focus adds no glyph:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn ctl_001_a_text_input_paints_its_field_in_input_and_its_cursor_reversed() {
    let _theme = Active::set(probe());
    let frame = shown(
        builder::div()
            .class("flex-col gap-1")
            .child(builder::text_input().value("Reactive").build().auto_focus())
            .child(builder::text_input().placeholder("Type here").build())
            .build(),
        (80, 8),
        "Type here",
    );
    let (column, row) = at(&frame, "Reactive");
    // The cursor stands on the first glyph until the user moves it.
    assert_eq!(
        cell_colors(&frame, column + 1, row),
        (Some(role("foreground")), Some(role("input"))),
        "the field is input with foreground text:\n{}",
        frame.text
    );
    assert_eq!(
        cell_colors(&frame, column, row),
        (Some(role("input")), Some(role("foreground"))),
        "the cursor cell is the field reversed:\n{}",
        frame.text
    );
    assert_eq!(
        colors(&frame, "Type here").0,
        Some(role("text-muted")),
        "the placeholder is text-muted:\n{}",
        frame.text
    );
    let (frame_color, _) = cell_colors(&frame, column - 1, row);
    assert_eq!(
        frame_color,
        Some(role("ring")),
        "the focused field's frame is ring:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn ctl_001_a_button_takes_the_primary_or_the_secondary_look() {
    let _theme = Active::set(probe());
    let frame = shown(
        builder::div()
            .class("flex-row gap-1")
            .child(builder::primary_button("Save", || {}))
            .child(builder::button().text("Cancel").on_click(|| {}).build())
            .build(),
        (80, 24),
        "Cancel",
    );
    assert_eq!(
        colors(&frame, "Save"),
        (Some(role("primary-foreground")), Some(role("primary"))),
        "the primary button:\n{}",
        frame.text
    );
    assert_eq!(
        colors(&frame, "Cancel"),
        (Some(role("secondary-foreground")), Some(role("secondary"))),
        "the secondary button:\n{}",
        frame.text
    );
    let (column, row) = at(&frame, "Save");
    assert_eq!(
        cell_colors(&frame, column - 1, row).1,
        Some(role("primary")),
        "one cell of padding before the label:\n{}",
        frame.text
    );
    assert_eq!(
        cell_colors(&frame, column + 4, row).1,
        Some(role("primary")),
        "one cell of padding after the label:\n{}",
        frame.text
    );
    assert_ne!(
        cell_colors(&frame, column + 5, row).1,
        Some(role("primary")),
        "one cell of padding, not more:\n{}",
        frame.text
    );
    assert_eq!(
        cell_colors(&frame, column, row + 1).1,
        Some(role("background")),
        "no row of padding below:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn ctl_001_a_slider_paints_its_track_fill_and_thumb_in_the_roles() {
    let _theme = Active::set(probe());
    let frame = shown(
        builder::slider()
            .label("Intensity")
            .min(0.0)
            .max(100.0)
            .step(5.0)
            .value(50.0)
            .build(),
        (80, 6),
        "Intensity",
    );
    let (start, row) = at(&frame, "[");
    let end = at(&frame, "]").0;
    let thumb = (start + 1..end)
        .find(|column| glyph(&frame, *column, row) == "●")
        .unwrap_or_else(|| {
            panic!(
                "a thumb between {start} and {end} on row {row}:\n{}",
                frame.text
            )
        });
    assert_eq!(
        cell_colors(&frame, start + 1, row).0,
        Some(role("primary")),
        "the filled part is primary:\n{}",
        frame.text
    );
    assert_eq!(
        cell_colors(&frame, end - 1, row).0,
        Some(role("border")),
        "the empty part is border:\n{}",
        frame.text
    );
    assert_eq!(
        cell_colors(&frame, thumb, row).0,
        Some(role("foreground")),
        "the thumb is foreground:\n{}",
        frame.text
    );
    assert_eq!(
        colors(&frame, "Intensity").0,
        Some(role("foreground")),
        "the label is foreground:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn ctl_001_a_select_paints_its_field_and_its_open_list_in_the_roles() {
    let _theme = Active::set(probe());
    let frames = shown_after(
        builder::select()
            .option("cyan", "Cyan")
            .option("violet", "Violet")
            .selected("cyan")
            .build()
            .auto_focus(),
        (80, 10),
        vec![("Cyan", key(KeyCode::Enter)), ("Violet", None)],
    );
    let closed = first_with(&frames, "Cyan");
    assert_eq!(
        colors(closed, "Cyan"),
        (Some(role("foreground")), Some(role("input"))),
        "the closed field is input with foreground text:\n{}",
        closed.text
    );
    assert_eq!(
        colors(closed, "▾").0,
        Some(role("text-muted")),
        "the caret is text-muted:\n{}",
        closed.text
    );
    let open = last(&frames);
    assert_eq!(
        colors(open, "Violet").1,
        Some(role("surface")),
        "a row of the list is surface:\n{}",
        open.text
    );
    let (field_column, field_row) = at(open, "Cyan");
    let (column, current) = (field_row + 1..field_row + 5)
        .find_map(|r| {
            (0..field_column + 6)
                .find(|c| glyph(open, *c, r) == "C" && glyph(open, c + 1, r) == "y")
                .map(|c| (c, r))
        })
        .unwrap_or_else(|| panic!("the list's Cyan row:\n{}", open.text));
    assert_eq!(
        cell_colors(open, column, current),
        (Some(role("selection-foreground")), Some(role("selection"))),
        "the current row of the list is selection:\n{}",
        open.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn ctl_001_a_control_looks_the_same_from_its_props_and_its_builder() {
    let _theme = Active::set(probe());
    let from_builder = shown(
        builder::div()
            .class("flex-col gap-1")
            .child(
                builder::checkbox()
                    .label("Capture-ready")
                    .checked(true)
                    .build(),
            )
            .child(
                builder::slider()
                    .min(0.0)
                    .max(100.0)
                    .step(5.0)
                    .value(50.0)
                    .build(),
            )
            .build(),
        (80, 6),
        "Capture-ready",
    );
    let from_props = shown(
        builder::div()
            .class("flex-col gap-1")
            .child(Element::typed::<Checkbox>(CheckboxProps {
                checked: true,
                label: Some("Capture-ready".into()),
                ..Default::default()
            }))
            .child(Element::typed::<Slider>(SliderProps {
                min: 0.0,
                max: 100.0,
                step: 5.0,
                value: 50.0,
                ..Default::default()
            }))
            .build(),
        (80, 6),
        "Capture-ready",
    );
    assert_eq!(from_props.text, from_builder.text);
    let (rows, columns) = from_props.screen.size();
    for row in 0..rows {
        for column in 0..columns {
            assert_eq!(
                cell_colors(&from_props, column, row),
                cell_colors(&from_builder, column, row),
                "cell ({column}, {row}) differs between the props and the builder"
            );
        }
    }
}

// ---------------------------------------------------------------- CTL-002

#[test]
#[serial_test::serial(theme)]
fn ctl_002_a_field_a_row_and_a_track_fill_a_box_of_100_cells() {
    let _theme = Active::set(probe());
    let frame = shown(
        boxed(
            100,
            8,
            builder::div()
                .class("flex-col gap-1")
                .child(builder::text_input().value("Reactive").build())
                .child(
                    builder::select()
                        .option("cyan", "Cyan")
                        .option("violet", "Violet")
                        .selected("cyan")
                        .build(),
                )
                .child(
                    builder::slider()
                        .label("Intensity")
                        .min(0.0)
                        .max(100.0)
                        .step(5.0)
                        .value(65.0)
                        .build(),
                )
                .build(),
        ),
        (240, 12),
        "Intensity",
    );
    for (needle, what) in [
        ("Reactive", "the text input's field"),
        ("Cyan", "the select's row"),
        ("65.0", "the slider with its value"),
    ] {
        let (_, row) = at(&frame, needle);
        assert_eq!(
            last_glyph_column(&frame, row),
            Some(99),
            "{what} ends at the box's last cell:\n{}",
            frame.text
        );
    }
}

#[test]
#[serial_test::serial(theme)]
fn ctl_002_a_width_class_on_the_builder_sets_the_width() {
    let _theme = Active::set(probe());
    let frame = shown(
        boxed(
            100,
            4,
            builder::text_input()
                .value("Reactive")
                .class("w-40")
                .build(),
        ),
        (240, 8),
        "Reactive",
    );
    let (_, row) = at(&frame, "Reactive");
    assert_eq!(
        last_glyph_column(&frame, row),
        Some(39),
        "a w-40 field ends at cell 39:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn ctl_002_a_checkbox_is_as_wide_as_its_box_and_its_label() {
    let _theme = Active::set(probe());
    let frame = shown(
        boxed(
            100,
            4,
            builder::checkbox()
                .label("Capture-on")
                .checked(true)
                .build(),
        ),
        (240, 8),
        "Capture-on",
    );
    let (column, row) = at(&frame, "[");
    assert_eq!(
        last_glyph_column(&frame, row),
        Some(column + 3 + 10),
        "a box, a space and a 10-cell label:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn ctl_002_a_select_of_thirty_options_paints_every_row_when_opened() {
    let _theme = Active::set(probe());
    let labels: Vec<(String, String)> = (1..=30)
        .map(|n| (format!("o{n}"), format!("Option {n}")))
        .collect();
    let mut select = builder::select();
    for (value, label) in &labels {
        select = select.option(value, label);
    }
    let frames = shown_after(
        select.selected("o1").build().auto_focus(),
        (240, 60),
        vec![("Option 1", key(KeyCode::Enter)), ("Option 2", None)],
    );
    let open = last(&frames);
    for n in 1..=30 {
        assert!(
            find(open, &format!("Option {n}")).is_some(),
            "Option {n} is painted:\n{}",
            open.text
        );
    }
}

// ---------------------------------------------------------------- CTL-003

#[test]
#[serial_test::serial(theme)]
fn ctl_003_opening_a_select_moves_nothing_under_it() {
    let _theme = Active::set(probe());
    let mut page = builder::div().class("flex-col").child(
        builder::select()
            .option("cyan", "Cyan")
            .option("violet", "Violet")
            .selected("cyan")
            .build()
            .auto_focus(),
    );
    for n in 1..=6 {
        page = page.child(Element::text(format!("LINE {n} UNDER THE SELECT")));
    }
    let frames = shown_after(
        page.build(),
        (80, 12),
        vec![("LINE 6 UNDER", key(KeyCode::Enter)), ("Violet", None)],
    );
    let before = first_with(&frames, "LINE 6 UNDER");
    let open = last(&frames);
    // The panel of two options takes four rows: it covers lines 1 to 4,
    // which are painted over, not moved; lines 5 and 6 keep their rows.
    for n in 1..=4 {
        assert!(
            find(open, &format!("LINE {n} UNDER")).is_none(),
            "line {n} is painted over, not moved:\n{}",
            open.text
        );
    }
    for n in 5..=6 {
        assert_eq!(
            find(open, &format!("LINE {n} UNDER")),
            find(before, &format!("LINE {n} UNDER")),
            "line {n} keeps its row:\n{}",
            open.text
        );
    }
}

#[test]
#[serial_test::serial(theme)]
fn ctl_003_a_select_on_the_last_row_opens_its_list_above() {
    let _theme = Active::set(probe());
    let frames = shown_after(
        builder::div()
            .class("absolute bottom-0 left-0 w-40")
            .child(
                builder::select()
                    .option("cyan", "Cyan")
                    .option("violet", "Violet")
                    .option("amber", "Amber")
                    .selected("cyan")
                    .build()
                    .auto_focus(),
            )
            .build(),
        (80, 12),
        vec![("Cyan", key(KeyCode::Enter)), ("Cyan", None)],
    );
    let open = last(&frames);
    let field_row = 11;
    let (_, amber_row) = at(open, "Amber");
    let (_, violet_row) = at(open, "Violet");
    assert!(
        amber_row < field_row && violet_row < field_row,
        "the list opens above the row:\n{}",
        open.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn ctl_003_a_select_inside_a_clipping_box_paints_its_list_whole() {
    let _theme = Active::set(probe());
    let frames = shown_after(
        builder::div()
            .class("flex-col h-2 overflow-hidden")
            .child(
                builder::select()
                    .option("cyan", "Cyan")
                    .option("violet", "Violet")
                    .option("amber", "Amber")
                    .option("olive", "Olive")
                    .selected("cyan")
                    .build()
                    .auto_focus(),
            )
            .build(),
        (80, 12),
        vec![("Cyan", key(KeyCode::Enter)), ("Cyan", None)],
    );
    let open = last(&frames);
    for label in ["Violet", "Amber", "Olive"] {
        assert!(
            find(open, label).is_some(),
            "{label} is painted outside the clipping box:\n{}",
            open.text
        );
    }
}

#[test]
#[serial_test::serial(theme)]
fn ctl_003_escape_closes_the_list_and_a_choice_shows_at_once() {
    let _theme = Active::set(probe());
    let select = || {
        builder::select()
            .option("cyan", "Cyan")
            .option("violet", "Violet")
            .selected("cyan")
            .build()
            .auto_focus()
    };
    let escaped = shown_after(
        select(),
        (80, 10),
        vec![
            ("Cyan", key(KeyCode::Enter)),
            ("Violet", key(KeyCode::Escape)),
            ("Cyan", None),
        ],
    );
    let closed = last(&escaped);
    assert!(
        find(closed, "Violet").is_none(),
        "Escape closes the list:\n{}",
        closed.text
    );
    let chosen = shown_after(
        select(),
        (80, 10),
        vec![
            ("Cyan", key(KeyCode::Enter)),
            ("Violet", key(KeyCode::Down)),
            ("Violet", key(KeyCode::Enter)),
            ("Violet", None),
        ],
    );
    let chosen = last(&chosen);
    assert!(
        find(chosen, "Cyan").is_none(),
        "the row shows the choice and the list is closed:\n{}",
        chosen.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn ctl_003_a_text_input_suggestion_list_inside_a_clipping_box_paints_whole() {
    use reactive_tui::widgets::input::{Suggestion, TextInput, TextInputProps};
    let _theme = Active::set(probe());
    let suggestion = |text: &str| Suggestion {
        text: text.into(),
        description: None,
        insert_text: text.into(),
    };
    let input = Element::typed::<TextInput>(TextInputProps {
        suggestions: vec![suggestion("Alpha"), suggestion("Amber"), suggestion("Atom")],
        placeholder: Some("Type here".into()),
        ..Default::default()
    })
    .auto_focus();
    let frames = shown_after(
        builder::div()
            .class("flex-col h-2 overflow-hidden")
            .child(input)
            .build(),
        (80, 12),
        vec![("Type here", key(KeyCode::Char('a'))), ("Alpha", None)],
    );
    let open = last(&frames);
    for label in ["Alpha", "Amber", "Atom"] {
        assert!(
            find(open, label).is_some(),
            "{label} is painted outside the clipping box:\n{}",
            open.text
        );
    }
    let (_, row) = at(open, "Atom");
    assert!(
        row >= 3,
        "the suggestions stand under the field, past the box:\n{}",
        open.text
    );
}

// ---------------------------------------------------------------- CTL-004

#[test]
#[serial_test::serial(theme)]
fn ctl_004_a_key_moves_a_slider_and_opens_a_select_as_a_click_does() {
    let _theme = Active::set(probe());
    let frames = shown_after(
        builder::div()
            .class("flex-col gap-1")
            .child(
                builder::slider()
                    .label("Intensity")
                    .min(0.0)
                    .max(100.0)
                    .step(5.0)
                    .value(50.0)
                    .build()
                    .auto_focus(),
            )
            .child(
                builder::select()
                    .option("cyan", "Cyan")
                    .option("violet", "Violet")
                    .selected("cyan")
                    .build(),
            )
            .build(),
        (80, 12),
        vec![
            ("50.0", key(KeyCode::Right)),
            ("55.0", key(KeyCode::End)),
            ("100.0", key(KeyCode::Tab)),
            ("100.0", key(KeyCode::Down)),
            ("Violet", None),
        ],
    );
    let open = last(&frames);
    assert!(
        find(open, "Violet").is_some(),
        "Down opens the select:\n{}",
        open.text
    );
}

// ---------------------------------------------------------------- BAR-003

/// Whether `color` is one the probe theme defines.
fn of_the_theme(color: Option<Rgb>) -> bool {
    color.is_some_and(|color| (0..ROLES.len()).any(|index| probe_color(index) == color))
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
                        "{part} {color:?} at ({column}, {row}) {:?}",
                        cell.contents()
                    ),
                ));
            }
        }
    }
    found.into_iter().map(|(_, where_)| where_).collect()
}

/// Each control as the widget catalog builds it, focused or not, with the
/// text that shows it is painted.
fn controls() -> Vec<(&'static str, Element, &'static str)> {
    let select = || {
        builder::select()
            .option("cyan", "Cyan")
            .option("violet", "Violet")
            .selected("cyan")
            .build()
    };
    vec![
        (
            "text input",
            builder::text_input()
                .value("Reactive TUI")
                .placeholder("Type here")
                .build(),
            "Reactive TUI",
        ),
        (
            "focused text input with an error",
            reactive_tui::widgets::input::TextInputBuilder::new()
                .value("ab1")
                .validator_pattern("alpha")
                .error_message("Letters only")
                .render()
                .auto_focus(),
            "Letters only",
        ),
        (
            "checkbox",
            builder::checkbox()
                .label("Capture-ready")
                .checked(true)
                .build(),
            "Capture-ready",
        ),
        (
            "focused checkbox",
            builder::checkbox()
                .label("Capture-ready")
                .build()
                .auto_focus(),
            "Capture-ready",
        ),
        (
            "disabled checkbox",
            builder::checkbox()
                .label("Locked")
                .checked(true)
                .disabled(true)
                .build(),
            "Locked",
        ),
        (
            "radio buttons",
            builder::div()
                .class("flex-col")
                .child(
                    builder::radio_button()
                        .group("quality")
                        .value("balanced")
                        .label("Balanced")
                        .checked(true)
                        .build()
                        .auto_focus(),
                )
                .child(
                    builder::radio_button()
                        .group("quality")
                        .value("high")
                        .label("High detail")
                        .build(),
                )
                .build(),
            "High detail",
        ),
        ("select", select(), "Cyan"),
        ("focused select", select().auto_focus(), "Cyan"),
        (
            "slider",
            builder::slider()
                .label("Intensity")
                .min(0.0)
                .max(100.0)
                .step(5.0)
                .value(65.0)
                .build(),
            "65.0",
        ),
        (
            "focused slider",
            builder::slider()
                .label("Intensity")
                .value(65.0)
                .build()
                .auto_focus(),
            "65.0",
        ),
        (
            "buttons",
            builder::div()
                .class("flex-row gap-1")
                .child(builder::primary_button("Save", || {}))
                .child(builder::button().text("Cancel").on_click(|| {}).build())
                .build(),
            "Cancel",
        ),
    ]
}

#[test]
#[serial_test::serial(theme)]
fn bar_003_a_control_takes_every_color_from_the_active_theme() {
    let _theme = Active::set(probe());
    let mut wrong = Vec::new();
    for (name, root, text) in controls() {
        let frame = shown(root, (80, 24), text);
        let foreign = foreign(&frame);
        if !foreign.is_empty() {
            wrong.push(format!("{name}: {foreign:#?}\n{}", frame.text));
        }
    }
    // The open list too.
    let frames = shown_after(
        builder::select()
            .option("cyan", "Cyan")
            .option("violet", "Violet")
            .selected("cyan")
            .build()
            .auto_focus(),
        (80, 24),
        vec![("Cyan", key(KeyCode::Enter)), ("Violet", None)],
    );
    let foreign = foreign(last(&frames));
    if !foreign.is_empty() {
        wrong.push(format!("open select: {foreign:#?}\n{}", last(&frames).text));
    }
    assert!(
        wrong.is_empty(),
        "BAR-003: a control paints a color the theme does not define:\n{}",
        wrong.join("\n")
    );
}

#[test]
#[serial_test::serial(theme)]
fn bar_003_a_control_follows_a_resize() {
    use reactive_tui::event::types::ResizeEvent;
    let _theme = Active::set(probe());
    let frames = shown_after(
        builder::div()
            .class("flex-col gap-1 w-full")
            .child(builder::text_input().value("Reactive").build())
            .child(
                builder::select()
                    .option("cyan", "Cyan")
                    .option("violet", "Violet")
                    .selected("cyan")
                    .build(),
            )
            .child(builder::slider().label("Intensity").value(65.0).build())
            .build(),
        (80, 24),
        vec![
            ("Intensity", Some(Event::Resize(ResizeEvent::new(160, 48)))),
            ("Intensity", None),
        ],
    );
    // The settled frame at 80 columns: the last one of that width.
    let before = frames
        .iter()
        .rev()
        .find(|frame| frame.screen.size().1 == 80)
        .expect("a frame at 80 columns");
    let after = last(&frames);
    for needle in ["Reactive", "Cyan", "Intensity"] {
        let (_, row) = at(before, needle);
        assert_eq!(
            last_glyph_column(before, row),
            Some(79),
            "{needle} fills 80 columns:\n{}",
            before.text
        );
        let (_, row) = at(after, needle);
        assert_eq!(
            last_glyph_column(after, row),
            Some(159),
            "{needle} fills 160 columns after the resize:\n{}",
            after.text
        );
    }
}

#[test]
#[serial_test::serial(theme)]
fn bar_003_every_pointer_action_of_a_control_has_a_key() {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    let _theme = Active::set(probe());
    let clicks = Arc::new(AtomicUsize::new(0));
    let counted = clicks.clone();
    let frames = shown_after(
        builder::div()
            .class("flex-col gap-1")
            .child(
                builder::checkbox()
                    .label("Capture-ready")
                    .build()
                    .auto_focus(),
            )
            .child(
                builder::div()
                    .class("flex-col")
                    .child(
                        builder::radio_button()
                            .group("quality")
                            .value("balanced")
                            .label("Balanced")
                            .checked(true)
                            .build(),
                    )
                    .child(
                        builder::radio_button()
                            .group("quality")
                            .value("high")
                            .label("High detail")
                            .build(),
                    )
                    .build(),
            )
            .child(
                builder::select()
                    .option("cyan", "Cyan")
                    .option("violet", "Violet")
                    .selected("cyan")
                    .build(),
            )
            .child(builder::slider().label("Intensity").value(50.0).build())
            .child(
                builder::button()
                    .text("Count")
                    .on_click(move || {
                        counted.fetch_add(1, Ordering::SeqCst);
                    })
                    .build(),
            )
            .build(),
        (80, 24),
        vec![
            // Space checks the box, as a click does.
            ("Capture-ready", key(KeyCode::Char(' '))),
            // Tab, Tab reaches the second radio; Space chooses it.
            ("[✓]", key(KeyCode::Tab)),
            ("[✓]", key(KeyCode::Tab)),
            ("[✓]", key(KeyCode::Char(' '))),
            // Tab reaches the select; Enter opens, Down moves, Enter chooses.
            ("(●) High detail", key(KeyCode::Tab)),
            ("(●) High detail", key(KeyCode::Enter)),
            ("Violet", key(KeyCode::Down)),
            ("Violet", key(KeyCode::Enter)),
            // Tab reaches the slider; End takes it to the maximum.
            ("[Violet", key(KeyCode::Tab)),
            ("[Violet", key(KeyCode::End)),
            // Tab reaches the button; Enter presses it.
            ("100.0", key(KeyCode::Tab)),
            ("100.0", key(KeyCode::Enter)),
            ("100.0", None),
        ],
    );
    let end = last(&frames);
    assert!(
        find(end, "[✓]").is_some(),
        "the box is checked:\n{}",
        end.text
    );
    assert!(
        find(end, "(●) High detail").is_some(),
        "the radio is chosen:\n{}",
        end.text
    );
    assert!(
        find(end, "[Violet").is_some() && find(end, "Cyan").is_none(),
        "the select shows the choice:\n{}",
        end.text
    );
    assert!(
        find(end, "100.0").is_some(),
        "the slider is at its maximum:\n{}",
        end.text
    );
    assert_eq!(
        clicks.load(Ordering::SeqCst),
        1,
        "Enter pressed the button:\n{}",
        end.text
    );
}
