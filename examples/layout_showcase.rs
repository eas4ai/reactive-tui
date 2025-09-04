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

    fn next_page() {
        let page_ref = get_current_page();
        let mut page = page_ref.lock().unwrap();
        *page = (*page + 1) % Self::total_pages();
    }

    fn get_page() -> usize {
        *get_current_page().lock().unwrap()
    }

    // Create actual visual layouts with colors using the builder API
    fn create_basic_layouts_page() -> Element {
        div()
            .class("h-screen flex flex-col bg-gray-100")
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
                                            .class("flex-1 flex items-center justify-center bg-red-500 text-white border-2 border-red-700 rounded-lg font-bold text-lg")
                                            .child(to_element("Header"))
                                            .build(),
                                        div()
                                            .class("flex-1 flex items-center justify-center bg-green-500 text-white border-2 border-green-700 rounded-lg font-bold text-lg")
                                            .child(to_element("Content"))
                                            .build(),
                                        div()
                                            .class("flex-1 flex items-center justify-center bg-blue-500 text-white border-2 border-blue-700 rounded-lg font-bold text-lg")
                                            .child(to_element("Footer"))
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
            .class("h-screen flex flex-col bg-gray-100")
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
}

impl reactive_tui::app::RootComponent for LayoutShowcase {
    fn render(&self) -> Element {
        let current_page = Self::get_page();
        
        match current_page {
            0 => Self::create_basic_layouts_page(),
            1 => Self::create_grid_layouts_page(),
            2 => Self::create_basic_layouts_page(), // TODO: Add flexbox page
            3 => Self::create_grid_layouts_page(),  // TODO: Add complex page
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
