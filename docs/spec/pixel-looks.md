# Pixel looks

Prefix: PIX

A pixel look is a widget's or an element's appearance drawn as a pixel
picture where the terminal takes pixels, with its cell look kept as the
fallback. Read on 2026-10-04. Pictures reach the terminal through the
canvas: a widget mounts `Element::typed::<Canvas>(CanvasProps::new(scene))`
with a `Scene` of paths (`Path::rounded_rect`, `Path::ellipse`), paints
(`Paint::solid(Color::token("primary"))`, gradients) and strokes
(src/graphics/widget.rs:44-121, src/graphics/scene.rs:24-122,553-617); the
painter turns the finished picture into an image plane and the SuprTUI
backend writes it as Kitty graphics or Sixel (src/layout/paint_tree/suprtui.rs:595-639,
src/backend/suprtui/graphics.rs:605-725). Pixels go around text, not under
it: every text grapheme and every painted cell of a CellGrid, an explicit
space included, covers the picture in its cells, and the canvas zeroes those
pixels, so only cells nothing painted show the picture
(src/layout/paint_tree/suprtui.rs:1175-1199,1273-1306,
src/layout/paint_tree/suprtui/images.rs:235-257). Tested on the private
display on 2026-10-01: pictures around text are clean in Kitty, Konsole,
WezTerm and xterm's Sixel; pictures under text paint a black box behind every
written cell in WezTerm and cannot work with Sixel. The two-axis charts draw
their plot areas this way (CHT-037), with `REACTIVE_TUI_CANVAS=blocks` and a
process-wide `GraphicsOptions` as the switch back to cells
(src/graphics/output.rs:24-77, src/widgets/display/charts.rs:39-58). A
picture is compared with the last frame's by the identity of its pixels
(src/widgets/display/image/paint.rs:87-97), and a plane whose cells moved is
retired and sent again (src/backend/suprtui/graphics.rs:484-518); the
backend learns the cell size at startup and on every resize
(src/backend/suprtui.rs:35-85,638-644); the hit grid records one element per
cell (src/layout/paint_tree/suprtui.rs:296-325). The controls draw their cell
looks with glyphs: a checkbox `[✓]`, a radio `(●)`, a slider `[═●─]`, a
progress bar `█░▒` (src/widgets/input/checkbox.rs:154-203,
src/widgets/input/radio_button.rs:127-154, src/widgets/input/slider.rs:448-498,
src/widgets/display/progress_bar/live.rs:84-153); `builder::button()` is
`h-1 px-1 bg-secondary` with its text role (src/widgets/input/look.rs:47-52);
`builder::card()` and `card_builder()` are `bg-white border border-gray-200
rounded-lg shadow-sm` under every theme (src/builder/layout.rs:21-24,85-89),
where the utility `border` and `rounded-*` approximate a fill and
`border-<color>` sets a background (src/layout/css/effects.rs:51-161,
src/layout/css/colors.rs:77-90). No switch widget exists; the checkbox and
the radio are the toggles. The developer ruled on 2026-10-01 that every
widget moves onto pixels with its cell look as the fallback, the shared
groundwork first, proven on buttons, text inputs, toggles, progress bars,
sliders and cards, then each family in a commitment of its own (item
pixel-widget-looks, 95feb091).

## Observed

(none yet)

## Agreed

