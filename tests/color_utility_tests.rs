use reactive_tui::layout::css::apply_utility_classes;
use reactive_tui::layout::style::StyleBuilder;
use reactive_tui::ui::paint::extract_paint_style;

#[test]
fn text_color_named_and_hex() {
    let mut sb1 = apply_utility_classes("text-red-500", StyleBuilder::new());
    let mut sb2 = apply_utility_classes("text-#00ff00", StyleBuilder::new());
    let p1 = extract_paint_style(&mut sb1).expect("p1");
    let p2 = extract_paint_style(&mut sb2).expect("p2");
    assert!(p1.fg.r > p1.fg.g && p1.fg.r > p1.fg.b);
    assert!(p2.fg.g > 0.9 && p2.fg.r < 0.1 && p2.fg.b < 0.1);
}

#[test]
fn bg_color_named() {
    let mut sb = apply_utility_classes("bg-blue-500", StyleBuilder::new());
    let p = extract_paint_style(&mut sb).expect("p");
    assert!(p.bg.b > p.bg.r && p.bg.b > p.bg.g);
}

/// Switching the active theme changes the color of a theme class at once,
/// both where a class is applied directly and on a DebugBackend that painted
/// it before the switch: a class resolved under the old theme is not reused
/// on any thread. No other test in this file reads theme colors, so the
/// switch cannot change what they see.
#[test]
fn a_theme_switch_recolors_theme_classes_that_were_already_resolved() {
    use reactive_tui::{
        backend::{Backend, DebugBackend},
        builder::core::div,
        theme::{Theme, ThemeVariables},
    };
    let primary = |hex: &str| {
        Theme::new("switch").with_variables(ThemeVariables::new().set("--color-primary", hex))
    };
    let element = div().class("text-primary w-4 h-1").text("x").build();
    let mut backend = DebugBackend::new(4, 1);
    let painted = |backend: &mut DebugBackend| {
        backend.render_frame(&element).unwrap();
        backend.present().unwrap();
        let fg = backend.cell(0, 0).unwrap().fg;
        (fg.r, fg.g, fg.b)
    };
    let applied = || {
        let mut sb = apply_utility_classes("text-primary", StyleBuilder::new());
        let fg = extract_paint_style(&mut sb)
            .expect("text-primary sets a color")
            .fg;
        (fg.r, fg.g, fg.b)
    };

    let previous = Theme::active();
    Theme::set_active(primary("#ff0000"));
    let (red_applied, red_painted) = (applied(), painted(&mut backend));
    Theme::set_active(primary("#0000ff"));
    let (blue_applied, blue_painted) = (applied(), painted(&mut backend));
    Theme::set_active((*previous).clone());

    assert_eq!(red_applied, (1.0, 0.0, 0.0));
    assert_eq!(red_painted, (1.0, 0.0, 0.0));
    assert_eq!(
        blue_applied,
        (0.0, 0.0, 1.0),
        "the class kept the old theme's color"
    );
    assert_eq!(
        blue_painted,
        (0.0, 0.0, 1.0),
        "DebugBackend kept the old theme's color"
    );
}
