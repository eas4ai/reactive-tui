//! pixel-looks mechanism: PIX-001 to PIX-006 (docs/spec/pixel-looks.md). Built
//! with `wgpu-graphics`.
//!
//! A widget's or element's look is one pixel picture over its rectangle where
//! the terminal takes Kitty graphics or Sixel, drawn around its text, with its
//! cell look kept as the fallback. The tests run an App on a backend that
//! writes to memory, decode the Kitty pictures the backend sends, and compare
//! them with the reference pictures under tests/snapshots/pixel-looks, which
//! `REGENERATE=1` refreshes and nothing else writes (BAR-004's rule).

mod canvas_support;
use canvas_support::app::*;

use reactive_tui::backend::ImageOutputOptions;
use reactive_tui::builder::{self, core::div};
use reactive_tui::component::Element;
use reactive_tui::event::types::{Event, KeyCode, KeyEvent, MouseEvent, MouseEventKind, Position};
use reactive_tui::graphics::{CanvasOutput, GraphicsFrame, GraphicsOptions};
use reactive_tui::widgets::display::{ProgressBar, ProgressBarProps};
use reactive_tui::widgets::input::{InputMode, TextInputProps};
use reactive_tui::widgets::layout::ScrollViewBuilder;
use reactive_tui::widgets::TextInput;
use std::io;
use std::sync::Mutex;
use std::time::Duration;

/// The terminal of the content tests.
const SIZE: (u16, u16) = (80, 24);

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code))
}

fn mouse(kind: MouseEventKind, x: u16, y: u16) -> Event {
    Event::Mouse(MouseEvent::new(kind, Position::cell(x, y)))
}

/// A mouse event with the left button: a press, a drag or a release.
fn pressed(kind: MouseEventKind, x: u16, y: u16) -> Event {
    let mut event = MouseEvent::new(kind, Position::cell(x, y));
    event.button = reactive_tui::event::types::MouseButton::Left;
    Event::Mouse(event)
}

/// A box of `width` by `height` cells holding `element`, at the top left.
fn sized(width: u16, height: u16, element: Element) -> Element {
    div()
        .class(&format!("w-{width} h-{height}"))
        .child(element)
        .build()
}

// ---------------------------------------------------------------------------
// The looks under test.

fn save_button() -> Element {
    builder::primary_button("Save", || {})
}

fn cancel_button() -> Element {
    builder::button().text("Cancel").on_click(|| {}).build()
}

/// A card holding one line of text that fills the box it is given: PIX-003's
/// case is 40 by 8 cells.
fn card() -> Element {
    let mut card = builder::card(vec![builder::text("Reactive TUI")]);
    card.class = Some(format!(
        "{} w-full h-full",
        card.class.as_deref().unwrap_or_default()
    ));
    card
}

fn text_input(value: &str, placeholder: &str) -> Element {
    builder::text_input()
        .value(value)
        .placeholder(placeholder)
        .build()
}

fn multiline_input() -> Element {
    Element::typed::<TextInput>(TextInputProps {
        value: "one\ntwo\nthree".into(),
        mode: InputMode::MultiLine { height: 4 },
        ..Default::default()
    })
}

fn checkbox(label: &str, checked: bool) -> Element {
    builder::checkbox().label(label).checked(checked).build()
}

fn radios(chosen: usize) -> Element {
    let mut column = div().class("flex-col");
    for (index, (value, label)) in [("a", "Alpha"), ("b", "Beta"), ("c", "Gamma")]
        .iter()
        .enumerate()
    {
        column = column.child(
            builder::radio_button()
                .group("letters")
                .value(value)
                .label(label)
                .checked(index == chosen)
                .build(),
        );
    }
    column.build()
}

fn slider(value: f64) -> Element {
    builder::slider().value(value).min(0.0).max(100.0).build()
}

fn progress(value: f64) -> Element {
    builder::progress_bar()
        .value(value)
        .max_value(100.0)
        .show_percentage(false)
        .build()
}

/// An indeterminate bar with no text, so its one row is the track: the
/// widget's own text for one is `Loading...`, which takes a row of its own.
fn indeterminate_progress(reduced_motion: bool) -> Element {
    Element::typed::<ProgressBar>(ProgressBarProps {
        indeterminate: true,
        animated: true,
        show_percentage: false,
        custom_formatter: Some(std::sync::Arc::new(|_, _, _| String::new())),
        // The bar reads `reduced-motion` from its own style classes
        // (`ProgressBarBuilder::style`).
        style: reduced_motion.then(|| "reduced-motion".to_owned()),
        ..Default::default()
    })
}

/// The columns of row `row` whose cells spell `text`, from the screen.
fn columns_of(screen: &vt100::Screen, row: usize, text: &str) -> Option<std::ops::Range<usize>> {
    let line: String = (0..screen.size().1 as usize)
        .map(|c| {
            let t = cell_text(screen, row, c);
            if t.is_empty() {
                " ".to_owned()
            } else {
                t
            }
        })
        .collect();
    line.find(text).map(|at| at..at + text.chars().count())
}

fn bg(screen: &vt100::Screen, row: usize, column: usize) -> Option<[u8; 3]> {
    match screen.cell(row as u16, column as u16)?.bgcolor() {
        vt100::Color::Rgb(r, g, b) => Some([r, g, b]),
        _ => None,
    }
}

fn fg(screen: &vt100::Screen, row: usize, column: usize) -> Option<[u8; 3]> {
    match screen.cell(row as u16, column as u16)?.fgcolor() {
        vt100::Color::Rgb(r, g, b) => Some([r, g, b]),
        _ => None,
    }
}

/// Whether any cell of the screen's rectangle holds one of `glyphs`.
fn glyph_in(
    screen: &vt100::Screen,
    rows: std::ops::Range<usize>,
    glyphs: &[char],
) -> Option<String> {
    for row in rows {
        for column in 0..screen.size().1 as usize {
            let text = cell_text(screen, row, column);
            if text.chars().any(|c| glyphs.contains(&c)) {
                return Some(format!("{text:?} at row {row}, column {column}"));
            }
        }
    }
    None
}

/// A stop rule that ends `settle` frames after the `count`th picture, or
/// after `frames` frames: a run on a tree that sends no picture ends on the
/// frame count, not on the guard.
fn pictures_or_frames(count: usize, settle: usize, frames: usize) -> Stop {
    let reached = Mutex::new(None);
    Box::new(move |frame, pictures, elapsed| {
        let mut reached = reached.lock().unwrap();
        if pictures >= count && reached.is_none() {
            *reached = Some(frame);
        }
        reached.is_some_and(|at| frame >= at + settle) || frame >= frames || elapsed >= GUARD
    })
}

/// Pictures sent after frame `frame`, per image id.
fn pictures_after(run: &Run, frame: usize) -> Vec<Picture> {
    run.pictures()
        .into_iter()
        .filter(|p| p.frame > frame)
        .collect()
}

// ---------------------------------------------------------------------------
// PIX-001: where a pixel look goes.

