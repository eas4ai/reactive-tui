//! External tool renderer for chafa and viu
//!
//! Provides image rendering using external command-line tools like chafa and viu.
//! Based on moggu's approach with enhanced error handling and configuration.

use crate::error::{ReactiveError, Result};
use crate::widgets::display::image::{Image, ImageQuality, ImageSource};
#[cfg(test)]
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

/// External tool renderer for chafa and viu
pub struct ExternalRenderer;

// Only files created by this renderer may be removed after a command.
enum PreparedImage {
    Borrowed(std::path::PathBuf),
    Owned(tempfile::NamedTempFile),
}

impl PreparedImage {
    fn path(&self) -> &Path {
        match self {
            Self::Borrowed(path) => path,
            Self::Owned(file) => file.path(),
        }
    }
}

impl ExternalRenderer {
    /// Create a new external renderer
    pub fn new() -> Self {
        Self
    }

    /// Render image using chafa
    pub fn render_with_chafa(&self, image: &Image) -> Result<String> {
        if image.has_empty_size() {
            return Ok(String::new());
        }
        let file = self.prepare_temp_file(image)?;
        self.render_chafa_file(file.path(), image, || false)
    }

    /// Render image using viu
    pub fn render_with_viu(&self, image: &Image) -> Result<String> {
        if image.has_empty_size() {
            return Ok(String::new());
        }
        let pixels = super::decoded::load(&image.source)?;
        let mut config = image.clone();
        config.display_mode = super::ImageDisplayMode::Viu;
        self.render_pixels(&pixels, &config, || false)
    }

    /// Render already decoded pixels; App owns the source and actual layout box.
    pub(super) fn render_pixels(
        &self,
        pixels: &image::RgbaImage,
        config: &Image,
        cancelled: impl Fn() -> bool,
    ) -> Result<String> {
        if config.has_empty_size() {
            return Ok(String::new());
        }
        let mut config = config.clone();
        let prepared;
        let pixels = if config.display_mode == super::ImageDisplayMode::Viu {
            let (width, height) = self.get_display_constraints(&config)?;
            let (width, height) = if config.preserve_aspect {
                let ratio = (width as f64 / pixels.width() as f64)
                    .min(height as f64 * 2.0 / pixels.height() as f64);
                (
                    (pixels.width() as f64 * ratio).round().max(1.0) as u32,
                    (pixels.height() as f64 * ratio / 2.0).round().max(1.0) as u32,
                )
            } else {
                (width, height)
            };
            let pixel_height = height.checked_mul(2).ok_or_else(|| {
                ReactiveError::ImageProcessing("External image dimensions overflow".into())
            })?;
            super::decoded::dimensions(width, pixel_height)?;
            let filter = match config.quality {
                ImageQuality::Fast => image::imageops::FilterType::Nearest,
                ImageQuality::Balanced => image::imageops::FilterType::Triangle,
                ImageQuality::High => image::imageops::FilterType::Lanczos3,
            };
            let rgb = super::decoded::rgb_pixels(pixels, config.background_color);
            prepared = image::DynamicImage::ImageRgb8(image::imageops::resize(
                &rgb,
                width,
                pixel_height,
                filter,
            ))
            .into_rgba8();
            config.size_constraints = Some((width, height));
            &prepared
        } else {
            pixels
        };
        let mut encoded = std::io::Cursor::new(Vec::new());
        pixels
            .write_to(&mut encoded, image::ImageFormat::Png)
            .map_err(|error| {
                ReactiveError::ImageProcessing(format!(
                    "Cannot encode image for external renderer: {error}"
                ))
            })?;
        let file = self.write_temp_file(encoded.get_ref(), "png")?;
        match config.display_mode {
            super::ImageDisplayMode::Chafa => {
                self.render_chafa_file(file.path(), &config, cancelled)
            }
            super::ImageDisplayMode::Viu => self.render_viu_file(file.path(), &config, cancelled),
            _ => Err(ReactiveError::ImageProcessing(
                "External renderer requires Chafa or Viu mode".into(),
            )),
        }
    }

