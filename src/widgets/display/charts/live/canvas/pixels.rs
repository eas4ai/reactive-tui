//! The plot as a picture (CHT-037): the scene a cartesian chart's plot is
//! drawn from on the canvas's drawing thread. The chart worker builds it
//! here in the terminal's pixels, with the chart's top left corner at the
//! origin and everything clipped to the inner plot rectangle; the canvas
//! that shows it sits on that rectangle, so the scene is moved to start
//! there. Colors are the ones the cells use, resolved by the chart worker.

use super::super::super::plot::{Curve, Rgba};
use crate::graphics::{
    Color, GradientStop, LineCap, LineJoin, Paint, Path, PathBuilder, Scene, Stroke, Transform,
};
use std::sync::Arc;

/// The scene of one plot picture, in the chart's pixels.
pub(in super::super) struct PlotScene {
    scene: Scene,
    /// Pixels per cell.
    cell: (f32, f32),
    /// Whether the inner rectangle has been begun: the picture's origin and
    /// clip are in force.
    begun: bool,
}

/// `color` as the scene draws it, its alpha scaled by `alpha`.
fn color(color: Rgba, alpha: f32) -> Color {
    let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
    Color::rgba(
        channel(color.0),
        channel(color.1),
        channel(color.2),
        channel(color.3 * alpha),
    )
}

/// A solid paint of `color` at `alpha`.
fn solid(color_: Rgba, alpha: f32) -> Paint {
    Paint::solid(color(color_, alpha))
}

impl PlotScene {
    /// An empty scene for cells of `cell` pixels.
    pub fn new(cell: (u16, u16)) -> Self {
        Self {
            scene: Scene::new(),
            cell: (f32::from(cell.0.max(1)), f32::from(cell.1.max(1))),
            begun: false,
        }
    }

    /// Pixels per cell, as the plot's scales measure.
    pub fn units(&self) -> (f64, f64) {
        (f64::from(self.cell.0), f64::from(self.cell.1))
    }

    /// The cell height in pixels, which the sizes of strokes, markers and
    /// hover marks follow (CHT-012, CHT-038).
    pub fn cell_height(&self) -> f32 {
        self.cell.1
    }

    /// A stroke an eighth of the cell height wide, at least one pixel, with
    /// round joins and caps (CHT-012).
    pub fn stroke_width(&self) -> f32 {
        (self.cell.1 / 8.0).max(1.0)
    }

    /// The picture starts at the inner plot rectangle `inner`, given in
    /// pixels as (x, y, width, height), and shows nothing outside it.
    pub fn begin(&mut self, inner: (f32, f32, f32, f32)) {
        self.scene
            .push_transform(Transform::translate(-inner.0, -inner.1));
        self.scene
            .push_clip(&Path::rect(inner.0, inner.1, inner.2, inner.3));
        self.begun = true;
    }

    /// The finished scene.
    pub fn finish(mut self) -> Arc<Scene> {
        if self.begun {
            self.scene.pop_clip();
            self.scene.pop_transform();
        }
        Arc::new(self.scene)
    }

    /// A rectangle between two corners, with its corners rounded by
    /// `radius` pixels.
    pub fn rect(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, paint: &Paint, radius: f32) {
        let (left, right) = (x0.min(x1) as f32, x0.max(x1) as f32);
        let (top, bottom) = (y0.min(y1) as f32, y0.max(y1) as f32);
        let (width, height) = (right - left, bottom - top);
        if width <= 0.0 || height <= 0.0 {
            return;
        }
        let path = if radius > 0.0 {
            Path::rounded_rect(left, top, width, height, radius)
        } else {
            Path::rect(left, top, width, height)
        };
        self.scene.fill(&path, paint);
    }

    /// A solid rectangle of `tint` at `alpha`.
    #[allow(clippy::too_many_arguments)]
    pub fn solid_rect(
        &mut self,
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
        tint: Rgba,
        alpha: f32,
        radius: f32,
    ) {
        self.rect(x0, y0, x1, y1, &solid(tint, alpha), radius);
    }

