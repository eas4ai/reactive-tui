//! The look every layout widget shares (docs/spec/layout-widgets.md,
//! NAV-001): the classes that name each part's role. A layout widget
//! shows focus, hover and disabled by these colors alone; it adds no
//! glyph and changes no text.

/// A tab's label, a breadcrumb's segment or separator, an accordion's
/// glyph: `text-muted`.
pub const MUTED: &str = "text-muted";
/// The selected tab's label, a breadcrumb's current segment, an
/// accordion's title: `foreground`.
pub const TEXT: &str = "text-foreground";
/// The selected tab in the `Line` variant: `foreground`, underlined.
pub const TAB_LINE: &str = "text-foreground underline";
/// The selected tab in the `Enclosed` variant: on `surface`.
pub const TAB_ENCLOSED: &str = "bg-surface text-foreground";
/// The selected tab in the `Soft` variant: on `secondary`.
pub const TAB_SOFT: &str = "bg-secondary text-secondary-foreground";
/// The selected tab in the `Solid` variant: on `primary`.
pub const TAB_SOLID: &str = "bg-primary text-primary-foreground";
/// A breadcrumb's clickable segment: `text-muted`, underlined as a link.
pub const LINK: &str = "text-muted underline";
/// The tab, section header or segment that holds the keyboard focus,
/// while the widget holds the focus.
pub const FOCUSED: &str = "bg-selection text-selection-foreground";
/// The tab, section header or segment under the pointer.
pub const HOVER: &str = "bg-hover";
/// A disabled tab, section or segment.
pub const DISABLED: &str = "text-muted";
/// A tooltip: `surface` with `foreground` text.
pub const TOOLTIP: &str = "bg-surface text-foreground";
/// A scroll bar's track.
pub const TRACK: &str = "text-border";
/// A scroll bar's thumb.
pub const THUMB: &str = "text-muted";
/// A split's divider and a sidebar's edge: `border` (NAV-007).
pub const DIVIDER: &str = "text-border";
/// A sidebar's fill: `surface` (NAV-007).
pub const BAR: &str = "bg-surface";
/// A sidebar's active item: `accent` with `accent-foreground` text
/// (NAV-007).
pub const ACTIVE: &str = "bg-accent text-accent-foreground";
/// A log view's info, success, warning and error rows (NAV-007); a plain
/// row is `TEXT`.
pub const LOG_INFO: &str = "text-info";
/// A log view's success row.
pub const LOG_SUCCESS: &str = "text-success";
/// A log view's warning row.
pub const LOG_WARNING: &str = "text-warning";
/// A log view's error row.
pub const LOG_ERROR: &str = "text-error";
/// A log view's `↓ N new` bar: `primary` with its text, drawn as a pill
/// where the terminal takes pixels (PIX-009).
pub const NEW_BAR: &str = "bg-primary text-primary-foreground rounded-full";
