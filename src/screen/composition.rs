//! Terminal transitions transform cell placement; glyph bitmaps stay upright.
use super::TransitionType;
use crate::{builder::core::div, component::Element, layout::style::StyleBuilder};
use std::sync::Arc;

pub(super) fn compose(from: Element, to: Element, kind: TransitionType, progress: f32) -> Element {
    let p = progress.clamp(0.0, 1.0);
    let mut source = StyleBuilder::new();
    let mut target = StyleBuilder::new();
    match kind {
        TransitionType::None => {
            source = source.opacity(0.0);
        }
        TransitionType::Fade => {
            target = target.opacity(p);
        }
        TransitionType::SlideLeft | TransitionType::Push => {
            source.motion.transform.x_percent = -p;
            target.motion.transform.x_percent = 1.0 - p;
        }
        TransitionType::SlideRight => {
            source.motion.transform.x_percent = p;
            target.motion.transform.x_percent = p - 1.0;
        }
        TransitionType::SlideUp => {
            source.motion.transform.y_percent = -p;
            target.motion.transform.y_percent = 1.0 - p;
        }
        TransitionType::SlideDown => {
            source.motion.transform.y_percent = p;
            target.motion.transform.y_percent = p - 1.0;
        }
        TransitionType::Scale => {
            target.motion.transform.scale_x = p;
            target.motion.transform.scale_y = p;
            target = target.opacity(p);
        }
        TransitionType::Flip => {
            source.motion.transform.scale_x = (1.0 - 2.0 * p).max(0.0);
            target.motion.transform.scale_x = (2.0 * p - 1.0).max(0.0);
        }
        TransitionType::Cube => {
            source.motion.transform.scale_x = 1.0 - p;
            source.motion.transform.x_percent = -p * 0.5;
            target.motion.transform.scale_x = p;
            target.motion.transform.x_percent = (1.0 - p) * 0.5;
        }
    }
    let layer = |element, style: StyleBuilder| {
        let mut layer = div()
            .class("absolute inset-0 w-full h-full")
            .child(element)
            .build();
        layer.metadata.styles = Some(Arc::new(style.snapshot()));
        layer
    };
    div()
        .class("relative w-full h-full overflow-hidden")
        .child(layer(from, source))
        .child(layer(to, target))
        .build()
}

pub(super) fn node_count(element: &Element) -> usize {
    1 + element.children.iter().map(node_count).sum::<usize>()
}