[PIX-001] When the crate is built with `wgpu-graphics` and the backend's startup report says the terminal takes Kitty graphics or Sixel, a widget or element with a pixel look (PIX-003 to PIX-005) MUST draw it as one picture over the rectangle it paints: a canvas drawn on the drawing thread (GFX-003), one picture pixel per screen pixel at the terminal's cell size (GFX-010), shown as GFX-005 chooses, made ready off the App's wait (GFX-009), with no screen-reader node of its own. Pixels go only where no text is: every cell of the rectangle that holds a glyph other than a space is cut out of the picture on its cell edges and painted in the flat color the look names for that cell, its text in the look's text role, so that the text cell's background and the picture beside it are one color; the look's text (a label, a value, a placeholder, the cursor and the selection) stays cell text and reaches the screen reader as before. Every color of a look MUST be a role of the active theme (BAR-003), and the first picture after a change of theme is in the new theme's colors (THM-003). Until a look's first picture is ready its cells show its text on its flat colors and none of the cell look's glyphs, and a picture never covers a cell outside the look's rectangle. Where the backend reports no pixels, with `REACTIVE_TUI_CANVAS=blocks`, or with a process-wide `GraphicsOptions` whose output is blocks (the switch CHT-037 gives the charts, which MUST turn every look back to cells too), the widget or element draws its cell look, and with the feature on and no pixels the bytes the terminal receives MUST be identical to the fallback's.
Falsifier: On a host that takes Kitty graphics, in a release build that writes the terminal's bytes to memory, a default `builder::primary_button()` with the label `Save` at 8 by 1 cells sends no picture, or one whose placement does not cover the button's cells, or whose pixels under the label's cells are not transparent, or whose label cells have a background other than `primary`; in a Kitty screenshot of the catalog's Input page on the private display, the background pixel of a button's label cell, of a text input's text cell or of a card's text cell differs from the picture pixel beside it by more than 2 of 255 in a channel; a look's picture has a screen-reader node; a look paints a color no role of the theme has, or its first picture after `Theme::set_active` holds a color of the old theme; a frame before a look's first picture holds a `[`, `(`, `═`, `●` or `█` of the cell look inside the look's rectangle; a picture covers a cell outside its rectangle; or, with the feature on and a host without pixels, with `REACTIVE_TUI_CANVAS=blocks`, or with blocks set through `GraphicsOptions`, the bytes differ from the fallback's or a picture is sent.
Mechanism: pixel-looks
Rationale: Tested on the private display on 2026-10-01: pictures only around text are clean in Kitty, Konsole, WezTerm and xterm's Sixel, while pictures under text paint a black box behind every cell in WezTerm and cannot work with Sixel; the developer chose around-text first and left under-text for later.
Status: Agreed 2026-10-04

[PIX-002] A look's picture MUST be kept with what drew it (its scene, its size in cells, the cell size and the cells cut out of it) and drawn again only when one of those changes. In a frame in which nothing of a look changed, no bytes of its picture MUST be sent; a change of a look (focus, hover, value, text, enabled, theme) sends one new picture for that look alone; a look whose picture is unchanged and whose rectangle moved (scrolled, laid out elsewhere) MUST, with Kitty graphics, be shown at its new cells by a new placement of the same image without its pixels being transmitted again, and with Sixel be sent again at its new cells with its old cells repainted; a look that leaves the screen MUST have its placement deleted (Kitty) or its cells repainted (Sixel) in that frame; and a change of the cell size MUST redraw every shown look at the new size before the next frame that shows it. A look's cells under a menu panel, a select's or a text input's list, a modal, a popover, a toast, a dialog or any later opaque element MUST show that element and nothing of the picture, in the frame it appears, and the picture whole again in the frame it goes.
Falsifier: On a host that takes Kitty graphics, in a release build that writes the terminal's bytes to memory: a frame after a frame with no change to any look carries image data (`a=T` or `a=t` with a payload) for one of them; a key typed into a focused text input sends two pictures for it, or a picture for another look; a button scrolled by one row inside a scroll view has its pixels transmitted again instead of placed again (`a=p` with its `i`), or on Sixel is not sent again at its new row or its old row still shows it; a widget removed from the tree leaves its placement (no `a=d` for its `i`), or on Sixel its cells; after the cell size changes from 8 by 16 to 9 by 18 pixels the next picture of a shown look is not 9 by 18 pixels per cell; or a select's list opened over a card shows the card's picture through a cell of the list, or closing it leaves a cell under it without the picture.
Mechanism: pixel-looks
Rationale: Today a plane whose cells moved is retired and sent again (src/backend/suprtui/graphics.rs:484-518) and a picture is compared by the identity of its pixels, so a page of looks that scrolls would send every picture every frame; the re-sends over Sixel and SSH are the cost the developer asked to have measured before every widget follows.
Status: Agreed 2026-10-04

