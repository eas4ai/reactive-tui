//! NAV-006: the breadcrumb's ellipsis and the tab bar's overflow mark.

use crate::common::app_input::{self, FramePredicate, Snapshot};
use reactive_tui::{app::RootComponent, builder, component::Element};

struct Root(Element);
impl RootComponent for Root {
    fn render(&self) -> Element {
        self.0.clone()
    }
}

/// Ten tabs of 20 cells each: an 18-cell label with one cell of padding on
/// each side.
fn ten_tabs() -> Element {
    let mut tabs = builder::tabs();
    for n in 1..=10 {
        let label = format!("{:<18}", format!("Tab {n:02}"));
        tabs = tabs.tab(&label, Element::text(format!("Panel {n:02}")));
    }
    tabs.build()
}

/// A box of `width` cells and `height` rows on the page.
fn boxed(width: u16, height: u16, child: Element) -> Element {
    builder::div()
        .class(&format!("flex-col w-{width} h-{height}"))
        .child(child)
        .build()
}

/// The frame once `needle` is painted.
fn shown(needle: &'static str) -> FramePredicate {
    Box::new(move |frame: &Snapshot| frame.text.contains(needle))
}

#[test]
#[serial_test::serial(theme)]
fn nav_006_a_tab_bar_with_tabs_out_of_view_paints_an_overflow_mark() {
    let frames = app_input::run_when_frame(
        Root(boxed(100, 10, ten_tabs())),
        (240, 60),
        vec![(shown("Panel 01"), None)],
    );
    let frame = frames.last().expect("a frame");
    let bar = frame
        .text
        .lines()
        .find(|line| line.contains("Tab 01"))
        .expect("the tab bar's row");
    assert!(
        bar.contains('»'),
        "ten tabs of 20 cells in a box of 100 cells paint an overflow mark at the bar's end:\n{}",
        frame.text
    );
}
