//! Example showing React + Tailwind-like API for terminal UIs
//!
//! This demonstrates how to build responsive terminal applications using
//! familiar web development patterns with div(), span(), CSS classes, etc.

use reactive_tui::backend::CrosstermBackend;
use reactive_tui::prelude::*;

struct WebLikeApp;

impl reactive_tui::app::RootComponent for WebLikeApp {
    fn render(&self) -> Element {
        app_layout()
    }
}

fn main() -> reactive_tui::Result<()> {
    // Create a full-screen responsive application
    let backend = CrosstermBackend::new()?;
    let app = App::builder().backend(backend).root(WebLikeApp).build()?;

    app.run()
}

/// Main application layout - fills the entire terminal
fn app_layout() -> Element {
    screen()
        .class("bg-gray-900 text-white")
        .children(vec![app_header(), app_body(), app_footer()])
        .build()
}

/// Application header with navigation
fn app_header() -> Element {
    header()
        .class("flex justify-between items-center p-4 bg-blue-600 border-b border-blue-700")
        .children(vec![
            // Logo/Title
            div()
                .class("flex items-center space-x-2")
                .children(vec![
                    span().class("text-2xl font-bold").text("🚀").build(),
                    h1().text("Terminal App").build(),
                ])
                .build(),
            // Navigation
            nav()
                .class("flex space-x-4")
                .children(vec![
                    nav_item("Home", true),
                    nav_item("Dashboard", false),
                    nav_item("Settings", false),
                ])
                .build(),
            // User info
            div()
                .class("flex items-center space-x-2")
                .children(vec![
                    span().class("text-sm").text("Welcome, User").build(),
                    button()
                        .class("px-3 py-1 bg-blue-500 hover:bg-blue-400 rounded")
                        .text("Logout")
                        .build(),
                ])
                .build(),
        ])
        .build()
}

/// Navigation item
fn nav_item(label: &str, active: bool) -> Element {
    let base_classes = "px-3 py-2 rounded transition-colors";
    let classes = if active {
        format!("{} bg-blue-500 text-white", base_classes)
    } else {
        format!(
            "{} text-blue-100 hover:bg-blue-500 hover:text-white",
            base_classes
        )
    };

    button().class(&classes).text(label).build()
}

/// Main application body with sidebar and content
fn app_body() -> Element {
    div()
        .class("flex flex-1 overflow-hidden")
        .children(vec![app_sidebar(), app_main_content()])
        .build()
}

/// Sidebar with navigation and info
fn app_sidebar() -> Element {
    sidebar()
        .class("bg-gray-800 border-r border-gray-700 p-4")
        .children(vec![
            // Sidebar navigation
            nav()
                .class("space-y-2")
                .children(vec![
                    sidebar_section("Navigation"),
                    sidebar_item("📊 Dashboard", true),
                    sidebar_item("📁 Files", false),
                    sidebar_item("⚙️ Settings", false),
                    sidebar_item("👥 Users", false),
                ])
                .build(),
            // Spacer
            div().class("flex-1").build(),
            // Sidebar footer
            div()
                .class("mt-8 p-3 bg-gray-700 rounded")
                .children(vec![
                    span()
                        .class("text-sm font-medium")
                        .text("System Status")
                        .build(),
                    div()
                        .class("mt-2 space-y-1")
                        .children(vec![
                            status_item("CPU", "45%", "text-green-400"),
                            status_item("Memory", "67%", "text-yellow-400"),
                            status_item("Disk", "23%", "text-green-400"),
                        ])
                        .build(),
                ])
                .build(),
        ])
        .build()
}

fn sidebar_section(title: &str) -> Element {
    span()
        .class("text-xs font-semibold text-gray-400 uppercase tracking-wider")
        .text(title)
        .build()
}

fn sidebar_item(label: &str, active: bool) -> Element {
    let classes = if active {
        "flex items-center px-3 py-2 bg-gray-700 text-white rounded"
    } else {
        "flex items-center px-3 py-2 text-gray-300 hover:bg-gray-700 hover:text-white rounded"
    };

    div()
        .class(classes)
        .child(span().text(label).build())
        .build()
}

