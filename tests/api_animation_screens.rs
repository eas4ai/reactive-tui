//! Acceptance checks for API-013 keyframes, owner-bound targets and screens.
use reactive_tui::animation::keyframes::{Keyframe, KeyframeAnimation, TypedKeyframe};
use std::time::Duration;

#[test]
fn typed_numeric_midpoint_interpolates() {
    let animation = KeyframeAnimation::from_typed(
        vec![
            TypedKeyframe {
                offset: 0.0,
                value: 0.0_f32,
                easing: None,
            },
            TypedKeyframe {
                offset: 1.0,
                value: 10.0_f32,
                easing: None,
            },
        ],
        Duration::from_secs(1),
    );
    assert_eq!(animation.get_value_at_time(0.5), Some(5.0));
}

#[test]
fn untyped_conversion_preserves_numeric_endpoints() {
    let animation = KeyframeAnimation::<f32>::new(vec![
        Keyframe::new(0.0).number("x", 4.0),
        Keyframe::new(1.0).number("x", 12.0),
    ]);
    assert_eq!(animation.get_value_at_time(0.0), Some(4.0));
    assert_eq!(animation.get_value_at_time(1.0), Some(12.0));
}

#[test]
fn typed_keyframes_use_destination_easing_and_native_precision() {
    use reactive_tui::animation::EasingFunction;
    let eased = KeyframeAnimation::from_typed(
        vec![
            TypedKeyframe {
                offset: 0.0,
                value: 0.0_f32,
                easing: None,
            },
            TypedKeyframe {
                offset: 1.0,
                value: 10.0_f32,
                easing: Some(EasingFunction::InPower(2.0)),
            },
        ],
        Duration::from_secs(1),
    );
    assert_eq!(eased.get_value_at_time(0.25), Some(0.625));
    let precise = KeyframeAnimation::from_typed(
        vec![
            TypedKeyframe {
                offset: 0.0,
                value: 16_777_217.0_f64,
                easing: None,
            },
            TypedKeyframe {
                offset: 1.0,
                value: 16_777_219.0_f64,
                easing: None,
            },
        ],
        Duration::from_secs(1),
    );
    assert_eq!(precise.get_value_at_time(0.5), Some(16_777_218.0));
}

#[test]
fn typed_offsets_are_sorted_and_duplicate_offsets_use_the_last_value() {
    let animation = KeyframeAnimation::from_typed(
        [(0.75, 9.0), (0.0, 1.0), (0.75, 13.0), (1.0, 17.0)]
            .into_iter()
            .map(|(offset, value)| TypedKeyframe {
                offset,
                value,
                easing: None,
            })
            .collect(),
        Duration::from_secs(2),
    );
    assert_eq!(animation.get_value_at_time(0.375), Some(7.0_f32));
    assert_eq!(animation.get_value_at_time(0.75), Some(13.0));
    assert_eq!(animation.get_value_at_time(1.5), Some(17.0));
    assert_eq!(animation.duration(), Duration::from_secs(2));
}

#[test]
fn untyped_conversion_keeps_tuples_colors_and_discrete_values() {
    use reactive_tui::animation::keyframes::KeyframeValue;
    let tuple = KeyframeAnimation::<(f32, f32)>::new(vec![
        Keyframe::new(0.0).set_property(
            "position",
            KeyframeValue::Multiple(vec![KeyframeValue::Number(2.0), KeyframeValue::Number(6.0)]),
        ),
        Keyframe::new(1.0).set_property(
            "position",
            KeyframeValue::Multiple(vec![
                KeyframeValue::Number(4.0),
                KeyframeValue::Number(10.0),
            ]),
        ),
    ]);
    assert_eq!(tuple.get_value_at_time(0.5), Some((3.0, 8.0)));
    let color = KeyframeAnimation::<(u8, u8, u8, u8)>::new(vec![
        Keyframe::new(0.0).color(10, 20, 30, Some(40)),
        Keyframe::new(1.0).color(30, 40, 50, Some(80)),
    ]);
    assert_eq!(color.get_value_at_time(0.5), Some((20, 30, 40, 60)));
    let text = KeyframeAnimation::<String>::new(vec![
        Keyframe::new(0.0).set_property("label", KeyframeValue::String("before".into())),
        Keyframe::new(1.0).set_property("label", KeyframeValue::String("after".into())),
    ]);
    assert_eq!(text.get_value_at_time(0.75).as_deref(), Some("before"));
    assert_eq!(text.get_value_at_time(1.0).as_deref(), Some("after"));
}

