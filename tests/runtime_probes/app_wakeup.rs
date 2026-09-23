//! Internal signal-driven application wakeup acceptance probe.

use reactive_tui::{
    app::{App, RootComponent},
    backend::SuprTuiBackend,
    component::Element,
    reactive::ThreadSafeSignal,
};
use std::time::Duration;

struct Counter(ThreadSafeSignal<usize>);

impl RootComponent for Counter {
    fn render(&self) -> Element {
        Element::text(format!("Wake count: {}", self.0.get()))
    }

    fn wake_driven(&self) -> bool {
        true
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let count = ThreadSafeSignal::new(0);
    let app = App::builder()
        .backend(SuprTuiBackend::new()?)
        .root(Counter(count.clone()))
        .build()?;
    let worker = if std::env::args().any(|arg| arg == "--probe") {
        Some(std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(200));
            count.set(1);
        }))
    } else {
        app.scheduler()
            .schedule_interval(Duration::from_millis(500), move || {
                count.update(|value| *value += 1)
            });
        None
    };
    let result = app.run();
    if let Some(worker) = worker {
        worker.join().map_err(|_| "counter worker panicked")?;
    }
    result.map_err(Into::into)
}