    /// Prepare a temporary file for external tools
    fn prepare_temp_file(&self, image: &Image) -> Result<PreparedImage> {
        match &image.source {
            ImageSource::FilePath(path) => {
                // Use the file directly if it exists
                if path.exists() {
                    Ok(PreparedImage::Borrowed(path.clone()))
                } else {
                    Err(ReactiveError::ImageProcessing(format!(
                        "Image file not found: {}",
                        path.display()
                    )))
                }
            }
            ImageSource::Url(url)
                if !url.starts_with("data:")
                    && !url.starts_with("http://")
                    && !url.starts_with("https://") =>
            {
                let path = Path::new(url);
                if !path.is_file() {
                    return Err(ReactiveError::ImageProcessing(format!(
                        "Image file not found: {}",
                        path.display()
                    )));
                }
                Ok(PreparedImage::Borrowed(path.to_owned()))
            }
            _ => {
                let pixels = super::decoded::load(&image.source)?;
                let mut encoded = std::io::Cursor::new(Vec::new());
                pixels
                    .write_to(&mut encoded, image::ImageFormat::Png)
                    .map_err(|error| {
                        ReactiveError::ImageProcessing(format!(
                            "Cannot encode image for external renderer: {error}"
                        ))
                    })?;
                self.write_temp_file(encoded.get_ref(), "png")
            }
        }
    }

    /// Write data to a temporary file
    fn write_temp_file(&self, data: &[u8], extension: &str) -> Result<PreparedImage> {
        let mut file = tempfile::Builder::new()
            .prefix("reactive_tui_image_")
            .suffix(&format!(".{extension}"))
            .tempfile()
            .map_err(|error| {
                ReactiveError::ImageProcessing(format!(
                    "Failed to create image temporary file: {error}"
                ))
            })?;
        file.write_all(data).map_err(|error| {
            ReactiveError::ImageProcessing(format!("Failed to write image temporary file: {error}"))
        })?;
        Ok(PreparedImage::Owned(file))
    }

    /// Render file using chafa
    fn render_chafa_file(
        &self,
        file_path: &Path,
        image: &Image,
        cancelled: impl Fn() -> bool,
    ) -> Result<String> {
        let (max_width, max_height) = self.get_display_constraints(image)?;

        let mut cmd = Command::new("chafa");
        cmd.args(["--format=symbols", "--threads=2"]);

        // Size constraints
        cmd.arg("--size")
            .arg(format!("{}x{}", max_width, max_height));

        // Quality settings based on image quality
        match image.quality {
            ImageQuality::Fast => {
                cmd.arg("--colors=16");
                cmd.arg("--symbols=ascii");
            }
            ImageQuality::Balanced => {
                cmd.arg("--colors=256");
                cmd.arg("--symbols=block");
            }
            ImageQuality::High => {
                cmd.arg("--colors=256");
                cmd.arg("--symbols=block+border+space");
                cmd.arg("--dither=bayer");
            }
        }

        // Additional options
        if !image.preserve_aspect {
            cmd.arg("--stretch");
        }
        cmd.arg("--animate=off");

        // Background color if specified
        if let Some(bg_color) = &image.background_color {
            let color_hex = format!(
                "#{:02x}{:02x}{:02x}",
                (bg_color.r * 255.0) as u8,
                (bg_color.g * 255.0) as u8,
                (bg_color.b * 255.0) as u8
            );
            cmd.arg("--bg").arg(color_hex);
        }

        cmd.arg("--").arg(file_path);

        Self::run_tool_cancelled(cmd, cancelled)
    }

    /// Render file using viu
    fn render_viu_file(
        &self,
        file_path: &Path,
        image: &Image,
        cancelled: impl Fn() -> bool,
    ) -> Result<String> {
        let (max_width, max_height) = self.get_display_constraints(image)?;

        let mut cmd = Command::new("viu");
        cmd.arg("--blocks");

        // Size constraints
        cmd.arg("-w").arg(max_width.to_string());
        cmd.arg("-h").arg(max_height.to_string());

        // Quality settings
        match image.quality {
            ImageQuality::Fast => {
                cmd.arg("--static");
            }
            ImageQuality::Balanced => {
                cmd.arg("--static");
            }
            ImageQuality::High => {
                cmd.arg("--static");
                // viu doesn't have as many quality options as chafa
            }
        }

        cmd.arg("--").arg(file_path);

        Self::run_tool_cancelled(cmd, cancelled)
    }

