//! charts-goldens mechanism, BAR-004 for the Canvas widget: its block-glyph
//! output on the debug backend, drawn by the software renderer with the
//! bundled font, against checked-in goldens of the text grid plus a color
//! digest at 80 by 24 and at 400 by 100, since a canvas scales with its
//! width. Goldens live in `tests/snapshots/canvas/<scene>_<size>.ansi`, or
//! under `REACTIVE_TUI_SNAPSHOTS` when the check points there; run with
//! `REGENERATE=1` to write them, review the diff, then commit.

mod canvas_support;
mod common;

use canvas_support::reference_options;
use common::app_input::{self, Snapshot};
use reactive_tui::app::RootComponent;
use reactive_tui::component::Element;
use reactive_tui::graphics::{Canvas, CanvasOutput, CanvasProps, GraphicsOptions};
use reactive_tui::widgets::display::{set_image_blitter, Blitter};
use std::sync::Arc;

struct Root(Element);
impl RootComponent for Root {
    fn render(&self) -> Element {
        self.0.clone()
    }
}

fn snapshots_dir() -> std::path::PathBuf {
    std::env::var_os("REACTIVE_TUI_SNAPSHOTS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots")
        })
        .join("canvas")
}

fn golden_bytes(frame: &Snapshot) -> Vec<u8> {
    let (rows, cols) = frame.screen.size();
    let mut hasher = common::digest::Digest::default();
    for r in 0..rows {
        for c in 0..cols {
            if let Some(cell) = frame.screen.cell(r, c) {
                hasher.field(format!("{:?}{:?}", cell.fgcolor(), cell.bgcolor()).as_bytes());
            }
        }
    }
    format!("{}\ncolors: {:016x}\n", frame.text, hasher.finish()).into_bytes()
}

#[test]
fn bar_004_canvas_goldens_at_80_by_24_and_400_by_100() {
    set_image_blitter(Some(Blitter::Sextant));
    let mut mismatches = Vec::new();
    for (scene_name, scene) in [
        ("shapes", canvas_support::shapes()),
        ("gradients", canvas_support::gradients()),
        ("cube", canvas_support::cube(0.6, 0.4)),
    ] {
        let scene = Arc::new(scene);
        for size in [(80u16, 24u16), (400u16, 100u16)] {
            let options = GraphicsOptions {
                output: Some(CanvasOutput::Blocks),
                ..reference_options(true)
            };
            let frame = app_input::run_when_painted_on_debug(
                Root(Element::typed::<Canvas>(
                    CanvasProps::new(scene.clone()).options(options),
                )),
                size,
                2,
            )
            .pop()
            .expect("a painted frame");
            let name = format!("{scene_name}_{}x{}", size.0, size.1);
            let path = snapshots_dir().join(format!("{name}.ansi"));
            let bytes = golden_bytes(&frame);
            if std::env::var("REGENERATE").as_deref() == Ok("1") {
                std::fs::create_dir_all(snapshots_dir()).expect("snapshot dir");
                std::fs::write(&path, &bytes).expect("write golden");
                continue;
            }
            match std::fs::read(&path) {
                Ok(expected) if expected == bytes => {}
                Ok(_) => mismatches.push(format!("golden mismatch for {name}")),
                Err(_) => mismatches.push(format!("{name}: no golden at {}", path.display())),
            }
        }
    }
    assert!(
        mismatches.is_empty(),
        "BAR-004: canvas goldens: {mismatches:?}"
    );
}
