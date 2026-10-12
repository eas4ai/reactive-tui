//! The medium widgets' contract, first batch (docs/spec/layout-widgets.md,
//! NAV-007 to NAV-010; docs/spec/data-widgets.md, DAT-006 to DAT-009): the
//! resizable split, the sidebar, the log view and the list under a theme
//! whose roles all differ, so a cell's color names its role; the size each
//! fills and how a split shares its length; the drags, keys, clicks and
//! arriving rows that move them, and the callbacks that report each move;
//! and what the screen reader is told about each of them. The
//! screen-reader checks run on Linux only and sit in a `reader` module.
//!
//! Outside that module every run sends its next event once the App is idle,
//! gated on text the run shows whatever the event did, so a widget that
//! ignores an event fails an assertion rather than the wait.

mod common;

use std::collections::VecDeque;
use std::ops::Range;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use common::app_input::{self, Snapshot, Until};
use reactive_tui::{
    app::RootComponent,
    builder,
    component::Element,
    event::{
        router::EventResult,
        types::{
            Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
            Position, WheelDelta, WheelEvent, WheelPhase,
        },
    },
    theme::{Theme, ThemeVariables},
    widgets::{
        display::{
            pieces::{
                badge::{badge as badge_piece, badge_text, BadgeContent, BadgeProps},
                icon::{Icon, SPINNER_ASCII_FRAMES, SPINNER_FRAMES},
                separator_glyph, SeparatorStyle,
            },
            List, ListItem, ListProps,
        },
        layout::{
            LogHandle, LogKind, LogRow, LogView, LogViewProps, Pane, Resizable, ResizableProps,
            Sidebar, SidebarItem, SidebarProps, SplitDirection,
        },
    },
};

type Rgb = [u8; 3];

/// How long a run may take to reach its last step before it fails instead
/// of hanging: a guard, not a timing check, so a loaded machine cannot fail
/// a correct test.
const WAIT: Duration = app_input::HANG_GUARD;

/// The viewport of every run, the one the falsifiers name.
const SCREEN: (u16, u16) = (240, 60);

/// A step gate every frame meets: the step waits only for the App to be
/// idle.
const ANY: &str = "";

struct Control(Element);
impl RootComponent for Control {
    fn render(&self) -> Element {
        self.0.clone()
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

/// A change a test makes to what the application holds between two
/// frames, as the application's own code would.
type Change = Box<dyn FnOnce() + Send>;

/// The root of a run whose application changes: it shows `view` of the
/// count of F2 presses so far, and on each F2, a key none of these widgets
/// takes, it makes its next change before it counts the press.
struct Scripted {
    view: Box<dyn Fn(usize) -> Element + Send + Sync>,
    presses: AtomicUsize,
    changes: Mutex<VecDeque<Change>>,
}

impl Scripted {
    fn new(view: impl Fn(usize) -> Element + Send + Sync + 'static, changes: Vec<Change>) -> Self {
        Self {
            view: Box::new(view),
            presses: AtomicUsize::new(0),
            changes: Mutex::new(changes.into()),
        }
    }
}

impl RootComponent for Scripted {
    fn render(&self) -> Element {
        page((self.view)(self.presses.load(Ordering::SeqCst)))
    }
    fn wake_driven(&self) -> bool {
        true
    }
    fn handle_event(&self, event: &Event) -> EventResult {
        let Event::Key(key) = event else {
            return EventResult::Ignored;
        };
        if key.code != KeyCode::F(2) {
            return EventResult::Ignored;
        }
        let next = self.changes.lock().unwrap().pop_front();
        if let Some(change) = next {
            change();
        }
        self.presses.fetch_add(1, Ordering::SeqCst);
        EventResult::Consumed
    }
}

/// The key that makes a `Scripted` root's next change.
fn change() -> Option<Event> {
    key(KeyCode::F(2))
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

/// The column of the first cell on `row` that shows `needle`.
fn column_on(frame: &Snapshot, row: u16, needle: &str) -> u16 {
    let (_, columns) = frame.screen.size();
    (0..columns)
        .find(|column| glyph(frame, *column, row) == needle)
        .unwrap_or_else(|| panic!("{needle:?} is not painted on row {row}:\n{}", frame.text))
}

/// The glyphs of `row` in `columns`, an unwritten cell read as a space,
/// with the trailing spaces cut.
fn line(frame: &Snapshot, row: u16, columns: Range<u16>) -> String {
    let text: String = columns
        .map(|column| {
            let cell = glyph(frame, column, row);
            if cell.is_empty() {
                " ".to_owned()
            } else {
                cell
            }
        })
        .collect();
    text.trim_end().to_owned()
}

/// One cell as (glyph, glyph color, background).
type Painted = (String, Option<Rgb>, Option<Rgb>);

/// The cells of `row` in `columns`.
fn cells(frame: &Snapshot, row: u16, columns: Range<u16>) -> Vec<Painted> {
    columns
        .map(|column| {
            let (fg, bg) = cell_colors(frame, column, row);
            (glyph(frame, column, row), fg, bg)
        })
        .collect()
}

/// Every cell of the screen, row by row, for comparing two frames.
fn paint(frame: &Snapshot) -> Vec<Painted> {
    let (rows, columns) = frame.screen.size();
    (0..rows)
        .flat_map(|row| cells(frame, row, 0..columns))
        .collect()
}

/// The columns of `row` whose background is the role `name`.
fn on_role(frame: &Snapshot, row: u16, name: &str) -> Vec<u16> {
    let (_, columns) = frame.screen.size();
    (0..columns)
        .filter(|column| cell_colors(frame, *column, row).1 == Some(role(name)))
        .collect()
}

/// The rows of `column` whose background is the role `name`.
fn rows_on_role(frame: &Snapshot, column: u16, name: &str) -> Vec<u16> {
    let (rows, _) = frame.screen.size();
    (0..rows)
        .filter(|row| cell_colors(frame, column, *row).1 == Some(role(name)))
        .collect()
}

/// Whether the keyboard cursor stands on `label`: its first cell is
/// `selection` with `selection-foreground` text.
fn cursor_on(frame: &Snapshot, label: &str) -> bool {
    find(frame, label).is_some_and(|(column, row)| {
        cell_colors(frame, column, row)
            == (Some(role("selection-foreground")), Some(role("selection")))
    })
}

/// The labels among `labels` the keyboard cursor stands on.
fn cursor_among<'a>(frame: &Snapshot, labels: &[&'a str]) -> Vec<&'a str> {
    labels
        .iter()
        .copied()
        .filter(|label| cursor_on(frame, label))
        .collect()
}

/// A page in the theme's background that holds `child`.
fn page(child: Element) -> Element {
    builder::div()
        .class("relative w-full h-full bg-background text-foreground")
        .child(child)
        .build()
}

/// A box of `width` cells and `height` rows at the page's top left corner.
fn boxed(width: u16, height: u16, child: Element) -> Element {
    builder::div()
        .class(&format!("flex-col w-{width} h-{height}"))
        .child(child)
        .build()
}

/// Every frame `root` presents at 240 by 60 while each of `events` goes out
/// once `text` is painted and the App is idle, ending on the frame the App
/// settles on after the last of them.
fn run(
    root: impl RootComponent + 'static,
    text: &'static str,
    events: Vec<Option<Event>>,
) -> Vec<Snapshot> {
    let mut steps: Vec<Until> = events
        .into_iter()
        .map(|event| Until {
            text,
            cell: None,
            event,
        })
        .collect();
    steps.push(Until {
        text,
        cell: None,
        event: None,
    });
    app_input::run_until(root, SCREEN, steps, WAIT)
}

fn last(frames: &[Snapshot]) -> &Snapshot {
    frames.last().expect("a frame")
}

fn last_of(mut frames: Vec<Snapshot>) -> Snapshot {
    frames.pop().expect("a frame")
}

/// The frame `root`, on the page, settles on after `events`.
fn settled(root: Element, text: &'static str, events: Vec<Option<Event>>) -> Snapshot {
    last_of(run(Control(page(root)), text, events))
}

/// The frame `root`, on the page, settles on once `text` is painted.
fn shown(root: Element, text: &'static str) -> Snapshot {
    settled(root, text, Vec::new())
}

fn key(code: KeyCode) -> Option<Event> {
    app_input::key(code)
}

/// `code` with Shift held.
fn shift(code: KeyCode) -> Option<Event> {
    Some(Event::Key(
        KeyEvent::new(code).with_modifiers(KeyModifiers::shift()),
    ))
}

/// The keys that type `text`.
fn typed(text: &str) -> Vec<Option<Event>> {
    text.chars().map(|c| key(KeyCode::Char(c))).collect()
}

/// A left-button mouse event of `kind` at (x, y).
fn mouse(kind: MouseEventKind, x: u16, y: u16) -> Option<Event> {
    Some(Event::Mouse(
        MouseEvent::new(kind, Position::cell(x, y)).with_button(MouseButton::Left),
    ))
}

/// A press and a release at (x, y).
fn click(x: u16, y: u16) -> Vec<Option<Event>> {
    vec![
        mouse(MouseEventKind::Down, x, y),
        mouse(MouseEventKind::Up, x, y),
    ]
}

/// Two clicks at (x, y). The events are made together, so their times
/// fall well inside the App's double-click time however slowly the run
/// sends them.
fn double_click(x: u16, y: u16) -> Vec<Option<Event>> {
    [click(x, y), click(x, y)].concat()
}

/// A press at `from`, a motion to `to` and a release there.
fn drag(from: (u16, u16), to: (u16, u16)) -> Vec<Option<Event>> {
    vec![
        mouse(MouseEventKind::Down, from.0, from.1),
        mouse(MouseEventKind::Drag, to.0, to.1),
        mouse(MouseEventKind::Up, to.0, to.1),
    ]
}

/// The pointer moves onto (x, y) with no button held.
fn hover(x: u16, y: u16) -> Option<Event> {
    Some(Event::Mouse(MouseEvent::new(
        MouseEventKind::Move,
        Position::cell(x, y),
    )))
}

/// A wheel event of `lines` lines over (x, y): negative is up, as the
/// terminal reports it.
fn wheel(x: u16, y: u16, lines: f32) -> Option<Event> {
    Some(Event::Mouse(MouseEvent::wheel(
        Position::cell(x, y),
        WheelEvent {
            delta: WheelDelta::Lines { x: 0.0, y: lines },
            phase: WheelPhase::Changed,
        },
    )))
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

/// A count of the calls of a callback that takes no value.
type Count = Arc<AtomicUsize>;

/// A closure that counts its calls in `count`.
fn counting(count: &Count) -> impl Fn() + Send + Sync + 'static {
    let count = count.clone();
    move || {
        count.fetch_add(1, Ordering::SeqCst);
    }
}

fn calls(count: &Count) -> usize {
    count.load(Ordering::SeqCst)
}

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
        // The second cell of a wide glyph carries no colors of its own: a
        // terminal paints it with the glyph's.
        if cell.is_wide_continuation() {
            continue;
        }
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

/// The names of `frames` that paint a color the probe theme does not
/// define, with the colors and the frame.
fn foreign_in(frames: &[(&str, Snapshot)]) -> Vec<String> {
    frames
        .iter()
        .filter_map(|(name, frame)| {
            let foreign = foreign(frame);
            (!foreign.is_empty()).then(|| format!("{name}: {foreign:#?}\n{}", frame.text))
        })
        .collect()
}

// ---------------------------------------------------------------- the split

/// The line of a divider between panes side by side, as the separator
/// piece draws it.
fn upright() -> String {
    separator_glyph(SeparatorStyle::Line, true).to_string()
}

/// The line of a divider between stacked panes.
fn flat() -> String {
    separator_glyph(SeparatorStyle::Line, false).to_string()
}

/// The name and the fill of a split's first, second and third pane.
const PANES: [(&str, &str); 3] = [
    ("First pane", "primary"),
    ("Second pane", "secondary"),
    ("Third pane", "info"),
];

/// Pane `index`'s content: its name on its fill, covering the pane, so the
/// cells of a row in that fill measure the pane.
fn filled(index: usize) -> Element {
    let (name, fill) = PANES[index];
    builder::div()
        .class(&format!("w-full h-full bg-{fill}"))
        .text(name)
        .build()
}

/// A split of panes side by side with `sizes` (None: an equal share) that
/// reports each move to `log`.
fn split(sizes: &[Option<u16>], log: &Log<Vec<u16>>) -> Element {
    let mut built = builder::resizable().on_resize(recorder(log));
    for (index, size) in sizes.iter().enumerate() {
        let pane = Pane::new(filled(index));
        built = built.pane_with(match size {
            Some(cells) => pane.size(*cells),
            None => pane,
        });
    }
    built.build()
}

/// NAV-009's split: two panes of 49 and 50 cells.
fn split_49_50(log: &Log<Vec<u16>>) -> Element {
    split(&[Some(49), None], log)
}

/// The same split with no callback, from its builder.
fn plain_split_49_50() -> Element {
    builder::resizable()
        .pane_with(Pane::new(filled(0)).size(49))
        .pane(filled(1))
        .build()
}

/// NAV-009's three panes of 33, 32 and 33 cells with minimums of 4.
fn three_panes(log: &Log<Vec<u16>>) -> Element {
    let mut built = builder::resizable().on_resize(recorder(log));
    for (index, size) in [33, 32, 33].into_iter().enumerate() {
        built = built.pane_with(Pane::new(filled(index)).size(size).min(4));
    }
    built.build()
}

/// Two stacked panes of 9 and 10 rows.
fn stacked(log: &Log<Vec<u16>>) -> Element {
    builder::resizable()
        .vertical()
        .on_resize(recorder(log))
        .pane_with(Pane::new(filled(0)).size(9))
        .pane(filled(1))
        .build()
}

/// The width of each of the first `count` panes on `row`: the cells of the
/// row in the pane's fill.
fn widths(frame: &Snapshot, row: u16, count: usize) -> Vec<u16> {
    PANES[..count]
        .iter()
        .map(|(_, fill)| on_role(frame, row, fill).len() as u16)
        .collect()
}

/// The height of each of the first `count` stacked panes in `column`.
fn heights(frame: &Snapshot, column: u16, count: usize) -> Vec<u16> {
    PANES[..count]
        .iter()
        .map(|(_, fill)| rows_on_role(frame, column, fill).len() as u16)
        .collect()
}

/// The columns of `row` that hold the line of a divider between panes side
/// by side.
fn dividers(frame: &Snapshot, row: u16) -> Vec<u16> {
    let (_, columns) = frame.screen.size();
    let line = upright();
    (0..columns)
        .filter(|column| glyph(frame, *column, row) == line)
        .collect()
}

/// The frame a split that `make` builds, in a box of 100 by 20, settles on
/// after `events`, and the sizes it reported.
fn split_after(
    make: impl FnOnce(&Log<Vec<u16>>) -> Element,
    events: Vec<Option<Event>>,
) -> (Snapshot, Vec<Vec<u16>>) {
    let log: Log<Vec<u16>> = Arc::default();
    let frame = settled(boxed(100, 20, make(&log)), ANY, events);
    (frame, held(&log))
}

/// A root that shows a split of `sizes` in a box 100 cells wide, which the
/// first F2 widens to 200.
fn widening(sizes: Vec<Option<u16>>) -> Scripted {
    let log: Log<Vec<u16>> = Arc::default();
    Scripted::new(
        move |presses| {
            let width = if presses == 0 { 100 } else { 200 };
            boxed(width, 20, split(&sizes, &log))
        },
        Vec::new(),
    )
}

// ---------------------------------------------------------------- the sidebar

/// The width of a default sidebar, and the column of the edge that faces
/// the content.
const BAR: u16 = 28;
const EDGE: u16 = BAR - 1;

/// A row of the box of 20 below the items of the sidebars here and above
/// their toggles, so no item's or toggle's fill stands on it.
const EMPTY_ROW: u16 = 10;

/// NAV-009's sidebar: Home, a group Projects holding Alpha and Beta, open
/// when `open` is set, and Settings. Home has no icon, so a collapsed bar
/// shows its H.
fn nav_items_open(open: bool) -> Vec<SidebarItem> {
    vec![
        SidebarItem::new("home", "Home"),
        SidebarItem::new("projects", "Projects")
            .icon(Icon::Folder)
            .open(open)
            .child(SidebarItem::new("alpha", "Alpha").icon(Icon::File))
            .child(SidebarItem::new("beta", "Beta").icon(Icon::File)),
        SidebarItem::new("settings", "Settings").icon(Icon::Settings),
    ]
}

/// NAV-009's sidebar with the group Projects closed.
fn nav_items() -> Vec<SidebarItem> {
    nav_items_open(false)
}

/// The badge `3` an item of `more_items` carries.
fn three() -> BadgeProps {
    BadgeProps {
        content: BadgeContent::Text("3".to_owned()),
        ..BadgeProps::default()
    }
}

