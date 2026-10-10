use reactive_tui::core::surface::{Cell, Surface};
use reactive_tui::core::terminal::Terminal;
use reactive_tui::core::window::Window;
use reactive_tui::escape::parser::Parser;
use reactive_tui::event::EventRouter;
use reactive_tui::render::reconcile::apply_patches;
use reactive_tui::render::{NodeKey, PatchOp, RenderTree};
use std::process::ExitCode;

fn exercise_diagnostics() -> Result<(), String> {
    let mut exercised = Vec::new();

    let mut parser = Parser::new();
    let mut oversized_osc = b"\x1b]".to_vec();
    oversized_osc.resize(8_195, b'x');
    parser.feed(&oversized_osc);
    exercised.push("parser");

    let mut terminal = Terminal::new().map_err(|error| error.to_string())?;
    terminal
        .capability_gate()
        .map_err(|error| error.to_string())?;
    exercised.push("terminal");

    let mut tree = RenderTree::new();
    apply_patches(
        &[PatchOp::Update {
            node_key: NodeKey::named("dqc-003"),
        }],
        &mut tree,
    )
    .map_err(|error| error.to_string())?;
    exercised.push("reconciliation");

    let mut router = EventRouter::new();
    let focusable = router.create_node(None);
    router.add_focusable(focusable, Some(1_001));
    router.update_spatial(focusable, f32::NAN, 0.0, 1.0, 1.0);
    exercised.push("focus");

    let mut surface = Surface::new(1, 1);
    let window = Window::new(&mut surface);
    window.write_cell(0, 0, Cell::default());
    exercised.push("window");

    if let Some(report) = std::env::var_os("DQC003_REPORT") {
        std::fs::write(report, exercised.join("\n")).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn main() -> ExitCode {
    match exercise_diagnostics() {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}
