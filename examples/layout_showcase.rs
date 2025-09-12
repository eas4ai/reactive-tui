//! Layout Showcase Example
//!
//! This example demonstrates various layout capabilities of reactive-tui:
//! - Basic layouts with actual visual grids and colors
//! - Flexbox layouts with proper styling
//! - Grid layouts with spanning and visual elements
//! - Complex app layouts with borders and backgrounds
//!
//! Use SPACE/ENTER to navigate between pages, CTRL+Q to quit.

use reactive_tui::prelude::*;
use reactive_tui::backend::CrosstermBackend;
use reactive_tui::builder::{div, header, footer, section, main as main_element, to_element};
use std::sync::{Arc, Mutex};

// Global state for keyboard navigation
static CURRENT_PAGE: std::sync::OnceLock<Arc<Mutex<usize>>> = std::sync::OnceLock::new();

fn get_current_page() -> &'static Arc<Mutex<usize>> {
    CURRENT_PAGE.get_or_init(|| Arc::new(Mutex::new(0)))
}

struct LayoutShowcase;

impl LayoutShowcase {
    fn new() -> Self {
        Self
    }

    fn total_pages() -> usize {
        4
    }



    fn get_page() -> usize {
        *get_current_page().lock().unwrap()
    }

    // Create actual visual layouts with colors using the builder API
    fn create_basic_layouts_page() -> Element {
        div()
            .class("h-screen w-full flex flex-col bg-gray-100")
            .children(vec![
                // Header
                header()
                    .class("h-16 flex items-center justify-center bg-blue-600 text-white text-2xl font-bold")
                    .child(to_element("BASIC LAYOUTS"))
                    .build(),
                
                // Main content with visual grids
                main_element()
                    .class("flex-1 flex flex-row p-4 gap-4")
                    .children(vec![
                        // Vertical Layout Demo
                        section()
                            .class("w-1/2 flex flex-col bg-white border-2 border-gray-300 rounded-lg p-4")
                            .children(vec![
                                div()
                                    .class("h-12 flex items-center justify-center bg-gray-200 mb-4 font-semibold rounded")
                                    .child(to_element("Vertical Layout"))
                                    .build(),
                                // Actual vertical layout with colors
                                div()
                                    .class("flex-1 flex flex-col gap-2")
                                    .children(vec![
                                        div()
                                            .class("flex-1 flex items-center justify-center bg-red-500 border-2 border-red-700 rounded-lg")
                                            .child(
                                                div()
                                                    .class("text-white font-bold text-lg w-full h-full flex items-center justify-center")
                                                    .text("Header")
                                                    .build()
                                            )
                                            .build(),
                                        div()
                                            .class("flex-1 flex items-center justify-center bg-green-500 border-2 border-green-700 rounded-lg")
                                            .child(
                                                div()
                                                    .class("text-white font-bold text-lg w-full h-full flex items-center justify-center")
                                                    .text("Content")
                                                    .build()
                                            )
                                            .build(),
                                        div()
                                            .class("flex-1 flex items-center justify-center bg-blue-500 border-2 border-blue-700 rounded-lg")
                                            .child(
                                                div()
                                                    .class("text-white font-bold text-lg w-full h-full flex items-center justify-center")
                                                    .text("Footer")
                                                    .build()
                                            )
                                            .build(),
                                    ])
                                    .build(),
                            ])
                            .build(),
                        
                        // Horizontal Layout Demo  
                        section()
                            .class("w-1/2 flex flex-col bg-white border-2 border-gray-300 rounded-lg p-4")
                            .children(vec![
                                div()
                                    .class("h-12 flex items-center justify-center bg-gray-200 mb-4 font-semibold rounded")
                                    .child(to_element("Horizontal Layout"))
                                    .build(),
                                // Actual horizontal layout with colors
                                div()
                                    .class("flex-1 flex flex-row gap-2")
                                    .children(vec![
                                        div()
                                            .class("flex-1 flex items-center justify-center bg-yellow-500 text-black border-2 border-yellow-700 rounded-lg font-bold text-lg")
                                            .child(to_element("Left"))
                                            .build(),
                                        div()
                                            .class("flex-1 flex items-center justify-center bg-purple-500 text-white border-2 border-purple-700 rounded-lg font-bold text-lg")
                                            .child(to_element("Center"))
                                            .build(),
                                        div()
                                            .class("flex-1 flex items-center justify-center bg-pink-500 text-white border-2 border-pink-700 rounded-lg font-bold text-lg")
                                            .child(to_element("Right"))
                                            .build(),
                                    ])
                                    .build(),
                            ])
                            .build(),
                    ])
                    .build(),
                
                // Footer with navigation
                footer()
                    .class("h-12 flex items-center justify-center bg-gray-800 text-white font-semibold")
                    .child(to_element(format!(
                        "Page {}/{} | SPACE/ENTER: next | CTRL+Q: quit",
                        Self::get_page() + 1,
                        Self::total_pages()
                    )))
                    .build(),
            ])
            .build()
    }

