//! THM-001 to THM-003 (docs/spec/theme.md): the theme's color roles, the
//! contrast of text with its fill, what a theme that lacks a role gets, and
//! a change of theme.
//!
//! Every test that reads or changes the active theme takes its turn
//! (`serial(theme)`), because the active theme is one for the process.

mod common;

use common::app_input;
use reactive_tui::{
    app::RootComponent,
    component::Element,
    components::{div, text},
    event::types::{Event, KeyCode, KeyEvent},
    events::EventResult,
    theme::{
        dark_theme, gruvbox_dark_theme, high_contrast_theme, light_theme, solarized_dark_theme,
        Theme, ThemeVariables,
    },
    ui::paint::extract_paint_style,
    widgets::menu::{MenuBar, MenuBarProps, MenuItem},
};
use std::sync::Arc;

type Rgba = (f32, f32, f32, f32);

/// The roles that are neither a fill with text of its own nor such a text.
const PLAIN: [&str; 10] = [
    "background",
    "surface",
    "foreground",
    "text-muted",
    "border",
    "input",
    "ring",
    "hover",
    "overlay",
    "shadow",
];

/// The fills; each has a text role named by the fill with `-foreground`.
const FILLS: [&str; 8] = [
    "primary",
    "secondary",
    "accent",
    "success",
    "warning",
    "error",
    "info",
    "selection",
];

fn text_role(fill: &str) -> String {
    format!("{fill}-foreground")
}

/// Every role of THM-001.
fn roles() -> Vec<String> {
    PLAIN
        .iter()
        .chain(&FILLS)
        .map(|role| (*role).to_owned())
        .chain(FILLS.iter().map(|fill| text_role(fill)))
        .collect()
}

fn presets() -> Vec<Theme> {
    vec![
        dark_theme(),
        light_theme(),
        high_contrast_theme(),
        solarized_dark_theme(),
        gruvbox_dark_theme(),
    ]
}

