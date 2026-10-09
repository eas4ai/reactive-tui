//! The Shader page's torus as a canvas scene: a ring of lit quads drawn
//! from the far side to the near one, for a picture of [`VIEW`] that the
//! canvas fits to its area.

use reactive_tui::graphics::{Color, GradientStop, Paint, Path, PathBuilder, Scene};
use std::time::Duration;

/// The size the scene is drawn for.
pub const VIEW: (f32, f32) = (320.0, 192.0);

/// Quads around the ring and around the tube.
const RING: usize = 56;
const TUBE: usize = 24;

type Vector = [f32; 3];

/// One quad of the surface: how far it is, its corners in the picture and
/// its color.
struct Quad {
    depth: f32,
    corners: [(f32, f32); 4],
    color: Color,
}

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

/// Where `point` is in the picture, seen from five units in front of it.
fn project([x, y, z]: Vector) -> (f32, f32) {
    let scale = 290.0 / (z + 5.0);
    (VIEW.0 / 2.0 + x * scale, VIEW.1 / 2.0 + y * scale)
}

/// The color at `along` of the way around the ring, from 0 to 1: fuchsia,
/// violet, cyan and back.
fn ring_color(along: f32) -> [f32; 3] {
    let stops = [
        [232.0, 80.0, 220.0],
        [140.0, 100.0, 255.0],
        [70.0, 210.0, 240.0],
        [140.0, 100.0, 255.0],
    ];
    let at = along.rem_euclid(1.0) * stops.len() as f32;
    let (from, to) = (at as usize % stops.len(), (at as usize + 1) % stops.len());
    let part = at.fract();
    [0, 1, 2].map(|i| stops[from][i] + (stops[to][i] - stops[from][i]) * part)
}

/// The torus `elapsed` after the page opened.
pub fn torus_scene(elapsed: Duration) -> Scene {
    use std::f32::consts::TAU;
    let seconds = elapsed.as_secs_f32();
    // The ring turns about its own axis while it leans and sways, so its
    // hole stays in sight.
    let spin = seconds * 0.8;
    let (angle_x, angle_y) = (
        1.05 + 0.3 * (seconds * 0.7).sin(),
        0.5 * (seconds * 0.45).sin(),
    );
    let (major, minor) = (1.3, 0.48);
    // A point of the surface and the direction the surface faces there.
    let surface = |ring: f32, tube: f32| -> (Vector, Vector) {
        let (sr, cr) = (ring * TAU + spin).sin_cos();
        let (st, ct) = (tube * TAU).sin_cos();
        let reach = major + minor * ct;
        (
            turn([reach * cr, reach * sr, minor * st], angle_x, angle_y),
            turn([ct * cr, ct * sr, st], angle_x, angle_y),
        )
    };
    let light = [-0.4, -0.75, -0.55];
    let light_length = dot(light, light).sqrt();
    let mut quads: Vec<Quad> = Vec::with_capacity(RING * TUBE);
    for ring in 0..RING {
        for tube in 0..TUBE {
            let at = |dr: usize, dt: usize| {
                surface(
                    (ring + dr) as f32 / RING as f32,
                    (tube + dt) as f32 / TUBE as f32,
                )
            };
            let (centre, normal) = surface(
                (ring as f32 + 0.5) / RING as f32,
                (tube as f32 + 0.5) / TUBE as f32,
            );
            let to_eye = [-centre[0], -centre[1], -5.0 - centre[2]];
            let distance = dot(to_eye, to_eye).sqrt();
            let facing = dot(normal, to_eye) / distance;
            if facing <= 0.0 {
                continue;
            }
            let diffuse = (dot(normal, light) / light_length).max(0.0);
            // Brighter where the surface turns away from the eye.
            let rim = (1.0 - facing).powi(3);
            let lit = 0.22 + 0.7 * diffuse + 0.5 * rim;
            let highlight = diffuse.powi(24) * 150.0;
            let base = ring_color((ring as f32 + 0.5) / RING as f32);
            let channel = |value: f32| (value * lit + highlight).clamp(0.0, 255.0) as u8;
            let color = Color::rgba(channel(base[0]), channel(base[1]), channel(base[2]), 255);
            let corners = [at(0, 0).0, at(1, 0).0, at(1, 1).0, at(0, 1).0].map(project);
            // Each quad is drawn a little larger than it is, so no
            // background shows between it and its neighbours.
            let middle = project(centre);
            let corners = corners.map(|(x, y)| {
                let (dx, dy) = (x - middle.0, y - middle.1);
                let length = (dx * dx + dy * dy).sqrt().max(0.001);
                (x + dx / length * 0.6, y + dy / length * 0.6)
            });
            quads.push(Quad {
                depth: centre[2],
                corners,
                color,
            });
        }
    }
    // The far quads first, so the near ones cover them.
    quads.sort_by(|a, b| b.depth.total_cmp(&a.depth));
    let mut scene = Scene::new();
    // The background reaches past the view on every side, so it also
    // fills what the canvas's area has beyond the fitted view.
    scene.fill(
        &Path::rect(-2.0 * VIEW.0, -2.0 * VIEW.1, 5.0 * VIEW.0, 5.0 * VIEW.1),
        &Paint::radial(
            (VIEW.0 / 2.0, VIEW.1 / 2.0),
            210.0,
            vec![
                GradientStop::new(0.0, Color::rgba(36, 22, 54, 255)),
                GradientStop::new(1.0, Color::rgba(6, 5, 12, 255)),
            ],
        ),
    );
    for quad in quads {
        let mut outline = PathBuilder::new().move_to(quad.corners[0].0, quad.corners[0].1);
        for corner in &quad.corners[1..] {
            outline = outline.line_to(corner.0, corner.1);
        }
        scene.fill(&outline.close().build(), &Paint::solid(quad.color));
    }
    scene
}
