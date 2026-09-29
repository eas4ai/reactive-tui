use reactive_tui::core::surface::{Attr, Rgba, Surface};
use reactive_tui::layout::css::parsers::parse_px;

#[test]
fn parse_px_accepts_px_suffix_and_raw() {
    assert_eq!(parse_px("p-8px", "p-"), Some(8.0));
    assert_eq!(parse_px("p-8", "p-"), Some(8.0));
}

#[test]
fn surface_write_str_handles_double_width() {
    let mut s = Surface::new(6, 1);
    let white = Rgba {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    let black = Rgba {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    s.write_str(0, 0, "古a", white, black, Attr::empty());
    let c0 = s.get(0, 0);
    let c1 = s.get(1, 0);
    let c2 = s.get(2, 0);
    assert_eq!(c0.ch, '古');
    assert_eq!(c1.ch, ' '); // padding cell
    assert_eq!(c2.ch, 'a');
}