    #[cfg(test)]
    fn run_tool(command: Command) -> Result<String> {
        Self::run_tool_cancelled(command, || false)
    }

    fn run_tool_cancelled(command: Command, cancelled: impl Fn() -> bool) -> Result<String> {
        let output = crate::core::owned_process::run(
            command,
            None,
            crate::core::owned_process::Options {
                purpose: "Image renderer",
                timeout: std::time::Duration::from_secs(5),
                max_input: 0,
                max_output: 16 * 1024 * 1024,
                capture_output: true,
                allow_background_after_success: false,
            },
            cancelled,
        )
        .map_err(ReactiveError::ExternalTool)?;
        String::from_utf8(output).map_err(|error| {
            ReactiveError::ExternalTool(format!("Image renderer returned invalid UTF-8: {error}"))
        })
    }

    /// Get display size constraints
    fn get_display_constraints(&self, image: &Image) -> Result<(u32, u32)> {
        if let Some((width, height)) = image.size_constraints {
            Ok((width, height))
        } else {
            // Use terminal size as default
            let (term_cols, term_rows) =
                crate::core::terminal::Terminal::get_size().unwrap_or((80, 24));

            // Leave some margin for UI elements
            let max_width = (term_cols as u32).saturating_sub(4).max(20);
            let max_height = (term_rows as u32).saturating_sub(2).max(10);

            Ok((max_width, max_height))
        }
    }

    pub(super) fn available(tool: &str, cancelled: impl Fn() -> bool) -> bool {
        let mut command = Command::new(tool);
        command.arg("--version");
        Self::run_tool_cancelled(command, cancelled).is_ok()
    }

    /// Check if chafa is available
    pub fn is_chafa_available() -> bool {
        Self::available("chafa", || false)
    }

    /// Check if viu is available
    pub fn is_viu_available() -> bool {
        Self::available("viu", || false)
    }

    /// Get installation instructions for missing tools
    pub fn get_installation_help() -> String {
        let mut help = String::new();

        help.push_str("📷 Image display tools not found.\n\n");

        if !Self::is_chafa_available() {
            help.push_str("For the best image preview experience, install chafa:\n\n");
            help.push_str("• Arch Linux / Manjaro:     sudo pacman -S chafa\n");
            help.push_str("• Ubuntu / Debian:          sudo apt install chafa\n");
            help.push_str("• Fedora / RHEL / CentOS:   sudo dnf install chafa\n");
            help.push_str("• macOS (Homebrew):         brew install chafa\n");
            help.push_str("• Windows (WSL):            sudo apt install chafa\n\n");
        }

        if !Self::is_viu_available() {
            help.push_str("Alternative: Install viu (Rust-based):\n");
            help.push_str("• cargo install viu\n\n");
        }

        help.push_str("After installation, restart the application for best results!");

        help
    }
}

