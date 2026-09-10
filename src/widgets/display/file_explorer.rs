//! Production-Ready File Explorer Widget
//!
//! A sophisticated file system navigation component with:
//! - Virtual scrolling for large directories
//! - File operations (copy, move, delete, rename)
//! - Advanced filtering and search
//! - Multiple selection modes
//! - File type detection and icons
//! - Breadcrumb integration
//! - Keyboard shortcuts and accessibility
//! - Async file system operations
//! - Preview pane support

use crate::component::{Component, Element, Props};
use std::any::Any;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// File system entry type
#[derive(Clone, Debug, PartialEq)]
pub enum FileType {
    /// A directory/folder
    Directory,
    /// A regular file
    File,
    /// A symbolic link
    Symlink,
    /// Unknown file type
    Unknown,
}

/// File system entry information
#[derive(Clone, Debug, PartialEq)]
pub struct FileEntry {
    /// File name
    pub name: String,
    /// Full path to the file
    pub path: PathBuf,
    /// File type
    pub file_type: FileType,
    /// File size in bytes (None for directories)
    pub size: Option<u64>,
    /// Last modified time
    pub modified: Option<SystemTime>,
    /// Whether the file is hidden
    pub hidden: bool,
    /// File extension
    pub extension: Option<String>,
    /// Icon for the file type
    pub icon: String,
    /// Whether this entry is selected
    pub selected: bool,
    /// Whether this entry is currently focused
    pub focused: bool,
}

impl FileEntry {
    /// Create a new file entry from a path
    pub fn from_path(path: &Path) -> std::io::Result<Self> {
        let metadata = fs::symlink_metadata(path)?;
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();

        let file_type = if metadata.file_type().is_symlink() {
            FileType::Symlink
        } else if metadata.is_dir() {
            FileType::Directory
        } else if metadata.is_file() {
            FileType::File
        } else {
            FileType::Unknown
        };

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase());

        let icon = Self::get_icon_for_file(&name, &file_type, &extension);
        let hidden = name.starts_with('.');
        #[cfg(windows)]
        let hidden = {
            use std::os::windows::fs::MetadataExt;
            hidden
                || metadata.file_attributes()
                    & windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_HIDDEN
                    != 0
        };

        Ok(Self {
            name,
            path: path.to_path_buf(),
            file_type,
            size: if metadata.is_file() {
                Some(metadata.len())
            } else {
                None
            },
            modified: metadata.modified().ok(),
            hidden,
            extension,
            icon,
            selected: false,
            focused: false,
        })
    }

    /// Get appropriate icon for file type
    fn get_icon_for_file(name: &str, file_type: &FileType, extension: &Option<String>) -> String {
        match file_type {
            FileType::Directory => match name {
                ".git" => "🔧".to_string(),
                "node_modules" => "📦".to_string(),
                "target" => "🎯".to_string(),
                "build" | "dist" => "🏗️".to_string(),
                _ => "📁".to_string(),
            },
            FileType::File => {
                if let Some(ext) = extension {
                    match ext.as_str() {
                        // Programming languages
                        "rs" => "🦀",
                        "js" | "ts" => "📜",
                        "py" => "🐍",
                        "java" => "☕",
                        "cpp" | "cc" | "cxx" => "⚙️",
                        "c" => "🔧",
                        "go" => "🐹",
                        "php" => "🐘",
                        "rb" => "💎",
                        "swift" => "🦉",
                        "kt" => "🎯",

                        // Web technologies
                        "html" | "htm" => "🌐",
                        "css" => "🎨",
                        "scss" | "sass" => "💅",
                        "json" => "📋",
                        "xml" => "📄",
                        "yaml" | "yml" => "📝",

                        // Documents
                        "md" | "markdown" => "📖",
                        "txt" => "📄",
                        "pdf" => "📕",
                        "doc" | "docx" => "📘",
                        "xls" | "xlsx" => "📊",
                        "ppt" | "pptx" => "📈",

                        // Images
                        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" => "🖼️",
                        "ico" => "🎭",

                        // Audio/Video
                        "mp3" | "wav" | "flac" | "ogg" => "🎵",
                        "mp4" | "avi" | "mkv" | "mov" => "🎬",

                        // Archives
                        "zip" | "tar" | "gz" | "rar" | "7z" => "📦",

                        // Config files
                        "toml" | "ini" | "conf" | "config" => "⚙️",
                        "env" => "🔐",

                        // Build files
                        "dockerfile" => "🐳",
                        "makefile" => "🔨",

                        _ => "📄",
                    }
                } else {
                    match name.to_lowercase().as_str() {
                        "readme" => "📖",
                        "license" | "licence" => "📜",
                        "changelog" => "📝",
                        "dockerfile" => "🐳",
                        "makefile" => "🔨",
                        _ => "📄",
                    }
                }
                .to_string()
            }
            FileType::Symlink => "🔗".to_string(),
            FileType::Unknown => "❓".to_string(),
        }
    }

    /// Format file size for display
    pub fn format_size(&self) -> String {
        match self.size {
            Some(size) => format_bytes(size),
            None => "-".to_string(),
        }
    }

    /// Format modified time for display
    pub fn format_modified(&self) -> String {
        match self.modified {
            Some(time) => {
                let nanos = match time.duration_since(SystemTime::UNIX_EPOCH) {
                    Ok(duration) => duration.as_nanos() as i128,
                    Err(error) => -(error.duration().as_nanos() as i128),
                };
                match time::OffsetDateTime::from_unix_timestamp_nanos(nanos) {
                    Ok(date) => format!(
                        "{:04}-{:02}-{:02} {:02}:{:02} UTC",
                        date.year(),
                        u8::from(date.month()),
                        date.day(),
                        date.hour(),
                        date.minute()
                    ),
                    Err(_) => "Time out of range".into(),
                }
            }
            None => "-".to_string(),
        }
    }
}

