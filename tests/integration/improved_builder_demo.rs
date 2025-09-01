//! Improved Builder Demo
//!
//! Shows off the enhanced builder API with:
//! - Auto-build with Into<Element>
//! - Composition macros (div![], span![], etc.)
//! - Component shortcuts
//! - Integration with existing utility CSS

use reactive_tui::prelude::*;

use reactive_tui::{div, span, button, input};

struct BuilderDemoApp;

impl reactive_tui::app::RootComponent for BuilderDemoApp {
    fn render(&self) -> Element {
        // Use the improved builder API
        div()
            .class("h-screen bg-gray-100 p-4")
            .children(vec![
                // Header using composition macro
                div![
                    class: "bg-white rounded-lg shadow p-6 mb-4",
                    h1().class("text-2xl font-bold text-gray-800").text("🚀 Improved Builder Demo"),
                    p().class("text-gray-600 mt-2").text("Showcasing the enhanced builder API")
                ],

                // Layout examples
                div()
                    .class("grid grid-cols-2 gap-4 mb-4")
                    .children(vec![
                        // Left column - Composition macros
                        div![
                            class: "bg-white rounded-lg shadow p-4",
                            h2().class("text-lg font-semibold mb-3").text("Composition Macros"),
                            div![
                                class: "space-y-2",
                                div!["Simple div with text"],
                                span![class: "text-blue-600", "Styled span"],
                                button!["Click me!"],
                                input![placeholder: "Type here..."]
                            ]
                        ],

                        // Right column - Component shortcuts
                        div()
                            .class("bg-white rounded-lg shadow p-4")
                            .children(vec![
                                h2().class("text-lg font-semibold mb-3").text("Component Shortcuts").build(),
                                
                                // Card component
                                card(vec![
                                    h3().class("font-medium").text("Card Component").build(),
                                    text("This is a pre-styled card with padding and shadow")
                                ]),

                                // Flex layouts
                                flex_row("2", vec![
                                    div().class("bg-blue-100 p-2 rounded").text("Item 1").build(),
                                    div().class("bg-green-100 p-2 rounded").text("Item 2").build(),
                                    div().class("bg-red-100 p-2 rounded").text("Item 3").build(),
                                ]),

                                // Search input
                                search_input("Search anything...")
                            ])
                            .build()
                    ])
                    .build(),

                // Auto-build demonstration
                div()
                    .class("bg-white rounded-lg shadow p-4")
                    .children(vec![
                        h2().class("text-lg font-semibold mb-3").text("Auto-Build with Into<Element>").build(),
                        p().class("text-gray-600 mb-2").text("No more .build() calls needed in many cases!").build(),
                        
                        // This function accepts anything that implements Into<Element>
                        demo_function(div().class("bg-yellow-100 p-2 rounded").text("Auto-converted!")),
                        demo_function("Just a string"),
                        demo_function(span().class("text-purple-600").text("Builder without .build()"))
                    ])
                    .build(),

                // Footer
                div()
                    .class("mt-4 text-center text-gray-500")
                    .text("Built with the improved reactive-tui builder API")
                    .build()
            ])
            .build()
    }
}

// Helper function that accepts anything convertible to Element
fn demo_function(element: impl IntoElement) -> Element {
    div()
        .class("mb-2 p-2 border border-gray-200 rounded")
        .child(element.into_element())
        .build()
}

fn main() -> reactive_tui::Result<()> {
    println!("🚀 Starting Improved Builder Demo...");
    
    let backend = reactive_tui::backend::CrosstermBackend::new()?;
    let app = reactive_tui::app::App::builder()
        .backend(backend)
        .root(BuilderDemoApp)
        .build()?;

    app.run()
}
