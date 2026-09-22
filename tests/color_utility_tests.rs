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
