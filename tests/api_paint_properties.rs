use reactive_tui::{
    app::RootComponent,
    builder::core::div,
    component::{registry::register_component, Component, Element, Props},
    layout::{css::gradients::GradientDirection, style::StyleBuilder},
};
use std::{any::Any, sync::Once};

#[allow(dead_code)]
mod common;
use app_input::{run, Snapshot};
use common::app_input;

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
    // The border API samples clockwise around a 12-cell perimeter.
    // Top-right is four cells from the start: one third red -> blue.
    assert_eq!(bg(&captured, 0, 4), vt100::Color::Rgb(160, 0, 80));
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

#[test]
fn rotation_moves_upright_glyphs_on_the_cell_grid() {
    let captured = frame(Element::text("ABC").class("w-3 h-3 rotate-90"), (6, 5));
    for (row, expected) in ["A", "B", "C"].into_iter().enumerate() {
        assert_eq!(
            captured.screen.cell(row as u16, 2).unwrap().contents(),
            expected
        );
    }
}

#[test]
fn scale_changes_the_painted_box_extent() {
    let captured = frame(div().class("w-2 h-2 bg-red-500 scale-200").build(), (6, 5));
    assert_eq!(bg(&captured, 2, 2), vt100::Color::Rgb(239, 68, 68));
    assert_eq!(bg(&captured, 3, 3), vt100::Color::Rgb(0, 0, 0));
}

#[test]
fn spin_moves_an_asymmetric_shape_between_scheduled_frames() {
    let frames = run(
        Tree(Element::text("ABC").class("w-5 h-5 animate-spin duration-300 ease-linear")),
        (8, 7),
        vec![(10, None)],
    );
    assert!(frames
        .windows(2)
        .any(|frames| frames[0].text != frames[1].text));
}

#[test]
fn translucent_background_preserves_underlying_wide_graphemes() {
    let root = div()
        .class("w-full h-full bg-blue-500")
        .child(Element::text("界A").class("absolute top-0 left-0 w-4 h-1 text-white"))
        .child(
            div()
                .class("absolute top-0 left-0 w-4 h-1 bg-red-500 opacity-50")
                .build(),
        )
        .build();
    let captured = frame(root, (5, 3));
    assert!(captured.text.contains("界A"), "{}", captured.text);
    assert_eq!(bg(&captured, 0, 0), vt100::Color::Rgb(149, 99, 157));
    // VT100 keeps attributes on the leading cell of a wide grapheme.
    assert!(captured.screen.cell(0, 1).unwrap().is_wide_continuation());
    assert_eq!(bg(&captured, 0, 2), vt100::Color::Rgb(149, 99, 157));
}

#[test]
fn opacity_without_a_background_keeps_the_parent_color() {
    let root = div()
        .class("w-full h-full bg-blue-500")
        .child(
            div()
                .class("w-4 h-1 opacity-50")
                .child(Element::text("X"))
                .build(),
        )
        .build();
    let captured = frame(root, (5, 3));
    assert_eq!(bg(&captured, 0, 0), vt100::Color::Rgb(59, 130, 246));
    assert_eq!(
        captured.screen.cell(0, 0).unwrap().fgcolor(),
        vt100::Color::Rgb(157, 193, 251)
    );
}

#[test]
fn a_border_does_not_paint_over_its_transparent_interior() {
    let root = div()
        .class("w-full h-full bg-blue-500")
        .child(div().class("w-5 h-3").gradient_border(1).build())
        .build();
    let captured = frame(root, (5, 3));
    assert_eq!(bg(&captured, 0, 0), vt100::Color::Rgb(255, 0, 0));
    assert_eq!(bg(&captured, 1, 2), vt100::Color::Rgb(59, 130, 246));
}

#[test]
fn one_cell_gradient_samples_the_midpoint() {
    let captured = frame(
        div()
            .class("w-1 h-1")
            .gradient(GradientDirection::ToRight)
            .from_color(240, 0, 0)
            .to_color(0, 0, 240)
            .build(),
        (3, 3),
    );
    assert_eq!(bg(&captured, 0, 0), vt100::Color::Rgb(120, 0, 120));
}