#[test]
fn pix_001_a_primary_button_on_a_kitty_host_sends_a_picture_over_its_cells() {
    let run = run(
        sized(8, 1, save_button()),
        (40, 3),
        kitty(),
        silent(),
        pictures_or_frames(1, 2, 90),
        true,
    );
    let picture = run.first_picture("PIX-001: a primary button of 8 by 1 cells");
    let screen = run.screen(picture.frame);
    let label = columns_of(&screen, 0, "Save").expect("the label is cell text");
    // The button is as wide as its label and one cell of padding at each
    // side (CTL-002): the picture covers those cells and no other.
    assert_eq!(
        picture.rect(),
        (0, label.start - 1, 1, label.len() + 2),
        "PIX-001: the placement covers the button's cells"
    );
    assert_eq!(
        picture.size,
        ((label.len() as u32 + 2) * 8, 16),
        "PIX-001: one picture pixel per screen pixel"
    );
    for column in label.clone() {
        for x in (column as u32 * 8)..(column as u32 * 8 + 8) {
            for y in 0..16 {
                assert_eq!(
                    picture.pixel(x, y)[3],
                    0,
                    "PIX-001: the pixels under the label cell {column} are cut out"
                );
            }
        }
        let background = bg(&screen, 0, column).expect("the label cell has a background");
        assert!(
            near([background[0], background[1], background[2], 255], role("primary"), 2),
            "PIX-001: the label cell {column} has the button's fill as its background: {background:?}"
        );
    }
    assert!(
        run.frames.iter().all(|frame| frame.image_nodes == 0),
        "PIX-001: a look's picture has no screen-reader node of its own"
    );
}

#[test]
fn pix_001_before_its_first_picture_a_checkbox_shows_no_glyph_of_the_cell_look() {
    let run = run(
        sized(30, 1, checkbox("Capture", true)),
        (40, 3),
        kitty(),
        silent(),
        pictures_or_frames(1, 1, 60),
        true,
    );
    let until = run
        .pictures()
        .first()
        .map_or(run.frames.len(), |picture| picture.frame + 1);
    for frame in 0..until {
        let screen = run.screen(frame);
        if let Some(found) = glyph_in(&screen, 0..1, &['[', ']', '✓', '▬', '(', ')', '●']) {
            panic!("PIX-001: frame {frame} before the first picture holds a glyph of the cell look: {found}");
        }
    }
}

/// A primary button whose theme switches after its first picture: the next
/// picture is in the new theme's colors and none after it in the old. Run in
/// a process of its own, since the theme is the process's.
#[test]
#[ignore = "run by pix_001_after_a_theme_change_the_next_picture_is_in_the_new_colors"]
fn theme_change_child() {
    use reactive_tui::theme::{dark_theme, light_theme, Theme};
    Theme::set_active(dark_theme());
    let old = role("primary");
    let switched = std::sync::Arc::new(Mutex::new(None::<usize>));
    let flag = switched.clone();
    let script: Script = Box::new(move |presented, pictures| {
        let mut at = flag.lock().unwrap();
        if at.is_none() && pictures >= 1 && presented >= 3 {
            Theme::set_active(light_theme());
            *at = Some(presented);
        }
        None
    });
    let run = run(
        sized(8, 1, save_button()),
        (40, 3),
        kitty(),
        script,
        pictures_or_frames(2, 6, 120),
        true,
    );
    let at = switched.lock().unwrap().unwrap_or_else(|| {
        panic!(
            "PIX-001: the theme never switched: the button sent no first picture in {} frames",
            run.frames.len()
        )
    });
    let new = role("primary");
    assert!(
        !near([old[0], old[1], old[2], 255], new, 12),
        "the two presets differ in primary"
    );
    let after = pictures_after(&run, at - 1);
    let dominant = |picture: &Picture| -> [u8; 3] {
        let mut counts: std::collections::HashMap<[u8; 3], usize> = Default::default();
        for pixel in &picture.pixels {
            if pixel[3] > 200 {
                *counts.entry([pixel[0], pixel[1], pixel[2]]).or_default() += 1;
            }
        }
        counts
            .into_iter()
            .max_by_key(|(_, n)| *n)
            .map_or([0; 3], |(c, _)| c)
    };
    assert!(
        after.iter().any(|p| near([dominant(p)[0], dominant(p)[1], dominant(p)[2], 255], new, 12)),
        "PIX-001: a picture in the new theme's colors followed the change ({} pictures after frame {at})",
        after.len()
    );
    let stale: Vec<usize> = after
        .iter()
        .filter(|p| {
            near(
                [dominant(p)[0], dominant(p)[1], dominant(p)[2], 255],
                old,
                12,
            )
        })
        .map(|p| p.frame)
        .collect();
    assert!(
        stale.is_empty(),
        "PIX-001: frames {stale:?} after the theme change carried a picture in the old colors"
    );
    // The first frame presented after the change shows nothing in the old
    // colors: a picture the terminal still holds from before the change is
    // taken off the screen in that frame, unless the new picture replaces
    // it there (THM-003).
    let id = run
        .pictures()
        .into_iter()
        .find(|p| p.frame < at)
        .and_then(|p| p.id)
        .expect("the button's picture before the change has an image id");
    assert!(
        run.frames.len() > at,
        "PIX-001: no frame was presented after the theme change"
    );
    let removed = kitty_commands(&run.frames[at..=at])
        .iter()
        .any(|c| c.id == Some(id) && c.action == "d");
    let replaced = run.pictures().iter().any(|p| {
        p.frame == at
            && p.id == Some(id)
            && near(
                [dominant(p)[0], dominant(p)[1], dominant(p)[2], 255],
                new,
                12,
            )
    });
    assert!(
        removed || replaced,
        "THM-003: the first frame after the theme change (frame {at}) left the button's old-theme picture {id} on the screen: {:?}",
        kitty_commands(&run.frames[at..=at])
            .iter()
            .map(|c| (c.action.clone(), c.id))
            .collect::<Vec<_>>()
    );
    Theme::set_active(dark_theme());
}

#[test]
fn pix_001_after_a_theme_change_the_next_picture_is_in_the_new_colors() {
    let child = std::process::Command::new(std::env::current_exe().expect("the test binary"))
        .args(["--exact", "theme_change_child", "--ignored", "--nocapture"])
        .output()
        .expect("the test binary runs");
    let said = String::from_utf8_lossy(&child.stdout).into_owned()
        + &String::from_utf8_lossy(&child.stderr);
    assert!(
        child.status.success() && said.contains("1 passed"),
        "PIX-001: {}",
        &said[said.find("PIX-001").unwrap_or(0)..]
    );
}

/// A checked checkbox on a Kitty host with the looks switched back to cells by
/// the environment or by the application's graphics options: no picture, and
/// the cells are the cell look's. Run in a process of its own, since both
/// choices are made once per process.
#[test]
#[ignore = "run by pix_001_without_pixels_and_with_the_switches_the_cells_are_the_fallbacks"]
fn cells_child() {
    if std::env::var("PIXEL_LOOKS_SWITCH").as_deref() == Ok("options") {
        reactive_tui::widgets::display::charts::set_graphics_options(GraphicsOptions {
            output: Some(CanvasOutput::Blocks),
            ..Default::default()
        });
    }
    let run = run(
        sized(30, 1, checkbox("Capture", true)),
        (40, 3),
        kitty(),
        silent(),
        after_frames(6),
        true,
    );
    assert!(
        run.pictures().is_empty(),
        "PIX-001: switched back to cells, a Kitty host is sent no picture"
    );
    let screen = run.screen(run.frames.len() - 1);
    assert!(
        columns_of(&screen, 0, "[✓]").is_some(),
        "PIX-001: the cells are the cell look's: {:?}",
        screen.contents()
    );
    println!(
        "PIXEL_LOOKS_SCREEN {}",
        screen.contents().replace('\n', "|")
    );
}

