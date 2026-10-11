//! The input pieces' contract (docs/spec/input-widgets.md, CTL-007 to
//! CTL-010): the number input, the code input, the rating, the switch and
//! the form under a theme whose roles all differ, so a cell's color names
//! its role; the width each one fills, takes or gives its form; the keys,
//! the pointer and the paste that change each piece, and the callbacks
//! that report every change; and what the screen reader is told about each
//! of them. The screen-reader checks run on Linux only and sit in a
//! `reader` module. The BAR-003 checks for these pieces are at the end.
//! Each piece's own unit tests sit beside the piece.

mod common;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use common::app_input::{self, FramePredicate, Snapshot, Until};
use reactive_tui::{
    app::RootComponent,
    builder,
    component::Element,
    event::types::{
        Event, KeyCode, MouseEvent, MouseEventKind, PasteEvent, Position, ResizeEvent, WheelDelta,
        WheelEvent, WheelPhase,
    },
    theme::{Theme, ThemeVariables},
    widgets::{
        display::pieces::icon::Icon,
        input::{
            CodeInput, CodeInputProps, Form, FormField, FormLabels, FormProps, NumberInput,
            NumberInputProps, Rating, RatingProps, Switch, SwitchProps,
        },
    },
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

/// A log of the values a callback was called with, in order.
type Log<T> = Arc<Mutex<Vec<T>>>;

/// A closure that records each value it is called with in `log`.
fn recorder<T: Send + 'static>(log: &Log<T>) -> impl Fn(T) + Send + Sync + 'static {
    let log = log.clone();
    move |value| log.lock().unwrap().push(value)
}

/// What a log holds now.
fn held<T: Clone>(log: &Log<T>) -> Vec<T> {
    log.lock().unwrap().clone()
}

/// The cells of `frame` that show `wanted`, as (column, row), row by row.
fn showing(frame: &Snapshot, wanted: &str) -> Vec<(u16, u16)> {
    let (rows, columns) = frame.screen.size();
    (0..rows)
        .flat_map(|row| (0..columns).map(move |column| (column, row)))
        .filter(|(column, row)| glyph(frame, *column, *row) == wanted)
        .collect()
}

/// The column of the first cell on `row` that shows `needle`.
fn column_on(frame: &Snapshot, row: u16, needle: &str) -> u16 {
    let (_, columns) = frame.screen.size();
    (0..columns)
        .find(|column| glyph(frame, *column, row) == needle)
        .unwrap_or_else(|| panic!("{needle:?} is not painted on row {row}:\n{}", frame.text))
}

/// The column of digit cell `index` of a six-digit code input in groups of
/// three, given the column of the first cell's `[`: one gap after the
/// first group.
fn code_cell(first: u16, index: usize) -> u16 {
    first + 3 * index as u16 + if index >= 3 { 1 } else { 0 }
}

/// The field of a number input as (its `[` column, its `]` column, row),
/// read from the `[-]` and `[+]` buttons around it.
fn number_field(frame: &Snapshot) -> (u16, u16, u16) {
    let (minus, row) = at(frame, &format!("[{}]", Icon::Minus.glyph()));
    let (plus, plus_row) = at(frame, &format!("[{}]", Icon::Plus.glyph()));
    assert_eq!(row, plus_row, "the buttons share a row:\n{}", frame.text);
    (minus + 3, plus - 1, row)
}

/// The middle cell of the button `[glyph]`, as (column, row).
fn button_middle(frame: &Snapshot, glyph: &str) -> (u16, u16) {
    let (column, row) = at(frame, &format!("[{glyph}]"));
    (column + 1, row)
}

/// Every cell of the screen as (text, glyph color, background color), for
/// comparing two frames cell by cell.
fn paint(frame: &Snapshot) -> Vec<(String, vt100::Color, vt100::Color)> {
    let (rows, columns) = frame.screen.size();
    (0..rows)
        .flat_map(|row| (0..columns).map(move |column| (row, column)))
        .map(|(row, column)| {
            let cell = frame.screen.cell(row, column).unwrap();
            (cell.contents().to_string(), cell.fgcolor(), cell.bgcolor())
        })
        .collect()
}

/// A typed paste of `text`.
fn paste(text: &str) -> Option<Event> {
    Some(Event::Paste(PasteEvent::new(text.to_owned())))
}

/// The pointer moves onto (x, y) with no button held.
fn move_to(x: u16, y: u16) -> Option<Event> {
    Some(Event::Mouse(MouseEvent::new(
        MouseEventKind::Move,
        Position::cell(x, y),
    )))
}

/// One wheel notch up over (x, y): a negative vertical delta, as the
/// terminal reports it.
fn wheel_up(x: u16, y: u16) -> Option<Event> {
    Some(Event::Mouse(MouseEvent::wheel(
        Position::cell(x, y),
        WheelEvent {
            delta: WheelDelta::Lines { x: 0.0, y: -1.0 },
            phase: WheelPhase::Changed,
        },
    )))
}

/// A frame predicate that holds on a frame whose text contains `text`.
fn when(text: String) -> FramePredicate {
    Box::new(move |frame: &Snapshot| frame.text.contains(text.as_str()))
}

/// Each step's event goes out at the first frame whose text holds the
/// step's text. Each step's text must differ from the one before it, so
/// that an earlier frame cannot pass for the state the step waits for.
fn frames_when(
    root: Element,
    size: (u16, u16),
    steps: Vec<(String, Option<Event>)>,
) -> Vec<Snapshot> {
    app_input::run_when_frame(
        Control(page(root)),
        size,
        steps
            .into_iter()
            .map(|(text, event)| (when(text), event))
            .collect(),
    )
}

/// A number input from 0 to 10 in steps of 0.5, at `value`.
fn number_at(value: f64) -> Element {
    builder::number_input()
        .value(value)
        .min(0.0)
        .max(10.0)
        .step(0.5)
        .build()
}

/// `number_at(value)`, disabled.
fn disabled_number_at(value: f64) -> Element {
    builder::number_input()
        .value(value)
        .min(0.0)
        .max(10.0)
        .step(0.5)
        .disabled(true)
        .build()
}

/// A six-digit code input in groups of three, with `value` typed.
fn code_at(value: &str) -> Element {
    builder::code_input()
        .length(6)
        .group(3)
        .value(value)
        .build()
}

/// A rating of five stars at `value`.
fn rating_at(value: usize) -> Element {
    builder::rating().value(value).max(5).build()
}

/// A switch labelled "Wi-Fi".
fn switch_at(checked: bool) -> Element {
    builder::switch().checked(checked).label("Wi-Fi").build()
}

/// A form of two fields: Name, required, and Email with the description
/// "Work address".
fn form_at() -> Element {
    builder::form()
        .field(FormField::new("Name", builder::text_input().value("Ada").build()).required())
        .field(FormField::new("Email", builder::text_input().build()).description("Work address"))
        .build()
}

// ---------------------------------------------------------------- CTL-007

/// The number input's buttons and field, and the form, the rating, the
/// code input and the switch built through their builders paint their
/// cells in the roles the spec names.
#[test]
#[serial_test::serial(theme)]
fn ctl_007_a_number_input_paints_its_field_and_buttons_by_role() {
    let _theme = Active::set(probe());
    let frame = shown(number_at(2.0), (80, 6), "2.0");
    let (start, end, row) = number_field(&frame);
    for column in [start, end] {
        assert_eq!(
            cell_colors(&frame, column, row).0,
            Some(role("border")),
            "the field's frame is border:\n{}",
            frame.text
        );
    }
    assert_eq!(
        colors(&frame, "2.0"),
        (Some(role("foreground")), Some(role("input"))),
        "the value is foreground on input:\n{}",
        frame.text
    );
    let minus = button_middle(&frame, Icon::Minus.glyph());
    let plus = button_middle(&frame, Icon::Plus.glyph());
    for (column, row) in [minus, plus] {
        assert_eq!(
            cell_colors(&frame, column, row),
            (Some(role("secondary-foreground")), Some(role("secondary"))),
            "a button that can step is secondary-foreground on secondary:\n{}",
            frame.text
        );
        assert_eq!(
            cell_colors(&frame, column - 1, row).1,
            Some(role("secondary")),
            "the button's first cell is on secondary:\n{}",
            frame.text
        );
        assert_eq!(
            cell_colors(&frame, column + 1, row).1,
            Some(role("secondary")),
            "the button's last cell is on secondary:\n{}",
            frame.text
        );
    }

    let frame = shown(number_at(10.0), (80, 6), "10.0");
    let (column, row) = button_middle(&frame, Icon::Plus.glyph());
    assert_eq!(
        cell_colors(&frame, column, row),
        (Some(role("text-muted")), Some(role("secondary"))),
        "+ at the maximum is text-muted on secondary:\n{}",
        frame.text
    );
    let (column, row) = button_middle(&frame, Icon::Minus.glyph());
    assert_eq!(
        cell_colors(&frame, column, row),
        (Some(role("secondary-foreground")), Some(role("secondary"))),
        "- can still step down, so it keeps secondary-foreground:\n{}",
        frame.text
    );

    let frame = shown(number_at(2.0).auto_focus(), (80, 6), "2.0");
    let (start, end, row) = number_field(&frame);
    for column in [start, end] {
        assert_eq!(
            cell_colors(&frame, column, row).0,
            Some(role("ring")),
            "the focused field's frame is ring:\n{}",
            frame.text
        );
    }

    let frame = shown(disabled_number_at(2.0), (80, 6), "2.0");
    for glyph in [Icon::Minus.glyph(), Icon::Plus.glyph()] {
        let (column, row) = button_middle(&frame, glyph);
        assert_eq!(
            cell_colors(&frame, column, row).0,
            Some(role("text-muted")),
            "a disabled number input's buttons are text-muted:\n{}",
            frame.text
        );
    }
}

#[test]
#[serial_test::serial(theme)]
fn ctl_007_a_code_input_paints_its_cells_by_role() {
    let _theme = Active::set(probe());
    let frame = shown(code_at("123"), (80, 6), "[1][2][3]");
    let (first, row) = at(&frame, "[");
    for (index, digit) in ["1", "2", "3"].into_iter().enumerate() {
        let column = code_cell(first, index) + 1;
        assert_eq!(
            glyph(&frame, column, row),
            digit,
            "digit {index} is painted:\n{}",
            frame.text
        );
        assert_eq!(
            cell_colors(&frame, column, row),
            (Some(role("foreground")), Some(role("input"))),
            "a digit is foreground on input:\n{}",
            frame.text
        );
    }
    let column = code_cell(first, 3) + 1;
    assert_eq!(
        glyph(&frame, column, row),
        "_",
        "an empty cell:\n{}",
        frame.text
    );
    assert_eq!(
        cell_colors(&frame, column, row),
        (Some(role("text-muted")), Some(role("input"))),
        "the empty cell's _ is text-muted on input:\n{}",
        frame.text
    );
    for wanted in ["[", "]"] {
        let cells = showing(&frame, wanted);
        assert_eq!(cells.len(), 6, "six {wanted} brackets:\n{}", frame.text);
        for (column, row) in cells {
            assert_eq!(
                cell_colors(&frame, column, row).0,
                Some(role("border")),
                "the brackets are border:\n{}",
                frame.text
            );
        }
    }

    let frame = shown(code_at("123").auto_focus(), (80, 6), "[1][2][3]");
    let (first, _) = at(&frame, "[");
    let current = code_cell(first, 3);
    for (column, row) in showing(&frame, "[").into_iter().chain(showing(&frame, "]")) {
        let expected = if column == current || column == current + 2 {
            "ring"
        } else {
            "border"
        };
        assert_eq!(
            cell_colors(&frame, column, row).0,
            Some(role(expected)),
            "the current cell's brackets are ring, the others border:\n{}",
            frame.text
        );
    }

    let frame = shown(
        builder::code_input()
            .length(6)
            .group(3)
            .value("123")
            .masked(true)
            .build(),
        (80, 6),
        "[_]",
    );
    let (first, row) = at(&frame, "[");
    for index in 0..3 {
        let column = code_cell(first, index) + 1;
        assert_eq!(
            glyph(&frame, column, row),
            Icon::Dot.glyph(),
            "a masked digit is a dot:\n{}",
            frame.text
        );
        assert_eq!(
            cell_colors(&frame, column, row).0,
            Some(role("foreground")),
            "the dot is foreground:\n{}",
            frame.text
        );
    }
    assert!(
        !frame.text.chars().any(|c| c.is_ascii_digit()),
        "a masked code input shows no digit:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn ctl_007_a_rating_paints_lit_and_unlit_stars_by_role() {
    let _theme = Active::set(probe());
    let frame = shown(rating_at(3), (80, 6), "★★★");
    let lit = showing(&frame, Icon::Star.glyph());
    let unlit = showing(&frame, Icon::StarOutline.glyph());
    assert_eq!(lit.len(), 3, "three lit stars:\n{}", frame.text);
    assert_eq!(unlit.len(), 2, "two unlit stars:\n{}", frame.text);
    for (column, row) in lit {
        assert_eq!(
            cell_colors(&frame, column, row).0,
            Some(role("warning")),
            "a lit star is text-warning:\n{}",
            frame.text
        );
    }
    for (column, row) in unlit {
        assert_eq!(
            cell_colors(&frame, column, row).0,
            Some(role("border")),
            "an unlit star is border:\n{}",
            frame.text
        );
    }

    let frame = shown(rating_at(3).auto_focus(), (80, 6), "★★★");
    for (column, row) in showing(&frame, Icon::StarOutline.glyph()) {
        assert_eq!(
            cell_colors(&frame, column, row).0,
            Some(role("ring")),
            "the unlit stars are ring while the rating has the focus:\n{}",
            frame.text
        );
    }
    for (column, row) in showing(&frame, Icon::Star.glyph()) {
        assert_eq!(
            cell_colors(&frame, column, row).0,
            Some(role("warning")),
            "a lit star stays text-warning while focused:\n{}",
            frame.text
        );
    }

    let frame = shown(
        builder::rating().value(3).max(5).disabled(true).build(),
        (80, 6),
        "★★★",
    );
    let stars = showing(&frame, Icon::Star.glyph())
        .into_iter()
        .chain(showing(&frame, Icon::StarOutline.glyph()));
    let mut count = 0;
    for (column, row) in stars {
        count += 1;
        assert_eq!(
            cell_colors(&frame, column, row).0,
            Some(role("text-muted")),
            "every star is text-muted while disabled:\n{}",
            frame.text
        );
    }
    assert_eq!(count, 5, "five stars:\n{}", frame.text);
}

#[test]
#[serial_test::serial(theme)]
fn ctl_007_a_switch_paints_its_box_thumb_and_label_by_role() {
    let _theme = Active::set(probe());
    let frame = shown(switch_at(true), (80, 6), "Wi-Fi");
    let (open, row) = at(&frame, "[");
    let close = open + 3;
    assert_eq!(
        glyph(&frame, close, row),
        "]",
        "the box closes:\n{}",
        frame.text
    );
    assert_eq!(
        glyph(&frame, open + 1, row),
        " ",
        "on: [ ●]:\n{}",
        frame.text
    );
    assert_eq!(
        glyph(&frame, open + 2, row),
        Icon::Dot.glyph(),
        "on: the thumb is the second cell:\n{}",
        frame.text
    );
    assert_eq!(
        cell_colors(&frame, open, row).0,
        Some(role("border")),
        "the open bracket is border:\n{}",
        frame.text
    );
    assert_eq!(
        cell_colors(&frame, close, row).0,
        Some(role("border")),
        "the close bracket is border:\n{}",
        frame.text
    );
    assert_eq!(
        cell_colors(&frame, open + 2, row).0,
        Some(role("primary")),
        "the thumb is primary when on:\n{}",
        frame.text
    );
    assert_eq!(
        colors(&frame, "Wi-Fi").0,
        Some(role("foreground")),
        "the label is foreground:\n{}",
        frame.text
    );

    let frame = shown(switch_at(false), (80, 6), "Wi-Fi");
    let (open, row) = at(&frame, "[");
    assert_eq!(
        glyph(&frame, open + 1, row),
        Icon::Dot.glyph(),
        "off: [● ], the thumb is the first cell:\n{}",
        frame.text
    );
    assert_eq!(
        glyph(&frame, open + 2, row),
        " ",
        "off: [● ]:\n{}",
        frame.text
    );
    assert_eq!(
        cell_colors(&frame, open + 1, row).0,
        Some(role("text-muted")),
        "the thumb is text-muted when off:\n{}",
        frame.text
    );

    let frame = shown(switch_at(true).auto_focus(), (80, 6), "Wi-Fi");
    let (open, row) = at(&frame, "[");
    assert_eq!(
        cell_colors(&frame, open, row).0,
        Some(role("ring")),
        "the open bracket is ring while focused:\n{}",
        frame.text
    );

    let frame = shown(
        builder::switch()
            .checked(true)
            .label("Wi-Fi")
            .disabled(true)
            .build(),
        (80, 6),
        "Wi-Fi",
    );
    assert_eq!(
        colors(&frame, "Wi-Fi").0,
        Some(role("text-muted")),
        "a disabled label is text-muted:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn ctl_007_a_form_paints_labels_marks_and_descriptions_by_role() {
    let _theme = Active::set(probe());
    let frame = shown(form_at(), (80, 8), "Work address");
    let (column, row) = at(&frame, "Name");
    assert_eq!(
        cell_colors(&frame, column, row).0,
        Some(role("foreground")),
        "a label is foreground:\n{}",
        frame.text
    );
    assert_eq!(
        glyph(&frame, column + 4, row),
        "*",
        "the required mark follows the label:\n{}",
        frame.text
    );
    assert_eq!(
        cell_colors(&frame, column + 4, row).0,
        Some(role("error")),
        "the required mark is text-error:\n{}",
        frame.text
    );
    assert_eq!(
        colors(&frame, "Work address").0,
        Some(role("text-muted")),
        "a description is text-muted:\n{}",
        frame.text
    );
}

/// Each of the five pieces, built from its props and through its builder
/// with the same settings, and a text that shows it.
fn twins() -> Vec<(&'static str, Element, Element, &'static str)> {
    vec![
        (
            "number input",
            Element::typed::<NumberInput>(NumberInputProps {
                value: 2.0,
                min: Some(0.0),
                max: Some(10.0),
                step: 0.5,
                aria_label: None,
                disabled: false,
                width: None,
            }),
            number_at(2.0),
            "2.0",
        ),
        (
            "code input",
            Element::typed::<CodeInput>(CodeInputProps {
                length: 6,
                group: Some(3),
                value: "123".to_owned(),
                masked: false,
                label: None,
                aria_label: None,
                disabled: false,
            }),
            code_at("123"),
            "[1][2][3]",
        ),
        (
            "rating",
            Element::typed::<Rating>(RatingProps {
                value: 3,
                max: 5,
                aria_label: None,
                disabled: false,
            }),
            rating_at(3),
            "★★★",
        ),
        (
            "switch",
            Element::typed::<Switch>(SwitchProps {
                checked: true,
                label: Some("Wi-Fi".to_owned()),
                aria_label: None,
                disabled: false,
            }),
            switch_at(true),
            "Wi-Fi",
        ),
        (
            "form",
            Element::typed::<Form>(FormProps {
                fields: vec![
                    FormField::new("Name", builder::text_input().value("Ada").build()).required(),
                ],
                labels: FormLabels::Beside,
                label_width: None,
                aria_label: None,
            }),
            builder::form()
                .field(
                    FormField::new("Name", builder::text_input().value("Ada").build()).required(),
                )
                .build(),
            "Name",
        ),
    ]
}

#[test]
#[serial_test::serial(theme)]
fn ctl_007_a_piece_built_through_its_builder_paints_the_same_cells_as_from_its_props() {
    let _theme = Active::set(probe());
    for (name, from_props, from_builder, text) in twins() {
        let props = shown(from_props, (80, 6), text);
        let built = shown(from_builder, (80, 6), text);
        assert!(
            paint(&props) == paint(&built),
            "CTL-007: the {name} built through its builder paints other cells than its props:\n{}\n---\n{}",
            props.text,
            built.text
        );
    }
}

#[test]
#[serial_test::serial(theme)]
fn ctl_007_focus_hover_and_disabled_add_no_glyph() {
    let _theme = Active::set(probe());
    let number = shown(number_at(2.0), (80, 6), "2.0");
    assert_eq!(
        shown(number_at(2.0).auto_focus(), (80, 6), "2.0").text,
        number.text,
        "CTL-007: focus adds no glyph to a number input"
    );
    assert_eq!(
        shown(disabled_number_at(2.0), (80, 6), "2.0").text,
        number.text,
        "CTL-007: disabled changes no text of a number input"
    );

    let code = shown(code_at("123"), (80, 6), "[1][2][3]");
    assert_eq!(
        shown(code_at("123").auto_focus(), (80, 6), "[1][2][3]").text,
        code.text,
        "CTL-007: focus adds no glyph to a code input"
    );
    assert_eq!(
        shown(
            builder::code_input()
                .length(6)
                .group(3)
                .value("123")
                .disabled(true)
                .build(),
            (80, 6),
            "[1][2][3]"
        )
        .text,
        code.text,
        "CTL-007: disabled changes no text of a code input"
    );

    let rating = shown(rating_at(3), (80, 6), "★★★");
    assert_eq!(
        shown(rating_at(3).auto_focus(), (80, 6), "★★★").text,
        rating.text,
        "CTL-007: focus adds no glyph to a rating"
    );
    assert_eq!(
        shown(
            builder::rating().value(3).max(5).disabled(true).build(),
            (80, 6),
            "★★★"
        )
        .text,
        rating.text,
        "CTL-007: disabled changes no text of a rating"
    );

    let switch = shown(switch_at(false), (80, 6), "Wi-Fi");
    assert_eq!(
        shown(switch_at(false).auto_focus(), (80, 6), "Wi-Fi").text,
        switch.text,
        "CTL-007: focus adds no glyph to a switch"
    );
    assert_eq!(
        shown(
            builder::switch().label("Wi-Fi").disabled(true).build(),
            (80, 6),
            "Wi-Fi"
        )
        .text,
        switch.text,
        "CTL-007: disabled changes no text of a switch"
    );

    let hovered = shown_after(
        switch_at(false),
        (80, 6),
        vec![("Wi-Fi", move_to(2, 0)), ("Wi-Fi", None)],
    );
    let hovered = last(&hovered);
    assert_eq!(
        hovered.text, switch.text,
        "CTL-007: the pointer over a switch moves no text:\n{}",
        hovered.text
    );
    let (open, row) = at(hovered, "[");
    for column in open..open + 9 {
        assert_eq!(
            cell_colors(hovered, column, row).1,
            Some(role("hover")),
            "CTL-007: the switch's row under the pointer is hover:\n{}",
            hovered.text
        );
    }
}

// ---------------------------------------------------------------- CTL-008

#[test]
fn ctl_008_a_number_input_fills_a_box_of_100_cells_and_a_width_class_sets_its_width() {
    let frame = shown(boxed(100, 4, number_at(2.0)), (240, 8), "2.0");
    let (_, row) = at(&frame, "2.0");
    assert_eq!(
        last_glyph_column(&frame, row),
        Some(99),
        "CTL-008: the number input fills the box:\n{}",
        frame.text
    );
    let (plus, _) = at(&frame, &format!("[{}]", Icon::Plus.glyph()));
    assert_eq!(
        plus + 1,
        98,
        "CTL-008: the + glyph is at 98:\n{}",
        frame.text
    );
    let (_, end, _) = number_field(&frame);
    assert_eq!(end, 96, "CTL-008: the field's ] is at 96:\n{}", frame.text);

    let frame = shown(
        boxed(
            100,
            4,
            builder::number_input()
                .value(2.0)
                .min(0.0)
                .max(10.0)
                .step(0.5)
                .class("w-40")
                .build(),
        ),
        (240, 8),
        "2.0",
    );
    let (_, row) = at(&frame, "2.0");
    assert_eq!(
        last_glyph_column(&frame, row),
        Some(39),
        "CTL-008: w-40 makes the number input 40 cells wide:\n{}",
        frame.text
    );
}

#[test]
fn ctl_008_a_code_input_a_rating_and_a_switch_take_their_contents_width() {
    let frame = shown(boxed(100, 1, code_at("123")), (240, 4), "[1][2][3]");
    let (start, row) = at(&frame, "[");
    let end = last_glyph_column(&frame, row).unwrap();
    assert_eq!(
        end - start + 1,
        19,
        "CTL-008: six digits in groups of three span 19 cells:\n{}",
        frame.text
    );

    let frame = shown(boxed(100, 1, rating_at(3)), (240, 4), "★★★");
    let (start, row) = at(&frame, "★");
    let end = last_glyph_column(&frame, row).unwrap();
    assert_eq!(
        end - start + 1,
        5,
        "CTL-008: five stars span 5 cells:\n{}",
        frame.text
    );

    let frame = shown(
        boxed(
            100,
            1,
            builder::switch().checked(true).label("Encryption").build(),
        ),
        (240, 4),
        "Encryption",
    );
    let (start, row) = at(&frame, "[");
    let end = last_glyph_column(&frame, row).unwrap();
    assert_eq!(
        end - start + 1,
        15,
        "CTL-008: a switch with a 10-cell label spans 15 cells:\n{}",
        frame.text
    );
}

#[test]
fn ctl_008_a_form_lays_its_label_column_and_fills_its_controls() {
    let frame = shown(
        boxed(
            100,
            10,
            builder::form()
                .field(FormField::new("Name", builder::text_input().build()).required())
                .field(
                    FormField::new("Email", builder::text_input().build())
                        .description("Work address"),
                )
                .field(FormField::new("Description", builder::text_input().build()))
                .build(),
        ),
        (240, 14),
        "Description",
    );
    for label in ["Name", "Email", "Description"] {
        let (_, row) = at(&frame, label);
        assert_eq!(
            glyph(&frame, 13, row),
            "[",
            "CTL-008: the control of {label} starts at column 13:\n{}",
            frame.text
        );
        assert_eq!(
            glyph(&frame, 99, row),
            "]",
            "CTL-008: the control of {label} ends at column 99:\n{}",
            frame.text
        );
    }
    let (_, email_row) = at(&frame, "Email");
    assert_eq!(
        at(&frame, "Work address"),
        (13, email_row + 1),
        "CTL-008: the description stands under the control, at column 13:\n{}",
        frame.text
    );
}

#[test]
fn ctl_008_a_narrow_form_puts_its_labels_above() {
    let frame = shown(
        boxed(
            20,
            8,
            builder::form()
                .field(FormField::new(
                    "Organisation",
                    builder::text_input().build(),
                ))
                .build(),
        ),
        (80, 10),
        "Organisation",
    );
    let (_, label_row) = at(&frame, "Organisation");
    let (column, control_row) = at(&frame, "[");
    assert!(
        control_row > label_row,
        "CTL-008: the label stands above its control:\n{}",
        frame.text
    );
    assert_eq!(
        column, 0,
        "CTL-008: the control starts at column 0:\n{}",
        frame.text
    );
    assert_eq!(
        last_glyph_column(&frame, control_row),
        Some(19),
        "CTL-008: the control ends at column 19:\n{}",
        frame.text
    );
}

#[test]
fn ctl_008_labels_above_puts_each_label_over_its_control() {
    let frame = shown(
        boxed(
            100,
            10,
            builder::form()
                .labels_above()
                .field(FormField::new("Name", builder::text_input().build()).required())
                .field(
                    FormField::new("Email", builder::text_input().build())
                        .description("Work address"),
                )
                .build(),
        ),
        (240, 14),
        "Work address",
    );
    let (_, name_row) = at(&frame, "Name");
    assert_eq!(
        glyph(&frame, 0, name_row + 1),
        "[",
        "CTL-008: Name's control starts at 0:\n{}",
        frame.text
    );
    assert_eq!(
        glyph(&frame, 99, name_row + 1),
        "]",
        "CTL-008: Name's control ends at 99:\n{}",
        frame.text
    );
    let (_, email_row) = at(&frame, "Email");
    assert_eq!(
        glyph(&frame, 0, email_row + 1),
        "[",
        "CTL-008: Email's control starts at 0:\n{}",
        frame.text
    );
    assert_eq!(
        glyph(&frame, 99, email_row + 1),
        "]",
        "CTL-008: Email's control ends at 99:\n{}",
        frame.text
    );
    assert_eq!(
        at(&frame, "Work address"),
        (0, email_row + 2),
        "CTL-008: the description is under its control, at column 0:\n{}",
        frame.text
    );
}

// ---------------------------------------------------------------- CTL-009

/// A number input from 0 to 10 in steps of 0.5 at 2.0 that reports each
/// change to `log`.
fn number_logged(log: &Log<f64>) -> Element {
    builder::number_input()
        .value(2.0)
        .min(0.0)
        .max(10.0)
        .step(0.5)
        .on_change(recorder(log))
        .build()
}

#[test]
fn ctl_009_a_number_input_steps_clamps_and_formats() {
    let log: Log<f64> = Arc::default();
    let frames = shown_after(
        builder::number_input()
            .value(2.0)
            .min(0.0)
            .max(10.0)
            .step(0.5)
            .on_change(recorder(&log))
            .build()
            .auto_focus(),
        (80, 6),
        vec![
            ("2.0", key(KeyCode::Up)),
            ("2.5", key(KeyCode::PageUp)),
            ("7.5", key(KeyCode::End)),
            ("10.0", key(KeyCode::Up)),
            ("10.0", None),
        ],
    );
    assert!(
        find(last(&frames), "10.0").is_some(),
        "CTL-009: End shows 10.0, and Up at the maximum keeps it:\n{}",
        last(&frames).text
    );
    assert_eq!(
        held(&log),
        vec![2.5, 7.5, 10.0],
        "CTL-009: each change is reported once, and Up at the maximum reports nothing:\n{}",
        last(&frames).text
    );
}

#[test]
fn ctl_009_a_number_input_clamps_a_typed_value_when_the_focus_leaves() {
    // Right moves the cursor past the text; Backspace clears it. The
    // "Next" button stays on screen, so it marks each step's frame once
    // the value's text changes.
    let changes: Log<f64> = Arc::default();
    let frames = shown_after(
        builder::div()
            .class("flex-col gap-1")
            .child(
                builder::number_input()
                    .value(0.0)
                    .min(0.0)
                    .max(10.0)
                    .step(0.5)
                    .on_change(recorder(&changes))
                    .build()
                    .auto_focus(),
            )
            .child(builder::button().text("Next").on_click(|| {}).build())
            .build(),
        (80, 6),
        vec![
            ("0.0", key(KeyCode::Right)),
            ("0.0", key(KeyCode::Right)),
            ("0.0", key(KeyCode::Right)),
            ("Next", key(KeyCode::Backspace)),
            ("Next", key(KeyCode::Backspace)),
            ("Next", key(KeyCode::Backspace)),
            ("Next", key(KeyCode::Char('4'))),
            ("Next", key(KeyCode::Char('2'))),
            ("Next", key(KeyCode::Tab)),
            ("10.0", None),
        ],
    );
    assert!(
        find(last(&frames), "10.0").is_some(),
        "CTL-009: a typed 42 is clamped to 10.0 when the focus leaves:\n{}",
        last(&frames).text
    );
    assert_eq!(
        held(&changes),
        vec![4.0, 10.0],
        "CTL-009: typing 4 then 2 in a range of 0 to 10 reports 4, never 42, then the 10 the clamp makes:\n{}",
        last(&frames).text
    );
}

#[test]
fn ctl_009_a_click_on_a_button_and_the_wheel_step_a_number_input() {
    let base = shown(number_at(2.0), (80, 6), "2.0");
    let (plus, row) = at(&base, &format!("[{}]", Icon::Plus.glyph()));
    let (field, _, _) = number_field(&base);
    let log: Log<f64> = Arc::default();
    let frames = shown_after(
        number_logged(&log),
        (80, 6),
        vec![
            ("2.0", app_input::click(plus + 1, row)),
            ("2.5", wheel_up(field + 1, row)),
            ("3.0", None),
        ],
    );
    assert!(
        find(last(&frames), "3.0").is_some(),
        "CTL-009: the wheel steps the number input up:\n{}",
        last(&frames).text
    );
    assert_eq!(
        held(&log),
        vec![2.5, 3.0],
        "CTL-009: the click on [+] and the wheel report their values:\n{}",
        last(&frames).text
    );
}

#[test]
fn ctl_009_a_code_input_fills_clears_pastes_and_completes() {
    let changes: Log<String> = Arc::default();
    let complete: Log<String> = Arc::default();
    let typed = frames_when(
        builder::code_input()
            .length(6)
            .on_change(recorder(&changes))
            .on_complete(recorder(&complete))
            .build()
            .auto_focus(),
        (80, 6),
        vec![
            ("[_][_][_][_][_][_]".to_owned(), key(KeyCode::Char('1'))),
            ("[1][_][_][_][_][_]".to_owned(), key(KeyCode::Char('2'))),
            ("[1][2][_][_][_][_]".to_owned(), key(KeyCode::Char('3'))),
            ("[1][2][3][_][_][_]".to_owned(), key(KeyCode::Backspace)),
            ("[1][2][_][_][_][_]".to_owned(), paste("98-7654")),
            ("[9][8][7][6][5][4]".to_owned(), None),
        ],
    );
    assert_eq!(
        held(&complete),
        vec!["987654".to_owned()],
        "CTL-009: a paste replaces the code and filling the last cell reports it once:\n{}",
        last(&typed).text
    );
    assert_eq!(
        held(&changes).last().map(String::as_str),
        Some("987654"),
        "CTL-009: the last change reports the pasted code:\n{}",
        last(&typed).text
    );

    let complete: Log<String> = Arc::default();
    let pasted = frames_when(
        builder::code_input()
            .length(6)
            .on_complete(recorder(&complete))
            .build()
            .auto_focus(),
        (80, 6),
        vec![
            ("[_][_][_][_][_][_]".to_owned(), paste("12")),
            ("[1][2][_][_][_][_]".to_owned(), None),
        ],
    );
    assert!(
        held(&complete).is_empty(),
        "CTL-009: a paste that does not fill the code reports nothing:\n{}",
        last(&pasted).text
    );
}

#[test]
fn ctl_009_a_masked_code_input_paints_dots_not_digits() {
    let frames = frames_when(
        builder::code_input()
            .length(6)
            .masked(true)
            .build()
            .auto_focus(),
        (80, 6),
        vec![
            ("[_][_][_][_][_][_]".to_owned(), key(KeyCode::Char('7'))),
            (format!("[{}][_][_][_][_][_]", Icon::Dot.glyph()), None),
        ],
    );
    let end = last(&frames);
    assert!(
        !end.text.contains('7'),
        "CTL-009: a typed digit is not painted in a masked code input:\n{}",
        end.text
    );
    let (first, row) = at(end, "[");
    assert_eq!(
        glyph(end, first + 1, row),
        Icon::Dot.glyph(),
        "CTL-009: the first cell shows a dot:\n{}",
        end.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn ctl_009_a_rating_moves_on_keys_digits_and_clicks() {
    let _theme = Active::set(probe());
    let changes: Log<usize> = Arc::default();
    let base = shown(rating_at(3), (80, 6), "★★★☆☆");
    let (first, row) = at(&base, "★★★☆☆");
    let frames = shown_after(
        builder::rating()
            .value(3)
            .max(5)
            .on_change(recorder(&changes))
            .build()
            .auto_focus(),
        (80, 6),
        vec![
            ("★★★☆☆", key(KeyCode::Right)),
            ("★★★★☆", key(KeyCode::Home)),
            ("☆☆☆☆☆", key(KeyCode::Char('2'))),
            ("★★☆☆☆", key(KeyCode::End)),
            ("★★★★★", key(KeyCode::Home)),
            ("☆☆☆☆☆", app_input::click(first + 4, row)),
            ("★★★★★", None),
        ],
    );
    assert_eq!(
        held(&changes),
        vec![4, 0, 2, 5, 0, 5],
        "CTL-009: Right, Home, a digit, End, Home and a click on the fifth star:\n{}",
        last(&frames).text
    );
}

/// CTL-009, BAR-003: a click on a star gives the rating the keyboard focus,
/// as a click gives every control its focus, so the next arrow key moves
/// the rating and not the field that held the focus before.
#[test]
#[serial_test::serial(theme)]
fn ctl_009_a_click_on_a_star_gives_the_rating_the_focus() {
    let _theme = Active::set(probe());
    let changes: Log<usize> = Arc::default();
    let typed: Log<String> = Arc::default();
    let root = || {
        builder::div()
            .class("flex-col gap-1")
            .child(
                builder::text_input()
                    .value("Ada")
                    .on_change(recorder(&typed))
                    .build()
                    .auto_focus(),
            )
            .child(
                builder::rating()
                    .value(1)
                    .max(5)
                    .on_change(recorder(&changes))
                    .build(),
            )
            .build()
    };
    let base = shown(root(), (80, 8), "★☆☆☆☆");
    let (first, row) = at(&base, "★☆☆☆☆");
    let frames = shown_after(
        root(),
        (80, 8),
        vec![
            ("★☆☆☆☆", app_input::click(first + 3, row)),
            ("★★★★☆", key(KeyCode::Right)),
            ("★★★★★", None),
        ],
    );
    assert_eq!(
        held(&changes),
        vec![4, 5],
        "CTL-009: a click on the fourth star sets 4 and the next Right reaches the rating:\n{}",
        last(&frames).text
    );
    assert!(
        held(&typed).is_empty(),
        "CTL-009: the Right after the click does not reach the text input that held the focus; it reported {:?}:\n{}",
        held(&typed),
        last(&frames).text
    );
    let (column, row) = at(last(&frames), "★★★★★");
    assert_eq!(
        cell_colors(last(&frames), column, row).0,
        Some(role("warning")),
        "CTL-007: the lit stars keep warning with the focus:\n{}",
        last(&frames).text
    );
}

#[test]
#[serial_test::serial(theme)]
fn ctl_009_a_switch_toggles_on_space_enter_and_click_and_not_when_disabled() {
    let _theme = Active::set(probe());
    let dot = Icon::Dot.glyph();
    let off = format!("[{dot} ] Wi-Fi");
    let on = format!("[ {dot}] Wi-Fi");
    let base = shown(switch_at(false), (80, 6), "Wi-Fi");
    let (label, row) = at(&base, "Wi-Fi");

    let changes: Log<bool> = Arc::default();
    let frames = frames_when(
        builder::switch()
            .label("Wi-Fi")
            .on_change(recorder(&changes))
            .build()
            .auto_focus(),
        (80, 6),
        vec![
            (off.clone(), key(KeyCode::Char(' '))),
            (on.clone(), key(KeyCode::Enter)),
            (off.clone(), app_input::click(label, row)),
            (on.clone(), None),
        ],
    );
    assert_eq!(
        held(&changes),
        vec![true, false, true],
        "CTL-009: Space, Enter and a click toggle the switch:\n{}",
        last(&frames).text
    );

    let disabled: Log<bool> = Arc::default();
    let frames = shown_after(
        builder::switch()
            .label("Wi-Fi")
            .disabled(true)
            .on_change(recorder(&disabled))
            .build()
            .auto_focus(),
        (80, 6),
        vec![
            ("Wi-Fi", key(KeyCode::Char(' '))),
            ("Wi-Fi", app_input::click(label, row)),
            ("Wi-Fi", None),
        ],
    );
    assert!(
        find(last(&frames), &off).is_some(),
        "CTL-009: a disabled switch stays off:\n{}",
        last(&frames).text
    );
    assert!(
        held(&disabled).is_empty(),
        "CTL-009: a disabled switch calls no on_change"
    );
}

#[test]
#[serial_test::serial(theme)]
fn ctl_009_a_forms_fields_take_the_focus_in_order() {
    let _theme = Active::set(probe());
    let form = || {
        builder::form()
            .field(FormField::new(
                "Name",
                builder::text_input().build().auto_focus(),
            ))
            .field(FormField::new("Email", builder::text_input().build()))
            .build()
    };
    // The settled frame before the Tab: the first frame a run presents is
    // painted before the App gives any control its auto focus.
    let before = &shown(form(), (80, 8), "Name");
    let frames = shown_after(
        form(),
        (80, 8),
        vec![("Name", key(KeyCode::Tab)), ("Email", None)],
    );
    let after = last(&frames);
    let (_, name_row) = at(before, "Name");
    let name_field = column_on(before, name_row, "[");
    assert_eq!(
        cell_colors(before, name_field, name_row).0,
        Some(role("ring")),
        "CTL-009: the first field is focused first:\n{}",
        before.text
    );
    let (_, email_row) = at(after, "Email");
    let email_field = column_on(after, email_row, "[");
    assert_eq!(
        cell_colors(after, email_field, email_row).0,
        Some(role("ring")),
        "CTL-009: Tab moves the focus to the second field:\n{}",
        after.text
    );
    assert_eq!(
        cell_colors(after, name_field, name_row).0,
        Some(role("border")),
        "CTL-009: the first field is no longer focused:\n{}",
        after.text
    );
}

#[test]
fn ctl_009_the_builders_take_the_callbacks_their_components_take() {
    let dot = Icon::Dot.glyph();
    let off = format!("[{dot} ] Wi-Fi");
    let on = format!("[ {dot}] Wi-Fi");

    // A switch: the builder and the props-built twin report Space alike.
    let built: Log<bool> = Arc::default();
    frames_when(
        builder::switch()
            .label("Wi-Fi")
            .on_change(recorder(&built))
            .build()
            .auto_focus(),
        (80, 6),
        vec![(off.clone(), key(KeyCode::Char(' '))), (on.clone(), None)],
    );
    let from_props: Log<bool> = Arc::default();
    let log = from_props.clone();
    let props_switch = Element::typed_with::<Switch>(
        SwitchProps {
            checked: false,
            label: Some("Wi-Fi".to_owned()),
            aria_label: None,
            disabled: false,
        },
        move |props| Switch::new(props).with_on_change(recorder(&log)),
    )
    .auto_focus();
    frames_when(
        props_switch,
        (80, 6),
        vec![(off.clone(), key(KeyCode::Char(' '))), (on.clone(), None)],
    );
    assert_eq!(
        held(&built),
        vec![true],
        "CTL-009: the builder switch reports Space"
    );
    assert_eq!(
        held(&from_props),
        vec![true],
        "CTL-009: the props switch reports Space"
    );

    // A number input: Confirm reports through on_submit, from both ways.
    let built: Log<f64> = Arc::default();
    shown_after(
        builder::number_input()
            .value(2.0)
            .min(0.0)
            .max(10.0)
            .step(0.5)
            .on_submit(recorder(&built))
            .build()
            .auto_focus(),
        (80, 6),
        vec![("2.0", key(KeyCode::Enter)), ("2.0", None)],
    );
    assert_eq!(
        held(&built),
        vec![2.0],
        "CTL-009: the builder number input submits on Confirm"
    );
    let from_props: Log<f64> = Arc::default();
    let log = from_props.clone();
    let props_number = Element::typed_with::<NumberInput>(
        NumberInputProps {
            value: 2.0,
            min: Some(0.0),
            max: Some(10.0),
            step: 0.5,
            aria_label: None,
            disabled: false,
            width: None,
        },
        move |props| NumberInput::new(props).with_on_submit(recorder(&log)),
    )
    .auto_focus();
    shown_after(
        props_number,
        (80, 6),
        vec![("2.0", key(KeyCode::Enter)), ("2.0", None)],
    );
    assert_eq!(
        held(&from_props),
        vec![2.0],
        "CTL-009: the props number input submits on Confirm"
    );

    // A code input: Confirm with every cell filled reports through on_complete.
    let built: Log<String> = Arc::default();
    shown_after(
        builder::code_input()
            .length(2)
            .value("12")
            .on_complete(recorder(&built))
            .build()
            .auto_focus(),
        (80, 6),
        vec![("[1][2]", key(KeyCode::Enter)), ("[1][2]", None)],
    );
    assert_eq!(
        held(&built),
        vec!["12".to_owned()],
        "CTL-009: the builder code input completes on Confirm"
    );
    let from_props: Log<String> = Arc::default();
    let log = from_props.clone();
    let props_code = Element::typed_with::<CodeInput>(
        CodeInputProps {
            length: 2,
            group: None,
            value: "12".to_owned(),
            masked: false,
            label: None,
            aria_label: None,
            disabled: false,
        },
        move |props| CodeInput::new(props).with_on_complete(recorder(&log)),
    )
    .auto_focus();
    shown_after(
        props_code,
        (80, 6),
        vec![("[1][2]", key(KeyCode::Enter)), ("[1][2]", None)],
    );
    assert_eq!(
        held(&from_props),
        vec!["12".to_owned()],
        "CTL-009: the props code input completes on Confirm"
    );

    // The data table takes its page callback.
    let _table = builder::data_table().on_page_change(|_| {}).build();
}

// ---------------------------------------------------------------- CTL-010

#[cfg(target_os = "linux")]
mod reader {
    use super::*;
    use accesskit::{Action, ActionData, Node, NodeId, Toggled, TreeUpdate};
    use reactive_tui::accessibility::{ReaderChannel, Role};

    /// A frame predicate for a frame whose text contains `text`.
    fn painted(text: &'static str) -> FramePredicate {
        Box::new(move |frame: &Snapshot| frame.text.contains(text))
    }

    /// A screen reader's request on the node with `role` named `label`.
    fn ask(
        role: Role,
        label: &'static str,
        action: Action,
        data: Option<ActionData>,
    ) -> Option<app_input::Input> {
        Some(app_input::Input::Request(app_input::ReaderRequest {
            role,
            label: Some(label),
            action,
            data,
        }))
    }

    /// A request's string value.
    fn words(value: &str) -> Option<ActionData> {
        Some(ActionData::Value(value.into()))
    }

    /// A request's numeric value.
    fn number(value: f64) -> Option<ActionData> {
        Some(ActionData::NumericValue(value))
    }

    /// The tree the App published last.
    fn published(reader: &ReaderChannel) -> TreeUpdate {
        reader.tree().expect("the App published a tree")
    }

    /// The node with `role` named `label` in `tree`.
    fn named<'a>(tree: &'a TreeUpdate, role: Role, label: &str) -> &'a Node {
        tree.nodes
            .iter()
            .map(|(_, node)| node)
            .find(|node| node.role() == role && node.label() == Some(label))
            .unwrap_or_else(|| panic!("no {role:?} node named {label:?} in the published tree"))
    }

    /// The node with `id` in `tree`.
    fn by_id(tree: &TreeUpdate, id: NodeId) -> Option<&Node> {
        tree.nodes
            .iter()
            .find(|(node_id, _)| *node_id == id)
            .map(|(_, node)| node)
    }

    /// The label of the one node that `node` is labelled by.
    fn labelled_by_label<'a>(tree: &'a TreeUpdate, node: &Node) -> Option<&'a str> {
        match node.labelled_by() {
            [id] => by_id(tree, *id)?.label(),
            _ => None,
        }
    }

    /// The labels of the nodes that `node` is described by.
    fn described_by_labels<'a>(tree: &'a TreeUpdate, node: &Node) -> Vec<&'a str> {
        node.described_by()
            .iter()
            .filter_map(|id| by_id(tree, *id)?.label())
            .collect()
    }

    /// The text input nodes of `tree`.
    fn text_inputs(tree: &TreeUpdate) -> Vec<&Node> {
        tree.nodes
            .iter()
            .map(|(_, node)| node)
            .filter(|node| node.role() == Role::TextInput)
            .collect()
    }

    /// Runs `root` with an in-process screen reader and `steps`.
    fn run(
        root: Element,
        reader: &ReaderChannel,
        steps: Vec<(FramePredicate, Option<app_input::Input>)>,
    ) -> Vec<Snapshot> {
        app_input::run_when_frame_reading(Control(page(root)), (80, 6), reader, steps)
    }

    #[test]
    fn ctl_010_a_number_input_is_a_spin_button_with_its_bounds_and_steps_on_request() {
        let volume = || {
            builder::number_input()
                .aria_label("Volume")
                .value(2.0)
                .min(0.0)
                .max(10.0)
                .step(0.5)
                .build()
        };
        let reader = ReaderChannel::new();
        run(volume(), &reader, vec![(painted("2.0"), None)]);
        let tree = published(&reader);
        let spin = named(&tree, Role::SpinButton, "Volume");
        assert_eq!(spin.numeric_value(), Some(2.0), "CTL-010: the value");
        assert_eq!(spin.min_numeric_value(), Some(0.0), "CTL-010: the minimum");
        assert_eq!(spin.max_numeric_value(), Some(10.0), "CTL-010: the maximum");
        assert_eq!(spin.numeric_value_step(), Some(0.5), "CTL-010: the step");
        for action in [Action::Increment, Action::Decrement, Action::SetValue] {
            assert!(
                spin.supports_action(action),
                "CTL-010: the spin button advertises {action:?}"
            );
        }

        let reader = ReaderChannel::new();
        let frames = run(
            volume(),
            &reader,
            vec![
                (
                    painted("2.0"),
                    ask(Role::SpinButton, "Volume", Action::Increment, None),
                ),
                (painted("2.5"), None),
            ],
        );
        assert!(
            last(&frames).text.contains("2.5"),
            "CTL-010: an Increment steps the value:\n{}",
            last(&frames).text
        );

        let reader = ReaderChannel::new();
        let frames = run(
            volume(),
            &reader,
            vec![
                (
                    painted("2.0"),
                    ask(Role::SpinButton, "Volume", Action::SetValue, number(7.0)),
                ),
                (painted("7.0"), None),
            ],
        );
        assert!(
            last(&frames).text.contains("7.0"),
            "CTL-010: a SetValue of 7.0 sets the value:\n{}",
            last(&frames).text
        );
    }

    #[test]
    fn ctl_010_a_code_input_is_one_text_field_named_with_its_length() {
        let reader = ReaderChannel::new();
        run(
            builder::code_input()
                .label("Code")
                .length(6)
                .group(3)
                .value("123")
                .build(),
            &reader,
            vec![(painted("[1][2][3]"), None)],
        );
        let tree = published(&reader);
        let fields = tree
            .nodes
            .iter()
            .map(|(_, node)| node)
            .filter(|node| matches!(node.role(), Role::TextInput | Role::PasswordInput))
            .count();
        assert_eq!(fields, 1, "CTL-010: one node for the whole code input");
        let field = named(&tree, Role::TextInput, "Code, 6 digits");
        assert_eq!(
            field.value(),
            Some("123"),
            "CTL-010: the digits are the value"
        );

        let reader = ReaderChannel::new();
        run(
            builder::code_input()
                .label("Code")
                .length(6)
                .group(3)
                .value("123")
                .masked(true)
                .build(),
            &reader,
            vec![(painted("[_]"), None)],
        );
        let tree = published(&reader);
        let masked = named(&tree, Role::PasswordInput, "Code, 6 digits");
        assert_eq!(
            masked.value(),
            Some("•••"),
            "CTL-010: a masked code gives bullets, not its digits"
        );

        let reader = ReaderChannel::new();
        let frames = run(
            builder::code_input().label("Code").length(6).build(),
            &reader,
            vec![
                (
                    painted("[_][_][_][_][_][_]"),
                    ask(
                        Role::TextInput,
                        "Code, 6 digits",
                        Action::SetValue,
                        words("987654"),
                    ),
                ),
                (painted("[9][8][7][6][5][4]"), None),
            ],
        );
        assert!(
            last(&frames).text.contains("[9][8][7][6][5][4]"),
            "CTL-010: a SetValue fills the empty code input:\n{}",
            last(&frames).text
        );
    }

    #[test]
    fn ctl_010_a_rating_is_a_slider_reading_n_of_m() {
        let rating = || {
            builder::rating()
                .aria_label("Quality")
                .value(3)
                .max(5)
                .build()
        };
        let reader = ReaderChannel::new();
        run(rating(), &reader, vec![(painted("★★★☆☆"), None)]);
        let tree = published(&reader);
        let slider = named(&tree, Role::Slider, "Quality");
        assert_eq!(slider.value(), Some("3 of 5"), "CTL-010: the value text");
        assert_eq!(slider.numeric_value(), Some(3.0), "CTL-010: the value");
        assert_eq!(
            slider.min_numeric_value(),
            Some(0.0),
            "CTL-010: the minimum"
        );
        assert_eq!(
            slider.max_numeric_value(),
            Some(5.0),
            "CTL-010: the maximum"
        );
        assert_eq!(slider.numeric_value_step(), Some(1.0), "CTL-010: the step");
        for action in [Action::Increment, Action::Decrement, Action::SetValue] {
            assert!(
                slider.supports_action(action),
                "CTL-010: the rating advertises {action:?}"
            );
        }

        let reader = ReaderChannel::new();
        let frames = run(
            rating(),
            &reader,
            vec![
                (
                    painted("★★★☆☆"),
                    ask(Role::Slider, "Quality", Action::SetValue, number(4.0)),
                ),
                (painted("★★★★☆"), None),
            ],
        );
        assert!(
            last(&frames).text.contains("★★★★☆"),
            "CTL-010: a SetValue of 4 shows four stars:\n{}",
            last(&frames).text
        );
    }

    #[test]
    fn ctl_010_a_switch_is_a_switch_toggled_on_or_off() {
        let reader = ReaderChannel::new();
        run(
            builder::switch().checked(true).label("Wi-Fi").build(),
            &reader,
            vec![(painted("Wi-Fi"), None)],
        );
        let tree = published(&reader);
        assert_eq!(
            named(&tree, Role::Switch, "Wi-Fi").toggled(),
            Some(Toggled::True),
            "CTL-010: a switch that is on reports toggled on"
        );

        let reader = ReaderChannel::new();
        run(
            builder::switch().checked(false).label("Wi-Fi").build(),
            &reader,
            vec![(painted("Wi-Fi"), None)],
        );
        let tree = published(&reader);
        assert_eq!(
            named(&tree, Role::Switch, "Wi-Fi").toggled(),
            Some(Toggled::False),
            "CTL-010: a switch that is off reports toggled off"
        );

        let reader = ReaderChannel::new();
        run(
            builder::switch()
                .checked(false)
                .label("Wi-Fi")
                .aria_label("Wireless")
                .build(),
            &reader,
            vec![(painted("Wi-Fi"), None)],
        );
        let tree = published(&reader);
        named(&tree, Role::Switch, "Wireless");
    }

    #[test]
    fn ctl_010_a_forms_controls_are_labelled_and_described_by_their_label_and_description() {
        let reader = ReaderChannel::new();
        run(
            builder::form()
                .aria_label("Account")
                .field(FormField::new("Name", builder::text_input().build()).required())
                .field(
                    FormField::new("Email", builder::text_input().build())
                        .description("Work address"),
                )
                .build(),
            &reader,
            vec![(painted("Work address"), None)],
        );
        let tree = published(&reader);
        named(&tree, Role::Form, "Account");
        let inputs = text_inputs(&tree);
        assert_eq!(inputs.len(), 2, "CTL-010: one text input node per field");
        let name = inputs
            .iter()
            .copied()
            .find(|node| labelled_by_label(&tree, node) == Some("Name"))
            .unwrap_or_else(|| panic!("CTL-010: no text input is labelled by Name"));
        assert!(
            name.is_required(),
            "CTL-010: the required field's node is required"
        );
        let email = inputs
            .iter()
            .copied()
            .find(|node| labelled_by_label(&tree, node) == Some("Email"))
            .unwrap_or_else(|| panic!("CTL-010: no text input is labelled by Email"));
        assert!(
            described_by_labels(&tree, email).contains(&"Work address"),
            "CTL-010: the Email field is described by its description"
        );
        assert!(
            !email.is_required(),
            "CTL-010: an optional field is not required"
        );
    }

    #[test]
    fn ctl_010_a_text_input_marked_as_a_rows_control_is_labelled_by_the_rows_label() {
        // A form row built by hand: the markers sit on the text input's own
        // element, which the runtime expands into the element it renders.
        use reactive_tui::component::{FieldPart, FieldRow};
        let reader = ReaderChannel::new();
        run(
            builder::div()
                .class("flex-col")
                .child(
                    builder::div()
                        .class("flex flex-row")
                        .child(Element::text("Phone").with_field_part(FieldPart::Label))
                        .child(
                            builder::text_input()
                                .build()
                                .with_field_part(FieldPart::Control),
                        )
                        .build()
                        .with_field_row(FieldRow { required: true }),
                )
                .build(),
            &reader,
            vec![(painted("Phone"), None)],
        );
        let tree = published(&reader);
        let inputs = text_inputs(&tree);
        assert_eq!(inputs.len(), 1, "CTL-010: one text input node");
        assert_eq!(
            labelled_by_label(&tree, inputs[0]),
            Some("Phone"),
            "CTL-010: the control is labelled by the row's label"
        );
        assert!(
            inputs[0].is_required(),
            "CTL-010: the row's control is required"
        );
    }

    #[test]
    fn ctl_010_a_disabled_piece_is_marked_disabled() {
        let reader = ReaderChannel::new();
        run(
            builder::div()
                .class("flex-col")
                .child(
                    builder::number_input()
                        .aria_label("Volume")
                        .value(2.0)
                        .min(0.0)
                        .max(10.0)
                        .step(0.5)
                        .disabled(true)
                        .build(),
                )
                .child(
                    builder::code_input()
                        .label("Code")
                        .length(2)
                        .disabled(true)
                        .build(),
                )
                .child(
                    builder::rating()
                        .aria_label("Quality")
                        .value(3)
                        .max(5)
                        .disabled(true)
                        .build(),
                )
                .child(builder::switch().label("Wi-Fi").disabled(true).build())
                .build(),
            &reader,
            vec![(painted("Wi-Fi"), None)],
        );
        let tree = published(&reader);
        assert!(
            named(&tree, Role::SpinButton, "Volume").is_disabled(),
            "CTL-010: the number input"
        );
        assert!(
            named(&tree, Role::TextInput, "Code, 2 digits").is_disabled(),
            "CTL-010: the code input"
        );
        assert!(
            named(&tree, Role::Slider, "Quality").is_disabled(),
            "CTL-010: the rating"
        );
        assert!(
            named(&tree, Role::Switch, "Wi-Fi").is_disabled(),
            "CTL-010: the switch"
        );
    }
}

