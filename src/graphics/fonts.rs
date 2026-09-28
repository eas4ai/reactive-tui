//! The font the canvas draws text in (GFX-001): the font the application
//! supplies, else on Linux the first file of fontconfig's sorted match for
//! `monospace` that the canvas can load, else the bundled DejaVu Sans Mono.
//! The canvas reads a font's outlines with skrifa and draws them with its
//! own rasterizer, so text is the same pixels on both renderers.

use super::scene::{Path, PathBuilder};
use skrifa::instance::{LocationRef, Size};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::{FontRef, GlyphId, MetadataProvider};
use std::path::{Path as FilePath, PathBuf};
use std::sync::Arc;

/// DejaVu Sans Mono, under the license beside it
/// (`fonts/DejaVuSansMono-LICENSE.txt`).
const BUNDLED: &[u8] = include_bytes!("fonts/DejaVuSansMono.ttf");

/// Where the canvas takes its font from.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum FontSource {
    /// No application font: on Linux the first loadable file fontconfig
    /// lists for `monospace`, else the bundled font.
    #[default]
    Auto,
    /// The monospace font the crate bundles, the same on every host.
    Bundled,
    /// A font file the system lists.
    System(PathBuf),
    /// A font file the application supplies.
    File(PathBuf),
    /// The bytes of a font the application supplies.
    Data(Arc<[u8]>),
}

/// Whether the canvas can draw text from the font file at `path`: it reads
/// as TrueType or OpenType and has an outline for a letter.
pub fn loads(path: &FilePath) -> bool {
    std::fs::read(path).is_ok_and(|data| usable(&data))
}

/// The first font of `data`, a font file or a collection of fonts.
fn first_font(data: &[u8]) -> Option<FontRef<'_>> {
    FontRef::from_index(data, 0).ok()
}

/// Outlines as the font's designer drew them: in font units, not hinted.
fn as_designed<'a>() -> DrawSettings<'a> {
    DrawSettings::unhinted(Size::unscaled(), LocationRef::default())
}

/// Whether `data` is a font the canvas can draw text from.
fn usable(data: &[u8]) -> bool {
    struct Any(bool);
    impl OutlinePen for Any {
        fn move_to(&mut self, _: f32, _: f32) {
            self.0 = true;
        }
        fn line_to(&mut self, _: f32, _: f32) {}
        fn quad_to(&mut self, _: f32, _: f32, _: f32, _: f32) {}
        fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {}
        fn close(&mut self) {}
    }
    let Some(font) = first_font(data) else {
        return false;
    };
    let Some(outline) = font
        .charmap()
        .map('a')
        .and_then(|glyph| font.outline_glyphs().get(glyph))
    else {
        return false;
    };
    let mut any = Any(false);
    let metrics = font.metrics(Size::unscaled(), LocationRef::default());
    metrics.units_per_em > 0 && outline.draw(as_designed(), &mut any).is_ok() && any.0
}

/// The font the canvas draws in: `application`'s when it names one that
/// loads, else the first of `candidates` that loads, else the bundled font.
/// `candidates` are the system's monospace fonts in order of preference, on
/// Linux the files of `fc-match -s monospace`.
pub fn choose(application: Option<&FontSource>, candidates: &[PathBuf]) -> FontSource {
    match application {
        Some(FontSource::Bundled) => return FontSource::Bundled,
        Some(source @ (FontSource::System(path) | FontSource::File(path))) if loads(path) => {
            return source.clone();
        }
        Some(source @ FontSource::Data(data)) if usable(data) => return source.clone(),
        _ => {}
    }
    candidates
        .iter()
        .find(|path| loads(path))
        .map_or(FontSource::Bundled, |path| FontSource::System(path.clone()))
}

/// The files fontconfig lists for `monospace`, best match first; none where
/// fontconfig is not installed. Asked once per process, on the first canvas
/// worker that needs it.
fn system_candidates() -> &'static [PathBuf] {
    static CANDIDATES: std::sync::OnceLock<Vec<PathBuf>> = std::sync::OnceLock::new();
    CANDIDATES.get_or_init(|| {
        if !cfg!(target_os = "linux") {
            return Vec::new();
        }
        std::process::Command::new("fc-match")
            .args(["-s", "-f", "%{file}\\n", "monospace"])
            .stdin(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
            .ok()
            .filter(|output| output.status.success())
            .map(|output| {
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .filter(|line| !line.is_empty())
                    .map(PathBuf::from)
                    .collect()
            })
            .unwrap_or_default()
    })
}

/// A loaded font: its bytes and the measures text layout needs, in font
/// units.
#[derive(Clone)]
pub(crate) struct Font {
    data: Arc<[u8]>,
    /// What the font was loaded from, after the choice was made.
    pub source: FontSource,
    pub units_per_em: f32,
    pub ascent: f32,
    /// Below the baseline, positive.
    pub descent: f32,
    /// The advance of a cell: the font's advance for `0`.
    pub advance: f32,
}

impl std::fmt::Debug for Font {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Font")
            .field("source", &self.source)
            .field("bytes", &self.data.len())
            .finish()
    }
}

impl Font {
    /// The font `source` asks for, by [`choose`]'s rule; the bundled font
    /// when the one chosen cannot be read after all.
    pub fn load(source: &FontSource) -> Self {
        let chosen = match source {
            // The bundled font needs no list of the system's fonts.
            FontSource::Bundled => FontSource::Bundled,
            FontSource::Auto => choose(None, system_candidates()),
            other => choose(Some(other), system_candidates()),
        };
        let data: Option<Arc<[u8]>> = match &chosen {
            FontSource::Auto | FontSource::Bundled => None,
            FontSource::System(path) | FontSource::File(path) => {
                std::fs::read(path).ok().map(Arc::from)
            }
            FontSource::Data(data) => Some(data.clone()),
        };
        data.and_then(|data| Self::parse(data, chosen))
            .or_else(|| Self::parse(Arc::from(BUNDLED), FontSource::Bundled))
            .expect("the bundled font reads")
    }