[PIX-003] An element with a background role and a `rounded`, `rounded-sm`, `rounded-md`, `rounded-lg`, `rounded-xl`, `rounded-2xl` or `rounded-full` class MUST have a pixel look: its box drawn as a rounded rectangle in its background role, the radius 4, 2, 6, 8, 12 or 16 pixels at a cell height of 16 pixels and in the same proportion to the cell height otherwise, or half the box's shorter side for `rounded-full`; a `border` class with a `border-R` role class adds a border one pixel wide in R inside the edge (`border` alone, in `border`); a `ring-1` or `ring-2` class with a `ring-R` role class, also under the `focus:` variant, adds a ring that many pixels wide in R inside the edge; the cells outside the rounded corners show what is under the element. `builder::button()` and `builder::primary_button()` MUST be `rounded` with `focus:ring-2 focus:ring-ring`, in `secondary` or `primary` with their text roles (CTL-001), under the pointer their fill at 90 percent over what is under them, and disabled with their text in `text-muted` and their fill at half over what is under them. `builder::card()` and `builder::card_builder()` MUST be `bg-surface border border-border rounded-lg` under every theme, so a card is a rounded box in `surface` with a one-pixel border in `border` and its content on `surface`. Without pixels, `rounded-*`, `border` and `ring-*` MUST keep the cell behavior they have today, and a card's box is `surface`.
Falsifier: Drawn by the software renderer at 8 by 16 pixels per cell, the picture decoded from the Kitty bytes of a listed case differs from its checked-in reference picture by more than GFX-002's tolerance: `builder::button()` and `builder::primary_button()` with the label `Save` at 8 by 1 cells, each default, with the focus, under the pointer and disabled; `builder::card()` at 40 by 8 cells holding one line of text, under the dark and the light preset; a `div` of 20 by 4 cells with `bg-primary rounded-2xl`, one with `bg-surface border border-ring rounded-full`, and one with `bg-surface ring-2 ring-ring rounded`; a corner cell of a rounded box has an opaque pixel outside the arc; a button's label cell has a background other than its fill; or, without pixels, a `rounded-lg`, `border` or `ring-2` class changes a cell it does not change today, or a card's box is a color other than `surface`.
Mechanism: pixel-looks
Rationale: Tailwind's radii in pixels (rounded 4, sm 2, md 6, lg 8, xl 12, 2xl 16) and gpui-kit 0.7.0's button, whose fill lightens under the pointer and which gets a ring with the focus; the card helpers write `bg-white border-gray-200` today under every theme (src/builder/layout.rs:21-24,85-89).
Status: Agreed 2026-10-04

[PIX-004] With pixels, a text input MUST draw its field as a rounded rectangle, radius a quarter of the cell height, in `input` over its rows, with a border one pixel wide in `border`, two pixels in `ring` while it holds the focus and in `text-error` while its value is invalid, in place of its two frame cells; its text, placeholder, line numbers, cursor and selection stay cells on `input` in the roles CTL-001 names. A checkbox MUST draw, in its box's cells, a square of the cell height less two pixels with a radius of two pixels and a border one pixel wide in `border` (`ring` while it holds the focus), filled `primary` with a check mark two pixels wide in `primary-foreground` when checked and a dash of the same when mixed, in place of `[`, `]` and the mark; a radio button MUST draw, in its circle's cells, a circle of the cell height less two pixels with a border one pixel wide in `border` (`ring` while it holds the focus) and, when chosen, a dot of half that diameter in `primary`, in place of `(`, `)` and `●`. The label stays cell text in `foreground`, `text-muted` when the control is disabled; the row under the pointer stays `hover`; a disabled control's look is drawn at half over what is under it.
Falsifier: Drawn by the software renderer at 8 by 16 pixels per cell, the picture decoded from the Kitty bytes of a listed case differs from its checked-in reference picture by more than GFX-002's tolerance: a text input of 30 by 1 cells with a placeholder, one with text and the focus, one with an invalid value, and one of 30 by 4 cells with three lines of text; a checkbox unchecked, checked, mixed, checked with the focus, and disabled; a radio group of three with the second chosen, and the same with the focus on the first; a cell before or after a text input's text holds a frame glyph of the cell look, or a checkbox's box cell holds `[`; a focused control's picture holds no `ring`; or a focused text input's cursor cell reaches the terminal other than as cell text in `foreground` with `input` text.
Mechanism: pixel-looks
Rationale: gpui-kit 0.7.0's input (a rounded field with a ring on focus), checkbox (a 16-pixel rounded square with a check path) and radio (a circle with a dot); the cell look's `[`, `(` and `▶` were stand-ins for a frame a cell cannot draw.
Status: Agreed 2026-10-04