#[test]
fn sparse_untyped_properties_interpolate_between_their_own_keyframes() {
    use reactive_tui::animation::{
        keyframes::{KeyframeSequence, KeyframeValue},
        EasingFunction,
    };
    let sequence = KeyframeSequence::new(Duration::from_secs(1))
        .default_easing(EasingFunction::Linear)
        .add_keyframe(Keyframe::new(0.0).number("x", 0.0))
        .add_keyframe(Keyframe::new(0.5).number("y", 2.0))
        .add_keyframe(Keyframe::new(1.0).number("x", 10.0));
    assert_eq!(
        sequence.sample(0.25).get("x"),
        Some(&KeyframeValue::Number(2.5))
    );
    assert_eq!(
        sequence.sample(0.5).get("x"),
        Some(&KeyframeValue::Number(5.0))
    );
    assert_eq!(
        sequence.sample(0.75).get("x"),
        Some(&KeyframeValue::Number(7.5))
    );
    assert_eq!(
        sequence.sample(0.25).get("y"),
        Some(&KeyframeValue::Number(2.0))
    );
}

#[test]
fn untyped_discrete_properties_remain_present_between_keyframes() {
    use reactive_tui::animation::keyframes::{KeyframeSequence, KeyframeValue};
    let sequence = KeyframeSequence::new(Duration::from_secs(1))
        .add_keyframe(
            Keyframe::new(0.0)
                .set_property("bold", KeyframeValue::Boolean(false))
                .set_property("fontStyle", KeyframeValue::String("normal".into())),
        )
        .add_keyframe(
            Keyframe::new(1.0)
                .set_property("bold", KeyframeValue::Boolean(true))
                .set_property("fontStyle", KeyframeValue::String("italic".into())),
        );
    assert_eq!(
        sequence.sample(0.75).get("bold"),
        Some(&KeyframeValue::Boolean(false))
    );
    assert_eq!(
        sequence.sample(0.75).get("fontStyle"),
        Some(&KeyframeValue::String("normal".into()))
    );
    assert_eq!(
        sequence.sample(1.0).get("bold"),
        Some(&KeyframeValue::Boolean(true))
    );
}

#[test]
fn checked_conversion_requires_a_named_property_and_rejects_value_loss() {
    use reactive_tui::animation::{keyframes::KeyframeValue, CssValue};
    let frames = vec![
        Keyframe::new(0.0).number("x", 3.0).number("y", 7.0),
        Keyframe::new(1.0).number("x", 9.0).number("y", 11.0),
    ];
    assert!(KeyframeAnimation::<f32>::try_new(frames.clone()).is_err());
    let x =
        KeyframeAnimation::<f32>::try_from_property(frames, "x", Duration::from_secs(3)).unwrap();
    assert_eq!(x.get_value_at_time(0.5), Some(6.0));
    assert_eq!(x.duration(), Duration::from_secs(3));
    assert!(KeyframeAnimation::<f32>::try_new(vec![
        Keyframe::new(0.0).number("x", 3.0),
        Keyframe::new(1.0).number("y", 9.0)
    ])
    .is_err());
    for value in [
        KeyframeValue::Number(1.5),
        KeyframeValue::Number(-1.0),
        KeyframeValue::Number(65536.0),
    ] {
        assert!(KeyframeAnimation::<u16>::try_new(vec![
            Keyframe::new(0.0).set_property("value", value)
        ])
        .is_err());
    }
    assert!(KeyframeAnimation::<f32>::try_new(vec![
        Keyframe::new(0.0).css_value("width", CssValue::Pixels(3.0))
    ])
    .is_err());
    assert!(
        KeyframeAnimation::<f32>::try_new(vec![Keyframe::new(0.0).number("x", f32::NAN)]).is_err()
    );
    assert!(KeyframeAnimation::<f32>::try_new(vec![Keyframe::new(0.0)]).is_err());
}

