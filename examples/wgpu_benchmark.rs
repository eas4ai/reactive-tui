//! Bounded, unpaced canvas-to-terminal comparison (not display scanout
//! timing): the demo cube drawn by the chosen renderer, turned into block
//! glyphs and presented, as fast as the loop runs.
#[path = "widget_catalog/scene.rs"]
mod scene;

use reactive_tui::{
    backend::{Backend, SuprTuiBackend},
    component::Element,
    graphics::{GraphicsMode, GraphicsOptions, HybridRenderer, Scene, Transform},
};
use serde_json::json;
use std::{
    error::Error,
    fs::OpenOptions,
    io::{IsTerminal, Write},
    sync::Arc,
    time::{Duration, Instant},
};

/// The cube at `elapsed`, fitted to a picture of `size`.
fn cube(elapsed: Duration, size: (u32, u32)) -> Scene {
    scene::cube_scene(elapsed)
        .transformed(Transform::fit(scene::VIEW, (size.0 as f32, size.1 as f32)))
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut columns = 144_u16;
    let mut rows = 50_u16;
    let mut seconds = 1.0_f64;
    let mut cpu = false;
    let mut report = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--cpu" => cpu = true,
            "--columns" => columns = args.next().ok_or("missing columns")?.parse()?,
            "--rows" => rows = args.next().ok_or("missing rows")?.parse()?,
            "--seconds" => seconds = args.next().ok_or("missing seconds")?.parse()?,
            "--report" => report = Some(args.next().ok_or("missing report path")?),
            "--help" => {
                println!("wgpu_benchmark --columns 144 --rows 50 --seconds 1 [--cpu] [--report NEW_FILE]");
                return Ok(());
            }
            _ => return Err(format!("unknown option: {arg}").into()),
        }
    }
    if !seconds.is_finite() || !(0.25..=30.0).contains(&seconds) {
        return Err("seconds must be finite and between 0.25 and 30".into());
    }
    // Scenes are drawn for cells of 8 by 16 pixels.
    let pixels = (u32::from(columns) * 8, u32::from(rows) * 16);
    if columns == 0 || rows == 0 || pixels.0 > 4096 || pixels.1 > 4096 {
        return Err("columns must be 1 to 512 and rows 1 to 256".into());
    }
    if !std::io::stdout().is_terminal() {
        return Err("run in a real terminal, not a redirected writer".into());
    }
    let mut output = report
        .map(|path| OpenOptions::new().write(true).create_new(true).open(path))
        .transpose()?;
    let initialization_started = Instant::now();
    let mut hardware = HybridRenderer::new(GraphicsOptions::default());
    let comparison = match hardware.mode() {
        GraphicsMode::Gpu(info) if info.is_hardware() => info.clone(),
        other => {
            return Err(format!(
                "hardware GPU required for the comparison; found {}",
                other.label()
            )
            .into())
        }
    };
    let mut software = HybridRenderer::new(GraphicsOptions {
        force_cpu: true,
        ..Default::default()
    });
    let fixed = cube(Duration::from_secs(1), pixels);
    let gpu_pixels = hardware.render(&fixed, pixels.0, pixels.1)?;
    let cpu_pixels = software.render(&fixed, pixels.0, pixels.1)?;
    if !matches!(gpu_pixels.mode(), GraphicsMode::Gpu(_)) {
        return Err(format!(
            "the hardware adapter did not draw the comparison: {}",
            gpu_pixels.mode().label()
        )
        .into());
    }
    let mut differing_pixels = 0_u64;
    let mut absolute_error = 0_u64;
    let mut max_channel_error = 0_u8;
    for (a, b) in gpu_pixels.pixels().iter().zip(cpu_pixels.pixels()) {
        differing_pixels += u64::from(a != b);
        for channel in 0..3 {
            let error = a[channel].abs_diff(b[channel]);
            absolute_error += u64::from(error);
            max_channel_error = max_channel_error.max(error);
        }
    }
    let mut renderer = if cpu { software } else { hardware };
    // Warm the chosen renderer up before the sampling deadline.
    renderer.render_cells(&cube(Duration::ZERO, pixels), columns, rows)?;
    let adapter = match renderer.mode() {
        GraphicsMode::Gpu(info) if !cpu && info.is_hardware() => info.name.clone(),
        GraphicsMode::CpuFallback(_) if cpu => "CPU (explicit selection)".into(),
        other => return Err(format!("requested mode was not selected: {}", other.label()).into()),
    };
    let initialization_ms = initialization_started.elapsed().as_secs_f64() * 1000.0;
    let mut backend = SuprTuiBackend::new()?;
    if backend.size() != (columns, rows) {
        backend.shutdown()?;
        return Err("terminal dimensions must match --columns and --rows".into());
    }
    let started = Instant::now();
    let duration = Duration::from_secs_f64(seconds);
    let mut sums = [0.0_f64; 3];
    let mut count = 0_u64;
    let mut failure = None;
    while started.elapsed() < duration {
        let frame_started = Instant::now();
        let grid = match renderer.render_cells(&cube(started.elapsed(), pixels), columns, rows) {
            Ok(grid) => grid,
            Err(error) => {
                failure = Some(error.to_string());
                break;
            }
        };
        let selected = matches!(
            (renderer.mode(), cpu),
            (GraphicsMode::Gpu(_), false) | (GraphicsMode::CpuFallback(_), true)
        );
        if !selected {
            failure = Some(format!(
                "mode changed during measurement: {}",
                renderer.mode().label()
            ));
            break;
        }
        let drawn = frame_started.elapsed();
        let presentation_started = Instant::now();
        let element = Element::text("")
            .with_class("w-screen h-screen")
            .with_cells(Arc::new(grid));
        backend.render_frame(&element)?;
        backend.present()?;
        let presentation = presentation_started.elapsed();
        for (sum, elapsed) in sums
            .iter_mut()
            .zip([drawn, presentation, frame_started.elapsed()])
        {
            *sum += elapsed.as_secs_f64() * 1000.0;
        }
        count += 1;
    }
    let sampled = started.elapsed().as_secs_f64();
    backend.shutdown()?;
    if let Some(failure) = failure {
        return Err(failure.into());
    }
    let averages = sums.map(|sum| sum / count.max(1) as f64);
    let data = json!({
        "schema": 2,
        "adapter": adapter,
        "comparison_adapter": {"name": comparison.name, "backend": comparison.backend, "hardware": true},
        "terminal": {"term": std::env::var("TERM").unwrap_or_default(), "term_program": std::env::var("TERM_PROGRAM").unwrap_or_default(), "kitty_pid_present": std::env::var_os("KITTY_PID").is_some()},
        "viewport": {"columns": columns, "rows": rows},
        "mode": if cpu {"CPU"} else {"GPU"},
        "requested_seconds": seconds, "sampled_seconds": sampled, "frames": count,
        "achieved_fps": count as f64 / sampled, "initialization_and_quality_ms": initialization_ms,
        "mean_ms": {"block_glyphs": averages[0], "presentation": averages[1], "total": averages[2]},
        "quality": {"elapsed_seconds": 1, "width": pixels.0, "height": pixels.1, "differing_pixels": differing_pixels, "pixels": gpu_pixels.pixels().len(), "mean_absolute_rgb_error": absolute_error as f64 / (gpu_pixels.pixels().len() * 3) as f64, "max_channel_error": max_channel_error, "description": "The same scene drawn by the hardware adapter and by the software renderer at 8 by 16 pixels per cell."},
        "timing_scope": "Unpaced native backend. block_glyphs: the scene drawn at the blitter's pixels per cell, read back on the hardware adapter, and turned into block glyphs. presentation: layout, paint and ANSI writes. Excludes initialization, App reconciliation, terminal display scanout and host acknowledgement. The last frame in flight may extend sampling past the requested duration."
    });
    let encoded = serde_json::to_string_pretty(&data)?;
    if let Some(file) = output.as_mut() {
        writeln!(file, "{encoded}")?;
        file.sync_all()?;
    }
    println!("{encoded}");
    Ok(())
}
