use reactive_tui::{
    backend::{Backend, DebugBackend, SuprTuiBackend},
    graphics::{GpuCubeRenderer, GraphicsFrame},
};
use std::{
    collections::HashSet,
    io::{self, Write},
    sync::{Arc, Mutex},
    time::Duration,
};

#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<u8>>>);
impl Write for Capture {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn gpu_cube_uses_a_real_adapter_and_resizes_to_terminal_viewports() {
    let renderer = GpuCubeRenderer::new().expect("real GPU adapter");
    let info = renderer.adapter_info();
    println!("GPU ADAPTER {} · {}", info.name, info.backend);
    assert!(
        info.is_hardware,
        "software adapter cannot prove GPU-002: {info:?}"
    );
    let capture = Capture::default();
    let mut terminal = SuprTuiBackend::with_writer(60, 24, capture.clone()).unwrap();

    for (columns, rows) in [(60, 24), (144, 50), (200, 60)] {
        let frame = renderer
            .render_terminal(columns, rows, Duration::from_millis(375))
            .unwrap();
        assert_eq!((frame.width(), frame.height()), (columns, rows * 2));
        assert_eq!(frame.pixels().len(), columns as usize * rows as usize * 2);
        let colors = frame
            .pixels()
            .iter()
            .filter(|pixel| pixel[3] == 255)
            .copied()
            .collect::<HashSet<_>>();
        assert!(colors.len() >= 32, "cube output is not shaded: {colors:?}");
        let background = frame.pixels()[0];
        let occupied: Vec<_> = frame
            .pixels()
            .iter()
            .enumerate()
            .filter(|(_, pixel)| **pixel != background)
            .map(|(index, _)| (index as u32 % columns, index as u32 / columns))
            .collect();
        let left = occupied.iter().map(|point| point.0).min().unwrap();
        let right = occupied.iter().map(|point| point.0).max().unwrap();
        let top = occupied.iter().map(|point| point.1).min().unwrap();
        let bottom = occupied.iter().map(|point| point.1).max().unwrap();
        assert!(
            left > 0 && right < columns - 1 && top > 0 && bottom < rows * 2 - 1,
            "cube is clipped at {columns}x{rows}"
        );
        let aspect = (right - left + 1) as f32 / (bottom - top + 1) as f32;
        assert!((0.8..1.3).contains(&aspect), "stretched cube: {aspect}");
        let mut backend = DebugBackend::new(columns as u16, rows as u16);
        backend
            .render_full(&frame.to_half_block_element().unwrap())
            .unwrap();
        assert_eq!(
            backend.screen_content().matches('▀').count(),
            columns as usize * rows as usize
        );
        terminal.resize(columns as usize, rows as usize);
        capture.0.lock().unwrap().clear();
        terminal
            .render_frame(&frame.to_half_block_element().unwrap())
            .unwrap();
        terminal.present().unwrap();
        let mut parser = vt100::Parser::new(rows as u16, columns as u16, 0);
        parser.process(&capture.0.lock().unwrap());
        for y in 0..rows {
            for x in 0..columns {
                let cell = parser.screen().cell(y as u16, x as u16).unwrap();
                let upper = frame.pixels()[(y * 2 * columns + x) as usize];
                let lower = frame.pixels()[((y * 2 + 1) * columns + x) as usize];
                assert_eq!(cell.contents(), "▀", "missing cell {x},{y}");
                assert_eq!(
                    cell.fgcolor(),
                    vt100::Color::Rgb(upper[0], upper[1], upper[2])
                );
                assert_eq!(
                    cell.bgcolor(),
                    vt100::Color::Rgb(lower[0], lower[1], lower[2])
                );
            }
        }
        println!("GPU VIEWPORT {columns}x{rows} colors={} bounds={left},{top}..{right},{bottom} aspect={aspect:.3}", colors.len());
        if let Ok(directory) = std::env::var("REACTIVE_TUI_GPU_CAPTURE_DIR") {
            let bytes: Vec<_> = frame.pixels().iter().flatten().copied().collect();
            image::save_buffer(
                std::path::Path::new(&directory).join(format!("cube-{columns}x{rows}.png")),
                &bytes,
                columns,
                rows * 2,
                image::ColorType::Rgba8,
            )
            .unwrap();
        }
    }
    terminal.shutdown().unwrap();
}

#[test]
fn half_blocks_present_through_the_existing_frame_path() {
    let width = 60;
    let rows = 24;
    let pixels = (0..width * rows * 2)
        .map(|index| {
            let x = (index % width) as u8;
            let y = (index / width) as u8;
            [x.wrapping_mul(3), y.wrapping_mul(5), 200, 255]
        })
        .collect();
    let frame = GraphicsFrame::from_rgba(width, rows * 2, pixels).unwrap();
    let element = frame.to_half_block_element().unwrap();
    assert_eq!(element.children.len(), rows as usize);
    assert!(element
        .children
        .iter()
        .all(|row| row.children.len() == width as usize));

    let mut backend = DebugBackend::new(width as u16, rows as u16);
    backend.render_full(&element).unwrap();
    let output = backend.screen_content();
    assert_eq!(output.matches('▀').count(), width as usize * rows as usize);
}

#[test]
fn graphics_dimensions_are_checked_before_allocation() {
    assert!(GraphicsFrame::from_rgba(0, 2, vec![]).is_err());
    assert!(GraphicsFrame::from_rgba(801, 2, vec![[0; 4]; 1602]).is_err());
    assert!(GraphicsFrame::from_rgba(2, 601, vec![[0; 4]; 1202]).is_err());
    assert!(GraphicsFrame::from_rgba(2, 2, vec![[0; 4]; 3]).is_err());
}