/// `nav_items` with Inbox, which carries the badge 3, and Archive, disabled
/// when `archive_disabled` is set.
fn more_items(archive_disabled: bool) -> Vec<SidebarItem> {
    let mut items = nav_items();
    items.insert(
        2,
        SidebarItem::new("inbox", "Inbox")
            .icon(Icon::File)
            .badge("3"),
    );
    items.insert(
        3,
        SidebarItem::new("archive", "Archive").disabled(archive_disabled),
    );
    items
}

/// A default sidebar of `items` in a box of 100 by 20.
fn sidebar_of(items: Vec<SidebarItem>) -> Element {
    boxed(100, 20, builder::sidebar().items(items).build())
}

/// A collapsed sidebar of NAV-009's items in a box of 100 by 20.
fn collapsed_sidebar() -> Element {
    boxed(
        100,
        20,
        builder::sidebar()
            .items(nav_items())
            .collapsed(true)
            .build(),
    )
}

/// The count of the first columns of `row` on `surface`: the bar's width.
fn bar_width(frame: &Snapshot, row: u16) -> u16 {
    let (_, columns) = frame.screen.size();
    (0..columns)
        .take_while(|column| cell_colors(frame, *column, row).1 == Some(role("surface")))
        .count() as u16
}

/// The badge 3 as the badge piece paints it alone, from its first cell to
/// its last.
fn badge_cells() -> Vec<Painted> {
    let text = badge_text(&three()).expect("a badge with a text shows");
    let frame = shown(badge_piece(&three()), "3");
    let (column, row) = at(&frame, "3");
    let start = column - text.find('3').unwrap() as u16;
    cells(&frame, row, start..start + text.chars().count() as u16)
}

/// The frame a default sidebar of `items` in a box of 100 by 20 settles on
/// after `events`, with what it reported through `on_select` and
/// `on_collapse`.
fn sidebar_after(
    items: Vec<SidebarItem>,
    events: Vec<Option<Event>>,
) -> (Snapshot, Vec<String>, Vec<bool>) {
    let selected: Log<String> = Arc::default();
    let collapsed: Log<bool> = Arc::default();
    let frame = settled(
        boxed(
            100,
            20,
            builder::sidebar()
                .items(items)
                .on_select(recorder(&selected))
                .on_collapse(recorder(&collapsed))
                .build(),
        ),
        ANY,
        events,
    );
    (frame, held(&selected), held(&collapsed))
}

// ---------------------------------------------------------------- the log view

/// `count` rows `Log row 001`... numbered from `first`.
fn rows(first: usize, count: usize) -> Vec<LogRow> {
    (first..first + count)
        .map(|n| LogRow::new(&format!("Log row {n:03}")))
        .collect()
}

/// A log of 100 rows, as NAV-009's falsifier gives it.
fn hundred() -> LogHandle {
    let handle = LogHandle::new();
    handle.append_all(rows(1, 100));
    handle
}

/// A change that appends `count` rows numbered from `first` to `handle`.
fn append(handle: &LogHandle, first: usize, count: usize) -> Change {
    let handle = handle.clone();
    Box::new(move || handle.append_all(rows(first, count)))
}

/// A change that puts `count` older rows, `Early row 01`..., above the rows
/// of `handle`.
fn prepend(handle: &LogHandle, count: usize) -> Change {
    let handle = handle.clone();
    Box::new(move || {
        handle.prepend(
            (1..=count)
                .map(|n| LogRow::new(&format!("Early row {n:02}")))
                .collect(),
        )
    })
}

/// A root that shows a focused log view of `handle` in a box of 100 by 20,
/// with `word` in place of `new` when one is given, and makes `changes` on
/// F2.
fn log_root(handle: &LogHandle, word: Option<&'static str>, changes: Vec<Change>) -> Scripted {
    let handle = handle.clone();
    Scripted::new(
        move |_| {
            let mut view = builder::log_view().handle(handle.clone());
            if let Some(word) = word {
                view = view.new_word(word);
            }
            boxed(100, 20, view.build().auto_focus())
        },
        changes,
    )
}

/// The frame a log root settles on after `events`.
fn log_after(
    handle: &LogHandle,
    word: Option<&'static str>,
    changes: Vec<Change>,
    events: Vec<Option<Event>>,
) -> Snapshot {
    last_of(run(log_root(handle, word, changes), "Log row", events))
}

/// Line `row` of a log view in the box of 100 cells, without the box's last
/// column, where its scroll bar stands.
fn log_line(frame: &Snapshot, row: u16) -> String {
    line(frame, row, 0..99)
}

/// The last line of a log view in the box of 100 by 20, every cell of it:
/// where the bar of new rows stands.
fn last_line(frame: &Snapshot) -> String {
    line(frame, 19, 0..100)
}

/// Every line of a log view in the box of 20 rows but its last, where the
/// bar of new rows stands.
fn upper_lines(frame: &Snapshot) -> Vec<String> {
    (0..19).map(|row| log_line(frame, row)).collect()
}

/// The rows of the box's last column that hold the scroll bar's thumb,
/// painted `text-muted`.
fn thumb(frame: &Snapshot) -> Vec<u16> {
    (0..20)
        .filter(|row| cell_colors(frame, 99, *row).0 == Some(role("text-muted")))
        .collect()
}

// ---------------------------------------------------------------- the list

/// DAT-008's list: Apple, Banana, a section Citrus of Lemon and Lime, and a
/// disabled Mango.
fn fruits() -> Vec<ListItem> {
    vec![
        ListItem::new("apple", "Apple"),
        ListItem::new("banana", "Banana"),
        ListItem::new("lemon", "Lemon").section("Citrus"),
        ListItem::new("lime", "Lime").section("Citrus"),
        ListItem::new("mango", "Mango").disabled(true),
    ]
}

/// The label of item `n` of a numbered list.
fn item(n: usize) -> String {
    format!("Item {n:06}")
}

/// `count` items `Item 000001`..., with the ids `i000001`....
fn numbered(count: usize) -> Vec<ListItem> {
    (1..=count)
        .map(|n| ListItem::new(&format!("i{n:06}"), &item(n)))
        .collect()
}

/// A default list of `items`.
fn list_of(items: Vec<ListItem>) -> Element {
    builder::list().items(items).build()
}

/// `list` in a box of 100 by 20.
fn in_box(list: Element) -> Element {
    boxed(100, 20, list)
}

/// The frame a focused list of `items` in a box of 100 by 20 settles on
/// after `events`, with what it reported through `on_select`.
fn list_after(
    items: Vec<ListItem>,
    text: &'static str,
    events: Vec<Option<Event>>,
) -> (Snapshot, Vec<String>) {
    let selected: Log<String> = Arc::default();
    let frame = settled(
        in_box(
            builder::list()
                .items(items)
                .on_select(recorder(&selected))
                .build()
                .auto_focus(),
        ),
        text,
        events,
    );
    (frame, held(&selected))
}

/// The cell that shows a frame of the spinner's glyph, as (column, row).
fn spinner_cell(frame: &Snapshot) -> Option<(u16, u16)> {
    let (rows, columns) = frame.screen.size();
    (0..rows)
        .flat_map(|row| (0..columns).map(move |column| (column, row)))
        .find(|(column, row)| {
            let shown = glyph(frame, *column, *row);
            SPINNER_FRAMES
                .iter()
                .chain(SPINNER_ASCII_FRAMES)
                .any(|frame| *frame == shown)
        })
}

// ---------------------------------------------------------------- NAV-007

#[test]
#[serial_test::serial(theme)]
fn nav_007_a_divider_is_border_on_hover_under_the_pointer_and_selection_when_focused_or_dragged() {
    let _theme = Active::set(probe());
    let line = upright();
    let (rest, _) = split_after(split_49_50, Vec::new());
    for row in 0..20 {
        assert_eq!(
            glyph(&rest, 49, row),
            line,
            "NAV-007: the divider is the separator piece's {line} on row {row}:\n{}",
            rest.text
        );
        assert_eq!(
            cell_colors(&rest, 49, row).0,
            Some(role("border")),
            "NAV-007: the divider is border on row {row}:\n{}",
            rest.text
        );
    }

    let (hovered, _) = split_after(split_49_50, vec![hover(49, 5)]);
    assert_eq!(
        cell_colors(&hovered, 49, 5),
        (Some(role("border")), Some(role("hover"))),
        "NAV-007: the divider under the pointer is border on hover:\n{}",
        hovered.text
    );

    // Tab gives the divider, the split's first focus stop, the focus.
    let (focused, _) = split_after(split_49_50, vec![key(KeyCode::Tab)]);
    for row in 0..20 {
        assert_eq!(
            cell_colors(&focused, 49, row),
            (Some(role("selection-foreground")), Some(role("selection"))),
            "NAV-007: the focused divider is selection-foreground on selection on row {row}:\n{}",
            focused.text
        );
    }

    // A press on the divider and a motion, the button still held.
    let (dragged, _) = split_after(
        split_49_50,
        vec![
            mouse(MouseEventKind::Down, 49, 5),
            mouse(MouseEventKind::Drag, 59, 5),
        ],
    );
    assert_eq!(
        glyph(&dragged, 59, 5),
        line,
        "NAV-007: the dragged divider follows the pointer:\n{}",
        dragged.text
    );
    assert_eq!(
        cell_colors(&dragged, 59, 5),
        (Some(role("selection-foreground")), Some(role("selection"))),
        "NAV-007: the dragged divider is selection-foreground on selection:\n{}",
        dragged.text
    );

    let (stacked_frame, _) = split_after(stacked, Vec::new());
    let line = flat();
    for column in 0..100 {
        assert_eq!(
            glyph(&stacked_frame, column, 9),
            line,
            "NAV-007: the divider between stacked panes is the separator piece's {line}:\n{}",
            stacked_frame.text
        );
        assert_eq!(
            cell_colors(&stacked_frame, column, 9).0,
            Some(role("border")),
            "NAV-007: the divider between stacked panes is border:\n{}",
            stacked_frame.text
        );
    }
}