#[test]
fn invalid_offsets_return_errors_and_empty_sampling_stays_empty() {
    for offset in [f32::NAN, f32::INFINITY, -0.5, 1.5] {
        assert!(KeyframeAnimation::try_from_typed(
            vec![TypedKeyframe {
                offset,
                value: 2.0_f32,
                easing: None
            }],
            Duration::from_secs(1)
        )
        .is_err());
    }
    let empty = KeyframeAnimation::<f32>::try_new(vec![]).unwrap();
    assert_eq!(empty.get_value_at_time(0.5), None);
    let valid = KeyframeAnimation::new(vec![Keyframe::new(0.0).number("x", 2.0)]);
    assert_eq!(valid.get_value_at_time(f32::NAN), None::<f32>);
}

#[test]
fn discrete_values_do_not_switch_early_when_easing_overshoots() {
    use reactive_tui::animation::{
        keyframes::{KeyframeSequence, KeyframeValue},
        EasingFunction,
    };
    let sequence = KeyframeSequence::new(Duration::from_secs(1))
        .default_easing(EasingFunction::OutBack(1.70158))
        .add_keyframe(Keyframe::new(0.0).set_property("bold", KeyframeValue::Boolean(false)))
        .add_keyframe(Keyframe::new(1.0).set_property("bold", KeyframeValue::Boolean(true)));
    assert_eq!(
        sequence.sample(0.75).get("bold"),
        Some(&KeyframeValue::Boolean(false))
    );
}

#[test]
fn a_custom_keyframe_type_defines_conversion_and_interpolation() {
    use reactive_tui::animation::{
        keyframes::KeyframeValue, EasingFunction, KeyframeError, KeyframeType,
    };
    #[derive(Clone, Debug, PartialEq)]
    struct Distance(f64);
    impl KeyframeType for Distance {
        fn interpolate_keyframe(&self, to: &Self, progress: f32, easing: &EasingFunction) -> Self {
            Self(self.0.interpolate_keyframe(&to.0, progress, easing))
        }
        fn from_keyframe(value: &KeyframeValue) -> Result<Self, KeyframeError> {
            f64::from_keyframe(value).map(Self)
        }
    }
    let animation = KeyframeAnimation::<Distance>::new(vec![
        Keyframe::new(0.0).number("distance", 2.0),
        Keyframe::new(1.0).number("distance", 10.0),
    ]);
    assert_eq!(animation.get_value_at_time(0.5), Some(Distance(6.0)));
}

#[test]
fn typed_css_transform_and_compound_values_keep_units_and_alpha() {
    use reactive_tui::animation::{keyframes::KeyframeValue, CssValue, TransformMatrix};
    let css = KeyframeAnimation::<CssValue>::new(vec![
        Keyframe::new(0.0).css_value("width", CssValue::Percentage(20.0)),
        Keyframe::new(1.0).css_value("width", CssValue::Percentage(60.0)),
    ]);
    assert_eq!(css.get_value_at_time(0.5), Some(CssValue::Percentage(40.0)));
    let mut from = TransformMatrix {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };
    from.e = 2.0;
    let mut to = TransformMatrix {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };
    to.e = 10.0;
    let matrix = KeyframeAnimation::<TransformMatrix>::new(vec![
        Keyframe::new(0.0).transform(from),
        Keyframe::new(1.0).transform(to),
    ]);
    assert_eq!(matrix.get_value_at_time(0.5).unwrap().e, 6.0);
    let compound = KeyframeAnimation::<KeyframeValue>::new(vec![
        Keyframe::new(0.0).set_property(
            "mixed",
            KeyframeValue::Multiple(vec![
                KeyframeValue::Number(2.0),
                KeyframeValue::Boolean(false),
                KeyframeValue::Color(0, 0, 0, 20),
            ]),
        ),
        Keyframe::new(1.0).set_property(
            "mixed",
            KeyframeValue::Multiple(vec![
                KeyframeValue::Number(6.0),
                KeyframeValue::Boolean(true),
                KeyframeValue::Color(0, 0, 0, 60),
            ]),
        ),
    ]);
    assert_eq!(
        compound.get_value_at_time(0.5),
        Some(KeyframeValue::Multiple(vec![
            KeyframeValue::Number(4.0),
            KeyframeValue::Boolean(false),
            KeyframeValue::Color(0, 0, 0, 40)
        ]))
    );
}

