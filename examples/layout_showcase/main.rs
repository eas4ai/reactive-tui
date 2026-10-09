mod layouts;

use layouts::Showcase;
use reactive_tui::{
    app::App,
    backend::SuprTuiBackend,
    event::types::{KeyCode, KeyModifiers},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