#[test]
fn pix_001_without_pixels_and_with_the_switches_the_cells_are_the_fallbacks() {
    let run = run(
        sized(30, 1, checkbox("Capture", true)),
        (40, 3),
        no_pixels(),
        silent(),
        after_frames(6),
        true,
    );
    assert!(
        run.pictures().is_empty(),
        "PIX-001: without pixels no picture is sent"
    );
    let fallback = run
        .screen(run.frames.len() - 1)
        .contents()
        .replace('\n', "|");
    assert!(
        fallback.contains("[✓]"),
        "PIX-001: the cell look without pixels: {fallback:?}"
    );
    for (name, value) in [
        ("REACTIVE_TUI_CANVAS", "blocks"),
        ("PIXEL_LOOKS_SWITCH", "options"),
    ] {
        let child = std::process::Command::new(std::env::current_exe().expect("the test binary"))
            .args(["--exact", "cells_child", "--ignored", "--nocapture"])
            .env(name, value)
            .output()
            .expect("the test binary runs");
        let said = String::from_utf8_lossy(&child.stdout).into_owned()
            + &String::from_utf8_lossy(&child.stderr);
        assert!(
            child.status.success() && said.contains("1 passed"),
            "PIX-001 with {name}={value}: {}",
            &said[said.find("PIX-001").unwrap_or(0)..]
        );
        let shown = said
            .lines()
            .find_map(|line| line.strip_prefix("PIXEL_LOOKS_SCREEN "))
            .expect("the child printed its screen");
        assert_eq!(
            shown, fallback,
            "PIX-001 with {name}={value}: the switched cells equal the fallback's"
        );
    }
}

// ---------------------------------------------------------------------------
// PIX-002: kept, sent when changed, hidden when covered.

#[test]
fn pix_002_an_unchanged_look_sends_no_bytes_after_its_picture() {
    let run = run(
        sized(8, 1, save_button()),
        (40, 3),
        kitty(),
        silent(),
        pictures_or_frames(1, 4, 90),
        true,
    );
    let picture = run.first_picture("PIX-002: a primary button");
    let later: Vec<&KittyCommand> = kitty_commands(&run.frames)
        .iter()
        .filter(|c| c.frame > picture.frame && (c.payload || c.action == "T" || c.action == "t"))
        .cloned()
        .collect::<Vec<_>>()
        .iter()
        .map(|c| Box::leak(Box::new(c.clone())) as &KittyCommand)
        .collect();
    assert!(
        later.is_empty(),
        "PIX-002: frames after the picture's frame {} carried image data for an unchanged look: {later:?}",
        picture.frame
    );
}

#[test]
fn pix_002_a_typed_key_sends_one_picture_for_the_text_input_alone() {
    let tree = div()
        .class("flex-col w-40")
        .child(sized(30, 1, text_input("Re", "").auto_focus()))
        .child(sized(8, 1, save_button()))
        .build();
    let typed = std::sync::Arc::new(Mutex::new(None::<usize>));
    let flag = typed.clone();
    let script: Script = Box::new(move |presented, pictures| {
        let mut at = flag.lock().unwrap();
        if at.is_none() && pictures >= 2 && presented >= 8 {
            *at = Some(presented);
            return Some(key(KeyCode::Char('a')));
        }
        None
    });
    let run = run(
        tree,
        SIZE,
        kitty(),
        script,
        pictures_or_frames(3, 4, 90),
        true,
    );
    let pictures = run.pictures();
    let at = typed.lock().unwrap().unwrap_or_else(|| {
        panic!(
            "PIX-002: the key was never typed: {} pictures came in {} frames",
            pictures.len(),
            run.frames.len()
        )
    });
    let before: Vec<&Picture> = pictures.iter().filter(|p| p.frame < at).collect();
    let input_id = before
        .iter()
        .find(|p| p.rect().1 == 0 && p.rect().3 == 30)
        .and_then(|p| p.id)
        .expect("PIX-002: the text input's picture before the key");
    let after: Vec<&Picture> = pictures.iter().filter(|p| p.frame >= at).collect();
    // The key adds a text cell, so the input's picture loses that cell and
    // is sent once more; no other look sends one.
    assert_eq!(
        after.len(),
        1,
        "PIX-002: one new picture after the key, got {:?}",
        after.iter().map(|p| (p.frame, p.id)).collect::<Vec<_>>()
    );
    assert_eq!(
        after[0].id,
        Some(input_id),
        "PIX-002: the new picture is the text input's"
    );
}

/// A focused scroll view of 20 by 4 cells holding a head line, a primary
/// button and twelve more lines, with a script that presses Down once the
/// first picture is sent and three frames are presented; the frame the key
/// was sent at.
fn scrolling_button() -> (Element, std::sync::Arc<Mutex<Option<usize>>>, Script) {
    let mut content = div().class("flex-col w-20");
    content = content.child(builder::text("head"));
    content = content.child(sized(8, 1, save_button()));
    for row in 0..12 {
        content = content.child(builder::text(&format!("line {row}")));
    }
    let scroll = ScrollViewBuilder::new(content.build())
        .scroll_x(false)
        .show_scrollbars(false)
        .render()
        .with_class("w-20 h-4")
        .auto_focus();
    let scrolled = std::sync::Arc::new(Mutex::new(None::<usize>));
    let flag = scrolled.clone();
    let script: Script = Box::new(move |presented, pictures| {
        let mut at = flag.lock().unwrap();
        if at.is_none() && pictures >= 1 && presented >= 3 {
            *at = Some(presented);
            return Some(key(KeyCode::Down));
        }
        None
    });
    (scroll, scrolled, script)
}

#[test]
fn pix_002_a_scrolled_look_is_placed_again_not_sent_again() {
    let (scroll, scrolled, script) = scrolling_button();
    let run = run(
        scroll,
        (40, 6),
        kitty(),
        script,
        pictures_or_frames(1, 8, 90),
        true,
    );
    let first = run.first_picture("PIX-002: a button inside a scroll view");
    let at = scrolled.lock().unwrap().expect("the scroll key was sent");
    let commands = kitty_commands(&run.frames);
    let placed = commands
        .iter()
        .any(|c| c.frame >= at && c.action == "p" && c.id == first.id);
    let resent = commands
        .iter()
        .any(|c| c.frame >= at && c.action == "T" && c.id == first.id);
    assert!(
        placed && !resent,
        "PIX-002: after scrolling, the button's image {:?} is placed again (a=p: {placed}) and not transmitted again (a=T: {resent})",
        first.id
    );
}

#[test]
fn pix_002_a_scrolled_look_on_sixel_is_sent_again_at_its_new_row() {
    let (scroll, scrolled, script) = scrolling_button();
    let run = run(
        scroll,
        (40, 6),
        sixel(),
        script,
        pictures_or_frames(1, 8, 90),
        true,
    );
    let at = scrolled.lock().unwrap().expect("the scroll key was sent");
    let rasters = run.rasters();
    // A raster's `at` is (row, column).
    let row_of = |raster: &Raster| raster.at.map(|(row, _)| row);
    let before: Vec<Option<usize>> = rasters
        .iter()
        .filter(|r| r.frame < at)
        .map(row_of)
        .collect();
    let after: Vec<Option<usize>> = rasters
        .iter()
        .filter(|r| r.frame >= at)
        .map(row_of)
        .collect();
    // The button is on the second row of the view, then on the first once
    // the view scrolled a row: its picture is sent again there.
    assert!(
        before.contains(&Some(1)) && after.contains(&Some(0)),
        "PIX-002: on Sixel the button's picture is sent at row 1 before the scroll and at row 0 after it; rows before {before:?}, after {after:?}"
    );
    // The old row shows the line that scrolled into it, not the picture:
    // the cells under the old picture are written again.
    let screen = run.screen(run.frames.len() - 1);
    assert!(
        columns_of(&screen, 1, "line 0").is_some(),
        "PIX-002: the old row shows the next line after the scroll: {:?}",
        screen.contents()
    );
}

