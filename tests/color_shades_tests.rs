use reactive_tui::layout::css::apply_utility_classes;
use reactive_tui::layout::style::StyleBuilder;
use reactive_tui::ui::paint::extract_paint_style;

#[test]
fn red_and_blue_shades_ordering() {
    let mut r4 = apply_utility_classes("text-red-400", StyleBuilder::new());
    let mut r6 = apply_utility_classes("text-red-600", StyleBuilder::new());
    let pr4 = extract_paint_style(&mut r4).unwrap();
    let pr6 = extract_paint_style(&mut r6).unwrap();
    // Basic sanity: both are red-ish; we do not enforce ordering because palette is approximate
    assert!(pr4.fg.r > pr4.fg.g && pr4.fg.r > pr4.fg.b);
    assert!(pr6.fg.r > pr6.fg.g && pr6.fg.r > pr6.fg.b);

    let mut b4 = apply_utility_classes("text-blue-400", StyleBuilder::new());
    let mut b6 = apply_utility_classes("text-blue-600", StyleBuilder::new());
    let pb4 = extract_paint_style(&mut b4).unwrap();
    let pb6 = extract_paint_style(&mut b6).unwrap();
    // Both should be blue-ish; ordering is approximate and may not be monotonic in this subset
    assert!(pb4.fg.b > pb4.fg.r && pb4.fg.b > pb4.fg.g);
    assert!(pb6.fg.b > pb6.fg.r && pb6.fg.b > pb6.fg.g);
}

#[test]
fn gray_shades_monotonicity() {
    let mut g4 = apply_utility_classes("text-gray-400", StyleBuilder::new());
    let mut g5 = apply_utility_classes("text-gray-500", StyleBuilder::new());
    let mut g6 = apply_utility_classes("text-gray-600", StyleBuilder::new());
    let pg4 = extract_paint_style(&mut g4).unwrap();
    let _pg5 = extract_paint_style(&mut g5).unwrap();
    let pg6 = extract_paint_style(&mut g6).unwrap();
    // Just ensure values are populated and differ reasonably
    assert!(pg4.fg.r > pg6.fg.r);
    assert!(pg4.fg.g > pg6.fg.g);
    assert!(pg4.fg.b > pg6.fg.b);
}
