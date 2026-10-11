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
        let (options, start_motion) = catalog::graphics_from(std::env::args().skip(1))?;
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