impl Default for ExternalRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::display::image::Image;

    #[cfg(unix)]
    #[test]
    fn api_image_external_render_preserves_caller_file() {
        const CHILD: &str = "REACTIVE_TUI_IMAGE_OWNERSHIP_CHILD";
        if let Ok(mode) = std::env::var(CHILD) {
            let source = tempfile::NamedTempFile::new().unwrap();
            let mut encoded = std::io::Cursor::new(Vec::new());
            image::RgbaImage::from_pixel(1, 1, image::Rgba([255, 0, 0, 255]))
                .write_to(&mut encoded, image::ImageFormat::Png)
                .unwrap();
            let original = encoded.into_inner();
            fs::write(source.path(), &original).unwrap();
            let renderer = ExternalRenderer::new();
            for (render, local_url) in [
                (SelfRender::Chafa, false),
                (SelfRender::Viu, false),
                (SelfRender::Chafa, true),
                (SelfRender::Viu, true),
            ] {
                let mut image = Image::from_file(source.path());
                if local_url {
                    image.source = ImageSource::Url(source.path().to_string_lossy().into_owned());
                }
                let result = match render {
                    SelfRender::Chafa => renderer.render_with_chafa(&image),
                    SelfRender::Viu => renderer.render_with_viu(&image),
                };
                assert_eq!(result.is_ok(), mode == "success", "{result:?}");
                assert_eq!(
                    fs::read(source.path()).expect("renderer deleted caller-owned image"),
                    original
                );
            }
            return;
        }
        use std::os::unix::fs::PermissionsExt;
        for mode in ["success", "failure", "unavailable"] {
            let tools = tempfile::tempdir().unwrap();
            if mode != "unavailable" {
                for name in ["chafa", "viu"] {
                    let path = tools.path().join(name);
                    let exit = if mode == "success" { 0 } else { 7 };
                    fs::write(
                        &path,
                        format!("#!/bin/sh\nprintf 'fixture image'\nexit {exit}\n"),
                    )
                    .unwrap();
                    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
                }
            }
            let output = Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "widgets::display::image::external_renderer::tests::api_image_external_render_preserves_caller_file", "--nocapture"])
                .env(CHILD, mode)
                .env("PATH", tools.path())
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{mode}: {}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn api_image_viu_prepares_aspect_background_and_stretched_pixels() {
        const CHILD: &str = "REACTIVE_IMAGE_VIU_PIXELS_CHILD";
        if std::env::var_os(CHILD).is_some() {
            let capture = std::env::var("REACTIVE_IMAGE_VIU_CAPTURE").unwrap();
            let renderer = ExternalRenderer::new();
            for preserve in [true, false] {
                let config = Image::from_raw_bytes(
                    vec![0; 2 * 2 * 4],
                    2,
                    2,
                    super::super::ImageFormat::RGBA8888,
                )
                .with_max_size(8, 2)
                .with_preserve_aspect(preserve)
                .with_background_color(crate::core::surface::Rgba::new(1.0, 0.0, 0.0, 1.0));
                renderer.render_with_viu(&config).unwrap();
                let pixels = image::open(&capture).unwrap().to_rgba8();
                assert_eq!(pixels.dimensions(), if preserve { (4, 4) } else { (8, 4) });
                assert!(pixels.pixels().all(|pixel| pixel.0 == [255, 0, 0, 255]));
            }
            return;
        }
        use std::os::unix::fs::PermissionsExt;
        let temp = tempfile::tempdir().unwrap();
        let program = temp.path().join("viu");
        fs::write(&program, "#!/bin/sh\nfor arg do last=\"$arg\"; done\n/bin/cp \"$last\" \"$REACTIVE_IMAGE_VIU_CAPTURE\"\nprintf 'image'\n").unwrap();
        fs::set_permissions(program, fs::Permissions::from_mode(0o700)).unwrap();
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "widgets::display::image::external_renderer::tests::api_image_viu_prepares_aspect_background_and_stretched_pixels", "--nocapture"])
            .env(CHILD,"1").env("PATH",temp.path())
            .env("REACTIVE_IMAGE_VIU_CAPTURE",temp.path().join("capture.png"))
            .output().unwrap();
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[cfg(unix)]
    enum SelfRender {
        Chafa,
        Viu,
    }

    #[test]
    fn api_image_external_commands_bound_resources() {
        const CHILD: &str = "REACTIVE_TUI_IMAGE_RESOURCE_CHILD";
        const PID_FILE: &str = "REACTIVE_TUI_IMAGE_RESOURCE_PID";
        if let Ok(mode) = std::env::var(CHILD) {
            fs::write(
                std::env::var_os(PID_FILE).unwrap(),
                std::process::id().to_string(),
            )
            .unwrap();
            match mode.as_str() {
                // The stalled renderer: it outlasts the 5 s renderer timeout.
                "stall" => std::thread::sleep(std::time::Duration::from_secs(30)),
                "flood" => std::io::stdout()
                    .write_all(&vec![b'x'; 17 * 1024 * 1024])
                    .unwrap(),
                "utf8" => std::io::stdout().write_all(&[255, 254]).unwrap(),
                "nonzero" => std::process::exit(7),
                _ => panic!("unknown fixture mode"),
            }
            return;
        }
        for (mode, expected) in [
            ("stall", "timed out"),
            ("flood", "16 MiB"),
            ("utf8", "invalid UTF-8"),
            ("nonzero", "exit"),
        ] {
            let pid_file = tempfile::NamedTempFile::new().unwrap();
            let mut command = Command::new(std::env::current_exe().unwrap());
            command.args(["--exact", "widgets::display::image::external_renderer::tests::api_image_external_commands_bound_resources", "--nocapture"])
                .env(CHILD, mode).env(PID_FILE, pid_file.path());
            let started = std::time::Instant::now();
            let error = ExternalRenderer::run_tool(command).unwrap_err().to_string();
            assert!(error.contains(expected), "{mode}: {error}");
            assert!(!error.contains("cleanup failed"), "{mode}: {error}");
            // The behavior under test: every mode ends by the renderer's 5 s
            // timeout, well before the stalled child's 30 s.
            assert!(
                started.elapsed() < std::time::Duration::from_secs(10),
                "{mode}"
            );
            #[cfg(unix)]
            {
                let pid: i32 = fs::read_to_string(pid_file.path())
                    .unwrap()
                    .parse()
                    .unwrap();
                assert_eq!(
                    unsafe { libc::kill(pid, 0) },
                    -1,
                    "owned child {pid} survived {mode}: {error}"
                );
                assert_eq!(
                    std::io::Error::last_os_error().raw_os_error(),
                    Some(libc::ESRCH)
                );
            }
        }
    }

    #[cfg(unix)]
    fn xis_002_validate_arguments(
        captures: &[serde_json::Value],
    ) -> std::result::Result<(), String> {
        if captures.len() != 2 {
            return Err(format!(
                "expected chafa and viu captures, got {}",
                captures.len()
            ));
        }
        for (capture, tool) in captures.iter().zip(["chafa", "viu"]) {
            if capture["tool"] != tool {
                return Err(format!("missing {tool} invocation"));
            }
            let arguments = capture["arguments"]
                .as_array()
                .ok_or_else(|| format!("{tool} capture has no arguments"))?;
            let Some(path) = arguments.iter().position(|value| value == "-fixture.png") else {
                return Err(format!("{tool} did not receive the fixture path"));
            };
            if path == 0 || arguments[path - 1] != "--" {
                return Err(format!("{tool} can parse the image path as an option"));
            }
        }
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn xis_002_argument_validator_rejects_unsafe_observations() {
        let valid = vec![
            serde_json::json!({"tool":"chafa", "arguments":["--format=symbols", "--", "-fixture.png"]}),
            serde_json::json!({"tool":"viu", "arguments":["--blocks", "--", "-fixture.png"]}),
        ];
        xis_002_validate_arguments(&valid).unwrap();
        let mut missing_separator = valid.clone();
        missing_separator[0]["arguments"]
            .as_array_mut()
            .unwrap()
            .remove(1);
        assert!(xis_002_validate_arguments(&missing_separator).is_err());
        let mut late_separator = valid.clone();
        late_separator[1]["arguments"] = serde_json::json!(["--blocks", "-fixture.png", "--"]);
        assert!(xis_002_validate_arguments(&late_separator).is_err());
        assert!(xis_002_validate_arguments(&valid[..1]).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn xis_002_external_paths_are_positional_data() {
        const CHILD: &str = "RTUI_XIS_002_RENDER_CHILD";
        const CAPTURE: &str = "RTUI_XIS_002_RENDER_CAPTURE";
        if std::env::var_os(CHILD).is_some() {
            let renderer = ExternalRenderer::new();
            let image = Image::default().with_max_size(8, 4);
            renderer
                .render_chafa_file(Path::new("-fixture.png"), &image, || false)
                .unwrap();
            renderer
                .render_viu_file(Path::new("-fixture.png"), &image, || false)
                .unwrap();
            let captures = fs::read_to_string(std::env::var_os(CAPTURE).unwrap())
                .unwrap()
                .lines()
                .map(|line| serde_json::from_str(line).unwrap())
                .collect::<Vec<_>>();
            if let Err(error) = xis_002_validate_arguments(&captures) {
                panic!("{error}");
            }
            return;
        }

        use std::os::unix::fs::PermissionsExt;
        let directory = tempfile::tempdir().unwrap();
        let capture = directory.path().join("capture.jsonl");
        for tool in ["chafa", "viu"] {
            let executable = directory.path().join(tool);
            let script = format!(
                r#"#!/usr/bin/python3
import json, sys
with open({}, "a", encoding="utf-8") as output:
    output.write(json.dumps({{"tool": {}, "arguments": sys.argv[1:]}}) + "\n")
sys.stdout.write("fixture")
"#,
                serde_json::to_string(&capture.to_string_lossy()).unwrap(),
                serde_json::to_string(tool).unwrap(),
            );
            fs::write(&executable, script).unwrap();
            fs::set_permissions(executable, fs::Permissions::from_mode(0o700)).unwrap();
        }
        let output = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "widgets::display::image::external_renderer::tests::xis_002_external_paths_are_positional_data",
                "--nocapture",
            ])
            .current_dir(directory.path())
            .env_clear()
            .env(CHILD, "1")
            .env(CAPTURE, &capture)
            .env("PATH", directory.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
    }

    #[test]
    fn test_get_display_constraints_with_size() {
        let renderer = ExternalRenderer::new();
        let image = Image::default().with_max_size(100, 50);

        let (width, height) = renderer.get_display_constraints(&image).unwrap();
        assert_eq!(width, 100);
        assert_eq!(height, 50);
    }

    #[test]
    fn test_get_display_constraints_default() {
        let renderer = ExternalRenderer::new();
        let image = Image::default();

        let (width, height) = renderer.get_display_constraints(&image).unwrap();
        // Should use terminal size with margins
        assert!(width >= 20);
        assert!(height >= 10);
    }

    #[test]
    fn api_image_external_raw_pixels_are_encoded_as_png() {
        let renderer = ExternalRenderer::new();
        let image = Image::from_raw_bytes(
            vec![255, 0, 0, 0, 255, 0],
            2,
            1,
            crate::widgets::display::image::ImageFormat::RGB888,
        );
        let file = renderer.prepare_temp_file(&image).unwrap();
        let decoded = image::open(file.path())
            .expect("external tool needs an encoded image")
            .to_rgba8();
        assert_eq!(decoded.dimensions(), (2, 1));
        assert_eq!(decoded.into_raw(), [255, 0, 0, 255, 0, 255, 0, 255]);
    }

    #[test]
    fn api_image_temporary_files_are_independent_and_owned() {
        let renderer = ExternalRenderer::new();
        let first = renderer.write_temp_file(b"first", "png").unwrap();
        let second = renderer.write_temp_file(b"second", "png").unwrap();
        assert_ne!(first.path(), second.path());
        let first_path = first.path().to_owned();
        assert_eq!(fs::read(first.path()).unwrap(), b"first");
        assert_eq!(fs::read(second.path()).unwrap(), b"second");
        drop(first);
        assert!(!first_path.exists());
        assert_eq!(fs::read(second.path()).unwrap(), b"second");
        let second_path = second.path().to_owned();
        drop(second);
        assert!(!second_path.exists());
    }

    #[test]
    fn test_write_temp_file() {
        let renderer = ExternalRenderer::new();
        let data = b"test data";

        let temp_file = renderer.write_temp_file(data, "txt").unwrap();
        assert!(temp_file.path().exists());

        let content = fs::read(temp_file.path()).unwrap();
        assert_eq!(content, data);

        let path = temp_file.path().to_owned();
        drop(temp_file);
        assert!(!path.exists());
    }
}