#[test]
fn partial_stops_extend_the_nearest_color() {
    let captured = frame(
        div()
            .class("w-5 h-1")
            .gradient(GradientDirection::ToRight)
            .from_color(240, 0, 0)
            .via_color(0, 240, 0)
            .build(),
        (5, 3),
    );
    assert_eq!(bg(&captured, 0, 0), vt100::Color::Rgb(240, 0, 0));
    assert_eq!(bg(&captured, 0, 2), vt100::Color::Rgb(0, 240, 0));
    assert_eq!(bg(&captured, 0, 4), vt100::Color::Rgb(0, 240, 0));
}

#[test]
fn empty_and_single_cell_borders_have_defined_sampling() {
    let border = reactive_tui::layout::css::gradients::GradientBorder::rainbow_border(1);
    assert!(border.render_border(0, 0).is_empty());
    assert!(border.render_border(0, 4).is_empty());
    assert_eq!(border.render_border(1, 1), vec![vec![(0, 255, 0, 1.0)]; 4]);
}

#[test]
fn zero_scale_hides_the_complete_node() {
    let captured = frame(
        Element::text("hidden").class("w-6 h-2 bg-white scale-0"),
        (8, 4),
    );
    assert!(captured.text.trim().is_empty());
    assert_eq!(bg(&captured, 0, 0), vt100::Color::Rgb(0, 0, 0));
}

#[test]
fn rotation_radians_agree_across_direct_wrapped_and_painted_paths() {
    use reactive_tui::animation::{
        AnimatedProperty, AnimatedValue, AnimationValue, EasingFunction, LoopMode,
        TransformProperty,
    };
    use reactive_tui::layout::css::animations::{
        create_css_animation_spec, register_css_animation,
    };
    let rotation = TransformProperty::Rotate(0.0, std::f32::consts::PI);
    let AnimatedValue::Rotation(degrees) = rotation.interpolate(0.5) else {
        panic!("expected rotation")
    };
    assert!((degrees - 90.0).abs() < 0.0001);
    let AnimatedValue::Animation(AnimationValue::Transform(matrix)) =
        AnimatedProperty::Transform(rotation).interpolate(0.5)
    else {
        panic!("expected matrix")
    };
    assert!(matrix.a.abs() < 0.0001);
    assert!((matrix.b - 1.0).abs() < 0.0001);
    assert!((matrix.c + 1.0).abs() < 0.0001);
    assert!(matrix.d.abs() < 0.0001);
    for (name, property) in [
        (
            "api-quarter-radians",
            AnimatedProperty::Transform(TransformProperty::Rotate(
                std::f32::consts::FRAC_PI_2,
                std::f32::consts::FRAC_PI_2,
            )),
        ),
        (
            "api-quarter-degrees",
            AnimatedProperty::Rotation(90.0, 90.0),
        ),
    ] {
        register_css_animation(
            name.to_owned(),
            create_css_animation_spec(
                name,
                1000,
                EasingFunction::Linear,
                LoopMode::None,
                vec![property],
            ),
        )
        .unwrap();
        let captured = frame(
            Element::text("ABC").class(format!("w-3 h-3 animate-{name}")),
            (6, 5),
        );
        for (row, expected) in ["A", "B", "C"].into_iter().enumerate() {
            assert_eq!(
                captured.screen.cell(row as u16, 2).unwrap().contents(),
                expected,
                "{name}"
            );
        }
    }
}

#[test]
fn rotated_single_axis_overflow_clips_in_local_coordinates() {
    let captured = frame(
        div()
            .class("w-3 h-3 rotate-90 translate-x-4 overflow-x-hidden")
            .child(Element::text("X").class("absolute left-1 top-4 w-1 h-1"))
            .child(Element::text("Y").class("absolute left-4 top-1 w-1 h-1"))
            .build(),
        (10, 7),
    );
    assert_eq!(captured.screen.cell(1, 2).unwrap().contents(), "X");
    assert!(!captured.text.contains('Y'), "{}", captured.text);
}

