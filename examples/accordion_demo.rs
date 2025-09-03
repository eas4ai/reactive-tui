//! Accordion Widget Demo
//!
//! Demonstrates the sophisticated accordion widget with various configurations:
//! - Different expansion modes (single, multiple, always-one)
//! - Custom styling and animations
//! - Accessibility features
//! - Keyboard navigation
//! - Complex content types

use reactive_tui::prelude::*;
use reactive_tui::widgets::{AccordionSection, AccordionMode};
use reactive_tui::builder::{accordion, settings_accordion, faq_accordion, navigation_accordion};

struct AccordionDemoApp;

impl reactive_tui::app::RootComponent for AccordionDemoApp {
    fn render(&self) -> Element {
        Element::layout(LayoutType::Flex)
            .with_class("h-screen bg-gray-100 p-4")
            .with_children(vec![
                // Header
                Element::layout(LayoutType::Flex)
                    .with_class("mb-6")
                    .with_child(
                        Element::text("🎛️ Accordion Widget Demo")
                            .with_class("text-3xl font-bold text-gray-800")
                    ),

                // Demo sections in a grid
                Element::layout(LayoutType::Grid)
                    .with_class("grid grid-cols-2 gap-6")
                    .with_children(vec![
                        // Settings-style accordion
                        self.render_settings_demo(),
                        
                        // FAQ-style accordion
                        self.render_faq_demo(),
                        
                        // Navigation accordion
                        self.render_navigation_demo(),
                        
                        // Advanced features demo
                        self.render_advanced_demo(),
                    ]),

                // Footer with instructions
                Element::layout(LayoutType::Flex)
                    .with_class("mt-6 p-4 bg-blue-50 rounded-lg")
                    .with_children(vec![
                        Element::text("💡 Instructions:")
                            .with_class("font-semibold text-blue-800 mb-2"),
                        Element::text("• Use ↑/↓ arrow keys to navigate sections")
                            .with_class("text-blue-700"),
                        Element::text("• Press Enter or Space to expand/collapse")
                            .with_class("text-blue-700"),
                        Element::text("• Use Home/End to jump to first/last section")
                            .with_class("text-blue-700"),
                    ]),
            ])
    }
}

impl AccordionDemoApp {
    fn render_settings_demo(&self) -> Element {
        Element::layout(LayoutType::Flex)
            .with_class("bg-white rounded-lg shadow p-4")
            .with_children(vec![
                Element::text("⚙️ Settings Accordion")
                    .with_class("text-lg font-semibold mb-4 text-gray-800"),
                
                settings_accordion()
                    .section(AccordionSection::new("general", "General Settings")
                        .content(self.render_general_settings())
                        .icon("⚙️")
                        .expanded(true))
                    .section(AccordionSection::new("privacy", "Privacy & Security")
                        .content(self.render_privacy_settings())
                        .icon("🔒"))
                    .section(AccordionSection::new("notifications", "Notifications")
                        .content(self.render_notification_settings())
                        .icon("🔔"))
                    .section(AccordionSection::new("advanced", "Advanced")
                        .content(self.render_advanced_settings())
                        .icon("🔧"))
                    .build(),
            ])
    }

    fn render_faq_demo(&self) -> Element {
        Element::layout(LayoutType::Flex)
            .with_class("bg-white rounded-lg shadow p-4")
            .with_children(vec![
                Element::text("❓ FAQ Accordion")
                    .with_class("text-lg font-semibold mb-4 text-gray-800"),
                
                faq_accordion()
                    .section(AccordionSection::new("getting-started", "How do I get started?")
                        .content(Element::layout(LayoutType::Flex)
                            .with_children(vec![
                                Element::text("Getting started is easy! Follow these steps:")
                                    .with_class("mb-2"),
                                Element::text("1. Install the application")
                                    .with_class("ml-4 text-gray-600"),
                                Element::text("2. Create your first project")
                                    .with_class("ml-4 text-gray-600"),
                                Element::text("3. Explore the features")
                                    .with_class("ml-4 text-gray-600"),
                            ])))
                    .section(AccordionSection::new("requirements", "What are the system requirements?")
                        .content(Element::layout(LayoutType::Flex)
                            .with_children(vec![
                                Element::text("System Requirements:")
                                    .with_class("font-semibold mb-2"),
                                Element::text("• Rust 1.70 or later")
                                    .with_class("text-gray-600"),
                                Element::text("• Modern terminal with 24-bit color")
                                    .with_class("text-gray-600"),
                                Element::text("• 4GB RAM minimum")
                                    .with_class("text-gray-600"),
                            ])))
                    .section(AccordionSection::new("troubleshooting", "Common troubleshooting steps")
                        .content(Element::text("If you encounter issues, try these steps:\n\n1. Check your terminal compatibility\n2. Update to the latest version\n3. Clear your cache\n4. Restart the application")))
                    .build(),
            ])
    }