#[test]
fn pix_002_a_removed_look_is_deleted_in_that_frame() {
    let run = run_swapping(
        sized(8, 1, save_button()),
        Some((6, div().class("w-8 h-1").build())),
        (40, 3),
        kitty(),
        silent(),
        pictures_or_frames(1, 10, 90),
        true,
    );
    let first = run.first_picture("PIX-002: a primary button");
    let deleted = kitty_commands(&run.frames)
        .into_iter()
        .find(|c| c.action == "d" && (c.id == first.id || c.key("I").is_some()));
    // The button leaves at frame 6, or in the frame after its first picture
    // when that came later.
    let swapped = (first.frame + 1).max(6).min(run.frames.len());
    assert!(
        deleted.as_ref().is_some_and(|c| c.frame >= swapped && c.frame <= swapped + 1),
        "PIX-002: the button's placement is deleted in the frame it leaves (frame {swapped}); deletions: {deleted:?}"
    );
}

#[test]
fn pix_002_a_look_is_drawn_at_the_terminals_cell_size() {
    let host = ImageOutputOptions {
        cell_pixels: (9, 18),
        ..kitty()
    };
    let run = run(
        sized(8, 1, save_button()),
        (40, 3),
        host,
        silent(),
        pictures_or_frames(1, 2, 90),
        true,
    );
    let picture = run.first_picture("PIX-002: a primary button on 9 by 18 pixel cells");
    // "Save" with a cell of padding at each side is six cells.
    assert_eq!(
        (picture.size, picture.cells),
        ((54, 18), Some((6, 1))),
        "PIX-002: a 6 by 1 look on 9 by 18 pixel cells is a 54 by 18 picture placed over 6 by 1 cells"
    );
}

#[test]
fn pix_002_a_list_opened_over_a_card_hides_the_picture_under_it() {
    let tree = div()
        .class("flex-col w-40")
        .child(sized(
            20,
            1,
            builder::select()
                .option("a", "Alpha")
                .option("b", "Beta")
                .option("c", "Gamma")
                .selected("a")
                .build()
                .auto_focus(),
        ))
        .child(sized(40, 8, card()))
        .build();
    let opened = std::sync::Arc::new(Mutex::new((None::<usize>, None::<usize>)));
    let flag = opened.clone();
    let script: Script = Box::new(move |presented, pictures| {
        let mut state = flag.lock().unwrap();
        if state.0.is_none() && pictures >= 1 && presented >= 3 {
            state.0 = Some(presented);
            return Some(key(KeyCode::Enter));
        }
        if let (Some(open), None) = *state {
            if presented >= open + 4 {
                state.1 = Some(presented);
                return Some(key(KeyCode::Escape));
            }
        }
        None
    });
    let run = run(
        tree,
        SIZE,
        kitty(),
        script,
        pictures_or_frames(1, 12, 120),
        true,
    );
    let first = run.first_picture("PIX-002: a card under a select");
    let (open, close) = *opened.lock().unwrap();
    let (open, close) = (
        open.expect("the list opened"),
        close.expect("the list closed"),
    );
    let covered_screen = run.screen(open);
    let list_rows: Vec<usize> = (1..8)
        .filter(|row| {
            columns_of(&covered_screen, *row, "Beta").is_some()
                || columns_of(&covered_screen, *row, "Gamma").is_some()
        })
        .collect();
    assert!(
        !list_rows.is_empty(),
        "the list opened over the card: {:?}",
        covered_screen.contents()
    );
    let pictures = run.pictures();
    let card_id = first.id;
    let while_open = pictures
        .iter()
        .rfind(|p| p.id == card_id && p.frame >= open && p.frame < close)
        .unwrap_or_else(|| {
            panic!("PIX-002: the card sent a new picture while the list covered it")
        });
    for row in &list_rows {
        let columns = columns_of(&covered_screen, *row, "Alpha")
            .or_else(|| columns_of(&covered_screen, *row, "Beta"))
            .or_else(|| columns_of(&covered_screen, *row, "Gamma"))
            .expect("the list's row");
        let y = ((*row - 1) as u32) * 16 + 8;
        for column in columns {
            assert_eq!(
                while_open.pixel(column as u32 * 8 + 4, y)[3],
                0,
                "PIX-002: the card's picture is cut out under the list's cell at row {row}, column {column}"
            );
        }
    }
    let after_close = pictures
        .iter()
        .rfind(|p| p.id == card_id && p.frame >= close)
        .unwrap_or_else(|| panic!("PIX-002: the card sent a picture after the list closed"));
    let row = list_rows[0];
    let y = ((row - 1) as u32) * 16 + 8;
    let history: Vec<String> = pictures
        .iter()
        .filter(|p| p.id == card_id)
        .map(|p| format!("frame {} alpha {}", p.frame, p.pixel(8 + 4, y)[3]))
        .collect();
    // Column 1 is the card's padding: no text cuts it out of the picture.
    assert!(
        after_close.pixel(8 + 4, y)[3] > 0,
        "PIX-002: after the list closed (frame {close}; opened at {open}) the card's picture is whole again at row {row}: {history:?}"
    );
}

// ---------------------------------------------------------------------------
// Reference pictures (PIX-003 to PIX-005): a look's picture on a Kitty host
// against tests/snapshots/pixel-looks/<name>.png, within GFX-002's tolerance.

/// `tests/snapshots/pixel-looks`, or `pixel-looks` under
/// `REACTIVE_TUI_SNAPSHOTS`: the pixel-looks check points that at a copy with
/// one reference altered, and the test must then fail.
fn snapshots() -> std::path::PathBuf {
    std::env::var_os("REACTIVE_TUI_SNAPSHOTS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots")
        })
        .join("pixel-looks")
}

fn reference_path(name: &str) -> std::path::PathBuf {
    snapshots().join(format!("{name}.png"))
}

/// The process's graphics options while a reference picture is drawn: the
/// software renderer, which the references are drawn by on every host.
static SOFTWARE: Mutex<()> = Mutex::new(());

/// A reference case: the element, its box, the events before the picture
/// that counts, and which picture of the run that is (0 for the first).
struct Case {
    name: &'static str,
    element: Element,
    box_size: (u16, u16),
    script: Script,
    picture: usize,
}

fn case(name: &'static str, width: u16, height: u16, element: Element) -> Case {
    Case {
        name,
        element: sized(width, height, element),
        box_size: (width, height),
        script: silent(),
        picture: 0,
    }
}

/// The case with the pointer over cell (x, y) once the first picture is
/// sent; the second picture counts.
fn hovered(mut case: Case, x: u16, y: u16) -> Case {
    let sent = Mutex::new(false);
    case.script = Box::new(move |_, pictures| {
        let mut sent = sent.lock().unwrap();
        if !*sent && pictures >= 1 {
            *sent = true;
            return Some(mouse(MouseEventKind::Move, x, y));
        }
        None
    });
    case.picture = 1;
    case
}

/// The case with the focus given to its first control by a Tab once the
/// first picture is sent; the second picture counts.
fn focused(mut case: Case) -> Case {
    let sent = Mutex::new(false);
    case.script = Box::new(move |_, pictures| {
        let mut sent = sent.lock().unwrap();
        if !*sent && pictures >= 1 {
            *sent = true;
            return Some(key(KeyCode::Tab));
        }
        None
    });
    case.picture = 1;
    case
}