fn animated_frame(
    name: &str,
    properties: Vec<reactive_tui::animation::AnimatedProperty>,
    class: &str,
    text: &str,
) -> Snapshot {
    use reactive_tui::{
        animation::{EasingFunction, LoopMode},
        layout::css::animations::{create_css_animation_spec, register_css_animation},
    };
    register_css_animation(
        name.to_owned(),
        create_css_animation_spec(
            name,
            1000,
            EasingFunction::Linear,
            LoopMode::None,
            properties,
        ),
    )
    .unwrap();
    frame(
        Element::text(text).class(format!("{class} animate-{name}")),
        (10, 7),
    )
}

#[test]
fn matrix_skew_and_separate_scale_axes_reach_the_painter() {
    use reactive_tui::animation::{AnimatedProperty as P, TransformMatrix, TransformProperty as T};
    let matrix = TransformMatrix {
        e: 2.0,
        f: 1.0,
        ..Default::default()
    };
    let captured = animated_frame(
        "api-matrix",
        vec![P::Transform(T::Matrix(matrix.clone(), matrix))],
        "w-3 h-3",
        "ABC",
    );
    assert_eq!(captured.screen.cell(1, 2).unwrap().contents(), "A");
    assert_eq!(captured.screen.cell(1, 4).unwrap().contents(), "C");
    let captured = animated_frame(
        "api-skew-x",
        vec![P::Transform(T::SkewX(45.0, 45.0))],
        "w-3 h-3 translate-x-3",
        "ABC",
    );
    assert_eq!(captured.screen.cell(0, 2).unwrap().contents(), "A");
    assert_eq!(captured.screen.cell(0, 4).unwrap().contents(), "C");
    let captured = animated_frame(
        "api-scale-x",
        vec![
            P::Transform(T::ScaleX(2.0, 2.0)),
            P::Transform(T::ScaleY(1.0, 1.0)),
        ],
        "w-3 h-3 translate-x-3",
        "ABC",
    );
    for (col, expected) in [(2, "A"), (4, "B"), (6, "C")] {
        assert_eq!(captured.screen.cell(0, col).unwrap().contents(), expected);
    }
}

#[test]
fn named_size_color_and_unit_properties_reach_layout_and_painting() {
    use reactive_tui::animation::{AnimatedProperty as P, CssValue};
    let captured = animated_frame(
        "api-named-values",
        vec![
            P::CssProperty(
                "width".into(),
                CssValue::Percentage(50.0),
                CssValue::Percentage(50.0),
            ),
            P::Property("height".into(), 2.0, 2.0),
            P::CssProperty(
                "backgroundColor".into(),
                CssValue::color(12, 34, 56),
                CssValue::color(12, 34, 56),
            ),
            P::Color((240, 120, 0), (240, 120, 0)),
            P::CssProperty(
                "translateY".into(),
                CssValue::ViewportHeight(100.0 / 7.0),
                CssValue::ViewportHeight(100.0 / 7.0),
            ),
        ],
        "",
        "X",
    );
    assert_eq!(captured.screen.cell(1, 0).unwrap().contents(), "X");
    assert_eq!(
        captured.screen.cell(1, 0).unwrap().fgcolor(),
        vt100::Color::Rgb(240, 120, 0)
    );
    assert_eq!(bg(&captured, 2, 4), vt100::Color::Rgb(12, 34, 56));
    assert_eq!(bg(&captured, 2, 5), vt100::Color::Rgb(0, 0, 0));
}

