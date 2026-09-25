//! Bounded, unpaced render-to-terminal comparison (not display scanout timing).
use reactive_tui::{
    backend::{Backend, SuprTuiBackend},
    graphics::{GpuCubeRenderer, GraphicsMode, GraphicsOptions, HybridCubeRenderer},
};
use serde_json::json;
use std::{
    error::Error,
    fs::OpenOptions,
    io::{IsTerminal, Write},
    time::{Duration, Instant},
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut columns = 144_u32;
    let mut rows = 50_u32;
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
    reactive_tui::graphics::FrameRequest::new(columns, rows, Duration::ZERO)?;
    if !std::io::stdout().is_terminal() {
        return Err("run in a real terminal, not a redirected writer".into());
    }
    let mut output = report
        .map(|path| OpenOptions::new().write(true).create_new(true).open(path))
        .transpose()?;
    let initialization_started = Instant::now();
    let reference = GpuCubeRenderer::new()?;
    if !reference.adapter_info().is_hardware {
        return Err("hardware GPU required for the comparison; software is not acceptance".into());
    }
    let mut renderer = HybridCubeRenderer::new(GraphicsOptions {
        force_cpu: cpu,
        fault: None,
        ..Default::default()
    });
    let mut cpu_reference = HybridCubeRenderer::new(GraphicsOptions {
        force_cpu: true,
        fault: None,
        ..Default::default()
    });
    let fixed = Duration::from_secs(1);
    let gpu_pixels = reference.render_terminal(columns, rows, fixed)?;
    let cpu_pixels = cpu_reference.render_terminal(columns, rows, fixed)?;
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
    // Warm up the actual selected renderer before the sampling deadline.
    let warmup = renderer.render_terminal(columns, rows, Duration::ZERO)?;
    let adapter = match warmup.mode() {
        GraphicsMode::Gpu(info) if !cpu && info.is_hardware => info.name.clone(),
        GraphicsMode::CpuFallback(_) if cpu => "CPU (explicit selection)".into(),
        other => return Err(format!("requested mode was not selected: {}", other.label()).into()),
    };
    let initialization_ms = initialization_started.elapsed().as_secs_f64() * 1000.0;
    let mut backend = SuprTuiBackend::new()?;
    if backend.size() != (columns as u16, rows as u16) {
        backend.shutdown()?;
        return Err("terminal dimensions must match --columns and --rows".into());
    }
    let started = Instant::now();
    let duration = Duration::from_secs_f64(seconds);
    let mut sums = [0.0_f64; 5];
    let mut count = 0_u64;
    while started.elapsed() < duration {
        let frame_started = Instant::now();
        let frame = renderer.render_terminal(columns, rows, started.elapsed())?;
        match frame.mode() {
            GraphicsMode::Gpu(info) if !cpu && info.is_hardware => {}
            GraphicsMode::CpuFallback(_) if cpu => {}
            other => {
                return Err(format!("mode changed during measurement: {}", other.label()).into())
            }
        }
        let conversion_started = Instant::now();
        let element = frame.to_half_block_element()?;
        let conversion = conversion_started.elapsed();
        let presentation_started = Instant::now();
        backend.render_frame(&element)?;
        backend.present()?;
        let presentation = presentation_started.elapsed();
        let timings = frame.timings();
        for (sum, elapsed) in sums.iter_mut().zip([
            timings.render,
            timings.readback,
            conversion,
            presentation,
            frame_started.elapsed(),
        ]) {
            *sum += elapsed.as_secs_f64() * 1000.0;
        }
        count += 1;
    }
    let sampled = started.elapsed().as_secs_f64();
    backend.shutdown()?;
    let averages = sums.map(|sum| sum / count as f64);
    let data = json!({
        "schema": 1,
        "adapter": adapter,
        "comparison_adapter": {"name": reference.adapter_info().name, "backend": reference.adapter_info().backend, "hardware": true},
        "terminal": {"term": std::env::var("TERM").unwrap_or_default(), "term_program": std::env::var("TERM_PROGRAM").unwrap_or_default(), "kitty_pid_present": std::env::var_os("KITTY_PID").is_some()},
        "viewport": {"columns": columns, "rows": rows},
        "mode": if cpu {"CPU"} else {"GPU"},
        "requested_seconds": seconds, "sampled_seconds": sampled, "frames": count,
        "achieved_fps": count as f64 / sampled, "initialization_and_quality_ms": initialization_ms,
        "mean_ms": {"render": averages[0], "readback": averages[1], "conversion": averages[2], "presentation": averages[3], "total": averages[4]},
        "quality": {"elapsed_seconds": 1, "differing_pixels": differing_pixels, "pixels": gpu_pixels.pixels().len(), "mean_absolute_rgb_error": absolute_error as f64 / (gpu_pixels.pixels().len() * 3) as f64, "max_channel_error": max_channel_error, "description": "Same shaded ray-box and sRGB output; measured floating-point/quantization differences, no CPU wireframe substitute."},
        "timing_scope": "Unpaced native backend: allocations and GPU completion; readback copy/map/depadding; styled Elements; layout/paint/ANSI writes. Excludes initialization, App reconciliation, terminal display scanout, and host acknowledgement. In-flight last frame may extend sampling past the requested duration. Demo requests are separately capped at 20 Hz."
    });
    let encoded = serde_json::to_string_pretty(&data)?;
    if let Some(file) = output.as_mut() {
        writeln!(file, "{encoded}")?;
        file.sync_all()?;
    }
    println!("{encoded}");
    Ok(())
}
