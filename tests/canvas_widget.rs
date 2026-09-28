//! widget-bar and frame-budget mechanisms for the Canvas widget (BAR-003,
//! BAR-005; docs/spec/quality-bar.md): it fills its parent, draws the new
//! size after a resize, describes its renderer to the screen reader, keeps
//! the App's per-frame work under 16.6 ms at 700 by 200 while animating and
//! draws on its `rtui-canvas-*` worker.

mod canvas_support;
mod common;

use canvas_support::reference_options;
use common::app_input::{self, Snapshot};
use reactive_tui::app::{RootComponent, RootUpdate};
use reactive_tui::component::Element;
use reactive_tui::error::Result;
use reactive_tui::event::types::{Event, ResizeEvent};
use reactive_tui::graphics::{Canvas, CanvasProps, Color, Paint, Path, Scene};
#[cfg(target_os = "linux")]
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Whether an `rtui-canvas-*` thread was alive during any frame. The
/// process's threads are read from /proc, so Linux makes the check.
#[cfg(target_os = "linux")]
static CANVAS_WORKER_SEEN: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "linux")]
fn note_canvas_worker() {
    if let Ok(tasks) = std::fs::read_dir("/proc/self/task") {
        for task in tasks.flatten() {
            let name = std::fs::read_to_string(task.path().join("comm")).unwrap_or_default();
            if name.starts_with("rtui-canvas") {
                CANVAS_WORKER_SEEN.store(true, Ordering::SeqCst);
            }
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn note_canvas_worker() {}

fn filled() -> Scene {
    let mut scene = Scene::new();
    scene.fill(
        &Path::rect(0.0, 0.0, 8192.0, 8192.0),
        &Paint::solid(Color::rgba(200, 40, 40, 255)),
    );
    scene
}

/// A root that shows one canvas filling the screen, a new cube angle each
/// frame when `animate`.
struct Root {
    animate: bool,
    frame: usize,
}
impl RootComponent for Root {
    fn render(&self) -> Element {
        note_canvas_worker();
        let scene = if self.animate {
            let angle = self.frame as f32 * 0.05;
            canvas_support::cube(angle, angle * 0.6)
        } else {
            filled()
        };
        Element::typed::<Canvas>(CanvasProps::new(Arc::new(scene)).options(reference_options(true)))
    }
    fn update(&mut self) -> Result<RootUpdate> {
        self.frame += 1;
        Ok(if self.animate {
            RootUpdate::Redraw
        } else {
            RootUpdate::Unchanged
        })
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

/// The columns and rows whose cells carry the fill's background or glyphs.
fn painted_extent(frame: &Snapshot) -> (u16, u16) {
    let (rows, columns) = frame.screen.size();
    let mut extent = (0, 0);
    for y in 0..rows {
        for x in 0..columns {
            if let Some(cell) = frame.screen.cell(y, x) {
                let inked =
                    !cell.contents().trim().is_empty() || cell.bgcolor() != vt100::Color::Default;
                if inked {
                    extent = (extent.0.max(x + 1), extent.1.max(y + 1));
                }
            }
        }
    }
    extent
}

#[test]
fn bar_003_the_canvas_fills_its_parent_and_follows_a_resize() {
    let old = (40u16, 12u16);
    let new = (140u16, 30u16);
    let frames = app_input::run(
        Root {
            animate: false,
            frame: 0,
        },
        old,
        // Frames that are not busy: the first, before the canvas knows its
        // size, and the first that shows the picture; the canvas never
        // waits for its worker, so the frame between them is busy.
        vec![
            (2, Some(Event::Resize(ResizeEvent::new(new.0, new.1)))),
            (3, None),
        ],
    );
    let before = frames
        .iter()
        .rfind(|f| f.screen.size() == (old.1, old.0) && !f.busy)
        .expect("a finished frame at the old size");
    let after = frames
        .iter()
        .rfind(|f| f.screen.size() == (new.1, new.0) && !f.busy)
        .expect("a finished frame at the new size");
    assert!(
        painted_extent(before) == old && painted_extent(after) == new,
        "BAR-003: the canvas painted {:?} of {old:?} and, after the resize, {:?} of {new:?}",
        painted_extent(before),
        painted_extent(after)
    );
}

#[test]
fn bar_003_the_canvas_describes_its_renderer_to_the_screen_reader() {
    let frames = app_input::run(
        Root {
            animate: false,
            frame: 0,
        },
        (40, 12),
        vec![(2, None)],
    );
    let spoken: Vec<String> = frames.iter().flat_map(|f| f.live.clone()).collect();
    assert!(
        spoken
            .iter()
            .any(|text| text.contains("Canvas") && text.contains("CPU fallback")),
        "BAR-003: the canvas on the software renderer described itself as {spoken:?}"
    );
}

#[test]
fn bar_005_an_animating_canvas_stays_under_the_frame_budget_at_700_by_200() {
    if cfg!(debug_assertions) {
        // As for the charts, the budget is measured on the optimized build;
        // a debug build still checks the worker at a size it handles.
        eprintln!("SKIP: the frame budget is measured on the optimized build");
        let frames = app_input::run(
            Root {
                animate: true,
                frame: 0,
            },
            (80, 24),
            vec![(3, None)],
        );
        assert!(frames.len() >= 3);
        #[cfg(target_os = "linux")]
        assert!(
            CANVAS_WORKER_SEEN.load(Ordering::SeqCst),
            "BAR-005: no rtui-canvas worker thread was seen while the canvas animated"
        );
        return;
    }
    let frames = app_input::run_on_debug(
        Root {
            animate: true,
            frame: 0,
        },
        (700, 200),
        vec![(12, None)],
    );
    let worst = frames.iter().map(|f| f.work_ms).fold(0.0, f64::max);
    #[cfg(target_os = "linux")]
    assert!(
        CANVAS_WORKER_SEEN.load(Ordering::SeqCst),
        "BAR-005: no rtui-canvas worker thread was seen while the canvas animated"
    );
    assert!(
        frames.len() >= 12 && worst < 16.6,
        "BAR-005: an animating canvas at 700 by 200 took up to {worst:.2} ms of App work per frame over {} frames",
        frames.len()
    );
}