#[test]
fn keyframes_keep_all_named_properties_and_color_alpha() {
    use reactive_tui::animation::{
        keyframes::{Keyframe, KeyframeSequence, KeyframeValue},
        AnimatedProperty as P,
    };
    let point = |offset| {
        Keyframe::new(offset)
            .number("translateX", 2.0)
            .number("width", 3.0)
            .set_property("backgroundColor", KeyframeValue::Color(240, 0, 0, 128))
    };
    let sequence = KeyframeSequence::new(std::time::Duration::from_secs(1))
        .add_keyframe(point(0.0))
        .add_keyframe(point(1.0));
    let captured = animated_frame("api-keyframe-map", vec![P::Keyframes(sequence)], "h-1", "X");
    assert_eq!(captured.screen.cell(0, 2).unwrap().contents(), "X");
    assert_eq!(bg(&captured, 0, 4), vt100::Color::Rgb(120, 0, 0));
    assert_eq!(bg(&captured, 0, 5), vt100::Color::Rgb(0, 0, 0));
}

#[test]
fn rainbow_borders_cycle_without_input_and_explicit_borders_stay_static() {
    let frames = run(
        Tree(div().class("w-5 h-3").gradient_border(1).build()),
        (6, 4),
        vec![(8, None)],
    );
    assert!(frames
        .windows(2)
        .any(|pair| bg(&pair[0], 0, 0) != bg(&pair[1], 0, 0)));
    let frames = run(
        Tree(
            div()
                .class("w-5 h-3")
                .gradient(GradientDirection::ToRight)
                .from_color(255, 0, 0)
                .via_color(0, 255, 0)
                .to_color(0, 0, 255)
                .gradient_border(1)
                .build(),
        ),
        (6, 4),
        vec![
            (
                1,
                Some(reactive_tui::event::types::Event::Resize(
                    reactive_tui::event::types::ResizeEvent::new(7, 4),
                )),
            ),
            (2, None),
        ],
    );
    assert!(frames
        .iter()
        .all(|frame| bg(frame, 0, 0) == vt100::Color::Rgb(255, 0, 0)));
}

#[test]
fn transformed_callbacks_follow_visible_and_clipped_bounds() {
    use std::sync::{Arc, Mutex};
    let calls = Arc::new(Mutex::new(Vec::new()));
    let callback = |name| {
        let calls = calls.clone();
        move || calls.lock().unwrap().push(name)
    };
    let root = div()
        .class("w-3 h-3 rotate-90 translate-x-4 overflow-x-hidden")
        .child(
            div()
                .class("absolute left-1 top-4 w-1 h-1")
                .text("X")
                .on_click(callback("X"))
                .build(),
        )
        .child(
            div()
                .class("absolute left-4 top-1 w-1 h-1")
                .text("Y")
                .on_click(callback("Y"))
                .build(),
        )
        .build();
    let frames = run(
        Tree(root),
        (10, 7),
        vec![
            (1, app_input::click(2, 1)),
            (1, app_input::click(5, 4)),
            (1, None),
        ],
    );
    assert_eq!(frames[0].screen.cell(1, 2).unwrap().contents(), "X");
    assert!(!frames[0].text.contains('Y'));
    assert_eq!(*calls.lock().unwrap(), vec!["X"]);
}

#[test]
fn percentage_translation_uses_the_laid_out_box() {
    use reactive_tui::animation::{AnimatedProperty, CssValue};
    let captured = animated_frame(
        "api-percent-translate",
        vec![AnimatedProperty::CssProperty(
            "translateX".into(),
            CssValue::Percentage(100.0),
            CssValue::Percentage(100.0),
        )],
        "w-3 h-1",
        "ABC",
    );
    assert_eq!(captured.screen.cell(0, 3).unwrap().contents(), "A");
    assert_eq!(captured.screen.cell(0, 5).unwrap().contents(), "C");
}

#[test]
fn offscreen_translation_clips_whole_wide_graphemes() {
    let captured = frame(Element::text("界A").class("w-3 h-1 -translate-x-1"), (5, 3));
    assert_eq!(captured.screen.cell(0, 1).unwrap().contents(), "A");
    assert!(!captured.text.contains('界'));
    assert!(!captured.screen.cell(0, 0).unwrap().is_wide_continuation());
}