    fn create_grid_layouts_page() -> Element {
        div()
            .class("h-screen w-full flex flex-col bg-gray-100")
            .children(vec![
                header()
                    .class("h-16 flex items-center justify-center bg-purple-600 text-white text-2xl font-bold")
                    .child(to_element("GRID LAYOUTS"))
                    .build(),
                
                main_element()
                    .class("flex-1 flex flex-row p-4 gap-4")
                    .children(vec![
                        // 3x3 Grid Demo
                        section()
                            .class("w-1/2 bg-white border-2 border-gray-300 rounded-lg p-4")
                            .children(vec![
                                div()
                                    .class("h-12 flex items-center justify-center bg-gray-200 mb-4 font-semibold rounded")
                                    .child(to_element("3x3 Grid"))
                                    .build(),
                                // Actual 3x3 grid with colors
                                div()
                                    .class("grid grid-cols-3 grid-rows-3 gap-2 flex-1")
                                    .children(vec![
                                        div().class("flex items-center justify-center bg-red-500 text-white border-2 border-red-700 rounded-lg font-bold text-xl").child(to_element("1")).build(),
                                        div().class("flex items-center justify-center bg-orange-500 text-white border-2 border-orange-700 rounded-lg font-bold text-xl").child(to_element("2")).build(),
                                        div().class("flex items-center justify-center bg-yellow-500 text-black border-2 border-yellow-700 rounded-lg font-bold text-xl").child(to_element("3")).build(),
                                        div().class("flex items-center justify-center bg-green-500 text-white border-2 border-green-700 rounded-lg font-bold text-xl").child(to_element("4")).build(),
                                        div().class("flex items-center justify-center bg-blue-500 text-white border-2 border-blue-700 rounded-lg font-bold text-xl").child(to_element("5")).build(),
                                        div().class("flex items-center justify-center bg-indigo-500 text-white border-2 border-indigo-700 rounded-lg font-bold text-xl").child(to_element("6")).build(),
                                        div().class("flex items-center justify-center bg-purple-500 text-white border-2 border-purple-700 rounded-lg font-bold text-xl").child(to_element("7")).build(),
                                        div().class("flex items-center justify-center bg-pink-500 text-white border-2 border-pink-700 rounded-lg font-bold text-xl").child(to_element("8")).build(),
                                        div().class("flex items-center justify-center bg-gray-600 text-white border-2 border-gray-800 rounded-lg font-bold text-xl").child(to_element("9")).build(),
                                    ])
                                    .build(),
                            ])
                            .build(),
                        
                        // Spanning Grid Demo
                        section()
                            .class("w-1/2 bg-white border-2 border-gray-300 rounded-lg p-4")
                            .children(vec![
                                div()
                                    .class("h-12 flex items-center justify-center bg-gray-200 mb-4 font-semibold rounded")
                                    .child(to_element("Grid Spanning"))
                                    .build(),
                                // Actual spanning grid with colors
                                div()
                                    .class("grid grid-cols-2 grid-rows-4 gap-2 flex-1")
                                    .children(vec![
                                        div()
                                            .class("col-span-2 flex items-center justify-center bg-blue-600 text-white border-2 border-blue-800 rounded-lg font-bold text-lg")
                                            .child(to_element("Header Span"))
                                            .build(),
                                        div()
                                            .class("flex items-center justify-center bg-red-500 text-white border-2 border-red-700 rounded-lg font-bold text-xl")
                                            .child(to_element("A"))
                                            .build(),
                                        div()
                                            .class("flex items-center justify-center bg-green-500 text-white border-2 border-green-700 rounded-lg font-bold text-xl")
                                            .child(to_element("B"))
                                            .build(),
                                        div()
                                            .class("col-span-2 flex items-center justify-center bg-purple-600 text-white border-2 border-purple-800 rounded-lg font-bold text-lg")
                                            .child(to_element("Footer Span"))
                                            .build(),
                                    ])
                                    .build(),
                            ])
                            .build(),
                    ])
                    .build(),
                
                footer()
                    .class("h-12 flex items-center justify-center bg-gray-800 text-white font-semibold")
                    .child(to_element(format!(
                        "Page {}/{} | SPACE/ENTER: next | CTRL+Q: quit",
                        Self::get_page() + 1,
                        Self::total_pages()
                    )))
                    .build(),
            ])
            .build()
    }