/// The case whose `picture`th picture counts, with no events.
fn nth(mut case: Case, picture: usize) -> Case {
    case.picture = picture;
    case
}

/// A case's picture as the backend sent it, with the path of its reference.
struct Captured {
    name: &'static str,
    frame: GraphicsFrame,
    path: std::path::PathBuf,
}

/// The picture a case sends on a Kitty host, drawn by the software renderer,
/// or why it sent none.
fn capture(case: Case) -> Result<Captured, String> {
    let _software = SOFTWARE.lock().unwrap_or_else(|e| e.into_inner());
    reactive_tui::widgets::display::charts::set_graphics_options(GraphicsOptions {
        force_cpu: true,
        ..Default::default()
    });
    let terminal = (case.box_size.0.max(40) + 2, case.box_size.1 + 2);
    let run = run(
        case.element,
        terminal,
        kitty(),
        case.script,
        pictures_or_frames(case.picture + 1, 2, 40),
        true,
    );
    reactive_tui::widgets::display::charts::set_graphics_options(GraphicsOptions::default());
    let name = case.name;
    let Some(picture) = run.pictures().into_iter().nth(case.picture) else {
        return Err(format!(
            "{name}: picture {} was not sent ({} pictures in {} frames)",
            case.picture + 1,
            run.pictures().len(),
            run.frames.len()
        ));
    };
    if picture.pixels.len() != (picture.size.0 * picture.size.1) as usize {
        return Err(format!("{name}: the picture's pixels do not fill its size"));
    }
    Ok(Captured {
        name,
        frame: picture.frame(),
        path: reference_path(name),
    })
}

/// The difference between a captured picture and its reference, or none.
fn compare(captured: &Captured) -> Option<String> {
    let name = captured.name;
    let reference = match image::open(&captured.path) {
        Ok(reference) => reference.to_rgba8(),
        Err(error) => {
            return Some(format!(
                "{name}: no reference picture {}: {error}",
                captured.path.display()
            ))
        }
    };
    let pixels = reference.pixels().map(|pixel| pixel.0).collect();
    let reference =
        GraphicsFrame::from_rgba(reference.width(), reference.height(), pixels).unwrap();
    canvas_support::difference(&captured.frame, &reference).map(|why| format!("{name}: {why}"))
}

/// Every case compared with its reference, as one failure naming each
/// picture that differs; with REGENERATE=1 the references are written
/// instead (BAR-004).
fn assert_references(requirement: &str, cases: Vec<Case>) {
    let mut problems = Vec::new();
    for case in cases {
        let captured = match capture(case) {
            Ok(captured) => captured,
            Err(problem) => {
                problems.push(problem);
                continue;
            }
        };
        if std::env::var("REGENERATE").as_deref() == Ok("1") {
            let frame = &captured.frame;
            let flat: Vec<u8> = frame.pixels().iter().flatten().copied().collect();
            let image = image::RgbaImage::from_raw(frame.width(), frame.height(), flat)
                .expect("a picture of its size");
            let mut bytes = Vec::new();
            image
                .write_to(&mut io::Cursor::new(&mut bytes), image::ImageFormat::Png)
                .expect("a PNG of the picture");
            std::fs::create_dir_all(captured.path.parent().unwrap())
                .expect("the pictures directory");
            std::fs::write(&captured.path, &bytes).expect("the reference picture written");
            continue;
        }
        problems.extend(compare(&captured));
    }
    assert!(
        problems.is_empty(),
        "{requirement}: {} reference pictures differ: {}",
        problems.len(),
        problems.join("; ")
    );
}

// ---------------------------------------------------------------------------
// PIX-003: rounded containers, buttons and cards.

fn rounded(classes: &str) -> Element {
    div().class(classes).child(builder::text("box")).build()
}

#[test]
fn pix_003_rounded_containers_buttons_and_cards_match_their_reference_pictures() {
    assert_references(
        "PIX-003",
        vec![
            case("button_default", 8, 1, cancel_button()),
            focused(case("button_focused", 8, 1, cancel_button())),
            hovered(case("button_hovered", 8, 1, cancel_button()), 3, 0),
            case(
                "button_disabled",
                8,
                1,
                builder::button().text("Cancel").disabled(true).build(),
            ),
            case("primary_default", 8, 1, save_button()),
            focused(case("primary_focused", 8, 1, save_button())),
            hovered(case("primary_hovered", 8, 1, save_button()), 3, 0),
            case(
                "primary_disabled",
                8,
                1,
                builder::primary_button("Save", || {}).disabled(true),
            ),
            case("card_dark", 40, 8, card()),
            case(
                "rounded_2xl_primary",
                20,
                4,
                rounded("w-20 h-4 bg-primary rounded-2xl"),
            ),
            case(
                "rounded_full_bordered",
                20,
                4,
                rounded("w-20 h-4 bg-surface border border-ring rounded-full"),
            ),
            case(
                "ring_rounded",
                20,
                4,
                rounded("w-20 h-4 bg-surface ring-2 ring-ring rounded"),
            ),
        ],
    );
}

/// The card under the light preset, in a process of its own since the theme
/// is the process's.
#[test]
#[ignore = "run by pix_003_the_card_under_the_light_preset_matches_its_reference_picture"]
fn card_light_child() {
    use reactive_tui::theme::{dark_theme, light_theme, Theme};
    Theme::set_active(light_theme());
    assert_references("PIX-003", vec![case("card_light", 40, 8, card())]);
    Theme::set_active(dark_theme());
}

