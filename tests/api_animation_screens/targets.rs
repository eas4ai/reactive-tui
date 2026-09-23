use reactive_tui::{
    animation::{
        api::{try_animate, AnimateParams, PropertyValue},
        AnimatedProperty, AnimatedValue, AnimationTargetError, TransformProperty,
    },
    backend::SuprTuiBackend,
    builder::core::div,
    component::Element,
    screen::{ScreenId, ScreenManager},
};
use std::{
    collections::HashMap,
    io::{self, Write},
    sync::{Arc, Mutex},
    time::Duration,
};
#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<u8>>>);
impl Write for Capture {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl Capture {
    fn screen(&self) -> vt100::Screen {
        let mut parser = vt100::Parser::new(4, 20, 0);
        parser.process(&self.0.lock().unwrap());
        parser.screen().clone()
    }
}
fn element(id: &str, css: &str) -> Element {
    use reactive_tui::vdom::{VElement, VNode};
    reactive_tui::vdom::bridge::vdom_to_element(VNode::Element(
        VElement::new("flex")
            .key(id)
            .class("w-full h-full bg-white")
            .style(css)
            .child(VNode::text("TARGET")),
    ))
}
/// The capture once every presented frame has been written (PIP-001).
fn synced<'a>(manager: &mut ScreenManager, output: &'a Capture) -> &'a Capture {
    manager.sync().unwrap();
    output
}

fn fixture(root: Element) -> (ScreenManager, Capture) {
    let output = Capture::default();
    let mut manager = ScreenManager::new(Box::new(
        SuprTuiBackend::with_writer(20, 4, output.clone()).unwrap(),
    ));
    manager.create_screen("main", "Main".into(), root).unwrap();
    (manager, output)
}
fn params(value: PropertyValue) -> AnimateParams {
    AnimateParams {
        opacity: Some(value),
        autoplay: Some(false),
        ..Default::default()
    }
}
fn relative(value: &str) -> PropertyValue {
    PropertyValue::Relative(value.into())
}
#[test]
fn relative_target_samples_paint_and_clear_restores_authored_values() {
    let (mut manager, output) = fixture(element("target", "opacity: 0.5"));
    let target = manager
        .animation_target(&ScreenId::from("main"), "target")
        .unwrap();
    let mut animation = try_animate(&target, params(relative("+0.25"))).unwrap();
    animation.try_seek(0.5).unwrap();
    manager.update().unwrap();
    assert_eq!(
        synced(&mut manager, &output)
            .screen()
            .cell(2, 10)
            .unwrap()
            .bgcolor(),
        vt100::Color::Rgb(159, 159, 159)
    );
    assert_eq!(
        animation.get_current_values(),
        Some(AnimatedValue::Opacity(0.625))
    );
    animation.try_seek(1.0).unwrap();
    manager.update().unwrap();
    assert_eq!(
        synced(&mut manager, &output)
            .screen()
            .cell(2, 10)
            .unwrap()
            .bgcolor(),
        vt100::Color::Rgb(191, 191, 191)
    );
    target.clear().unwrap();
    manager.update().unwrap();
    assert_eq!(
        synced(&mut manager, &output)
            .screen()
            .cell(2, 10)
            .unwrap()
            .bgcolor(),
        vt100::Color::Rgb(128, 128, 128)
    );
}

