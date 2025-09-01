//! Z-Index Layering Demo
//!
//! Demonstrates proper z-index layering for:
//! - Modals and overlays
//! - Popovers and dropdowns  
//! - Tooltips and notifications
//! - Sticky headers and sidebars

use reactive_tui::builder::*;
use reactive_tui::prelude::*;

struct LayeringDemoApp {
    show_modal: bool,
    show_popover: bool,
    show_dropdown: bool,
}

impl LayeringDemoApp {
    fn new() -> Self {
        Self {
            show_modal: false,
            show_popover: false,
            show_dropdown: false,
        }
    }
}

impl reactive_tui::app::RootComponent for LayeringDemoApp {
    fn render(&self) -> Element {
        div()
            .class("h-screen bg-gray-100 relative")
            .children(vec![
                // Base content layer (z-0)
                self.render_main_content(),
                // Dropdown layer (z-10)
                if self.show_dropdown {
                    self.render_dropdown()
                } else {
                    Element::empty()
                },
                // Popover layer (z-20)
                if self.show_popover {
                    self.render_popover()
                } else {
                    Element::empty()
                },
                // Modal backdrop (z-40)
                if self.show_modal {
                    self.render_modal_backdrop()
                } else {
                    Element::empty()
                },
                // Modal content (z-50)
                if self.show_modal {
                    self.render_modal()
                } else {
                    Element::empty()
                },
                // Notification layer (z-60) - always on top
                self.render_notifications(),
            ])
            .build()
    }
}

impl LayeringDemoApp {
    fn render_main_content(&self) -> Element {
        div()
            .class("static z-0 p-6")
            .children(vec![
                // Sticky header (z-20)
                div()
                    .class("sticky z-20 top-0 bg-white shadow-md p-4 mb-6")
                    .children(vec![
                        h1().class("text-2xl font-bold").text("🎯 Z-Index Layering Demo").build(),
                        p().class("text-gray-600").text("Proper layering for modals, popovers, and overlays").build(),
                    ])
                    .build(),
                
                // Main content
                div()
                    .class("relative z-0 space-y-4")
                    .children(vec![
                        // Button row
                        div()
                            .class("flex gap-4 mb-6")
                            .children(vec![
                                button()
                                    .class("px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700")
                                    .text("Show Modal (z-50)")
                                    .build(),
                                
                                div()
                                    .class("relative")
                                    .children(vec![
                                        button()
                                            .class("px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700")
                                            .text("Show Dropdown (z-10)")
                                            .build(),
                                    ])
                                    .build(),
                                
                                button()
                                    .class("px-4 py-2 bg-purple-600 text-white rounded hover:bg-purple-700")
                                    .text("Show Popover (z-20)")
                                    .build(),
                            ])
                            .build(),
                        
                        // Content cards
                        self.render_content_cards(),
                        
                        // Sidebar (z-10)
                        div()
                            .class("absolute z-10 right-0 top-20 w-64 bg-white shadow-lg p-4")
                            .children(vec![
                                h3().class("font-semibold mb-2").text("Sidebar (z-10)").build(),
                                p().class("text-sm text-gray-600").text("This sidebar floats above content but below modals").build(),
                            ])
                            .build(),
                    ])
                    .build(),
            ])
            .build()
    }

    fn render_content_cards(&self) -> Element {
        div()
            .class("grid grid-cols-2 gap-4")
            .children(vec![
                card(vec![
                    h3().class("font-semibold mb-2")
                        .text("Base Content (z-0)")
                        .build(),
                    text("This is the main content layer. Everything else floats above it."),
                ]),
                card(vec![
                    h3().class("font-semibold mb-2")
                        .text("Layering Order")
                        .build(),
                    div()
                        .class("space-y-1 text-sm")
                        .children(vec![
                            div().text("• Base content: z-0").build(),
                            div().text("• Dropdowns: z-10").build(),
                            div().text("• Popovers: z-20").build(),
                            div().text("• Modal backdrop: z-40").build(),
                            div().text("• Modal content: z-50").build(),
                            div().text("• Notifications: z-60").build(),
                        ])
                        .build(),
                ]),
            ])
            .build()
    }

