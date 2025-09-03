//! Demo of the new production-ready widgets
//!
//! Demonstrates the Accordion, Breadcrumb, and File Explorer widgets
//! with their full feature sets and production-quality implementations.

use reactive_tui::prelude::*;
use reactive_tui::widgets::{AccordionSection, AccordionMode, BreadcrumbSegment, ViewMode, SelectionMode};
use reactive_tui::builder::{accordion, breadcrumb, file_explorer};

struct NewWidgetsDemo;

impl reactive_tui::app::RootComponent for NewWidgetsDemo {
    fn render(&self) -> Element {
        Element::layout(LayoutType::Flex)
            .with_class("h-screen bg-gray-100 p-4")
            .with_children(vec![
                // Header
                Element::text("🚀 Production-Ready Widgets Demo")
                    .with_class("text-2xl font-bold mb-4"),

                // Demo grid
                Element::layout(LayoutType::Grid)
                    .with_class("grid grid-cols-1 lg:grid-cols-3 gap-6")
                    .with_children(vec![
                        self.render_accordion_demo(),
                        self.render_breadcrumb_demo(),
                        self.render_file_explorer_demo(),
                    ]),
            ])
    }
}

impl NewWidgetsDemo {
    fn render_accordion_demo(&self) -> Element {
        Element::layout(LayoutType::Flex)
            .with_class("bg-white rounded-lg shadow p-4")
            .with_children(vec![
                Element::text("📋 Accordion Widget")
                    .with_class("text-lg font-semibold mb-4"),
                
                accordion()
                    .mode(AccordionMode::Single)
                    .section(AccordionSection::new("features", "Features")
                        .content(Element::layout(LayoutType::Flex)
                            .with_children(vec![
                                Element::text("✨ Smooth animations"),
                                Element::text("⌨️ Keyboard navigation"),
                                Element::text("♿ Full accessibility"),
                                Element::text("🎨 Customizable styling"),
                            ]))
                        .icon("🚀")
                        .expanded(true))
                    .section(AccordionSection::new("performance", "Performance")
                        .content(Element::text("Built for production with efficient state management, minimal re-renders, and memory leak prevention."))
                        .icon("⚡"))
                    .section(AccordionSection::new("accessibility", "Accessibility")
                        .content(Element::text("ARIA labels, screen reader support, keyboard navigation, and reduced motion support."))
                        .icon("♿"))
                    .build(),
            ])
    }

    fn render_breadcrumb_demo(&self) -> Element {
        Element::layout(LayoutType::Flex)
            .with_class("bg-white rounded-lg shadow p-4")
            .with_children(vec![
                Element::text("🧭 Breadcrumb Widget")
                    .with_class("text-lg font-semibold mb-4"),
                
                Element::text("Navigation Path:")
                    .with_class("text-sm text-gray-600 mb-2"),
                
                breadcrumb()
                    .segment(BreadcrumbSegment::new("home", "Home", "/")
                        .icon("🏠"))
                    .segment(BreadcrumbSegment::new("projects", "Projects", "/projects")
                        .icon("📁"))
                    .segment(BreadcrumbSegment::new("reactive-tui", "reactive-tui", "/projects/reactive-tui")
                        .icon("🦀"))
                    .segment(BreadcrumbSegment::new("widgets", "widgets", "/projects/reactive-tui/src/widgets")
                        .current(true)
                        .icon("🧩"))
                    .separator(" › ")
                    .show_tooltips(true)
                    .build(),

                Element::text("Features:")
                    .with_class("text-sm font-semibold mt-4 mb-2"),
                
                Element::layout(LayoutType::Flex)
                    .with_children(vec![
                        Element::text("🔄 Overflow handling"),
                        Element::text("🖱️ Click navigation"),
                        Element::text("⌨️ Keyboard support"),
                        Element::text("💡 Tooltips"),
                    ]),
            ])
    }

    fn render_file_explorer_demo(&self) -> Element {
        Element::layout(LayoutType::Flex)
            .with_class("bg-white rounded-lg shadow p-4")
            .with_children(vec![
                Element::text("📁 File Explorer Widget")
                    .with_class("text-lg font-semibold mb-4"),
                
                file_explorer()
                    .current_path(std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/")))
                    .view_mode(ViewMode::List)
                    .selection_mode(SelectionMode::Multiple)
                    .show_details(true)
                    .show_breadcrumb(true)
                    .max_visible_items(10)
                    .build(),

                Element::text("Features:")
                    .with_class("text-sm font-semibold mt-4 mb-2"),
                
                Element::layout(LayoutType::Flex)
                    .with_children(vec![
                        Element::text("📂 Directory navigation"),
                        Element::text("🔍 File filtering"),
                        Element::text("📊 Multiple view modes"),
                        Element::text("⚡ Virtual scrolling"),
                        Element::text("🎯 File type detection"),
                        Element::text("⌨️ Keyboard shortcuts"),
                    ]),
            ])
    }
}

fn main() -> reactive_tui::Result<()> {
    println!("🚀 Starting New Widgets Demo...");
    
    let backend = reactive_tui::backend::CrosstermBackend::new()?;
    let app = reactive_tui::app::App::builder()
        .backend(backend)
        .root(NewWidgetsDemo)
        .build()?;

    app.run()
}
