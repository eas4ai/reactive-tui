//! File Explorer widget builder
//!
//! Provides fluent API for creating file explorer widgets with
//! file system navigation, filtering, and advanced features.

use crate::component::Element;
use crate::widgets::display::{FileExplorerBuilder, SelectionMode, SortCriteria, ViewMode};
use std::path::PathBuf;

/// Create a file explorer widget builder
///
/// Returns a `FileExplorerBuilder` for creating file system navigation widgets
/// with advanced features like filtering, sorting, and multiple view modes.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::file_explorer;
/// use reactive_tui::widgets::{ViewMode, SelectionMode, SortCriteria};
///
/// let explorer = file_explorer()
///     .current_path("/home/user/projects")
///     .show_hidden(false)
///     .view_mode(ViewMode::List)
///     .selection_mode(SelectionMode::Multiple)
///     .sort_by(SortCriteria::Name)
///     .show_details(true)
///     .build();
/// ```
pub fn file_explorer() -> FileExplorerBuilder {
    FileExplorerBuilder::new()
}

/// Create a simple file browser
///
/// A convenience function for creating a basic file browser with
/// common settings for general file navigation.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::simple_file_browser;
///
/// let browser = simple_file_browser("/home/user");
/// ```
pub fn simple_file_browser(path: impl Into<PathBuf>) -> Element {
    FileExplorerBuilder::new()
        .current_path(path)
        .view_mode(ViewMode::List)
        .selection_mode(SelectionMode::Single)
        .show_details(true)
        .show_breadcrumb(true)
        .keyboard_navigation(true)
        .build()
}

/// Create a code project explorer
///
/// Creates a file explorer optimized for code projects with
/// appropriate filters and settings for development workflows.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::code_project_explorer;
///
/// let explorer = code_project_explorer("/home/user/my-project")
///     .file_filters(vec!["rs".to_string(), "toml".to_string(), "md".to_string()])
///     .build();
/// ```
pub fn code_project_explorer(project_path: impl Into<PathBuf>) -> FileExplorerBuilder {
    FileExplorerBuilder::new()
        .current_path(project_path)
        .view_mode(ViewMode::Tree)
        .selection_mode(SelectionMode::Multiple)
        .show_hidden(false)
        .show_details(true)
        .show_breadcrumb(true)
        .sort_by(SortCriteria::Type)
        .keyboard_navigation(true)
        .class("code-project-explorer")
}

/// Create a media gallery explorer
///
/// Creates a file explorer optimized for browsing media files with
/// grid view and image/video file filters.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::media_gallery_explorer;
///
/// let gallery = media_gallery_explorer("/home/user/Pictures")
///     .show_preview(true)
///     .build();
/// ```
pub fn media_gallery_explorer(media_path: impl Into<PathBuf>) -> FileExplorerBuilder {
    FileExplorerBuilder::new()
        .current_path(media_path)
        .view_mode(ViewMode::Grid)
        .selection_mode(SelectionMode::Multiple)
        .file_filters(vec![
            "jpg".to_string(),
            "jpeg".to_string(),
            "png".to_string(),
            "gif".to_string(),
            "bmp".to_string(),
            "svg".to_string(),
            "webp".to_string(),
            "mp4".to_string(),
            "avi".to_string(),
            "mkv".to_string(),
            "mov".to_string(),
            "mp3".to_string(),
            "wav".to_string(),
            "flac".to_string(),
            "ogg".to_string(),
        ])
        .show_preview(true)
        .show_details(false)
        .sort_by(SortCriteria::Modified)
        .class("media-gallery-explorer")
}

/// Create a document browser
///
/// Creates a file explorer optimized for browsing documents with
/// appropriate filters and list view for detailed information.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::document_browser;
///
/// let browser = document_browser("/home/user/Documents")
///     .search("report")
///     .build();
/// ```
pub fn document_browser(docs_path: impl Into<PathBuf>) -> FileExplorerBuilder {
    FileExplorerBuilder::new()
        .current_path(docs_path)
        .view_mode(ViewMode::List)
        .selection_mode(SelectionMode::Multiple)
        .file_filters(vec![
            "pdf".to_string(),
            "doc".to_string(),
            "docx".to_string(),
            "txt".to_string(),
            "md".to_string(),
            "rtf".to_string(),
            "odt".to_string(),
            "xls".to_string(),
            "xlsx".to_string(),
            "ppt".to_string(),
            "pptx".to_string(),
        ])
        .show_details(true)
        .show_preview(false)
        .sort_by(SortCriteria::Modified)
        .class("document-browser")
}

