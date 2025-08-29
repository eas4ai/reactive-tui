use criterion::{Criterion, black_box, criterion_group, criterion_main};
use reactive_tui::app::{App, RootComponent};
use reactive_tui::backend::DebugBackend;
use reactive_tui::component::Element;
use reactive_tui::core::renderer::Renderer;
use reactive_tui::core::surface::{Attr, Rgba, Surface};
use reactive_tui::event::router::{EventRouter, NodeId};
use reactive_tui::event::types::{Event, KeyCode, KeyEvent};
use std::sync::{Arc, Mutex};

/// Simple test component for benchmarking
#[derive(Clone)]
struct BenchComponent {
    counter: Arc<Mutex<usize>>,
}

impl BenchComponent {
    fn new() -> Self {
        Self {
            counter: Arc::new(Mutex::new(0)),
        }
    }
}

impl RootComponent for BenchComponent {
    fn render(&self) -> Element {
        let count = {
            let mut counter = self.counter.lock().unwrap();
            *counter += 1;
            *counter
        };
        Element::text(format!("Counter: {}", count))
    }
}

fn bench_surface_creation(c: &mut Criterion) {
    c.bench_function("surface_creation_80x24", |b| {
        b.iter(|| black_box(Surface::new(80, 24)))
    });

    c.bench_function("surface_creation_120x40", |b| {
        b.iter(|| black_box(Surface::new(120, 40)))
    });
}

fn bench_surface_write_operations(c: &mut Criterion) {
    let mut surface = Surface::new(80, 24);

    c.bench_function("surface_write_str", |b| {
        b.iter(|| {
            surface.write_str(
                black_box(0),
                black_box(0),
                black_box("Hello, World!"),
                black_box(Rgba::white()),
                black_box(Rgba::black()),
                black_box(Attr::empty()),
            )
        })
    });

    c.bench_function("surface_write_str_long", |b| {
        b.iter(|| {
            surface.write_str(
                black_box(0),
                black_box(0),
                black_box(
                    "This is a longer string that might be more representative of real usage",
                ),
                black_box(Rgba::white()),
                black_box(Rgba::black()),
                black_box(Attr::empty()),
            )
        })
    });
}

fn bench_renderer_operations(c: &mut Criterion) {
    c.bench_function("renderer_creation", |b| {
        b.iter(|| black_box(Renderer::new(80, 24).unwrap()))
    });

    let mut renderer = Renderer::new(80, 24).unwrap();
    c.bench_function("renderer_frame_cycle", |b| {
        b.iter(|| {
            renderer.begin_frame().unwrap();
            let surface = renderer.surface_mut();
            surface.write_str(
                0,
                0,
                "Benchmark",
                Rgba::white(),
                Rgba::black(),
                Attr::empty(),
            );
            renderer.end_frame().unwrap();
        })
    });
}

fn bench_event_processing(c: &mut Criterion) {
    let mut router = EventRouter::new_with_size(80, 24);
    let root = NodeId::new();
    router.set_root(root);
    router.add_focusable(root);

    let key_event = Event::Key(KeyEvent::new(KeyCode::Space));
    c.bench_function("event_key_processing", |b| {
        b.iter(|| black_box(router.process_event(black_box(&key_event))))
    });

    let tab_event = Event::Key(KeyEvent::new(KeyCode::Tab));
    c.bench_function("event_focus_traversal", |b| {
        b.iter(|| black_box(router.process_event(black_box(&tab_event))))
    });
}

fn bench_component_operations(c: &mut Criterion) {
    let component = BenchComponent::new();

    c.bench_function("component_render", |b| {
        b.iter(|| black_box(component.render()))
    });

    c.bench_function("app_creation", |b| {
        b.iter(|| {
            let backend = DebugBackend::new(80, 24);
            let component = BenchComponent::new();
            black_box(
                App::builder()
                    .backend(backend)
                    .root(component)
                    .build()
                    .unwrap(),
            )
        })
    });
}

fn bench_string_operations(c: &mut Criterion) {
    c.bench_function("string_formatting", |b| {
        b.iter(|| black_box(format!("Counter: {}", black_box(42))))
    });

    c.bench_function("string_formatting_complex", |b| {
        b.iter(|| {
            black_box(format!(
                "Renders: {} | Keys: {:?} | Mouse: {:?} | Focus: {}",
                black_box(42),
                black_box(vec![KeyCode::Space, KeyCode::Enter]),
                black_box(vec![(10, 20), (30, 40)]),
                black_box("FOCUSED")
            ))
        })
    });
}

fn bench_memory_operations(c: &mut Criterion) {
    c.bench_function("vector_creation_small", |b| {
        b.iter(|| {
            let mut vec = Vec::with_capacity(10);
            for i in 0..10 {
                vec.push(black_box(i));
            }
            black_box(vec)
        })
    });

    c.bench_function("vector_creation_large", |b| {
        b.iter(|| {
            let mut vec = Vec::with_capacity(1000);
            for i in 0..1000 {
                vec.push(black_box(i));
            }
            black_box(vec)
        })
    });

    c.bench_function("arc_mutex_operations", |b| {
        let counter = Arc::new(Mutex::new(0));
        b.iter(|| {
            let mut count = counter.lock().unwrap();
            *count += 1;
            black_box(*count)
        })
    });
}

fn bench_realistic_workload(c: &mut Criterion) {
    c.bench_function("typical_ui_update", |b| {
        b.iter(|| {
            // Simulate a typical UI update cycle
            let mut surface = Surface::new(80, 24);

            // Clear background
            surface.clear(Rgba::black());

            // Write some text (simulating UI elements)
            surface.write_str(0, 0, "Title Bar", Rgba::white(), Rgba::black(), Attr::BOLD);
            surface.write_str(
                0,
                2,
                "Content Area",
                Rgba::white(),
                Rgba::black(),
                Attr::empty(),
            );
            surface.write_str(
                0,
                20,
                "Status: Ready",
                Rgba::white(),
                Rgba::black(),
                Attr::empty(),
            );

            // Process an event
            let mut router = EventRouter::new_with_size(80, 24);
            let root = NodeId::new();
            router.set_root(root);
            let key_event = Event::Key(KeyEvent::new(KeyCode::Space));
            router.process_event(&key_event);

            black_box((surface, router))
        })
    });

    c.bench_function("rapid_event_processing", |b| {
        b.iter(|| {
            let mut router = EventRouter::new_with_size(80, 24);
            let root = NodeId::new();
            router.set_root(root);
            router.add_focusable(root);

            // Process multiple events rapidly
            for i in 0..10 {
                let event = if i % 2 == 0 {
                    Event::Key(KeyEvent::new(KeyCode::Char('a')))
                } else {
                    Event::Key(KeyEvent::new(KeyCode::Space))
                };
                black_box(router.process_event(&event));
            }
        })
    });
}

criterion_group!(
    benches,
    bench_surface_creation,
    bench_surface_write_operations,
    bench_renderer_operations,
    bench_event_processing,
    bench_component_operations,
    bench_string_operations,
    bench_memory_operations,
    bench_realistic_workload
);
criterion_main!(benches);
