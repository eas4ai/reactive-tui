mod showcase;

use reactive_tui::{
    app::App,
    backend::SuprTuiBackend,
    event::types::{KeyCode, KeyModifiers},
};
use showcase::Showcase;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "wgpu-graphics")]
    let showcase = {
        use reactive_tui::graphics::GraphicsOptions;
        let mut options = GraphicsOptions::default();
        for arg in std::env::args().skip(1) {
            match arg.as_str() {
                "--cpu" => options.force_cpu = true,
                _ => return Err(format!("unknown showcase option: {arg}").into()),
            }
        }
        Showcase::with_graphics(options)
    };
    #[cfg(not(feature = "wgpu-graphics"))]
    let showcase = Showcase::default();
    let backend = SuprTuiBackend::new()?;

    App::builder()
        .backend(backend)
        .root(showcase)
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
