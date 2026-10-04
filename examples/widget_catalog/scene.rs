//! The Motion page's cube as a canvas scene: three lit faces, their edges
//! and a shadow, drawn for a picture of [`VIEW`] that the canvas fits to
//! its area.

use reactive_tui::graphics::{
    Color, GradientStop, LineJoin, Paint, Path, PathBuilder, Scene, Stroke,
};
use std::time::Duration;

/// The size the scene is drawn for.
pub const VIEW: (f32, f32) = (320.0, 192.0);

type Vector = [f32; 3];

fn dot(a: Vector, b: Vector) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// `point` turned by `x` around the x axis, then by `y` around the y axis.
fn turn(point: Vector, x: f32, y: f32) -> Vector {
    let (sx, cx) = x.sin_cos();
    let (sy, cy) = y.sin_cos();
    let [px, py, pz] = point;
    let (py, pz) = (py * cx - pz * sx, py * sx + pz * cx);
    [px * cy + pz * sy, py, -px * sy + pz * cy]
}

/// Where `point` is in the picture, seen from four units in front of it.
fn project([x, y, z]: Vector) -> (f32, f32) {
    let scale = 168.0 / (z + 4.0);
    (VIEW.0 / 2.0 + x * scale, VIEW.1 / 2.0 - 6.0 + y * scale)
}

fn shade(color: [f32; 3], light: f32) -> Color {
    let channel = |value: f32| (value * light).clamp(0.0, 255.0) as u8;
    Color::rgba(channel(color[0]), channel(color[1]), channel(color[2]), 255)
}

/// The cube `elapsed` after the page opened.
pub fn cube_scene(elapsed: Duration) -> Scene {
    let seconds = elapsed.as_secs_f32();
    let (angle_x, angle_y) = (0.35 + seconds * 0.44, seconds * 0.7);
    let mut scene = Scene::new();
    // The background reaches past the view on every side, so it also
    // fills what the canvas's area has beyond the fitted view.
    scene.fill(
        &Path::rect(-2.0 * VIEW.0, -2.0 * VIEW.1, 5.0 * VIEW.0, 5.0 * VIEW.1),
        &Paint::radial(
            (VIEW.0 / 2.0, VIEW.1 / 2.0 - 10.0),
            200.0,
            vec![
                GradientStop::new(0.0, Color::rgba(34, 40, 66, 255)),
                GradientStop::new(1.0, Color::rgba(7, 8, 14, 255)),
            ],
        ),
    );
    scene.fill(
        &Path::ellipse(VIEW.0 / 2.0, 172.0, 78.0, 9.0),
        &Paint::radial(
            (VIEW.0 / 2.0, 172.0),
            78.0,
            vec![
                GradientStop::new(0.0, Color::rgba(0, 0, 0, 150)),
                GradientStop::new(1.0, Color::rgba(0, 0, 0, 0)),
            ],
        ),
    );
    // Each face: its outward normal, its corners in order around it, and
    // its color.
    let faces: [(Vector, [Vector; 4], [f32; 3]); 6] = [
        (
            [0.0, 0.0, -1.0],
            [
                [-1., -1., -1.],
                [1., -1., -1.],
                [1., 1., -1.],
                [-1., 1., -1.],
            ],
            [92.0, 170.0, 255.0],
        ),
        (
            [0.0, 0.0, 1.0],
            [[-1., -1., 1.], [-1., 1., 1.], [1., 1., 1.], [1., -1., 1.]],
            [92.0, 170.0, 255.0],
        ),
        (
            [-1.0, 0.0, 0.0],
            [
                [-1., -1., -1.],
                [-1., 1., -1.],
                [-1., 1., 1.],
                [-1., -1., 1.],
            ],
            [140.0, 120.0, 255.0],
        ),
        (
            [1.0, 0.0, 0.0],
            [[1., -1., -1.], [1., -1., 1.], [1., 1., 1.], [1., 1., -1.]],
            [140.0, 120.0, 255.0],
        ),
        (
            [0.0, -1.0, 0.0],
            [
                [-1., -1., -1.],
                [-1., -1., 1.],
                [1., -1., 1.],
                [1., -1., -1.],
            ],
            [80.0, 220.0, 230.0],
        ),
        (
            [0.0, 1.0, 0.0],
            [[-1., 1., -1.], [1., 1., -1.], [1., 1., 1.], [-1., 1., 1.]],
            [80.0, 220.0, 230.0],
        ),
    ];
    // Light from the upper left, in front; the picture's y grows downwards.
    let light = [-0.45, -0.7, -0.55];
    for (normal, corners, color) in faces {
        let normal = turn(normal, angle_x, angle_y);
        let corners = corners.map(|corner| turn(corner, angle_x, angle_y));
        // A face is seen when it faces the eye, which looks along z.
        let centre = corners
            .iter()
            .fold([0.0; 3], |sum, c| {
                [sum[0] + c[0], sum[1] + c[1], sum[2] + c[2]]
            })
            .map(|sum| sum / 4.0);
        let to_eye = [-centre[0], -centre[1], -4.0 - centre[2]];
        if dot(normal, to_eye) <= 0.0 {
            continue;
        }
        let lit = 0.3 + 0.7 * dot(normal, light).max(0.0) / dot(light, light).sqrt();
        let points = corners.map(project);
        let mut outline = PathBuilder::new().move_to(points[0].0, points[0].1);
        for point in &points[1..] {
            outline = outline.line_to(point.0, point.1);
        }
        let outline = outline.close().build();
        scene.fill(
            &outline,
            &Paint::linear(
                points[0],
                points[2],
                vec![
                    GradientStop::new(0.0, shade(color, lit * 1.12)),
                    GradientStop::new(1.0, shade(color, lit * 0.8)),
                ],
            ),
        );
        scene.stroke(
            &outline,
            &Stroke::new(1.5).join(LineJoin::Round),
            &Paint::solid(shade([225.0, 240.0, 255.0], 0.55 + 0.45 * lit)),
        );
    }
    scene
}
