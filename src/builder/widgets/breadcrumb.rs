//! Breadcrumb widget builder
//!
//! Provides fluent API for creating breadcrumb navigation widgets with
//! overflow handling, icons, and accessibility features.

use crate::component::Element;
use crate::widgets::layout::{BreadcrumbBuilder, BreadcrumbSegment, OverflowStrategy};

/// Create a breadcrumb widget builder
///
/// Returns a `BreadcrumbBuilder` for creating breadcrumb navigation widgets
/// with segments, separators, and overflow handling.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::breadcrumb;
/// use reactive_tui::widgets::BreadcrumbSegment;
///
/// let breadcrumb = breadcrumb()
///     .segment(BreadcrumbSegment::new("home", "Home", "/"))
///     .segment(BreadcrumbSegment::new("docs", "Documentation", "/docs"))
///     .segment(BreadcrumbSegment::new("api", "API Reference", "/docs/api")
///         .current(true))
///     .separator(" > ")
///     .max_width(80)
///     .build();
/// ```
pub fn breadcrumb() -> BreadcrumbBuilder {
    BreadcrumbBuilder::new()
}

/// Create a simple breadcrumb from path segments
///
/// A convenience function for creating breadcrumbs from simple path strings.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::simple_breadcrumb;
///
/// let breadcrumb = simple_breadcrumb(vec![
///     ("home", "Home", "/"),
///     ("products", "Products", "/products"),
///     ("laptops", "Laptops", "/products/laptops"),
/// ], Some("laptops")); // Mark "laptops" as current
/// ```
pub fn simple_breadcrumb(segments: Vec<(&str, &str, &str)>, current_id: Option<&str>) -> Element {
    let mut builder = BreadcrumbBuilder::new();

    for (id, label, path) in segments {
        let is_current = current_id == Some(id);
        let segment = BreadcrumbSegment::new(id, label, path).current(is_current);
        builder = builder.segment(segment);
    }

    builder.build()
}

/// Create a file system breadcrumb
///
/// Creates a breadcrumb optimized for file system navigation with
/// folder icons and appropriate separators.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::filesystem_breadcrumb;
/// use reactive_tui::widgets::BreadcrumbSegment;
///
/// let breadcrumb = filesystem_breadcrumb()
///     .segment(BreadcrumbSegment::new("root", "Root", "/")
///         .icon("🏠"))
///     .segment(BreadcrumbSegment::new("home", "home", "/home")
///         .icon("📁"))
///     .segment(BreadcrumbSegment::new("user", "user", "/home/user")
///         .icon("👤")
///         .current(true))
///     .build();
/// ```
pub fn filesystem_breadcrumb() -> BreadcrumbBuilder {
    BreadcrumbBuilder::new()
        .separator("/")
        .show_icons(true)
        .home_icon("🏠")
        .overflow_strategy(OverflowStrategy::MiddleEllipsis)
        .class("filesystem-breadcrumb")
}

/// Create a website breadcrumb
///
/// Creates a breadcrumb optimized for website navigation with
/// appropriate styling and separators.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::website_breadcrumb;
/// use reactive_tui::widgets::BreadcrumbSegment;
///
/// let breadcrumb = website_breadcrumb()
///     .segment(BreadcrumbSegment::new("home", "Home", "/"))
///     .segment(BreadcrumbSegment::new("blog", "Blog", "/blog"))
///     .segment(BreadcrumbSegment::new("post", "My Post", "/blog/my-post")
///         .current(true))
///     .build();
/// ```
pub fn website_breadcrumb() -> BreadcrumbBuilder {
    BreadcrumbBuilder::new()
        .separator(" › ")
        .show_icons(false)
        .show_tooltips(true)
        .overflow_strategy(OverflowStrategy::TruncateStart)
        .class("website-breadcrumb")
}

/// Create a compact breadcrumb
///
/// Creates a breadcrumb with minimal spacing and compact layout,
/// ideal for toolbars or limited space scenarios.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::compact_breadcrumb;
/// use reactive_tui::widgets::BreadcrumbSegment;
///
/// let breadcrumb = compact_breadcrumb()
///     .segment(BreadcrumbSegment::new("root", "~", "/home/user"))
///     .segment(BreadcrumbSegment::new("projects", "projects", "/home/user/projects"))
///     .segment(BreadcrumbSegment::new("app", "my-app", "/home/user/projects/my-app")
///         .current(true))
///     .build();
/// ```
pub fn compact_breadcrumb() -> BreadcrumbBuilder {
    BreadcrumbBuilder::new()
        .separator("/")
        .compact(true)
        .show_icons(false)
        .max_width(60)
        .overflow_strategy(OverflowStrategy::TruncateStart)
        .class("compact-breadcrumb")
}

