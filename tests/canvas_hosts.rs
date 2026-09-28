//! canvas-hosts mechanism, GFX-002 (docs/spec/canvas.md): run on the Linux
//! development host, the macOS test host and the Windows test tablet, each
//! of which has a hardware adapter. The canvas must draw on it, the software
//! renderer must draw the same picture for every reference scene, and each
//! picture must name the renderer that drew it. GFX-004's speed bound is in
//! canvas_speed.rs.

mod canvas_support;

use canvas_support::{difference, reference_options, references};
use reactive_tui::graphics::{GraphicsMode, HybridRenderer};

#[test]
// Tests that use the GPU take turns: one adapter serves them all.
#[serial_test::serial(gpu)]
fn gfx_002_the_hardware_adapter_draws() {
    let mut renderer = HybridRenderer::new(reference_options(false));
    let reference = &references()[0];
    let frame = renderer
        .render(&reference.scene, reference.size.0, reference.size.1)
        .expect("a picture");
    assert!(
        matches!(frame.mode(), GraphicsMode::Gpu(info) if info.is_hardware()),
        "GFX-002: this host has a hardware adapter, but the canvas drew with {:?}",
        frame.mode()
    );
    println!("GFX-002 adapter: {}", frame.mode().label());
}

#[test]
// Tests that use the GPU take turns: one adapter serves them all.
#[serial_test::serial(gpu)]
fn gfx_002_both_renderers_draw_the_same_picture() {
    let mut hardware = HybridRenderer::new(reference_options(false));
    let mut software = HybridRenderer::new(reference_options(true));
    let mut failures = Vec::new();
    for reference in references() {
        let (width, height) = reference.size;
        let gpu = hardware
            .render(&reference.scene, width, height)
            .expect("a GPU picture");
        let cpu = software
            .render(&reference.scene, width, height)
            .expect("a CPU picture");
        if !matches!(gpu.mode(), GraphicsMode::Gpu(_)) {
            failures.push(format!(
                "{}: the hardware side drew with {:?}",
                reference.name,
                gpu.mode()
            ));
        } else if let Some(why) = difference(&gpu, &cpu) {
            failures.push(format!("{}: {why}", reference.name));
        }
    }
    assert!(
        failures.is_empty(),
        "GFX-002: the hardware adapter and the software renderer do not draw the same picture: {failures:?}"
    );
}

#[test]
// Tests that use the GPU take turns: one adapter serves them all.
#[serial_test::serial(gpu)]
fn gfx_002_each_picture_names_the_renderer_that_drew_it() {
    let reference = &references()[2];
    let (width, height) = reference.size;
    let mut hardware = HybridRenderer::new(reference_options(false));
    let mut software = HybridRenderer::new(reference_options(true));
    let gpu = hardware
        .render(&reference.scene, width, height)
        .expect("a GPU picture");
    let cpu = software
        .render(&reference.scene, width, height)
        .expect("a CPU picture");
    assert!(
        gpu.mode() == hardware.mode()
            && matches!(gpu.mode(), GraphicsMode::Gpu(_))
            && cpu.mode() == software.mode()
            && matches!(cpu.mode(), GraphicsMode::CpuFallback(_)),
        "GFX-002: the pictures report {:?} and {:?}; the renderers report {:?} and {:?}",
        gpu.mode(),
        cpu.mode(),
        hardware.mode(),
        software.mode()
    );
}