/// Relative luminance as WCAG 2.1 defines it.
fn luminance(color: Rgba) -> f32 {
    let channel = |value: f32| {
        if value <= 0.03928 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(color.0) + 0.7152 * channel(color.1) + 0.0722 * channel(color.2)
}

/// Contrast ratio as WCAG 2.1 defines it, from 1 to 21.
fn contrast(one: Rgba, other: Rgba) -> f32 {
    let (one, other) = (luminance(one), luminance(other));
    (one.max(other) + 0.05) / (one.min(other) + 0.05)
}

fn bytes(color: Rgba) -> [u8; 3] {
    let byte = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
    [byte(color.0), byte(color.1), byte(color.2)]
}

const BLACK: [u8; 3] = [0, 0, 0];
const WHITE: [u8; 3] = [255, 255, 255];

/// The pairs THM-001 names: a text, what it is drawn on, and the least
/// contrast between them.
fn pairs() -> Vec<(String, String, f32)> {
    let mut pairs: Vec<(String, String, f32)> = FILLS
        .iter()
        .map(|fill| (text_role(fill), (*fill).to_owned(), 4.5))
        .collect();
    for ground in ["background", "surface", "input", "hover"] {
        pairs.push(("foreground".into(), ground.into(), 4.5));
    }
    for ground in ["background", "surface"] {
        pairs.push(("text-muted".into(), ground.into(), 4.5));
        pairs.push(("ring".into(), ground.into(), 3.0));
    }
    pairs
}

#[test]
#[serial_test::serial(theme)]
fn thm_001_every_preset_defines_every_role() {
    let mut missing = Vec::new();
    for preset in presets() {
        for role in roles() {
            let defined = preset.get_variable(&Theme::color_variable(&role)).is_some();
            if !defined || preset.resolve_variable(&role).is_none() {
                missing.push(format!("{}: {role}", preset.name));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "THM-001: {} roles are not defined: {missing:?}",
        missing.len()
    );
}

#[test]
#[serial_test::serial(theme)]
fn thm_001_text_contrasts_with_what_it_is_drawn_on_in_every_preset() {
    let mut low = Vec::new();
    for preset in presets() {
        for (text, ground, least) in pairs() {
            match (
                preset.resolve_variable(&text),
                preset.resolve_variable(&ground),
            ) {
                (Some(text_color), Some(ground_color)) => {
                    let ratio = contrast(text_color, ground_color);
                    if ratio < least {
                        low.push(format!(
                            "{}: {text} on {ground} is {ratio:.2} to 1, under {least}",
                            preset.name
                        ));
                    }
                }
                _ => low.push(format!("{}: {text} or {ground} is not defined", preset.name)),
            }
        }
    }
    assert!(low.is_empty(), "THM-001: {low:#?}");
}

/// A theme as an application wrote it before the roles of THM-001 existed:
/// it names three colors and extends nothing.
fn sparse(background: &str, foreground: &str) -> Theme {
    Theme::new("sparse").with_variables(
        ThemeVariables::new()
            .set("--color-background", background)
            .set("--color-foreground", foreground)
            .set("--color-primary", "#c81e28"),
    )
}

const SPARSE_PRIMARY: [u8; 3] = [200, 30, 40];

#[test]
#[serial_test::serial(theme)]
fn thm_002_a_class_that_names_a_role_always_sets_its_color() {
    let theme = sparse("#000000", "#e6e6e6");
    let mut unset = Vec::new();
    for role in roles() {
        let expected = theme.resolve_color(&role).map(bytes);
        for prefix in ["bg", "text"] {
            let class = format!("{prefix}-{role}");
            let painted = extract_paint_style(&mut theme.apply_classes(&class)).map(|style| {
                let color = if prefix == "bg" { style.bg } else { style.fg };
                bytes((color.r, color.g, color.b, color.a))
            });
            if painted.is_none() || painted != expected {
                unset.push(format!("{class}: {painted:?}, the role is {expected:?}"));
            }
        }
    }
    assert!(
        unset.is_empty(),
        "THM-002: under a theme of three colors these classes do not set their role's color: {unset:#?}"
    );
}

#[test]
#[serial_test::serial(theme)]
fn thm_002_text_on_a_fill_is_black_or_white_whichever_contrasts_more() {
    let mut wrong = Vec::new();
    for theme in [
        sparse("#000000", "#e6e6e6"),
        sparse("#ffffff", "#1e1e1e"),
    ] {
        for fill in FILLS {
            let role = text_role(fill);
            let (Some(text), Some(ground)) = (theme.resolve_color(&role), theme.resolve_color(fill))
            else {
                wrong.push(format!("{role} or {fill} does not resolve"));
                continue;
            };
            let expected = if contrast((0.0, 0.0, 0.0, 1.0), ground)
                > contrast((1.0, 1.0, 1.0, 1.0), ground)
            {
                BLACK
            } else {
                WHITE
            };
            if bytes(text) != expected || contrast(text, ground) < 4.5 {
                wrong.push(format!(
                    "{role} is {:?} on {:?}, expected {expected:?} ({:.2} to 1)",
                    bytes(text),
                    bytes(ground),
                    contrast(text, ground)
                ));
            }
        }
    }
    assert!(wrong.is_empty(), "THM-002: {wrong:#?}");
}

#[test]
#[serial_test::serial(theme)]
fn thm_002_selection_ring_input_and_hover_come_from_the_theme_s_own_colors() {
    let theme = sparse("#000000", "#e6e6e6");
    let resolved = |role: &str| theme.resolve_color(role).map(bytes);
    assert_eq!(
        resolved("selection"),
        Some(SPARSE_PRIMARY),
        "THM-002: selection"
    );
    assert_eq!(resolved("ring"), Some(SPARSE_PRIMARY), "THM-002: ring");
    let surface = theme.resolve_color("surface");
    assert!(surface.is_some(), "THM-002: surface does not resolve");
    assert_eq!(resolved("input"), surface.map(bytes), "THM-002: input");
    let (surface, foreground) = (surface.unwrap(), (0.902, 0.902, 0.902, 1.0));
    let mixed = |ground: f32, text: f32| (7.0 * ground + text) / 8.0;
    let expected = bytes((
        mixed(surface.0, foreground.0),
        mixed(surface.1, foreground.1),
        mixed(surface.2, foreground.2),
        1.0,
    ));
    let hover = resolved("hover").unwrap_or([0; 3]);
    assert!(
        theme.resolve_color("hover").is_some()
            && hover
                .iter()
                .zip(expected)
                .all(|(got, want)| got.abs_diff(want) <= 1),
        "THM-002: hover is {hover:?}, seven parts of the surface and one of the foreground are {expected:?}"
    );
}

#[test]
#[serial_test::serial(theme)]
fn thm_002_other_roles_come_from_the_preset_nearest_to_the_background() {
    let others = [
        "surface",
        "text-muted",
        "border",
        "secondary",
        "accent",
        "success",
        "warning",
        "error",
        "info",
        "overlay",
        "shadow",
    ];
    let mut wrong = Vec::new();
    let no_background =
        Theme::new("bare").with_variables(ThemeVariables::new().set("--color-primary", "#c81e28"));
    for (theme, preset) in [
        (sparse("#000000", "#e6e6e6"), dark_theme()),
        (sparse("#ffffff", "#1e1e1e"), light_theme()),
        (no_background, dark_theme()),
    ] {
        for role in others {
            let (got, expected) = (theme.resolve_color(role), preset.resolve_variable(role));
            if got.is_none() || got != expected {
                wrong.push(format!(
                    "{} with the {} preset: {role} is {got:?}, the preset's is {expected:?}",
                    theme.name, preset.name
                ));
            }
        }
    }
    assert!(wrong.is_empty(), "THM-002: {wrong:#?}");
}

/// A theme whose roles all differ from one another and from every role of
/// the probe with another `number`: role `i` of probe 0 is a red, of probe 1
/// a green.
fn probe(number: usize) -> Theme {
    let mut variables = ThemeVariables::new();
    for (index, role) in roles().iter().enumerate() {
        let level = 60 + 6 * index;
        let hex = if number == 0 {
            format!("#{level:02x}1414")
        } else {
            format!("#14{level:02x}14")
        };
        variables = variables.set(Theme::color_variable(role), hex);
    }
    Theme::new(format!("probe-{number}")).with_variables(variables)
}

/// Whether `color` is one of the colors `probe(number)` defines.
fn of_probe(number: usize, color: vt100::Color) -> bool {
    let vt100::Color::Rgb(r, g, b) = color else {
        return false;
    };
    let (level, low) = if number == 0 { (r, g) } else { (g, r) };
    low == 0x14 && b == 0x14 && level >= 60 && (usize::from(level) - 60) % 6 == 0
}

/// Elements whose classes name roles, and a menu bar with its default
/// style; a key gives the application the second probe theme.
struct Themed;
impl RootComponent for Themed {
    fn render(&self) -> Element {
        let line = |classes: &str, label: &str| {
            div()
                .class(format!("{classes} w-full h-1"))
                .child(text(label).build())
                .build()
        };
        div()
            .class("flex flex-col w-full h-full bg-background text-foreground")
            .child(line("bg-primary text-primary-foreground", "PRIMARY"))
            .child(line("bg-surface text-text-muted", "SURFACE"))
            .child(line("bg-selection text-selection-foreground", "SELECTION"))
            .child(Element::typed::<MenuBar>(MenuBarProps {
                items: vec![MenuItem::new("file", "FILE"), MenuItem::new("edit", "EDIT")],
                ..Default::default()
            }))
            .build()
    }
    fn try_handle_event(&mut self, event: &Event) -> reactive_tui::error::Result<EventResult> {
        if matches!(event, Event::Key(_)) {
            Theme::set_active(probe(1));
            return Ok(EventResult::Handled);
        }
        Ok(EventResult::Ignored)
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

/// How many cells of `frame` have a glyph or a background color of the
/// probe theme `number`.
fn cells_of_probe(frame: &app_input::Snapshot, number: usize) -> usize {
    let (rows, columns) = frame.screen.size();
    (0..rows)
        .flat_map(|row| (0..columns).map(move |column| (row, column)))
        .filter_map(|(row, column)| frame.screen.cell(row, column))
        .filter(|cell| of_probe(number, cell.bgcolor()) || of_probe(number, cell.fgcolor()))
        .count()
}

#[test]
#[serial_test::serial(theme)]
fn thm_003_the_frame_after_a_change_of_theme_holds_no_color_of_the_old_theme() {
    let before: Arc<Theme> = Theme::active();
    Theme::set_active(probe(0));
    let frames = app_input::run(
        Themed,
        (60, 12),
        vec![
            (1, Some(Event::Key(KeyEvent::new(KeyCode::Char('t'))))),
            (2, None),
        ],
    );
    Theme::set_active((*before).clone());
    let (first, next) = (&frames[0], &frames[1]);
    assert!(
        cells_of_probe(first, 0) >= 60 && cells_of_probe(first, 1) == 0,
        "THM-003: the first frame is not painted in the first theme's colors: {} cells of it, {} of the second",
        cells_of_probe(first, 0),
        cells_of_probe(first, 1)
    );
    assert_eq!(
        (cells_of_probe(next, 0), cells_of_probe(next, 1) >= 60),
        (0, true),
        "THM-003: the frame after the change holds {} cells in the old theme's colors and {} in the new theme's",
        cells_of_probe(next, 0),
        cells_of_probe(next, 1)
    );
}
