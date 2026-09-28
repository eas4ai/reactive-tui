//! Shared by the canvas tests (docs/spec/canvas.md, GFX-001 to GFX-008): the
//! reference scenes, the same-picture measure of GFX-002 and the reference
//! images in tests/snapshots/canvas.
#![allow(dead_code)]

use reactive_tui::core::surface::Rgba;
use reactive_tui::graphics::{
    fonts::FontSource, CanvasImage, Color, GradientStop, GraphicsFrame, GraphicsOptions, LineCap,
    LineJoin, Paint, Path, PathBuilder, Scene, Stroke, Transform,
};
use reactive_tui::layout::CellGrid;
use std::path::PathBuf;
use std::sync::Arc;

/// A reference scene: one feature of GFX-001, drawn at `size` pixels.
pub struct Reference {
    pub name: &'static str,
    pub scene: Scene,
    pub size: (u32, u32),
}

/// The renderer options the reference scenes are drawn with: the bundled
/// font, so a reference is the same on every host (GFX-001).
pub fn reference_options(force_cpu: bool) -> GraphicsOptions {
    GraphicsOptions {
        force_cpu,
        font: FontSource::Bundled,
        ..Default::default()
    }
}

/// 40 by 12 cells of 8 by 16 pixels.
pub const SIZE: (u32, u32) = (320, 192);

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::rgba(r, g, b, 255)
}

/// Every reference scene, one per feature GFX-001 lists.
pub fn references() -> Vec<Reference> {
    vec![
        Reference {
            name: "strokes",
            scene: strokes(),
            size: SIZE,
        },
        Reference {
            name: "curves",
            scene: curves(),
            size: SIZE,
        },
        Reference {
            name: "shapes",
            scene: shapes(),
            size: SIZE,
        },
        Reference {
            name: "gradients",
            scene: gradients(),
            size: SIZE,
        },
        Reference {
            name: "image",
            scene: image(),
            size: SIZE,
        },
        Reference {
            name: "text",
            scene: text(),
            size: SIZE,
        },
        Reference {
            name: "cells",
            scene: cells(),
            size: SIZE,
        },
        Reference {
            name: "transforms-and-clips",
            scene: transforms_and_clips(),
            size: SIZE,
        },
        Reference {
            name: "cube",
            scene: cube(0.6, 0.4),
            size: SIZE,
        },
    ]
}

/// Strokes of three widths with each join, cap and a dash pattern.
pub fn strokes() -> Scene {
    let mut scene = Scene::new();
    scene.fill(
        &Path::rect(0.0, 0.0, 320.0, 192.0),
        &Paint::solid(rgb(16, 18, 24)),
    );
    let zigzag = |y: f32| {
        PathBuilder::new()
            .move_to(20.0, y + 30.0)
            .line_to(60.0, y)
            .line_to(100.0, y + 30.0)
            .build()
    };
    for (index, (join, cap)) in [
        (LineJoin::Miter, LineCap::Butt),
        (LineJoin::Round, LineCap::Round),
        (LineJoin::Bevel, LineCap::Square),
    ]
    .into_iter()
    .enumerate()
    {
        let y = 16.0 + 56.0 * index as f32;
        for (column, width) in [1.0, 3.0, 8.0].into_iter().enumerate() {
            let shift = Transform::translate(100.0 * column as f32, 0.0);
            scene.push_transform(shift);
            scene.stroke(
                &zigzag(y),
                &Stroke::new(width).join(join).cap(cap),
                &Paint::solid(rgb(230, 230, 240)),
            );
            scene.pop_transform();
        }
    }
    scene.stroke(
        &PathBuilder::new()
            .move_to(20.0, 180.0)
            .line_to(300.0, 180.0)
            .build(),
        &Stroke::new(4.0).dash(&[12.0, 6.0]),
        &Paint::solid(rgb(250, 200, 60)),
    );
    scene
}