// ---------------------------------------------------------------- BAR-003 helpers

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

// ---------------------------------------------------------------- BAR-003

#[test]
#[serial_test::serial(theme)]
fn bar_003_an_input_piece_takes_every_color_from_the_active_theme() {
    let _theme = Active::set(probe());
    let pieces: Vec<(&str, Element, &str)> = vec![
        ("number input", number_at(2.0), "2.0"),
        ("focused number input", number_at(2.0).auto_focus(), "2.0"),
        ("disabled number input", disabled_number_at(2.0), "2.0"),
        ("code input", code_at("123"), "[1][2][3]"),
        (
            "focused code input",
            code_at("123").auto_focus(),
            "[1][2][3]",
        ),
        (
            "disabled code input",
            builder::code_input()
                .length(6)
                .group(3)
                .value("123")
                .disabled(true)
                .build(),
            "[1][2][3]",
        ),
        ("rating", rating_at(3), "★★★"),
        ("focused rating", rating_at(3).auto_focus(), "★★★"),
        (
            "disabled rating",
            builder::rating().value(3).max(5).disabled(true).build(),
            "★★★",
        ),
        ("switch", switch_at(true), "Wi-Fi"),
        ("focused switch", switch_at(true).auto_focus(), "Wi-Fi"),
        (
            "disabled switch",
            builder::switch()
                .checked(true)
                .label("Wi-Fi")
                .disabled(true)
                .build(),
            "Wi-Fi",
        ),
        ("form", form_at(), "Work address"),
    ];
    let mut wrong = Vec::new();
    for (name, root, text) in pieces {
        let frame = shown(root, (80, 24), text);
        let foreign = foreign(&frame);
        if !foreign.is_empty() {
            wrong.push(format!("{name}: {foreign:#?}\n{}", frame.text));
        }
    }
    assert!(
        wrong.is_empty(),
        "BAR-003: an input piece paints a color the theme does not define:\n{}",
        wrong.join("\n")
    );
}

