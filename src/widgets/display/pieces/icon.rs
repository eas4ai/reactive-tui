//! The icon catalog (DIS-005): one set of named marks for kinds, states and
//! navigation. Each mark has a single-width Unicode glyph and a one-byte ASCII
//! fallback. An application may give the catalog a Nerd Font set that replaces
//! the glyphs for every icon the crate paints.

use std::sync::RwLock;

/// A named mark from the icon catalog (DIS-005). Widgets paint kind and state
/// marks through this type, so one idea draws one glyph everywhere.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Icon {
    /// Information, such as a note or a hint.
    Info,
    /// A caution that does not stop the work.
    Warning,
    /// A failure.
    Error,
    /// A finished step that worked.
    Success,
    /// Dismiss or close a panel.
    Close,
    /// A check mark for a chosen or done item.
    Check,
    /// A plain marker for a list item or a status.
    Dot,
    /// An open circle, such as a step not reached yet.
    Circle,
    /// An arrow that points up, such as a sort order.
    ChevronUp,
    /// An arrow that points down, such as an expanded order.
    ChevronDown,
    /// An arrow that points left, such as a previous page.
    ChevronLeft,
    /// An arrow that points right, such as a next page.
    ChevronRight,
    /// A closed folder.
    Folder,
    /// An open folder.
    FolderOpen,
    /// A file that is not a folder.
    File,
    /// Search or find.
    Search,
    /// Settings or configuration.
    Settings,
    /// Add or create.
    Plus,
    /// Remove.
    Minus,
    /// Omitted text, or more items than shown.
    Ellipsis,
    /// The first frame of a spinner that shows work in progress.
    Spinner,
}

/// A table that gives a Nerd Font glyph for an icon, or `None` for an icon the
/// font set does not name.
pub type NerdFontTable = fn(Icon) -> Option<&'static str>;

/// The application's Nerd Font set, if it gave one (see [`set_nerd_font`]).
static NERD_FONT: RwLock<Option<NerdFontTable>> = RwLock::new(None);

/// The frames of [`Icon::Spinner`] where the terminal takes Unicode: the
/// braille dots, one cell each, turning in a circle.
pub const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// The frames of [`Icon::Spinner`] where the terminal reports no Unicode:
/// the one-byte ASCII fallback, turning in the same order.
pub const SPINNER_ASCII_FRAMES: &[&str] = &["|", "/", "-", "\\"];

impl Icon {
    /// Every icon of the catalog, in declaration order.
    pub const ALL: &'static [Icon] = &[
        Icon::Info,
        Icon::Warning,
        Icon::Error,
        Icon::Success,
        Icon::Close,
        Icon::Check,
        Icon::Dot,
        Icon::Circle,
        Icon::ChevronUp,
        Icon::ChevronDown,
        Icon::ChevronLeft,
        Icon::ChevronRight,
        Icon::Folder,
        Icon::FolderOpen,
        Icon::File,
        Icon::Search,
        Icon::Settings,
        Icon::Plus,
        Icon::Minus,
        Icon::Ellipsis,
        Icon::Spinner,
    ];

    /// The single-width Unicode glyph for this icon. Used where the terminal
    /// takes Unicode.
    pub fn unicode(self) -> &'static str {
        match self {
            Icon::Info => "•",
            Icon::Warning => "⚠",
            Icon::Error => "×",
            Icon::Success => "✓",
            Icon::Close => "×",
            Icon::Check => "✓",
            Icon::Dot => "●",
            Icon::Circle => "○",
            Icon::ChevronUp => "▲",
            Icon::ChevronDown => "▼",
            Icon::ChevronLeft => "‹",
            Icon::ChevronRight => "›",
            Icon::Folder => "▪",
            Icon::FolderOpen => "▫",
            Icon::File => "·",
            Icon::Search => "⌕",
            Icon::Settings => "⚙",
            Icon::Plus => "+",
            Icon::Minus => "−",
            Icon::Ellipsis => "…",
            Icon::Spinner => "⠋",
        }
    }

    /// The one-byte ASCII fallback for this icon. Used where the terminal
    /// reports no Unicode.
    pub fn ascii(self) -> &'static str {
        match self {
            Icon::Info => "i",
            Icon::Warning => "!",
            Icon::Error => "x",
            Icon::Success => "v",
            Icon::Close => "x",
            Icon::Check => "v",
            Icon::Dot => "*",
            Icon::Circle => "o",
            Icon::ChevronUp => "^",
            Icon::ChevronDown => "v",
            Icon::ChevronLeft => "<",
            Icon::ChevronRight => ">",
            Icon::Folder => "+",
            Icon::FolderOpen => "-",
            Icon::File => ".",
            Icon::Search => "?",
            Icon::Settings => "*",
            Icon::Plus => "+",
            Icon::Minus => "-",
            Icon::Ellipsis => "~",
            Icon::Spinner => "|",
        }
    }

    /// The glyph to paint: the application's Nerd Font glyph when it set one
    /// for this icon, else the Unicode glyph where the terminal takes Unicode,
    /// else the ASCII fallback. The Unicode check is the same report the charts
    /// read for braille and block glyphs.
    pub fn glyph(self) -> &'static str {
        let nerd = *NERD_FONT.read().unwrap_or_else(|e| e.into_inner());
        if let Some(glyph) = nerd.and_then(|table| table(self)) {
            glyph
        } else if crate::widgets::display::charts::glyph_support() {
            self.unicode()
        } else {
            self.ascii()
        }
    }

    /// The English name a screen reader speaks for this icon, such as
    /// "Warning" or "Folder open".
    pub fn name(self) -> &'static str {
        match self {
            Icon::Info => "Information",
            Icon::Warning => "Warning",
            Icon::Error => "Error",
            Icon::Success => "Success",
            Icon::Close => "Close",
            Icon::Check => "Check",
            Icon::Dot => "Dot",
            Icon::Circle => "Circle",
            Icon::ChevronUp => "Chevron up",
            Icon::ChevronDown => "Chevron down",
            Icon::ChevronLeft => "Chevron left",
            Icon::ChevronRight => "Chevron right",
            Icon::Folder => "Folder",
            Icon::FolderOpen => "Folder open",
            Icon::File => "File",
            Icon::Search => "Search",
            Icon::Settings => "Settings",
            Icon::Plus => "Plus",
            Icon::Minus => "Minus",
            Icon::Ellipsis => "Ellipsis",
            Icon::Spinner => "Loading",
        }
    }
}

