mod property;
use property::apply_property;

// Per-App motion clocks. Rendering a keyed node again does not restart its clock.

use crate::{
    animation::{EasingFunction, LoopMode},
    component::{bridge::element_style, Element},
    error::Result,
    layout::{
        css::animations::{animation_spec, extract_css_animations_from_classes, CssAnimationSpec},
        motion::{CellTransform, Transition},
        style::StyleBuilder,
    },
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Clone, Hash, PartialEq, Eq)]
enum Slot {
    Key(String),
    Index(usize),
}

#[derive(Clone, Copy, PartialEq)]
struct Values {
    fg: (f32, f32, f32, f32),
    bg: Option<(f32, f32, f32, f32)>,
    opacity: f32,
    transform: CellTransform,
}

impl Values {
    fn read(style: &StyleBuilder) -> Self {
        Self {
            fg: style.fg_rgba.unwrap_or((1.0, 1.0, 1.0, 1.0)),
            bg: style.bg_rgba,
            opacity: style.opacity.unwrap_or(1.0),
            transform: style.motion.transform,
        }
    }
    fn apply(self, style: &mut StyleBuilder) {
        style.fg_rgba = Some(self.fg);
        style.bg_rgba = self.bg;
        style.opacity = Some(self.opacity);
        style.motion.transform = self.transform;
    }
    fn between(self, end: Self, t: f32, properties: Transition) -> Self {
        let lerp = |a: f32, b: f32| a + (b - a) * t;
        let color = |a: (f32, f32, f32, f32), b: (f32, f32, f32, f32)| {
            (
                lerp(a.0, b.0),
                lerp(a.1, b.1),
                lerp(a.2, b.2),
                lerp(a.3, b.3),
            )
        };
        let mut result = end;
        if matches!(properties, Transition::All | Transition::Colors) {
            result.fg = color(self.fg, end.fg);
        }
        if matches!(
            properties,
            Transition::All | Transition::Colors | Transition::Shadow
        ) {
            result.bg = match (self.bg, end.bg) {
                (None, None) => None,
                (from, to) => Some(color(
                    from.unwrap_or((0.0, 0.0, 0.0, 0.0)),
                    to.unwrap_or((0.0, 0.0, 0.0, 0.0)),
                )),
            };
        }
        if matches!(properties, Transition::All | Transition::Opacity) {
            result.opacity = lerp(self.opacity, end.opacity);
        }
        if matches!(properties, Transition::All | Transition::Transform) {
            result.transform = CellTransform {
                x: lerp(self.transform.x, end.transform.x),
                y: lerp(self.transform.y, end.transform.y),
                x_percent: lerp(self.transform.x_percent, end.transform.x_percent),
                y_percent: lerp(self.transform.y_percent, end.transform.y_percent),
                scale_x: lerp(self.transform.scale_x, end.transform.scale_x),
                scale_y: lerp(self.transform.scale_y, end.transform.scale_y),
                rotation: lerp(self.transform.rotation, end.transform.rotation),
                skew_x: lerp(self.transform.skew_x, end.transform.skew_x),
                skew_y: lerp(self.transform.skew_y, end.transform.skew_y),
                matrix: std::array::from_fn(|i| {
                    lerp(self.transform.matrix[i], end.transform.matrix[i])
                }),
            };
        }
        result
    }
}

struct NodeMotion {
    border_cycle: Option<Duration>,
    border_started: Instant,
    transition: Transition,
    duration: Duration,
    easing: EasingFunction,
    animations: Vec<CssAnimationSpec>,
    started: Instant,
    target: Values,
    from: Values,
    changed: Instant,
}

#[derive(Default)]
pub(super) struct MotionTree {
    nodes: HashMap<Vec<Slot>, NodeMotion>,
    active: bool,
    viewport: (u16, u16),
}

