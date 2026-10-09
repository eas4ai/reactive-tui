//! Display-only controls for the real reader workflow.
use reactive_tui::{
    builder, component::Element, widgets::display::progress_bar::ProgressBarBuilder,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

pub(super) fn progress(value: &Arc<AtomicUsize>) -> Element {
    let advance = value.clone();
    builder::div()
        .class("w-full h-full flex flex-col")
        .children(vec![
            ProgressBarBuilder::new()
                .label("Download progress")
                .range(0.0, 200.0)
                .value(value.load(Ordering::SeqCst) as f64)
                .animated(false)
                .render(),
            builder::button()
                .text("Advance download")
                .class("w-20 h-1 p-0")
                .on_click(move || {
                    advance.store(150, Ordering::SeqCst);
                })
                .build()
                .auto_focus(),
            ProgressBarBuilder::new()
                .label("Waiting for download")
                .indeterminate(true)
                .style("reduced-motion")
                .render(),
            ProgressBarBuilder::new()
                .label("Invalid download")
                .range(100.0, 0.0)
                .render(),
            ProgressBarBuilder::new()
                .label("Completed download")
                .range(0.0, 200.0)
                .value(250.0)
                .animated(false)
                .render(),
        ])
        .build()
}

pub(super) fn chart() -> Element {
    use reactive_tui::widgets::display::{Chart, ChartProps, DataPoint, DataSeries};
    Element::typed::<Chart>(ChartProps {
        title: Some("Request latency".into()),
        width: 40,
        height: 12,
        series: vec![DataSeries::new(
            "Latency",
            vec![
                DataPoint::with_label(2.0, "Low"),
                DataPoint::with_label(8.0, "High").with_metadata("unit", "ms"),
            ],
        )],
        show_tooltips: true,
        ..Default::default()
    })
    .auto_focus()
}

pub(super) fn scroll() -> Element {
    use reactive_tui::widgets::layout::ScrollViewBuilder;
    let content = builder::div()
        .class("w-30 flex flex-col")
        .children(vec![
            Element::text("Start of report\nRow two\nRow three\nRow four\nRow five\nRow six")
                .class("w-30 h-6 flex-none whitespace-pre"),
            builder::text_input()
                .value("draft")
                .class("w-20 h-1 flex-none")
                .build()
                .with_accessibility_label("Report annotation"),
        ])
        .build();
    ScrollViewBuilder::new(content)
        .viewport_size(32, 3)
        .scroll_x(false)
        .show_scrollbars(false)
        .render()
        .with_accessibility_label("Report viewport")
        .auto_focus()
}

pub(super) fn image(value: &Arc<AtomicUsize>) -> Element {
    use reactive_tui::widgets::{ImageDisplayMode, ImageFormat, ImageQuality};
    let black = value.load(Ordering::SeqCst) != 0;
    let change = value.clone();
    builder::div()
        .class("w-full h-full flex flex-col")
        .children(vec![
            builder::button()
                .text("Change sample")
                .class("w-20 h-1 p-0")
                .on_click(move || change.store(0, Ordering::SeqCst))
                .build()
                .auto_focus(),
            builder::image()
                .source_raw_bytes(
                    vec![if black { 0 } else { 255 }; 12],
                    2,
                    2,
                    ImageFormat::RGB888,
                )
                .display_mode(ImageDisplayMode::AsciiArt)
                .quality(ImageQuality::Fast)
                .class("w-8 h-4")
                .build()
                .with_key("sample-image")
                .with_accessibility_label(if black {
                    "Black square sample"
                } else {
                    "White square sample"
                }),
        ])
        .build()
}

pub(super) fn terminal_fixture() -> std::io::Result<tempfile::TempDir> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("reader-shell");
    std::fs::write(&path, "#!/bin/sh\nstty -echo\nprintf 'Ready for command\\r\\n'\nwhile IFS= read -r value; do printf '\\033[32mReceived:%s\\033[0m\\r\\n' \"$value\"; done\n")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(dir)
}

pub(super) fn terminal(dir: &std::path::Path) -> Element {
    use reactive_tui::widgets::{TerminalProps, TerminalWidget};
    let terminal = Element::typed::<TerminalWidget>(TerminalProps {
        title: "Build terminal".into(),
        shell_command: Some(dir.join("reader-shell").to_string_lossy().into_owned()),
        auto_focus: true,
        ..Default::default()
    })
    .class("w-40 h-8")
    .with_accessibility_label("Build terminal");
    builder::div()
        .class("w-full h-full flex flex-col")
        .children(vec![
            terminal,
            Element::text("Outside terminal")
                .class("w-20 h-1")
                .with_focus(reactive_tui::component::FocusProps::input()),
        ])
        .build()
}
