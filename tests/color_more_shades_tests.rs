use reactive_tui::layout::css::apply_utility_classes;
use reactive_tui::layout::style::StyleBuilder;
use reactive_tui::ui::paint::extract_paint_style;

#[test]
fn yellow_purple_orange_cyan_shades_parse() {
    for token in [
        "text-yellow-400",
        "text-yellow-600",
        "text-purple-400",
        "text-purple-600",
        "text-orange-400",
        "text-orange-600",
        "text-cyan-400",
        "text-cyan-600",
    ] {
        let mut sb = apply_utility_classes(token, StyleBuilder::new());
        let p = extract_paint_style(&mut sb).expect("paint style");
        assert!(p.fg.a >= 0.99);
    }
}