    fn render_navigation_demo(&self) -> Element {
        Element::layout(LayoutType::Flex)
            .with_class("bg-white rounded-lg shadow p-4")
            .with_children(vec![
                Element::text("🧭 Navigation Accordion")
                    .with_class("text-lg font-semibold mb-4 text-gray-800"),
                
                navigation_accordion()
                    .section(AccordionSection::new("dashboard", "Dashboard")
                        .content(Element::layout(LayoutType::Flex)
                            .with_children(vec![
                                Element::text("📊 Overview")
                                    .with_class("p-2 hover:bg-gray-100 cursor-pointer"),
                                Element::text("📈 Analytics")
                                    .with_class("p-2 hover:bg-gray-100 cursor-pointer"),
                                Element::text("📋 Reports")
                                    .with_class("p-2 hover:bg-gray-100 cursor-pointer"),
                            ]))
                        .expanded(true))
                    .section(AccordionSection::new("projects", "Projects")
                        .content(Element::layout(LayoutType::Flex)
                            .with_children(vec![
                                Element::text("📁 All Projects")
                                    .with_class("p-2 hover:bg-gray-100 cursor-pointer"),
                                Element::text("⭐ Favorites")
                                    .with_class("p-2 hover:bg-gray-100 cursor-pointer"),
                                Element::text("🗂️ Archives")
                                    .with_class("p-2 hover:bg-gray-100 cursor-pointer"),
                            ])))
                    .section(AccordionSection::new("tools", "Tools")
                        .content(Element::layout(LayoutType::Flex)
                            .with_children(vec![
                                Element::text("🔧 Settings")
                                    .with_class("p-2 hover:bg-gray-100 cursor-pointer"),
                                Element::text("📊 Diagnostics")
                                    .with_class("p-2 hover:bg-gray-100 cursor-pointer"),
                                Element::text("🔄 Sync")
                                    .with_class("p-2 hover:bg-gray-100 cursor-pointer"),
                            ])))
                    .build(),
            ])
    }

    fn render_advanced_demo(&self) -> Element {
        Element::layout(LayoutType::Flex)
            .with_class("bg-white rounded-lg shadow p-4")
            .with_children(vec![
                Element::text("🚀 Advanced Features")
                    .with_class("text-lg font-semibold mb-4 text-gray-800"),
                
                accordion()
                    .mode(AccordionMode::Multiple)
                    .icons("🔽", "🔼")
                    .class("advanced-accordion")
                    .section(AccordionSection::new("animations", "Smooth Animations")
                        .content(Element::text("This accordion features smooth spring-based animations with configurable easing and duration."))
                        .icon("✨"))
                    .section(AccordionSection::new("accessibility", "Full Accessibility")
                        .content(Element::layout(LayoutType::Flex)
                            .with_children(vec![
                                Element::text("Accessibility Features:")
                                    .with_class("font-semibold mb-2"),
                                Element::text("• ARIA labels and roles")
                                    .with_class("text-gray-600"),
                                Element::text("• Keyboard navigation")
                                    .with_class("text-gray-600"),
                                Element::text("• Screen reader support")
                                    .with_class("text-gray-600"),
                                Element::text("• Reduced motion support")
                                    .with_class("text-gray-600"),
                            ]))
                        .icon("♿")
                        .aria_label("Accessibility features section"))
                    .section(AccordionSection::new("performance", "Performance Optimized")
                        .content(Element::text("Built with performance in mind:\n• Efficient state management\n• Minimal re-renders\n• Memory leak prevention\n• Smooth 60fps animations"))
                        .icon("⚡"))
                    .section(AccordionSection::new("customization", "Highly Customizable")
                        .content(Element::text("Customize every aspect:\n• Custom icons and styling\n• Flexible content types\n• Multiple expansion modes\n• Event callbacks"))
                        .icon("🎨"))
                    .build(),
            ])
    }

    fn render_general_settings(&self) -> Element {
        Element::layout(LayoutType::Flex)
            .with_children(vec![
                Element::text("🌐 Language: English")
                    .with_class("p-2 border-b"),
                Element::text("🎨 Theme: Dark Mode")
                    .with_class("p-2 border-b"),
                Element::text("💾 Auto-save: Enabled")
                    .with_class("p-2"),
            ])
    }

    fn render_privacy_settings(&self) -> Element {
        Element::layout(LayoutType::Flex)
            .with_children(vec![
                Element::text("🔐 Two-factor authentication: Enabled")
                    .with_class("p-2 border-b"),
                Element::text("📊 Analytics: Disabled")
                    .with_class("p-2 border-b"),
                Element::text("🍪 Cookies: Essential only")
                    .with_class("p-2"),
            ])
    }

    fn render_notification_settings(&self) -> Element {
        Element::layout(LayoutType::Flex)
            .with_children(vec![
                Element::text("📧 Email notifications: Enabled")
                    .with_class("p-2 border-b"),
                Element::text("🔔 Push notifications: Disabled")
                    .with_class("p-2 border-b"),
                Element::text("📱 SMS alerts: Enabled")
                    .with_class("p-2"),
            ])
    }

    fn render_advanced_settings(&self) -> Element {
        Element::layout(LayoutType::Flex)
            .with_children(vec![
                Element::text("🔧 Debug mode: Disabled")
                    .with_class("p-2 border-b"),
                Element::text("⚡ Performance mode: Enabled")
                    .with_class("p-2 border-b"),
                Element::text("🧪 Experimental features: Disabled")
                    .with_class("p-2"),
            ])
    }
}

fn main() -> reactive_tui::Result<()> {
    println!("🎛️ Starting Accordion Widget Demo...");
    
    let backend = reactive_tui::backend::CrosstermBackend::new()?;
    let app = reactive_tui::app::App::builder()
        .backend(backend)
        .root(AccordionDemoApp)
        .build()?;

    app.run()
}