/// Quadratic and cubic curves and elliptical arcs, stroked and filled.
pub fn curves() -> Scene {
    let mut scene = Scene::new();
    scene.fill(
        &Path::rect(0.0, 0.0, 320.0, 192.0),
        &Paint::solid(rgb(250, 250, 252)),
    );
    let quad = PathBuilder::new()
        .move_to(16.0, 160.0)
        .quad_to(80.0, 8.0, 150.0, 160.0)
        .build();
    scene.stroke(&quad, &Stroke::new(3.0), &Paint::solid(rgb(30, 90, 200)));
    let cubic = PathBuilder::new()
        .move_to(170.0, 170.0)
        .cubic_to(180.0, 10.0, 300.0, 200.0, 305.0, 20.0)
        .build();
    scene.stroke(&cubic, &Stroke::new(2.0), &Paint::solid(rgb(200, 40, 60)));
    let lens = PathBuilder::new()
        .move_to(110.0, 96.0)
        .arc_to(60.0, 30.0, 20.0, false, true, 230.0, 96.0)
        .arc_to(60.0, 30.0, 20.0, false, true, 110.0, 96.0)
        .close()
        .build();
    scene.fill(&lens, &Paint::solid(rgb(40, 170, 90)).opacity(0.7));
    scene
}

/// Rectangles, rounded rectangles and ellipses, filled and outlined.
pub fn shapes() -> Scene {
    let mut scene = Scene::new();
    scene.fill(
        &Path::rect(0.0, 0.0, 320.0, 192.0),
        &Paint::solid(rgb(20, 24, 32)),
    );
    scene.fill(
        &Path::rect(16.0, 16.0, 80.0, 60.0),
        &Paint::solid(rgb(220, 60, 60)),
    );
    scene.fill(
        &Path::rounded_rect(116.0, 16.0, 90.0, 60.0, 14.0),
        &Paint::solid(rgb(60, 180, 220)),
    );
    scene.fill(
        &Path::ellipse(264.0, 46.0, 40.0, 28.0),
        &Paint::solid(rgb(240, 200, 60)),
    );
    scene.stroke(
        &Path::rect(16.0, 104.0, 80.0, 60.0),
        &Stroke::new(3.0),
        &Paint::solid(rgb(220, 60, 60)),
    );
    scene.stroke(
        &Path::rounded_rect(116.0, 104.0, 90.0, 60.0, 14.0),
        &Stroke::new(3.0),
        &Paint::solid(rgb(60, 180, 220)),
    );
    scene.stroke(
        &Path::ellipse(264.0, 134.0, 40.0, 28.0),
        &Stroke::new(3.0),
        &Paint::solid(rgb(240, 200, 60)),
    );
    scene
}

/// Linear and radial gradients, with opacity and a theme token.
pub fn gradients() -> Scene {
    let mut scene = Scene::new();
    scene.fill(
        &Path::rect(0.0, 0.0, 320.0, 96.0),
        &Paint::linear(
            (0.0, 0.0),
            (320.0, 0.0),
            vec![
                GradientStop::new(0.0, rgb(255, 0, 80)),
                GradientStop::new(0.5, Color::token("blue-500")),
                GradientStop::new(1.0, rgb(0, 220, 120)),
            ],
        ),
    );
    scene.fill(
        &Path::rect(0.0, 96.0, 320.0, 96.0),
        &Paint::radial(
            (160.0, 144.0),
            120.0,
            vec![
                GradientStop::new(0.0, rgb(255, 255, 255)),
                GradientStop::new(1.0, rgb(10, 10, 40)),
            ],
        ),
    );
    scene.fill(
        &Path::ellipse(160.0, 96.0, 70.0, 40.0),
        &Paint::solid(rgb(255, 200, 0)).opacity(0.5),
    );
    scene
}

/// An 8 by 8 checker image drawn scaled into a rectangle.
pub fn image() -> Scene {
    let mut pixels = Vec::with_capacity(64);
    for y in 0..8u8 {
        for x in 0..8u8 {
            pixels.push(if (x + y) % 2 == 0 {
                [240, 240, 240, 255]
            } else {
                [30, 60, 200, 255]
            });
        }
    }
    let checker = Arc::new(CanvasImage::from_rgba(8, 8, pixels).expect("an 8 by 8 image"));
    let mut scene = Scene::new();
    scene.fill(
        &Path::rect(0.0, 0.0, 320.0, 192.0),
        &Paint::solid(rgb(0, 0, 0)),
    );
    scene.image((40.0, 16.0, 240.0, 160.0), checker);
    scene
}

