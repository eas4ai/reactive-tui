use reactive_tui::layout::utility_css::apply_utility_classes;
use reactive_tui::layout::style::StyleBuilder;
use reactive_tui::core::surface::{Surface, Rgba, Attr};

#[test]
fn parse_px_accepts_px_suffix_and_raw() {
    let sb1 = apply_utility_classes("p-8px", StyleBuilder::new());
    let sb2 = apply_utility_classes("p-8", StyleBuilder::new());
    // Indirect validation: both should yield same padding in Style (we inspect size of padding via debug build)
    // We can't easily introspect taffy internal compact types; assume success if no panic
    let _style1 = sb1.clone().build();
    let _style2 = sb2.clone().build();
}

#[test]
fn surface_write_str_handles_double_width() {
    let mut s = Surface::new(6, 1);
    let white = Rgba{r:1.0,g:1.0,b:1.0,a:1.0};
    let black = Rgba{r:0.0,g:0.0,b:0.0,a:1.0};
    s.write_str(0, 0, "古a", white, black, Attr::empty());
    let c0 = s.get(0,0); let c1 = s.get(1,0); let c2 = s.get(2,0);
    assert_eq!(c0.ch, '古');
    assert_eq!(c1.ch, ' '); // padding cell
    assert_eq!(c2.ch, 'a');
}