    /// A rectangle filled with a gradient from `from` to `to` through
    /// `stops` (offsets along it, 0 at `from`), at `alpha` (CHT-013).
    #[allow(clippy::too_many_arguments)]
    pub fn gradient_rect(
        &mut self,
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
        stops: &[(f32, Rgba)],
        from: (f64, f64),
        to: (f64, f64),
        alpha: f32,
        radius: f32,
    ) {
        let stops: Vec<GradientStop> = stops
            .iter()
            .map(|(offset, stop)| GradientStop::new(*offset, color(*stop, alpha)))
            .collect();
        let paint = Paint::linear(
            (from.0 as f32, from.1 as f32),
            (to.0 as f32, to.1 as f32),
            stops,
        );
        self.rect(x0, y0, x1, y1, &paint, radius);
    }

    /// The path through `points` following `curve`: a cubic per pair of
    /// points for a natural curve, a run and a riser for a step, a
    /// straight segment otherwise. `reversed` walks the points backwards,
    /// so a step keeps its shape. One segment per pair of points keeps the
    /// two points a pixel column is thinned to (CHT-027).
    fn append_curve(
        mut path: PathBuilder,
        points: &[(f64, f64)],
        curve: Curve,
        reversed: bool,
    ) -> PathBuilder {
        let n = points.len();
        match curve {
            Curve::Linear => {
                for point in &points[1..] {
                    path = path.line_to(point.0 as f32, point.1 as f32);
                }
            }
            Curve::StepAfter => {
                for pair in points.windows(2) {
                    let (at, next) = (pair[0], pair[1]);
                    path = if reversed {
                        path.line_to(at.0 as f32, next.1 as f32)
                    } else {
                        path.line_to(next.0 as f32, at.1 as f32)
                    };
                    path = path.line_to(next.0 as f32, next.1 as f32);
                }
            }
            Curve::Natural => {
                for i in 0..n.saturating_sub(1) {
                    let p0 = if i == 0 { points[0] } else { points[i - 1] };
                    let p1 = points[i];
                    let p2 = points[i + 1];
                    let p3 = if i + 2 < n {
                        points[i + 2]
                    } else {
                        points[n - 1]
                    };
                    let c1 = (p1.0 + (p2.0 - p0.0) / 6.0, p1.1 + (p2.1 - p0.1) / 6.0);
                    let c2 = (p2.0 - (p3.0 - p1.0) / 6.0, p2.1 - (p3.1 - p1.1) / 6.0);
                    path = path.cubic_to(
                        c1.0 as f32,
                        c1.1 as f32,
                        c2.0 as f32,
                        c2.1 as f32,
                        p2.0 as f32,
                        p2.1 as f32,
                    );
                }
            }
        }
        path
    }

    /// A line through `points` following `curve`, `width` pixels wide,
    /// dashed by `dash` when given.
    pub fn polyline(
        &mut self,
        points: &[(f64, f64)],
        curve: Curve,
        tint: Rgba,
        width: f32,
        dash: Option<&[f32]>,
    ) {
        if points.len() < 2 {
            return;
        }
        let path = PathBuilder::new().move_to(points[0].0 as f32, points[0].1 as f32);
        let path = Self::append_curve(path, points, curve, false);
        let mut stroke = Stroke::new(width).join(LineJoin::Round).cap(LineCap::Round);
        if let Some(dash) = dash {
            stroke = stroke.dash(dash);
        }
        self.scene.stroke(&path.build(), &stroke, &solid(tint, 1.0));
    }

    /// A one-pixel line from `from` to `to`, dashed when `dash` is given,
    /// at `alpha`: a grid, reference or crosshair line.
    pub fn line(
        &mut self,
        from: (f64, f64),
        to: (f64, f64),
        tint: Rgba,
        dash: Option<&[f32]>,
        alpha: f32,
    ) {
        // An axis-aligned line one pixel wide sits on a pixel's centre, so
        // it fills one column or row instead of two half-covered ones.
        let crisp = |v: f64| v.floor() + 0.5;
        let (mut from, mut to) = (from, to);
        if from.0 == to.0 {
            from.0 = crisp(from.0);
            to.0 = from.0;
        }
        if from.1 == to.1 {
            from.1 = crisp(from.1);
            to.1 = from.1;
        }
        let path = PathBuilder::new()
            .move_to(from.0 as f32, from.1 as f32)
            .line_to(to.0 as f32, to.1 as f32)
            .build();
        let mut stroke = Stroke::new(1.0);
        if let Some(dash) = dash {
            stroke = stroke.dash(dash);
        }
        self.scene.stroke(&path, &stroke, &solid(tint, alpha));
    }