    fn parse(data: Arc<[u8]>, source: FontSource) -> Option<Self> {
        if !usable(&data) {
            return None;
        }
        let (units_per_em, ascent, descent, advance) = {
            let font = first_font(&data)?;
            let metrics = font.metrics(Size::unscaled(), LocationRef::default());
            let units_per_em = f32::from(metrics.units_per_em);
            let glyphs = font.glyph_metrics(Size::unscaled(), LocationRef::default());
            (
                units_per_em,
                metrics.ascent,
                // The font gives the descent as a height below zero.
                metrics.descent.abs(),
                font.charmap()
                    .map('0')
                    .and_then(|glyph| glyphs.advance_width(glyph))
                    .unwrap_or(units_per_em * 0.6),
            )
        };
        Some(Self {
            data,
            source,
            units_per_em,
            ascent,
            descent,
            advance,
        })
    }

    fn font(&self) -> FontRef<'_> {
        // The bytes read when the font was loaded, so they read again.
        first_font(&self.data).expect("a font that was loaded")
    }

    /// The glyph of `character`; the font's missing-glyph box when it has
    /// none.
    pub fn glyph(&self, character: char) -> u32 {
        self.font()
            .charmap()
            .map(character)
            .map_or(0, |glyph| glyph.to_u32())
    }

    /// The advance of `glyph` in font units.
    pub fn glyph_advance(&self, glyph: u32) -> f32 {
        self.font()
            .glyph_metrics(Size::unscaled(), LocationRef::default())
            .advance_width(GlyphId::new(glyph))
            .unwrap_or(self.advance)
    }

    /// The outline of `glyph` in font units with its origin on the
    /// baseline and y growing downwards, as the canvas draws; `None` for a
    /// glyph that draws nothing, such as a space.
    pub fn outline(&self, glyph: u32) -> Option<Path> {
        struct Outline {
            builder: PathBuilder,
            any: bool,
        }
        impl Outline {
            fn point(&self, x: f32, y: f32) -> (f32, f32) {
                (x, -y)
            }
            fn step(&mut self, step: impl FnOnce(PathBuilder) -> PathBuilder) {
                self.builder = step(std::mem::take(&mut self.builder));
            }
        }
        impl OutlinePen for Outline {
            fn move_to(&mut self, x: f32, y: f32) {
                let (x, y) = self.point(x, y);
                self.any = true;
                self.step(|path| path.move_to(x, y));
            }
            fn line_to(&mut self, x: f32, y: f32) {
                let (x, y) = self.point(x, y);
                self.step(|path| path.line_to(x, y));
            }
            fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
                let (cx, cy) = self.point(cx, cy);
                let (x, y) = self.point(x, y);
                self.step(|path| path.quad_to(cx, cy, x, y));
            }
            fn curve_to(&mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
                let (c1x, c1y) = self.point(c1x, c1y);
                let (c2x, c2y) = self.point(c2x, c2y);
                let (x, y) = self.point(x, y);
                self.step(|path| path.cubic_to(c1x, c1y, c2x, c2y, x, y));
            }
            fn close(&mut self) {
                self.step(PathBuilder::close);
            }
        }
        let mut outline = Outline {
            builder: PathBuilder::new(),
            any: false,
        };
        self.font()
            .outline_glyphs()
            .get(GlyphId::new(glyph))?
            .draw(as_designed(), &mut outline)
            .ok()?;
        outline.any.then(|| outline.builder.build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bundled_font_loads_and_is_monospace() {
        let font = Font::load(&FontSource::Bundled);
        assert_eq!(font.source, FontSource::Bundled);
        assert_eq!(font.units_per_em, 2048.0);
        assert_eq!(
            font.glyph_advance(font.glyph('i')),
            font.glyph_advance(font.glyph('W'))
        );
        assert!(font.outline(font.glyph('a')).is_some());
        assert!(font.outline(font.glyph(' ')).is_none());
    }

    #[test]
    fn a_file_that_is_no_font_does_not_load_and_the_choice_passes_it_by() {
        let dir = tempfile::tempdir().unwrap();
        let bad = dir.path().join("bad.ttf");
        std::fs::write(&bad, b"not a font").unwrap();
        let good = dir.path().join("good.ttf");
        std::fs::write(&good, BUNDLED).unwrap();
        assert!(!loads(&bad));
        assert!(loads(&good));
        assert_eq!(
            choose(None, &[bad.clone(), good.clone()]),
            FontSource::System(good.clone())
        );
        assert_eq!(
            choose(None, std::slice::from_ref(&bad)),
            FontSource::Bundled
        );
        // The application's font comes first when it loads.
        assert_eq!(
            choose(Some(&FontSource::File(good.clone())), &[]),
            FontSource::File(good.clone())
        );
        assert_eq!(
            choose(Some(&FontSource::File(bad)), std::slice::from_ref(&good)),
            FontSource::System(good)
        );
        // An application font that does not load gives way to the system's
        // font where fontconfig lists one, else to the bundled font.
        let junk = FontSource::Data(Arc::from(&b"junk"[..]));
        let loaded = Font::load(&junk).source;
        assert!(
            matches!(loaded, FontSource::System(_) | FontSource::Bundled),
            "{loaded:?}"
        );
    }
}
