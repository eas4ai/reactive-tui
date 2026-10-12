//! The look every control shares (docs/spec/input-widgets.md, CTL-001, and
//! CTL-007 for the number input, the code input, the rating, the switch and
//! the form): the classes that name each part's role, and the rows of
//! colored text pieces the controls are drawn with. A control shows focus,
//! hover and disabled by these colors alone; it adds no glyph and changes no
//! text.

use crate::component::{Element, LayoutType};

/// A field: the row of cells a text input, a select or a number input
/// shows its value in, and a code input's digit cells.
pub const FIELD: &str = "bg-input text-foreground";
/// A field's placeholder, a select's caret, a text input's line numbers, a
/// code input's empty cell.
pub const MUTED: &str = "bg-input text-muted";
/// A disabled field: its text in `text-muted`.
pub const FIELD_DISABLED: &str = "bg-input text-muted";
/// A field's text where the field is a picture: the cells take the
/// picture's fill under them, so a text cell and the picture beside it are
/// one color, at half where the input is disabled (PIX-001, PIX-004).
pub const FIELD_PICTURE: &str = "text-foreground";
/// A placeholder, and a disabled field's text, where the field is a
/// picture.
pub const MUTED_PICTURE: &str = "text-muted";
/// The frame of a field, a checkbox's box or a radio's circle.
pub const FRAME: &str = "text-border";
/// The frame while its control holds the focus.
pub const FRAME_FOCUSED: &str = "text-ring";
/// A field's frame, on the field's fill: also each code input cell's `[`
/// and `]`.
pub const FIELD_FRAME: &str = "bg-input text-border";
/// A field's frame while the control holds the focus.
pub const FIELD_FRAME_FOCUSED: &str = "bg-input text-ring";
/// A field's frame while its value is invalid.
pub const FIELD_FRAME_INVALID: &str = "bg-input text-error";
/// A blank run of a field drawn with pixels: it holds no glyph, so the
/// field's picture shows there (PIX-004); never joined with the text.
pub const FIELD_PAD: &str = "field-pad";
/// The cursor cell of a text input or a number input: the field reversed.
pub const CURSOR: &str = "bg-foreground text-input";
/// The text input's selection.
pub const SELECTION: &str = "bg-selection text-selection-foreground";
/// A checked or mixed box's mark, a chosen radio's dot, a slider's filled
/// part, a chosen option's mark, a switch's thumb while it is on.
pub const MARK: &str = "text-primary";
/// A switch's thumb while it is off (CTL-007).
pub const MARK_OFF: &str = "text-muted";
/// A control's label and text.
pub const LABEL: &str = "text-foreground";
/// A disabled control's label and text, and a disabled rating's stars.
pub const LABEL_DISABLED: &str = "text-muted";
/// A form field's required mark, after its label (CTL-007).
pub const REQUIRED_MARK: &str = "text-error";
/// A form field's description (CTL-007).
pub const DESCRIPTION: &str = "text-muted";
/// A slider's track.
pub const TRACK: &str = "text-border";
/// A slider's thumb.
pub const THUMB: &str = "text-foreground";
/// A slider's thumb while the slider holds the focus.
pub const THUMB_FOCUSED: &str = "text-ring";
/// A lit star of a rating (CTL-007).
pub const RATING_LIT: &str = "text-warning";
/// An unlit star of a rating.
pub const RATING_UNLIT: &str = "text-border";
/// An unlit star of a rating while the rating holds the focus.
pub const RATING_UNLIT_FOCUSED: &str = "text-ring";
/// A number input's button cells, `[-]` and `[+]`: `secondary` with the
/// glyph in `secondary-foreground` (CTL-007).
pub const STEPPER: &str = "bg-secondary text-secondary-foreground";
/// A number input's button that cannot step further, and both buttons
/// while the number input is disabled: the glyph in `text-muted` on
/// `secondary`.
pub const STEPPER_MUTED: &str = "bg-secondary text-muted";
/// The row of a checkbox, a radio, a select's option or a switch under the
/// pointer.
pub const HOVER: &str = "bg-hover";
/// A select's open list: a panel in `surface` with a border in `border`.
pub const PANEL: &str = "bg-surface text-foreground";
/// The current row of a select's open list.
pub const CURRENT_ROW: &str = "bg-selection text-selection-foreground";
/// The look of `builder::button()`: one row, one cell of padding at each
/// side, `secondary` with its text, `selection` while focused, its text
/// `text-muted` when disabled. Where the terminal takes pixels the same
/// classes draw its pixel look (PIX-003): a rounded fill, a two-pixel ring
/// in `ring` while focused with a pixel left empty between it and the
/// `selection` fill (PIX-008), the fill at 90 percent under the pointer and at
/// half when disabled. The ring tokens come before `focus:bg-selection`, so
/// in cells the focused fill stays `selection`.
pub const BUTTON: &str = "h-1 px-1 bg-secondary text-secondary-foreground cursor-pointer rounded focus:ring-2 focus:ring-ring focus:bg-selection focus:text-selection-foreground hover:bg-opacity-90 disabled:text-muted disabled:bg-opacity-50";
/// The look of `builder::primary_button()`: as `BUTTON`, in `primary`.
pub const PRIMARY_BUTTON: &str = "h-1 px-1 bg-primary text-primary-foreground cursor-pointer rounded focus:ring-2 focus:ring-ring focus:bg-selection focus:text-selection-foreground hover:bg-opacity-90 disabled:text-muted disabled:bg-opacity-50";