    /// The area between the `top` and `bottom` lines following `curve`,
    /// as a closed path: along the top, down to the bottom's end and back
    /// along the bottom.
    fn area(top: &[(f64, f64)], bottom: &[(f64, f64)], curve: Curve) -> Option<Path> {
        let first = top.first()?;
        let path = PathBuilder::new().move_to(first.0 as f32, first.1 as f32);
        let mut path = Self::append_curve(path, top, curve, false);
        let back: Vec<(f64, f64)> = bottom.iter().rev().copied().collect();
        if let Some(start) = back.first() {
            path = path.line_to(start.0 as f32, start.1 as f32);
            path = Self::append_curve(path, &back, curve, true);
        }
        Some(path.close().build())
    }

    /// The area between `top` and `bottom` filled with `tint` at `opacity`
    /// (CHT-012).
    pub fn fill_between(
        &mut self,
        top: &[(f64, f64)],
        bottom: &[(f64, f64)],
        curve: Curve,
        tint: Rgba,
        opacity: f32,
    ) {
        if let Some(path) = Self::area(top, bottom, curve) {
            self.scene.fill(&path, &solid(tint, opacity));
        }
    }

    /// The area between `top` and `bottom` filled with a gradient of `tint`:
    /// `opacity` at the highest point of the stroke, transparent at the
    /// baseline (CHT-012).
    pub fn gradient_between(
        &mut self,
        top: &[(f64, f64)],
        bottom: &[(f64, f64)],
        curve: Curve,
        tint: Rgba,
        opacity: f32,
    ) {
        let Some(path) = Self::area(top, bottom, curve) else {
            return;
        };
        let highest = top.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
        let baseline = bottom.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
        if !(highest.is_finite() && baseline.is_finite()) || baseline <= highest {
            self.scene.fill(&path, &solid(tint, opacity));
            return;
        }
        let paint = Paint::linear(
            (0.0, highest as f32),
            (0.0, baseline as f32),
            vec![
                GradientStop::new(0.0, color(tint, opacity)),
                GradientStop::new(1.0, color(tint, 0.0)),
            ],
        );
        self.scene.fill(&path, &paint);
    }

    /// Clip what follows to the area between `top` and `bottom`, for a
    /// pattern drawn over it; `end_clip` undoes it.
    pub fn clip_between(
        &mut self,
        top: &[(f64, f64)],
        bottom: &[(f64, f64)],
        curve: Curve,
    ) -> bool {
        match Self::area(top, bottom, curve) {
            Some(path) => {
                self.scene.push_clip(&path);
                true
            }
            None => false,
        }
    }

    pub fn end_clip(&mut self) {
        self.scene.pop_clip();
    }

    /// A filled disc of `radius` pixels around (`x`, `y`) at `alpha`.
    pub fn disc(&mut self, x: f64, y: f64, radius: f32, tint: Rgba, alpha: f32) {
        if radius <= 0.0 {
            return;
        }
        let path = Path::ellipse(x as f32, y as f32, radius, radius);
        self.scene.fill(&path, &solid(tint, alpha));
    }

    /// A one-pixel ring of `radius` pixels around (`x`, `y`).
    pub fn ring(&mut self, x: f64, y: f64, radius: f32, tint: Rgba, alpha: f32) {
        if radius <= 0.0 {
            return;
        }
        let path = Path::ellipse(x as f32, y as f32, radius, radius);
        self.scene
            .stroke(&path, &Stroke::new(1.0), &solid(tint, alpha));
    }

    /// `text` with its baseline at (`x`, `baseline`), `size` pixels tall:
    /// the tile glyph of a pattern fill.
    pub fn text(&mut self, x: f64, baseline: f64, size: f32, text: &str, tint: Rgba) {
        self.scene
            .text((x as f32, baseline as f32), size, text, &solid(tint, 1.0));
    }
}
