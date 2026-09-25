//! Widget library for reactive-tui
//!
//! This module provides a comprehensive collection of pre-built UI components
//! for building terminal applications. The widgets are organized into categories:
//!
//! - **dialog**: Modal dialogs, toasts, and interactive prompts
//! - **display**: Data visualization and content display widgets
//! - **input**: Form controls and user input widgets
//! - **layout**: Container and layout management widgets
//! - **menu**: Menu system including menubar, context menus, and popup menus
//! - **terminal**: Terminal emulator widget with PTY support

/// Dialog system for modal interactions and notifications
pub mod dialog;
/// Display widgets for data visualization and content
pub mod display;
/// Input widgets for forms and user interaction
pub mod input;
/// Layout widgets for organizing UI components
pub mod layout;
/// Menu system for navigation and context menus
pub mod menu;
/// Terminal emulator widget with PTY support
pub mod terminal;

// Re-export all dialog components and their types
pub use dialog::{
    AutocompleteDialog, AutocompleteDialogOptions, AutocompleteSuggestion, ConfirmationDialog,
    ConfirmationDialogOptions, DialogBuilder, DialogComponent, DialogEngine, DialogEngineConfig,
    DialogEvent, DialogId, DialogResult, DialogTheme, DialogThemes, InputDialog,
    InputDialogOptions, ProgressDialog, Toast, WizardDialog,
};

// Re-export all display components and their types
pub use display::{
    Chart, ChartAxis, ChartLegend, ChartProps, ChartState, ChartType, ColumnFilter, DataPoint,
    DataSeries, DataTable, DataTableProps, DataTableState, FileEntry, FileExplorer,
    FileExplorerBuilder, FileExplorerProps, FileExplorerState, FileType, FillStyle, FilterType,
    Image, ImageCapabilities, ImageDisplayMode, ImageFormat, ImageQuality, ImageSource,
    LegendPosition, LineStyle, Modal, PaginationConfig, Popover, ProgressBar, SelectionMode,
    SortCriteria, SortOrder, Table, Tree, ViewMode, VirtualScrollConfig,
};
pub use input::{
    Checkbox, CheckboxProps, CheckboxState, RadioButton, Select, SelectOption, SelectProps,
    SelectState, Slider, TextInput, TextInputProps, TextInputState,
};
pub use layout::{
    Accordion, AccordionBuilder, AccordionMode, AccordionProps, AccordionSection, AccordionState,
    Breadcrumb, BreadcrumbBuilder, BreadcrumbProps, BreadcrumbSegment, BreadcrumbState,
    OverflowStrategy, ScrollView, Stack, Tabs,
};
pub use menu::{
    ContextMenu, ContextMenuProps, ContextMenuState, DialogMenu, DialogMenuProps, DialogMenuState,
    DialogMenuType, MenuAction, MenuBar, MenuBarBuilder, MenuBarProps, MenuBarState, MenuItem,
    MenuItemType, MenuSeparator, MenuShortcut, MenuStyle, MenuTheme, PopupMenu, PopupMenuProps,
    PopupMenuState, PopupPlacement,
};
pub use terminal::{TerminalProps, TerminalState, TerminalWidget};
