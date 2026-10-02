//! Coordinate conversion shared by measured overlays.

use crate::component::LayoutInfo;
use std::sync::Arc;
use taffy::geometry::Rect;

fn map_rect(rect: Rect<f32>, map: impl Fn(f32, f32) -> (f32, f32)) -> Rect<f32> {
    let points = [
        (rect.left, rect.top),
        (rect.right, rect.top),
        (rect.left, rect.bottom),
        (rect.right, rect.bottom),
    ]
    .map(|(x, y)| map(x, y));
    Rect {
        left: points.iter().map(|p| p.0).fold(f32::INFINITY, f32::min),
        top: points.iter().map(|p| p.1).fold(f32::INFINITY, f32::min),
        right: points.iter().map(|p| p.0).fold(f32::NEG_INFINITY, f32::max),
        bottom: points.iter().map(|p| p.1).fold(f32::NEG_INFINITY, f32::max),
    }
}
pub(in crate::widgets) fn global_rect(layout: LayoutInfo, rect: Rect<f32>) -> Rect<f32> {
    let [a, b, c, d, tx, ty] = layout.transform;
    map_rect(rect, |x, y| (a * x + c * y + tx, b * x + d * y + ty))
}
pub(in crate::widgets) fn local_rect(layout: LayoutInfo, rect: Rect<f32>) -> Rect<f32> {
    let [a, b, c, d, tx, ty] = layout.transform;
    let determinant = a * d - b * c;
    if determinant.abs() < f32::EPSILON {
        return Rect::default();
    }
    map_rect(rect, |x, y| {
        (
            (d * (x - tx) - c * (y - ty)) / determinant,
            (-b * (x - tx) + a * (y - ty)) / determinant,
        )
    })
}

pub(in crate::widgets) fn same_callback<T: ?Sized>(a: &Option<Arc<T>>, b: &Option<Arc<T>>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}