#[test]
fn numeric_array_keyframes_keep_middle_values_in_samples_and_presented_frames() {
    let (mut manager, output) = fixture(element("target", "opacity: 0.5"));
    let target = manager
        .animation_target(&ScreenId::from("main"), "target")
        .unwrap();
    for bound in [false, true] {
        let input: reactive_tui::animation::api::AnimationTargetInput = if bound {
            (&target).into()
        } else {
            "target".into()
        };
        let mut animation =
            try_animate(input, params(PropertyValue::Array(vec![0.0, 1.0, 0.0]))).unwrap();
        animation.try_seek(0.5).unwrap();
        assert_eq!(
            animation.get_current_values(),
            Some(AnimatedValue::Custom("opacity".into(), 1.0))
        );
        if bound {
            manager.update().unwrap();
            assert_eq!(
                synced(&mut manager, &output)
                    .screen()
                    .cell(2, 10)
                    .unwrap()
                    .bgcolor(),
                vt100::Color::Rgb(255, 255, 255)
            );
            animation.try_seek(0.75).unwrap();
            manager.update().unwrap();
            assert_eq!(
                synced(&mut manager, &output)
                    .screen()
                    .cell(2, 10)
                    .unwrap()
                    .bgcolor(),
                vt100::Color::Rgb(128, 128, 128)
            );
        }
    }
}
#[test]
fn relative_targets_resolve_current_values_at_play_and_restart_but_not_resume() {
    let (mut manager, _) = fixture(element("target", "opacity: 0.25"));
    let id = ScreenId::from("main");
    let target = manager.animation_target(&id, "target").unwrap();
    let mut animation = try_animate(&target, params(relative("+0.25"))).unwrap();
    manager
        .update_screen(&id, element("target", "opacity: 0.5"))
        .unwrap();
    animation.try_play().unwrap();
    assert_eq!(animation.property, AnimatedProperty::Opacity(0.5, 0.75));
    animation.try_seek(0.5).unwrap();
    manager.update().unwrap();
    animation.pause();
    animation.try_play().unwrap();
    assert_eq!(animation.property, AnimatedProperty::Opacity(0.5, 0.75));
    animation.stop();
    animation.try_play().unwrap();
    assert_eq!(animation.property, AnimatedProperty::Opacity(0.625, 0.875));
}
#[test]
fn relative_transform_arithmetic_uses_presented_values_and_preserves_units() {
    let mut root = element("target", "");
    root.class =
        Some("w-full h-full bg-white translate-x-8 translate-y-6 scale-200 rotate-90".into());
    let (manager, _) = fixture(root);
    let target = manager
        .animation_target(&ScreenId::from("main"), "target")
        .unwrap();
    for (expression, end) in [("+3", 11.0), ("-3", 5.0), ("*2", 16.0), ("/2", 4.0)] {
        let animation = try_animate(
            &target,
            AnimateParams {
                translate_x: Some(relative(expression)),
                autoplay: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            animation.property,
            AnimatedProperty::Transform(TransformProperty::TranslateX(8.0, end))
        );
    }
    let animation = try_animate(
        &target,
        AnimateParams {
            scale: Some(relative("*3")),
            rotate: Some(relative("+45")),
            autoplay: Some(false),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        animation.property,
        AnimatedProperty::Multiple(vec![
            AnimatedProperty::Transform(TransformProperty::Scale(2.0, 6.0)),
            AnimatedProperty::Transform(TransformProperty::Rotate(
                90.0_f32.to_radians(),
                135.0_f32.to_radians()
            ))
        ])
    );
}
#[test]
fn relative_target_removal_and_owner_drop_invalidate_handles_without_rebinding() {
    let (mut manager, _) = fixture(element("target", "opacity: 0.5"));
    let id = ScreenId::from("main");
    let target = manager.animation_target(&id, "target").unwrap();
    let context = manager.animation_targets(&id).unwrap();
    let mut animation = try_animate(&target, params(relative("+0.25"))).unwrap();
    animation.try_play().unwrap();
    manager
        .update_screen(&id, element("replacement", "opacity: 0.1"))
        .unwrap();
    manager
        .update_screen(&id, element("target", "opacity: 0.9"))
        .unwrap();
    assert!(matches!(
        animation.try_play(),
        Err(AnimationTargetError::MissingTarget(_))
    ));
    assert!(animation.try_seek(1.0).is_err());
    assert!(!animation.update(Duration::from_millis(10)));
    assert!(animation.get_current_values().is_none());
    let replacement = context.target("target").unwrap();
    drop(manager);
    assert!(replacement.clear().is_err());
    assert!(context.target("target").is_err());
}
#[test]
fn relative_multi_target_and_owner_isolation_keep_distinct_start_values() {
    let (manager_a, _) = fixture(element("same", "opacity: 0.25"));
    let (manager_b, _) = fixture(element("same", "opacity: 0.5"));
    let id = ScreenId::from("main");
    let a = manager_a.animation_target(&id, "same").unwrap();
    let b = manager_b.animation_target(&id, "same").unwrap();
    let animation = try_animate(vec![a, b], params(relative("*2"))).unwrap();
    assert_eq!(
        animation.property,
        AnimatedProperty::Multiple(vec![
            AnimatedProperty::Opacity(0.25, 0.5),
            AnimatedProperty::Opacity(0.5, 1.0)
        ])
    );
}
#[test]
fn relative_values_reject_guesses_invalid_expressions_and_undeclared_custom_values() {
    assert!(matches!(
        try_animate("target", params(relative("+1"))),
        Err(AnimationTargetError::HandleRequired(_))
    ));
    assert!(try_animate(
        "target",
        params(PropertyValue::FromTo { from: 0.0, to: 1.0 })
    )
    .is_ok());
    let (manager, _) = fixture(element("target", "opacity: 0.5"));
    let target = manager
        .animation_target(&ScreenId::from("main"), "target")
        .unwrap();
    for expression in ["/0", "+NaN", "*inf", "+", "no", "🙂"] {
        assert!(
            matches!(
                try_animate(&target, params(relative(expression))),
                Err(AnimationTargetError::InvalidValue(_, _))
            ),
            "{expression}"
        );
    }
    assert!(matches!(
        try_animate(
            &target,
            AnimateParams {
                custom: Some(HashMap::from([("count".into(), relative("+1"))])),
                ..Default::default()
            }
        ),
        Err(AnimationTargetError::MissingProperty(_))
    ));
}
#[test]
fn relative_custom_values_are_explicit_and_duplicate_target_ids_are_rejected() {
    let mut root = element("target", "opacity: 0.5");
    root.metadata.animation_values.insert("count".into(), 8.0);
    let (mut manager, _) = fixture(root);
    let id = ScreenId::from("main");
    let target = manager.animation_target(&id, "target").unwrap();
    let animation = try_animate(
        &target,
        AnimateParams {
            custom: Some(HashMap::from([("count".into(), relative("/2"))])),
            autoplay: Some(false),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        animation.property,
        AnimatedProperty::Property("count".into(), 8.0, 4.0)
    );
    manager
        .update_screen(
            &id,
            div()
                .child(div().child(element("same", "")).build())
                .child(div().child(element("same", "")).build())
                .build(),
        )
        .unwrap();
    assert!(matches!(
        manager.animation_target(&id, "same"),
        Err(AnimationTargetError::AmbiguousTarget(_))
    ));
}

#[reactive_tui::component]
fn RelativeNumericComponent() -> Element {
    let mut root = element("inner", "");
    root.metadata.animation_values.insert("count".into(), 2.0);
    root
}
#[reactive_tui::component]
fn RelativeReplacementComponent() -> Element {
    element("inner", "")
}
#[test]
fn relative_component_targets_inherit_declared_values_and_reject_replaced_instances() {
    static REGISTER: std::sync::Once = std::sync::Once::new();
    REGISTER.call_once(|| {
        reactive_tui::component::registry::register_component::<RelativeNumericComponent>(
            "RelativeNumericComponent",
        )
        .unwrap();
        reactive_tui::component::registry::register_component::<RelativeReplacementComponent>(
            "RelativeReplacementComponent",
        )
        .unwrap();
    });
    let mut root = RelativeNumericComponent::element().key("target");
    root.metadata.animation_values.insert("count".into(), 8.0);
    let (mut manager, _) = fixture(root);
    let id = ScreenId::from("main");
    let target = manager.animation_target(&id, "target").unwrap();
    let animation = try_animate(
        &target,
        AnimateParams {
            custom: Some(HashMap::from([("count".into(), relative("+1"))])),
            autoplay: Some(false),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        animation.property,
        AnimatedProperty::Property("count".into(), 8.0, 9.0)
    );
    manager
        .update_screen(&id, RelativeReplacementComponent::element().key("target"))
        .unwrap();
    assert!(target.clear().is_err());
    assert!(manager.animation_target(&id, "target").is_ok());
}
