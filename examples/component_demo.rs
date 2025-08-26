use reactive_tui::component::{Component, Element, Props, ComponentInstance, LifecycleEvent, ElementType};
use reactive_tui::core::renderer::Renderer;
use reactive_tui::core::surface::Rgba;
use reactive_tui::layout::paint_tree::{layout_and_paint, NodeSpec};

// Example: Counter component
struct Counter {
    count: i32,
}

#[derive(Clone, PartialEq, Debug)]
struct CounterProps {
    initial: i32,
    step: i32,
}

impl Default for CounterProps {
    fn default() -> Self {
        Self { initial: 0, step: 1 }
    }
}

impl Props for CounterProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Default)]
struct CounterState {
    clicks: usize,
}

impl Component for Counter {
    type Props = CounterProps;
    type State = CounterState;
    
    fn new(props: Self::Props) -> Self {
        Self {
            count: props.initial,
        }
    }
    
    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        // In a real app, this would handle events
        self.count += props.step;
        state.clicks += 1;
        true // Always re-render for demo
    }
    
    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        Element::text(format!(
            "Counter: {} (step: {}, clicks: {})",
            self.count, props.step, state.clicks
        ))
    }
    
    fn on_lifecycle(&mut self, event: LifecycleEvent, _state: &mut Self::State) {
        match event {
            LifecycleEvent::Mount => println!("Counter mounted!"),
            LifecycleEvent::Unmount => println!("Counter unmounting!"),
            _ => {}
        }
    }
}

// Example: Button component
struct Button {
    _label: String,
}

#[derive(Clone, PartialEq, Debug)]
struct ButtonProps {
    text: String,
    style: String,
}

impl Props for ButtonProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Component for Button {
    type Props = ButtonProps;
    type State = ();
    
    fn new(props: Self::Props) -> Self {
        Self {
            _label: props.text.clone(),
        }
    }
    
    fn render(&self, props: &Self::Props, _state: &Self::State) -> Element {
        // For now, return a simple text element
        // In the future, this would create a styled button
        Element::text(format!("[ {} ]", props.text))
    }
}

// Bridge function to render components using our existing layout system
fn render_component_tree() -> NodeSpec<'static> {
    // Create component instances
    let mut counter = ComponentInstance::<Counter>::new(CounterProps {
        initial: 0,
        step: 1,
    });
    
    let button = ComponentInstance::<Button>::new(ButtonProps {
        text: "Click Me".to_string(),
        style: "bg-blue-500 text-white p-2".to_string(),
    });
    
    // Mount components
    counter.mount();
    
    // Simulate an update
    counter.update_props(CounterProps {
        initial: 0,
        step: 2,
    });
    
    // Render to get text content
    let counter_element = counter.render();
    let button_element = button.render();
    
    // Extract text for demo (in real implementation, we'd convert Elements to NodeSpec)
    let counter_text = match counter_element.element_type {
        ElementType::Text(text) => text,
        _ => "Counter".to_string(),
    };
    
    let button_text = match button_element.element_type {
        ElementType::Text(text) => text,
        _ => "Button".to_string(),
    };
    
    // Unmount when done
    counter.unmount();
    use std::borrow::Cow;
    // Create a layout using our existing system
    NodeSpec {
        class: Cow::from("flex flex-col gap-2 p-2"),
        text: None,
        children: vec![
            NodeSpec {
                class: Cow::from("bg-gray-800 text-white p-2"),
                text: Some(Cow::from(counter_text)),
                children: vec![],
            },
            NodeSpec {
                class: std::borrow::Cow::from("bg-blue-500 text-white p-2"),
                text: Some(std::borrow::Cow::from(button_text)),
                children: vec![],
            },
        ],
    }
}

fn main() -> std::io::Result<()> {
    let mut r = Renderer::new(80, 24)?;
    r.begin_frame()?;
    r.clear(Rgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 });
    
    // Render component tree
    let tree = render_component_tree();
    let surf = r.surface_mut();
    layout_and_paint(&tree, surf, 80);
    
    r.end_frame()?;
    std::thread::sleep(std::time::Duration::from_secs(2));
    r.shutdown()?;
    Ok(())
}