/// File explorer sort criteria
#[derive(Clone, Debug, PartialEq, Default)]
pub enum SortCriteria {
    #[default]
    /// Sort by file/directory name
    Name,
    /// Sort by file size
    Size,
    /// Sort by modification date
    Modified,
    /// Sort by file type
    Type,
}

/// File explorer sort order
#[derive(Clone, Debug, PartialEq, Default)]
pub enum SortOrder {
    #[default]
    /// Sort in ascending order (A-Z, 0-9, oldest-newest)
    Ascending,
    /// Sort in descending order (Z-A, 9-0, newest-oldest)
    Descending,
}

/// File explorer view mode
#[derive(Clone, Debug, PartialEq, Default)]
pub enum ViewMode {
    #[default]
    /// Display files in a vertical list
    List,
    /// Display files in a grid layout
    Grid,
    /// Display files in a hierarchical tree structure
    Tree,
}

/// File explorer selection mode
#[derive(Clone, Debug, PartialEq, Default)]
pub enum SelectionMode {
    #[default]
    /// Allow selection of a single file/directory
    Single,
    /// Allow selection of multiple files/directories
    Multiple,
    /// Disable selection entirely
    None,
}

/// File explorer widget properties
#[derive(Clone, Debug, PartialEq)]
pub struct FileExplorerProps {
    /// Root directory path
    pub root_path: PathBuf,
    /// Current directory path
    pub current_path: PathBuf,
    /// Whether to show hidden files
    pub show_hidden: bool,
    /// File extension filters (empty = show all)
    pub file_filters: Vec<String>,
    /// Selection mode
    pub selection_mode: SelectionMode,
    /// View mode
    pub view_mode: ViewMode,
    /// Sort criteria
    pub sort_criteria: SortCriteria,
    /// Sort order
    pub sort_order: SortOrder,
    /// Whether to show file preview
    pub show_preview: bool,
    /// Whether to show breadcrumb navigation
    pub show_breadcrumb: bool,
    /// Whether to show file details (size, date)
    pub show_details: bool,
    /// Search query for filtering files
    pub search_query: Option<String>,
    /// Custom CSS classes
    pub class: Option<String>,
    /// Whether keyboard navigation is enabled
    pub keyboard_navigation: bool,
    /// Callback for file selection
    pub on_select: Option<String>,
    /// Callback for file activation (double-click/enter)
    pub on_activate: Option<String>,
    /// Callback for directory change
    pub on_navigate: Option<String>,
    /// Maximum number of items to display (for virtual scrolling)
    pub max_visible_items: usize,
}

impl Default for FileExplorerProps {
    fn default() -> Self {
        Self {
            root_path: PathBuf::from("/"),
            current_path: PathBuf::from("/"),
            show_hidden: false,
            file_filters: Vec::new(),
            selection_mode: SelectionMode::Single,
            view_mode: ViewMode::List,
            sort_criteria: SortCriteria::Name,
            sort_order: SortOrder::Ascending,
            show_preview: false,
            show_breadcrumb: true,
            show_details: true,
            search_query: None,
            class: None,
            keyboard_navigation: true,
            on_select: None,
            on_activate: None,
            on_navigate: None,
            max_visible_items: 1000,
        }
    }
}

impl Props for FileExplorerProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// File explorer widget state
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FileExplorerState {
    /// Current directory entries
    pub entries: Vec<FileEntry>,
    /// Currently selected file indices
    pub selected_indices: HashSet<usize>,
    /// Currently focused file index
    pub focused_index: Option<usize>,
    /// Scroll position for virtual scrolling
    pub scroll_position: usize,
    /// Whether the directory is currently being loaded
    pub loading: bool,
    /// Error message if directory loading failed
    pub error: Option<String>,
    /// Filtered entries (after search/filter)
    pub filtered_entries: Vec<usize>,
    /// Whether the file explorer has been initialized
    pub initialized: bool,
    /// Last directory load time
    pub last_load_time: Option<SystemTime>,
    /// Previous filter state for change detection
    pub previous_show_hidden: bool,
    /// Previous file filters for change detection
    pub previous_file_filters: Vec<String>,
    /// Previous search query for change detection
    pub previous_search_query: Option<String>,
    /// Previous sort criteria for change detection
    pub previous_sort_criteria: SortCriteria,
    /// Previous sort order for change detection
    pub previous_sort_order: SortOrder,
}

