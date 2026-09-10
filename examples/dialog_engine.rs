//! `cargo run --locked --example dialog_engine`
//! Enter opens an input dialog. Submit or cancel to restore the opener's focus.

use reactive_tui::{
    app::{App, RootComponent},
    backend::SuprTuiBackend,
    builder,
    component::Element,
    reactive::ThreadSafeSignal,
    widgets::dialog::{DialogEngine, InputDialogOptions},
};
use std::sync::Arc;

struct Example {
    dialogs: DialogEngine,
    result: ThreadSafeSignal<String>,
}
impl RootComponent for Example {
    fn render(&self) -> Element {
        let controller = self.dialogs.downgrade();
        let result = self.result.clone();
        let button = builder::button()
            .text("Open dialog")
            .on_click(move || {
                let Some(mut engine) = controller.upgrade() else {
                    return;
                };
                // This example displays results through the close callback. Consumers
                // can instead take synchronous events or enable async result handles.
                while engine.take_event().is_some() {}
                let completed = result.clone();
                if let Err(error) = engine.try_show_input(InputDialogOptions {
                    title: "Your name".into(),
                    prompt: "Enter a name, then choose OK".into(),
                    on_close: Some(Arc::new(move |value| completed.set(format!("{value:?}")))),
                    ..Default::default()
                }) {
                    result.set(error.to_string());
                }
            })
            .build()
            .auto_focus()
            .with_key("opener");
        builder::div()
            .class("flex-col w-full h-full")
            .children(vec![
                button,
                Element::text(self.result.get()),
                Element::text("Escape or Ctrl+C quits when no dialog is open."),
                self.dialogs.render().with_key("dialogs"),
            ])
            .build()
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

fn main() -> reactive_tui::error::Result<()> {
    App::builder()
        .backend(SuprTuiBackend::new()?)
        .root(Example {
            dialogs: DialogEngine::new(),
            result: ThreadSafeSignal::new("No result yet".into()),
        })
        .build()?
        .run()
}
