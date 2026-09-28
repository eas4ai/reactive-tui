//! The color a paint gives a pixel, the one rule both renderers follow
//! (GFX-002): colors resolved once per draw, gradients evaluated at the
//! pixel's centre in scene coordinates, stops interpolated premultiplied in
//! sRGB, then the paint's opacity.

use super::scene::{CanvasImage, Paint, PaintKind, Transform};
use std::sync::Arc;

/// The most gradient stops a paint keeps; the GPU's uniform block holds as
/// many.
pub(crate) const MAX_STOPS: usize = 16;

/// A premultiplied sRGB color in `0.0..=1.0`.
pub(crate) type Premul = [f32; 4];

/// Straight color to premultiplied.
pub(crate) fn premultiply([r, g, b, a]: [f32; 4]) -> Premul {
    [r * a, g * a, b * a, a]
}

/// A paint ready to color pixels of a picture.
#[derive(Clone, Debug)]
pub(crate) struct Shader {
    pub kind: ShaderKind,
    /// Picture pixels to scene coordinates, for gradients.
    pub inverse: Transform,
    pub opacity: f32,
}

#[derive(Clone, Debug)]
pub(crate) enum ShaderKind {
    Solid(Premul),
    /// t = dot(p - start, axis) / |axis|^2.
    Linear {
        start: (f32, f32),
        axis: (f32, f32),
        stops: Vec<(f32, Premul)>,
    },
    /// t = |p - center| / radius.
    Radial {
        center: (f32, f32),
        radius: f32,
        stops: Vec<(f32, Premul)>,
    },
    /// The image stretched over `rect` (x, y, width, height), sampled
    /// between its four nearest pixels and held at its edges.
    Image {
        rect: (f32, f32, f32, f32),
        image: Arc<CanvasImage>,
    },
}

/// The stops of a gradient, resolved, sorted by offset and cut to
/// [`MAX_STOPS`].
fn resolve_stops(stops: &[super::scene::GradientStop]) -> Vec<(f32, Premul)> {
    let mut resolved: Vec<(f32, Premul)> = stops
        .iter()
        .take(MAX_STOPS)
        .map(|stop| {
            (
                stop.offset.clamp(0.0, 1.0),
                premultiply(stop.color.resolve()),
            )
        })
        .collect();
    resolved.sort_by(|a, b| a.0.total_cmp(&b.0));
    resolved
}

impl Shader {
    /// `paint` drawn under `transform`, scene to picture.
    pub fn new(paint: &Paint, transform: &Transform) -> Self {
        let kind = match &paint.kind {
            PaintKind::Solid(color) => ShaderKind::Solid(premultiply(color.resolve())),
            PaintKind::Linear { start, end, stops } => ShaderKind::Linear {
                start: *start,
                axis: (end.0 - start.0, end.1 - start.1),
                stops: resolve_stops(stops),
            },
            PaintKind::Radial {
                center,
                radius,
                stops,
            } => ShaderKind::Radial {
                center: *center,
                radius: radius.max(1e-6),
                stops: resolve_stops(stops),
            },
        };
        Self {
            kind,
            inverse: transform.inverse().unwrap_or_default(),
            opacity: paint.opacity,
        }
    }

    /// `image` stretched over `rect` in the scene, drawn under `transform`.
    pub fn image(
        rect: (f32, f32, f32, f32),
        image: Arc<CanvasImage>,
        transform: &Transform,
    ) -> Self {
        Self {
            kind: ShaderKind::Image { rect, image },
            inverse: transform.inverse().unwrap_or_default(),
            opacity: 1.0,
        }
    }

    /// The color when the paint is one opaque color everywhere, which a
    /// renderer can write without blending where coverage is whole.
    pub fn opaque(&self) -> Option<Premul> {
        match self.kind {
            ShaderKind::Solid(color) if color[3] >= 1.0 && self.opacity >= 1.0 => Some(color),
            _ => None,
        }
    }