impl MotionTree {
    pub(super) fn active(&self) -> bool {
        self.active
    }
    pub(super) fn clear(&mut self) {
        self.nodes.clear();
        self.active = false;
    }
    pub(super) fn apply(
        &mut self,
        element: &mut Element,
        now: Instant,
        viewport: (u16, u16),
    ) -> Result<()> {
        self.active = false;
        self.viewport = viewport;
        let mut seen = HashSet::new();
        self.visit(element, Vec::new(), 0, now, &mut seen)?;
        self.nodes.retain(|path, _| seen.contains(path));
        Ok(())
    }
    fn visit(
        &mut self,
        element: &mut Element,
        mut path: Vec<Slot>,
        index: usize,
        now: Instant,
        seen: &mut HashSet<Vec<Slot>>,
    ) -> Result<()> {
        path.push(
            element
                .key
                .as_ref()
                .map_or(Slot::Index(index), |key| Slot::Key(key.clone())),
        );
        let mut style = element_style(element)?;
        if style.accessibility.contains_key("reduced-motion") {
            // Use the requested static values immediately and release this
            // node's animation clock when the current traversal finishes.
            element.metadata.paint_style = Some(Arc::new(style.snapshot()));
            for (index, child) in element.children.iter_mut().enumerate() {
                self.visit(child, path.clone(), index, now, seen)?;
            }
            return Ok(());
        }
        let mut names = extract_css_animations_from_classes(element.class.as_deref().unwrap_or(""));
        if names.is_empty()
            && !element
                .class
                .as_deref()
                .unwrap_or("")
                .split_whitespace()
                .any(|token| token == "animate-none")
        {
            if let Some(name) = style.get_css_animation() {
                names.push(name.to_owned());
            }
        }
        let border_cycle = style
            .gradient_border
            .as_ref()
            .filter(|border| border.width > 0)
            .and_then(|border| border.cycle_duration)
            .filter(|duration| !duration.is_zero())
            .filter(|_| {
                !element
                    .class
                    .as_deref()
                    .unwrap_or("")
                    .split_whitespace()
                    .any(|token| token == "animate-none")
            });
        if !names.is_empty()
            || style.motion.transition != Transition::None
            || border_cycle.is_some()
        {
            let specs = names
                .into_iter()
                .map(|name| {
                    let mut spec = animation_spec(&name)?;
                    if let Some(duration) = style.motion.duration {
                        spec.duration = duration;
                    }
                    if let Some(easing) = &style.motion.easing {
                        spec.easing = easing.clone();
                    }
                    Ok(spec)
                })
                .collect::<Result<Vec<_>>>()?;
            seen.insert(path.clone());
            let target = Values::read(&style);
            let duration = style.motion.duration.unwrap_or(Duration::from_millis(150));
            let easing = style
                .motion
                .easing
                .clone()
                .unwrap_or(EasingFunction::EaseInOut);
            let state = self
                .nodes
                .entry(path.clone())
                .or_insert_with(|| NodeMotion {
                    border_cycle,
                    border_started: now,
                    transition: style.motion.transition,
                    duration,
                    easing: easing.clone(),
                    animations: specs.clone(),
                    started: now,
                    target,
                    from: target,
                    changed: now,
                });
            if state.animations != specs {
                state.animations = specs;
                state.started = now;
            }
            if state.border_cycle != border_cycle {
                state.border_cycle = border_cycle;
                state.border_started = now;
            }
            let elapsed = now.saturating_duration_since(state.changed);
            let t = fraction(elapsed, state.duration);
            let mut current =
                state
                    .from
                    .between(state.target, state.easing.apply(t), state.transition);
            if state.target != target
                || state.duration != duration
                || state.easing != easing
                || state.transition != style.motion.transition
            {
                state.from = current;
                state.target = target;
                state.changed = now;
                state.duration = duration;
                state.easing = easing;
                state.transition = style.motion.transition;
                current = state.from.between(
                    target,
                    if duration.is_zero() { 1.0 } else { 0.0 },
                    style.motion.transition,
                );
            }
            self.active |= state.from != state.target
                && now.saturating_duration_since(state.changed) < duration;
            current.apply(&mut style);
            for spec in &state.animations {
                let elapsed = now.saturating_duration_since(state.started);
                let (progress, active) = animation_progress(elapsed, spec);
                self.active |= active;
                for property in &spec.properties {
                    apply_property(
                        &mut style,
                        property,
                        spec.easing.apply(progress),
                        self.viewport,
                    )?;
                }
            }
            if let Some(duration) = border_cycle {
                let phase = (now
                    .saturating_duration_since(state.border_started)
                    .as_secs_f64()
                    / duration.as_secs_f64())
                .fract() as f32;
                if let Some(border) = &mut style.gradient_border {
                    cycle_stops(&mut border.gradient.stops, phase);
                }
                self.active = true;
            }
            element.metadata.paint_style = Some(Arc::new(style.snapshot()));
        }
        for (index, child) in element.children.iter_mut().enumerate() {
            self.visit(child, path.clone(), index, now, seen)?;
        }
        Ok(())
    }
}