/// Text in the bundled monospace font at two sizes.
pub fn text() -> Scene {
    let mut scene = Scene::new();
    scene.fill(
        &Path::rect(0.0, 0.0, 320.0, 192.0),
        &Paint::solid(rgb(12, 12, 16)),
    );
    scene.text(
        (12.0, 40.0),
        24.0,
        "Reactive TUI",
        &Paint::solid(rgb(240, 240, 240)),
    );
    scene.text(
        (12.0, 96.0),
        14.0,
        "canvas text 0123456789",
        &Paint::solid(rgb(120, 200, 255)),
    );
    scene
}

/// A CellGrid of box drawing, blocks and letters, drawn through the glyph atlas.
pub fn cells() -> Scene {
    let mut grid = CellGrid::new(20, 6);
    let fg = Rgba::new(0.9, 0.9, 0.95, 1.0);
    for x in 0..20 {
        grid.set(x, 0, "─", Some(fg));
        grid.set(x, 5, "─", Some(fg));
    }
    for (x, glyph) in "cells ▀▄█ ab".chars().enumerate() {
        grid.set(
            2 + x as u16,
            2,
            &glyph.to_string(),
            Some(Rgba::new(1.0, 0.8, 0.2, 1.0)),
        );
    }
    let mut scene = Scene::new();
    scene.fill(
        &Path::rect(0.0, 0.0, 320.0, 192.0),
        &Paint::solid(rgb(0, 0, 0)),
    );
    scene.cells((0.0, 0.0), Arc::new(grid), (8, 16));
    scene
}

/// Nested transforms and a clip.
pub fn transforms_and_clips() -> Scene {
    let mut scene = Scene::new();
    scene.fill(
        &Path::rect(0.0, 0.0, 320.0, 192.0),
        &Paint::solid(rgb(245, 245, 245)),
    );
    scene.push_transform(Transform::translate(160.0, 96.0));
    for step in 0..6 {
        scene.push_transform(Transform::rotate(step as f32 * 30.0));
        scene.fill(
            &Path::rect(0.0, -6.0, 80.0, 12.0),
            &Paint::solid(rgb(40 * step as u8, 80, 200)),
        );
        scene.pop_transform();
    }
    scene.pop_transform();
    scene.push_clip(&Path::ellipse(60.0, 96.0, 40.0, 40.0));
    scene.fill(
        &Path::rect(0.0, 0.0, 120.0, 192.0),
        &Paint::solid(rgb(220, 40, 40)),
    );
    scene.pop_clip();
    scene
}

/// The demo cube at two angles, as filled faces with flat shading, far
/// faces first.
pub fn cube(angle_x: f32, angle_y: f32) -> Scene {
    let corners: Vec<[f32; 3]> = (0..8)
        .map(|i| {
            [
                if i & 1 == 0 { -1.0 } else { 1.0 },
                if i & 2 == 0 { -1.0 } else { 1.0 },
                if i & 4 == 0 { -1.0 } else { 1.0 },
            ]
        })
        .collect();
    let rotate = |[x, y, z]: [f32; 3]| {
        let (sx, cx) = angle_x.sin_cos();
        let (sy, cy) = angle_y.sin_cos();
        let (y, z) = (y * cx - z * sx, y * sx + z * cx);
        let (x, z) = (x * cy + z * sy, -x * sy + z * cy);
        [x, y, z]
    };
    let rotated: Vec<[f32; 3]> = corners.into_iter().map(rotate).collect();
    let project = |[x, y, z]: [f32; 3]| {
        let scale = 70.0 / (z + 4.0) * 4.0;
        (160.0 + x * scale * 1.0, 96.0 + y * scale * 0.9)
    };
    let faces: [[usize; 4]; 6] = [
        [0, 1, 3, 2],
        [4, 5, 7, 6],
        [0, 1, 5, 4],
        [2, 3, 7, 6],
        [0, 2, 6, 4],
        [1, 3, 7, 5],
    ];
    let mut ordered: Vec<([usize; 4], f32)> = faces
        .iter()
        .map(|face| {
            (
                *face,
                face.iter().map(|&i| rotated[i][2]).sum::<f32>() / 4.0,
            )
        })
        .collect();
    ordered.sort_by(|a, b| b.1.total_cmp(&a.1));
    let mut scene = Scene::new();
    scene.fill(
        &Path::rect(0.0, 0.0, 320.0, 192.0),
        &Paint::radial(
            (160.0, 96.0),
            180.0,
            vec![
                GradientStop::new(0.0, rgb(40, 44, 70)),
                GradientStop::new(1.0, rgb(8, 8, 14)),
            ],
        ),
    );
    for (face, depth) in ordered {
        let mut path = PathBuilder::new();
        for (n, &corner) in face.iter().enumerate() {
            let (x, y) = project(rotated[corner]);
            path = if n == 0 {
                path.move_to(x, y)
            } else {
                path.line_to(x, y)
            };
        }
        let light = (0.55 - depth * 0.2).clamp(0.2, 1.0);
        let shade = (light * 255.0) as u8;
        scene.fill(
            &path.close().build(),
            &Paint::solid(rgb(shade / 2, shade, 255)),
        );
    }
    scene
}