#[test]
fn pix_003_the_card_under_the_light_preset_matches_its_reference_picture() {
    let mut command = std::process::Command::new(std::env::current_exe().expect("the test binary"));
    command.args(["--exact", "card_light_child", "--ignored", "--nocapture"]);
    for name in ["REGENERATE", "REACTIVE_TUI_SNAPSHOTS"] {
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
    let child = command.output().expect("the test binary runs");
    let said = String::from_utf8_lossy(&child.stdout).into_owned()
        + &String::from_utf8_lossy(&child.stderr);
    assert!(
        child.status.success() && said.contains("1 passed"),
        "PIX-003: {}",
        &said[said.find("PIX-003").unwrap_or(0)..]
    );
}

#[test]
fn pix_003_a_corner_cell_has_no_opaque_pixel_outside_the_arc() {
    let run = run(
        sized(20, 4, rounded("w-20 h-4 bg-primary rounded-2xl")),
        (40, 6),
        kitty(),
        silent(),
        pictures_or_frames(1, 2, 90),
        true,
    );
    let picture = run.first_picture("PIX-003: a 20 by 4 box with bg-primary rounded-2xl");
    assert_eq!(picture.size, (160, 64));
    assert_eq!(
        picture.pixel(0, 0)[3],
        0,
        "PIX-003: the corner pixel outside a 16-pixel arc is transparent"
    );
    assert_eq!(
        picture.pixel(159, 63)[3],
        0,
        "PIX-003: the opposite corner too"
    );
    assert!(
        picture.pixel(16, 16)[3] > 0,
        "PIX-003: the pixel at the arc's center is painted"
    );
    assert!(
        picture.pixel(80, 2)[3] > 0 && picture.pixel(2, 32)[3] > 0,
        "PIX-003: the straight edges are painted"
    );
}

#[test]
fn pix_003_the_card_helpers_name_theme_roles() {
    for (what, element) in [
        ("builder::card()", card()),
        (
            "builder::card_builder()",
            builder::card_builder().child(builder::text("box")).build(),
        ),
    ] {
        let class = element.class.clone().unwrap_or_default();
        for token in ["bg-surface", "border", "border-border", "rounded-lg"] {
            assert!(
                class.split_whitespace().any(|c| c == token),
                "PIX-003: {what} names `{token}`: {class:?}"
            );
        }
        assert!(
            !class
                .split_whitespace()
                .any(|c| c.starts_with("bg-white") || c.starts_with("border-gray")),
            "PIX-003: {what} names no palette literal: {class:?}"
        );
    }
    let run = run(
        sized(40, 8, card()),
        SIZE,
        no_pixels(),
        silent(),
        after_frames(4),
        true,
    );
    let screen = run.screen(run.frames.len() - 1);
    let background = bg(&screen, 1, 1).expect("the card's box has a background");
    assert!(
        near(
            [background[0], background[1], background[2], 255],
            role("surface"),
            2
        ),
        "PIX-003: without pixels a card's box is `surface`: {background:?}"
    );
}

/// The screen as text with every cell's colors, for the cell goldens.
fn cells_dump(screen: &vt100::Screen) -> String {
    let (rows, columns) = screen.size();
    let mut out = String::new();
    for row in 0..rows {
        for column in 0..columns {
            let cell = screen.cell(row, column).unwrap();
            let text = cell.contents();
            out.push_str(&format!(
                "{}|{:?}|{:?} ",
                if text.is_empty() { " " } else { &text },
                cell.fgcolor(),
                cell.bgcolor()
            ));
        }
        out.push('\n');
    }
    out
}

#[test]
fn pix_003_without_pixels_the_classes_keep_todays_cells() {
    let cases = [
        ("rounded_lg", "w-20 h-4 bg-primary rounded-lg"),
        ("border", "w-20 h-4 bg-primary border"),
        ("ring_2", "w-20 h-4 bg-primary ring-2 ring-ring"),
        (
            "rounded_full_border_ring",
            "w-20 h-4 bg-surface border border-ring rounded-full",
        ),
    ];
    let mut problems = Vec::new();
    for (name, classes) in cases {
        let run = run(
            sized(20, 4, rounded(classes)),
            (40, 6),
            no_pixels(),
            silent(),
            after_frames(3),
            true,
        );
        let dump = cells_dump(&run.screen(run.frames.len() - 1));
        let path = snapshots().join("cells").join(format!("{name}.cells"));
        if std::env::var("REGENERATE").as_deref() == Ok("1") {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, &dump).unwrap();
            continue;
        }
        match std::fs::read_to_string(&path) {
            Ok(golden) if golden == dump => {}
            Ok(_) => problems.push(format!("{name}: the cells differ from {}", path.display())),
            Err(error) => problems.push(format!("{name}: no golden {}: {error}", path.display())),
        }
    }
    assert!(problems.is_empty(), "PIX-003: {}", problems.join("; "));
}

// ---------------------------------------------------------------------------
// PIX-004: the text input, the checkbox and the radio.

#[test]
fn pix_004_fields_checkboxes_and_radios_match_their_reference_pictures() {
    assert_references(
        "PIX-004",
        vec![
            case("input_placeholder", 30, 1, text_input("", "Search")),
            focused(case(
                "input_focused_text",
                30,
                1,
                text_input("Reactive", ""),
            )),
            case(
                "input_invalid",
                30,
                1,
                Element::typed::<TextInput>(TextInputProps {
                    value: "ab".into(),
                    validator_pattern: Some("^[a-z]{3,}$".into()),
                    error_message: Some("Three letters at least".into()),
                    ..Default::default()
                }),
            ),
            case("input_multiline", 30, 4, multiline_input()),
            case("checkbox_unchecked", 30, 1, checkbox("Capture", false)),
            case("checkbox_checked", 30, 1, checkbox("Capture", true)),
            case(
                "checkbox_mixed",
                30,
                1,
                builder::checkbox()
                    .label("Capture")
                    .indeterminate(true)
                    .build(),
            ),
            focused(case(
                "checkbox_checked_focused",
                30,
                1,
                checkbox("Capture", true),
            )),
            case(
                "checkbox_disabled",
                30,
                1,
                builder::checkbox()
                    .label("Capture")
                    .checked(true)
                    .disabled(true)
                    .build(),
            ),
            case("radios_second_chosen", 30, 3, radios(1)),
            focused(case("radios_focus_first", 30, 3, radios(1))),
        ],
    );
}

#[test]
fn pix_004_no_frame_glyph_remains_in_a_field_or_box_cell() {
    let tree = div()
        .class("flex-col w-40")
        .child(sized(30, 1, text_input("Reactive", "")))
        .child(sized(30, 1, checkbox("Capture", true)))
        .build();
    let run = run(
        tree,
        SIZE,
        kitty(),
        silent(),
        pictures_or_frames(2, 2, 90),
        true,
    );
    let screen = run.screen(run.frames.len() - 1);
    assert!(
        columns_of(&screen, 0, "Reactive").is_some() && columns_of(&screen, 1, "Capture").is_some(),
        "the text stays cell text: {:?}",
        screen.contents()
    );
    if let Some(found) = glyph_in(&screen, 0..2, &['[', ']', '✓', '▬']) {
        panic!("PIX-004: a frame glyph of the cell look remains with pixels: {found}");
    }
}

#[test]
fn pix_004_a_focused_control_has_its_ring_and_the_cursor_stays_cell_text() {
    let sent = Mutex::new(false);
    let script: Script = Box::new(move |_, pictures| {
        let mut sent = sent.lock().unwrap();
        if !*sent && pictures >= 1 {
            *sent = true;
            return Some(key(KeyCode::Tab));
        }
        None
    });
    let run = run(
        sized(30, 1, text_input("Reactive", "")),
        SIZE,
        kitty(),
        script,
        pictures_or_frames(2, 2, 90),
        true,
    );
    run.first_picture("PIX-004: a text input");
    let picture = run
        .pictures()
        .into_iter()
        .last()
        .expect("PIX-004: the focused text input's picture");
    let ring = role("ring");
    assert!(
        picture.pixels.iter().any(|p| near(*p, ring, 2)),
        "PIX-004: the focused field's picture holds the `ring` color"
    );
    let screen = run.screen(picture.frame);
    let text = columns_of(&screen, 0, "Reactive").expect("the text is cell text");
    // The cursor stands on the first glyph when the field gains the focus:
    // its cell is the field reversed, wherever it is.
    let reversed = |column: usize| {
        bg(&screen, 0, column).is_some_and(|c| near([c[0], c[1], c[2], 255], role("foreground"), 2))
            && fg(&screen, 0, column)
                .is_some_and(|c| near([c[0], c[1], c[2], 255], role("input"), 2))
    };
    let cursor = text.start;
    assert!(
        (0..30).any(reversed),
        "PIX-004: the cursor cell is cell text in `foreground` with `input` text: bg {:?} fg {:?}",
        bg(&screen, 0, cursor),
        fg(&screen, 0, cursor)
    );
}

// ---------------------------------------------------------------------------
// PIX-005: the slider and the progress bar.

