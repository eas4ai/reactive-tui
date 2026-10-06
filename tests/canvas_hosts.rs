//! canvas-hosts mechanism, GFX-002 (docs/spec/canvas.md): run on the Linux
//! development host, the macOS test host and the Windows test tablet, each
//! of which has a hardware adapter. The canvas must draw on it, the software
//! renderer must draw the same picture for every reference scene, and each
//! picture must name the renderer that drew it. GFX-004's speed bound is in
//! canvas_speed.rs.
//!
//! The hosts differ in how they keep shared memory too, so GFX-005's rule
//! that a Kitty picture travels through it is checked here on each Unix
//! host, by a terminal that reads the picture as Kitty does.

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

#[cfg(unix)]
mod shared_memory {
    use super::canvas_support::{reference_options, shapes};
    use reactive_tui::app::{App, RootComponent, RootUpdate};
    use reactive_tui::backend::{ImageOutputOptions, SuprTuiBackend};
    use reactive_tui::component::Element;
    use reactive_tui::error::Result;
    use reactive_tui::graphics::{Canvas, CanvasProps};
    use std::io::{self, Write};
    use std::sync::{Arc, Mutex};

    /// A root that shows one canvas and stops after `frames` frames.
    struct Frames {
        frame: usize,
        frames: usize,
    }
    impl RootComponent for Frames {
        fn render(&self) -> Element {
            let props = CanvasProps::new(Arc::new(shapes())).options(reference_options(true));
            Element::typed::<Canvas>(props)
        }
        fn update(&mut self) -> Result<RootUpdate> {
            self.frame += 1;
            Ok(if self.frame >= self.frames {
                RootUpdate::Exit
            } else {
                RootUpdate::Redraw
            })
        }
        /// The backend writes to memory and has no terminal to read keys from.
        fn accepts_input(&self) -> bool {
            false
        }
    }

    /// What a picture's shared-memory object held.
    #[derive(Debug)]
    struct Held {
        /// The bytes of a picture of the size the command names.
        wanted: usize,
        /// How many of them are not zero.
        painted: usize,
    }

    /// The first `wanted` bytes of the shared-memory object `name`, read as
    /// Kitty reads them: opened by name, mapped, and the name removed.
    fn read(name: &str, wanted: usize) -> std::result::Result<Held, String> {
        use rustix::fs::Mode;
        use rustix::mm::{mmap, munmap, MapFlags, ProtFlags};
        use rustix::shm;
        let object = shm::open(name, shm::OFlags::RDONLY, Mode::empty())
            .map_err(|error| format!("{name} did not open: {error}"))?;
        let _ = shm::unlink(name);
        let size = rustix::fs::fstat(&object)
            .map_err(|error| format!("{name} has no size: {error}"))?
            .st_size as usize;
        if wanted == 0 || size < wanted {
            return Err(format!("{name} holds {size} bytes of {wanted}"));
        }
        // SAFETY: the mapping is new, of `wanted` bytes of an object that
        // holds at least as many, and nothing else knows its address. The
        // bytes are read while it lasts and it is unmapped before returning.
        let painted = unsafe {
            let start = mmap(
                std::ptr::null_mut(),
                wanted,
                ProtFlags::READ,
                MapFlags::SHARED,
                &object,
                0,
            )
            .map_err(|error| format!("{name} did not map: {error}"))?;
            let bytes = std::slice::from_raw_parts(start.cast::<u8>(), wanted);
            let painted = bytes.iter().filter(|byte| **byte != 0).count();
            let _ = munmap(start, wanted);
            painted
        };
        Ok(Held { wanted, painted })
    }

    /// A terminal that reads every Kitty picture it is sent through shared
    /// memory, and counts those it is sent in the command itself.
    #[derive(Clone, Default)]
    struct Terminal {
        pending: Arc<Mutex<Vec<u8>>>,
        shared: Arc<Mutex<Vec<std::result::Result<Held, String>>>>,
        direct: Arc<Mutex<usize>>,
    }
    impl Write for Terminal {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.pending.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            use base64::Engine;
            let chunk = std::mem::take(&mut *self.pending.lock().unwrap());
            let chunk = String::from_utf8_lossy(&chunk);
            for command in chunk.split("\x1b_G").skip(1) {
                let Some((keys, rest)) = command.split_once(';') else {
                    continue;
                };
                let keys: Vec<&str> = keys.split(',').collect();
                if !keys.contains(&"a=T") {
                    continue;
                }
                if !keys.contains(&"t=s") {
                    *self.direct.lock().unwrap() += 1;
                    continue;
                }
                let number = |key: &str| {
                    keys.iter()
                        .find_map(|pair| pair.strip_prefix(key)?.parse::<usize>().ok())
                        .unwrap_or(0)
                };
                let name = rest.split('\x1b').next().unwrap_or("");
                let held = base64::engine::general_purpose::STANDARD
                    .decode(name)
                    .ok()
                    .and_then(|name| String::from_utf8(name).ok())
                    .ok_or_else(|| format!("the name {name:?} is not base64 text"))
                    .and_then(|name| read(&name, number("s=") * number("v=") * 4));
                self.shared.lock().unwrap().push(held);
            }
            Ok(())
        }
    }

    #[test]
    fn gfx_005_a_canvas_picture_is_read_from_shared_memory() {
        let images = ImageOutputOptions {
            kitty_graphics: true,
            kitty_shared_memory: true,
            ..Default::default()
        };
        let terminal = Terminal::default();
        let backend =
            SuprTuiBackend::with_writer_and_images(60, 20, terminal.clone(), images).unwrap();
        App::builder()
            .backend(backend)
            .root(Frames {
                frame: 0,
                frames: 12,
            })
            .build()
            .unwrap()
            .run()
            .unwrap();
        let shared = terminal.shared.lock().unwrap();
        let direct = *terminal.direct.lock().unwrap();
        let read = shared
            .iter()
            .filter(|held| matches!(held, Ok(held) if held.painted > 0 && held.wanted > 0))
            .count();
        assert!(
            read > 0 && read == shared.len() && direct == 0,
            "GFX-005: with shared memory accepted, {} pictures named shared memory, {read} of them \
             could be read and held a picture, and {direct} came in the command itself; the first \
             held {:?}",
            shared.len(),
            &shared[..shared.len().min(3)]
        );
    }
}