/// Give the catalog an application's Nerd Font set, or clear it with `None`.
/// The set replaces the Unicode glyphs for every icon the crate paints, in
/// every widget, for the whole process. An icon the set does not name keeps
/// its Unicode or ASCII glyph.
pub fn set_nerd_font(table: Option<fn(Icon) -> Option<&'static str>>) {
    *NERD_FONT.write().unwrap_or_else(|e| e.into_inner()) = table;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::display::charts::{
        glyph_support, report_glyph_support, GLYPH_REPORT_TEST_LOCK,
    };
    use unicode_width::UnicodeWidthStr;

    #[test]
    fn dis_005_every_icon_has_a_one_cell_unicode_glyph_and_a_one_byte_ascii_fallback() {
        assert_eq!(Icon::ALL.len(), 21, "the catalog names twenty-one icons");
        for &icon in Icon::ALL {
            let unicode = icon.unicode();
            assert_eq!(
                UnicodeWidthStr::width(unicode),
                1,
                "{icon:?} glyph {unicode:?} must be one cell wide"
            );
            let ascii = icon.ascii();
            assert_eq!(
                ascii.len(),
                1,
                "{icon:?} fallback {ascii:?} must be one byte"
            );
            assert!(
                ascii.is_ascii(),
                "{icon:?} fallback {ascii:?} must be ASCII"
            );
        }
    }

    #[test]
    fn dis_005_error_and_warning_paint_different_glyphs_in_both_forms() {
        assert_ne!(Icon::Error.unicode(), Icon::Warning.unicode());
        assert_ne!(Icon::Error.ascii(), Icon::Warning.ascii());
    }

    #[test]
    #[serial_test::serial(icon_font)]
    fn dis_005_nerd_font_set_replaces_a_glyph_until_it_is_cleared() {
        let _lock = GLYPH_REPORT_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let before = glyph_support();
        report_glyph_support(true);

        fn nerd(icon: Icon) -> Option<&'static str> {
            // Nerd Font `nf-fa-times`, a private-use glyph.
            (icon == Icon::Error).then_some("\u{f00d}")
        }
        let base = Icon::Error.unicode();

        set_nerd_font(Some(nerd));
        assert_eq!(Icon::Error.glyph(), "\u{f00d}");
        assert_eq!(Icon::Warning.glyph(), Icon::Warning.unicode());

        set_nerd_font(None);
        assert_eq!(Icon::Error.glyph(), base);

        report_glyph_support(before);
    }

    #[test]
    #[serial_test::serial(icon_font)]
    fn dis_005_no_unicode_report_paints_the_ascii_fallback() {
        let _lock = GLYPH_REPORT_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let before = glyph_support();
        report_glyph_support(false);
        assert_eq!(Icon::Warning.glyph(), Icon::Warning.ascii());
        report_glyph_support(before);
    }
}