/// Why `a` and `b` are not the same picture (GFX-002), or `None` when they
/// are: over the whole picture the mean difference per channel is at most 1
/// of 255, and no block of 8 by 16 pixels differs in mean color by more
/// than 8 of 255 in any channel.
pub fn difference(a: &GraphicsFrame, b: &GraphicsFrame) -> Option<String> {
    if (a.width(), a.height()) != (b.width(), b.height()) {
        return Some(format!(
            "sizes differ: {}x{} and {}x{}",
            a.width(),
            a.height(),
            b.width(),
            b.height()
        ));
    }
    let (width, height) = (a.width() as usize, a.height() as usize);
    let mut total = [0f64; 4];
    for (p, q) in a.pixels().iter().zip(b.pixels()) {
        for channel in 0..4 {
            total[channel] += (p[channel] as f64 - q[channel] as f64).abs();
        }
    }
    let count = (width * height).max(1) as f64;
    let mean = total.map(|sum| sum / count);
    if let Some(channel) = (0..4).find(|&c| mean[c] > 1.0) {
        return Some(format!(
            "channel {channel} differs by {:.2} of 255 on average",
            mean[channel]
        ));
    }
    for block_y in (0..height).step_by(16) {
        for block_x in (0..width).step_by(8) {
            let mut sums = [[0f64; 4]; 2];
            let mut pixels = 0f64;
            for y in block_y..(block_y + 16).min(height) {
                for x in block_x..(block_x + 8).min(width) {
                    let index = y * width + x;
                    for channel in 0..4 {
                        sums[0][channel] += a.pixels()[index][channel] as f64;
                        sums[1][channel] += b.pixels()[index][channel] as f64;
                    }
                    pixels += 1.0;
                }
            }
            for channel in 0..4 {
                let gap = (sums[0][channel] - sums[1][channel]).abs() / pixels;
                if gap > 8.0 {
                    return Some(format!(
                        "the 8 by 16 block at ({block_x}, {block_y}) differs by {gap:.1} of 255 in channel {channel}"
                    ));
                }
            }
        }
    }
    None
}

/// The checked-in reference image of scene `name`.
pub fn reference_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/snapshots/canvas")
        .join(format!("{name}.png"))
}

/// Compares `frame` with the reference image of `name`, or writes it when
/// REGENERATE=1 is set (BAR-004's rule), so a reference is never made by a
/// comparison.
pub fn check_reference(name: &str, frame: &GraphicsFrame) -> Option<String> {
    let path = reference_path(name);
    if std::env::var("REGENERATE").as_deref() == Ok("1") {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let bytes: Vec<u8> = frame.pixels().iter().flatten().copied().collect();
        image::save_buffer(
            &path,
            &bytes,
            frame.width(),
            frame.height(),
            image::ColorType::Rgba8,
        )
        .unwrap();
        return None;
    }
    let reference = match image::open(&path) {
        Ok(reference) => reference.to_rgba8(),
        Err(error) => return Some(format!("no reference image {}: {error}", path.display())),
    };
    let pixels = reference.pixels().map(|pixel| pixel.0).collect();
    let reference =
        GraphicsFrame::from_rgba(reference.width(), reference.height(), pixels).unwrap();
    difference(frame, &reference)
}