    /// The premultiplied color at picture pixel (`x`, `y`), its centre.
    pub fn at(&self, x: u32, y: u32) -> Premul {
        let color = match &self.kind {
            ShaderKind::Solid(color) => *color,
            ShaderKind::Image { rect, image } => {
                let p = self.inverse.apply((x as f32 + 0.5, y as f32 + 0.5));
                let u = (p.0 - rect.0) / rect.2 * image.width as f32 - 0.5;
                let v = (p.1 - rect.1) / rect.3 * image.height as f32 - 0.5;
                if !(u.is_finite() && v.is_finite()) {
                    return [0.0; 4];
                }
                let (u, v) = (
                    u.clamp(0.0, (image.width - 1) as f32),
                    v.clamp(0.0, (image.height - 1) as f32),
                );
                let (left, top) = (u.floor(), v.floor());
                let (fx, fy) = (u - left, v - top);
                let (left, top) = (left as u32, top as u32);
                let right = (left + 1).min(image.width - 1);
                let bottom = (top + 1).min(image.height - 1);
                let texel = |x: u32, y: u32| {
                    image.pixels[(y * image.width + x) as usize].map(|c| f32::from(c) / 255.0)
                };
                let (a, b, c, d) = (
                    texel(left, top),
                    texel(right, top),
                    texel(left, bottom),
                    texel(right, bottom),
                );
                std::array::from_fn(|i| {
                    let upper = a[i] + (b[i] - a[i]) * fx;
                    let lower = c[i] + (d[i] - c[i]) * fx;
                    upper + (lower - upper) * fy
                })
            }
            ShaderKind::Linear { start, axis, stops } => {
                let p = self.inverse.apply((x as f32 + 0.5, y as f32 + 0.5));
                let length = axis.0 * axis.0 + axis.1 * axis.1;
                let t = if length > 0.0 {
                    ((p.0 - start.0) * axis.0 + (p.1 - start.1) * axis.1) / length
                } else {
                    0.0
                };
                ramp(stops, t)
            }
            ShaderKind::Radial {
                center,
                radius,
                stops,
            } => {
                let p = self.inverse.apply((x as f32 + 0.5, y as f32 + 0.5));
                let t = ((p.0 - center.0).powi(2) + (p.1 - center.1).powi(2)).sqrt() / radius;
                ramp(stops, t)
            }
        };
        color.map(|c| c * self.opacity)
    }
}

/// The color at `t` along stops sorted by offset: flat before the first
/// and after the last, linear between.
pub(crate) fn ramp(stops: &[(f32, Premul)], t: f32) -> Premul {
    let t = t.clamp(0.0, 1.0);
    let Some(first) = stops.first() else {
        return [0.0; 4];
    };
    if t <= first.0 {
        return first.1;
    }
    for pair in stops.windows(2) {
        let ((t0, c0), (t1, c1)) = (pair[0], pair[1]);
        if t <= t1 {
            let f = if t1 > t0 { (t - t0) / (t1 - t0) } else { 1.0 };
            return std::array::from_fn(|i| c0[i] + (c1[i] - c0[i]) * f);
        }
    }
    stops[stops.len() - 1].1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::scene::{Color, GradientStop};

    #[test]
    fn a_linear_gradient_runs_from_its_first_stop_to_its_last() {
        let paint = Paint::linear(
            (0.0, 0.0),
            (10.0, 0.0),
            vec![
                GradientStop::new(0.0, Color::rgba(0, 0, 0, 255)),
                GradientStop::new(1.0, Color::rgba(255, 255, 255, 255)),
            ],
        );
        let shader = Shader::new(&paint, &Transform::identity());
        assert!((shader.at(0, 0)[0] - 0.05).abs() < 1e-6);
        assert!((shader.at(4, 0)[0] - 0.45).abs() < 1e-6);
        assert_eq!(shader.at(20, 0)[0], 1.0);
    }

    #[test]
    fn an_image_is_sampled_between_its_pixels_and_held_at_its_edges() {
        let image = Arc::new(
            CanvasImage::from_rgba(2, 1, vec![[0, 0, 0, 255], [255, 255, 255, 255]]).unwrap(),
        );
        let shader = Shader::image((0.0, 0.0, 8.0, 4.0), image, &Transform::identity());
        // The first image pixel's centre is at x = 2, the second's at x = 6.
        assert_eq!(shader.at(0, 0)[0], 0.0);
        assert!((shader.at(3, 0)[0] - 0.375).abs() < 1e-6);
        assert_eq!(shader.at(7, 0)[0], 1.0);
    }

    #[test]
    fn opacity_scales_every_channel() {
        let paint = Paint::solid(Color::rgba(255, 0, 0, 255)).opacity(0.5);
        assert_eq!(
            Shader::new(&paint, &Transform::identity()).at(0, 0),
            [0.5, 0.0, 0.0, 0.5]
        );
    }

    #[test]
    fn a_gradient_follows_the_transform() {
        let paint = Paint::radial(
            (0.0, 0.0),
            10.0,
            vec![
                GradientStop::new(0.0, Color::rgba(255, 255, 255, 255)),
                GradientStop::new(1.0, Color::rgba(0, 0, 0, 255)),
            ],
        );
        let moved = Shader::new(&paint, &Transform::translate(100.0, 100.0));
        // The centre moved with the transform: pixel (99, 99)'s centre is at
        // (-0.5, -0.5) in the scene.
        assert!(moved.at(99, 99)[0] > 0.9);
        assert_eq!(moved.at(0, 0)[0], 0.0);
    }
}
