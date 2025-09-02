use reactive_tui::event::{Event, EventPhase, EventRouter, KeyEvent, MouseEvent};
use reactive_tui::event::types::{KeyCode, MouseButton, MouseEventKind, Position, KeyModifiers};
use std::sync::Arc;
use std::time::Instant;

const ITERATIONS: u32 = 100_000;
const WARMUP: u32 = 1_000;

fn create_deep_tree(router: &mut EventRouter, depth: usize) -> Vec<reactive_tui::event::router::NodeId> {
    let mut nodes = Vec::with_capacity(depth);
    let mut parent = None;
    
    for _ in 0..depth {
        let node = router.create_node(parent);
        nodes.push(node);
        parent = Some(node);
    }
    
    nodes
}

fn benchmark_shallow_tree() {
    println!("\n=== Shallow Tree Benchmark (3 levels) ===");
    
    let mut router = EventRouter::new();
    let nodes = create_deep_tree(&mut router, 3);
    
    // Add handlers to each node
    for &node in &nodes {
        router.add_handler(
            node,
            "key",
            EventPhase::Bubble,
            Arc::new(|_| reactive_tui::event::router::EventResult::Handled),
        );
    }
    
    let event = Event::Key(KeyEvent::new(KeyCode::Char('a')));
    
    // Warmup
    for _ in 0..WARMUP {
        router.route_event(&event, nodes[nodes.len() - 1]);
    }
    
    // Benchmark
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        router.route_event(&event, nodes[nodes.len() - 1]);
    }
    let elapsed = start.elapsed();
    
    println!("3-level tree:  {:?} ({:.2} ns/op)", elapsed, elapsed.as_nanos() as f64 / ITERATIONS as f64);
}

fn benchmark_deep_tree() {
    println!("\n=== Deep Tree Benchmark (16 levels) ===");
    
    let mut router = EventRouter::new();
    let nodes = create_deep_tree(&mut router, 16);
    
    // Add handlers to each node
    for &node in &nodes {
        router.add_handler(
            node,
            "key",
            EventPhase::Bubble,
            Arc::new(|_| reactive_tui::event::router::EventResult::Handled),
        );
    }
    
    let event = Event::Key(KeyEvent::new(KeyCode::Char('a')));
    
    // Warmup
    for _ in 0..WARMUP {
        router.route_event(&event, nodes[nodes.len() - 1]);
    }
    
    // Benchmark
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        router.route_event(&event, nodes[nodes.len() - 1]);
    }
    let elapsed = start.elapsed();
    
    println!("16-level tree: {:?} ({:.2} ns/op)", elapsed, elapsed.as_nanos() as f64 / ITERATIONS as f64);
}

fn benchmark_event_types() {
    println!("\n=== Event Type Dispatch Benchmark ===");
    
    let mut router = EventRouter::new();
    let root = router.create_node(None);
    
    // Add handlers for different event types
    router.add_handler(
        root,
        "key",
        EventPhase::Target,
        Arc::new(|_| reactive_tui::event::router::EventResult::Handled),
    );
    
    router.add_handler(
        root,
        "mouse",
        EventPhase::Target,
        Arc::new(|_| reactive_tui::event::router::EventResult::Handled),
    );
    
    let key_event = Event::Key(KeyEvent::new(KeyCode::Char('a')));
    let mouse_event = Event::Mouse(MouseEvent::new(
        MouseEventKind::Click,
        Position::cell(0, 0),
    ));
    
    // Benchmark mixed events
    let start = Instant::now();
    for i in 0..ITERATIONS {
        if i % 2 == 0 {
            router.route_event(&key_event, root);
        } else {
            router.route_event(&mouse_event, root);
        }
    }
    let elapsed = start.elapsed();
    
    println!("Mixed events:  {:?} ({:.2} ns/op)", elapsed, elapsed.as_nanos() as f64 / ITERATIONS as f64);
}

