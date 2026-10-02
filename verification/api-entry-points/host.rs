use reactive_tui::{
    app::{App, AppBuilder, RootComponent},
    backend::{direct_tty::DirectTtyBackend, CrosstermBackend, SuprTuiBackend},
    builder,
    component::{Element, LayoutType},
    error::{ReactiveError, Result},
    event::{
        router::EventResult,
        types::{Event, KeyCode},
    },
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

struct Root {
    phase: usize,
    clicks: Arc<AtomicUsize>,
    size: (u16, u16),
}

impl RootComponent for Root {
    fn render(&self) -> Element {
        let text = |key, value: &str| Element::text(value).with_key(key).with_class("w-full h-1");
        let clicks = self.clicks.clone();
        let phase = ["alpha", "omega", "reversed", "empty"][self.phase];
        let mut children = vec![
            text(
                "state",
                &format!("state {phase} clicks{}", clicks.load(Ordering::SeqCst)),
            ),
            builder::button()
                .key("activate")
                .class("w-12 h-1 p-0")
                .text("Activate")
                .on_click(move || {
                    clicks.fetch_add(1, Ordering::SeqCst);
                })
                .build(),
            text("size", &format!("size {} {}", self.size.0, self.size.1)),
        ];
        if self.phase != 3 {
            children.push(text("unicode", "e\u{301}界👩‍💻"));
            let mut items = vec![text("north", "North"), text("south", "South")];
            if self.phase == 2 {
                items.reverse();
            }
            children.extend(items);
        }
        Element::layout(LayoutType::Flex)
            .with_key("root")
            .with_class("flex flex-col w-full h-full")
            .with_children(children)
    }
    fn resize(&mut self, width: u16, height: u16) -> Result<()> {
        self.size = (width, height);
        Ok(())
    }
    fn try_handle_event(&mut self, event: &Event) -> Result<EventResult> {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('x') => self.phase = 1,
                KeyCode::Char('r') => self.phase = 2,
                KeyCode::Char('e') => self.phase = 3,
                KeyCode::Char('f') => {
                    return Err(ReactiveError::terminal("entry point controlled root error"))
                }
                _ => return Ok(EventResult::Ignored),
            }
            return Ok(EventResult::Handled);
        }
        Ok(EventResult::Ignored)
    }
}

fn backend(route: &str) -> Result<AppBuilder> {
    let builder = App::builder().screen_reader(false);
    Ok(match route {
        "suprtui" => builder.backend(SuprTuiBackend::new()?),
        "crossterm" => builder.backend(CrosstermBackend::new()?),
        "crossterm-surface" => {
            let mut backend = CrosstermBackend::new()?;
            backend.set_use_render_ops(false);
            builder.backend(backend)
        }
        "direct" => builder.backend(DirectTtyBackend::new()?),
        _ => {
            return Err(ReactiveError::invalid_parameter(
                "unknown entry-point fixture route",
            ))
        }
    })
}

fn main() {
    let route = std::env::args().nth(1).expect("backend route");
    let result = backend(&route)
        .and_then(|builder| {
            builder
                .root(Root {
                    phase: 0,
                    clicks: Arc::new(AtomicUsize::new(0)),
                    size: (0, 0),
                })
                .build()
        })
        .and_then(App::run);
    match result {
        Ok(()) => println!("ENTRY_POINT_CLEAN_EXIT"),
        Err(error) => {
            eprintln!("ENTRY_POINT_ERROR: {error}");
            std::process::exit(1);
        }
    }
}
