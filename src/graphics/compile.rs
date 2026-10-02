//! A scene turned into the draws both renderers carry out: transforms
//! applied, curves flattened, strokes expanded, colors resolved and text
//! laid out, once, so the hardware adapter and the software renderer start
//! from the same shapes (GFX-002).

use super::geometry::{fill_polygons, stroke_polygons, Polygon};
use super::glyphs::{Glyphs, Placed, Text};
use super::paint::{premultiply, Premul, Shader};
use super::raster::{window_of, Window};
use super::scene::{Command, Path, Scene, Transform};
use crate::layout::CellGrid;
use std::sync::Arc;

/// One step of drawing a picture, in picture pixels.
#[derive(Debug)]
pub(crate) enum Draw {
    /// Polygons filled together by the non-zero rule over `window`.
    Fill {
        polygons: Vec<Polygon>,
        window: Window,
        shader: Shader,
    },
    /// A rectangle with sides along the pixel grid, as (left, top, right,
    /// bottom): its coverage of a pixel is the overlap across times the
    /// overlap down, so it needs no polygon.
    Rect {
        rect: [f32; 4],
        shader: Shader,
    },
    /// Glyph bitmaps of one line of text.
    Glyphs {
        glyphs: Vec<Placed>,
        shader: Shader,
    },
    /// A cell grid with its top left corner at (`x`, `y`). A glyph whose
    /// cell names no color is drawn in `foreground`, premultiplied.
    Cells {
        x: i32,
        y: i32,
        grid: Arc<CellGrid>,
        cell: (u16, u16),
        foreground: Premul,
    },
    /// What follows shows only inside these polygons, within any clip
    /// already in force.
    PushClip {
        polygons: Vec<Polygon>,
        window: Window,
    },
    PopClip,
}

/// `polygons` as (left, top, right, bottom) when they are one rectangle
/// with its sides along the axes.
fn axis_rectangle(polygons: &[Polygon]) -> Option<[f32; 4]> {
    let [polygon] = polygons else {
        return None;
    };
    let &[a, b, c, d] = polygon.as_slice() else {
        return None;
    };
    let across_first = a.1 == b.1 && b.0 == c.0 && c.1 == d.1 && d.0 == a.0;
    let down_first = a.0 == b.0 && b.1 == c.1 && c.0 == d.0 && d.1 == a.1;
    (across_first || down_first).then(|| [a.0.min(c.0), a.1.min(c.1), a.0.max(c.0), a.1.max(c.1)])
}

/// The color of a cell that names none: `theme`'s text color, which a
/// theme that leaves it out takes from the light or the dark preset
/// (THM-002).
fn cell_foreground(theme: &crate::theme::Theme) -> Premul {
    theme
        .resolve_color("foreground")
        .map_or([0.0; 4], |(r, g, b, a)| premultiply([r, g, b, a]))
}

