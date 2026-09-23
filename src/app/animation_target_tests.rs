use super::*;
use crate::{
    animation::{
        api::{try_animate, AnimateParams, PropertyValue},
        AnimatedProperty, AnimationTargetError,
    },
    backend::SuprTuiBackend,
    builder::core::div,
};
use std::{
    io::{self, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
};
#[derive(Clone, Default)]
struct Capture {
    bytes: Arc<Mutex<Vec<u8>>>,
    fail: Arc<AtomicBool>,
}
impl Write for Capture {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.fail.load(Ordering::SeqCst) {
            return Err(io::Error::other("injected presentation failure"));
        }
        self.bytes.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
struct Root(Arc<Mutex<Element>>);
impl RootComponent for Root {
    fn render(&self) -> Element {
        self.0.lock().unwrap().clone()
    }
}
fn element(opacity: f32) -> Element {
    div()
        .id("target")
        .class("w-full h-full bg-white")
        .styles(crate::layout::style::StyleBuilder::new().opacity(opacity))
        .build()
}
fn app(opacity: f32) -> (App, Arc<Mutex<Element>>, Capture) {
    let root = Arc::new(Mutex::new(element(opacity)));
    let output = Capture::default();
    let app = App::builder()
        .backend(SuprTuiBackend::with_writer(20, 4, output.clone()).unwrap())
        .root(Root(root.clone()))
        .build()
        .unwrap();
    (app, root, output)
}
fn params() -> AnimateParams {
    AnimateParams {
        opacity: Some(PropertyValue::Relative("+0.25".into())),
        autoplay: Some(false),
        ..Default::default()
    }
}
#[test]
fn relative_app_target_context_and_managed_samples_use_the_owner_and_paint() {
    if !isolated("relative_app_target_context_and_managed_samples_use_the_owner_and_paint") {
        return;
    }
    let (mut app, root, output) = app(0.25);
    let context = app.animation_targets();
    assert!(matches!(
        context.target("target"),
        Err(AnimationTargetError::MissingTarget(_))
    ));
    app.render().unwrap();
    let target = context.target("target").unwrap();
    let mut animation = try_animate(&target, params()).unwrap();
    *root.lock().unwrap() = element(0.5);
    app.render().unwrap();
    animation.try_play().unwrap();
    assert_eq!(animation.property, AnimatedProperty::Opacity(0.5, 0.75));
    let id = app.animation_manager().add_animation(animation);
    app.animation_manager()
        .get_animation_mut(&id)
        .unwrap()
        .try_seek(0.5)
        .unwrap();
    app.render().unwrap();
    let mut parser = vt100::Parser::new(4, 20, 0);
    app.backend.sync().unwrap();
    parser.process(&output.bytes.lock().unwrap());
    assert_eq!(
        parser.screen().cell(2, 10).unwrap().bgcolor(),
        vt100::Color::Rgb(159, 159, 159)
    );
    app.cleanup().unwrap();
    assert!(target.clear().is_err());
    assert!(context.target("target").is_err());
}
#[test]
fn relative_app_target_keeps_last_presented_properties_after_backend_failure() {
    if !isolated("relative_app_target_keeps_last_presented_properties_after_backend_failure") {
        return;
    }
    let (mut app, root, output) = app(0.5);
    app.render().unwrap();
    app.backend.sync().unwrap();
    let target = app.animation_target("target").unwrap();
    *root.lock().unwrap() = element(0.1);
    output.fail.store(true, Ordering::SeqCst);
    // The write fails after the present returned; the next render reports it
    // and the targets fall back to the last acknowledged frame (PIP-002).
    app.render().unwrap();
    assert!(app.render().is_err());
    let animation = try_animate(&target, params()).unwrap();
    assert_eq!(animation.property, AnimatedProperty::Opacity(0.5, 0.75));
}
#[test]
fn relative_app_target_handles_do_not_keep_an_app_alive() {
    if !isolated("relative_app_target_handles_do_not_keep_an_app_alive") {
        return;
    }
    let (mut first, _, _) = app(0.25);
    let (mut second, _, _) = app(0.5);
    first.render().unwrap();
    second.render().unwrap();
    let target = first.animation_target("target").unwrap();
    let other = second.animation_target("target").unwrap();
    drop(first);
    assert!(try_animate(&target, params()).is_err());
    assert_eq!(
        try_animate(&other, params()).unwrap().property,
        AnimatedProperty::Opacity(0.5, 0.75)
    );
}

#[test]
fn relative_app_target_percentage_translation_uses_presented_size() {
    if !isolated("relative_app_target_percentage_translation_uses_presented_size") {
        return;
    }
    let (mut app, root, output) = app(1.0);
    let mut style = crate::layout::style::StyleBuilder::new();
    style.motion.transform.x_percent = 0.5;
    *root.lock().unwrap() = div()
        .id("target")
        .class("w-full h-full bg-white")
        .styles(style)
        .text("TARGET")
        .build();
    app.render().unwrap();
    let target = app.animation_target("target").unwrap();
    let mut animation = try_animate(
        &target,
        AnimateParams {
            translate_x: Some(PropertyValue::Relative("-5".into())),
            autoplay: Some(false),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        animation.property,
        AnimatedProperty::Transform(crate::animation::TransformProperty::TranslateX(10.0, 5.0))
    );
    animation.try_seek(1.0).unwrap();
    app.render().unwrap();
    let mut parser = vt100::Parser::new(4, 20, 0);
    app.backend.sync().unwrap();
    parser.process(&output.bytes.lock().unwrap());
    assert!(parser.screen().contents().starts_with("     TARGET"));
}

// App publishes a legacy global performance context. Keep these render probes
// in their own process so they do not change unrelated FPS-hook test defaults.
fn isolated(name: &str) -> bool {
    const FLAG: &str = "RTUI_RELATIVE_APP_TARGET_TEST";
    if std::env::var(FLAG).as_deref() == Ok(name) {
        return true;
    }
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            &format!("app::animation_target_tests::{name}"),
            "--nocapture",
        ])
        .env(FLAG, name)
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success(), "{name}: {status}");
            return false;
        }
        if std::time::Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("{name} exceeded 20 seconds");
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