#[test]
fn bar_003_an_input_piece_follows_a_resize() {
    let frames = shown_after(
        builder::div()
            .class("flex-col gap-1 w-full")
            .child(number_at(2.0))
            .child(
                builder::form()
                    .field(FormField::new("Name", builder::text_input().build()))
                    .build(),
            )
            .build(),
        (80, 24),
        vec![
            ("2.0", Some(Event::Resize(ResizeEvent::new(160, 48)))),
            ("2.0", None),
        ],
    );
    // The settled frame at 80 columns: the last one of that width.
    let before = frames
        .iter()
        .rev()
        .find(|frame| frame.screen.size().1 == 80)
        .expect("a frame at 80 columns");
    let after = last(&frames);
    let (_, number_row) = at(before, "2.0");
    assert_eq!(
        last_glyph_column(before, number_row),
        Some(79),
        "the number input fills 80 columns:\n{}",
        before.text
    );
    let (_, number_row) = at(after, "2.0");
    assert_eq!(
        last_glyph_column(after, number_row),
        Some(159),
        "the number input fills 160 columns after the resize:\n{}",
        after.text
    );
    let (_, name_row) = at(before, "Name");
    assert_eq!(
        last_glyph_column(before, name_row),
        Some(79),
        "the form's control fills 80 columns:\n{}",
        before.text
    );
    let (_, name_row) = at(after, "Name");
    assert_eq!(
        last_glyph_column(after, name_row),
        Some(159),
        "the form's control fills 160 columns after the resize:\n{}",
        after.text
    );
}