#[test]
fn pix_005_sliders_and_progress_bars_match_their_reference_pictures() {
    assert_references(
        "PIX-005",
        vec![
            case("slider_0", 40, 1, slider(0.0)),
            case("slider_37", 40, 1, slider(37.0)),
            case("slider_100", 40, 1, slider(100.0)),
            focused(case("slider_37_focused", 40, 1, slider(37.0))),
            case(
                "slider_disabled",
                40,
                1,
                builder::slider()
                    .value(37.0)
                    .min(0.0)
                    .max(100.0)
                    .disabled(true)
                    .build(),
            ),
            case("progress_0", 40, 1, progress(0.0)),
            case("progress_37", 40, 1, progress(37.0)),
            case("progress_100", 40, 1, progress(100.0)),
            case(
                "progress_2rows_37",
                40,
                2,
                Element::typed::<ProgressBar>(ProgressBarProps {
                    value: 37.0,
                    max_value: 100.0,
                    show_percentage: false,
                    show_value: false,
                    height: 2,
                    ..Default::default()
                }),
            ),
            nth(
                case(
                    "progress_indeterminate_first",
                    40,
                    1,
                    indeterminate_progress(false),
                ),
                0,
            ),
            nth(
                case(
                    "progress_indeterminate_tenth",
                    40,
                    1,
                    indeterminate_progress(false),
                ),
                9,
            ),
        ],
    );
}

/// The columns of the picture whose pixels are near `color`, with the
/// pixels' count per column.
fn columns_near(picture: &Picture, color: [u8; 3]) -> Vec<(u32, usize)> {
    (0..picture.size.0)
        .map(|x| {
            let count = (0..picture.size.1)
                .filter(|y| near(picture.pixel(x, *y), color, 3))
                .count();
            (x, count)
        })
        .filter(|(_, count)| *count > 0)
        .collect()
}

#[test]
fn pix_005_the_thumb_and_the_fill_end_at_the_exact_value_pixel() {
    let shown = run(
        sized(40, 1, slider(37.0)),
        (44, 3),
        kitty(),
        silent(),
        pictures_or_frames(1, 2, 90),
        true,
    );
    let picture = shown.first_picture("PIX-005: a slider at 37 of 100");
    let thumb = columns_near(&picture, role("foreground"));
    assert!(
        !thumb.is_empty(),
        "PIX-005: the slider's picture holds its thumb in `foreground`"
    );
    let center = (thumb.first().unwrap().0 + thumb.last().unwrap().0) as f64 / 2.0;
    let track = columns_near(&picture, role("border"))
        .into_iter()
        .chain(columns_near(&picture, role("primary")))
        .map(|(x, _)| x)
        .collect::<Vec<_>>();
    let (left, right) = (
        *track.iter().min().unwrap() as f64,
        *track.iter().max().unwrap() as f64,
    );
    let expected = left + (right - left) * 0.37;
    assert!(
        (center - expected).abs() <= 2.0,
        "PIX-005: the thumb's center is at {center} while 37 percent of the track ({left}..{right}) is {expected}"
    );
    let bar = run(
        sized(40, 1, progress(37.0)),
        (44, 3),
        kitty(),
        silent(),
        pictures_or_frames(1, 2, 90),
        true,
    );
    let picture = bar.first_picture("PIX-005: a progress bar at 37 percent");
    let fill = columns_near(&picture, role("primary"));
    let edge = fill.last().map(|(x, _)| *x).unwrap_or(0);
    let expected = 320.0 * 0.37;
    assert!(
        (edge as f64 - expected).abs() <= 2.0,
        "PIX-005: the fill ends at pixel {edge} while 37 percent of 320 is {expected}"
    );
    assert!(
        !edge.is_multiple_of(8) && edge % 8 != 7,
        "PIX-005: the fill's edge {edge} is not on a cell boundary"
    );
}

#[test]
fn pix_005_an_indeterminate_bar_sends_a_picture_every_frame_and_steps_under_reduced_motion() {
    let gliding = run(
        sized(40, 1, indeterminate_progress(false)),
        (44, 3),
        kitty(),
        silent(),
        after_frames(20),
        true,
    );
    let frames_with_pictures: std::collections::BTreeSet<usize> =
        gliding.pictures().iter().map(|p| p.frame).collect();
    assert!(
        frames_with_pictures.len() >= 12,
        "PIX-005: an indeterminate bar sends a new picture every frame while it glides: {} frames of 20 had one",
        frames_with_pictures.len()
    );
    let stepping = run(
        sized(40, 1, indeterminate_progress(true)),
        (44, 3),
        kitty(),
        silent(),
        after_frames(20),
        true,
    );
    let span = stepping.span();
    let pictures = stepping.pictures().len();
    assert!(
        pictures >= 1 && (pictures as f64) <= 2.0 + span.as_secs_f64().ceil(),
        "PIX-005: under reduced-motion the bar steps once a second: {pictures} pictures in {span:?}"
    );
}

#[test]
fn pix_005_no_track_glyph_remains() {
    let tree = div()
        .class("flex-col w-44")
        .child(sized(40, 1, slider(37.0)))
        .child(sized(40, 1, progress(37.0)))
        .build();
    let run = run(
        tree,
        (48, 4),
        kitty(),
        silent(),
        pictures_or_frames(2, 2, 90),
        true,
    );
    assert!(
        run.pictures().len() >= 2,
        "PIX-005: the slider and the progress bar sent their pictures, got {}",
        run.pictures().len()
    );
    let screen = run.screen(run.frames.len() - 1);
    if let Some(found) = glyph_in(&screen, 0..2, &['═', '─', '●', '█', '░', '▒', '▌'])
    {
        panic!("PIX-005: a track glyph of the cell look remains with pixels: {found}");
    }
}

// ---------------------------------------------------------------------------
// PIX-006: speed and scale, in release on the Linux host; recorded elsewhere.

/// The catalog's Input page: its six cards at 240 by 60.
fn input_page() -> Element {
    let card_of = |title: &str, body: Element| builder::card(vec![builder::text(title), body]);
    div()
        .class("grid grid-cols-3 gap-1 w-full h-full p-1")
        .child(card_of(
            "TextInput",
            text_input("", "Type here").auto_focus(),
        ))
        .child(card_of("Checkbox", checkbox("Capture-ready", true)))
        .child(card_of("RadioButton", radios(0)))
        .child(card_of(
            "Select",
            builder::select()
                .option("a", "Balanced")
                .option("b", "High")
                .selected("a")
                .build(),
        ))
        .child(card_of(
            "Slider",
            builder::slider().label("Intensity").value(50.0).build(),
        ))
        .child(card_of(
            "Buttons",
            div()
                .class("flex flex-row gap-1")
                .child(save_button())
                .child(cancel_button())
                .build(),
        ))
        .build()
}

/// A page of 48 cards in 8 by 6, each with a text input, a checkbox and a button.
fn many_looks_page() -> Element {
    let mut grid = div().class("grid grid-cols-8 gap-1 w-full h-full p-1");
    for index in 0..48 {
        let input = if index == 0 {
            text_input("", "Type").auto_focus()
        } else {
            text_input("", "Type")
        };
        grid = grid.child(builder::card(vec![
            input,
            checkbox("Ready", index % 2 == 0),
            cancel_button(),
        ]));
    }
    grid.build()
}

fn percentile(values: &mut [Duration], p: f64) -> Duration {
    values.sort();
    let at = ((values.len() as f64 - 1.0) * p).round() as usize;
    values.get(at).copied().unwrap_or_default()
}