fn status_item(label: &str, value: &str, color_class: &str) -> Element {
    div()
        .class("flex justify-between text-sm")
        .children(vec![
            span().class("text-gray-300").text(label).build(),
            span().class(color_class).text(value).build(),
        ])
        .build()
}

/// Main content area
fn app_main_content() -> Element {
    content()
        .class("overflow-auto")
        .children(vec![
            // Page header
            div()
                .class("p-6 border-b border-gray-700")
                .children(vec![
                    h2().text("Dashboard").build(),
                    p().class("text-gray-400 mt-1")
                        .text("Welcome to your terminal dashboard")
                        .build(),
                ])
                .build(),
            // Content grid
            div().class("p-6").child(dashboard_content()).build(),
        ])
        .build()
}

/// Dashboard content with responsive grid
fn dashboard_content() -> Element {
    div()
        .class("space-y-6")
        .children(vec![
            // Stats cards
            responsive_grid(3)
                .children(vec![
                    stat_card("Total Users", "1,234", "👥", "text-blue-400"),
                    stat_card("Active Sessions", "89", "🔥", "text-green-400"),
                    stat_card("Revenue", "$12,345", "💰", "text-yellow-400"),
                ])
                .build(),
            // Charts section
            div()
                .class("grid grid-cols-1 lg:grid-cols-2 gap-6")
                .children(vec![
                    chart_card("Performance Metrics"),
                    chart_card("User Activity"),
                ])
                .build(),
            // Recent activity
            activity_card(),
        ])
        .build()
}

fn stat_card(title: &str, value: &str, icon: &str, color: &str) -> Element {
    card()
        .class("p-6 bg-gray-800 border-gray-700")
        .children(vec![div()
            .class("flex items-center justify-between")
            .children(vec![
                div()
                    .children(vec![
                        span().class("text-sm text-gray-400").text(title).build(),
                        div().class("text-2xl font-bold mt-1").text(value).build(),
                    ])
                    .build(),
                span()
                    .class(&format!("text-3xl {}", color))
                    .text(icon)
                    .build(),
            ])
            .build()])
        .build()
}

fn chart_card(title: &str) -> Element {
    card()
        .class("p-6 bg-gray-800 border-gray-700")
        .children(vec![
            h3().class("text-lg font-semibold mb-4").text(title).build(),
            div()
                .class("h-32 bg-gray-700 rounded flex items-center justify-center")
                .child(
                    span()
                        .class("text-gray-400")
                        .text("[Chart Placeholder]")
                        .build(),
                )
                .build(),
        ])
        .build()
}

fn activity_card() -> Element {
    card()
        .class("p-6 bg-gray-800 border-gray-700")
        .children(vec![
            h3().class("text-lg font-semibold mb-4")
                .text("Recent Activity")
                .build(),
            div()
                .class("space-y-3")
                .children(vec![
                    activity_item("User john_doe logged in", "2 minutes ago"),
                    activity_item("New file uploaded: report.pdf", "5 minutes ago"),
                    activity_item("System backup completed", "1 hour ago"),
                    activity_item("User jane_smith updated profile", "2 hours ago"),
                ])
                .build(),
        ])
        .build()
}

fn activity_item(message: &str, time: &str) -> Element {
    div()
        .class("flex justify-between items-center py-2 border-b border-gray-700 last:border-b-0")
        .children(vec![
            span().class("text-sm").text(message).build(),
            span().class("text-xs text-gray-400").text(time).build(),
        ])
        .build()
}

/// Application footer
fn app_footer() -> Element {
    footer()
        .class("p-3 bg-gray-800 border-t border-gray-700 text-center")
        .children(vec![div()
            .class("flex justify-between items-center text-sm text-gray-400")
            .children(vec![
                span()
                    .text("© 2024 Terminal App. Built with Reactive-TUI.")
                    .build(),
                div()
                    .class("flex space-x-4")
                    .children(vec![
                        span().text("Press 'q' to quit").build(),
                        span().text("Press 'h' for help").build(),
                    ])
                    .build(),
            ])
            .build()])
        .build()
}