/// Create a compact file picker
///
/// Creates a minimal file explorer suitable for file selection dialogs
/// with compact layout and single selection mode.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::compact_file_picker;
///
/// let picker = compact_file_picker("/home/user")
///     .file_filters(vec!["txt".to_string(), "md".to_string()])
///     .build();
/// ```
pub fn compact_file_picker(start_path: impl Into<PathBuf>) -> FileExplorerBuilder {
    FileExplorerBuilder::new()
        .current_path(start_path)
        .view_mode(ViewMode::List)
        .selection_mode(SelectionMode::Single)
        .show_hidden(false)
        .show_details(false)
        .show_breadcrumb(true)
        .show_preview(false)
        .max_visible_items(20)
        .class("compact-file-picker")
}

/// Create a system file manager
///
/// Creates a full-featured file explorer suitable for system administration
/// with all features enabled and appropriate permissions handling.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::system_file_manager;
///
/// let manager = system_file_manager("/")
///     .show_hidden(true)
///     .build();
/// ```
pub fn system_file_manager(root_path: impl Into<PathBuf> + Clone) -> FileExplorerBuilder {
    let path = root_path.into();
    FileExplorerBuilder::new()
        .root_path(path.clone())
        .current_path(path)
        .view_mode(ViewMode::List)
        .selection_mode(SelectionMode::Multiple)
        .show_hidden(true)
        .show_details(true)
        .show_breadcrumb(true)
        .show_preview(false)
        .sort_by(SortCriteria::Name)
        .keyboard_navigation(true)
        .class("system-file-manager")
}

/// Create a file explorer from current directory
///
/// Convenience function to create a file explorer starting from
/// the current working directory.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::current_directory_explorer;
///
/// let explorer = current_directory_explorer();
/// ```
pub fn current_directory_explorer() -> Element {
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
    simple_file_browser(current_dir)
}

/// Create a home directory explorer
///
/// Convenience function to create a file explorer starting from
/// the user's home directory.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::home_directory_explorer;
///
/// let explorer = home_directory_explorer();
/// ```
pub fn home_directory_explorer() -> Element {
    let home_dir = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"));
    simple_file_browser(home_dir)
}

/// Create a filtered file explorer
///
/// Convenience function to create a file explorer with specific file type filters.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::filtered_file_explorer;
///
/// let explorer = filtered_file_explorer(
///     "/home/user/projects",
///     vec!["rs", "toml", "md"]
/// );
/// ```
pub fn filtered_file_explorer(path: impl Into<PathBuf>, extensions: Vec<&str>) -> Element {
    let filters: Vec<String> = extensions.into_iter().map(|s| s.to_string()).collect();

    FileExplorerBuilder::new()
        .current_path(path)
        .file_filters(filters)
        .view_mode(ViewMode::List)
        .selection_mode(SelectionMode::Multiple)
        .show_details(true)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::ViewMode;

    #[test]
    fn test_file_explorer_builder() {
        let explorer = file_explorer()
            .current_path("/test")
            .view_mode(ViewMode::List)
            .build();

        assert!(explorer.is_component());
        assert_eq!(explorer.component_name(), Some("FileExplorer"));
    }

    #[test]
    fn test_simple_file_browser() {
        let browser = simple_file_browser("/home/user");
        assert!(browser.is_component());
    }

    #[test]
    fn test_code_project_explorer() {
        let explorer = code_project_explorer("/project")
            .file_filters(vec!["rs".to_string()])
            .build();

        assert!(explorer.is_component());
    }

    #[test]
    fn test_media_gallery_explorer() {
        let gallery = media_gallery_explorer("/pictures")
            .show_preview(true)
            .build();

        assert!(gallery.is_component());
    }

    #[test]
    fn test_document_browser() {
        let browser = document_browser("/documents").search("test").build();

        assert!(browser.is_component());
    }

    #[test]
    fn test_compact_file_picker() {
        let picker = compact_file_picker("/")
            .file_filters(vec!["txt".to_string()])
            .build();

        assert!(picker.is_component());
    }

    #[test]
    fn test_current_directory_explorer() {
        let explorer = current_directory_explorer();
        assert!(explorer.is_component());
    }

    #[test]
    fn test_filtered_file_explorer() {
        let explorer = filtered_file_explorer("/test", vec!["rs", "md"]);
        assert!(explorer.is_component());
    }
}