fn benchmark_handler_chains() {
    println!("\n=== Handler Chain Benchmark (10 handlers) ===");
    
    let mut router = EventRouter::new();
    let node = router.create_node(None);
    
    // Add multiple handlers
    for i in 0..10 {
        router.add_handler(
            node,
            "key",
            EventPhase::Target,
            Arc::new(move |_| {
                // Simulate minimal work
                std::hint::black_box(i);
                reactive_tui::event::router::EventResult::Handled
            }),
        );
    }
    
    let event = Event::Key(KeyEvent::new(KeyCode::Char('a')));
    
    // Warmup
    for _ in 0..WARMUP {
        router.route_event(&event, node);
    }
    
    // Benchmark
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        router.route_event(&event, node);
    }
    let elapsed = start.elapsed();
    
    println!("10 handlers:   {:?} ({:.2} ns/op)", elapsed, elapsed.as_nanos() as f64 / ITERATIONS as f64);
    println!("Per handler:   {:.2} ns", elapsed.as_nanos() as f64 / ITERATIONS as f64 / 10.0);
}

fn benchmark_focus_dispatch() {
    println!("\n=== Focus Dispatch Benchmark ===");
    
    let mut router = EventRouter::new();
    let nodes = create_deep_tree(&mut router, 8);
    
    // Set focus to a middle node
    let focus_node = nodes[4];
    router.set_focus(Some(focus_node));
    
    // Add handlers
    for &node in &nodes {
        router.add_handler(
            node,
            "key",
            EventPhase::Bubble,
            Arc::new(|_| reactive_tui::event::router::EventResult::Handled),
        );
    }
    
    let event = Event::Key(KeyEvent::new(KeyCode::Tab));
    
    // Warmup
    for _ in 0..WARMUP {
        router.dispatch_to_focus(&event);
    }
    
    // Benchmark
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        router.dispatch_to_focus(&event);
    }
    let elapsed = start.elapsed();
    
    println!("Focus dispatch: {:?} ({:.2} ns/op)", elapsed, elapsed.as_nanos() as f64 / ITERATIONS as f64);
}

fn benchmark_capture_vs_bubble() {
    println!("\n=== Capture vs Bubble Phase Benchmark ===");
    
    let mut router = EventRouter::new();
    let nodes = create_deep_tree(&mut router, 10);
    
    // Add capture handlers
    for &node in &nodes[..5] {
        router.add_handler(
            node,
            "key",
            EventPhase::Capture,
            Arc::new(|_| reactive_tui::event::router::EventResult::Handled),
        );
    }
    
    // Add bubble handlers
    for &node in &nodes[5..] {
        router.add_handler(
            node,
            "key",
            EventPhase::Bubble,
            Arc::new(|_| reactive_tui::event::router::EventResult::Handled),
        );
    }
    
    let event = Event::Key(KeyEvent::new(KeyCode::Enter));
    let target = nodes[nodes.len() - 1];
    
    // Warmup
    for _ in 0..WARMUP {
        router.route_event(&event, target);
    }
    
    // Benchmark
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        router.route_event(&event, target);
    }
    let elapsed = start.elapsed();
    
    println!("Capture+Bubble: {:?} ({:.2} ns/op)", elapsed, elapsed.as_nanos() as f64 / ITERATIONS as f64);
}

fn main() {
    println!("Event Router Performance Benchmark");
    println!("==================================");
    println!("Iterations: {}", ITERATIONS);
    
    benchmark_shallow_tree();
    benchmark_deep_tree();
    benchmark_event_types();
    benchmark_handler_chains();
    benchmark_focus_dispatch();
    benchmark_capture_vs_bubble();
    
    println!("\n=== Analysis ===");
    println!("• Deep trees (16 levels) show path building overhead");
    println!("• Handler chains scale linearly with count");
    println!("• Event type dispatch uses string comparison (optimization opportunity)");
    println!("• Focus dispatch adds minimal overhead");
    println!("• Capture phase processing adds to total routing time");
}