#[test]
fn bar_003_every_pointer_action_of_an_input_piece_has_a_key() {
    let completed: Log<String> = Arc::default();
    let frames = shown_after(
        builder::div()
            .class("flex-col gap-1")
            .child(builder::switch().label("Wi-Fi").build().auto_focus())
            .child(builder::rating().value(0).max(5).build())
            .child(
                builder::number_input()
                    .value(2.0)
                    .min(0.0)
                    .max(10.0)
                    .step(0.5)
                    .build(),
            )
            .child(
                builder::code_input()
                    .length(6)
                    .on_complete(recorder(&completed))
                    .build(),
            )
            .build(),
        (80, 24),
        vec![
            // Space turns the switch on.
            ("Wi-Fi", key(KeyCode::Char(' '))),
            // Tab moves to the rating; End fills it.
            ("Wi-Fi", key(KeyCode::Tab)),
            ("☆☆☆☆☆", key(KeyCode::End)),
            // Tab moves to the number input; Up steps it.
            ("★★★★★", key(KeyCode::Tab)),
            ("2.0", key(KeyCode::Up)),
            // Tab moves to the code input; the digits fill it.
            ("2.5", key(KeyCode::Tab)),
            ("[_][_][_][_][_][_]", key(KeyCode::Char('1'))),
            ("[1][_][_][_][_][_]", key(KeyCode::Char('2'))),
            ("[1][2][_][_][_][_]", key(KeyCode::Char('3'))),
            ("[1][2][3][_][_][_]", key(KeyCode::Char('4'))),
            ("[1][2][3][4][_][_]", key(KeyCode::Char('5'))),
            ("[1][2][3][4][5][_]", key(KeyCode::Char('6'))),
            ("[1][2][3][4][5][6]", None),
        ],
    );
    let end = last(&frames);
    assert!(
        find(end, &format!("[ {}] Wi-Fi", Icon::Dot.glyph())).is_some(),
        "the switch is on:\n{}",
        end.text
    );
    assert!(
        find(end, "★★★★★").is_some(),
        "the rating is at its maximum:\n{}",
        end.text
    );
    assert!(
        find(end, "2.5").is_some(),
        "the number input stepped up:\n{}",
        end.text
    );
    assert!(
        find(end, "[1][2][3][4][5][6]").is_some(),
        "the code input is full:\n{}",
        end.text
    );
    assert_eq!(
        held(&completed),
        vec!["123456".to_owned()],
        "the code input completed once:\n{}",
        end.text
    );
}
