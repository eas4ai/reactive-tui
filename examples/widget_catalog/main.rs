mod catalog;

use catalog::Catalog;
use reactive_tui::{app::App, backend::SuprTuiBackend};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let backend = SuprTuiBackend::new()?;

    App::builder()
        .backend(backend)
        .root(Catalog::default())
        .build()?
        .run()?;

    Ok(())
}