/// The draws of `scene` for a picture of `size` pixels, with `base` applied
/// to every scene coordinate.
pub(crate) fn compile(
    scene: &Scene,
    base: &Transform,
    size: (u32, u32),
    glyphs: &mut Glyphs,
) -> Vec<Draw> {
    let mut draws = Vec::with_capacity(scene.commands.len());
    let mut transforms = vec![*base];
    let foreground = cell_foreground(&crate::theme::Theme::active());
    // Whether anything can show inside each clip in force.
    let mut clips: Vec<bool> = Vec::new();
    let fill = |draws: &mut Vec<Draw>, polygons: Vec<Polygon>, shader: Shader| {
        let Some(window) = window_of(&polygons, size) else {
            return;
        };
        match axis_rectangle(&polygons) {
            Some(rect) => draws.push(Draw::Rect { rect, shader }),
            None => draws.push(Draw::Fill {
                polygons,
                window,
                shader,
            }),
        }
    };
    for command in &scene.commands {
        let transform = *transforms.last().unwrap_or(base);
        let hidden = clips.last().is_some_and(|shows| !shows);
        match command {
            Command::PushTransform(next) => transforms.push(next.then(transform)),
            Command::PopTransform => {
                if transforms.len() > 1 {
                    transforms.pop();
                }
            }
            Command::PushClip(path) => {
                let polygons = fill_polygons(path, &transform);
                match window_of(&polygons, size).filter(|_| !hidden) {
                    Some(window) => {
                        clips.push(true);
                        draws.push(Draw::PushClip { polygons, window });
                    }
                    None => clips.push(false),
                }
            }
            Command::PopClip => {
                if clips.pop() == Some(true) {
                    draws.push(Draw::PopClip);
                }
            }
            _ if hidden => {}
            Command::Fill(path, paint) => fill(
                &mut draws,
                fill_polygons(path, &transform),
                Shader::new(paint, &transform),
            ),
            Command::Stroke(path, stroke, paint) => fill(
                &mut draws,
                stroke_polygons(path, stroke, &transform),
                Shader::new(paint, &transform),
            ),
            Command::Image(rect, image) => {
                if rect.2 > 0.0 && rect.3 > 0.0 {
                    fill(
                        &mut draws,
                        fill_polygons(&Path::rect(rect.0, rect.1, rect.2, rect.3), &transform),
                        Shader::image(*rect, image.clone(), &transform),
                    );
                }
            }
            Command::Text(origin, text_size, text, paint) => {
                let shader = Shader::new(paint, &transform);
                match glyphs.text(*origin, *text_size, text, &transform) {
                    Text::Bitmaps(placed) => {
                        let placed: Vec<Placed> = placed
                            .into_iter()
                            .filter(|glyph| {
                                glyph.x < size.0 as i32
                                    && glyph.y < size.1 as i32
                                    && glyph.x + glyph.bitmap.width as i32 > 0
                                    && glyph.y + glyph.bitmap.height as i32 > 0
                            })
                            .collect();
                        if !placed.is_empty() {
                            draws.push(Draw::Glyphs {
                                glyphs: placed,
                                shader,
                            });
                        }
                    }
                    Text::Outlines(polygons) => fill(&mut draws, polygons, shader),
                }
            }
            Command::Cells(origin, grid, cell) => {
                let (x, y) = transform.apply(*origin);
                // A transform that stretches along the axes stretches the
                // cells, to whole pixels so that glyphs stay sharp and
                // neighbours meet; one that turns or shears leaves the
                // cells their size.
                let stretch = if transform.b == 0.0 && transform.c == 0.0 {
                    (transform.a, transform.d)
                } else {
                    (1.0, 1.0)
                };
                let side = |cell: u16, stretch: f32| {
                    (f32::from(cell) * stretch).round().clamp(0.0, 4096.0) as u16
                };
                let cell = (side(cell.0, stretch.0), side(cell.1, stretch.1));
                if cell.0 > 0 && cell.1 > 0 && x.is_finite() && y.is_finite() {
                    draws.push(Draw::Cells {
                        x: x.round() as i32,
                        y: y.round() as i32,
                        grid: grid.clone(),
                        cell,
                        foreground,
                    });
                }
            }
        }
    }
    // A clip the scene never popped ends with the scene.
    for shows in clips.into_iter().rev() {
        if shows {
            draws.push(Draw::PopClip);
        }
    }
    draws
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::fonts::{Font, FontSource};
    use crate::graphics::scene::{Color, Paint, Stroke};

    fn draws(scene: &Scene) -> Vec<Draw> {
        let mut glyphs = Glyphs::new(Font::load(&FontSource::Bundled));
        compile(scene, &Transform::identity(), (100, 100), &mut glyphs)
    }

    fn red() -> Paint {
        Paint::solid(Color::rgba(255, 0, 0, 255))
    }

    #[test]
    fn a_cell_without_a_color_takes_the_themes_text_color_else_the_default_themes() {
        use crate::theme::{dark_theme, Theme, ThemeVariables};
        let channels = |color: Premul| color.map(|channel| (channel * 255.0).round() as u8);
        let own = Theme::new("own")
            .with_variables(ThemeVariables::new().set("--color-foreground", "#102030"));
        assert_eq!(channels(cell_foreground(&own)), [16, 32, 48, 255]);
        // A theme with no text color and no theme below it takes the dark
        // preset's (THM-002).
        let bare = Theme::new("bare");
        let (r, g, b, a) = dark_theme()
            .resolve_color("foreground")
            .expect("the default theme has a text color");
        assert_eq!(bare.resolve_color("foreground"), Some((r, g, b, a)));
        assert_eq!(cell_foreground(&bare), premultiply([r, g, b, a]));
        // Not the white the canvas once drew such a cell in.
        assert!(a > 0.0 && [r, g, b] != [1.0; 3]);
    }

    #[test]
    fn a_rectangle_under_a_scale_stays_a_rectangle_and_a_turned_one_is_polygons() {
        let mut scene = Scene::new();
        scene.push_transform(Transform::scale(2.0, 0.5));
        scene.fill(&Path::rect(1.0, 2.0, 3.0, 4.0), &red());
        scene.pop_transform();
        scene.push_transform(Transform::rotate(10.0));
        scene.fill(&Path::rect(10.0, 10.0, 30.0, 30.0), &red());
        let draws = draws(&scene);
        assert!(matches!(draws[0], Draw::Rect { rect, .. } if rect == [2.0, 1.0, 8.0, 3.0]));
        assert!(matches!(draws[1], Draw::Fill { .. }));
    }

    #[test]
    fn a_transform_pushed_inside_another_applies_first() {
        let mut scene = Scene::new();
        scene.push_transform(Transform::translate(50.0, 50.0));
        scene.push_transform(Transform::scale(2.0, 2.0));
        scene.fill(&Path::rect(1.0, 1.0, 1.0, 1.0), &red());
        let draws = draws(&scene);
        assert!(matches!(draws[0], Draw::Rect { rect, .. } if rect == [52.0, 52.0, 54.0, 54.0]));
    }

    #[test]
    fn what_lies_outside_the_picture_or_inside_an_empty_clip_is_not_drawn() {
        let mut scene = Scene::new();
        scene.fill(&Path::rect(200.0, 0.0, 10.0, 10.0), &red());
        scene.push_clip(&Path::rect(-50.0, -50.0, 10.0, 10.0));
        scene.fill(&Path::rect(0.0, 0.0, 10.0, 10.0), &red());
        scene.stroke(&Path::rect(0.0, 0.0, 10.0, 10.0), &Stroke::new(2.0), &red());
        scene.pop_clip();
        scene.pop_clip();
        scene.pop_transform();
        assert!(draws(&scene).is_empty());
    }

    #[test]
    fn a_cell_grid_stretches_with_a_scale_and_keeps_its_size_under_a_turn() {
        let grid = Arc::new(CellGrid::new(4, 2));
        let cells = |transform: Transform| {
            let mut scene = Scene::new();
            scene.push_transform(transform);
            scene.cells((1.0, 2.0), grid.clone(), (8, 16));
            match draws(&scene).pop() {
                Some(Draw::Cells { x, y, cell, .. }) => (x, y, cell),
                other => panic!("a cell grid, not {other:?}"),
            }
        };
        assert_eq!(cells(Transform::translate(5.0, 5.0)), (6, 7, (8, 16)));
        assert_eq!(
            cells(Transform::scale(2.375, 2.375).then(Transform::translate(20.0, 0.0))),
            (22, 5, (19, 38))
        );
        assert_eq!(cells(Transform::rotate(90.0)).2, (8, 16));
    }

    #[test]
    fn a_clip_left_open_is_closed_at_the_end() {
        let mut scene = Scene::new();
        scene.push_clip(&Path::ellipse(50.0, 50.0, 20.0, 20.0));
        scene.fill(&Path::rect(0.0, 0.0, 100.0, 100.0), &red());
        let draws = draws(&scene);
        assert!(matches!(draws[0], Draw::PushClip { .. }));
        assert!(matches!(draws[2], Draw::PopClip));
        assert_eq!(draws.len(), 3);
    }
}
