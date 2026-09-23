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
        Showcase::with_graphics(GraphicsOptions::default())
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