[PIX-005] With pixels, a slider MUST draw its track as a bar four pixels tall with rounded ends, centered in its row, in `border`, its filled part in `primary` to the exact pixel of its value, and its thumb as a circle of the cell height less two pixels in `foreground`, `ring` while the slider holds the focus, centered on the value's pixel, in place of `═`, `─` and `●`; its label, value and end labels stay cell text, and a click or a drag sets the value by the cell under the pointer as CTL-004 has it. A progress bar MUST draw its track as a bar with a radius of half its height in `border` over its rows and its filled part in `primary` with the same radius to the exact pixel of its value, in place of `█`, `░` and `▒`; an indeterminate bar draws a segment a quarter of the track long that glides from end to end and back over two seconds at the App's frame rate, one picture per frame, and under `reduced-motion` steps a quarter of the track once a second; a bar in error draws its filled part in `error`; its label and value stay cell text.
Falsifier: Drawn by the software renderer at 8 by 16 pixels per cell, the picture decoded from the Kitty bytes of a listed case differs from its checked-in reference picture by more than GFX-002's tolerance: a slider of 40 by 1 cells at 0, 37 and 100 of 100, at 37 with the focus, and disabled; a progress bar of 40 by 1 cells at 0, 37 and 100 percent, one of 40 by 2 cells at 37, one in error at 37, and an indeterminate bar's first and tenth frames; a slider at 37 of 100 has its thumb's center at a pixel other than 37 percent of its track's pixels; a progress bar at 37 percent has its fill's edge on a cell boundary instead of at 37 percent of its track's pixels; an indeterminate bar sends no new picture for a frame while it glides, or glides under `reduced-motion`; or a track cell holds `═`, `░` or `█`.
Mechanism: pixel-looks
Rationale: gpui-kit 0.7.0's slider (a thin track and a round thumb) and progress bar (a rounded track with a smooth fill); a cell can show a value only in steps of its width.
Status: Agreed 2026-10-04

[PIX-006] On the Linux development host, in a release build on the hardware adapter, with Kitty graphics through shared memory and with Sixel, each written to memory, the catalog's Input page at 240 by 60 cells and a page of 48 cards in a grid of 8 by 6, each card holding a text input, a checkbox and a button (192 looks), MUST each have every look's first picture sent within one second of the first frame; over the following 60 frames in which a key is typed into a focused text input of the page every frame, and on the Input page 60 more in which the pointer drags the slider's thumb one cell every frame, the App's work per frame (BAR-005's measure) and its wait in `present` (GFX-009's) MUST stay under 16.6 ms at the 95th percentile, and each such frame MUST send the pictures of the changed looks alone; the same runs on the Windows test tablet and the macOS test host are measured and their numbers recorded, and no bound binds there.
Falsifier: On the Linux development host in release on the hardware adapter, with Kitty through shared memory or with Sixel written to memory, a look of either page has sent no picture one second after the first frame; the 95th percentile of the App's work per frame or of its wait in `present` over the 60 typing or the 60 dragging frames exceeds 16.6 ms; or a typing frame sends a picture for a look other than the text input, or a dragging frame for one other than the slider.
Mechanism: pixel-looks
Rationale: GFX-011 measured fifteen canvases on one thread at 49 pictures a second each; the developer asked for speed measured on the tablet at 240 columns before every widget follows, and ruled on 2026-10-01 that speed bounds bind on the Linux host and the tablet's numbers are recorded.
Status: Agreed 2026-10-04
