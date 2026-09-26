mod catalog;

use catalog::Catalog;
use reactive_tui::{
    app::App,
    backend::SuprTuiBackend,
    event::types::{KeyCode, KeyModifiers},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "wgpu-graphics")]
    let catalog = {
        use reactive_tui::graphics::{GraphicsFault, GraphicsOptions};
        let mut options = GraphicsOptions::default();
        let mut start_motion = false;
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--motion" => start_motion = true,
                "--cpu" => options.force_cpu = true,
                "--graphics-fault" => {
                    options.fault = Some(match args.next().as_deref() {
                        Some("adapter") => GraphicsFault::Adapter,
                        Some("device-loss") => GraphicsFault::DeviceLoss,
                        Some("readback") => GraphicsFault::Readback,
                        _ => {
                            return Err(
                                "--graphics-fault requires adapter, device-loss, or readback"
                                    .into(),
                            )
                        }
                    })
                }
                _ => return Err(format!("unknown catalog option: {arg}").into()),
            }
        }
        Catalog::with_graphics(options, start_motion)
    };
    #[cfg(not(feature = "wgpu-graphics"))]
    let catalog = Catalog::default();
    let backend = SuprTuiBackend::new()?;

    App::builder()
        .backend(backend)
        .root(catalog)
        .quit_key(
            KeyCode::Char('q'),
            KeyModifiers {
                ctrl: true,
                ..KeyModifiers::empty()
            },
        )
        .build()?
        .run()?;

    Ok(())
}
