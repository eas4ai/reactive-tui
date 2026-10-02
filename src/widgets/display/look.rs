//! The look every data widget shares (docs/spec/data-widgets.md, DAT-001):
//! the classes that name each part's role. A data widget shows focus,
//! hover and selection by these colors alone; it adds no glyph and changes
//! no text.

/// A row's text, a progress bar's label and its value: `foreground`.
pub const TEXT: &str = "text-foreground";
/// A header's title: `foreground`, bold.
pub const HEADER: &str = "text-foreground font-bold";
/// The sorted column's mark, a checked tree checkbox's mark: `primary`.
pub const MARK: &str = "text-primary";
/// A box's border, a progress bar's track: `border`.
pub const BORDER: &str = "text-border";
/// A tree's lines, expanders and checkbox frames, a file explorer's hints
/// and sizes, a data table's page count: `text-muted`.
pub const MUTED: &str = "text-muted";
/// A selected row: `accent` with its text.
pub const SELECTED: &str = "bg-accent text-accent-foreground";
/// The row that holds the keyboard cursor, while the widget holds the
/// focus: `selection` with its text.
pub const CURSOR: &str = "bg-selection text-selection-foreground";
/// The row under the pointer: `hover`, with the row's own text.
pub const HOVER: &str = "bg-hover text-foreground";
/// A disabled row: `text-muted`.
pub const DISABLED: &str = "text-muted";
/// An error line: `text-error`.
pub const ERROR: &str = "text-error";
/// A progress bar's filled part: `primary`.
pub const FILL: &str = "text-primary";

/// The classes of a row by its state. The cursor row shows `selection`
/// while the widget holds the focus, a selected row `accent`, the row
/// under the pointer `hover`, a disabled row `text-muted`, any other row
/// its text in `foreground`; the row's text pieces inherit the color.
pub fn row(cursor: bool, selected: bool, hovered: bool, disabled: bool) -> &'static str {
    if disabled {
        DISABLED
    } else if cursor {
        CURSOR
    } else if selected {
        SELECTED
    } else if hovered {
        HOVER
    } else {
        TEXT
    }
}