    fn render_dropdown(&self) -> Element {
        div()
            .class("absolute z-10 top-32 left-40 bg-white shadow-lg border rounded-lg p-2 min-w-48")
            .children(vec![
                div()
                    .class("px-3 py-2 hover:bg-gray-100 cursor-pointer")
                    .text("Dropdown Item 1")
                    .build(),
                div()
                    .class("px-3 py-2 hover:bg-gray-100 cursor-pointer")
                    .text("Dropdown Item 2")
                    .build(),
                div()
                    .class("px-3 py-2 hover:bg-gray-100 cursor-pointer")
                    .text("Dropdown Item 3")
                    .build(),
                div()
                    .class("border-t mt-2 pt-2 px-3 py-1 text-xs text-gray-500")
                    .text("z-index: 10")
                    .build(),
            ])
            .build()
    }

    fn render_popover(&self) -> Element {
        div()
            .class("absolute z-20 top-32 left-80 bg-white shadow-xl border rounded-lg p-4 w-64")
            .children(vec![
                div()
                    .class("flex items-center justify-between mb-2")
                    .children(vec![
                        h4().class("font-semibold").text("Popover (z-20)").build(),
                        button()
                            .class("text-gray-400 hover:text-gray-600")
                            .text("×")
                            .build(),
                    ])
                    .build(),
                p().class("text-sm text-gray-600 mb-3")
                    .text("This popover floats above dropdowns but below modals.")
                    .build(),
                button()
                    .class("px-3 py-1 bg-blue-600 text-white rounded text-sm")
                    .text("Action")
                    .build(),
            ])
            .build()
    }

    fn render_modal_backdrop(&self) -> Element {
        div()
            .class("fixed z-40 inset-0 bg-black opacity-50")
            .build()
    }

    fn render_modal(&self) -> Element {
        div()
            .class("fixed z-50 inset-0 flex items-center justify-center p-4")
            .children(vec![
                div()
                    .class("bg-white rounded-lg shadow-2xl max-w-md w-full p-6")
                    .children(vec![
                        div()
                            .class("flex items-center justify-between mb-4")
                            .children(vec![
                                h2().class("text-xl font-bold").text("Modal Dialog (z-50)").build(),
                                button().class("text-gray-400 hover:text-gray-600 text-xl").text("×").build(),
                            ])
                            .build(),
                        
                        p().class("text-gray-600 mb-6").text("This modal appears above everything else, including the backdrop at z-40.").build(),
                        
                        div()
                            .class("flex gap-3 justify-end")
                            .children(vec![
                                button().class("px-4 py-2 text-gray-600 hover:bg-gray-100 rounded").text("Cancel").build(),
                                button().class("px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700").text("Confirm").build(),
                            ])
                            .build(),
                    ])
                    .build(),
            ])
            .build()
    }

    fn render_notifications(&self) -> Element {
        div()
            .class("fixed z-60 top-4 right-4 space-y-2")
            .children(vec![div()
                .class("bg-green-600 text-white px-4 py-2 rounded-lg shadow-lg")
                .children(vec![
                    div()
                        .class("flex items-center gap-2")
                        .children(vec![
                            span().text("✓").build(),
                            span().text("Notification (z-60)").build(),
                        ])
                        .build(),
                    div()
                        .class("text-xs opacity-90")
                        .text("Always on top")
                        .build(),
                ])
                .build()])
            .build()
    }
}

fn main() -> reactive_tui::Result<()> {
    println!("🎯 Starting Z-Index Layering Demo...");
    println!("This demo shows proper layering for modals, popovers, and overlays");

    let backend = reactive_tui::backend::CrosstermBackend::new()?;
    let app = reactive_tui::app::App::builder()
        .backend(backend)
        .root(LayeringDemoApp::new())
        .build()?;

    app.run()
}