fn cycle_stops(stops: &mut crate::layout::css::gradients::GradientStops, phase: f32) {
    let phase = phase * 3.0;
    let sector = phase.floor() as usize % 3;
    let fraction = phase.fract();
    for stop in [&mut stops.from, &mut stops.via, &mut stops.to]
        .into_iter()
        .flatten()
    {
        let colors = [stop.0, stop.1, stop.2];
        let next: [u8; 3] = std::array::from_fn(|channel| {
            let from = f32::from(colors[(channel + 3 - sector) % 3]);
            let to = f32::from(colors[(channel + 2 - sector) % 3]);
            (from + (to - from) * fraction).round() as u8
        });
        (stop.0, stop.1, stop.2) = (next[0], next[1], next[2]);
    }
}

fn fraction(elapsed: Duration, duration: Duration) -> f32 {
    if duration.is_zero() {
        1.0
    } else {
        (elapsed.as_secs_f64() / duration.as_secs_f64()).min(1.0) as f32
    }
}

fn animation_progress(elapsed: Duration, spec: &CssAnimationSpec) -> (f32, bool) {
    if spec.duration.is_zero() {
        return (1.0, false);
    }
    let cycles = elapsed.as_secs_f64() / spec.duration.as_secs_f64();
    match spec.loop_mode {
        LoopMode::None => (cycles.min(1.0) as f32, cycles < 1.0),
        LoopMode::Count(count) if cycles >= f64::from(count.max(1)) => (1.0, false),
        LoopMode::PingPong => {
            let progress = cycles % 2.0;
            (
                if progress > 1.0 {
                    2.0 - progress
                } else {
                    progress
                } as f32,
                true,
            )
        }
        _ => (cycles.fract() as f32, true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduced_motion_releases_clocks_and_applies_static_values_immediately() {
        let now = Instant::now();
        let mut tree = MotionTree::default();
        let mut element = Element::text("X")
            .key("stable")
            .class("animate-pulse transition-opacity opacity-50");
        tree.apply(&mut element, now, (24, 6)).unwrap();
        assert!(tree.active());
        let mut reduced = Element::text("X")
            .key("stable")
            .class("animate-pulse transition-opacity opacity-100 reduced-motion");
        tree.apply(&mut reduced, now + Duration::from_millis(50), (24, 6))
            .unwrap();
        assert!(!tree.active());
        assert!(tree.nodes.is_empty());
        let style = reduced
            .metadata
            .paint_style
            .as_ref()
            .unwrap()
            .restore()
            .unwrap();
        assert_eq!(style.opacity, Some(1.0));
    }

    #[test]
    fn property_sets_keep_names_offsets_and_easing() {
        use crate::animation::{AnimatedProperty, AnimationValue, PropertyAnimation};
        let property = AnimatedProperty::PropertySet(vec![
            PropertyAnimation {
                name: "width".into(),
                from: AnimationValue::Number(2.0),
                to: AnimationValue::Number(10.0),
                duration_offset: 0.5,
                easing_override: Some(EasingFunction::InPower(2.0)),
            },
            PropertyAnimation {
                name: "opacity".into(),
                from: AnimationValue::Number(0.0),
                to: AnimationValue::Number(1.0),
                duration_offset: 0.0,
                easing_override: None,
            },
        ]);
        let mut style = StyleBuilder::new();
        apply_property(&mut style, &property, 0.75, (20, 10)).unwrap();
        assert_eq!(
            style.clone().build().size.width,
            taffy::Dimension::length(4.0)
        );
        assert_eq!(style.opacity, Some(0.75));
        apply_property(&mut style, &property, 1.0, (20, 10)).unwrap();
        assert_eq!(style.build().size.width, taffy::Dimension::length(10.0));
    }

    #[test]
    fn incompatible_css_units_keep_endpoints_and_unknown_styles_report_errors() {
        use crate::animation::{AnimatedProperty, CssValue};
        let property = AnimatedProperty::CssProperty(
            "width".into(),
            CssValue::Pixels(3.0),
            CssValue::Percentage(50.0),
        );
        let mut style = StyleBuilder::new();
        apply_property(&mut style, &property, 0.25, (20, 10)).unwrap();
        assert_eq!(
            style.clone().build().size.width,
            taffy::Dimension::length(3.0)
        );
        apply_property(&mut style, &property, 0.75, (20, 10)).unwrap();
        assert_eq!(
            style.clone().build().size.width,
            taffy::Dimension::percent(0.5)
        );
        let error = apply_property(
            &mut style,
            &AnimatedProperty::Property("not-a-style".into(), 0.0, 1.0),
            0.5,
            (20, 10),
        )
        .unwrap_err();
        assert!(error.to_string().contains("not-a-style"));
    }

    #[test]
    fn rainbow_clock_is_keyed_owned_and_removed() {
        use crate::builder::core::div;
        let start = Instant::now();
        let child = |key: &str| div().class("w-5 h-3").gradient_border(1).build().key(key);
        let color = |element: &Element| {
            element
                .metadata
                .paint_style
                .as_ref()
                .unwrap()
                .restore()
                .unwrap()
                .gradient_border
                .unwrap()
                .gradient
                .stops
                .from
                .unwrap()
        };
        let mut first = MotionTree::default();
        let mut root = Element::fragment().children(vec![child("a"), child("b")]);
        first.apply(&mut root, start, (20, 10)).unwrap();
        assert_eq!(color(&root.children[0]), (255, 0, 0, 1.0));
        let mut root = Element::fragment().children(vec![child("b"), child("a")]);
        first
            .apply(&mut root, start + Duration::from_secs(1), (20, 10))
            .unwrap();
        assert_eq!(color(&root.children[0]), (0, 128, 128, 1.0));
        assert_eq!(color(&root.children[1]), (0, 128, 128, 1.0));
        let mut second = MotionTree::default();
        let mut fresh = child("a");
        second
            .apply(&mut fresh, start + Duration::from_secs(1), (20, 10))
            .unwrap();
        assert_eq!(color(&fresh), (255, 0, 0, 1.0));
        first
            .apply(
                &mut Element::empty(),
                start + Duration::from_secs(1),
                (20, 10),
            )
            .unwrap();
        assert!(!first.active());
        assert!(first.nodes.is_empty());
        let mut fresh = child("a");
        first
            .apply(&mut fresh, start + Duration::from_secs(2), (20, 10))
            .unwrap();
        assert_eq!(color(&fresh), (255, 0, 0, 1.0));
    }

    #[test]
    fn static_and_zero_duration_borders_do_not_start_a_clock() {
        use crate::layout::css::gradients::{Gradient, GradientBorder, GradientDirection};
        for duration in [None, Some(Duration::ZERO)] {
            let mut border = GradientBorder::new(Gradient::new(GradientDirection::ToRight), 1);
            border.cycle_duration = duration;
            let mut element = Element::text("X");
            element.metadata.gradient_border = Some(border);
            let mut tree = MotionTree::default();
            tree.apply(&mut element, Instant::now(), (10, 5)).unwrap();
            assert!(!tree.active());
            assert!(tree.nodes.is_empty());
        }
    }

    #[test]
    fn style_snapshots_reject_nonfinite_layout_and_paint_numbers() {
        for style in [
            StyleBuilder::new().width_px(f32::NAN),
            StyleBuilder::new().height_percent(f32::INFINITY),
            StyleBuilder::new().opacity(f32::NAN),
            StyleBuilder::new().bg_rgba(f32::NAN, 0.0, 0.0, 1.0),
        ] {
            assert!(style.snapshot().restore().is_err());
        }
        assert!(StyleBuilder::new()
            .width_px(3.0)
            .opacity(0.5)
            .snapshot()
            .restore()
            .is_ok());
    }

    fn sample(tree: &mut MotionTree, class: &str, now: Instant) -> StyleBuilder {
        let mut element = Element::text("sample").class(class).key("stable");
        tree.apply(&mut element, now, (80, 24)).unwrap();
        element
            .metadata
            .paint_style
            .as_ref()
            .unwrap()
            .restore()
            .unwrap()
    }

    #[test]
    fn color_transition_has_exact_midpoint_and_stops() {
        let start = Instant::now();
        let mut tree = MotionTree::default();
        sample(
            &mut tree,
            "bg-black transition-colors duration-1000 ease-linear",
            start,
        );
        assert!(!tree.active());
        sample(
            &mut tree,
            "bg-white transition-colors duration-1000 ease-linear",
            start,
        );
        let middle = sample(
            &mut tree,
            "bg-white transition-colors duration-1000 ease-linear",
            start + Duration::from_millis(500),
        );
        assert_eq!(middle.bg_rgba, Some((0.5, 0.5, 0.5, 1.0)));
        assert!(tree.active());
        let end = sample(
            &mut tree,
            "bg-white transition-colors duration-1000 ease-linear",
            start + Duration::from_secs(1),
        );
        assert_eq!(end.bg_rgba, Some((1.0, 1.0, 1.0, 1.0)));
        assert!(!tree.active());
    }

    #[test]
    fn interrupted_transition_uses_old_curve_then_new_duration() {
        let start = Instant::now();
        let mut tree = MotionTree::default();
        sample(
            &mut tree,
            "bg-black transition-colors duration-1000 ease-linear",
            start,
        );
        sample(
            &mut tree,
            "bg-white transition-colors duration-1000 ease-linear",
            start,
        );
        let changed = sample(
            &mut tree,
            "bg-black transition-colors duration-200 ease-linear",
            start + Duration::from_millis(500),
        );
        assert_eq!(changed.bg_rgba, Some((0.5, 0.5, 0.5, 1.0)));
        let middle = sample(
            &mut tree,
            "bg-black transition-colors duration-200 ease-linear",
            start + Duration::from_millis(600),
        );
        assert_eq!(middle.bg_rgba, Some((0.25, 0.25, 0.25, 1.0)));
    }

    #[test]
    fn equal_keys_in_separate_apps_have_independent_clocks() {
        let start = Instant::now();
        let mut first = MotionTree::default();
        let mut second = MotionTree::default();
        let class = "animate-pulse duration-1000 ease-linear";
        sample(&mut first, class, start);
        let first = sample(&mut first, class, start + Duration::from_millis(500));
        let second = sample(&mut second, class, start + Duration::from_millis(500));
        assert_eq!(first.opacity, Some(0.75));
        assert_eq!(second.opacity, Some(1.0));
        assert_eq!(
            first.bg_rgba, None,
            "motion must not add an opaque background"
        );
    }

    #[test]
    fn removing_a_node_drops_its_clock_and_readding_starts_again() {
        let start = Instant::now();
        let mut tree = MotionTree::default();
        let class = "animate-pulse duration-1000 ease-linear";
        sample(&mut tree, class, start);
        tree.apply(
            &mut Element::empty(),
            start + Duration::from_millis(500),
            (80, 24),
        )
        .unwrap();
        assert!(tree.nodes.is_empty());
        assert!(!tree.active());
        let readded = sample(&mut tree, class, start + Duration::from_millis(600));
        assert_eq!(readded.opacity, Some(1.0));
    }

    #[test]
    fn keyed_reorder_preserves_animation_phase() {
        let start = Instant::now();
        let mut tree = MotionTree::default();
        let child = |key: &str| {
            Element::text(key)
                .key(key)
                .class("animate-pulse duration-1000 ease-linear")
        };
        let mut root = Element::fragment().children(vec![child("a"), child("b")]);
        tree.apply(&mut root, start, (80, 24)).unwrap();
        let mut root = Element::fragment().children(vec![child("b"), child("a")]);
        tree.apply(&mut root, start + Duration::from_millis(500), (80, 24))
            .unwrap();
        for child in root.children {
            assert_eq!(
                child
                    .metadata
                    .paint_style
                    .unwrap()
                    .restore()
                    .unwrap()
                    .opacity,
                Some(0.75)
            );
        }
        assert_eq!(tree.nodes.len(), 2);
    }
}
