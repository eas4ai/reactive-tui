//! The look every control shares (docs/spec/input-widgets.md, CTL-001):
//! the classes that name each part's role, and the rows of colored text
//! pieces the controls are drawn with. A control shows focus, hover and
//! disabled by these colors alone; it adds no glyph and changes no text.

use crate::component::{Element, LayoutType};

/// A field: the row of cells a text input or a select shows its value in.
pub const FIELD: &str = "bg-input text-foreground";
/// A field's placeholder, a select's caret, a text input's line numbers.
pub const MUTED: &str = "bg-input text-muted";
/// A disabled field: its text in `text-muted`.
pub const FIELD_DISABLED: &str = "bg-input text-muted";
/// The frame of a field, a checkbox's box or a radio's circle.
pub const FRAME: &str = "text-border";
/// The frame while its control holds the focus.
pub const FRAME_FOCUSED: &str = "text-ring";
/// A field's frame, on the field's fill.
pub const FIELD_FRAME: &str = "bg-input text-border";
/// A field's frame while the control holds the focus.
pub const FIELD_FRAME_FOCUSED: &str = "bg-input text-ring";
/// The text input's cursor cell: the field reversed.
pub const CURSOR: &str = "bg-foreground text-input";
/// The text input's selection.
pub const SELECTION: &str = "bg-selection text-selection-foreground";
/// A checked or mixed box's mark, a chosen radio's dot, a slider's filled
/// part, a chosen option's mark.
pub const MARK: &str = "text-primary";
/// A control's label and text.
pub const LABEL: &str = "text-foreground";
/// A disabled control's label and text.
pub const LABEL_DISABLED: &str = "text-muted";
/// A slider's track.
pub const TRACK: &str = "text-border";
/// A slider's thumb.
pub const THUMB: &str = "text-foreground";
/// A slider's thumb while the slider holds the focus.
pub const THUMB_FOCUSED: &str = "text-ring";
/// The row of a checkbox, a radio or a select's option under the pointer.
pub const HOVER: &str = "bg-hover";
/// A select's open list: a panel in `surface` with a border in `border`.
pub const PANEL: &str = "bg-surface text-foreground";
/// The current row of a select's open list.
pub const CURRENT_ROW: &str = "bg-selection text-selection-foreground";
/// The look of `builder::button()`: one row, one cell of padding at each
/// side, `secondary` with its text, `selection` while focused.
pub const BUTTON: &str = "h-1 px-1 bg-secondary text-secondary-foreground cursor-pointer focus:bg-selection focus:text-selection-foreground";
/// The look of `builder::primary_button()`: as `BUTTON`, in `primary`.
pub const PRIMARY_BUTTON: &str = "h-1 px-1 bg-primary text-primary-foreground cursor-pointer focus:bg-selection focus:text-selection-foreground";

/// One row of colored text pieces: each piece is drawn in the classes
/// beside it, so a frame, a mark and a label take their own roles. Pieces
/// with the same classes are joined. `row_classes` style the row itself
/// (the hover fill, for one).
pub fn row(pieces: &[(&str, &str)], row_classes: &str) -> Element {
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
    Element::layout(LayoutType::Flex)
        .class(format!(
            "flex flex-row h-1 shrink-0 whitespace-pre {row_classes}"
        ))
        .children(
            joined
                .into_iter()
                .map(|(text, classes)| {
                    Element::text(text).class(format!("shrink-0 whitespace-pre {classes}"))
                })
                .collect(),
        )
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