/// Create an application breadcrumb
///
/// Creates a breadcrumb optimized for application navigation with
/// icons and tooltips for better user experience.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::application_breadcrumb;
/// use reactive_tui::widgets::BreadcrumbSegment;
///
/// let breadcrumb = application_breadcrumb()
///     .segment(BreadcrumbSegment::new("dashboard", "Dashboard", "/dashboard")
///         .icon("📊")
///         .tooltip("Go to main dashboard"))
///     .segment(BreadcrumbSegment::new("settings", "Settings", "/settings")
///         .icon("⚙️")
///         .tooltip("Application settings"))
///     .segment(BreadcrumbSegment::new("users", "User Management", "/settings/users")
///         .icon("👥")
///         .current(true))
///     .build();
/// ```
pub fn application_breadcrumb() -> BreadcrumbBuilder {
    BreadcrumbBuilder::new()
        .separator(" ▸ ")
        .show_icons(true)
        .show_tooltips(true)
        .keyboard_navigation(true)
        .overflow_strategy(OverflowStrategy::MiddleEllipsis)
        .class("application-breadcrumb")
}

/// Create a breadcrumb from a file path
///
/// Convenience function to create a breadcrumb from a file system path.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::path_breadcrumb;
///
/// let breadcrumb = path_breadcrumb("/home/user/projects/my-app/src/main.rs");
/// ```
pub fn path_breadcrumb(path: &str) -> Element {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let mut builder = filesystem_breadcrumb();

    // Add root segment
    builder = builder.segment(BreadcrumbSegment::new("root", "Root", "/").icon("🏠"));

    // Add path segments
    let mut current_path = String::new();
    for (index, segment) in segments.iter().enumerate() {
        current_path.push('/');
        current_path.push_str(segment);

        let is_current = index == segments.len() - 1;
        let icon = if segment.contains('.') {
            "📄"
        } else {
            "📁"
        };

        builder = builder.segment(
            BreadcrumbSegment::new(format!("segment_{}", index), *segment, &current_path)
                .icon(icon)
                .current(is_current),
        );
    }

    builder.build()
}

/// Create a breadcrumb from URL segments
///
/// Convenience function to create a breadcrumb from a URL path.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::url_breadcrumb;
///
/// let breadcrumb = url_breadcrumb("https://example.com/docs/api/reference");
/// ```
pub fn url_breadcrumb(url: &str) -> Element {
    // Parse URL and extract path segments
    let path = if let Some(path_start) = url.find("://") {
        if let Some(path_start) = url[path_start + 3..].find('/') {
            &url[path_start + 3 + path_start..]
        } else {
            "/"
        }
    } else {
        url
    };

    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let mut builder = website_breadcrumb();

    // Add home segment
    builder = builder.segment(BreadcrumbSegment::new("home", "Home", "/"));

    // Add URL segments
    let mut current_path = String::new();
    for (index, segment) in segments.iter().enumerate() {
        current_path.push('/');
        current_path.push_str(segment);

        let is_current = index == segments.len() - 1;
        let label = segment.replace(['-', '_'], " ");
        let title_case_label = title_case(&label);

        builder = builder.segment(
            BreadcrumbSegment::new(
                format!("segment_{}", index),
                &title_case_label,
                &current_path,
            )
            .current(is_current),
        );
    }

    builder.build()
}

/// Convert string to title case
fn title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::BreadcrumbSegment;

    #[test]
    fn test_breadcrumb_builder() {
        let breadcrumb = breadcrumb()
            .segment(BreadcrumbSegment::new("home", "Home", "/"))
            .segment(BreadcrumbSegment::new("docs", "Docs", "/docs"))
            .build();

        assert!(breadcrumb.is_component());
        assert!(breadcrumb.metadata.factory.is_some());
        let props = breadcrumb
            .props
            .downcast_ref::<crate::widgets::layout::BreadcrumbProps>()
            .unwrap();
        assert_eq!(props.segments.len(), 2);
        assert_eq!(props.segments[0].id, "home");
        assert_eq!(props.segments[1].id, "docs");
    }

    #[test]
    fn test_simple_breadcrumb() {
        let breadcrumb = simple_breadcrumb(
            vec![("home", "Home", "/"), ("docs", "Documentation", "/docs")],
            Some("docs"),
        );

        assert!(breadcrumb.is_component());
    }

    #[test]
    fn test_filesystem_breadcrumb() {
        let breadcrumb = filesystem_breadcrumb()
            .segment(BreadcrumbSegment::new("root", "Root", "/"))
            .build();

        assert!(breadcrumb.is_component());
    }

    #[test]
    fn test_website_breadcrumb() {
        let breadcrumb = website_breadcrumb()
            .segment(BreadcrumbSegment::new("home", "Home", "/"))
            .build();

        assert!(breadcrumb.is_component());
    }

    #[test]
    fn test_path_breadcrumb() {
        let breadcrumb = path_breadcrumb("/home/user/projects/app");
        assert!(breadcrumb.is_component());
    }

    #[test]
    fn test_url_breadcrumb() {
        let breadcrumb = url_breadcrumb("https://example.com/docs/api");
        assert!(breadcrumb.is_component());
    }

    #[test]
    fn test_title_case() {
        assert_eq!(title_case("hello world"), "Hello World");
        assert_eq!(title_case("api reference"), "Api Reference");
        assert_eq!(title_case(""), "");
    }
}
