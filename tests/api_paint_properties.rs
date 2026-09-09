use reactive_tui::{
    app::RootComponent,
    builder::core::div,
    component::{registry::register_component, Component, Element, Props},
    layout::{css::gradients::GradientDirection, style::StyleBuilder},
};
use std::{any::Any, sync::Once};

#[allow(dead_code)]
#[path = "support/app_input.rs"]
mod app_input;
use app_input::{run, Snapshot};

struct Tree(Element);
impl RootComponent for Tree {
    fn render(&self) -> Element {
        self.0.clone()
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

fn frame(element: Element, size: (u16, u16)) -> Snapshot {
    run(Tree(element), size, vec![(1, None)]).remove(0)
}

fn bg(frame: &Snapshot, row: u16, col: u16) -> vt100::Color {
    frame.screen.cell(row, col).unwrap().bgcolor()
}

#[test]
fn solid_color_control_reaches_the_terminal() {
    let captured = frame(div().class("w-full h-full bg-red-500").build(), (5, 3));
    assert_eq!(bg(&captured, 0, 0), vt100::Color::Rgb(239, 68, 68));
}

#[test]
fn builder_gradient_paints_both_endpoints_and_midpoint() {
    let captured = frame(
        div()
            .class("w-full h-full")
            .gradient(GradientDirection::ToRight)
            .from_color(240, 0, 0)
            .to_color(0, 0, 240)
            .build(),
        (5, 3),
    );
    assert_eq!(bg(&captured, 1, 0), vt100::Color::Rgb(240, 0, 0));
    assert_eq!(bg(&captured, 1, 2), vt100::Color::Rgb(120, 0, 120));
    assert_eq!(bg(&captured, 1, 4), vt100::Color::Rgb(0, 0, 240));
}

#[test]
fn css_gradient_keeps_the_via_stop() {
    let captured = frame(
        div()
            .class("w-full h-full bg-linear-to-r from-red-500 via-green-500 to-blue-500")
            .build(),
        (5, 3),
    );
    assert_eq!(bg(&captured, 1, 0), vt100::Color::Rgb(239, 68, 68));
    assert_eq!(bg(&captured, 1, 2), vt100::Color::Rgb(34, 197, 94));
    assert_eq!(bg(&captured, 1, 4), vt100::Color::Rgb(59, 130, 246));
}

#[test]
fn vertical_gradient_uses_rows() {
    let captured = frame(
        div()
            .class("w-full h-full")
            .gradient(GradientDirection::ToBottom)
            .from_color(240, 0, 0)
            .to_color(0, 0, 240)
            .build(),
        (5, 3),
    );
    assert_eq!(bg(&captured, 0, 2), vt100::Color::Rgb(240, 0, 0));
    assert_eq!(bg(&captured, 1, 2), vt100::Color::Rgb(120, 0, 120));
    assert_eq!(bg(&captured, 2, 2), vt100::Color::Rgb(0, 0, 240));
}

#[test]
fn gradient_border_has_visible_edge_colors() {
    let captured = frame(
        div()
            .class("w-full h-full")
            .gradient(GradientDirection::ToRight)
            .from_color(240, 0, 0)
            .to_color(0, 0, 240)
            .gradient_border(1)
            .build(),
        (5, 3),
    );
    assert_eq!(bg(&captured, 0, 0), vt100::Color::Rgb(240, 0, 0));
    assert_eq!(bg(&captured, 0, 4), vt100::Color::Rgb(0, 0, 240));
}

#[derive(Clone, Default, PartialEq)]
struct Label(String);
impl Props for Label {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
struct GradientLeaf;
impl Component for GradientLeaf {
    type Props = Label;
    type State = ();
    fn new(_: Label) -> Self {
        Self
    }
    fn render(&self, props: &Label, _: &()) -> Element {
        div()
            .class("w-full h-full")
            .gradient(GradientDirection::ToRight)
            .from_color(240, 0, 0)
            .to_color(0, 0, 240)
            .child(Element::text(&props.0))
            .build()
    }
}

#[test]
fn registered_component_output_retains_gradient() {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| register_component::<GradientLeaf>("ApiPaintGradientLeaf").unwrap());
    let captured = frame(
        Element::component_with_props("ApiPaintGradientLeaf", Label("ok".into())),
        (5, 3),
    );
    assert!(captured.text.contains("ok"));
    assert_eq!(bg(&captured, 1, 2), vt100::Color::Rgb(120, 0, 120));
}

#[test]
fn explicit_style_properties_reach_layout_and_paint() {
    let captured = frame(
        div()
            .styles(
                StyleBuilder::new()
                    .size_px(Some(3.0), Some(2.0))
                    .bg_rgba(1.0, 0.0, 0.0, 1.0),
            )
            .build(),
        (8, 4),
    );
    assert_eq!(bg(&captured, 1, 2), vt100::Color::Rgb(255, 0, 0));
    assert_ne!(bg(&captured, 1, 3), vt100::Color::Rgb(255, 0, 0));
}

#[test]
fn translate_changes_visible_text_position() {
    let captured = frame(
        Element::text("X").class("w-1 h-1 translate-x-2 translate-y-1"),
        (8, 4),
    );
    assert_eq!(captured.screen.cell(1, 2).unwrap().contents(), "X");
    assert!(captured
        .screen
        .cell(0, 0)
        .unwrap()
        .contents()
        .trim()
        .is_empty());
}

#[test]
fn css_pulse_schedules_intermediate_frames_without_input() {
    let frames = run(
        Tree(
            div()
                .class("w-full h-full bg-white animate-pulse duration-200 ease-linear")
                .build(),
        ),
        (5, 3),
        vec![(8, None)],
    );
    let colors: Vec<_> = frames.iter().map(|frame| bg(frame, 1, 2)).collect();
    assert!(
        colors.windows(2).any(|pair| pair[0] != pair[1]),
        "scheduled frames must visibly change: {colors:?}"
    );
    assert!(colors.iter().any(|color| matches!(color, vt100::Color::Rgb(r, g, b) if *r > 127 && *r < 255 && r == g && g == b)));
}
