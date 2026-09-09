# Text and interaction styling

The App/SuprTUI path resolves `focus:`, `focus-within:`, `hover:` and
`disabled:` against node state. Compound prefixes require every condition,
for example `focus:hover:bg-red-500`. Utility order is preserved: the last
matching utility for a property wins. State is resolved before class caching.

Focus belongs to the event router. Hover uses the last cell mouse position
and the most recently presented bounds; a hovered descendant also makes its
ancestors hovered. `focus-within:` includes the node itself and descendants.
New autofocus targets receive focus after their first successful presentation;
App schedules a second frame if that changes styling. Geometry changes retest
a stationary pointer after presentation. Focus and hover changes repaint even
when the application has no activation handler.

Set `Element::disabled(bool)` or `ElementBuilder::disabled(bool)` to control
disabled styling, focus eligibility and owned activation callbacks. A disabled
component applies that flag to its rendered root. Disabled state is per node;
it does not disable independently interactive descendants. This is the shared
element behavior, not a claim that every widget has completed its integration.

Standalone class parsers have no event state, so these prefixes are inactive.
Pass ordinary classes to a direct `NodeSpec`; use App for interaction variants.

## Terminal typography

Text is measured and painted in terminal cells. Unicode grapheme clusters stay
whole through clipping, wrapping and ellipsis. Font attributes, case, whitespace,
word breaking, alignment and baseline spacing inherit from ancestors; an explicit
child utility overrides the inherited setting. Overflow mode applies to its own
box. Without whitespace utilities, existing explicit lines and spaces are kept.

| Tokens | Cell behavior |
| --- | --- |
| `uppercase`, `lowercase` | Unicode case conversion, including expansions such as `ß` to `SS`. |
| `capitalize` | Uppercase the first character of each Unicode word segment; retain the rest of its case. |
| `normal-case` | Keep the original text, overriding inherited case conversion. |
| `text-left`, `text-center`, `text-right` | Position text within the box's content width; center rounds down. They do not move flex children. |
| `text-justify` | Distribute spare cells across spaces on soft-wrapped lines. Keep the final line of each explicit paragraph left aligned. |
| `truncate` | Collapse whitespace, use one line, clip overflow and reserve a final `…` when text exceeds the content width. |
| `text-ellipsis`, `text-clip` | Select ellipsis or whole-grapheme clipping on an overflowing line; they do not independently change whitespace or wrapping. |
| `whitespace-normal` | Collapse whitespace and wrap at Unicode word boundaries. |
| `whitespace-nowrap` | Collapse whitespace and keep one line. |
| `whitespace-pre` | Preserve spaces and newlines without soft wrapping. |
| `whitespace-pre-line` | Preserve newlines, collapse other whitespace and wrap. |
| `whitespace-pre-wrap` | Preserve spaces and newlines and allow soft wrapping. |
| `break-normal` | Wrap at Unicode word boundaries; an overlong word overflows and is clipped at the box edge. |
| `break-words` | Also split an overlong word at whole grapheme boundaries. |
| `break-all` | Allow wrapping between any two grapheme clusters. |
| `italic`, `not-italic` | Set or reset italic. |
| `underline`, `no-underline` | Set or reset underline. |
| `overline` | Retain the existing underline approximation. |
| `line-through` | Emit terminal strikethrough (SGR 9). |
| `font-thin`, `font-extralight`, `font-light`, `font-normal`, `font-medium`, `font-semibold` | Normal terminal weight, retaining the existing approximation for weights 100–600. |
| `font-bold`, `font-extrabold`, `font-black` | Bold terminal weight. |
| `leading-none`, `leading-tight`, `leading-snug` | One cell row between baselines (ratios 1, 1.25 and 1.375 rounded to cells). |
| `leading-normal`, `leading-relaxed`, `leading-loose` | Two rows between baselines (ratios 1.5, 1.625 and 2 rounded to cells). |
| `leading-N` | Round finite nonnegative N to cell rows, clamped to 1–65535. No extra spacing follows the final baseline. Invalid values are not accepted. |
| `tracking-tighter`, `tracking-tight`, `tracking-normal`, `tracking-wide`, `tracking-wider`, `tracking-widest` | Their fractional-cell distances round to zero; retain adjacent graphemes. Negative overlap is not drawn. |
| `text-xs` through `text-9xl`, `font-sans`, `font-serif`, `font-mono` | Host-font aliases. The terminal controls physical glyph size and font family; these labels retain cell geometry. |

Preserved tabs advance to four-cell stops. Application control characters are
removed before measurement and painting; text cannot inject terminal commands.
The shared layout counts baseline spacing without allocating blank rows for a
large `leading-N` value.

Independent expected text, attributes, output commands and state transitions
are exercised by `tests/api_styling.rs` through the API-009 mechanism. Gradients,
animation and widget-specific integration have separate acceptance requirements.