    // Create flexbox layouts demonstration page
    fn create_flexbox_layouts_page() -> Element {
        div()
            .class("h-screen w-full flex flex-col bg-gray-100")
            .children(vec![
                // Header
                header()
                    .class("h-16 flex items-center justify-center bg-purple-600 text-white text-2xl font-bold")
                    .child(
                        div()
                            .class("text-white font-bold text-lg w-full h-full flex items-center justify-center")
                            .text("FLEXBOX LAYOUTS")
                            .build()
                    )
                    .build(),

                // Main content with flexbox demonstrations
                main_element()
                    .class("flex-1 flex flex-col p-4 gap-4")
                    .children(vec![
                        // Flex Direction Demo
                        section()
                            .class("flex-1 flex flex-row bg-white border-2 border-gray-300 rounded-lg p-4 gap-4")
                            .children(vec![
                                // Flex Row
                                div()
                                    .class("flex-1 flex flex-col")
                                    .children(vec![
                                        div()
                                            .class("h-8 flex items-center justify-center bg-gray-200 mb-2 font-semibold rounded")
                                            .child(
                                                div()
                                                    .class("text-gray-800 font-bold w-full h-full flex items-center justify-center")
                                                    .text("Flex Row")
                                                    .build()
                                            )
                                            .build(),
                                        div()
                                            .class("flex-1 flex flex-row gap-2")
                                            .children(vec![
                                                div()
                                                    .class("flex-1 flex items-center justify-center bg-red-500 border-2 border-red-700 rounded")
                                                    .child(
                                                        div()
                                                            .class("text-white font-bold w-full h-full flex items-center justify-center")
                                                            .text("1")
                                                            .build()
                                                    )
                                                    .build(),
                                                div()
                                                    .class("flex-1 flex items-center justify-center bg-green-500 border-2 border-green-700 rounded")
                                                    .child(
                                                        div()
                                                            .class("text-white font-bold w-full h-full flex items-center justify-center")
                                                            .text("2")
                                                            .build()
                                                    )
                                                    .build(),
                                                div()
                                                    .class("flex-1 flex items-center justify-center bg-blue-500 border-2 border-blue-700 rounded")
                                                    .child(
                                                        div()
                                                            .class("text-white font-bold w-full h-full flex items-center justify-center")
                                                            .text("3")
                                                            .build()
                                                    )
                                                    .build(),
                                            ])
                                            .build(),
                                    ])
                                    .build(),

                                // Flex Column
                                div()
                                    .class("flex-1 flex flex-col")
                                    .children(vec![
                                        div()
                                            .class("h-8 flex items-center justify-center bg-gray-200 mb-2 font-semibold rounded")
                                            .child(
                                                div()
                                                    .class("text-gray-800 font-bold w-full h-full flex items-center justify-center")
                                                    .text("Flex Column")
                                                    .build()
                                            )
                                            .build(),
                                        div()
                                            .class("flex-1 flex flex-col gap-2")
                                            .children(vec![
                                                div()
                                                    .class("flex-1 flex items-center justify-center bg-yellow-500 border-2 border-yellow-700 rounded")
                                                    .child(
                                                        div()
                                                            .class("text-black font-bold w-full h-full flex items-center justify-center")
                                                            .text("A")
                                                            .build()
                                                    )
                                                    .build(),
                                                div()
                                                    .class("flex-1 flex items-center justify-center bg-cyan-500 border-2 border-cyan-700 rounded")
                                                    .child(
                                                        div()
                                                            .class("text-white font-bold w-full h-full flex items-center justify-center")
                                                            .text("B")
                                                            .build()
                                                    )
                                                    .build(),
                                                div()
                                                    .class("flex-1 flex items-center justify-center bg-pink-500 border-2 border-pink-700 rounded")
                                                    .child(
                                                        div()
                                                            .class("text-white font-bold w-full h-full flex items-center justify-center")
                                                            .text("C")
                                                            .build()
                                                    )
                                                    .build(),
                                            ])
                                            .build(),
                                    ])
                                    .build(),
                            ])
                            .build(),
                    ])
                    .build(),

                // Navigation footer
                footer()
                    .class("h-12 flex items-center justify-between bg-gray-800 text-white px-6")
                    .children(vec![
                        div()
                            .class("text-white w-32 h-full flex items-center")
                            .text("Page 3 of 4")
                            .build(),
                        div()
                            .class("text-white w-64 h-full flex items-center justify-center")
                            .text("SPACE/ENTER: Next | CTRL+Q: Quit")
                            .build(),
                        div()
                            .class("text-white w-32 h-full flex items-center justify-end")
                            .text("Flexbox Demo")
                            .build(),
                    ])
                    .build(),
            ])
            .build()
    }

