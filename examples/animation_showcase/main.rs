mod showcase;

use reactive_tui::{
    app::App,
    backend::SuprTuiBackend,
    event::types::{KeyCode, KeyModifiers},
};
use showcase::Showcase;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "wgpu-graphics")]
    let showcase = Showcase::with_graphics(showcase::graphics_from(std::env::args().skip(1))?);
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