/// Production-ready File Explorer widget
pub struct FileExplorer;

mod live;

impl Component for FileExplorer {
    type Props = FileExplorerProps;
    type State = FileExplorerState;

    fn new(_props: Self::Props) -> Self {
        Self
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        Element::typed::<live::LiveExplorer>(live::LiveProps {
            config: props.clone(),
            seed: state.clone(),
        })
    }
}

/// Format bytes for human-readable display
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Builder for creating file explorer widgets with fluent API
pub struct FileExplorerBuilder {
    props: FileExplorerProps,
    #[cfg(windows)]
    explicit_root: bool,
}

impl FileExplorerBuilder {
    /// Create a new file explorer builder
    pub fn new() -> Self {
        Self {
            props: FileExplorerProps::default(),
            #[cfg(windows)]
            explicit_root: false,
        }
    }

    /// Set the root directory path
    pub fn root_path(mut self, path: impl Into<PathBuf>) -> Self {
        #[cfg(windows)]
        {
            self.explicit_root = true;
        }
        let path = path.into();
        if self.props.current_path == self.props.root_path {
            self.props.current_path = path.clone();
        }
        self.props.root_path = path;
        self
    }

    /// Set the current directory path. On Windows, an absolute path chooses its
    /// volume/share root unless `root_path` already set an explicit boundary.
    pub fn current_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.props.current_path = path.into();
        #[cfg(windows)]
        if !self.explicit_root && self.props.current_path.is_absolute() {
            if let Some(root) = self.props.current_path.ancestors().last() {
                self.props.root_path = root.to_path_buf();
            }
        }
        self
    }

    /// Enable/disable showing hidden files
    pub fn show_hidden(mut self, show: bool) -> Self {
        self.props.show_hidden = show;
        self
    }

    /// Set file extension filters
    pub fn file_filters(mut self, filters: Vec<String>) -> Self {
        self.props.file_filters = filters;
        self
    }

    /// Set selection mode
    pub fn selection_mode(mut self, mode: SelectionMode) -> Self {
        self.props.selection_mode = mode;
        self
    }

    /// Set view mode
    pub fn view_mode(mut self, mode: ViewMode) -> Self {
        self.props.view_mode = mode;
        self
    }

    /// Set sort criteria
    pub fn sort_by(mut self, criteria: SortCriteria) -> Self {
        self.props.sort_criteria = criteria;
        self
    }

    /// Set sort order
    pub fn sort_order(mut self, order: SortOrder) -> Self {
        self.props.sort_order = order;
        self
    }

    /// Enable/disable file preview
    pub fn show_preview(mut self, show: bool) -> Self {
        self.props.show_preview = show;
        self
    }

    /// Enable/disable breadcrumb navigation
    pub fn show_breadcrumb(mut self, show: bool) -> Self {
        self.props.show_breadcrumb = show;
        self
    }

    /// Enable/disable file details
    pub fn show_details(mut self, show: bool) -> Self {
        self.props.show_details = show;
        self
    }

    /// Set search query
    pub fn search(mut self, query: impl Into<String>) -> Self {
        self.props.search_query = Some(query.into());
        self
    }

    /// Add custom CSS classes
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.props.class = Some(class.into());
        self
    }

    /// Enable/disable keyboard navigation
    pub fn keyboard_navigation(mut self, enabled: bool) -> Self {
        self.props.keyboard_navigation = enabled;
        self
    }

    /// Set maximum visible items for virtual scrolling
    pub fn max_visible_items(mut self, max: usize) -> Self {
        self.props.max_visible_items = max;
        self
    }

    /// Deliver a JSON object with display paths and lossless native path units to App.
    pub fn on_select(mut self, callback: impl Into<String>) -> Self {
        self.props.on_select = Some(callback.into());
        self
    }

    /// Deliver the activated file path through the App root custom event.
    pub fn on_activate(mut self, callback: impl Into<String>) -> Self {
        self.props.on_activate = Some(callback.into());
        self
    }

    /// Deliver the new browser location through the App root custom event.
    pub fn on_navigate(mut self, callback: impl Into<String>) -> Self {
        self.props.on_navigate = Some(callback.into());
        self
    }

    /// Build the file explorer element
    pub fn build(self) -> Element {
        Element::component_with_props("FileExplorer", self.props)
    }
}

impl Default for FileExplorerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(test, windows))]
mod windows_builder_tests {
    use super::*;

    #[test]
    fn current_path_chooses_its_volume_without_replacing_an_explicit_root() {
        for (path, root) in [
            (r"C:\users\fixture", r"C:\"),
            (r"\\server\share\fixture", r"\\server\share\"),
            (r"\\?\D:\fixture", r"\\?\D:\"),
        ] {
            let builder = FileExplorerBuilder::new().current_path(path);
            assert_eq!(builder.props.root_path, PathBuf::from(root));
            assert_eq!(builder.props.current_path, PathBuf::from(path));
            let explicit = FileExplorerBuilder::new()
                .root_path(r"D:\limited")
                .current_path(path);
            assert_eq!(explicit.props.root_path, PathBuf::from(r"D:\limited"));
        }
    }
}