    // Create complex layouts demonstration page
    fn create_complex_layouts_page() -> Element {
        div()
            .class("h-screen w-full flex flex-col bg-gray-100")
            .children(vec![
                // Header
                header()
                    .class("h-16 flex items-center justify-center bg-indigo-600 text-white text-2xl font-bold")
                    .child(
                        div()
                            .class("text-white font-bold text-lg w-full h-full flex items-center justify-center")
                            .text("COMPLEX LAYOUTS")
                            .build()
                    )
                    .build(),

                // Main content with complex layout patterns
                main_element()
                    .class("flex-1 flex flex-row p-4 gap-4")
                    .children(vec![
                        // Sidebar + Main Content Pattern
                        section()
                            .class("flex-1 flex flex-row bg-white border-2 border-gray-300 rounded-lg overflow-hidden")
                            .children(vec![
                                // Sidebar
                                div()
                                    .class("w-1/4 flex flex-col bg-gray-800")
                                    .children(vec![
                                        div()
                                            .class("h-12 flex items-center justify-center bg-gray-700 border-b border-gray-600")
                                            .child(
                                                div()
                                                    .class("text-white font-bold w-full h-full flex items-center justify-center")
                                                    .text("Sidebar")
                                                    .build()
                                            )
                                            .build(),
                                        div()
                                            .class("flex-1 flex flex-col p-2 gap-2")
                                            .children(vec![
                                                div()
                                                    .class("h-8 flex items-center justify-center bg-blue-600 text-white rounded")
                                                    .child(
                                                        div()
                                                            .class("text-white font-bold w-full h-full flex items-center justify-center")
                                                            .text("Nav 1")
                                                            .build()
                                                    )
                                                    .build(),
                                                div()
                                                    .class("h-8 flex items-center justify-center bg-blue-500 text-white rounded")
                                                    .child(
                                                        div()
                                                            .class("text-white font-bold w-full h-full flex items-center justify-center")
                                                            .text("Nav 2")
                                                            .build()
                                                    )
                                                    .build(),
                                                div()
                                                    .class("h-8 flex items-center justify-center bg-blue-400 text-white rounded")
                                                    .child(
                                                        div()
                                                            .class("text-white font-bold w-full h-full flex items-center justify-center")
                                                            .text("Nav 3")
                                                            .build()
                                                    )
                                                    .build(),
                                            ])
                                            .build(),
                                    ])
                                    .build(),

                                // Main Content Area
                                div()
                                    .class("flex-1 flex flex-col")
                                    .children(vec![
                                        // Content Header
                                        div()
                                            .class("h-12 flex items-center justify-center bg-gray-100 border-b border-gray-300")
                                            .child(
                                                div()
                                                    .class("text-gray-800 font-bold w-full h-full flex items-center justify-center")
                                                    .text("Main Content")
                                                    .build()
                                            )
                                            .build(),

                                        // Content Body with Grid
                                        div()
                                            .class("flex-1 p-4")
                                            .child(
                                                div()
                                                    .class("h-full grid grid-cols-2 gap-4")
                                                    .children(vec![
                                                        div()
                                                            .class("flex items-center justify-center bg-green-500 text-white rounded-lg border-2 border-green-700")
                                                            .child(
                                                                div()
                                                                    .class("text-white font-bold w-full h-full flex items-center justify-center")
                                                                    .text("Card 1")
                                                                    .build()
                                                            )
                                                            .build(),
                                                        div()
                                                            .class("flex items-center justify-center bg-orange-500 text-white rounded-lg border-2 border-orange-700")
                                                            .child(
                                                                div()
                                                                    .class("text-white font-bold w-full h-full flex items-center justify-center")
                                                                    .text("Card 2")
                                                                    .build()
                                                            )
                                                            .build(),
                                                        div()
                                                            .class("flex items-center justify-center bg-purple-500 text-white rounded-lg border-2 border-purple-700")
                                                            .child(
                                                                div()
                                                                    .class("text-white font-bold w-full h-full flex items-center justify-center")
                                                                    .text("Card 3")
                                                                    .build()
                                                            )
                                                            .build(),
                                                        div()
                                                            .class("flex items-center justify-center bg-teal-500 text-white rounded-lg border-2 border-teal-700")
                                                            .child(
                                                                div()
                                                                    .class("text-white font-bold w-full h-full flex items-center justify-center")
                                                                    .text("Card 4")
                                                                    .build()
                                                            )
                                                            .build(),
                                                    ])
                                                    .build()
                                            )
                                            .build(),
                                    ])
                                    .build(),
                            ])
                            .build(),
                    ])
                    .build(),

                // Navigation footer
                footer()
                    .class("h-12 flex items-center justify-between bg-gray-800 text-white px-6")
                    .children(vec![
                        div()
                            .class("text-white w-32 h-full flex items-center")
                            .text("Page 4 of 4")
                            .build(),
                        div()
                            .class("text-white w-64 h-full flex items-center justify-center")
                            .text("SPACE/ENTER: Next | CTRL+Q: Quit")
                            .build(),
                        div()
                            .class("text-white w-32 h-full flex items-center justify-end")
                            .text("Complex Demo")
                            .build(),
                    ])
                    .build(),
            ])
            .build()
    }
}

impl reactive_tui::app::RootComponent for LayoutShowcase {
    fn render(&self) -> Element {
        println!("DEBUG: LayoutShowcase render() called");
        let current_page = Self::get_page();
        
        match current_page {
            0 => Self::create_basic_layouts_page(),
            1 => Self::create_grid_layouts_page(),
            2 => Self::create_flexbox_layouts_page(),
            3 => Self::create_complex_layouts_page(),
            _ => Self::create_basic_layouts_page(),
        }
    }
}

fn main() -> reactive_tui::Result<()> {
    println!("Starting Layout Showcase...");
    
    let backend = CrosstermBackend::new()?;
    let showcase = LayoutShowcase::new();
    
    let app = reactive_tui::app::App::builder()
        .backend(backend)
        .root(showcase)
        .build()?;

    app.run()
}