/// One row of colored text pieces: each piece is drawn in the classes
/// beside it, so a frame, a mark and a label take their own roles. Pieces
/// with the same classes are joined. `row_classes` style the row itself
/// (the hover fill, for one).
pub fn row(pieces: &[(&str, &str)], row_classes: &str) -> Element {
    row_of(self::pieces(pieces), row_classes)
}

/// The text elements of `pieces`, each in the classes beside it, pieces
/// with the same classes joined.
pub fn pieces(pieces: &[(&str, &str)]) -> Vec<Element> {
    let mut joined: Vec<(String, String)> = Vec::new();
    for (text, classes) in pieces {
        if text.is_empty() {
            continue;
        }
        match joined.last_mut() {
            Some((last, last_classes)) if last_classes == classes => last.push_str(text),
            _ => joined.push(((*text).to_owned(), (*classes).to_owned())),
        }
    }
    joined
        .into_iter()
        .map(|(text, classes)| {
            Element::text(text).class(format!("shrink-0 whitespace-pre {classes}"))
        })
        .collect()
}

/// One row of `children`, styled as `row` styles its rows.
pub fn row_of(children: Vec<Element>, row_classes: &str) -> Element {
    Element::layout(LayoutType::Flex)
        .class(format!(
            "flex flex-row h-1 shrink-0 whitespace-pre {row_classes}"
        ))
        .children(children)
}

/// Whether a control rendering now draws its pixel look: the terminal of
/// the App rendering takes pixels and no switch turns the looks back to
/// cells (PIX-001). Without the `wgpu-graphics` feature every control
/// draws its cells.
pub fn pixels() -> bool {
    #[cfg(feature = "wgpu-graphics")]
    {
        crate::graphics::look::pixels_on()
    }
    #[cfg(not(feature = "wgpu-graphics"))]
    {
        false
    }
}

/// A run of `cells` cells on one row that holds no glyph: the cells a
/// control's picture is placed over, in place of its frame glyphs.
pub fn blank(cells: usize) -> Element {
    Element::layout(LayoutType::Flex).class(format!("w-{cells} h-1 shrink-0"))
}

/// A run of `cells` cells on one row that holds no glyph, over which the
/// picture of `look` is placed: a control's box, circle or track where the
/// terminal takes pixels (PIX-004, PIX-005).
#[cfg(feature = "wgpu-graphics")]
pub fn spacer(cells: usize, look: crate::graphics::look::Look) -> Element {
    let mut element = blank(cells);
    element.metadata.look = Some(std::sync::Arc::new(look));
    element
}

/// The classes of a label: `foreground`, or `text-muted` when disabled.
pub fn label(disabled: bool) -> &'static str {
    if disabled {
        LABEL_DISABLED
    } else {
        LABEL
    }
}

/// The classes of a frame: `border`, or `ring` while focused.
pub fn frame(focused: bool) -> &'static str {
    if focused {
        FRAME_FOCUSED
    } else {
        FRAME
    }
}

/// The classes of a field's frame: `border` or `ring` on the field's fill.
pub fn field_frame(focused: bool) -> &'static str {
    if focused {
        FIELD_FRAME_FOCUSED
    } else {
        FIELD_FRAME
    }
}
