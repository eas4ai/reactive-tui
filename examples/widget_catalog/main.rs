mod catalog;

use catalog::Catalog;
use reactive_tui::{
    app::App,
    backend::SuprTuiBackend,
    event::types::{KeyCode, KeyModifiers},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let backend = SuprTuiBackend::new()?;

    App::builder()
        .backend(backend)
        .root(Catalog::default())
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
