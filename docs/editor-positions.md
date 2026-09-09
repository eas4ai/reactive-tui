# Editor text positions and rendering

`TextEditor` and `SyntaxEditor` use the same editing and painting rules.
`GapBuffer` positions, lengths, ranges, line columns and `Cursor.position`
count Unicode scalars (`char` values), not UTF-8 bytes or terminal cells.
For example, `界é` contains three scalars, two graphemes and three terminal
columns. GapBuffer remains a low-level scalar buffer; its individual operations
can edit a scalar inside a grapheme.

Use Cursor movement methods when editing displayed text. Left, Right, Backspace,
Delete and selection replacement operate on complete extended grapheme clusters.
`move_to` clamps to the document and snaps backward if given a position inside
a grapheme. Direct writes to the public cursor fields must use valid scalar
boundaries; they do not convert byte offsets.

Insertion moves after the inserted scalars and any grapheme they join. Deletion
snaps backward if joining the remaining text creates a new grapheme around the
cursor. LF separates lines; CRLF moves and deletes as one grapheme. Up and Down
preserve a preferred terminal column across short lines. A column inside a wide
grapheme resolves to its leading boundary. Horizontal movement and edits reset
the preferred column. Word movement skips separators, then crosses graphemes
containing letters, numbers or underscores.

Tabs expand to four-column stops measured from the text origin, excluding the
line-number gutter. A standalone zero-width grapheme is displayed with a dotted
circle. Other control characters display as replacement characters. They are
never emitted as terminal commands. The text stored in the buffer is unchanged.

Styled lines preserve syntax styles and overlay only selected graphemes. Selection
color takes precedence over cursor color within a selection. Both styled lines
and Surface rendering clip whole graphemes at the viewport boundary. Surface
rendering clears the viewport with spaces before painting, including unused rows.
There is vertical scrolling; text beyond the right edge is clipped. A zero-size
viewport produces no painted content.

The existing `render` methods now retain complete graphemes in private Surface
metadata. `Surface::get` and the existing C-compatible `Cell` still expose one
scalar; `Surface::grapheme` returns complete leading text and an empty string for
a wide continuation cell. `DiffWriter` emits that complete text. Copying a Surface
copies the metadata; overwriting either cell of a wide grapheme clears the whole
old grapheme. Clearing or reinitializing the Surface removes the metadata. Direct
mutable access to the legacy cell buffer also clears it, because `Cell` cannot
represent a multi-scalar grapheme. Existing public layouts and signatures remain
unchanged; `SyntaxEditor::delete_forward` and the Surface grapheme methods are
additive.

Behavior evidence: `tests/api_editor_unicode.rs`, run by
`scripts/check-api-editor-unicode.py` for API-007. Other legacy Surface writers
and application adapters retain their separate API-016 integration requirement.