#[test]
fn active_screen_routes_keyboard_activation_to_its_control() {
    use reactive_tui::{
        backend::SuprTuiBackend,
        builder::core::div,
        event::types::{Event, KeyCode, KeyEvent},
        screen::ScreenManager,
    };
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
    let element = div()
        .class("w-8 h-1")
        .text("Activate")
        .on_click(move || {
            count.fetch_add(1, Ordering::SeqCst);
        })
        .build()
        .auto_focus();
    let backend = SuprTuiBackend::with_writer(20, 4, std::io::sink()).unwrap();
    let mut screens = ScreenManager::new(Box::new(backend));
    screens
        .create_screen("main", "Main".into(), element)
        .unwrap();
    screens
        .process_event(&Event::Key(KeyEvent::new(KeyCode::Enter)))
        .unwrap();
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "active screen discarded keyboard input"
    );
}

#[path = "api_animation_screens/screens.rs"]
mod screens;

#[path = "api_animation_screens/targets.rs"]
mod targets;

#[test]
fn untyped_animation_wrapper_preserves_every_property_unit_and_alpha() {
    use reactive_tui::animation::{
        api::{try_animate, AnimateParams},
        keyframes::{KeyframeSequence, KeyframeValue},
        AnimatedValue, AnimationValue, CssValue, EasingFunction,
    };
    use std::collections::HashMap;
    let frame = |offset, x| {
        Keyframe::new(offset)
            .number("x", x)
            .set_property("text", KeyframeValue::String("kept".into()))
            .set_property("flag", KeyframeValue::Boolean(true))
            .set_property("width", KeyframeValue::Css(CssValue::Percentage(40.0)))
            .set_property("color", KeyframeValue::Color(10, 20, 30, 128))
    };
    let sequence = KeyframeSequence {
        keyframes: vec![frame(0.0, 4.0), frame(1.0, 12.0)],
        duration: Duration::from_secs(1),
        default_easing: EasingFunction::Linear,
    };
    let mut animation = try_animate(
        "explicit-keyframes",
        AnimateParams {
            keyframes: Some(sequence),
            autoplay: Some(false),
            ..Default::default()
        },
    )
    .unwrap();
    animation.seek(0.5);
    assert_eq!(
        animation.get_current_values(),
        Some(AnimatedValue::Animation(AnimationValue::Map(
            HashMap::from([
                ("x".into(), AnimationValue::Number(8.0)),
                ("text".into(), AnimationValue::String("kept".into())),
                ("flag".into(), AnimationValue::Boolean(true)),
                ("width".into(), AnimationValue::Unit(40.0, "%".into())),
                (
                    "color".into(),
                    AnimationValue::Map(HashMap::from([
                        ("r".into(), AnimationValue::Number(10.0)),
                        ("g".into(), AnimationValue::Number(20.0)),
                        ("b".into(), AnimationValue::Number(30.0)),
                        ("a".into(), AnimationValue::Number(128.0))
                    ]))
                )
            ])
        )))
    );
}