/// Where the slider's thumb of `page` is, as a cell for the pointer: the
/// center of the slider's picture, the one-row picture on the row of its
/// label, found in a short run of the page at 240 by 60.
fn thumb_of(page: Element, output: &str, images: ImageOutputOptions) -> (u16, u16) {
    let run = run(
        page,
        (240, 60),
        images,
        silent(),
        pictures_or_frames(14, 2, 120),
        true,
    );
    let screen = run.screen(run.frames.len() - 1);
    let row = (0..60)
        .find(|row| columns_of(&screen, *row, "Intensity").is_some())
        .expect("the slider's row");
    let boxes: Vec<(usize, usize, usize, usize)> = if output == "sixel" {
        run.rasters()
            .iter()
            .filter_map(|r| r.at.map(|(y, x)| (y, x, r.size.1 / 16, r.size.0 / 8)))
            .collect()
    } else {
        run.pictures().iter().map(|p| p.rect()).collect()
    };
    let (y, x, _, columns) = boxes
        .into_iter()
        .find(|(y, _, rows, _)| *y == row && *rows == 1)
        .expect("the slider's picture on its label's row");
    ((x + columns / 2) as u16, y as u16)
}

/// One measured run: the pictures' arrival and 60 frames of typing, then,
/// when `slider_at` names the slider's thumb, 60 frames of dragging.
fn measure(
    page: &str,
    output: &str,
    element: Element,
    expected_looks: usize,
    slider_at: Option<(u16, u16)>,
    images: ImageOutputOptions,
) -> Vec<String> {
    let mut events: Vec<(usize, Event)> = Vec::new();
    let typing_from = 70;
    for i in 0..60 {
        events.push((typing_from + i, key(KeyCode::Char('a'))));
    }
    let drag_from = typing_from + 60 + 5;
    // The thumb is dragged one cell a frame, fifteen cells to the right and
    // back, so the pointer stays on the slider's track and no other look
    // changes under it.
    let drag_offset = |i: usize| {
        let step = i % 30;
        (if step <= 15 { step } else { 30 - step }) as u16
    };
    if let Some((x, y)) = slider_at {
        // The press takes the focus from the text input, whose look then
        // changes once; it comes before the measured drag frames.
        events.push((drag_from - 4, pressed(MouseEventKind::Down, x, y)));
        for i in 1..=60 {
            events.push((
                drag_from + i,
                pressed(MouseEventKind::Drag, x + drag_offset(i), y),
            ));
        }
        events.push((
            drag_from + 61,
            pressed(MouseEventKind::Up, x + drag_offset(60), y),
        ));
    }
    let last = drag_from + 64;
    let run = run(
        element,
        (240, 60),
        images,
        at_frames(events),
        after_frames(last),
        false,
    );
    let mut problems = Vec::new();
    let first = run.frames.first().map(|f| f.began);
    // Every picture sent, as (frame, which look): a Kitty picture by its
    // image id, a Sixel raster by the cells it covers.
    let pictures: Vec<(usize, String)> = if output == "sixel" {
        run.rasters()
            .iter()
            .map(|r| (r.frame, format!("{:?} {:?}", r.at, r.size)))
            .collect()
    } else {
        run.pictures()
            .iter()
            .map(|p| (p.frame, format!("{:?} at {:?}", p.id, p.rect())))
            .collect()
    };
    let ids: std::collections::BTreeSet<&String> = pictures.iter().map(|(_, id)| id).collect();
    let within = pictures
        .iter()
        .filter(|(frame, _)| {
            first.is_some_and(|f| {
                run.frames[*frame].began.duration_since(f) <= Duration::from_secs(1)
            })
        })
        .map(|(_, id)| id)
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    if within < expected_looks {
        problems.push(format!(
            "{page} ({output}): {within} of {expected_looks} looks sent a picture within a second ({} ids in all, {} pictures)",
            ids.len(),
            pictures.len()
        ));
    }
    let typed: Vec<&Frame> = run.frames.iter().skip(typing_from).take(60).collect();
    let mut work: Vec<Duration> = typed.iter().map(|f| f.work).collect();
    let mut waited: Vec<Duration> = typed.iter().map(|f| f.waited).collect();
    let (work95, wait95) = (percentile(&mut work, 0.95), percentile(&mut waited, 0.95));
    let typing_ids: std::collections::BTreeSet<&String> = pictures
        .iter()
        .filter(|(frame, _)| *frame >= typing_from && *frame < typing_from + 60)
        .map(|(_, id)| id)
        .collect();
    println!(
        "PIX-006 {page} {output}: {within}/{expected_looks} looks within 1 s; typing p95 work {:.2} ms, p95 wait {:.2} ms, {} looks repainted",
        work95.as_secs_f64() * 1000.0,
        wait95.as_secs_f64() * 1000.0,
        typing_ids.len()
    );
    if cfg!(target_os = "linux") {
        if work95 > BOUND || wait95 > BOUND {
            problems.push(format!(
                "{page} ({output}): typing p95 work {work95:?}, wait {wait95:?} over the frame"
            ));
        }
        if typing_ids.len() > 1 {
            problems.push(format!(
                "{page} ({output}): typing repainted {} looks, not the text input alone: {typing_ids:?}",
                typing_ids.len()
            ));
        }
    }
    if slider_at.is_some() {
        let dragged: Vec<&Frame> = run.frames.iter().skip(drag_from).take(60).collect();
        let mut work: Vec<Duration> = dragged.iter().map(|f| f.work).collect();
        let mut waited: Vec<Duration> = dragged.iter().map(|f| f.waited).collect();
        let (work95, wait95) = (percentile(&mut work, 0.95), percentile(&mut waited, 0.95));
        let drag_ids: std::collections::BTreeSet<&String> = pictures
            .iter()
            .filter(|(frame, _)| *frame > drag_from && *frame <= drag_from + 60)
            .map(|(_, id)| id)
            .collect();
        println!(
            "PIX-006 {page} {output}: dragging p95 work {:.2} ms, p95 wait {:.2} ms, {} looks repainted",
            work95.as_secs_f64() * 1000.0,
            wait95.as_secs_f64() * 1000.0,
            drag_ids.len()
        );
        if drag_ids.is_empty() {
            problems.push(format!(
                "{page} ({output}): the drag repainted no look: the pointer missed the slider's thumb"
            ));
        }
        if cfg!(target_os = "linux") {
            if work95 > BOUND || wait95 > BOUND {
                problems.push(format!("{page} ({output}): dragging p95 work {work95:?}, wait {wait95:?} over the frame"));
            }
            if drag_ids.len() > 1 {
                problems.push(format!(
                    "{page} ({output}): dragging repainted {} looks, not the slider alone: {drag_ids:?}",
                    drag_ids.len()
                ));
            }
        }
    }
    problems
}

#[test]
#[ignore = "release build on the hardware adapter; run by scripts/cairn/pixel_looks.py"]
fn pix_006_the_input_page_and_a_page_of_192_looks_show_within_a_second_and_stay_within_a_frame() {
    if cfg!(debug_assertions) {
        panic!("PIX-006 is measured on the optimized build");
    }
    let shared = ImageOutputOptions {
        kitty_shared_memory: cfg!(unix),
        ..kitty()
    };
    let mut problems = Vec::new();
    for (output, images) in [("kitty", shared), ("sixel", sixel())] {
        // The Input page: six cards, a text input, a checkbox, three radios,
        // a slider and two buttons: 14 looks (the select has none). The
        // slider's thumb at 50 of 100 sits mid-track in its card.
        let thumb = thumb_of(input_page(), output, images);
        problems.extend(measure(
            "input-page",
            output,
            input_page(),
            14,
            Some(thumb),
            images,
        ));
        problems.extend(measure(
            "192-looks",
            output,
            many_looks_page(),
            192,
            None,
            images,
        ));
    }
    assert!(problems.is_empty(), "PIX-006: {}", problems.join("; "));
}