#[test]
#[serial_test::serial(theme)]
fn nav_007_a_sidebar_paints_its_fill_edge_labels_icons_badge_and_toggle_by_role() {
    let _theme = Active::set(probe());
    let frame = shown(
        boxed(
            100,
            20,
            builder::sidebar()
                .items(more_items(true))
                .active("settings")
                .build(),
        ),
        "Settings",
    );
    let line = upright();
    for row in 0..20 {
        assert_eq!(
            glyph(&frame, EDGE, row),
            line,
            "NAV-007: a {line} runs along the edge that faces the content on row {row}:\n{}",
            frame.text
        );
        assert_eq!(
            cell_colors(&frame, EDGE, row).0,
            Some(role("border")),
            "NAV-007: the edge is border on row {row}:\n{}",
            frame.text
        );
    }
    for column in [0, EDGE - 1] {
        assert_eq!(
            cell_colors(&frame, column, EMPTY_ROW).1,
            Some(role("surface")),
            "NAV-007: the sidebar is on surface:\n{}",
            frame.text
        );
    }
    assert_eq!(
        colors(&frame, "Home"),
        (Some(role("foreground")), Some(role("surface"))),
        "NAV-007: an item's label is foreground on surface:\n{}",
        frame.text
    );
    let (label, row) = at(&frame, "Projects");
    let icon = column_on(&frame, row, Icon::Folder.glyph());
    let expander = column_on(&frame, row, Icon::ChevronRight.glyph());
    assert!(
        expander < icon && icon < label,
        "NAV-007: a closed group's row is its expander, its icon, then its label:\n{}",
        frame.text
    );
    assert_eq!(
        cell_colors(&frame, icon, row).0,
        Some(role("text-muted")),
        "NAV-007: an item's icon is text-muted:\n{}",
        frame.text
    );
    assert_eq!(
        cell_colors(&frame, expander, row).0,
        Some(role("text-muted")),
        "NAV-007: a group's expander is text-muted:\n{}",
        frame.text
    );
    assert_eq!(
        colors(&frame, "Settings"),
        (Some(role("accent-foreground")), Some(role("accent"))),
        "NAV-007: the active item is accent-foreground on accent:\n{}",
        frame.text
    );
    assert_eq!(
        colors(&frame, "Archive").0,
        Some(role("text-muted")),
        "NAV-007: a disabled item is text-muted:\n{}",
        frame.text
    );
    let (toggle, toggle_row) = at(&frame, Icon::ChevronLeft.glyph());
    assert_eq!(
        cell_colors(&frame, toggle, toggle_row).0,
        Some(role("text-muted")),
        "NAV-007: the toggle is text-muted:\n{}",
        frame.text
    );
    let piece = badge_cells();
    let (_, inbox) = at(&frame, "Inbox");
    let start = EDGE - piece.len() as u16;
    assert_eq!(
        cells(&frame, inbox, start..EDGE),
        piece,
        "NAV-007: the badge is painted as the badge piece paints it, at the row's end:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn nav_007_a_sidebars_cursor_hovered_item_and_collapsed_label_take_their_roles() {
    let _theme = Active::set(probe());
    let with_active = || {
        boxed(
            100,
            20,
            builder::sidebar()
                .items(nav_items())
                .active("settings")
                .build(),
        )
    };
    // Tab gives the item tree the focus; Home puts the cursor on Home, away
    // from the active Settings.
    let focused = settled(
        with_active(),
        "Home",
        vec![key(KeyCode::Tab), key(KeyCode::Home)],
    );
    assert_eq!(
        colors(&focused, "Home"),
        (Some(role("selection-foreground")), Some(role("selection"))),
        "NAV-007: the cursor's item is selection while the sidebar holds the focus:\n{}",
        focused.text
    );
    assert_eq!(
        colors(&focused, "Settings"),
        (Some(role("accent-foreground")), Some(role("accent"))),
        "NAV-007: the active item away from the cursor stays accent:\n{}",
        focused.text
    );

    let base = shown(with_active(), "Projects");
    let (column, row) = at(&base, "Projects");
    let hovered = settled(with_active(), "Projects", vec![hover(column + 2, row)]);
    assert_eq!(
        colors(&hovered, "Projects").1,
        Some(role("hover")),
        "NAV-007: the item under the pointer is on hover:\n{}",
        hovered.text
    );

    let collapsed = settled(collapsed_sidebar(), ANY, vec![key(KeyCode::Tab)]);
    let (column, row) = at(&collapsed, "Home");
    assert!(
        column >= 3,
        "NAV-007: the cursor item's label stands beside the collapsed bar:\n{}",
        collapsed.text
    );
    assert_eq!(
        cell_colors(&collapsed, column, row),
        (Some(role("foreground")), Some(role("surface"))),
        "NAV-007: the label beside a collapsed bar is painted as NAV-001 paints a tooltip:\n{}",
        collapsed.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn nav_007_a_log_view_paints_its_rows_by_kind_its_scroll_bar_and_its_new_bar_by_role() {
    let _theme = Active::set(probe());
    let handle = LogHandle::new();
    handle.append_all(rows(1, 95));
    handle.append_all(vec![
        LogRow::new("Plain row").kind(LogKind::Plain),
        LogRow::new("Info row").kind(LogKind::Info),
        LogRow::new("Success row").kind(LogKind::Success),
        LogRow::new("Warning row").kind(LogKind::Warning),
        LogRow::new("Error row").kind(LogKind::Error),
    ]);
    let frame = log_after(&handle, None, Vec::new(), Vec::new());
    for (text, name) in [
        ("Log row 090", "foreground"),
        ("Plain row", "foreground"),
        ("Info row", "info"),
        ("Success row", "success"),
        ("Warning row", "warning"),
        ("Error row", "error"),
    ] {
        assert_eq!(
            colors(&frame, text).0,
            Some(role(name)),
            "NAV-007: the row {text:?} is {name}:\n{}",
            frame.text
        );
    }
    let bar: Vec<Option<Rgb>> = (0..20).map(|row| cell_colors(&frame, 99, row).0).collect();
    assert!(
        bar.iter()
            .all(|color| *color == Some(role("border")) || *color == Some(role("text-muted"))),
        "NAV-007: the scroll bar is its track in border and its thumb in text-muted:\n{}",
        frame.text
    );
    assert_eq!(
        bar[19],
        Some(role("text-muted")),
        "NAV-007: the thumb of a view at its last row stands at the bottom, text-muted:\n{}",
        frame.text
    );
    assert_eq!(
        bar[0],
        Some(role("border")),
        "NAV-007: the track above the thumb is border:\n{}",
        frame.text
    );

    let handle = hundred();
    let frame = log_after(
        &handle,
        None,
        vec![append(&handle, 101, 3)],
        vec![key(KeyCode::PageUp), change()],
    );
    let (column, row) = at(&frame, "↓ 3 new");
    for offset in 0..7 {
        assert_eq!(
            cell_colors(&frame, column + offset, row),
            (Some(role("primary-foreground")), Some(role("primary"))),
            "NAV-007: the new bar is primary-foreground on primary:\n{}",
            frame.text
        );
    }
}

#[test]
#[serial_test::serial(theme)]
fn nav_007_a_split_a_sidebar_and_a_log_view_paint_the_same_cells_from_their_props_and_builders() {
    let _theme = Active::set(probe());
    let twins: Vec<(&str, Element, Element, &str)> = vec![
        (
            "split",
            Element::typed::<Resizable>(ResizableProps {
                direction: SplitDirection::Row,
                panes: vec![Pane::new(filled(0)).size(49), Pane::new(filled(1))],
                aria_label: None,
                disabled: false,
            }),
            plain_split_49_50(),
            "First pane",
        ),
        (
            "sidebar",
            Element::typed::<Sidebar>(SidebarProps {
                items: more_items(true),
                active: Some("settings".to_owned()),
                collapsed: false,
                header: None,
                footer: None,
                aria_label: None,
                width: None,
            }),
            builder::sidebar()
                .items(more_items(true))
                .active("settings")
                .build(),
            "Settings",
        ),
        (
            "log view",
            Element::typed::<LogView>(LogViewProps {
                handle: hundred(),
                new_word: None,
                aria_label: None,
                width: None,
                height: None,
            }),
            builder::log_view().handle(hundred()).build(),
            "Log row",
        ),
    ];
    for (name, from_props, from_builder, text) in twins {
        let props = shown(boxed(100, 20, from_props), text);
        let built = shown(boxed(100, 20, from_builder), text);
        assert!(
            paint(&props) == paint(&built),
            "NAV-007: the {name} built through its builder paints other cells than from its props:\n{}\n---\n{}",
            props.text,
            built.text
        );
    }
}

#[test]
#[serial_test::serial(theme)]
fn nav_007_focus_hover_an_active_item_and_disabled_change_no_glyph_or_text() {
    let _theme = Active::set(probe());
    let (rest, _) = split_after(split_49_50, Vec::new());
    let disabled = |_: &Log<Vec<u16>>| {
        builder::resizable()
            .pane_with(Pane::new(filled(0)).size(49))
            .pane(filled(1))
            .disabled(true)
            .build()
    };
    for (what, (frame, _)) in [
        (
            "the focus",
            split_after(split_49_50, vec![key(KeyCode::Tab)]),
        ),
        ("the pointer", split_after(split_49_50, vec![hover(49, 5)])),
        ("disabled", split_after(disabled, Vec::new())),
    ] {
        assert_eq!(
            frame.text, rest.text,
            "NAV-007: {what} changes no glyph or text of a split"
        );
    }

    let sidebar = |active: Option<&str>, archive_disabled: bool| {
        let mut built = builder::sidebar().items(more_items(archive_disabled));
        if let Some(id) = active {
            built = built.active(id);
        }
        boxed(100, 20, built.build())
    };
    let plain = shown(sidebar(None, false), "Home");
    let (column, row) = at(&plain, "Projects");
    for (what, frame) in [
        (
            "an active item",
            shown(sidebar(Some("settings"), false), "Home"),
        ),
        (
            "the focus",
            settled(
                sidebar(None, false),
                "Home",
                vec![key(KeyCode::Tab), key(KeyCode::Down)],
            ),
        ),
        (
            "the pointer",
            settled(sidebar(None, false), "Home", vec![hover(column + 2, row)]),
        ),
        ("a disabled item", shown(sidebar(None, true), "Home")),
    ] {
        assert_eq!(
            frame.text, plain.text,
            "NAV-007: {what} changes no glyph or text of a sidebar"
        );
    }

    let unfocused = shown(
        boxed(100, 20, builder::log_view().handle(hundred()).build()),
        "Log row",
    );
    let focused = shown(
        boxed(
            100,
            20,
            builder::log_view().handle(hundred()).build().auto_focus(),
        ),
        "Log row",
    );
    assert_eq!(
        focused.text, unfocused.text,
        "NAV-007: the focus changes no glyph or text of a log view"
    );
}

#[test]
#[serial_test::serial(theme)]
fn nav_007_a_split_a_sidebar_and_a_log_view_take_every_color_from_the_theme() {
    let _theme = Active::set(probe());
    let base = shown(sidebar_of(more_items(true)), "Home");
    let (column, row) = at(&base, "Projects");
    let kinds = LogHandle::new();
    kinds.append_all(rows(1, 30));
    kinds.append_all(vec![
        LogRow::new("Info row").kind(LogKind::Info),
        LogRow::new("Success row").kind(LogKind::Success),
        LogRow::new("Warning row").kind(LogKind::Warning),
        LogRow::new("Error row").kind(LogKind::Error),
    ]);
    let handle = hundred();
    let frames: Vec<(&str, Snapshot)> = vec![
        ("split", split_after(split_49_50, Vec::new()).0),
        (
            "focused split",
            split_after(split_49_50, vec![key(KeyCode::Tab)]).0,
        ),
        (
            "hovered split",
            split_after(split_49_50, vec![hover(49, 5)]).0,
        ),
        ("stacked split", split_after(stacked, Vec::new()).0),
        (
            "sidebar with an active, a badged and a disabled item",
            shown(
                boxed(
                    100,
                    20,
                    builder::sidebar()
                        .items(more_items(true))
                        .active("settings")
                        .build(),
                ),
                "Home",
            ),
        ),
        (
            "focused sidebar",
            settled(
                sidebar_of(more_items(true)),
                "Home",
                vec![key(KeyCode::Tab), key(KeyCode::Down)],
            ),
        ),
        (
            "hovered sidebar",
            settled(
                sidebar_of(more_items(true)),
                "Home",
                vec![hover(column + 2, row)],
            ),
        ),
        (
            "collapsed sidebar with the cursor's label beside it",
            settled(collapsed_sidebar(), ANY, vec![key(KeyCode::Tab)]),
        ),
        (
            "log view with rows of every kind",
            log_after(&kinds, None, Vec::new(), Vec::new()),
        ),
        (
            "log view with its new bar",
            log_after(
                &handle,
                None,
                vec![append(&handle, 101, 5)],
                vec![key(KeyCode::PageUp), change()],
            ),
        ),
    ];
    let wrong = foreign_in(&frames);
    assert!(
        wrong.is_empty(),
        "NAV-007: a widget paints a color the theme does not define:\n{}",
        wrong.join("\n")
    );
}

// ---------------------------------------------------------------- NAV-008

#[test]
#[serial_test::serial(theme)]
fn nav_008_a_default_split_of_two_panes_shares_99_cells_inside_a_box_of_100_by_20() {
    let _theme = Active::set(probe());
    let (frame, _) = split_after(|log| split(&[None, None], log), Vec::new());
    for row in 0..20 {
        let sizes = widths(&frame, row, 2);
        assert_eq!(
            sizes[0] + sizes[1],
            99,
            "NAV-008: the panes share the box's 100 cells less the divider's on row {row}:\n{}",
            frame.text
        );
        assert!(
            sizes[0].abs_diff(sizes[1]) <= 1,
            "NAV-008: two panes with no size differ by one cell at most, not {sizes:?}:\n{}",
            frame.text
        );
        assert_eq!(
            on_role(&frame, row, "primary").first(),
            Some(&0),
            "NAV-008: the first pane starts at the box's first cell:\n{}",
            frame.text
        );
        assert_eq!(
            dividers(&frame, row),
            vec![sizes[0]],
            "NAV-008: the divider stands between the panes on row {row}:\n{}",
            frame.text
        );
        assert_eq!(
            on_role(&frame, row, "secondary").last(),
            Some(&99),
            "NAV-008: the second pane ends at the box's last cell:\n{}",
            frame.text
        );
    }
    assert_eq!(
        widths(&frame, 20, 2),
        vec![0, 0],
        "NAV-008: the panes stop at the box's 20 rows:\n{}",
        frame.text
    );
    assert!(
        dividers(&frame, 20).is_empty(),
        "NAV-008: the divider stops at the box's 20 rows:\n{}",
        frame.text
    );

    let (frame, _) = split_after(
        |_| {
            builder::resizable()
                .pane(filled(0))
                .pane(filled(1))
                .class("w-60 h-10")
                .build()
        },
        Vec::new(),
    );
    for row in 0..10 {
        let sizes = widths(&frame, row, 2);
        assert_eq!(
            sizes[0] + sizes[1],
            59,
            "NAV-008: w-60 makes the split 60 cells wide on row {row}:\n{}",
            frame.text
        );
    }
    assert_eq!(
        widths(&frame, 10, 2),
        vec![0, 0],
        "NAV-008: h-10 makes the split 10 rows tall:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn nav_008_panes_share_by_their_sizes_and_weights_and_none_falls_below_its_minimum() {
    let _theme = Active::set(probe());
    let (frame, _) = split_after(
        |_| {
            builder::resizable()
                .pane_with(Pane::new(filled(0)).size(20))
                .pane_with(Pane::new(filled(1)).weight(1.0))
                .pane_with(Pane::new(filled(2)).weight(2.0))
                .build()
        },
        Vec::new(),
    );
    assert_eq!(
        widths(&frame, 5, 3),
        vec![20, 26, 52],
        "NAV-008: of 98 cells the sized pane takes its 20 and the weights 1 and 2 share the 78 left:\n{}",
        frame.text
    );

    let (frame, _) = split_after(|log| split(&[None, None, None], log), Vec::new());
    let sizes = widths(&frame, 5, 3);
    assert_eq!(
        sizes.iter().sum::<u16>(),
        98,
        "NAV-008: three panes share 98 cells:\n{}",
        frame.text
    );
    assert!(
        sizes.iter().max().unwrap() - sizes.iter().min().unwrap() <= 1,
        "NAV-008: three panes with no size share equally, not {sizes:?}:\n{}",
        frame.text
    );

    let (frame, _) = split_after(|log| split(&[Some(2), None], log), Vec::new());
    assert_eq!(
        widths(&frame, 5, 2),
        vec![4, 95],
        "NAV-008: a pane asked for 2 cells keeps its minimum of 4:\n{}",
        frame.text
    );

    let (frame, _) = split_after(
        |_| {
            builder::resizable()
                .pane_with(Pane::new(filled(0)).size(5).min(10))
                .pane(filled(1))
                .build()
        },
        Vec::new(),
    );
    assert_eq!(
        widths(&frame, 5, 2),
        vec![10, 89],
        "NAV-008: a minimum the application sets holds:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn nav_008_a_split_widened_from_100_to_200_cells_keeps_each_panes_share() {
    let _theme = Active::set(probe());
    for (what, sizes, events) in [
        (
            "sizes of 30 and 69 cells from the application",
            vec![Some(30), None],
            vec![change()],
        ),
        (
            "panes the keys moved to 30 and 69 cells",
            vec![Some(49), None],
            vec![
                key(KeyCode::Tab),
                shift(KeyCode::Left),
                shift(KeyCode::Left),
                key(KeyCode::Right),
                change(),
            ],
        ),
    ] {
        let frames = run(widening(sizes), ANY, events);
        let before = frames
            .iter()
            .rev()
            .find(|frame| widths(frame, 5, 2).iter().sum::<u16>() == 99)
            .expect("a frame of the box of 100 cells");
        assert_eq!(
            widths(before, 5, 2),
            vec![30, 69],
            "NAV-008: with {what} the panes are 30 and 69 cells before the box widens:\n{}",
            before.text
        );
        let after = last(&frames);
        let sizes = widths(after, 5, 2);
        assert_eq!(
            sizes[0] + sizes[1],
            199,
            "NAV-008: with {what} the panes share the widened box's 199 cells:\n{}",
            after.text
        );
        assert!(
            sizes[0] == 60 || sizes[0] == 61,
            "NAV-008: with {what} the first pane keeps its share, 30 of 99, so 60 or 61 of 199, not {}:\n{}",
            sizes[0],
            after.text
        );
    }
}

#[test]
#[serial_test::serial(theme)]
fn nav_008_a_default_sidebar_is_28_cells_wide_3_while_collapsed_and_fills_the_height_of_its_box() {
    let _theme = Active::set(probe());
    let line = upright();
    let frame = shown(sidebar_of(nav_items()), "Home");
    for row in 0..20 {
        assert_eq!(
            bar_width(&frame, row),
            BAR,
            "NAV-008: a default sidebar is 28 cells wide on row {row}:\n{}",
            frame.text
        );
        assert_eq!(
            glyph(&frame, EDGE, row),
            line,
            "NAV-008: the sidebar's edge stands in its 28th cell on row {row}:\n{}",
            frame.text
        );
    }
    assert_eq!(
        bar_width(&frame, 20),
        0,
        "NAV-008: the sidebar is the box's 20 rows tall:\n{}",
        frame.text
    );
    assert_eq!(
        at(&frame, Icon::ChevronLeft.glyph()).1,
        19,
        "NAV-008: the toggle stands on the last row:\n{}",
        frame.text
    );

    let frame = shown(collapsed_sidebar(), ANY);
    for row in 0..20 {
        assert_eq!(
            bar_width(&frame, row),
            3,
            "NAV-008: a collapsed sidebar is 3 cells wide on row {row}:\n{}",
            frame.text
        );
        assert_eq!(
            glyph(&frame, 2, row),
            line,
            "NAV-008: a collapsed sidebar's edge stands in its third cell on row {row}:\n{}",
            frame.text
        );
    }
    assert_eq!(
        bar_width(&frame, 20),
        0,
        "NAV-008: a collapsed sidebar is the box's 20 rows tall:\n{}",
        frame.text
    );
    assert!(
        column_on(&frame, 19, Icon::ChevronRight.glyph()) < 3,
        "NAV-008: the collapsed toggle stands on the bar's last row:\n{}",
        frame.text
    );

    for (what, sidebar) in [
        (
            "w-40 on its builder",
            builder::sidebar().items(nav_items()).class("w-40").build(),
        ),
        (
            "a width of 40 on its props",
            Element::typed::<Sidebar>(SidebarProps {
                items: nav_items(),
                active: None,
                collapsed: false,
                header: None,
                footer: None,
                aria_label: None,
                width: Some(40),
            }),
        ),
    ] {
        let frame = shown(boxed(100, 20, sidebar), "Home");
        assert_eq!(
            bar_width(&frame, EMPTY_ROW),
            40,
            "NAV-008: {what} makes the sidebar 40 cells wide:\n{}",
            frame.text
        );
    }
    let frame = shown(
        boxed(
            100,
            20,
            builder::sidebar().items(nav_items()).class("h-10").build(),
        ),
        "Home",
    );
    assert_eq!(
        (bar_width(&frame, 9), bar_width(&frame, 10)),
        (BAR, 0),
        "NAV-008: h-10 makes the sidebar 10 rows tall:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn nav_008_a_sidebar_lays_out_its_header_items_footer_and_toggle_and_indents_a_child_two_cells() {
    let _theme = Active::set(probe());
    let frame = shown(
        boxed(
            100,
            20,
            builder::sidebar()
                .items(nav_items_open(true))
                .header(Element::text("Workspace"))
                .footer(Element::text("Signed in"))
                .build(),
        ),
        "Signed in",
    );
    let row = |label: &str| at(&frame, label).1;
    assert!(
        row("Workspace") < row("Home"),
        "NAV-008: the header stands above the items:\n{}",
        frame.text
    );
    let (projects, projects_row) = at(&frame, "Projects");
    assert_eq!(
        projects_row,
        row("Home") + 1,
        "NAV-008: one row for each item:\n{}",
        frame.text
    );
    let (alpha, alpha_row) = at(&frame, "Alpha");
    assert_eq!(
        alpha_row,
        projects_row + 1,
        "NAV-008: an open group's children follow it:\n{}",
        frame.text
    );
    assert_eq!(
        alpha,
        projects + 2,
        "NAV-008: a child's label stands two cells right of its group's:\n{}",
        frame.text
    );
    assert_eq!(
        at(&frame, "Beta"),
        (alpha, alpha_row + 1),
        "NAV-008: the second child under the first:\n{}",
        frame.text
    );
    assert_eq!(
        row("Settings"),
        alpha_row + 2,
        "NAV-008: the next item after the children:\n{}",
        frame.text
    );
    assert!(
        row("Signed in") > row("Settings") && row("Signed in") < 19,
        "NAV-008: the footer stands below the items and above the toggle:\n{}",
        frame.text
    );
    assert_eq!(
        at(&frame, Icon::ChevronLeft.glyph()).1,
        19,
        "NAV-008: the toggle stands on the last row:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn nav_008_a_long_label_is_cut_with_an_ellipsis_and_a_badge_ends_its_row() {
    let _theme = Active::set(probe());
    const LONG: &str = "Quarterly planning and roadmap documents";
    assert_eq!(
        LONG.chars().count(),
        40,
        "the falsifier's label of 40 characters"
    );
    let frame = shown(
        boxed(
            100,
            20,
            builder::sidebar()
                .item(SidebarItem::new("plans", LONG).icon(Icon::File))
                .item(
                    SidebarItem::new("inbox", "Inbox")
                        .icon(Icon::File)
                        .badge("3"),
                )
                .build(),
        ),
        "Inbox",
    );
    assert!(
        find(&frame, LONG).is_none(),
        "NAV-008: a label of 40 characters is cut in a bar of 28 cells:\n{}",
        frame.text
    );
    let (start, row) = at(&frame, "Quarterly");
    let ellipsis = column_on(&frame, row, "…");
    assert!(
        ellipsis < EDGE,
        "NAV-008: the cut label ends with … inside the bar:\n{}",
        frame.text
    );
    let kept = line(&frame, row, start..ellipsis);
    assert!(
        LONG.starts_with(&kept),
        "NAV-008: the cut label is the label's start, not {kept:?}:\n{}",
        frame.text
    );
    let text = badge_text(&three()).expect("a badge with a text shows");
    let width = text.chars().count() as u16;
    let (_, inbox) = at(&frame, "Inbox");
    assert_eq!(
        line(&frame, inbox, EDGE - width..EDGE),
        text.trim_end(),
        "NAV-008: the badge stands at the row's end:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn nav_008_a_default_log_view_fills_a_box_of_100_by_20_and_wraps_a_long_row() {
    let _theme = Active::set(probe());
    let frame = log_after(&hundred(), None, Vec::new(), Vec::new());
    for row in 0..20 {
        assert_eq!(
            log_line(&frame, row),
            format!("Log row {:03}", 81 + row),
            "NAV-008: a default log view shows 20 rows, its last row on its last line:\n{}",
            frame.text
        );
        assert!(
            [Some(role("border")), Some(role("text-muted"))]
                .contains(&cell_colors(&frame, 99, row).0)
                && !glyph(&frame, 99, row).trim().is_empty(),
            "NAV-008: the view reaches the box's last cell, where its scroll bar stands, on row {row}:\n{}",
            frame.text
        );
    }
    assert!(
        line(&frame, 20, 0..100).is_empty(),
        "NAV-008: the log view stops at the box's 20 rows:\n{}",
        frame.text
    );

    let long = "0123456789".repeat(25);
    let handle = LogHandle::new();
    handle.append(LogRow::new(&long));
    let frame = shown(
        boxed(100, 20, builder::log_view().handle(handle).build()),
        "0123456789",
    );
    let (_, first) = at(&frame, "0123456789");
    let lines: Vec<String> = (first..first + 3)
        .map(|row| line(&frame, row, 0..100))
        .collect();
    assert_eq!(
        lines.concat(),
        long,
        "NAV-008: a row of 250 characters continues on the next lines:\n{}",
        frame.text
    );
    assert!(
        lines.iter().all(|text| !text.is_empty()) && line(&frame, first + 3, 0..100).is_empty(),
        "NAV-008: a row of 250 characters takes three lines:\n{}",
        frame.text
    );

    for (what, view) in [
        (
            "w-40 h-10 on its builder",
            builder::log_view()
                .handle(hundred())
                .class("w-40 h-10")
                .build(),
        ),
        (
            "a width of 40 and a height of 10 on its props",
            Element::typed::<LogView>(LogViewProps {
                handle: hundred(),
                new_word: None,
                aria_label: None,
                width: Some(40),
                height: Some(10),
            }),
        ),
    ] {
        let frame = shown(boxed(100, 20, view), "Log row");
        for row in 0..10 {
            assert_eq!(
                line(&frame, row, 0..39),
                format!("Log row {:03}", 91 + row),
                "NAV-008: {what} shows 10 rows:\n{}",
                frame.text
            );
            assert!(
                !glyph(&frame, 39, row).trim().is_empty() && line(&frame, row, 40..100).is_empty(),
                "NAV-008: {what} makes the view 40 cells wide:\n{}",
                frame.text
            );
        }
        assert!(
            line(&frame, 10, 0..100).is_empty(),
            "NAV-008: {what} makes the view 10 rows tall:\n{}",
            frame.text
        );
    }
}

// ---------------------------------------------------------------- NAV-009

#[test]
#[serial_test::serial(theme)]
fn nav_009_a_divider_follows_a_drag_by_the_cell_and_reports_once_per_move() {
    let _theme = Active::set(probe());
    let (frame, reported) = split_after(split_49_50, drag((49, 5), (59, 5)));
    assert_eq!(
        widths(&frame, 5, 2),
        vec![59, 40],
        "NAV-009: a drag of the divider ten cells right makes the first pane 59 cells:\n{}",
        frame.text
    );
    assert_eq!(
        dividers(&frame, 5),
        vec![59],
        "NAV-009: the divider stands where the pointer let go:\n{}",
        frame.text
    );
    assert_eq!(
        reported,
        vec![vec![59, 40]],
        "NAV-009: the one move is reported once:\n{}",
        frame.text
    );

    let (frame, reported) = split_after(
        split_49_50,
        vec![
            mouse(MouseEventKind::Down, 49, 5),
            mouse(MouseEventKind::Drag, 52, 5),
            mouse(MouseEventKind::Drag, 59, 5),
            mouse(MouseEventKind::Up, 59, 5),
        ],
    );
    assert_eq!(
        reported,
        vec![vec![52, 47], vec![59, 40]],
        "NAV-009: each motion of a drag is one move, reported once:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn nav_009_a_focused_divider_moves_one_cell_ten_with_shift_and_to_its_ends_on_home_and_end() {
    let _theme = Active::set(probe());
    for (what, event, expected) in [
        ("Right", key(KeyCode::Right), vec![50, 49]),
        ("Shift+Right", shift(KeyCode::Right), vec![59, 40]),
        ("End", key(KeyCode::End), vec![95, 4]),
        ("Left", key(KeyCode::Left), vec![48, 51]),
        ("Shift+Left", shift(KeyCode::Left), vec![39, 60]),
        ("Home", key(KeyCode::Home), vec![4, 95]),
    ] {
        let (frame, reported) = split_after(split_49_50, vec![key(KeyCode::Tab), event]);
        assert_eq!(
            widths(&frame, 5, 2),
            expected,
            "NAV-009: {what} on the focused divider:\n{}",
            frame.text
        );
        assert_eq!(
            reported,
            vec![expected.clone()],
            "NAV-009: {what} reports the panes' sizes once:\n{}",
            frame.text
        );
    }
    let (frame, reported) = split_after(
        split_49_50,
        vec![
            key(KeyCode::Tab),
            key(KeyCode::Right),
            key(KeyCode::Left),
            shift(KeyCode::Right),
            key(KeyCode::End),
        ],
    );
    assert_eq!(
        reported,
        vec![vec![50, 49], vec![49, 50], vec![59, 40], vec![95, 4]],
        "NAV-009: each move is reported once, in order:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn nav_009_a_divider_takes_from_the_pane_beside_it_then_from_the_next_each_down_to_its_minimum() {
    let _theme = Active::set(probe());
    let (frame, reported) = split_after(three_panes, vec![key(KeyCode::Tab), key(KeyCode::End)]);
    assert_eq!(
        widths(&frame, 5, 3),
        vec![90, 4, 4],
        "NAV-009: End on the first divider takes the second and the third pane down to 4:\n{}",
        frame.text
    );
    assert_eq!(
        reported,
        vec![vec![90, 4, 4]],
        "NAV-009: End reports the sizes once:\n{}",
        frame.text
    );

    let (frame, reported) = split_after(
        three_panes,
        vec![key(KeyCode::Tab), key(KeyCode::Tab), key(KeyCode::Home)],
    );
    assert_eq!(
        widths(&frame, 5, 3),
        vec![4, 4, 90],
        "NAV-009: Home on the second divider takes the second and the first pane down to 4:\n{}",
        frame.text
    );
    assert_eq!(
        reported,
        vec![vec![4, 4, 90]],
        "NAV-009: Home reports the sizes once:\n{}",
        frame.text
    );

    let (frame, reported) = split_after(three_panes, drag((33, 5), (80, 5)));
    assert_eq!(
        widths(&frame, 5, 3),
        vec![80, 4, 14],
        "NAV-009: a drag past the second pane's minimum takes the rest from the third:\n{}",
        frame.text
    );
    assert_eq!(
        reported,
        vec![vec![80, 4, 14]],
        "NAV-009: the drag's one move is reported once:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn nav_009_a_stacked_split_moves_its_divider_on_up_and_down() {
    let _theme = Active::set(probe());
    let line = flat();
    for (what, event, expected) in [
        ("Down", key(KeyCode::Down), vec![10, 9]),
        ("Up", key(KeyCode::Up), vec![8, 11]),
        ("End", key(KeyCode::End), vec![15, 4]),
    ] {
        let (frame, reported) = split_after(stacked, vec![key(KeyCode::Tab), event]);
        assert_eq!(
            heights(&frame, 50, 2),
            expected,
            "NAV-009: {what} on the focused divider between stacked panes:\n{}",
            frame.text
        );
        assert_eq!(
            glyph(&frame, 50, expected[0]),
            line,
            "NAV-009: the divider stands under the first pane after {what}:\n{}",
            frame.text
        );
        assert_eq!(
            reported,
            vec![expected.clone()],
            "NAV-009: {what} reports the sizes once:\n{}",
            frame.text
        );
    }
}

#[test]
#[serial_test::serial(theme)]
fn nav_009_a_sidebars_cursor_moves_over_its_visible_items_on_up_down_home_and_end() {
    let _theme = Active::set(probe());
    let labels = ["Home", "Projects", "Alpha", "Beta", "Settings"];
    for (events, expected, what) in [
        (
            vec![key(KeyCode::Tab)],
            "Home",
            "the item tree takes the focus with the cursor on Home",
        ),
        (
            vec![key(KeyCode::Tab), key(KeyCode::Down)],
            "Projects",
            "Down",
        ),
        (
            vec![key(KeyCode::Tab), key(KeyCode::Down), key(KeyCode::Down)],
            "Settings",
            "Down past a closed group's hidden children",
        ),
        (
            vec![key(KeyCode::Tab), key(KeyCode::End)],
            "Settings",
            "End",
        ),
        (
            vec![key(KeyCode::Tab), key(KeyCode::End), key(KeyCode::Up)],
            "Projects",
            "Up",
        ),
        (
            vec![key(KeyCode::Tab), key(KeyCode::End), key(KeyCode::Home)],
            "Home",
            "Home",
        ),
    ] {
        let (frame, _, _) = sidebar_after(nav_items(), events);
        assert_eq!(
            cursor_among(&frame, &labels),
            vec![expected],
            "NAV-009: {what}:\n{}",
            frame.text
        );
    }
}

#[test]
#[serial_test::serial(theme)]
fn nav_009_right_opens_a_group_and_moves_into_it_and_left_moves_back_and_closes_it() {
    let _theme = Active::set(probe());
    let labels = ["Home", "Projects", "Alpha", "Beta", "Settings"];
    let down = || vec![key(KeyCode::Tab), key(KeyCode::Down)];
    let (frame, _, _) = sidebar_after(nav_items(), [down(), vec![key(KeyCode::Right)]].concat());
    assert!(
        find(&frame, "Alpha").is_some() && find(&frame, "Beta").is_some(),
        "NAV-009: Right opens the closed group Projects:\n{}",
        frame.text
    );
    assert_eq!(
        cursor_among(&frame, &labels),
        vec!["Projects"],
        "NAV-009: the cursor stays on the group it opened:\n{}",
        frame.text
    );

    let (frame, _, _) = sidebar_after(
        nav_items(),
        [down(), vec![key(KeyCode::Right), key(KeyCode::Right)]].concat(),
    );
    assert_eq!(
        cursor_among(&frame, &labels),
        vec!["Alpha"],
        "NAV-009: Right on an open group moves into it:\n{}",
        frame.text
    );

    let (frame, _, _) = sidebar_after(
        nav_items(),
        [
            down(),
            vec![key(KeyCode::Right), key(KeyCode::Right), key(KeyCode::Left)],
        ]
        .concat(),
    );
    assert_eq!(
        cursor_among(&frame, &labels),
        vec!["Projects"],
        "NAV-009: Left moves from a child to its group:\n{}",
        frame.text
    );
    assert!(
        find(&frame, "Alpha").is_some(),
        "NAV-009: Left from a child leaves its group open:\n{}",
        frame.text
    );

    let (frame, _, _) = sidebar_after(
        nav_items(),
        [
            down(),
            vec![
                key(KeyCode::Right),
                key(KeyCode::Right),
                key(KeyCode::Left),
                key(KeyCode::Left),
            ],
        ]
        .concat(),
    );
    assert!(
        find(&frame, "Alpha").is_none(),
        "NAV-009: Left on an open group closes it:\n{}",
        frame.text
    );
    assert_eq!(
        cursor_among(&frame, &labels),
        vec!["Projects"],
        "NAV-009: the cursor stays on the group it closed:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn nav_009_enter_space_and_a_click_make_an_item_active_and_report_it() {
    let _theme = Active::set(probe());
    let to_settings = || vec![key(KeyCode::Tab), key(KeyCode::End)];
    for (what, activate) in [
        ("Enter", key(KeyCode::Enter)),
        ("Space", key(KeyCode::Char(' '))),
    ] {
        // Home moves the cursor away, so the active item's own color shows.
        let (frame, selected, _) = sidebar_after(
            nav_items(),
            [to_settings(), vec![activate, key(KeyCode::Home)]].concat(),
        );
        assert_eq!(
            selected,
            vec!["settings"],
            "NAV-009: {what} on Settings reports it through on_select:\n{}",
            frame.text
        );
        assert_eq!(
            colors(&frame, "Settings"),
            (Some(role("accent-foreground")), Some(role("accent"))),
            "NAV-009: {what} makes Settings the active item:\n{}",
            frame.text
        );
        assert_eq!(
            colors(&frame, "Projects"),
            (Some(role("foreground")), Some(role("surface"))),
            "NAV-009: no other item is active:\n{}",
            frame.text
        );
    }

    let (frame, _, _) = sidebar_after(
        nav_items(),
        vec![key(KeyCode::Tab), key(KeyCode::Down), key(KeyCode::Enter)],
    );
    assert!(
        find(&frame, "Alpha").is_some(),
        "NAV-009: Enter on a closed group opens it:\n{}",
        frame.text
    );
    let (frame, _, _) = sidebar_after(
        nav_items(),
        vec![
            key(KeyCode::Tab),
            key(KeyCode::Down),
            key(KeyCode::Enter),
            key(KeyCode::Enter),
        ],
    );
    assert!(
        find(&frame, "Alpha").is_none(),
        "NAV-009: Enter on an open group closes it:\n{}",
        frame.text
    );

    // A click on Settings while a text input holds the focus: Up then
    // reaches the sidebar's tree, and a typed x reaches no text input.
    let typed_log: Log<String> = Arc::default();
    let selected: Log<String> = Arc::default();
    let root = || {
        builder::div()
            .class("flex-col")
            .child(
                builder::text_input()
                    .value("Ada")
                    .on_change(recorder(&typed_log))
                    .build()
                    .auto_focus(),
            )
            .child(boxed(
                100,
                20,
                builder::sidebar()
                    .items(nav_items())
                    .on_select(recorder(&selected))
                    .build(),
            ))
            .build()
    };
    let base = shown(root(), "Settings");
    let (column, row) = at(&base, "Settings");
    let frame = settled(
        root(),
        "Settings",
        [click(column + 1, row), vec![key(KeyCode::Up)], typed("x")].concat(),
    );
    assert_eq!(
        held(&selected),
        vec!["settings"],
        "NAV-009: a click on Settings reports it through on_select:\n{}",
        frame.text
    );
    assert_eq!(
        cursor_among(&frame, &["Home", "Projects", "Settings"]),
        vec!["Projects"],
        "NAV-009: the click gave the tree the focus with its cursor on Settings, so Up moved it to Projects:\n{}",
        frame.text
    );
    assert_eq!(
        colors(&frame, "Settings"),
        (Some(role("accent-foreground")), Some(role("accent"))),
        "NAV-009: the click made Settings the active item:\n{}",
        frame.text
    );
    assert!(
        held(&typed_log).is_empty(),
        "NAV-009: the text input lost the focus to the click; it reported {:?}:\n{}",
        held(&typed_log),
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn nav_009_the_toggle_collapses_and_expands_the_sidebar_on_enter_space_and_a_click() {
    let _theme = Active::set(probe());
    let to_toggle = || vec![key(KeyCode::Tab), key(KeyCode::Tab)];
    let (frame, _, collapsed) = sidebar_after(
        nav_items(),
        [to_toggle(), vec![key(KeyCode::Enter)]].concat(),
    );
    assert_eq!(
        bar_width(&frame, EMPTY_ROW),
        3,
        "NAV-009: Enter on the toggle collapses the sidebar to 3 cells:\n{}",
        frame.text
    );
    assert_eq!(
        collapsed,
        vec![true],
        "NAV-009: the collapse is reported through on_collapse:\n{}",
        frame.text
    );

    let (frame, _, collapsed) = sidebar_after(
        nav_items(),
        [
            to_toggle(),
            vec![key(KeyCode::Enter), key(KeyCode::Char(' '))],
        ]
        .concat(),
    );
    assert_eq!(
        bar_width(&frame, EMPTY_ROW),
        BAR,
        "NAV-009: Space on the toggle expands the sidebar again:\n{}",
        frame.text
    );
    assert_eq!(
        collapsed,
        vec![true, false],
        "NAV-009: each change is reported through on_collapse:\n{}",
        frame.text
    );

    let base = shown(sidebar_of(nav_items()), "Home");
    let (column, row) = at(&base, Icon::ChevronLeft.glyph());
    let (frame, _, collapsed) = sidebar_after(nav_items(), click(column, row));
    assert_eq!(
        bar_width(&frame, EMPTY_ROW),
        3,
        "NAV-009: a click on the toggle collapses the sidebar:\n{}",
        frame.text
    );
    assert_eq!(
        collapsed,
        vec![true],
        "NAV-009: the click's collapse is reported once:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn nav_009_a_collapsed_sidebar_shows_icons_or_initials_and_the_cursors_label_beside_the_bar() {
    let _theme = Active::set(probe());
    let frame = shown(collapsed_sidebar(), ANY);
    let (_, home) = at(&frame, "H");
    for (offset, (what, expected)) in [
        ("Home's first character, as it has no icon", "H"),
        ("Projects' icon", Icon::Folder.glyph()),
        ("Settings' icon", Icon::Settings.glyph()),
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(
            glyph(&frame, 1, home + offset as u16),
            expected,
            "NAV-009: a collapsed bar shows {what} in its middle cell:\n{}",
            frame.text
        );
    }
    for label in ["Home", "Projects", "Settings"] {
        assert!(
            find(&frame, label).is_none(),
            "NAV-009: a collapsed sidebar without the focus paints no label ({label}):\n{}",
            frame.text
        );
    }

    let frame = settled(collapsed_sidebar(), ANY, vec![key(KeyCode::Tab)]);
    let (column, row) = at(&frame, "Home");
    assert!(
        column >= 3 && row == home,
        "NAV-009: the cursor's label stands beside the bar, on its item's row:\n{}",
        frame.text
    );
    assert!(
        find(&frame, "Projects").is_none(),
        "NAV-009: only the cursor's label shows:\n{}",
        frame.text
    );

    let frame = settled(
        collapsed_sidebar(),
        ANY,
        vec![key(KeyCode::Tab), key(KeyCode::Down)],
    );
    let (column, row) = at(&frame, "Projects");
    assert!(
        column >= 3 && row == home + 1,
        "NAV-009: Down moves the label beside the bar to Projects' row:\n{}",
        frame.text
    );
    assert!(
        find(&frame, "Home").is_none(),
        "NAV-009: Home's label left with the cursor:\n{}",
        frame.text
    );
}

#[test]
fn nav_009_a_following_log_view_keeps_its_last_row_on_its_last_line_as_rows_arrive() {
    let handle = hundred();
    let frame = log_after(
        &handle,
        None,
        vec![append(&handle, 101, 10)],
        vec![change()],
    );
    assert_eq!(
        log_line(&frame, 19),
        "Log row 110",
        "NAV-009: ten rows appended while the view follows leave the last on the last line:\n{}",
        frame.text
    );
    assert_eq!(
        log_line(&frame, 0),
        "Log row 091",
        "NAV-009: the view moved with them:\n{}",
        frame.text
    );
    assert!(
        find(&frame, "↓").is_none(),
        "NAV-009: a following view shows no new bar:\n{}",
        frame.text
    );
}

#[test]
fn nav_009_page_up_stops_the_following_and_rows_that_arrive_show_the_new_bar_and_leave_the_view() {
    let stopped = log_after(&hundred(), None, Vec::new(), vec![key(KeyCode::PageUp)]);
    assert_ne!(
        log_line(&stopped, 19),
        "Log row 100",
        "NAV-009: Page Up scrolls the view up:\n{}",
        stopped.text
    );

    let handle = hundred();
    let frame = log_after(
        &handle,
        None,
        vec![append(&handle, 101, 5)],
        vec![key(KeyCode::PageUp), change()],
    );
    assert_eq!(
        log_line(&frame, 0),
        log_line(&stopped, 0),
        "NAV-009: five rows appended after Page Up leave the first line:\n{}\n---\n{}",
        stopped.text,
        frame.text
    );
    assert_eq!(
        upper_lines(&frame),
        upper_lines(&stopped),
        "NAV-009: the rows in view stay on their lines:\n{}\n---\n{}",
        stopped.text,
        frame.text
    );
    assert!(
        last_line(&frame).contains("↓ 5 new"),
        "NAV-009: the last line reads ↓ 5 new:\n{}",
        frame.text
    );

    let handle = hundred();
    let frame = log_after(
        &handle,
        Some("neu"),
        vec![append(&handle, 101, 5)],
        vec![key(KeyCode::PageUp), change()],
    );
    assert!(
        last_line(&frame).contains("↓ 5 neu") && !frame.text.contains("new"),
        "NAV-009: the application's word neu replaces new:\n{}",
        frame.text
    );
}

#[test]
fn nav_009_end_resumes_the_following() {
    let handle = hundred();
    let frame = log_after(
        &handle,
        None,
        vec![append(&handle, 101, 5)],
        vec![key(KeyCode::PageUp), change(), key(KeyCode::End)],
    );
    assert_eq!(
        log_line(&frame, 19),
        "Log row 105",
        "NAV-009: End puts the last row on the last line:\n{}",
        frame.text
    );
    assert!(
        find(&frame, "↓").is_none(),
        "NAV-009: End leaves no new bar:\n{}",
        frame.text
    );

    let handle = hundred();
    let frame = log_after(
        &handle,
        None,
        vec![append(&handle, 101, 5), append(&handle, 106, 2)],
        vec![key(KeyCode::PageUp), change(), key(KeyCode::End), change()],
    );
    assert_eq!(
        log_line(&frame, 19),
        "Log row 107",
        "NAV-009: after End the view follows the rows that arrive:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn nav_009_up_home_the_wheel_a_click_on_the_track_and_a_drag_of_the_thumb_stop_the_following() {
    let _theme = Active::set(probe());
    let base = log_after(&hundred(), None, Vec::new(), Vec::new());
    let top = *thumb(&base).first().unwrap_or_else(|| {
        panic!(
            "no scroll bar thumb in the box's last column:\n{}",
            base.text
        )
    });
    assert!(
        top > 0,
        "the thumb of a view at its last row leaves track above it:\n{}",
        base.text
    );
    for (what, events) in [
        ("Up", vec![key(KeyCode::Up)]),
        ("Home", vec![key(KeyCode::Home)]),
        ("the wheel up", vec![wheel(50, 10, -1.0)]),
        ("a click on the track above the thumb", click(99, 0)),
        (
            "a drag of the thumb up",
            drag((99, top), (99, top.saturating_sub(5))),
        ),
    ] {
        let handle = hundred();
        let frame = log_after(
            &handle,
            None,
            vec![append(&handle, 101, 5)],
            [events, vec![change()]].concat(),
        );
        assert!(
            last_line(&frame).contains("↓ 5 new"),
            "NAV-009: {what} stops the following, so five rows that arrive show the new bar:\n{}",
            frame.text
        );
    }
}

#[test]
fn nav_009_a_click_on_the_new_bar_and_scrolling_down_to_the_last_row_resume_the_following() {
    let handle = hundred();
    let barred = log_after(
        &handle,
        None,
        vec![append(&handle, 101, 5)],
        vec![key(KeyCode::PageUp), change()],
    );
    let (column, row) = at(&barred, "↓ 5 new");
    let handle = hundred();
    let frame = log_after(
        &handle,
        None,
        vec![append(&handle, 101, 5), append(&handle, 106, 2)],
        [
            vec![key(KeyCode::PageUp), change()],
            click(column + 2, row),
            vec![change()],
        ]
        .concat(),
    );
    assert_eq!(
        log_line(&frame, 19),
        "Log row 107",
        "NAV-009: a click on the new bar resumes the following:\n{}",
        frame.text
    );
    assert!(
        find(&frame, "↓").is_none(),
        "NAV-009: the new bar is gone once the view follows:\n{}",
        frame.text
    );

    // Three pages down are more than the one page Page Up went.
    let handle = hundred();
    let frame = log_after(
        &handle,
        None,
        vec![append(&handle, 101, 5), append(&handle, 106, 2)],
        vec![
            key(KeyCode::PageUp),
            change(),
            key(KeyCode::PageDown),
            key(KeyCode::PageDown),
            key(KeyCode::PageDown),
            change(),
        ],
    );
    assert_eq!(
        log_line(&frame, 19),
        "Log row 107",
        "NAV-009: scrolling down to the last row resumes the following:\n{}",
        frame.text
    );
    assert!(
        find(&frame, "↓").is_none(),
        "NAV-009: no new bar once the view follows again:\n{}",
        frame.text
    );
}

#[test]
fn nav_009_rows_prepended_while_the_view_does_not_follow_leave_the_rows_in_view_on_their_lines() {
    let stopped = log_after(&hundred(), None, Vec::new(), vec![key(KeyCode::PageUp)]);
    let handle = hundred();
    let frame = log_after(
        &handle,
        None,
        vec![prepend(&handle, 10)],
        vec![key(KeyCode::PageUp), change()],
    );
    assert_eq!(
        log_line(&frame, 0),
        log_line(&stopped, 0),
        "NAV-009: ten rows prepended leave the first line:\n{}\n---\n{}",
        stopped.text,
        frame.text
    );
    assert_eq!(
        upper_lines(&frame),
        upper_lines(&stopped),
        "NAV-009: ten rows prepended leave the rows in view on their lines:\n{}\n---\n{}",
        stopped.text,
        frame.text
    );
}

#[test]
fn nav_009_the_builders_take_the_callbacks_their_components_take() {
    let props = || ResizableProps {
        direction: SplitDirection::Row,
        panes: vec![Pane::new(filled(0)).size(49), Pane::new(filled(1))],
        aria_label: None,
        disabled: false,
    };
    let (_, built) = split_after(split_49_50, vec![key(KeyCode::Tab), key(KeyCode::Right)]);
    let (_, from_props) = split_after(
        |log| {
            let log = log.clone();
            Element::typed_with::<Resizable>(props(), move |props| {
                Resizable::new(props).with_on_resize(recorder(&log))
            })
        },
        vec![key(KeyCode::Tab), key(KeyCode::Right)],
    );
    assert_eq!(
        built,
        vec![vec![50, 49]],
        "NAV-009: the builder's on_resize reports Right"
    );
    assert_eq!(
        from_props,
        vec![vec![50, 49]],
        "NAV-009: the component's with_on_resize reports Right"
    );

    let events = || {
        vec![
            key(KeyCode::Tab),
            key(KeyCode::End),
            key(KeyCode::Enter),
            key(KeyCode::Tab),
            key(KeyCode::Enter),
        ]
    };
    let (_, built_selected, built_collapsed) = sidebar_after(nav_items(), events());
    let selected: Log<String> = Arc::default();
    let collapsed: Log<bool> = Arc::default();
    let (on_select, on_collapse) = (selected.clone(), collapsed.clone());
    let sidebar = Element::typed_with::<Sidebar>(
        SidebarProps {
            items: nav_items(),
            active: None,
            collapsed: false,
            header: None,
            footer: None,
            aria_label: None,
            width: None,
        },
        move |props| {
            Sidebar::new(props)
                .with_on_select(recorder(&on_select))
                .with_on_collapse(recorder(&on_collapse))
        },
    );
    settled(boxed(100, 20, sidebar), ANY, events());
    assert_eq!(
        (built_selected, built_collapsed),
        (vec!["settings".to_owned()], vec![true]),
        "NAV-009: the builder's on_select and on_collapse report Enter on Settings and on the toggle"
    );
    assert_eq!(
        (held(&selected), held(&collapsed)),
        (vec!["settings".to_owned()], vec![true]),
        "NAV-009: the component's with_on_select and with_on_collapse report the same"
    );
}

// ---------------------------------------------------------------- NAV-010

/// NAV-010: each pointer action of the split, the sidebar and the log view
/// has a key that reaches the same state.
#[test]
#[serial_test::serial(theme)]
fn nav_010_every_pointer_action_of_a_split_a_sidebar_and_a_log_view_has_a_key() {
    let _theme = Active::set(probe());
    for (what, pointer, keys) in [
        (
            "a drag of the divider ten cells right",
            drag((49, 5), (59, 5)),
            vec![key(KeyCode::Tab), shift(KeyCode::Right)],
        ),
        (
            "a drag of the divider to the box's end",
            drag((49, 5), (99, 5)),
            vec![key(KeyCode::Tab), key(KeyCode::End)],
        ),
        (
            "a drag of the divider to the box's start",
            drag((49, 5), (0, 5)),
            vec![key(KeyCode::Tab), key(KeyCode::Home)],
        ),
    ] {
        let (dragged, by_pointer) = split_after(split_49_50, pointer);
        let (keyed, by_keys) = split_after(split_49_50, keys);
        assert_eq!(
            widths(&dragged, 5, 2),
            widths(&keyed, 5, 2),
            "NAV-010: {what} and its keys leave the same panes:\n{}\n---\n{}",
            dragged.text,
            keyed.text
        );
        assert_eq!(
            by_pointer.last(),
            by_keys.last(),
            "NAV-010: {what} and its keys report the same sizes"
        );
    }

    let base = shown(sidebar_of(nav_items()), "Home");
    let settings = at(&base, "Settings");
    let projects = at(&base, "Projects");
    let toggle = at(&base, Icon::ChevronLeft.glyph());
    for (what, pointer, keys) in [
        (
            "a click on Settings",
            click(settings.0 + 1, settings.1),
            vec![key(KeyCode::Tab), key(KeyCode::End), key(KeyCode::Enter)],
        ),
        (
            "a click on the group Projects",
            click(projects.0 + 1, projects.1),
            vec![key(KeyCode::Tab), key(KeyCode::Down), key(KeyCode::Enter)],
        ),
        (
            "a click on the toggle",
            click(toggle.0, toggle.1),
            vec![key(KeyCode::Tab), key(KeyCode::Tab), key(KeyCode::Enter)],
        ),
    ] {
        let (clicked, clicked_selected, clicked_collapsed) = sidebar_after(nav_items(), pointer);
        let (keyed, keyed_selected, keyed_collapsed) = sidebar_after(nav_items(), keys);
        assert_eq!(
            (clicked_selected, clicked_collapsed),
            (keyed_selected, keyed_collapsed),
            "NAV-010: {what} and its keys report the same:\n{}\n---\n{}",
            clicked.text,
            keyed.text
        );
        assert_eq!(
            (
                bar_width(&clicked, EMPTY_ROW),
                find(&clicked, "Alpha").is_some()
            ),
            (
                bar_width(&keyed, EMPTY_ROW),
                find(&keyed, "Alpha").is_some()
            ),
            "NAV-010: {what} and its keys leave the sidebar the same:\n{}\n---\n{}",
            clicked.text,
            keyed.text
        );
    }

    let base = log_after(&hundred(), None, Vec::new(), Vec::new());
    let top = *thumb(&base).first().unwrap_or_else(|| {
        panic!(
            "no scroll bar thumb in the box's last column:\n{}",
            base.text
        )
    });
    for (what, pointer, keys) in [
        (
            "the wheel up",
            vec![wheel(50, 10, -1.0)],
            vec![key(KeyCode::Up)],
        ),
        (
            "a click on the track above the thumb",
            click(99, 0),
            vec![key(KeyCode::PageUp)],
        ),
        (
            "a drag of the thumb up",
            drag((99, top), (99, top.saturating_sub(5))),
            vec![key(KeyCode::Up)],
        ),
    ] {
        let handle = hundred();
        let by_pointer = log_after(
            &handle,
            None,
            vec![append(&handle, 101, 5)],
            [pointer, vec![change()]].concat(),
        );
        let handle = hundred();
        let by_keys = log_after(
            &handle,
            None,
            vec![append(&handle, 101, 5)],
            [keys, vec![change()]].concat(),
        );
        assert!(
            last_line(&by_pointer).contains("↓ 5 new") && last_line(&by_keys).contains("↓ 5 new"),
            "NAV-010: {what} and its key both stop the following:\n{}\n---\n{}",
            by_pointer.text,
            by_keys.text
        );
    }
    let handle = hundred();
    let by_track = log_after(&handle, None, Vec::new(), click(99, 0));
    let by_page_up = log_after(&hundred(), None, Vec::new(), vec![key(KeyCode::PageUp)]);
    assert_eq!(
        log_line(&by_track, 0),
        log_line(&by_page_up, 0),
        "NAV-010: a click on the track above the thumb scrolls the page Page Up scrolls:\n{}\n---\n{}",
        by_track.text,
        by_page_up.text
    );

    let handle = hundred();
    let barred = log_after(
        &handle,
        None,
        vec![append(&handle, 101, 5)],
        vec![key(KeyCode::PageUp), change()],
    );
    let (column, row) = at(&barred, "↓ 5 new");
    let handle = hundred();
    let clicked = log_after(
        &handle,
        None,
        vec![append(&handle, 101, 5)],
        [vec![key(KeyCode::PageUp), change()], click(column + 2, row)].concat(),
    );
    let handle = hundred();
    let ended = log_after(
        &handle,
        None,
        vec![append(&handle, 101, 5)],
        vec![key(KeyCode::PageUp), change(), key(KeyCode::End)],
    );
    assert_eq!(
        clicked.text, ended.text,
        "NAV-010: a click on the new bar and End leave the same view"
    );
}

// ---------------------------------------------------------------- DAT-006

#[test]
#[serial_test::serial(theme)]
fn dat_006_a_list_paints_its_box_labels_descriptions_titles_and_disabled_items_by_role() {
    let _theme = Active::set(probe());
    let mut items = fruits();
    items[0] = ListItem::new("apple", "Apple").description("Crisp and red");
    let frame = shown(in_box(list_of(items)), "Apple");
    assert_eq!(
        colors(&frame, "Apple"),
        (Some(role("foreground")), Some(role("background"))),
        "DAT-006: a label is foreground on the page's background:\n{}",
        frame.text
    );
    let (apple, row) = at(&frame, "Apple");
    let (description, description_row) = at(&frame, "Crisp and red");
    assert!(
        description_row == row && description > apple + 5,
        "DAT-006: the description stands after the label:\n{}",
        frame.text
    );
    assert_eq!(
        cell_colors(&frame, description, row).0,
        Some(role("text-muted")),
        "DAT-006: a description is text-muted:\n{}",
        frame.text
    );
    assert_eq!(
        colors(&frame, "Citrus").0,
        Some(role("text-muted")),
        "DAT-006: a section's title is text-muted:\n{}",
        frame.text
    );
    assert_eq!(
        colors(&frame, "Mango").0,
        Some(role("text-muted")),
        "DAT-006: a disabled item is text-muted:\n{}",
        frame.text
    );
    let (_, mango) = at(&frame, "Mango");
    for row in mango + 1..20 {
        for column in [0, 50, 99] {
            assert_eq!(
                cell_colors(&frame, column, row).1,
                Some(role("background")),
                "DAT-006: the box below the last row shows the page: it has no fill of its own:\n{}",
                frame.text
            );
        }
    }
}

#[test]
#[serial_test::serial(theme)]
fn dat_006_the_cursor_the_selected_and_the_hovered_item_take_their_roles() {
    let _theme = Active::set(probe());
    let focused = shown(in_box(list_of(fruits()).auto_focus()), "Apple");
    assert_eq!(
        colors(&focused, "Apple"),
        (Some(role("selection-foreground")), Some(role("selection"))),
        "DAT-006: the cursor's item is selection while the list holds the focus:\n{}",
        focused.text
    );
    let selected = shown(
        in_box(builder::list().items(fruits()).selected("banana").build()),
        "Banana",
    );
    assert_eq!(
        colors(&selected, "Banana"),
        (Some(role("accent-foreground")), Some(role("accent"))),
        "DAT-006: the selected item is accent-foreground on accent:\n{}",
        selected.text
    );
    let (column, row) = at(&selected, "Lemon");
    let hovered = settled(
        in_box(list_of(fruits())),
        "Lemon",
        vec![hover(column + 1, row)],
    );
    assert_eq!(
        colors(&hovered, "Lemon").1,
        Some(role("hover")),
        "DAT-006: the item under the pointer is on hover:\n{}",
        hovered.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn dat_006_matched_characters_are_primary_and_the_filter_row_is_painted_as_a_text_inputs_field() {
    let _theme = Active::set(probe());
    let filtered = settled(
        in_box(
            builder::list()
                .items(fruits())
                .filterable(true)
                .build()
                .auto_focus(),
        ),
        ANY,
        typed("le"),
    );
    let (lemon, row) = at(&filtered, "Lemon");
    for offset in 0..2 {
        assert_eq!(
            cell_colors(&filtered, lemon + offset, row).0,
            Some(role("primary")),
            "DAT-006: the characters of Lemon that match le are primary:\n{}",
            filtered.text
        );
    }
    for offset in 2..5 {
        assert_eq!(
            cell_colors(&filtered, lemon + offset, row).0,
            Some(role("foreground")),
            "DAT-006: the rest of the label stays foreground:\n{}",
            filtered.text
        );
    }
    let field = settled(
        boxed(100, 1, builder::text_input().build().auto_focus()),
        ANY,
        typed("le"),
    );
    assert_eq!(
        cells(&filtered, 0, 0..100),
        cells(&field, 0, 0..100),
        "DAT-006: the filter row is painted as a focused text input that holds le:\n{}\n---\n{}",
        filtered.text,
        field.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn dat_006_the_loading_row_and_the_empty_view_are_painted_as_the_spinner_and_the_empty_piece() {
    let _theme = Active::set(probe());
    let spinner = shown(builder::spinner().animated(false).build(), ANY);
    let (column, row) = spinner_cell(&spinner)
        .unwrap_or_else(|| panic!("the spinner piece paints its glyph:\n{}", spinner.text));
    let spinner_color = cell_colors(&spinner, column, row).0;
    let loading = shown(
        in_box(builder::list().items(fruits()).loading(true).build()),
        "Mango",
    );
    let (_, mango) = at(&loading, "Mango");
    let (column, row) = spinner_cell(&loading).unwrap_or_else(|| {
        panic!(
            "DAT-006: a loading list paints the spinner:\n{}",
            loading.text
        )
    });
    assert_eq!(
        row,
        mango + 1,
        "DAT-006: the loading row follows the last item:\n{}",
        loading.text
    );
    assert_eq!(
        cell_colors(&loading, column, row).0,
        spinner_color,
        "DAT-006: the loading row's glyph is painted as the spinner paints it:\n{}",
        loading.text
    );

    let piece = shown(
        builder::empty().title("No fruit yet").build(),
        "No fruit yet",
    );
    let empty = shown(
        in_box(builder::list().empty_text("No fruit yet").build()),
        ANY,
    );
    assert_eq!(
        colors(&empty, "No fruit yet"),
        colors(&piece, "No fruit yet"),
        "DAT-006: the empty view's message is painted as the empty piece paints its title:\n{}\n---\n{}",
        empty.text,
        piece.text
    );
    assert_eq!(
        colors(&empty, Icon::Info.glyph()),
        colors(&piece, Icon::Info.glyph()),
        "DAT-006: the empty view's icon is painted as the empty piece paints it:\n{}\n---\n{}",
        empty.text,
        piece.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn dat_006_a_list_built_through_its_builder_paints_the_same_cells_as_from_its_props() {
    let _theme = Active::set(probe());
    let from_props = shown(
        in_box(Element::typed::<List>(ListProps {
            items: fruits(),
            selected: Some("banana".to_owned()),
            filterable: true,
            has_more: false,
            loading: false,
            empty_text: None,
            aria_label: None,
            width: None,
            height: None,
        })),
        "Banana",
    );
    let built = shown(
        in_box(
            builder::list()
                .items(fruits())
                .selected("banana")
                .filterable(true)
                .build(),
        ),
        "Banana",
    );
    assert!(
        paint(&from_props) == paint(&built),
        "DAT-006: the list built through builder::list() paints other cells than from its props:\n{}\n---\n{}",
        from_props.text,
        built.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn dat_006_focus_hover_and_a_selection_change_no_glyph_or_text() {
    let _theme = Active::set(probe());
    let plain = shown(in_box(list_of(fruits())), "Apple");
    let (column, row) = at(&plain, "Lemon");
    for (what, frame) in [
        (
            "the focus",
            shown(in_box(list_of(fruits()).auto_focus()), "Apple"),
        ),
        (
            "the pointer",
            settled(
                in_box(list_of(fruits())),
                "Apple",
                vec![hover(column + 1, row)],
            ),
        ),
        (
            "a selection",
            shown(
                in_box(builder::list().items(fruits()).selected("banana").build()),
                "Apple",
            ),
        ),
    ] {
        assert_eq!(
            frame.text, plain.text,
            "DAT-006: {what} changes no glyph or text of a list"
        );
    }
}

#[test]
#[serial_test::serial(theme)]
fn dat_006_a_list_takes_every_color_from_the_theme() {
    let _theme = Active::set(probe());
    let mut items = fruits();
    items[0] = ListItem::new("apple", "Apple").description("Crisp and red");
    let plain = shown(in_box(list_of(fruits())), "Apple");
    let (column, row) = at(&plain, "Lemon");
    let frames: Vec<(&str, Snapshot)> = vec![
        (
            "list with a description, a section and a disabled item",
            shown(in_box(list_of(items)), "Apple"),
        ),
        (
            "focused list",
            shown(in_box(list_of(fruits()).auto_focus()), "Apple"),
        ),
        (
            "list with a selection",
            shown(
                in_box(builder::list().items(fruits()).selected("banana").build()),
                "Apple",
            ),
        ),
        (
            "hovered list",
            settled(
                in_box(list_of(fruits())),
                "Apple",
                vec![hover(column + 1, row)],
            ),
        ),
        (
            "filtered list",
            settled(
                in_box(
                    builder::list()
                        .items(fruits())
                        .filterable(true)
                        .build()
                        .auto_focus(),
                ),
                ANY,
                typed("le"),
            ),
        ),
        (
            "loading list",
            shown(
                in_box(builder::list().items(fruits()).loading(true).build()),
                "Apple",
            ),
        ),
        (
            "empty list",
            shown(
                in_box(builder::list().empty_text("No fruit yet").build()),
                ANY,
            ),
        ),
    ];
    let wrong = foreign_in(&frames);
    assert!(
        wrong.is_empty(),
        "DAT-006: a list paints a color the theme does not define:\n{}",
        wrong.join("\n")
    );
}

// ---------------------------------------------------------------- DAT-007

#[test]
#[serial_test::serial(theme)]
fn dat_007_a_default_list_of_40_items_shows_20_and_fills_a_box_of_100_by_20() {
    let _theme = Active::set(probe());
    let frame = shown(in_box(list_of(numbered(40)).auto_focus()), "Item");
    for n in 1..=20 {
        assert_eq!(
            at(&frame, &item(n)).1,
            n as u16 - 1,
            "DAT-007: a default list shows one item on each of its box's 20 rows:\n{}",
            frame.text
        );
    }
    assert!(
        find(&frame, &item(21)).is_none(),
        "DAT-007: the 21st item does not fit the box's 20 rows:\n{}",
        frame.text
    );
    for column in [0, 99] {
        assert_eq!(
            cell_colors(&frame, column, 0).1,
            Some(role("selection")),
            "DAT-007: the cursor's row spans the box from its first to its last cell:\n{}",
            frame.text
        );
    }
    assert_ne!(
        cell_colors(&frame, 100, 0).1,
        Some(role("selection")),
        "DAT-007: the row ends with the box:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn dat_007_a_size_class_and_size_props_set_the_lists_size() {
    let _theme = Active::set(probe());
    for (what, list, rows) in [
        (
            "w-40 on its builder",
            builder::list().items(numbered(40)).class("w-40").build(),
            20,
        ),
        (
            "w-40 h-10 on its builder",
            builder::list()
                .items(numbered(40))
                .class("w-40 h-10")
                .build(),
            10,
        ),
        (
            "width(40) and height(10) on its builder",
            builder::list()
                .items(numbered(40))
                .width(40)
                .height(10)
                .build(),
            10,
        ),
        (
            "a width of 40 and a height of 10 on its props",
            Element::typed::<List>(ListProps {
                items: numbered(40),
                selected: None,
                filterable: false,
                has_more: false,
                loading: false,
                empty_text: None,
                aria_label: None,
                width: Some(40),
                height: Some(10),
            }),
            10,
        ),
    ] {
        let frame = shown(in_box(list.auto_focus()), "Item");
        assert_eq!(
            (
                cell_colors(&frame, 39, 0).1,
                cell_colors(&frame, 40, 0).1 == Some(role("selection"))
            ),
            (Some(role("selection")), false),
            "DAT-007: {what} makes the list 40 cells wide:\n{}",
            frame.text
        );
        assert!(
            find(&frame, &item(rows)).is_some() && find(&frame, &item(rows + 1)).is_none(),
            "DAT-007: {what} shows {rows} items:\n{}",
            frame.text
        );
    }
}

#[test]
fn dat_007_the_filter_row_and_the_loading_row_each_take_a_row() {
    let frame = shown(
        in_box(builder::list().items(numbered(40)).filterable(true).build()),
        "Item",
    );
    assert_eq!(
        at(&frame, &item(1)).1,
        1,
        "DAT-007: the filter row takes the first row:\n{}",
        frame.text
    );
    assert!(
        find(&frame, &item(19)).is_some_and(|(_, row)| row == 19)
            && find(&frame, &item(20)).is_none(),
        "DAT-007: under the filter row the box holds 19 items:\n{}",
        frame.text
    );

    let frame = shown(
        in_box(
            builder::list()
                .items(numbered(10))
                .filterable(true)
                .loading(true)
                .build(),
        ),
        "Item",
    );
    assert_eq!(
        (at(&frame, &item(1)).1, at(&frame, &item(10)).1),
        (1, 10),
        "DAT-007: the items one row each under the filter row:\n{}",
        frame.text
    );
    assert_eq!(
        spinner_cell(&frame).map(|(_, row)| row),
        Some(11),
        "DAT-007: the loading row takes the row after the last item:\n{}",
        frame.text
    );
}

/// DAT-007: a list builds elements only for the rows in view and ten at
/// most beyond them at each side. A list of 40 items in a box of 20 rows
/// has 40 rows at most to build, so a list of 100,000 items that lays out
/// more elements than it builds more than 40 rows.
#[test]
fn dat_007_a_list_of_100000_items_builds_no_more_rows_than_a_list_of_40() {
    let laid_out = |count: usize, events: Vec<Option<Event>>| {
        let frame = settled(
            in_box(list_of(numbered(count)).auto_focus()),
            "Item",
            events,
        );
        (frame.geometry.len(), frame.text)
    };
    for (what, events) in [
        ("at its first item", Vec::new()),
        ("after End", vec![key(KeyCode::End)]),
    ] {
        let (small, small_text) = laid_out(40, events.clone());
        assert!(
            small >= 20,
            "the frame reports the elements it lays out, {small} for 20 rows in view:\n{small_text}"
        );
        let (large, large_text) = laid_out(100_000, events);
        assert!(
            large <= small,
            "DAT-007: {what} a list of 100,000 items lays out {large} elements where one of 40 items lays out {small}, so it builds more than 40 rows:\n{large_text}"
        );
    }
}

// ---------------------------------------------------------------- DAT-008

#[test]
#[serial_test::serial(theme)]
fn dat_008_the_cursor_moves_one_item_and_to_the_ends_passing_over_titles_and_disabled_items() {
    let _theme = Active::set(probe());
    let labels = ["Apple", "Banana", "Lemon", "Lime", "Mango"];
    for (events, expected, what) in [
        (Vec::new(), "Apple", "the cursor starts on the first item"),
        (
            vec![key(KeyCode::Down), key(KeyCode::Down)],
            "Lemon",
            "Down twice passes over the title Citrus",
        ),
        (
            vec![key(KeyCode::End)],
            "Lime",
            "End passes over the disabled Mango",
        ),
        (
            vec![key(KeyCode::Down), key(KeyCode::Down), key(KeyCode::Up)],
            "Banana",
            "Up passes back over the title",
        ),
        (vec![key(KeyCode::End), key(KeyCode::Home)], "Apple", "Home"),
    ] {
        let (frame, _) = list_after(fruits(), "Apple", events);
        assert_eq!(
            cursor_among(&frame, &labels),
            vec![expected],
            "DAT-008: {what}:\n{}",
            frame.text
        );
    }
    let (frame, reported) = list_after(
        fruits(),
        "Apple",
        vec![
            key(KeyCode::Down),
            key(KeyCode::Down),
            key(KeyCode::End),
            key(KeyCode::Up),
            key(KeyCode::Home),
        ],
    );
    assert_eq!(
        reported,
        vec!["banana", "lemon", "lime", "lemon", "apple"],
        "DAT-008: each move reports the cursor's item through on_select:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn dat_008_page_down_moves_by_the_items_the_height_holds_and_end_brings_the_last_item_into_view() {
    let _theme = Active::set(probe());
    let (frame, reported) = list_after(numbered(1000), "Item", vec![key(KeyCode::PageDown)]);
    assert!(
        cursor_on(&frame, &item(21)),
        "DAT-008: Page Down from item 1 in a box of 20 rows puts the cursor on item 21, in view:\n{}",
        frame.text
    );
    assert_eq!(
        reported,
        vec!["i000021"],
        "DAT-008: Page Down reports the cursor's item:\n{}",
        frame.text
    );

    let (frame, _) = list_after(numbered(1000), "Item", vec![key(KeyCode::End)]);
    assert!(
        cursor_on(&frame, &item(1000)),
        "DAT-008: End puts the cursor on item 1,000 and brings it into view:\n{}",
        frame.text
    );

    let (frame, _) = list_after(
        numbered(1000),
        "Item",
        vec![key(KeyCode::End), key(KeyCode::PageUp)],
    );
    assert!(
        cursor_on(&frame, &item(980)),
        "DAT-008: Page Up from item 1,000 puts the cursor on item 980, in view:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn dat_008_a_click_on_an_item_moves_the_cursor_to_it_and_gives_the_list_the_focus() {
    let _theme = Active::set(probe());
    let typed_log: Log<String> = Arc::default();
    let selected: Log<String> = Arc::default();
    let root = || {
        builder::div()
            .class("flex-col")
            .child(
                builder::text_input()
                    .value("Ada")
                    .on_change(recorder(&typed_log))
                    .build()
                    .auto_focus(),
            )
            .child(in_box(
                builder::list()
                    .items(fruits())
                    .on_select(recorder(&selected))
                    .build(),
            ))
            .build()
    };
    let base = shown(root(), "Banana");
    let (column, row) = at(&base, "Banana");
    let clicked = settled(root(), "Banana", click(column + 1, row));
    assert!(
        cursor_on(&clicked, "Banana"),
        "DAT-008: a click on Banana puts the cursor on it and gives the list the focus:\n{}",
        clicked.text
    );
    assert_eq!(
        held(&selected).last().map(String::as_str),
        Some("banana"),
        "DAT-008: the click's move reports Banana:\n{}",
        clicked.text
    );

    let frame = settled(
        root(),
        "Banana",
        [click(column + 1, row), vec![key(KeyCode::Down)], typed("x")].concat(),
    );
    assert!(
        cursor_on(&frame, "Lemon"),
        "DAT-008: after the click Down reaches the list:\n{}",
        frame.text
    );
    assert!(
        held(&typed_log).is_empty(),
        "DAT-008: the text input lost the focus to the click; it reported {:?}:\n{}",
        held(&typed_log),
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn dat_008_enter_and_a_double_click_select_the_item_and_report_it_through_on_confirm() {
    let _theme = Active::set(probe());
    let list = |confirmed: &Log<String>| {
        in_box(
            builder::list()
                .items(fruits())
                .on_confirm(recorder(confirmed))
                .build()
                .auto_focus(),
        )
    };
    let confirmed: Log<String> = Arc::default();
    let frame = settled(
        list(&confirmed),
        "Apple",
        vec![key(KeyCode::Enter), key(KeyCode::Down)],
    );
    assert_eq!(
        held(&confirmed),
        vec!["apple"],
        "DAT-008: Enter reports the cursor's item through on_confirm:\n{}",
        frame.text
    );
    assert_eq!(
        colors(&frame, "Apple"),
        (Some(role("accent-foreground")), Some(role("accent"))),
        "DAT-008: Enter selected Apple, accent once the cursor left it:\n{}",
        frame.text
    );

    let base = shown(list(&Arc::default()), "Banana");
    let (column, row) = at(&base, "Banana");
    let confirmed: Log<String> = Arc::default();
    let frame = settled(list(&confirmed), "Banana", double_click(column + 1, row));
    assert_eq!(
        held(&confirmed),
        vec!["banana"],
        "DAT-008: a double click on Banana reports it through on_confirm once:\n{}",
        frame.text
    );
}

#[test]
#[serial_test::serial(theme)]
fn dat_008_typed_characters_filter_the_items_under_the_titles_of_their_sections() {
    let _theme = Active::set(probe());
    let filterable = || {
        in_box(
            builder::list()
                .items(fruits())
                .filterable(true)
                .build()
                .auto_focus(),
        )
    };
    for (what, events) in [
        ("le", typed("le")),
        ("LE, letters compared without case", typed("LE")),
        (
            "le, the cursor on Lime before",
            [vec![key(KeyCode::End)], typed("le")].concat(),
        ),
    ] {
        let frame = settled(filterable(), ANY, events);
        for label in ["Apple", "Citrus", "Lemon"] {
            assert!(
                find(&frame, label).is_some(),
                "DAT-008: after {what} the list shows {label}:\n{}",
                frame.text
            );
        }
        for label in ["Banana", "Lime", "Mango"] {
            assert!(
                find(&frame, label).is_none(),
                "DAT-008: after {what} the list hides {label}:\n{}",
                frame.text
            );
        }
        let rows: Vec<u16> = ["Apple", "Citrus", "Lemon"]
            .iter()
            .map(|label| at(&frame, label).1)
            .collect();
        assert!(
            rows[0] < rows[1] && rows[1] < rows[2],
            "DAT-008: after {what} Lemon stands under its section's title Citrus:\n{}",
            frame.text
        );
        assert!(
            cursor_on(&frame, "Apple"),
            "DAT-008: after {what} the cursor is on the first item shown:\n{}",
            frame.text
        );
    }

    let frame = settled(
        filterable(),
        ANY,
        [typed("le"), vec![key(KeyCode::Backspace)]].concat(),
    );
    assert!(
        find(&frame, "Lime").is_some() && find(&frame, "Banana").is_none(),
        "DAT-008: Backspace takes the e out, so the filter l shows Lime again:\n{}",
        frame.text
    );

    let frame = settled(
        in_box(
            builder::list()
                .items(numbered(1000))
                .filterable(true)
                .build()
                .auto_focus(),
        ),
        ANY,
        [vec![key(KeyCode::End)], typed("00099")].concat(),
    );
    assert_eq!(
        find(&frame, &item(99)).map(|(_, row)| row),
        Some(1),
        "DAT-008: a filter typed at the end puts the view back at the top:\n{}",
        frame.text
    );
    assert!(
        cursor_on(&frame, &item(99)),
        "DAT-008: the cursor goes to the first item shown:\n{}",
        frame.text
    );
}

#[test]
fn dat_008_escape_empties_a_filter_that_holds_text_and_otherwise_reports_through_on_cancel() {
    let list = |cancels: &Count| {
        in_box(
            builder::list()
                .items(fruits())
                .filterable(true)
                .on_cancel(counting(cancels))
                .build()
                .auto_focus(),
        )
    };
    let cancels: Count = Arc::default();
    let frame = settled(
        list(&cancels),
        ANY,
        [typed("le"), vec![key(KeyCode::Escape)]].concat(),
    );
    assert!(
        find(&frame, "Banana").is_some() && !line(&frame, 0, 0..100).contains("le"),
        "DAT-008: Escape empties the filter, so every item shows again:\n{}",
        frame.text
    );
    assert_eq!(
        calls(&cancels),
        0,
        "DAT-008: Escape on a filter that holds text does not cancel:\n{}",
        frame.text
    );

    let cancels: Count = Arc::default();
    let frame = settled(
        list(&cancels),
        ANY,
        [
            typed("le"),
            vec![key(KeyCode::Escape), key(KeyCode::Escape)],
        ]
        .concat(),
    );
    assert_eq!(
        calls(&cancels),
        1,
        "DAT-008: a second Escape reports through on_cancel:\n{}",
        frame.text
    );
}

#[test]
fn dat_008_a_list_with_more_to_come_asks_for_it_once_until_its_items_change() {
    let asked = |events: Vec<Option<Event>>| {
        let more: Count = Arc::default();
        let count = more.clone();
        let root = Scripted::new(
            move |presses| {
                let items = if presses == 0 { 100 } else { 200 };
                in_box(
                    builder::list()
                        .items(numbered(items))
                        .has_more(true)
                        .on_load_more(counting(&count))
                        .build()
                        .auto_focus(),
                )
            },
            Vec::new(),
        );
        let frame = last_of(run(root, "Item", events));
        (calls(&more), frame.text)
    };
    let (asks, text) = asked(Vec::new());
    assert_eq!(
        asks, 0,
        "DAT-008: a list whose view ends 80 items before its last asks for nothing:\n{text}"
    );
    let (asks, text) = asked(vec![key(KeyCode::End)]);
    assert_eq!(
        asks, 1,
        "DAT-008: End brings the cursor within 20 items of the last, so the list asks for more:\n{text}"
    );
    let (asks, text) = asked(vec![
        key(KeyCode::End),
        key(KeyCode::Up),
        key(KeyCode::PageUp),
        key(KeyCode::End),
    ]);
    assert_eq!(
        asks, 1,
        "DAT-008: the list asks once until its items change:\n{text}"
    );
    let (asks, text) = asked(vec![key(KeyCode::End), change(), key(KeyCode::End)]);
    assert_eq!(
        asks, 2,
        "DAT-008: once its items changed the list asks again:\n{text}"
    );
}

#[test]
fn dat_008_a_loading_list_shows_its_loading_row_and_a_list_with_nothing_to_show_its_empty_view() {
    let frame = shown(
        in_box(builder::list().items(fruits()).loading(true).build()),
        "Apple",
    );
    assert!(
        spinner_cell(&frame).is_some(),
        "DAT-008: a loading list shows its loading row:\n{}",
        frame.text
    );
    let frame = shown(in_box(list_of(fruits())), "Apple");
    assert!(
        spinner_cell(&frame).is_none(),
        "DAT-008: a list that is not loading shows no loading row:\n{}",
        frame.text
    );
    let frame = shown(
        in_box(builder::list().empty_text("No fruit yet").build()),
        ANY,
    );
    assert!(
        find(&frame, "No fruit yet").is_some(),
        "DAT-008: a list of no items shows its empty view:\n{}",
        frame.text
    );
    let frame = settled(
        in_box(
            builder::list()
                .items(fruits())
                .filterable(true)
                .empty_text("No fruit yet")
                .build()
                .auto_focus(),
        ),
        ANY,
        typed("zz"),
    );
    assert!(
        find(&frame, "No fruit yet").is_some(),
        "DAT-008: a filter that keeps no item shows the empty view:\n{}",
        frame.text
    );
}

#[test]
fn dat_008_the_builder_and_the_props_report_through_the_same_callbacks() {
    // Down moves to item 2, Enter confirms it, End moves to item 100 and
    // asks for more, and Escape without a filter cancels.
    let events = || {
        vec![
            key(KeyCode::Down),
            key(KeyCode::Enter),
            key(KeyCode::End),
            key(KeyCode::Escape),
        ]
    };
    let expected = (
        vec!["i000002".to_owned(), "i000100".to_owned()],
        vec!["i000002".to_owned()],
        1,
        1,
    );

    let selected: Log<String> = Arc::default();
    let confirmed: Log<String> = Arc::default();
    let more: Count = Arc::default();
    let cancels: Count = Arc::default();
    settled(
        in_box(
            builder::list()
                .items(numbered(100))
                .has_more(true)
                .on_select(recorder(&selected))
                .on_confirm(recorder(&confirmed))
                .on_load_more(counting(&more))
                .on_cancel(counting(&cancels))
                .build()
                .auto_focus(),
        ),
        "Item",
        events(),
    );
    assert_eq!(
        (
            held(&selected),
            held(&confirmed),
            calls(&more),
            calls(&cancels)
        ),
        expected,
        "DAT-008: the builder's on_select, on_confirm, on_load_more and on_cancel report"
    );

    let selected: Log<String> = Arc::default();
    let confirmed: Log<String> = Arc::default();
    let more: Count = Arc::default();
    let cancels: Count = Arc::default();
    let (on_select, on_confirm, on_load_more, on_cancel) = (
        selected.clone(),
        confirmed.clone(),
        more.clone(),
        cancels.clone(),
    );
    let list = Element::typed_with::<List>(
        ListProps {
            items: numbered(100),
            selected: None,
            filterable: false,
            has_more: true,
            loading: false,
            empty_text: None,
            aria_label: None,
            width: None,
            height: None,
        },
        move |props| {
            List::new(props)
                .with_on_select(recorder(&on_select))
                .with_on_confirm(recorder(&on_confirm))
                .with_on_load_more(counting(&on_load_more))
                .with_on_cancel(counting(&on_cancel))
        },
    );
    settled(in_box(list.auto_focus()), "Item", events());
    assert_eq!(
        (
            held(&selected),
            held(&confirmed),
            calls(&more),
            calls(&cancels)
        ),
        expected,
        "DAT-008: the component's with_on_* callbacks report the same"
    );
}

// ---------------------------------------------------------------- DAT-009

/// DAT-009: each pointer action of a list has a key that reaches the same
/// state.
#[test]
#[serial_test::serial(theme)]
fn dat_009_every_pointer_action_of_a_list_has_a_key() {
    let _theme = Active::set(probe());
    let labels = ["Apple", "Banana", "Lemon", "Lime", "Mango"];
    let base = shown(in_box(list_of(fruits())), "Lemon");
    let (column, row) = at(&base, "Lemon");
    let (clicked, by_click) = list_after(fruits(), "Apple", click(column + 1, row));
    let (keyed, by_keys) = list_after(
        fruits(),
        "Apple",
        vec![key(KeyCode::Down), key(KeyCode::Down)],
    );
    assert_eq!(
        (cursor_among(&clicked, &labels), by_click.last()),
        (cursor_among(&keyed, &labels), by_keys.last()),
        "DAT-009: a click on Lemon and Down twice put the cursor on Lemon and report it:\n{}\n---\n{}",
        clicked.text,
        keyed.text
    );

    let confirming = |events: Vec<Option<Event>>| {
        let confirmed: Log<String> = Arc::default();
        settled(
            in_box(
                builder::list()
                    .items(fruits())
                    .on_confirm(recorder(&confirmed))
                    .build()
                    .auto_focus(),
            ),
            "Apple",
            events,
        );
        held(&confirmed)
    };
    let (column, row) = at(&base, "Banana");
    assert_eq!(
        confirming(double_click(column + 1, row)),
        confirming(vec![key(KeyCode::Down), key(KeyCode::Enter)]),
        "DAT-009: a double click on Banana and Down then Enter confirm the same item"
    );

    // Twenty notches move the view past its first item whether a notch
    // scrolls the view or moves the cursor.
    let notches = |lines: f32| (0..20).map(|_| wheel(50, 10, lines)).collect::<Vec<_>>();
    let thousand = |events: Vec<Option<Event>>| {
        settled(in_box(list_of(numbered(1000)).auto_focus()), "Item", events)
    };
    for (what, frame) in [
        ("the wheel down", thousand(notches(1.0))),
        ("Page Down", thousand(vec![key(KeyCode::PageDown)])),
    ] {
        assert!(
            find(&frame, &item(1)).is_none(),
            "DAT-009: {what} moves the list past its first item:\n{}",
            frame.text
        );
    }
    for (what, frame) in [
        (
            "the wheel up",
            thousand([vec![key(KeyCode::End)], notches(-1.0)].concat()),
        ),
        (
            "Page Up",
            thousand(vec![key(KeyCode::End), key(KeyCode::PageUp)]),
        ),
    ] {
        assert!(
            find(&frame, &item(1000)).is_none(),
            "DAT-009: {what} from the end moves the list off its last item:\n{}",
            frame.text
        );
    }
}

// ---------------------------------------------------------------- NAV-010, DAT-009: the screen reader

#[cfg(target_os = "linux")]
mod reader {
    use super::*;
    use crate::common::app_input::{FramePredicate, Input, ReaderRequest};
    use accesskit::{Action, ActionData, Node, NodeId, Orientation, TreeUpdate};
    use reactive_tui::accessibility::{Live, ReaderChannel, Role};

    /// A frame predicate for a frame whose text contains `text`.
    fn painted(text: &'static str) -> FramePredicate {
        Box::new(move |frame: &Snapshot| frame.text.contains(text))
    }

    /// A frame predicate for a frame whose text does not contain `text`.
    fn absent(text: &'static str) -> FramePredicate {
        Box::new(move |frame: &Snapshot| !frame.text.contains(text))
    }

    /// A frame predicate for a split whose first divider stands in
    /// `column`.
    fn divider_at(column: u16) -> FramePredicate {
        Box::new(move |frame: &Snapshot| glyph(frame, column, 5) == upright())
    }

    /// A frame predicate for a log view in the box of 20 rows that follows
    /// with `text` on its last line.
    fn following_at(text: &'static str) -> FramePredicate {
        Box::new(move |frame: &Snapshot| log_line(frame, 19) == text && !frame.text.contains('↓'))
    }

    /// An input event as a step's input.
    fn event(event: Option<Event>) -> Option<Input> {
        event.map(Input::Event)
    }

    /// A screen reader's request on the first node with `role`, named
    /// `label` when one is given.
    fn ask(
        role: Role,
        label: Option<&'static str>,
        action: Action,
        data: Option<ActionData>,
    ) -> Option<Input> {
        Some(Input::Request(ReaderRequest {
            role,
            label,
            action,
            data,
        }))
    }

    /// The tree the App published last.
    fn published(reader: &ReaderChannel) -> TreeUpdate {
        reader.tree().expect("the App published a tree")
    }

    /// The node with `id` in `tree`.
    fn by_id(tree: &TreeUpdate, id: NodeId) -> Option<&Node> {
        tree.nodes
            .iter()
            .find(|(node_id, _)| *node_id == id)
            .map(|(_, node)| node)
    }

    /// The nodes of `tree` with `role`.
    fn of_role(tree: &TreeUpdate, role: Role) -> Vec<&Node> {
        tree.nodes
            .iter()
            .map(|(_, node)| node)
            .filter(|node| node.role() == role)
            .collect()
    }

    /// A node's name: its label, or the label of the node it is labelled
    /// by.
    fn name_of<'a>(tree: &'a TreeUpdate, node: &'a Node) -> Option<&'a str> {
        node.label().or_else(|| {
            node.labelled_by()
                .first()
                .and_then(|id| by_id(tree, *id))
                .and_then(Node::label)
        })
    }

    /// The node with `role` named `name`.
    fn named<'a>(tree: &'a TreeUpdate, role: Role, name: &str) -> &'a Node {
        of_role(tree, role)
            .into_iter()
            .find(|node| name_of(tree, node) == Some(name))
            .unwrap_or_else(|| {
                panic!(
                    "no {role:?} node named {name:?} in the published tree; its {role:?} nodes are named {:?}",
                    of_role(tree, role)
                        .into_iter()
                        .map(|node| name_of(tree, node))
                        .collect::<Vec<_>>()
                )
            })
    }

    /// The one node of `tree` with `role`.
    fn only(tree: &TreeUpdate, role: Role) -> &Node {
        let nodes = of_role(tree, role);
        assert_eq!(
            nodes.len(),
            1,
            "one {role:?} node in the published tree, not {}",
            nodes.len()
        );
        nodes[0]
    }

    /// Whether a node with `role` lies under `node`.
    fn holds(tree: &TreeUpdate, node: &Node, role: Role) -> bool {
        let mut pending: Vec<NodeId> = node.children().to_vec();
        while let Some(id) = pending.pop() {
            if let Some(child) = by_id(tree, id) {
                if child.role() == role {
                    return true;
                }
                pending.extend_from_slice(child.children());
            }
        }
        false
    }

    /// Runs `root` at 240 by 60 with an in-process screen reader through
    /// `steps`.
    fn read(
        root: impl RootComponent + 'static,
        reader: &ReaderChannel,
        steps: Vec<(FramePredicate, Option<Input>)>,
    ) -> Vec<Snapshot> {
        app_input::run_when_frame_reading(root, SCREEN, reader, steps)
    }

    /// The tree `root`, on the page, publishes through `steps`.
    fn tree_after(root: Element, steps: Vec<(FramePredicate, Option<Input>)>) -> TreeUpdate {
        let reader = ReaderChannel::new();
        read(Control(page(root)), &reader, steps);
        published(&reader)
    }

    /// The tree `root`, on the page, publishes once `until` holds.
    fn tree_of(root: Element, until: FramePredicate) -> TreeUpdate {
        tree_after(root, vec![(until, None)])
    }

    #[test]
    fn nav_010_a_divider_is_a_splitter_with_its_orientation_value_bounds_and_actions() {
        let tree = tree_of(
            boxed(
                100,
                20,
                builder::resizable()
                    .aria_label("Panes")
                    .pane_with(Pane::new(filled(0)).size(49))
                    .pane(filled(1))
                    .build(),
            ),
            painted("First pane"),
        );
        let splitter = named(&tree, Role::Splitter, "Panes");
        assert_eq!(
            splitter.orientation(),
            Some(Orientation::Vertical),
            "NAV-010: a divider between panes side by side is a vertical splitter"
        );
        assert_eq!(
            splitter.numeric_value(),
            Some(49.0),
            "NAV-010: the value is the size of the pane before the divider"
        );
        assert_eq!(
            splitter.min_numeric_value(),
            Some(4.0),
            "NAV-010: the minimum is the least that pane can take"
        );
        assert_eq!(
            splitter.max_numeric_value(),
            Some(95.0),
            "NAV-010: the maximum is the greatest that pane can take"
        );
        for action in [Action::Increment, Action::Decrement, Action::SetValue] {
            assert!(
                splitter.supports_action(action),
                "NAV-010: the splitter advertises {action:?}"
            );
        }

        let tree = tree_of(boxed(100, 20, plain_split_49_50()), painted("First pane"));
        assert_eq!(
            name_of(&tree, only(&tree, Role::Splitter)),
            None,
            "NAV-010: a split whose props set no aria_label names its divider nothing"
        );

        let tree = tree_of(
            boxed(
                100,
                20,
                builder::resizable()
                    .vertical()
                    .pane(filled(0))
                    .pane(filled(1))
                    .build(),
            ),
            painted("First pane"),
        );
        assert_eq!(
            only(&tree, Role::Splitter).orientation(),
            Some(Orientation::Horizontal),
            "NAV-010: a divider between stacked panes is a horizontal splitter"
        );

        let tree = tree_of(
            boxed(
                100,
                20,
                builder::resizable()
                    .pane_with(Pane::new(filled(0)).size(49))
                    .pane(filled(1))
                    .disabled(true)
                    .build(),
            ),
            painted("First pane"),
        );
        assert!(
            only(&tree, Role::Splitter).is_disabled(),
            "NAV-010: a disabled split's divider is marked disabled"
        );
    }

    #[test]
    fn nav_010_set_value_increment_and_decrement_move_the_divider_as_its_keys_do() {
        for (action, data, column, sizes) in [
            (
                Action::SetValue,
                Some(ActionData::NumericValue(30.0)),
                30,
                vec![30, 69],
            ),
            (Action::Increment, None, 50, vec![50, 49]),
            (Action::Decrement, None, 48, vec![48, 51]),
        ] {
            let log: Log<Vec<u16>> = Arc::default();
            let reader = ReaderChannel::new();
            let frames = read(
                Control(page(boxed(100, 20, split_49_50(&log)))),
                &reader,
                vec![
                    (
                        painted("First pane"),
                        ask(Role::Splitter, None, action, data),
                    ),
                    (divider_at(column), None),
                ],
            );
            assert_eq!(
                dividers(last(&frames), 5),
                vec![column],
                "NAV-010: {action:?} moves the divider to column {column}:\n{}",
                last(&frames).text
            );
            assert_eq!(
                held(&log),
                vec![sizes],
                "NAV-010: {action:?} reports the panes' sizes once, as a key's move does"
            );
        }
    }

    #[test]
    fn nav_010_a_sidebar_is_a_navigation_landmark_holding_a_tree_of_its_items() {
        let items = || {
            let mut items = nav_items();
            items.push(SidebarItem::new("archive", "Archive").disabled(true));
            items
        };
        let sidebar = || {
            boxed(
                100,
                20,
                builder::sidebar()
                    .aria_label("Main")
                    .items(items())
                    .active("home")
                    .build(),
            )
        };
        let tree = tree_of(sidebar(), painted("Archive"));
        let navigation = named(&tree, Role::Navigation, "Main");
        assert!(
            holds(&tree, navigation, Role::Tree),
            "NAV-010: the navigation landmark holds a tree"
        );
        for (label, position) in [
            ("Home", 1),
            ("Projects", 2),
            ("Settings", 3),
            ("Archive", 4),
        ] {
            let item = named(&tree, Role::TreeItem, label);
            assert_eq!(
                (item.level(), item.position_in_set(), item.size_of_set()),
                (Some(1), Some(position), Some(4)),
                "NAV-010: {label} tells its level, its position and the count of its siblings"
            );
        }
        assert_eq!(
            named(&tree, Role::TreeItem, "Projects").is_expanded(),
            Some(false),
            "NAV-010: a closed group is marked not expanded"
        );
        assert_eq!(
            named(&tree, Role::TreeItem, "Home").is_selected(),
            Some(true),
            "NAV-010: the active item is marked selected"
        );
        assert_ne!(
            named(&tree, Role::TreeItem, "Settings").is_selected(),
            Some(true),
            "NAV-010: an item that is not active is not marked selected"
        );
        assert!(
            named(&tree, Role::TreeItem, "Archive").is_disabled(),
            "NAV-010: a disabled item is marked disabled"
        );

        let tree = tree_after(
            sidebar(),
            vec![
                (painted("Home"), event(key(KeyCode::Tab))),
                (painted("Home"), event(key(KeyCode::Down))),
                (painted("Home"), event(key(KeyCode::Right))),
                (painted("Alpha"), None),
            ],
        );
        assert_eq!(
            named(&tree, Role::TreeItem, "Projects").is_expanded(),
            Some(true),
            "NAV-010: an open group is marked expanded"
        );
        for (label, position) in [("Alpha", 1), ("Beta", 2)] {
            let item = named(&tree, Role::TreeItem, label);
            assert_eq!(
                (item.level(), item.position_in_set(), item.size_of_set()),
                (Some(2), Some(position), Some(2)),
                "NAV-010: the child {label} tells its level, its position and the count of its siblings"
            );
        }

        let tree = tree_of(sidebar_of(nav_items()), painted("Home"));
        assert_eq!(
            name_of(&tree, only(&tree, Role::Navigation)),
            None,
            "NAV-010: a sidebar whose props set no aria_label has no name"
        );
    }

    #[test]
    fn nav_010_the_toggle_is_a_button_named_by_the_sidebar_and_expanded_with_it() {
        let tree = tree_of(
            boxed(
                100,
                20,
                builder::sidebar()
                    .aria_label("Main")
                    .items(nav_items())
                    .build(),
            ),
            painted("Home"),
        );
        assert_eq!(
            named(&tree, Role::Button, "Main").is_expanded(),
            Some(true),
            "NAV-010: the toggle of an expanded sidebar is a button named by the sidebar, marked expanded"
        );
        let tree = tree_of(
            boxed(
                100,
                20,
                builder::sidebar()
                    .aria_label("Main")
                    .items(nav_items())
                    .collapsed(true)
                    .build(),
            ),
            painted(Icon::Folder.glyph()),
        );
        assert_eq!(
            named(&tree, Role::Button, "Main").is_expanded(),
            Some(false),
            "NAV-010: the toggle of a collapsed sidebar is marked not expanded"
        );
    }

    #[test]
    fn nav_010_a_log_view_is_a_polite_log_with_the_offsets_of_a_scroll_view() {
        let tree = tree_of(
            boxed(
                100,
                20,
                builder::log_view()
                    .aria_label("Build log")
                    .handle(hundred())
                    .build(),
            ),
            painted("Log row 100"),
        );
        let log = named(&tree, Role::Log, "Build log");
        assert_eq!(
            log.live(),
            Some(Live::Polite),
            "NAV-010: a log view announces its rows politely"
        );
        assert_eq!(
            (log.scroll_y(), log.scroll_y_min(), log.scroll_y_max()),
            (Some(80.0), Some(0.0), Some(80.0)),
            "NAV-010: a log view of 100 rows in 20 lines following its last row is at offset 80 of 0 to 80"
        );

        let tree = tree_of(
            boxed(100, 20, builder::log_view().handle(hundred()).build()),
            painted("Log row 100"),
        );
        assert_eq!(
            name_of(&tree, only(&tree, Role::Log)),
            None,
            "NAV-010: a log view whose props set no aria_label has no name"
        );
    }

    #[test]
    fn nav_010_the_new_bar_is_a_button_named_by_its_text_whose_click_resumes_the_following() {
        for (word, name) in [(None, "↓ 5 new"), (Some("neu"), "↓ 5 neu")] {
            let handle = hundred();
            let reader = ReaderChannel::new();
            read(
                log_root(&handle, word, vec![append(&handle, 101, 5)]),
                &reader,
                vec![
                    (painted("Log row 100"), event(key(KeyCode::PageUp))),
                    (absent("Log row 100"), event(change())),
                    (painted(name), None),
                ],
            );
            named(&published(&reader), Role::Button, name);
        }

        let handle = hundred();
        let reader = ReaderChannel::new();
        let frames = read(
            log_root(&handle, None, vec![append(&handle, 101, 5)]),
            &reader,
            vec![
                (painted("Log row 100"), event(key(KeyCode::PageUp))),
                (absent("Log row 100"), event(change())),
                (
                    painted("↓ 5 new"),
                    ask(Role::Button, Some("↓ 5 new"), Action::Click, None),
                ),
                (following_at("Log row 105"), None),
            ],
        );
        let end = last(&frames);
        assert_eq!(
            log_line(end, 19),
            "Log row 105",
            "NAV-010: the bar's Click resumes the following:\n{}",
            end.text
        );
    }

    #[test]
    fn dat_009_a_list_is_a_list_box_named_by_its_aria_label_only() {
        let tree = tree_of(
            in_box(builder::list().aria_label("Fruits").items(fruits()).build()),
            painted("Apple"),
        );
        named(&tree, Role::ListBox, "Fruits");

        let tree = tree_of(
            in_box(Element::typed::<List>(ListProps {
                items: fruits(),
                selected: None,
                filterable: false,
                has_more: false,
                loading: false,
                empty_text: None,
                aria_label: Some("Fruits".to_owned()),
                width: None,
                height: None,
            })),
            painted("Apple"),
        );
        named(&tree, Role::ListBox, "Fruits");

        let tree = tree_of(in_box(list_of(fruits())), painted("Apple"));
        assert_eq!(
            name_of(&tree, only(&tree, Role::ListBox)),
            None,
            "DAT-009: a list whose props set no aria_label has no name"
        );
    }

    #[test]
    fn dat_009_each_option_tells_its_label_position_count_and_selected_state() {
        let tree = tree_of(
            in_box(
                builder::list()
                    .aria_label("Fruits")
                    .items(fruits())
                    .selected("banana")
                    .build(),
            ),
            painted("Mango"),
        );
        for (label, position) in [
            ("Apple", 1),
            ("Banana", 2),
            ("Lemon", 3),
            ("Lime", 4),
            ("Mango", 5),
        ] {
            let option = named(&tree, Role::ListBoxOption, label);
            assert_eq!(
                (option.position_in_set(), option.size_of_set()),
                (Some(position), Some(5)),
                "DAT-009: {label} tells its position among the items shown and their count"
            );
            assert_eq!(
                option.is_selected(),
                Some(label == "Banana"),
                "DAT-009: {label} tells whether it is selected"
            );
            assert_eq!(
                option.is_disabled(),
                label == "Mango",
                "DAT-009: only the disabled Mango is marked disabled"
            );
        }
        named(&tree, Role::Group, "Citrus");
    }

    #[test]
    fn dat_009_the_options_of_a_list_of_100000_items_tell_their_place_among_all_of_them() {
        let tree = tree_after(
            in_box(
                builder::list()
                    .aria_label("Numbers")
                    .items(numbered(100_000))
                    .build()
                    .auto_focus(),
            ),
            vec![
                (painted("Item 000001"), event(key(KeyCode::End))),
                (painted("Item 100000"), None),
            ],
        );
        let option = named(&tree, Role::ListBoxOption, "Item 100000");
        assert_eq!(
            (option.position_in_set(), option.size_of_set()),
            (Some(100_000), Some(100_000)),
            "DAT-009: the last of 100,000 items is at position 100,000 of 100,000"
        );

        // The list shows its selected item, item 70,000.
        let tree = tree_of(
            in_box(
                builder::list()
                    .aria_label("Numbers")
                    .items(numbered(100_000))
                    .selected("i070000")
                    .build(),
            ),
            painted("Item 070000"),
        );
        let option = named(&tree, Role::ListBoxOption, "Item 070000");
        assert_eq!(
            (
                option.position_in_set(),
                option.size_of_set(),
                option.is_selected()
            ),
            (Some(70_000), Some(100_000), Some(true)),
            "DAT-009: item 70,000 is at position 70,000 of 100,000, selected"
        );
    }

    #[test]
    fn dat_009_the_filter_is_a_search_input_labelled_by_the_lists_name() {
        let tree = tree_after(
            in_box(
                builder::list()
                    .aria_label("Fruits")
                    .items(fruits())
                    .filterable(true)
                    .build()
                    .auto_focus(),
            ),
            vec![
                (painted("Apple"), event(key(KeyCode::Char('l')))),
                (painted("Apple"), event(key(KeyCode::Char('e')))),
                (absent("Lime"), None),
            ],
        );
        named(&tree, Role::SearchInput, "Fruits");
        for (label, position) in [("Apple", 1), ("Lemon", 2)] {
            let option = named(&tree, Role::ListBoxOption, label);
            assert_eq!(
                (option.position_in_set(), option.size_of_set()),
                (Some(position), Some(2)),
                "DAT-009: under the filter le, {label} tells its position among the 2 items shown"
            );
        }
        named(&tree, Role::Group, "Citrus");
    }
}
