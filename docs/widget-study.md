# Widget study: gpui-kit 0.6.6 and reactive-tui

Date: 2026-09-26. Read at reactive-tui d664fadc and gpui-kit 0.6.6.

On 2026-09-26 the developer asked for a comparison, not parity: how
gpui-kit builds its widgets and what they can do, set against ours, and a
shortlist of widgets we might add. This study answers that. It changes no
code. The developer chooses what to build; each choice is its own
commitment.

## How to read this

- Paths such as src/widgets/input/slider.rs:514-529 are in this
  repository.
- `G/` is gpui-kit's component crate,
  /home/shawn/workspace2/gpui-kit-0.6.6/crates/component/src. `B/` is its
  base crate, /home/shawn/workspace2/gpui-kit-0.6.6/crates/base/src. Many
  gpui-kit components are a styled layer over a headless part in `B/`,
  which holds the keyboard handling and the screen-reader roles, so the
  comparisons follow both.
- Every claim cites a file and a line range on each side it is about. "None
  found" names the search that was run.
- Sizes are rough: small is a few hundred lines with tests, medium is a new
  widget of the size of our select, large is several widgets or a new
  subsystem.
- Terminal width: on 2026-09-26 the developer said to expect 240 to 512
  columns and more. That is lower than the 500 to 700 that the spec
  overview (docs/spec/overview.md:25) and the recon (docs/recon.md:398-401)
  record; the overview's wording goes to the developer after this
  commitment. The per-screen figures are estimates from an assumed cell
  width of 0.6 of the font size at 8 to 10 pt, not measured: about 245 to
  305 columns on a MacBook Air, 320 to 400 on a 2560 x 1440 monitor, 430 to
  535 on a 3440 x 1440 one. Findings about content that does not fit matter
  mostly for split panes and side panels at those widths.
- The study was read by eight read-only reviewers, one per group of
  families, and assembled and spot-checked by the builder. Each citation
  was checked to name a file that exists and lines inside it.

The body has four parts: the twenty families both libraries have, the
gpui-kit components we lack with a build or skip call for each, the
modules that are support code rather than widgets, and a coverage list of
all 80 modules of gpui-kit's component crate.

## Summary

### What to change in the widgets we have

These recur across many families, so each one fixes several widgets at
once. They are listed in the order we recommend. One defect comes before
all of them: the default backend never turns on mouse reporting (see
"Defects found" below), so every mouse feature in this study, ours or
proposed, works in a real terminal only once that is fixed.

1. Change callbacks on the builders. `builder::radio_button()` has no way
   to report a change (src/builder/specialized.rs:246-296), and
   `builder::text_input()` builds its input with no change or submit
   callback (src/builder/widgets/input.rs:230-232). The checkbox, tabs and
   accordion callbacks exist only on the component or as a props field, so
   a developer needs `Element::typed_with` or hand-filled props
   (src/widgets/input/checkbox.rs:105-108;
   src/widgets/layout/tabs.rs:443-453;
   src/widgets/layout/accordion.rs:157-159). gpui-kit puts `on_change` on
   the builder (G/checkbox.rs:96-105; G/radio.rs:325-334;
   G/tab/tab_bar.rs:168-177; G/accordion.rs:72-81). Small for each widget.
2. Screen-reader actions beyond Focus and Click. The App drops every other
   request (src/app.rs:411-417), so a screen-reader user can hear a
   slider's value or a field's text but cannot set it. gpui-kit handles
   SetValue on inputs (G/input/input.rs:454-464) and Increment and
   Decrement on sliders (B/slider.rs:489-508). Medium, because it changes
   how the App dispatches accessibility actions.
3. Names, positions and counts for screen readers. gpui-kit lets a
   checkbox, radio, select or progress bar carry a spoken name apart from
   its visible text (G/checkbox.rs:77-83; G/radio.rs:75-82;
   G/select.rs:675-682; G/progress/progress.rs:61-65). Our select's node
   has a value and no name (src/widgets/input/select.rs:267-278), and an
   unlabelled progress bar is read as "Progress"
   (src/widgets/display/progress_bar/live.rs:186-195). gpui-kit also sets
   position in set on radios and tabs (B/radio.rs:213-219;
   B/tabs.rs:75-81) and row and column counts on tables
   (B/table.rs:91-110), which our tabs, tree and table do not
   (src/widgets/layout/tabs.rs:698-711;
   src/widgets/display/tree/live/paint.rs:210-221;
   src/widgets/display/table/live.rs:467-471). Small to medium.
4. The theme follows the terminal. Only the DirectTty backend asks the
   terminal for its background color (src/platform/mod.rs:1182-1203,
   reached from src/backend/direct_tty.rs:46), and its reply parser reads
   only the DA1, DA2, Kitty graphics and DECRQM answers
   (src/platform/mod.rs:1277-1301). The default backend, which the widget
   catalog and the other examples use (examples/widget_catalog/main.rs:41),
   sends no query at all (src/backend/suprtui.rs:157;
   src/backend/suprtui/output.rs:171). So a light terminal always gets the
   dark theme. gpui-kit switches light and dark from the window appearance
   (G/theme/mod.rs:227-237), loads themes from files
   (G/theme/registry.rs:151-161), and pairs each fill color with a text
   color (B/theme_tokens.rs:19-45). Several of our widgets fix their colors
   instead of using theme roles: the progress bar
   (src/widgets/display/progress_bar.rs:233-234) and the table's selected
   row (src/widgets/display/table.rs:166-167). Theme files and the color
   roles are small each. Following the terminal's background is medium: the
   default backend has to send the query and read the reply first, which
   fits the input-protocols work on the roadmap.
5. Content that does not fit. Tabs that do not fit are clipped with no
   scrolling or overflow menu (src/widgets/layout/tabs.rs:778-784), where
   gpui-kit scrolls the tab row and lists every tab in a menu
   (G/tab/tab_bar.rs:109-130, 535-585). Breadcrumb segments hidden by
   overflow cannot be reached by keyboard
   (src/widgets/layout/breadcrumb.rs:256-265). Medium.
6. Copy through the terminal (OSC 52). The text input copies into a
   private buffer (src/widgets/input/text_input.rs:674-704), and the
   clipboard hook runs local commands such as wl-copy and pbcopy
   (src/hooks/clipboard.rs:11-21), which over SSH reach the remote host,
   not the user's clipboard. OSC 52 asks the terminal itself to set the
   clipboard. Small.
7. Named key actions. gpui-kit defines shared actions (Confirm, Cancel,
   SelectUp, SelectPageDown and others) that the app's keymap binds to keys
   (B/actions.rs:6-28), and its menus read an item's shortcut hint from the
   real binding (G/menu/popup_menu.rs:1119-1150). Each of our widgets
   matches key codes in its own handler
   (src/widgets/layout/tabs.rs:876-878;
   src/widgets/input/select.rs:670-672), and a menu's shortcut hint is text
   kept apart from its keys (src/widgets/menu/item.rs:5-10), so apps cannot
   rebind keys and hints can drift. Large, since it touches every widget.

Each family section below lists smaller items worth adopting for that
widget alone.

### Widgets to build

Recommended order: the small shared pieces first, because the larger
widgets use them.

| Widget | In a terminal | Size |
|---|---|---|
| Icon catalog (`icon`) | Named glyphs with an ASCII fallback, replacing each widget's own strings | small |
| Spinner (`spinner`) | One cell cycling braille or `\|/-\` frames, static under reduced motion | small |
| Separator (`separator`) | A `─` row or `│` column, optionally with a label | small |
| Badge and tag (`badge`, `tag`) | ` 3 ` or ` done ` on a variant color; one widget, lifted from the tab badge | small |
| Key hint (`kbd`) | `Ctrl+S` formatted from our key types, for menus, palette and help | small |
| Empty state (`empty`) | A centered title, description and action for empty lists and trees | small |
| Skeleton and shimmer (`skeleton`, `shimmer`) | Pulsing placeholder rows; a bright band moving over busy text | small |
| Status bar (`status_bar`) | The last row with left, center and right regions | small |
| Description list (`description_list`) | Label and value columns with the screen-reader term roles | small |
| Form layout (`form`) | Aligned labels, required marks and descriptions, each label linked to its control | small |
| Inline alert (`alert`) | A bar-and-icon message block in the page flow, reusing the toast colors and roles | small |
| Link (`link`) | A focusable underlined link, carried as OSC 8 where the terminal supports it | small |
| Pagination (`pagination`) | `‹ 1 … 4 [5] 6 … 20 ›`, moved out of the data table | small |
| Stepper (`stepper`) | `✓ Account ── ● Profile ── ○ Confirm`, for the wizard | small |
| Number input (`input`) | `[-] 12.5 [+]` with step, min and max, as a text input mode | small |
| Code input (`input`, OTP) | `[1][2][3] [_][_][_]` for one-time codes, pasting fills every cell | small |
| Rating (`rating`) | `★★★☆☆`, a slider with a star painter | small |
| Switch (`switch`) | `[ ●]` on, `[● ]` off, a checkbox variant with the Switch role | small |
| Resizable split (`resizable`) | Panes with a draggable, focusable `│` divider | medium |
| Sidebar (`sidebar`) | A navigation column that collapses to icons | medium |
| List (`list`) | One shared windowed list with filter, sections and load more | medium |
| Command palette (`command`) | A query line over grouped commands with their shortcuts | medium |
| Log view (`message_scroller`) | A scroll area that follows new rows until the user scrolls up | medium |
| Markdown view (`text`) | A scrolling view over our Markdown renderer with links and appending | medium |
| Calendar and date picker (`time`) | A month grid with arrow-key navigation, and an input that opens it | medium |
| Code editor (`input`) | Our syntax editor as a widget with undo, search, folding, then completion | large |

Skip: attachment, avatar, bubble, carousel, the clipboard button (fix the
hook instead, item 6 above), color picker, dock (the resizable split
covers the common need), input group (prefix and suffix cells on the text
input cover it), marker, message, native menu and the settings screen.
Each entry below says why.

Already covered under another name, compared in their entries: button,
collapsible (our accordion), combobox (our select and autocomplete
dialog), group box (our card), hover card (our popover), label, sheet (our
modal), tooltip (our popover and per-widget
tooltips) and the input mask (our input dialog's mask).

### Where we are ahead

gpui-kit is a desktop kit and leans on the pointer. Several of its widgets
have little or no keyboard handling:
- Tabs: its base file lists keyboard support as a TODO (B/tabs.rs:15-21).
- Radios: Enter and Space activate one radio, but there are no arrow keys.
  A search of G/radio.rs, B/radio.rs and B/radio_group.rs for
  `key_context`, `on_action` and `KeyBinding` finds none.
- Sliders: a search of G/slider.rs and B/slider.rs for key and focus
  handling (`key_context`, `KeyBinding`, `on_key`, `track_focus`,
  `focus_handle`, `on_action`) finds none; they work by pointer only
  (B/slider.rs:583-588, 613-618).
- Charts: a search of G/chart and G/plot for roles and key handling finds
  none; hover is by mouse (G/plot/mod.rs:57-72).

Ours handle the keyboard in all four (src/widgets/layout/tabs.rs:865-901;
src/widgets/input/radio_button.rs:349-365;
src/widgets/input/slider.rs:514-529;
src/widgets/display/charts/live.rs:380-454). We also have dialog kinds it
lacks: a wizard (src/widgets/dialog/wizard.rs:43-56), a validated input
dialog (src/widgets/dialog/input.rs:100-128) and a progress dialog
(src/widgets/dialog/progress.rs:20-33). We also have data-table filters
and multi-column sort (src/widgets/display/data_table/filters.rs:15-43;
src/widgets/display/data_table/live.rs:391-403), and twelve popover
placements with edge flipping (src/widgets/display/popover.rs:12-38,
123-134).

### Defects found

The study changes no code, so these are recorded here, not fixed. They are
captured for later as the next-feature items default-backend-mouse (the
first one) and widget-study-defects (the rest).

- The default backend never turns on mouse reporting, so no mouse event
  reaches a widget in a real terminal. SuprTuiBackend's setup writes only
  the alternate-screen, hidden-cursor and focus-event modes
  (src/backend/suprtui/output.rs:171), CrosstermBackend wraps it
  (src/backend/mod.rs:238-248), and only the DirectTty backend enables the
  mouse (src/backend/direct_tty.rs:69; src/platform/mod.rs:582-587). On
  2026-09-26 the widget catalog, run for three seconds in a pseudo-terminal
  of 240 by 60 cells, wrote only the modes 1049, 25, 1004 and 2026: no
  mouse mode and no background-color query. The widget tests pass because
  they send synthetic mouse events. Found by the adversary review, confirmed
  by that run.

- A Markdown code fence written ```` ```rust ```` is not highlighted. The
  highlighter looks languages up by exact, case-sensitive display name
  (src/syntax/resources.rs:101-108; src/syntax/highlighter.rs:48-50), so
  only ```` ```Rust ```` works (src/markdown/tests.rs:76). The test with a
  lowercase fence checks only for a background color
  (src/markdown/tests.rs:56-71).
- The table sorts every column as text, so "10" sorts before "9"
  (src/widgets/display/table.rs:351-356).
- Table column minimums look like pixel values: 50 cells from
  `TableColumn::new` (src/widgets/display/table.rs:661) and 100 cells from
  the builder (src/builder/widgets/table.rs:61), and the table enforces
  them (src/widgets/display/table/live.rs:75). Three builder columns need
  300 columns, more than a 240-column terminal. The data table's virtual
  scroll defaults are 32 and 400 and are documented as pixels
  (src/widgets/display/data_table.rs:352-358).
- An area chart's fill replaces its stroke color, so the two cannot differ
  (src/widgets/display/charts/typed.rs:345-351), and the typed line chart
  can turn dots on but not off (src/widgets/display/charts/typed.rs:231-234).
- Toasts shown at the same position draw over each other: the engine gives
  them empty bounds (src/widgets/dialog/engine/content.rs:128-131), and
  empty bounds leave the position unchanged
  (src/widgets/dialog/frame.rs:26-33).
- A tree node selected while its parent is collapsed stays hidden, and the
  cursor falls back to the first row
  (src/widgets/display/tree/live.rs:223-229).
- Plain text from the syntax highlighter's fallback is painted black
  (src/syntax/highlighter.rs:322-364), on the editor's near-black
  background (src/editor/syntax_editor.rs:200-205).

## Families both libraries have

The twenty families docs/recon.md section 13 lists, in alphabetical order. The chart entry covers gpui-kit's chart and plot modules together.

### accordion
- Files: ours src/widgets/layout/accordion.rs, src/widgets/layout/accordion/live.rs, src/widgets/layout/accordion/live/motion.rs, src/builder/widgets/accordion.rs; gpui-kit G/accordion.rs, B/accordion.rs, B/collapsible.rs.
- Structure:
  - gpui-kit: `Accordion` and `AccordionItem` are RenderOnce structs with builder methods (G/accordion.rs:17-27, 97-156, 159-174, 278-380). They keep no state between frames. Each item's `open` comes from the caller (G/accordion.rs:208-211), and the root rebuilds its set of open indices on every render (G/accordion.rs:99, 120-122).
  - gpui-kit, base layer: the component wraps unstyled base parts `Accordion`, `AccordionItem`, `AccordionHeader`, `AccordionPanel` and `AccordionTrigger` (B/accordion.rs:15-360). `Collapsible` is a separate base region that shows its content only while open (B/collapsible.rs:12-19, 68-80).
  - ours: `AccordionBuilder` produces a registered component element (src/widgets/layout/accordion.rs:378-431). The `Accordion` component keeps `AccordionState` (src/widgets/layout/accordion.rs:188-201, 208-251) and renders a `LiveAccordion` (src/widgets/layout/accordion.rs:231-236; src/widgets/layout/accordion/live.rs:37-44).
  - ours, live widget: it measures headers and bodies through layout callbacks and animates the body height (src/widgets/layout/accordion/live.rs:163-168, 182-207; src/widgets/layout/accordion/live/motion.rs:124-140).
- Features and options:
  - Both sides: per-item icon, disabled item, custom title content, and a start-open flag (G/accordion.rs:196-216; src/widgets/layout/accordion.rs:99-127).
  - Only ours:
    - Three modes: Single, Multiple and AlwaysOne (src/widgets/layout/accordion.rs:21-31, 279-301, 352-357).
    - Custom expand and collapse glyphs, and a switch to hide them (src/widgets/layout/accordion.rs:145-150, 408-413).
    - A stagger delay between sections that change together (src/widgets/layout/accordion.rs:33-52; src/widgets/layout/accordion/live/motion.rs:78-83).
    - Reduced motion (src/widgets/layout/accordion.rs:160-161, 402-406; src/widgets/layout/accordion/live/motion.rs:85-90).
    - `persist_state` (src/widgets/layout/accordion.rs:155-156, 222-229) and a per-section `aria_label` (src/widgets/layout/accordion.rs:73-74, 129-133).
    - A change event whose JSON carries the section id, the new state and the ordered list of open sections (src/widgets/layout/accordion.rs:157-159, 365-373).
    - Presets `simple_accordion`, `settings_accordion`, `faq_accordion` and `navigation_accordion` (src/builder/widgets/accordion.rs:49-58, 80-139).
  - Only gpui-kit:
    - A `multiple` flag instead of modes (G/accordion.rs:44-48, 131-141).
    - A `bordered` rounded card (G/accordion.rs:50-54, 108-113).
    - Disabling the whole accordion at once (G/accordion.rs:56-60, 128).
    - Sizes (G/accordion.rs:84-89, 280-284, 299-304).
    - Title, hover and content styles (G/accordion.rs:218-238).
    - `keep_mounted` on the base panel, so closed content can be unmounted (B/accordion.rs:232-235, 257-259).
  - Terminal note: the rounded border and the rotating chevron (G/accordion.rs:108-113, 326-331) do not carry over. Ours swaps the glyphs ▼ and ▲ instead (src/widgets/layout/accordion.rs:171-172; src/widgets/layout/accordion/live.rs:121-133). The hover style is mouse-only.
- Builder API:
  - gpui-kit, root: `Accordion::new(id)`, `.multiple`, `.bordered`, `.disabled`, `.item(|item| ..)` and `.on_toggle_click(|open_indices, ..| ..)` (G/accordion.rs:31-81).
  - gpui-kit, items: `AccordionItem` has `.icon`, `.title`, `.open`, `.disabled`, `.title_style`, `.hover` and `.content_style`, and takes children through ParentElement (G/accordion.rs:196-238, 259-263).
  - ours, root: `accordion()` or `AccordionBuilder::new()`, then `.section`, `.mode`, `.animated`, `.icons`, `.class`, `.keyboard_navigation` and `.build()` (src/builder/widgets/accordion.rs:31-33; src/widgets/layout/accordion.rs:382-430).
  - ours, sections: `AccordionSection::new(id, title)` with `.content`, `.icon`, `.expanded`, `.disabled`, `.class`, `.custom_header` and `.aria_label` (src/widgets/layout/accordion.rs:77-134).
  - ours, gap: `on_change`, `show_icons` and `animation` are public fields of `AccordionProps` only. The builder has no method for them (src/widgets/layout/accordion.rs:136-162, 382-431).
- States:
  - gpui-kit: open and disabled per item (G/accordion.rs:170-172), and a spring progress value per item for the reveal (G/accordion.rs:285-291, 354-367). It keeps no focus or hover state; the only focus code passes a handle through (B/accordion.rs:50-53, 336-339).
  - ours, component: a map of expanded sections, the focused section, animation flags, an initialized flag and the last interaction time (src/widgets/layout/accordion.rs:188-201). Disabled is set per section (src/widgets/layout/accordion.rs:65-66).
  - ours, live widget: whether it has focus (src/widgets/layout/accordion/live.rs:42, 258-265) and the transition progress of each section (src/widgets/layout/accordion/live/motion.rs:12-23, 46-107).
- Keyboard and screen reader:
  - ours, keys: Down and Up move the focus and wrap at the ends. Home and End jump. Enter and Space toggle. Key releases are ignored (src/widgets/layout/accordion.rs:321-345). Keys work only while the accordion has focus (src/widgets/layout/accordion/live.rs:266-268). A left click on a header toggles it (src/widgets/layout/accordion/live.rs:269-305).
  - ours, screen reader: each header is a Button with a label, expanded state, click action and disabled flag (src/widgets/layout/accordion/live.rs:139-153). A focus event links the focused header to the tree (src/widgets/layout/accordion/live.rs:154-162, 243-257). Decorations are hidden (src/widgets/layout/accordion/live.rs:95-96). A collapsed body is inert and a hidden Group (src/widgets/layout/accordion/live.rs:208-214).
  - gpui-kit, keys: none found. I searched `key|focus|tab_index|action` in G/accordion.rs, B/accordion.rs and B/collapsible.rs. The only matches are focus pass-throughs (B/accordion.rs:50-53, 336-339).
  - gpui-kit, screen reader: the root is a Group (B/accordion.rs:58-65). The trigger is a Button with aria-expanded (B/accordion.rs:344-360). The header is a Heading with aria-level, default 3 (B/accordion.rs:146-155, 188-203). The panel is a Region (B/accordion.rs:255-270).
- Worth adopting:
  - A Heading role with a level on each header, and a Region role on the open panel (B/accordion.rs:188-203, 255-270). Ours sets only Button and a hidden Group (src/widgets/layout/accordion/live.rs:139-153, 208-214). Screen-reader users could jump by heading and hear where a panel starts. Size: small.
  - An `on_change` method on `AccordionBuilder`, like `on_toggle_click` (G/accordion.rs:72-81). Today a developer must fill in `AccordionProps` by hand to get the change event (src/widgets/layout/accordion.rs:157-159, 382-431). Size: small.
  - A switch that disables the whole accordion (G/accordion.rs:56-60, 128). It saves marking each section during a busy state. Size: small.

### breadcrumb
- Files: ours src/widgets/layout/breadcrumb.rs, src/widgets/layout/breadcrumb/live.rs, src/widgets/layout/breadcrumb/overflow.rs, src/builder/widgets/breadcrumb.rs; gpui-kit G/breadcrumb.rs (no B/ part).
- Structure:
  - gpui-kit: `Breadcrumb` is a RenderOnce struct holding `BreadcrumbItem`s, which are also RenderOnce (G/breadcrumb.rs:11-27, 91-115, 156-177). It keeps no state. The separators are ChevronRight icons (G/breadcrumb.rs:139-148).
  - ours: `BreadcrumbBuilder::build` returns `Element::typed::<Breadcrumb>` (src/widgets/layout/breadcrumb.rs:389-392). The component keeps `BreadcrumbState` (src/widgets/layout/breadcrumb.rs:166-181, 202-222) and renders `LiveBreadcrumb` (src/widgets/layout/breadcrumb.rs:224-229; src/widgets/layout/breadcrumb/live.rs:35-41).
  - ours, live widget: it measures each segment and computes an overflow plan (src/widgets/layout/breadcrumb/live.rs:297-317; src/widgets/layout/breadcrumb/overflow.rs:8-130).
- Features and options:
  - gpui-kit: each item has a label, a disabled flag, a closure `on_click` and a style, and the last item gets the foreground color (G/breadcrumb.rs:19-65, 100-106). Items can be made from `&str`, `String` or `SharedString` (G/breadcrumb.rs:73-89).
  - Only ours:
    - Separator text and a compact mode (src/widgets/layout/breadcrumb.rs:116-117, 137-138; src/widgets/layout/breadcrumb/live.rs:43-49).
    - A maximum width and five overflow strategies: MiddleEllipsis, TruncateStart, TruncateEnd, Scroll and Wrap (src/widgets/layout/breadcrumb.rs:95-121; src/widgets/layout/breadcrumb/overflow.rs:30-122).
    - Icons, and a home icon on the first segment (src/widgets/layout/breadcrumb/live.rs:132-142).
    - The current segment in bold and disabled segments dimmed (src/widgets/layout/breadcrumb/live.rs:203-211).
    - Tooltip text shown as a line under the trail on hover (src/widgets/layout/breadcrumb/live.rs:394-407).
    - A navigation event with segment_id, path and label as JSON (src/widgets/layout/breadcrumb.rs:130-132, 288-301).
    - Presets: simple (src/builder/widgets/breadcrumb.rs:46-56), filesystem (src/builder/widgets/breadcrumb.rs:78-85), website (src/builder/widgets/breadcrumb.rs:104-111), compact (src/builder/widgets/breadcrumb.rs:130-138), application (src/builder/widgets/breadcrumb.rs:162-170) and path (src/builder/widgets/breadcrumb.rs:182).
  - Only gpui-kit: a Rust closure per item instead of a named event (G/breadcrumb.rs:47-53, 107-113).
- Builder API:
  - gpui-kit: `Breadcrumb::new()`, `.child(item)` and `.children(items)` (G/breadcrumb.rs:117-137). `BreadcrumbItem::new(label)`, `.disabled` and `.on_click` (G/breadcrumb.rs:29-53).
  - ours, builder: `breadcrumb()` (src/builder/widgets/breadcrumb.rs:28-30). `BreadcrumbBuilder` has `.segment`, `.separator`, `.max_width`, `.overflow_strategy`, `.show_icons`, `.show_tooltips`, `.class`, `.keyboard_navigation`, `.home_icon`, `.compact`, `.on_click(event_name)`, `.show_home_icon` and `.build` (src/widgets/layout/breadcrumb.rs:309-392).
  - ours, segments: `BreadcrumbSegment::new(id, label, path)` with `.icon`, `.current`, `.clickable`, `.class`, `.tooltip` and `.aria_label` (src/widgets/layout/breadcrumb.rs:42-93).
- States:
  - gpui-kit: disabled and is_last per item. Nothing is kept between frames (G/breadcrumb.rs:25-26, 60-64).
  - ours, component: the focused segment, the visible segments, an overflow flag, the scroll position, the measured widths and an initialized flag (src/widgets/layout/breadcrumb.rs:166-181). Each segment has current and clickable flags (src/widgets/layout/breadcrumb.rs:30-33).
  - ours, live widget: focus and the hovered segment (src/widgets/layout/breadcrumb/live.rs:35-41, 486-493, 525-534).
- Keyboard and screen reader:
  - ours, keys: Left and Right move without wrapping. Home and End jump. Enter and Space activate. Focus skips the current segment and segments that cannot be clicked (src/widgets/layout/breadcrumb.rs:247-286).
  - ours, scrolling and mouse: in Scroll mode the row scrolls to keep the focused segment visible (src/widgets/layout/breadcrumb/live.rs:92-120, 494-498). Click, hover and wheel are handled (src/widgets/layout/breadcrumb/live.rs:513-559).
  - ours, screen reader: the root is Navigation (src/widgets/layout/breadcrumb/live.rs:419). Each segment is a Link with its label or aria_label, a click action, disabled when it cannot be clicked, aria-current=page when current, and the tooltip as its description (src/widgets/layout/breadcrumb/live.rs:183-202). The separators and the "..." are hidden (src/widgets/layout/breadcrumb/live.rs:322-334, 337-343).
  - ours, gap: segments hidden by overflow cannot be reached. Keyboard focus is limited to visible segments (src/widgets/layout/breadcrumb.rs:256-265), and the "..." is plain hidden text (src/widgets/layout/breadcrumb/live.rs:322-334).
  - gpui-kit, keys: none found. I searched `key|focus|action|aria|role` in G/breadcrumb.rs; only the role lines matched.
  - gpui-kit, screen reader: an item is a Link when it has a click handler and is enabled, otherwise a ListItem (G/breadcrumb.rs:95-99). The row itself has no role (G/breadcrumb.rs:171-176).
- Worth adopting:
  - Make the "..." open a menu of the hidden segments, the way gpui-kit's TabBar lists all tabs in an overflow menu (G/tab/tab_bar.rs:109-113, 554-585). In a narrow terminal the middle of a long path cannot be reached today (src/widgets/layout/breadcrumb.rs:256-265). Size: medium.

### chart, plot
- Files: ours src/widgets/display/charts.rs, src/widgets/display/charts/typed.rs, src/widgets/display/charts/live.rs, src/widgets/display/charts/live/canvas.rs, src/widgets/display/charts/live/canvas/cartesian.rs, src/widgets/display/charts/live/canvas/pie.rs, src/widgets/display/charts/live/canvas/sankey.rs, src/widgets/display/charts/live/motion.rs, src/widgets/display/charts/live/worker.rs, src/widgets/display/charts/mask.rs, src/widgets/display/charts/plot/mod.rs, src/widgets/display/charts/plot/tooltip.rs, src/widgets/display/charts/plot/layout.rs, src/widgets/display/charts/plot/decimate.rs, src/builder/widgets/chart.rs, src/builder/macros.rs, docs/spec/charts.md; gpui-kit G/chart/mod.rs, G/chart/line_chart.rs, G/chart/area_chart.rs, G/chart/bar_chart.rs, G/chart/candlestick_chart.rs, G/chart/pie_chart.rs, G/chart/radar_chart.rs, G/chart/sankey_chart.rs, G/plot/mod.rs, G/plot/tooltip.rs, G/plot/shape/stack.rs. B/ has no chart code. It supplies only the `Spring` that the hover pointer uses (G/chart/mod.rs:20, G/chart/mod.rs:39-41).
- Structure:
  - gpui-kit: each chart is a struct that holds `Vec<T>` and `Rc` accessor closures. It derives `IntoPlot` and implements the `Plot` trait (`paint`, `tooltip_state`, `hover`, `tooltip`) (G/chart/line_chart.rs:32-51, G/chart/line_chart.rs:172-331, G/plot/mod.rs:25-111). The hover memory (last datum, cursor, fade) lives in keyed element state (G/plot/tooltip.rs:324-401). Tooltips stay off until `id()` is set (G/chart/line_chart.rs:78-85).
  - ours: typed builders run their closures once at `build()` and store plain `ChartProps` (src/widgets/display/charts/typed.rs:1-13). The `Chart` component shares the props with a `LiveChart` element (src/widgets/display/charts.rs:862-901). `LiveChart` rasterizes on a named `rtui-chart-*` worker thread through one mask canvas (src/widgets/display/charts/live/worker.rs:1-4, src/widgets/display/charts/live/worker.rs:56, src/widgets/display/charts/mask.rs:1-19). The selection is kept in `ChartState` (src/widgets/display/charts.rs:848-859).
- Features and options:
  - Shared: line, area, bar, candlestick, pie, radar and Sankey (G/chart/mod.rs:1-15; src/widgets/display/charts.rs:617-638). Both have natural, linear and step-after curves, `tick_margin`, `grid` and `x_axis`.
  - Only ours:
    - A scatter type and a separate donut type (src/widgets/display/charts.rs:617-638).
    - A legend with a position setting (src/widgets/display/charts.rs:436-470).
    - Axis title, min, max, custom labels and tick count (src/widgets/display/charts.rs:403-420).
    - Value-axis tick labels on every cartesian chart (src/widgets/display/charts/live/canvas/cartesian.rs:121-142).
    - Several series per chart, with grouped or stacked bars and stacked areas (src/widgets/display/charts/typed.rs:469-477, src/widgets/display/charts/typed.rs:508-512, src/widgets/display/charts/live/canvas/cartesian.rs:566-581).
    - Dashed and dotted lines, fill patterns and a shaded gradient fill (src/widgets/display/charts.rs:589-613, src/widgets/display/charts/live/canvas/cartesian.rs:626-677, src/widgets/display/charts/live/canvas/cartesian.rs:742-748).
    - Mini, medium and large size classes (src/widgets/display/charts/plot/layout.rs:86-98).
    - An ASCII fallback (src/widgets/display/charts.rs:16-31, src/widgets/display/charts.rs:258-262).
    - Min/max decimation for long series (src/widgets/display/charts/plot/decimate.rs:14-17).
    - A reveal animation and an animation when data changes (src/widgets/display/charts/live/motion.rs:1-4).
    - A written error or "No data to display" message for bad or empty input (src/widgets/display/charts/live/canvas.rs:252-340, src/widgets/display/charts/live/canvas.rs:404-420).
    - A tooltip that sums up the rows past eight (src/widgets/display/charts/plot/tooltip.rs:11-12, src/widgets/display/charts/plot/tooltip.rs:54-91).
  - Only gpui-kit:
    - Bar fill from a per-datum closure, or a gradient fill (G/chart/bar_chart.rs:131-206).
    - A `value_axis` switch and `value_tick_count` on bars (G/chart/bar_chart.rs:229-250).
    - `corner_radii` on bars (G/chart/bar_chart.rs:265-272). This does not carry over to cells.
    - A curve style per area series (G/chart/area_chart.rs:117-130, G/chart/area_chart.rs:227-230).
    - Pie radius closures per slice, plus label and leader-line colors (G/chart/pie_chart.rs:105-146, G/chart/pie_chart.rs:177-187).
    - A pie tooltip that shows the slice's share in percent (G/chart/pie_chart.rs:447-463).
    - Radar `label_color`, `label_gap` and labels that are elements (G/chart/radar_chart.rs:36-45, G/chart/radar_chart.rs:209-222).
    - Candlestick `body_width_ratio` (G/chart/candlestick_chart.rs:120-123).
    - Hover emphasis: the other bars dim (G/chart/bar_chart.rs:31-32, G/chart/bar_chart.rs:628-644). The hovered pie slice lifts 6 px and the other slices dim (G/chart/pie_chart.rs:25-29, G/chart/pie_chart.rs:412-437). The lift does not carry over to cells.
    - A public `Plot` trait and a `Stack` layout helper for custom charts (G/plot/mod.rs:25-111, G/plot/shape/stack.rs:1-76).
  - Differences in shared parts:
    - Area colors: gpui-kit keeps a stroke and a fill for each area series (G/chart/area_chart.rs:225-239). In ours, `fill` overwrites the series color, so stroke and fill cannot differ, and there is one curve per chart (src/widgets/display/charts/typed.rs:304-351).
    - Line dots: gpui-kit dots are off by default (G/chart/line_chart.rs:66). Ours are on by default (src/widgets/display/charts.rs:836), and `LineChartBuilder::dot()` can only set them on (src/widgets/display/charts/typed.rs:232-235). So the typed line builder cannot turn dots off.
    - Scatter: ours places points by index, not by a numeric x (src/widgets/display/charts/typed.rs:389-393, src/widgets/display/charts/live/canvas/cartesian.rs:606).
    - Sankey with a cycle or a missing node: gpui-kit paints nothing (G/chart/sankey_chart.rs:378, G/chart/sankey_chart.rs:484-501, G/chart/sankey_chart.rs:506-509). Ours writes an error message.
- Builder API:
  - gpui-kit:
    - `LineChart::new(data)` with `x`, `y`, `stroke`, `natural`, `linear`, `step_after`, `dot`, `tick_margin`, `x_axis`, `grid`, `id` and `name` (G/chart/line_chart.rs:58-144).
    - `BarChart::new(data)` with `band`, `value`, `fill`, `fill_gradient`, `label`, `label_axis`, `value_axis` and `alignment` (G/chart/bar_chart.rs:78-272).
    - `PieChart::new(data)` with `value`, `color`, `label`, `inner_radius`, `outer_radius`, `pad_angle` and `label_gap` (G/chart/pie_chart.rs:60-194).
    - `SankeyChart::new(nodes, links)` (G/chart/sankey_chart.rs:141).
    - Colors are `Hsla` or `Background` values.
  - ours:
    - Typed builders use the same names. `LineChartBuilder::new(data)` adds one series per `y` call (src/widgets/display/charts/typed.rs:155-241). All typed builders share `title`, `size`, `size_class`, `x_axis`, `ascii`, `class` and `render` (src/widgets/display/charts/typed.rs:66-109).
    - There are also bar, candlestick, pie, donut, Sankey and radar builders (src/widgets/display/charts/typed.rs:437-526, src/widgets/display/charts/typed.rs:541-638, src/widgets/display/charts/typed.rs:651-738, src/widgets/display/charts/typed.rs:746-856, src/widgets/display/charts/typed.rs:871-1033, src/widgets/display/charts/typed.rs:1047-1148).
    - `ChartsBuilder::line()` and its siblings take `stacked`, `curve`, `dots`, `size_class`, `ascii`, `radial` and `sankey_options`. `build()` returns `ChartProps` and `render()` returns an `Element` (src/widgets/display/charts.rs:69-127, src/widgets/display/charts.rs:130-327).
    - The older `builder::chart()` has shortcuts for bar, horizontal bar, line, pie, area and scatter (src/builder/widgets/chart.rs:50-90). The `chart!` macro wraps it (src/builder/macros.rs:203-237).
    - Colors are string tokens.
- States:
  - gpui-kit: hover only. It tracks the hovered datum, the cursor, a focus value from 0 to 1 that fades in and out, `is_hovered` and `is_entering` (G/plot/tooltip.rs:264-332). Each chart also keeps a sprung pointer position or pie lift (G/chart/line_chart.rs:23-30, G/chart/bar_chart.rs:34-41, G/chart/pie_chart.rs:31-38). It has no focus or selection state.
  - ours: `ChartState` holds reveal progress, an animating flag, `hovered_point` (series, index) shared by mouse and keyboard, and tooltip text (src/widgets/display/charts.rs:848-859). The node is marked busy while the worker draws (src/widgets/display/charts/live.rs:309-316). The chart is focusable as a button (src/widgets/display/charts/live.rs:306).
- Keyboard and screen reader:
  - ours:
    - Left, Right, Home and End move the selection, and Escape clears it. Pie, donut and Sankey charts step through slices or nodes (src/widgets/display/charts/live.rs:380-454).
    - Mouse movement selects the nearest index. A move that keeps the same index changes nothing (src/widgets/display/charts/live.rs:340-379).
    - The node has role Image. Its label holds the title, the series names and the selected value (src/widgets/display/charts/live.rs:307-308, src/widgets/display/charts/live.rs:822-839).
    - The selection is also spoken through an `sr-only aria-live-polite` child (src/widgets/display/charts/live.rs:318-323).
  - gpui-kit: none found. `grep -rn -i "accessib\|a11y\|role\|aria\|announce\|on_key\|key_down\|KeyBinding\|actions!" chart plot` under G/ returned no matches. Hover works by mouse only, through `tooltip_state` (G/plot/mod.rs:57-72).
- Worth adopting:
  - Dim the bars, pie slices and radar series that are not selected, as gpui-kit does for bars and pies (G/chart/bar_chart.rs:628-644, G/chart/pie_chart.rs:25-29). Our Sankey chart already fades unselected links (src/widgets/display/charts/live/canvas/sankey.rs:18-20, src/widgets/display/charts/live/canvas/sankey.rs:248-252), but only Sankey passes the selection to the worker (src/widgets/display/charts/live.rs:205-209). Why: a keyboard selection is easy to see where a one-cell crosshair or ray is thin. Size: medium.
  - Show each slice's share in the pie and donut tooltip and announcement, as "value (share%)" (G/chart/pie_chart.rs:447-463). Ours shows only "label: value" (src/widgets/display/charts/live.rs:759-775, src/widgets/display/charts/live/canvas.rs:593-601). Why: users read pies as parts of a whole. Size: small.
  - Keep a separate stroke and fill for each area series, and allow a curve per series (G/chart/area_chart.rs:225-239). Also add the reference names our typed builders still lack: radar `label_gap` and `label_color`, pie `label_color` and `label_line_color`, candlestick `body_width_ratio` (G/chart/radar_chart.rs:209-222, G/chart/pie_chart.rs:177-187, G/chart/candlestick_chart.rs:120-123), and a way to turn line dots off. Why: fixes the lost stroke color (src/widgets/display/charts/typed.rs:345-351) and keeps the method names comparable. Size: small.
  - A public hook for custom charts, like the `Plot` trait (G/plot/mod.rs:25-111). An app would draw into our mask canvas, supply a hit test, and get our tooltip, keyboard selection and aria-live announcement. We export the plot layer (src/widgets/display/mod.rs:21, src/widgets/display/charts/plot/mod.rs:1-8) but no such trait. Why: new chart kinds without forking the widget. Size: large.

### checkbox
- Files: ours src/widgets/input/checkbox.rs, src/builder/widgets/input.rs; gpui-kit G/checkbox.rs, B/checkbox.rs.
- Structure:
  - gpui-kit: `Checkbox` is a `RenderOnce` struct with builder methods. It wraps `gpui_base::Checkbox` (G/checkbox.rs:15-19, 248-249) and keeps its focus handle in window keyed state (G/checkbox.rs:232-236). The value is controlled: the owner stores it and re-renders after `on_change` (G/checkbox.rs:96-105). The base part owns toggle, focus, keyboard and accessibility, and the app owns all visuals (B/checkbox.rs:41-44).
  - ours: `builder::checkbox()` builds `Element::typed::<Checkbox>` from `CheckboxProps` (src/builder/widgets/input.rs:390-397). `Checkbox` is a live component. It flips `props.checked` itself in its event handler (src/widgets/input/checkbox.rs:228-234) and keeps focus and hover in `CheckboxState` (src/widgets/input/checkbox.rs:83-89). It draws one text string such as `▶ [✓] label` (src/widgets/input/checkbox.rs:127-158).
- Features and options:
  - Both sides have label, checked and disabled (src/widgets/input/checkbox.rs:65-74; G/checkbox.rs:71-89, 133-138).
  - Indeterminate: ours has it in props and in both builders (src/widgets/input/checkbox.rs:41-45, 72-73; src/builder/widgets/input.rs:366-373). In gpui-kit only the base part has it (B/checkbox.rs:16-22, 102-114). The styled `Checkbox` keeps only `checked: bool` (G/checkbox.rs:26, 248-249).
  - Only gpui-kit has these:
    - a screen-reader name that differs from the visible label (G/checkbox.rs:77-83)
    - tooltip (G/checkbox.rs:65-69)
    - sizes (G/checkbox.rs:167-172, 219-224)
    - extra child content under the label (G/checkbox.rs:161-165, 324-326)
    - tab index and tab stop (G/checkbox.rs:107-117)
    - a switch to turn the focus ring off (G/checkbox.rs:140-142, 283-285)
    - role override (G/checkbox.rs:60-63; B/checkbox.rs:82-85)
    - styles per state, and a separate indicator part (B/checkbox.rs:122-126, 191-197, 222-227)
    - a check mark that fades in and out on a spring (G/checkbox.rs:182-191)
  - The spring fade and the hover-only tooltip do not carry over to a terminal.
  - Only ours has these:
    - a CSS class on the builder (src/builder/widgets/input.rs:375-382)
    - a lock glyph when disabled and `_label_` on hover (src/widgets/input/checkbox.rs:131-137, 155-158)
- Builder API:
  - ours:
    - `builder::checkbox()` (src/builder/widgets/input.rs:41-43) has `checked`, `label`, `disabled`, `indeterminate`, `class` and `build() -> Element` (src/builder/widgets/input.rs:339-404).
    - A second `widgets::CheckboxBuilder` has the same four setters, plus `render() -> Element` (src/widgets/input/checkbox.rs:17-60).
    - The change callback `Checkbox::with_on_change` is on the component, not on either builder (src/widgets/input/checkbox.rs:105-108). A test attaches it through `Element::typed_with` and a factory closure (tests/api_widget_behavior.rs:345-355; src/component/element.rs:261-266).
  - gpui-kit:
    - `Checkbox::new(id)` (G/checkbox.rs:39-40) has `role`, `tooltip`, `label`, `accessibility_label`, `checked`, `on_change`/`on_click`, `tab_stop` and `tab_index` (G/checkbox.rs:60-117). Traits add `disabled`, `with_size` and child elements (G/checkbox.rs:133-138, 161-172).
    - The base part adds `state`, `indeterminate`, `styles`, `on_change(CheckboxState, &ClickEvent, ..)` and `track_focus` (B/checkbox.rs:87-163).
- States:
  - ours: props `checked`, `indeterminate`, `disabled` (src/widgets/input/checkbox.rs:65-74); state `is_focused`, `is_hover` (src/widgets/input/checkbox.rs:83-89).
  - gpui-kit: Unchecked, Checked or Indeterminate (B/checkbox.rs:16-22), disabled, and focus read from the focus handle (G/checkbox.rs:232-236). Hover and active are style refinements, not stored state (B/checkbox.rs:571-573).
  - On both sides, activating an indeterminate box makes it checked (src/widgets/input/checkbox.rs:231-234; B/checkbox.rs:33-38).
- Keyboard and screen reader:
  - ours:
    - Space or Enter toggles when focused (src/widgets/input/checkbox.rs:199-203, 228-229).
    - A left mouse Down or Click toggles and also takes focus (src/widgets/input/checkbox.rs:257-262).
    - Screen reader: role CheckBox, name from the visible label, toggled True/False/Mixed, and either disabled or a Click action (src/widgets/input/checkbox.rs:164-180).
  - gpui-kit:
    - Enter and Space each fire `on_change` on the focused element (B/checkbox.rs:524-526, test).
    - Tab reaches the box unless tab stop is off (B/checkbox.rs:381-387).
    - A pointer press does not move focus (G/checkbox.rs:349-353).
    - Disabled drops both focus tracking and the click handler (B/checkbox.rs:381-395).
    - Screen reader: role CheckBox (can be overridden), toggled state and label (B/checkbox.rs:374-380). The Click action is offered only when enabled (B/checkbox.rs:702-703).
- Worth adopting:
  - An `accessibility_label` setter separate from the visible label; a short visible "Agree" can be read as "Agree to the terms", and today we always read the label (src/widgets/input/checkbox.rs:166-168); small.
  - An `on_change` method on the builder; today a callback needs `Element::typed_with` and a factory, which is hard to find; small.
  - An optional line of help text under the label, also set as the node description (src/accessibility/mod.rs:33-36); it gives both sighted and screen-reader users the extra context; small.

### dialog
- Files: ours src/widgets/dialog/mod.rs, src/widgets/dialog/engine.rs, src/widgets/dialog/engine/live.rs, src/widgets/dialog/engine/content.rs, src/widgets/dialog/frame.rs, src/widgets/dialog/confirmation.rs, src/widgets/dialog/confirmation/live.rs, src/widgets/dialog/input.rs, src/widgets/dialog/input/live.rs, src/widgets/dialog/wizard.rs, src/widgets/dialog/progress.rs, src/widgets/display/modal.rs, src/widgets/display/modal/live.rs, src/widgets/display/modal/live/events.rs, src/widgets/display/modal/live/render.rs, src/builder/widgets/dialog.rs, src/builder/dialog_builders.rs, src/builder/specialized.rs; gpui-kit G/dialog/dialog.rs, G/dialog/alert_dialog.rs, G/dialog/footer.rs, G/root.rs, G/window_ext.rs, B/dialog.rs, B/alert_dialog.rs, B/focus_trap.rs.
- Structure:
  - gpui-kit: `Dialog` is a RenderOnce struct with builder methods (G/dialog/dialog.rs:256-275, 506-731). It wraps the unstyled base host `gpui_base::Dialog`, which owns focus, key actions and dismissal (B/dialog.rs:122-143). Open dialogs are a stack in `Root.active_dialogs`, which `window.open_dialog` pushes onto (G/root.rs:297-323; G/window_ext.rs:29-32).
  - Ours has two layers. `Modal` is a component configured by filling in the `ModalProps` struct (src/widgets/display/modal.rs:8-68). A live runtime keeps its visibility, drag and animation data behind a Mutex (src/widgets/display/modal/live.rs:102-123). The typed dialogs are opened through a shared `DialogEngine` that holds sessions and renders one host element (src/widgets/dialog/engine.rs:317-322, 459-463, 550-585). Each typed dialog builds `ModalProps` and hands them to `Modal` (src/widgets/dialog/confirmation/live.rs:153-195).
- Features and options:
  - Both sides:
    - OK/Cancel callbacks that can veto the close (G/dialog/dialog.rs:84-104; ours src/widgets/dialog/confirmation.rs:21, src/widgets/dialog/confirmation/live.rs:55-61).
    - A close button (G/dialog/dialog.rs:388-392, 685-700; ours src/widgets/display/modal/live/render.rs:350-369).
    - A switch for closing on a backdrop click (G/dialog/dialog.rs:430-436; ours src/widgets/display/modal.rs:24-25, src/widgets/display/modal/live/render.rs:160-163).
    - A switch for closing with Escape (G/dialog/dialog.rs:438-442; ours src/widgets/display/modal/live/events.rs:25-31).
    - Several open dialogs stacked (G/root.rs:313-318; ours src/widgets/dialog/engine.rs:498-501, src/widgets/dialog/engine/live.rs:105-114).
    - The size is clamped to the window (G/dialog/dialog.rs:524-535; ours src/widgets/display/modal/live/render.rs:119-127).
  - Only gpui-kit:
    - A `.trigger()` element that opens the dialog when clicked (G/dialog/dialog.rs:305-311, 472-503).
    - An `AlertDialog` preset: no backdrop close, optional icon, centered footer (G/dialog/alert_dialog.rs:14-22, 74-91, 217-221).
    - `DialogChangeReason` tells the caller why the open state changed (B/dialog.rs:25-32).
    - A controlled `DialogHandle` (B/dialog.rs:34-72).
    - Each stacked dialog is drawn 16px lower than the one below (G/dialog/dialog.rs:529).
    - A slide, fade and shadow animation (G/dialog/dialog.rs:701-728). This is pixel animation and does not carry over to a terminal.
  - Only ours:
    - Confirmation dialogs with preset button sets (src/widgets/dialog/confirmation.rs:132-147), single-key shortcuts such as y, n and r (src/widgets/dialog/confirmation.rs:162-163, 348-356, 403, 417, 431) and text icons (src/widgets/dialog/confirmation/live.rs:130-138).
    - Input dialogs with validation and input masks (src/widgets/dialog/input.rs:27-58, 100-120).
    - Wizard dialogs (src/widgets/dialog/wizard.rs:43-56) and progress dialogs (src/widgets/dialog/progress.rs:20-33).
    - Async completion receivers and an event stream (src/widgets/dialog/engine.rs:82-111, 396-414).
    - Live updates to an open dialog (src/widgets/dialog/engine.rs:69-80, 464-480).
    - Drag and resize with the mouse (src/widgets/display/modal.rs:48-51, src/widgets/display/modal/live/events.rs:35-151).
    - Corner positions and positions anchored to another element (src/widgets/dialog/frame.rs:80-137).
    - Auto size to the content (src/widgets/display/modal/live/render.rs:79-114). gpui-kit instead uses a fixed default width of 448px (G/dialog/dialog.rs:177).
    - Custom animation callbacks (src/widgets/dialog/engine.rs:423-445).
    - Opening returns an error type instead of failing silently (src/widgets/dialog/engine.rs:22-67).
    - A dimmed backdrop stands in for blur (src/widgets/dialog/mod.rs:70-71).
- Builder API:
  - gpui-kit:
    - `window.open_dialog(cx, |dialog, window, cx| ...)` (G/window_ext.rs:29-32) and `Dialog::new(cx)` (G/dialog/dialog.rs:286-303).
    - `Dialog` methods: `.trigger`, `.content`, `.title`, `.footer`, `.button_props`, `.on_close`, `.on_ok`, `.on_cancel`, `.close_button`, `.width`, `.overlay`, `.overlay_closable`, `.keyboard` (G/dialog/dialog.rs:305-442).
    - `DialogButtonProps` methods: `ok_text`, `cancel_text`, `show_cancel` and others (G/dialog/dialog.rs:53-104).
    - `AlertDialog::new(cx).confirm().icon().title().description()` (G/dialog/alert_dialog.rs:79-181).
    - Closing: `close_dialog` and `close_all_dialogs` (G/window_ext.rs:55-62).
  - Ours:
    - `builder::modal().title().content().visible().closable().backdrop_dismissible().size().class().build()` (src/builder/widgets/dialog.rs:25-27, 103-215), or `ModalProps` passed to `Modal::with_props` (src/widgets/display/modal.rs:350-353).
    - `confirmation_dialog().title().message().confirm_text().cancel_text().danger().build()` (src/builder/specialized.rs:777-857).
    - `ProgressDialogBuilder` (src/builder/dialog_builders.rs:28-99) and `WizardBuilder` (src/builder/dialog_builders.rs:111-118).
    - Engine: `DialogEngine::new()` and `show_*`/`try_show_*` for confirmation, input, autocomplete, progress, toast and wizard (src/widgets/dialog/engine.rs:351-354, 550-585), plus `close_dialog`, `close_all`, `render` and `update` (src/widgets/dialog/engine.rs:450-480).
- States:
  - gpui-kit: open (B/dialog.rs:47-49, 519-526); layer index and topmost (B/dialog.rs:486-491); the previously focused element, restored on close (G/root.rs:301-318, 325-339).
  - Ours: `ModalState` tracks focused, scroll, the animation phase (Hidden, Showing, Visible, Hiding), position, size, dragging, resizing and the focused button (src/widgets/display/modal.rs:248-288). Buttons have disabled and autofocus (src/widgets/display/modal.rs:143-156). The engine tracks active sessions, closing ("retiring") sessions, order and priority (src/widgets/dialog/engine.rs:113-161).
- Keyboard and screen reader:
  - gpui-kit keys:
    - Escape sends Cancel and Enter sends Confirm inside the Dialog key context (B/dialog.rs:89-94, 554-589).
    - Tab and Shift-Tab stay inside the dialog through a focus trap (B/dialog.rs:553; B/focus_trap.rs:21-24).
    - Closing restores the earlier focus (G/root.rs:325-339).
  - gpui-kit screen reader:
    - Roles are Dialog (B/dialog.rs:421) and AlertDialog (B/alert_dialog.rs:189-195).
    - The close button is named "Close" (B/dialog.rs:355-359).
    - I found no code that names the dialog from its title (searched B/dialog.rs, B/alert_dialog.rs and G/dialog/*.rs for "label", "labelled" and "aria").
  - Ours keys:
    - Escape closes when the dialog is closable (src/widgets/display/modal/live/events.rs:10-34).
    - `FocusProps::modal()` gives a focus trap with auto focus and focus restore (src/widgets/display/modal/live/render.rs:282-287; src/component/focus.rs:93-100).
    - Buttons are focus stops (src/widgets/display/modal/live/render.rs:411-415), and the default button gets autofocus (src/widgets/dialog/confirmation/live.rs:117).
    - Confirmation letter shortcuts are caught before normal routing, while Enter, Escape and Tab pass through to it (src/widgets/dialog/confirmation/live.rs:198-225).
  - Ours screen reader:
    - The dialog node has the Dialog role and takes its label from the title (src/widgets/display/modal/live/render.rs:273-280).
    - The close button is labeled "Close" (src/widgets/display/modal/live/render.rs:366-368).
    - Input validation messages use the Alert and Status roles (src/widgets/dialog/input/live.rs:340-355).
    - The confirmation dialog uses Role::Dialog, not AlertDialog (src/widgets/dialog/confirmation/live.rs:191-195).
- Worth adopting:
  - Give confirmation dialogs the AlertDialog role (gpui-kit: B/alert_dialog.rs:189-195). This tells a screen reader that the dialog needs an answer now. Our Role already maps AlertDialog to a platform role (src/accessibility/platform/translation/node.rs:94), and the file explorer already uses it (src/widgets/display/file_explorer/live/paint.rs:271). Size: small.
  - Move each stacked dialog one row down, as gpui-kit steps 16px per layer (G/dialog/dialog.rs:529). The user can then see that another dialog is waiting underneath; ours stacks by z-index only (src/widgets/dialog/engine/live.rs:105-114). Size: small.

### highlighter
- **Files:** ours src/syntax/mod.rs, src/syntax/highlighter.rs, src/syntax/resources.rs, src/syntax/theme.rs, src/syntax/cache.rs, used from src/markdown/ast_walker.rs, src/markdown/renderer.rs, src/editor/syntax_editor.rs; gpui-kit G/highlighter/mod.rs, G/highlighter/highlighter.rs, G/highlighter/registry.rs, G/highlighter/languages.rs, G/highlighter/language_name.rs, G/highlighter/input_adapter.rs, G/highlighter/diagnostic_styles.rs, G/highlighter/wasm_stub.rs, G/highlighter/languages/*/highlights.scm and injections.scm, used from G/text/mod.rs.
- **Structure:**
  - gpui-kit:
    - `SyntaxHighlighter` owns a tree-sitter parser, the last tree, the text as a rope, and parsed layers for embedded languages (G/highlighter/highlighter.rs:29-68).
    - `update(edit, text, timeout)` reparses from the old tree (G/highlighter/highlighter.rs:521-600). `styles(range, theme)` returns byte ranges with styles (G/highlighter/highlighter.rs:1061-1110).
    - Grammars and queries live in a `LanguageRegistry` singleton (G/highlighter/registry.rs:59-107, 496-540).
    - An adapter plugs it into the text input (G/highlighter/input_adapter.rs:22-157). Code blocks in text views use a per-language cache (G/text/mod.rs:76-112). Without the tree-sitter feature, a stub is used (G/highlighter/mod.rs:11-40).
  - Ours:
    - `SyntaxHighlighter` wraps Lumis (tree-sitter) with a Language and a per-line cache (src/syntax/highlighter.rs:1-5, 39-69). It returns `HighlightedLine`s made of `StyledRun`s (src/syntax/highlighter.rs:19-26, 264-320).
    - The active theme and custom themes live in a global `SYNTAX_RESOURCES` (src/syntax/resources.rs:14-36).
    - Markdown calls it once per code block (src/markdown/ast_walker.rs:158-180). `SyntaxEditor` calls it per visible line, with a hash cache (src/editor/syntax_editor.rs:182-240; src/syntax/cache.rs:7-42).
- **Features and options:**
  - **Languages:**
    - gpui-kit has Json and Plain plus grammars from Astro to Zig, each behind a feature flag (G/highlighter/languages.rs:5-81). Apps can register more, including parsers loaded at runtime (G/highlighter/registry.rs:524-540).
    - Ours compiles in 15 Lumis grammars (Cargo.toml:47) and cannot load more at runtime (src/syntax/resources.rs:1-5).
  - **Name lookup:**
    - gpui-kit maps aliases such as js, py, rs, sh and yml (G/highlighter/language_name.rs:5-37; G/highlighter/languages.rs:178-185; G/highlighter/registry.rs:606-613).
    - Ours matches the display name exactly, with case, or matches file globs (src/syntax/resources.rs:82-108, 128-150). The markdown test writes ```` ```Rust ```` (src/markdown/tests.rs:76). Lumis's crate docs name the language "Rust", so a common ```` ```rust ```` fence falls back to plain code (src/markdown/ast_walker.rs:162-179).
  - **Only gpui-kit:**
    - Embedded languages with size and time limits (G/highlighter/highlighter.rs:21-27, 60-68).
    - Fold ranges built from the tree (G/highlighter/input_adapter.rs:167-228).
    - Diagnostic colors for each severity (G/highlighter/diagnostic_styles.rs:6-34).
    - A search of our src/syntax, src/markdown and src/editor for "inject" and "fold" found nothing.
  - **Themes:**
    - gpui-kit's highlight theme is JSON compatible with Zed. Besides syntax colors, it carries editor background, line-number, active-line and status colors (G/highlighter/registry.rs:436-470).
    - It is part of each app theme config, so switching light or dark swaps it too (G/theme/schema.rs:76-81, 1060-1073).
    - A dotted name falls back to its prefix, so `keyword.modifier` uses `keyword` (G/highlighter/registry.rs:292-305).
    - Ours loads Lumis JSON theme files and defaults to "onedark" (src/syntax/resources.rs:14-15, 39-80, 110-119). `Theme::set_active` does not change it (src/theme/mod.rs:97-106). It can copy a syntax theme into `--syntax-*` app variables (src/syntax/theme.rs:12-125, 297-318).
  - **Terminal output:**
    - Ours maps foreground, bold, italic and underline to cell attributes (src/syntax/highlighter.rs:302-320). Every run gets a transparent background (src/syntax/highlighter.rs:286-291).
    - Plain fallback lines are painted black (src/syntax/highlighter.rs:173-182, 322-364). The editor paints a near-black background (src/editor/syntax_editor.rs:200-205), so this text is hard to read there.
    - gpui-kit returns a default style, so the caller's text color applies (G/highlighter/highlighter.rs:1096-1099).
  - **Limits:**
    - Ours rejects input over 1 MiB (src/syntax/highlighter.rs:16-17, 82-87).
    - gpui-kit keeps the old tree when a parse times out (G/highlighter/highlighter.rs:589-594). It parses large text in the background (G/highlighter/input_adapter.rs:68-81).
- **Builder API:**
  - gpui-kit:
    - `SyntaxHighlighter::new(lang)`, `update(edit, &rope, timeout)`, `edit_tree`, `styles(&range, &theme)` (G/highlighter/highlighter.rs:332-345, 502-534, 1061-1065).
    - `LanguageRegistry::singleton().register(name, &LanguageConfig::new(..))` and `register_parser_factory` (G/highlighter/registry.rs:73-89, 512-540).
    - `HighlightTheme::default_dark()` and `default_light()` (G/highlighter/registry.rs:480-487).
  - Ours:
    - `SyntaxHighlighter::new(name)` and `from_extension(ext)` (src/syntax/highlighter.rs:46-69).
    - `highlight_text`, `try_highlight_text`, `highlight_lines(text, start, end)`, `rehighlight_line`, `invalidate_range`, `clear_cache` (src/syntax/highlighter.rs:71-126, 188-206).
    - `SYNTAX_RESOURCES` with `load_theme_from_file` and `set_active_theme` (src/syntax/resources.rs:43-58).
    - `create_syntax_theme` (src/syntax/theme.rs:298).
    - `MarkdownRenderer::with_syntax_highlighting` (src/markdown/renderer.rs:56-60).
- **States:**
  - gpui-kit:
    - The text rope, the tree and the embedded-language layers (G/highlighter/highlighter.rs:49-57).
    - A pending background parse task (G/highlighter/input_adapter.rs:30-33, 157).
    - A stale tree kept after a timeout (G/highlighter/highlighter.rs:589-594).
  - Ours:
    - `cached_lines` (src/syntax/highlighter.rs:43).
    - A `LineCache` keyed by line with a content hash and a time stamp; the default is 1000 entries and 60 seconds (src/syntax/cache.rs:7-42).
    - The name of the active theme (src/syntax/resources.rs:22-27).
- **Keyboard and screen reader:** none found on either side. The highlighter is a service, not a widget. Searches: `Role::|aria_|accessib` in G/highlighter; `Role::|accessib|KeyCode` in src/syntax. Both found nothing.
- **Worth adopting:**
  - **Language aliases and case-insensitive names** (G/highlighter/language_name.rs:5-37; G/highlighter/languages.rs:178-185); Markdown fences such as ```` ```rust ````, ```` ```py ```` and ```` ```sh ```` would then highlight; small.
  - **Keeping the parse tree and reparsing with edits** (G/highlighter/highlighter.rs:502-600); ours re-highlights the whole text on each `highlight_lines` call (src/syntax/highlighter.rs:152-162), and the editor highlights a changed line on its own (src/editor/syntax_editor.rs:226-233; src/syntax/highlighter.rs:205-245), which loses block comments and multi-line strings; medium, or large if Lumis does not expose its tree (I did not check).
  - **A parse time budget with a background reparse** (G/highlighter/input_adapter.rs:68-157; G/highlighter/highlighter.rs:560-594); typing stays responsive on big files instead of stopping at a hard 1 MiB limit (src/syntax/highlighter.rs:16-17, 82-87); medium.
  - **Tying the syntax theme to the app theme's light or dark mode, and using the theme's text color for plain text** (G/theme/schema.rs:1060-1073; G/highlighter/highlighter.rs:1096-1099); today a light app theme still gets "onedark", and fallback text is black (src/syntax/resources.rs:14-15; src/syntax/highlighter.rs:322-364); small.

### input (text)
- Files: ours src/widgets/input/text_input.rs, src/widgets/input/text_input/accessibility.rs, src/widgets/input/text_input/paint.rs, src/builder/widgets/input.rs, src/widgets/dialog/input.rs, src/widgets/dialog/input/validation.rs, src/widgets/dialog/input/live.rs, src/widgets/dialog/input/live/remote.rs, src/accessibility/text.rs. gpui-kit G/input/input.rs, G/input/textarea.rs, G/input/clear_button.rs, B/input/input/mod.rs, B/input/textarea/mod.rs, B/input/base/state.rs, B/input/base/kind.rs, B/input/base/undo_manager.rs, B/input/base/selection.rs, B/text_boundary.rs. G/input/mod.rs:1-51 declares these modules: input, textarea, editor, number_input, otp_input, group, search, popovers, overlay, content_type, clear_button, state, language_config and syntax_context.
- Structure: gpui-kit's `Input` is a RenderOnce struct with builder methods (G/input/input.rs:109-141). It is bound to an `Entity<InputState>` (G/input/input.rs:180-182). `InputState` is the shared editing engine in single-line mode (B/input/input/mod.rs:10). `Textarea` and `Editor` are the same engine in other modes (B/input/textarea/mod.rs:10; B/input/base/kind.rs:39-46), and `Textarea` renders through `Input` (G/input/textarea.rs:138-164). Ours is `TextInput`, a `Component` with `TextInputProps` and `TextInputState` (src/widgets/input/text_input.rs:206-235, src/widgets/input/text_input.rs:324-349, src/widgets/input/text_input.rs:902-904). One widget covers single-line, multi-line, password and numeric input through `InputMode` (src/widgets/input/text_input.rs:15-30). `builder::text_input()` wraps it in `ConfiguredTextInput` (src/builder/widgets/input.rs:23-25, src/builder/widgets/input.rs:224-296). The input dialog mounts a `TextInput` inside a modal (src/widgets/dialog/input/live.rs:278-297). We have two unrelated types named `TextInputBuilder` (src/widgets/input/text_input.rs:44-59; src/builder/widgets/input.rs:94-103).
- Features and options:
  - Both sides:
    - placeholder (B/input/base/state.rs:766-769; src/widgets/input/text_input.rs:79-83)
    - disabled (G/input/input.rs:309-313; src/widgets/input/text_input.rs:91-95)
    - read-only (G/input/input.rs:315-323; src/builder/widgets/input.rs:146-153, src/builder/widgets/input.rs:260-290)
    - password masking (B/input/base/state.rs:8981-8991; src/widgets/input/text_input.rs:121-125, src/widgets/input/text_input/paint.rs:123-124)
    - validation (B/input/base/state.rs:8993-9021, B/input/base/state.rs:3312-3332; src/widgets/input/text_input.rs:470-503)
    - multi-line with soft wrap (B/input/base/state.rs:9108-9135; src/widgets/input/text_input.rs:115-119, src/widgets/input/text_input.rs:163-167)
    - line breaks removed on paste into a single-line field (B/input/base/state.rs:3305-3306; src/widgets/input/text_input.rs:981-989)
    - undo and redo (B/input/base/undo_manager.rs:6-7; src/widgets/input/text_input.rs:505-566)
  - Only gpui-kit:
    - prefix and suffix slots (G/input/input.rs:240-248)
    - a clear button (G/input/input.rs:280-284, G/input/input.rs:652-656, G/input/input.rs:749-757; G/input/clear_button.rs:8-15)
    - a show/hide button for passwords (G/input/input.rs:286-290, G/input/input.rs:437-452)
    - a loading spinner (G/input/input.rs:743-745; B/input/base/state.rs:9081-9084)
    - a content type that picks the role and the macOS autofill hint (G/input/input.rs:292-299)
    - a custom context menu and a paste hook (G/input/input.rs:331-361)
    - clear the field on Escape (B/input/base/state.rs:1085-1093, B/input/base/state.rs:2039-2041)
    - submit on Enter in multi-line, with Shift+Enter for a newline (B/input/base/state.rs:1095-1103, B/input/base/state.rs:1897-1901)
    - textarea rows and auto-grow (B/input/base/state.rs:9167-9215)
    - multiple cursors (B/input/base/state.rs:200-212)
    - double-click selects a word and triple-click selects a line (B/input/base/state.rs:2263-2278; B/input/base/selection.rs:22-25, B/input/base/selection.rs:45-48; B/text_boundary.rs:28-53)
    - undo that merges a run of typing into one step and keeps 1000 steps (B/input/base/undo_manager.rs:6, B/input/base/undo_manager.rs:176-203)
    - the system clipboard (G/input/input.rs:584, G/input/input.rs:765)
    - touch selection handles (G/input/input.rs:363-435). These do not carry over to a terminal.
  - Only ours:
    - a maximum length counted in graphemes (src/widgets/input/text_input.rs:621-626). `rg -n -i 'max_length|maxlength|max_len\b|max_chars'` over G/input and B/input finds nothing.
    - a suggestion list under a single-line input, refreshed by a callback as the user types (src/widgets/input/text_input.rs:32-41, src/widgets/input/text_input.rs:405-412, src/widgets/input/text_input.rs:1009-1017; src/widgets/input/text_input/paint.rs:357-375)
    - named validators ("numeric", "alpha", "alphanumeric", "email") with an error line (src/widgets/input/text_input.rs:470-503; src/widgets/input/text_input/paint.rs:352-356)
    - a numeric character filter (src/widgets/input/text_input.rs:627-633)
    - line numbers, tab size and auto-indent in multi-line (src/widgets/input/text_input.rs:157-179, src/widgets/input/text_input.rs:1063-1073, src/widgets/input/text_input.rs:1086-1092)
    - a status glyph column (src/widgets/input/text_input/paint.rs:57-67)
    - The input dialog adds: required fields, rule lists, a custom validator, debounce, validation on blur, remote HTTP validation, a character count and warnings (src/widgets/dialog/input.rs:100-128, src/widgets/dialog/input.rs:159-231; src/widgets/dialog/input/live.rs:79-118, src/widgets/dialog/input/live.rs:314-356; src/widgets/dialog/input/live/remote.rs:32-37).
- Builder API:
  - gpui-kit:
    - `InputState::new(window, cx)` (B/input/base/state.rs:8958-8960).
    - State methods: `placeholder`, `default_value`, `masked`, `pattern`, `validate`, `mask_pattern`, `clean_on_escape` (B/input/base/state.rs:766, B/input/base/state.rs:1181, B/input/base/state.rs:8982, B/input/base/state.rs:8994, B/input/base/state.rs:9010, B/input/base/state.rs:3343, B/input/base/state.rs:1086).
    - `Input::new(&state)` (G/input/input.rs:180), then `prefix`, `suffix`, `cleanable`, `mask_toggle`, `content_type`, `role`, `aria_label`, `disabled`, `readonly`, `tab_index`, `context_menu`, `on_paste`, `appearance`, `bordered` (G/input/input.rs:230-361).
    - `Textarea::new` (G/input/textarea.rs:36), with `rows` and `auto_grow` on its state (B/input/base/state.rs:9178, B/input/base/state.rs:9212).
  - Ours:
    - `TextInputBuilder::new()` (src/widgets/input/text_input.rs:63), then `value`, `placeholder`, `max_length`, `disabled`, `width`, `mode`, `single_line`, `multi_line`, `password`, `numeric`, `validator_pattern`, `error_message`, `suggestions`, `show_line_numbers`, `wrap_text`, `tab_size`, `auto_indent`, `build`, `render` (src/widgets/input/text_input.rs:73-203).
    - Callbacks on the widget: `with_on_change`, `with_on_submit`, `with_suggestions` (src/widgets/input/text_input.rs:393-412).
    - `builder::text_input()` has `value`, `placeholder`, `disabled`, `readonly`, `max_length`, `input_type`, `class` and `build` (src/builder/widgets/input.rs:119-196). It has no change or submit callback: it builds `TextInput::new` without callbacks (src/builder/widgets/input.rs:230-232).
    - Dialog: `InputDialog::new(id, options)` (src/widgets/dialog/input.rs:275), with the `ValidationRule` helpers (src/widgets/dialog/input.rs:761-805).
- States:
  - gpui-kit: focus, disabled, read-only, loading, masked, multi-line, code-editor, text alignment, placeholder and mask placeholder (B/input/base/state.rs:465-475). The engine also tracks several selections, the IME marked range, drag-selecting, touch selection, the undo manager and the search session (B/input/base/state.rs:344-376). Events are Change, PressEnter{secondary, shift}, Focus and Blur (B/input/base/state.rs:121-127). There is also a `selected` look (G/input/input.rs:150-159).
  - Ours: cursor, one selection, focused, valid, x and y scroll, undo and redo stacks, suggestion index, whether suggestions are open, and the lines (src/widgets/input/text_input.rs:324-349). Disabled is a prop (src/widgets/input/text_input.rs:215-216) and read-only is a closure (src/widgets/input/text_input.rs:381-391). The dialog adds error, warnings, visible and remote-pending (src/widgets/dialog/input/live.rs:36-49).
- Keyboard and screen reader:
  - gpui-kit keys (non-macOS set, B/input/base/state.rs:131-303):
    - Backspace and Delete; Ctrl+Backspace and Ctrl+Delete delete a word.
    - Enter, Shift+Enter and secondary Enter; Escape.
    - Arrows, PageUp/PageDown; Tab and Shift+Tab indent.
    - Shift+arrows select; Home/End; Ctrl+Home/End; Ctrl+Shift+Left/Right select by word.
    - Ctrl+A/C/X/V; Ctrl+Left/Right move by word; Ctrl+Z/Y.
    - Ctrl+F search, Ctrl+H replace.
    - macOS adds Alt+Left/Right for word moves, and Ctrl+A/Ctrl+E for start and end of line (B/input/base/state.rs:259-278).
    - Right-click opens a native menu with Cut, Copy, Paste and Select All (G/input/input.rs:547-596).
  - gpui-kit screen reader:
    - The role follows the mode and content type: TextInput, PasswordInput, EmailInput, UrlInput, PhoneNumberInput, MultilineTextInput and others (G/input/input.rs:27-87).
    - The label falls back to the placeholder, unless the placeholder is a mask placeholder (G/input/input.rs:660-669).
    - The placeholder and value are exposed. The value is withheld when masked or a password (G/input/input.rs:89-95, G/input/input.rs:685-691).
    - The SetValue action replaces the text (G/input/input.rs:454-464, G/input/input.rs:692-696).
  - Our keys:
    - Ctrl+A/C/X/V, Ctrl+Z/Y, Ctrl+Left/Right/Home/End (with Shift to select), Ctrl+Backspace/Delete by word (src/widgets/input/text_input.rs:830-884). Alt keys are ignored (src/widgets/input/text_input.rs:890-899).
    - Enter submits in single-line, or inserts a newline with auto-indent in multi-line (src/widgets/input/text_input.rs:1063-1077).
    - Tab accepts a suggestion or indents in multi-line; otherwise focus moves on (src/widgets/input/text_input.rs:1078-1096).
    - Escape closes suggestions or clears the selection (src/widgets/input/text_input.rs:1097-1104).
    - Arrows, Home/End and PageUp/PageDown move, with Shift to select; Up/Down move within the suggestion list (src/widgets/input/text_input.rs:1105-1164).
    - Mouse: click, drag and Shift-click select; click a suggestion to accept it; the wheel scrolls (src/widgets/input/text_input.rs:1194-1263).
  - Our screen reader:
    - The role follows the mode: PasswordInput, MultilineTextInput, NumberInput or TextInput (src/widgets/input/text_input/accessibility.rs:14-19).
    - A read-only flag (src/widgets/input/text_input/accessibility.rs:20-22); the placeholder becomes the description (src/widgets/input/text_input/accessibility.rs:23-25).
    - The value, caret and selection are sent as text runs, with password graphemes as "•" (src/widgets/input/text_input/accessibility.rs:35-41; src/accessibility/text.rs:27-30, src/accessibility/text.rs:72-73).
    - The input's own error line has no alert role (src/widgets/input/text_input/paint.rs:352-356). The dialog labels the field and marks errors as Alert and warnings as Status (src/widgets/dialog/input/live.rs:300-306, src/widgets/dialog/input/live.rs:339-356).
    - The app acts only on Focus and Click requests, so a screen reader's SetValue is dropped (src/app.rs:411-417).
- Worth adopting:
  - Handle SetValue for text inputs, and pick EmailInput, UrlInput or PhoneNumberInput from `input_type` (src/builder/widgets/input.rs:213-218), as gpui-kit does (G/input/input.rs:454-464, G/input/input.rs:27-87). This lets screen-reader users set a field's text through the reader, and hear what kind of field it is. Medium.
  - Copy and cut to the system clipboard with OSC 52. Today copied text goes to a private buffer and never leaves the widget (src/widgets/input/text_input.rs:674-704); gpui-kit uses the system clipboard (G/input/input.rs:584). Small.
  - Prefix and suffix cells inside the existing `[ ]` frame (src/widgets/input/text_input/paint.rs:278, src/widgets/input/text_input/paint.rs:333), with a clear `[x]` and a password show/hide toggle (G/input/input.rs:240-248, G/input/input.rs:280-290). This saves app code for search boxes and password fields. Small.
  - Merge a run of typed characters into one undo step (B/input/base/undo_manager.rs:176-199), and bind Alt+Left/Right to word moves (B/input/base/state.rs:275-278). Today every insert is its own undo entry with a cap of 100 (src/widgets/input/text_input.rs:505-518, src/widgets/input/text_input.rs:637-644), and Alt keys do nothing (src/widgets/input/text_input.rs:890-899). Many macOS terminals send Alt+arrows for Option+arrows. Small.

#### number input
- gpui-kit:
  - A spin button: a centered text field between `-` and `+` buttons (G/input/number_input.rs:19-31, G/input/number_input.rs:133-186). Up and Down step the value (B/number_input.rs:13-22).
  - Options: a fixed step or one computed from the value, plus `min` and `max` (B/input/base/state.rs:8976-8979, B/input/base/state.rs:9032-9055; B/number_input.rs:30-54).
  - A step keeps the decimal digits and clamps to the range (B/number_input.rs:330-359). A typed value out of range is clamped on blur (B/input/base/state.rs:3198-3232).
  - With no step, min or max set, it emits `NumberInputEvent::Step` so the app can handle it (B/number_input.rs:56-91).
  - `controls_right` puts both buttons on the right (B/number_input.rs:203-207). The buttons never take focus (B/number_input.rs:247-271).
  - Role SpinButton with a numeric value (B/number_input.rs:304-305); the buttons are labeled Increment and Decrement (G/input/number_input.rs:137, G/input/number_input.rs:170).
- Ours: absent as a widget. `rg -n -i 'number_input|numberinput|spinbox|spin_box|stepper|NumberInput' src` and `rg -n -i 'spinbutton|spin_button|increment|decrement' src/widgets src/builder` find no widget. The nearest things:
  - `TextInput` numeric mode filters typing to digits, "." and "-" (src/widgets/input/text_input.rs:28-29, src/widgets/input/text_input.rs:627-633) and uses role NumberInput (src/widgets/input/text_input/accessibility.rs:17).
  - `Slider` has step, min and max, arrow, Page and Home/End keys, and role Slider (src/widgets/input/slider.rs:62-65, src/widgets/input/slider.rs:454-461, src/widgets/input/slider.rs:516-527).
- In a terminal: one row, `[-] 12.5 [+]`. Typing edits the number as in `TextInput`. Up/Down step; PageUp/PageDown take a bigger step; Home/End jump to min/max when set. Clicking `[-]`/`[+]` or turning the wheel over the field steps. Leaving the field clamps the value. The screen reader gets role SpinButton with value, min, max and step; our platform layer already maps SpinButton and numeric value and step (src/accessibility/platform/translation/node.rs:254, src/accessibility/platform/translation/node.rs:1805-1823).
- Call: build, small. Add step, min, max and Up/Down to `TextInput` numeric mode, and draw the two button cells.

#### OTP input
- gpui-kit:
  - A fixed-length one-time code, drawn as separate cells in groups (G/input/otp_input.rs:13-28, G/input/otp_input.rs:42-46, G/input/otp_input.rs:94-165).
  - `OtpState::new(length)` with `default_value`, `set_value` and `masked` (B/otp_input.rs:19-90).
  - Digits fill the next cell (full-width digits count too), Backspace removes the last digit, and entry stops at the length (B/otp_input.rs:95-119).
  - Events: Change, Complete (last cell filled), Focus and Blur (B/otp_input.rs:8-17, B/otp_input.rs:121-139). Masked cells show an asterisk icon (G/input/otp_input.rs:139-148).
  - The root sets no accessibility role or label (B/otp_input.rs:196-207). It handles only key-down, so pasting a code does not work (B/otp_input.rs:121-139).
- Ours: absent. `rg -n -i 'otp|pin_input|pininput|one.time|verification code' src --glob '*.rs'` finds no widget. The nearest is `TextInput` numeric mode with `max_length` (src/widgets/input/text_input.rs:621-633).
- In a terminal: one row of cells, `[1][2][3] [_][_][_]`, with the current cell in reverse video and masked cells as `•`. Digits fill; Backspace clears the last; a pasted code fills every cell; Enter, or typing the last digit, submits. A mouse click focuses the field. The screen reader gets one text node, not one per cell, labeled for example "Verification code, 6 digits", with the value or bullets.
- Call: build, small, as a thin variant of `TextInput`: digits only, fixed length, a Complete callback and cell drawing. CLI logins that ask for 2FA codes would use it.

#### input group
- gpui-kit:
  - A shared frame around one `Input` or `Textarea`, with addons on any of four sides (G/input/group.rs:1-9, G/input/group.rs:33-45, G/input/group.rs:327-335).
  - Addons hold text, icons or compact buttons: `InputGroupAddon`, `InputGroupButton`, `InputGroupText` (G/input/group.rs:341-347, G/input/group.rs:472-476, G/input/group.rs:673-677).
  - Options: `disabled`, `readonly`, `invalid` (visual only) and `aria_label` (G/input/group.rs:75-97).
  - Role Group (G/input/group.rs:203). When disabled, it blocks the mouse and every key except Tab (G/input/group.rs:219-227). Clicking an addon focuses the text control (G/input/group.rs:234-243). Border and background colors animate, and it draws a focus ring (G/input/group.rs:185-233).
- Ours: absent. `rg -n -i 'addon|input_group|inputgroup|form_field|formfield|FieldGroup' src --glob '*.rs'` finds nothing. The nearest is `TextInput`'s own `[ ]` frame with its status column (src/widgets/input/text_input/paint.rs:268-278, src/widgets/input/text_input/paint.rs:333, src/widgets/input/text_input/paint.rs:57-67).
- In a terminal: inline addons become cells in the same row, for example `[https:// example.com  .com]` or `[query  [x]]`. Block addons become rows inside a box border. Addon buttons would be Tab stops and clickable cells, and the group would get Role::Group with a label. The color animations and the soft ring do not carry over.
- Call: skip. Prefix and suffix cells on `TextInput` (see "input (text)") cover inline addons, and block addons are plain layout rows.

#### code editor
- gpui-kit:
  - `Editor` renders `EditorState` through `Input` in a monospace font (G/input/editor.rs:16-36, G/input/editor.rs:134-160).
  - The state starts with line numbers, soft tabs, indent guides and search turned on (B/input/base/state.rs:9218-9235).
  - Options: `language`, `folding` and `unfold_at`, `line_number`, `auto_close`, `smart_indent` (B/input/base/state.rs:9240, B/input/base/state.rs:9265-9297, B/input/base/state.rs:9322, B/input/base/state.rs:9343, B/input/base/state.rs:9360). Enter between a bracket pair splits it onto separate lines (B/input/base/state.rs:1921-1976).
  - Editor-only language features: LSP providers for completion, code actions, hover, go to definition, document colors and semantic tokens (B/input/editor/lsp/mod.rs:39-69; B/input/base/kind.rs:335-344). Also diagnostics (B/input/editor/diagnostics.rs:186-259), and a tree-sitter code/string/comment check plus per-language rules (G/input/syntax_context.rs:7-22; G/input/language_config.rs:9-29).
  - Keys and menu: multiple cursors (B/input/base/state.rs:200-212), Ctrl+. for code actions (B/input/base/state.rs:291-294), and right-click Go to Definition (G/input/input.rs:558-571).
- Ours: an editing core, not a widget.
  - `TextEditor` and `SyntaxEditor` sit on a gap buffer with a cursor, movements and line numbers, and draw to a `Surface` (src/editor/text_editor.rs:15-32, src/editor/text_editor.rs:153-172; src/editor/syntax_editor.rs:16-37; src/editor/cursor.rs:201-226).
  - `SyntaxEditor` picks a highlighter by language name or file extension (src/editor/syntax_editor.rs:79-105).
  - Only the FFI uses them (src/ffi/editor.rs:5, src/ffi/editor.rs:13-18). `rg -n -i 'undo|redo|search|fold|diagnos|complet' src/editor` finds no undo, search, folding or diagnostics.
  - `TextInput` multi-line has line numbers, tab size and auto-indent (src/widgets/input/text_input.rs:157-179, src/widgets/input/text_input.rs:1063-1073). But it rebuilds the whole string on every edit (src/widgets/input/text_input.rs:415-417, src/widgets/input/text_input.rs:525-544), so it is a poor base for large files.
- In a terminal:
  - Look: a gutter such as `  12 │` with `▸`/`▾` fold markers, and syntax colors from src/syntax. Diagnostics show as underlines, or as colored text where the terminal lacks underline colors, with an `E`/`W` sign in the gutter and the message on a status row. Extra cursors are reverse-video cells.
  - Input: keys do everything. The mouse places the cursor, drags to select, scrolls with the wheel, and clicks fold markers.
  - Screen reader: MultilineTextInput with text runs, which we already produce (src/accessibility/text.rs:6-74).
  - Key conflicts: in most terminals Ctrl+[ sends Escape and Ctrl+H sends Backspace. gpui-kit's Outdent and Replace keys (B/input/base/state.rs:191, B/input/base/state.rs:302) need other bindings.
- Call: build, large, in stages. First wrap `SyntaxEditor` in a Component with undo, selection, clipboard and scrolling. Then add search, folding and diagnostics. LSP comes last.

#### search and replace panel
- gpui-kit:
  - A panel above a multi-line input with a find field, a replace field, a match-case toggle, and Replace and Replace All (G/input/search.rs:61-69, G/input/search.rs:358-366, G/input/search.rs:449-458).
  - Engine calls: open, set query, next, previous, replace current, replace all (B/input/editor/search.rs:104-241).
  - Ctrl+F opens search and Ctrl+H opens replace (B/input/base/state.rs:295-302). The panel is off by default and on for the code editor (B/input/base/state.rs:9090-9100, B/input/base/state.rs:9233).
- Ours: absent. `rg -n -i 'code_editor|codeeditor|fn search_replace|find_replace|FindReplace' src --glob '*.rs'` finds nothing, and `rg -n -i 'search' src/editor` finds nothing.
- In a terminal:
  - Layout: a row at the top of the text area, `Find [needle   ] 3/12 [Aa] [↑][↓] [x]`, and a second row, `Replace [    ] [Replace] [All]`. Matches get a background color; the current match is in reverse video.
  - Keys: Enter goes to the next match. Shift+Enter works only where the terminal reports it (for example with the kitty keyboard protocol), so Up/Down or F3 should also go to the previous match. Escape closes the panel. Ctrl+H is Backspace in most terminals, so replace needs another key.
  - Screen reader: announce "3 of 12" as a Status.
- Call: build, medium, with the code editor. Multi-line `TextInput` can use it too.

#### editor popovers (completion, code actions, hover, diagnostics)
- gpui-kit: four popovers, all tied to `EditorState` (G/input/popovers/mod.rs:1-9):
  - a completion list from a `CompletionProvider`, which also supports inline ghost completions (G/input/popovers/completion_menu.rs:170-172, G/input/popovers/completion_menu.rs:254-261; B/input/editor/lsp/completions.rs:40-75)
  - a code action menu (G/input/popovers/code_action_menu.rs:137-139)
  - hover and diagnostic popovers that render Markdown (G/input/popovers/hover_popover.rs:15-16; G/input/popovers/diagnostic_popover.rs:15-18; G/input/popovers/mod.rs:21-44)
  - An overlay registry hosts them next to the search panel (G/input/overlay.rs:26-31).
- Ours:
  - A suggestion list under a single-line `TextInput`: a callback returns suggestions as the user types, Up/Down pick one, Tab or a click accepts it, and five rows show at a time (src/widgets/input/text_input.rs:32-41, src/widgets/input/text_input.rs:405-412, src/widgets/input/text_input.rs:1009-1017, src/widgets/input/text_input.rs:1078-1085, src/widgets/input/text_input.rs:1105-1112, src/widgets/input/text_input.rs:1170-1192; src/widgets/input/text_input/paint.rs:240-246, src/widgets/input/text_input/paint.rs:357-375).
  - The autocomplete dialog (src/widgets/dialog/autocomplete.rs:1-4, src/widgets/dialog/autocomplete.rs:27-35).
  - No code action, hover or diagnostic popovers. The building blocks exist: `Popover` (src/widgets/display/popover.rs:322) and `PopupMenu` (src/widgets/menu/popup.rs:183).
- In a terminal: completion is a list of rows below the cursor row, flipped above near the bottom edge. Ghost text is dim text after the cursor. Code actions are a popup menu; hover and diagnostic text sit in a bordered box. Keys: Up/Down, Enter or Tab to accept, Escape to close. The screen reader gets a list role that announces the selected item, and hover text as a Status. Shadows do not carry over.
- Call: build with the code editor, medium. Skip it as standalone work, since single-line completion already exists.

#### mask pattern
- gpui-kit:
  - `MaskPattern` formats text as the user types. Tokens: `9` is a digit, `A` an ASCII letter, `#` a letter or digit, `*` any character; anything else is literal (B/input/base/mask_pattern.rs:5-19, B/input/base/mask_pattern.rs:99-131).
  - A number mode adds a group separator and a fixed number of fraction digits (B/input/base/mask_pattern.rs:85-90, B/input/base/mask_pattern.rs:142-148, B/input/base/mask_pattern.rs:283-300).
  - The pattern sets a placeholder such as `(___)___-____` (B/input/base/mask_pattern.rs:150-158; B/input/base/state.rs:3343-3350).
  - Input that does not fit is refused, and the text is re-masked (B/input/base/state.rs:3312-3332, B/input/base/state.rs:3845-3859). `unmask_value` returns the raw text (B/input/base/state.rs:1210-1213).
- Ours: the input dialog's `mask` (src/widgets/dialog/input.rs:117-123), checked by `mask_matches`. There `#` is an ASCII digit, `A` a Unicode letter, `*` any non-control grapheme, and backslash escapes (src/widgets/dialog/input/validation.rs:43-72). The dialog shows `Format: <mask>` (src/widgets/dialog/input/live.rs:328-331). `TextInput` has no mask field (src/widgets/input/text_input.rs:206-235).
- Differences: gpui-kit formats and blocks as the user types. Ours checks afterwards and does not insert separators (src/widgets/dialog/input.rs:121). Only gpui-kit has number grouping and an unmasked value. Only ours has escapes and Unicode letters. `#` means letter-or-digit in gpui-kit and digit in ours.
- Worth adopting:
  - A typing-time mask on `TextInput`: insert the literal separators, refuse wrong characters, show `(___) ___-____` as the placeholder, and return the raw value. This helps phone, date and card entry. Medium.
  - Number grouping (`1,234.50`) in numeric mode. Small once the mask exists.

#### content type (support code)
- `InputContentType` is a hint about what a field holds (name, email, password, one-time code and so on). It picks the accessible role and tells macOS autofill what to offer (G/input/content_type.rs:3-7, G/input/content_type.rs:152-174; G/input/input.rs:27-87). Our equivalents are `InputType` in the dialog (src/widgets/dialog/input.rs:138-157) and `input_type` on the builder (src/builder/widgets/input.rs:164-171, src/builder/widgets/input.rs:205-222). We use them only for validation and mode. Autofill has no terminal equivalent.

#### state.rs and overlay.rs (support code)
- `AnyInputState` wraps any of the four input states, for APIs like "the focused input" (G/input/state.rs:8-26). `TextInputState` lets one `Input` element render three modes (G/input/state.rs:28-52). `overlay.rs` keeps a registry of search panels and popovers (G/input/overlay.rs:12-31). Ours: one `TextInput` handles every mode (src/widgets/input/text_input.rs:15-30), and we have no overlay registry.

#### language_config.rs and syntax_context.rs (support code)
- Per-language editing rules (brackets, auto-closing pairs, indentation) and a tree-sitter check that tells code from strings and comments. The editor uses them for auto-close and smart indent (G/input/language_config.rs:1-29; G/input/syntax_context.rs:7-30). Ours: `SyntaxEditor` detects the language and highlights through src/syntax (src/editor/syntax_editor.rs:11-12, src/editor/syntax_editor.rs:79-105), with no editing rules.

#### B/text_selection.rs, B/selectable_text.rs, B/text_boundary.rs (support code)
- Window-wide selection and copy of static text, with double-click for a word and triple-click for a line (B/selectable_text.rs:11-27; B/text_selection.rs:16, B/text_selection.rs:460-461). `text_boundary` also drives word selection inside inputs (B/input/base/selection.rs:9, B/input/base/selection.rs:22-25; B/text_boundary.rs:28-53). Ours: none found. `rg -n -i 'selectable_text|SelectableText|text_selection' src --glob '*.rs' --glob '!src/accessibility/**'` finds only accessibility plumbing. In a terminal, the emulator's own Shift+drag selection often does this job.

### menu
- Files: ours src/widgets/menu/mod.rs, src/widgets/menu/item.rs, src/widgets/menu/model.rs, src/widgets/menu/view.rs, src/widgets/menu/panels.rs, src/widgets/menu/popup.rs, src/widgets/menu/popup_live.rs, src/widgets/menu/menubar_live.rs, src/widgets/menu/context.rs, src/widgets/menu/context_live.rs, src/widgets/menu/dialog.rs, src/widgets/menu/runtime.rs, src/builder/widgets/menu.rs; gpui-kit G/menu/mod.rs, G/menu/popup_menu.rs, G/menu/menu_item.rs, G/menu/context_menu.rs, G/menu/dropdown_menu.rs, G/menu/app_menu_bar.rs.
- Structure:
  - gpui-kit:
    - `PopupMenu` is an Entity made with `PopupMenu::build(window, cx, |menu, ..| ..)` (G/menu/popup_menu.rs:282-310, 358-364).
    - Items are the `PopupMenuItem` enum, and their actions are gpui `Action`s (G/menu/popup_menu.rs:33-65, 145).
    - `ContextMenu` wraps any element through `.context_menu(..)` (G/menu/context_menu.rs:13-39). `DropdownMenu` is a trait on Button (G/menu/dropdown_menu.rs:12-34).
    - `AppMenuBar` reads gpui's `OwnedMenu` app menus, the same list the native menu uses (G/menu/app_menu_bar.rs:46-62).
  - Ours:
    - `MenuItem` is a data tree with callback actions (src/widgets/menu/item.rs:22-59, 105-130).
    - A shared `MenuModel` keeps the selected path and carries it across updates (src/widgets/menu/model.rs:4-9, 121-143).
    - Each kind (menubar, popup, context, dialog menu) has props, a builder and a live runtime. `WorldEvents` delivers events to overlays outside the root rectangle (src/widgets/menu/mod.rs:9-38; src/widgets/menu/runtime.rs:1-53).
- Features and options:
  - Both sides:
    - Action items, submenus, checked items, disabled items, separators, icons and shortcut text.
    - A right-click context menu (G/menu/context_menu.rs:296-300; ours src/widgets/menu/context_live.rs:191-200).
    - Height limits: gpui-kit has `max_h` and `scrollable` (G/menu/popup_menu.rs:440-450); ours has `max_visible_items` (src/widgets/menu/context.rs:302-306).
  - Only gpui-kit:
    - Non-interactive label items (G/menu/popup_menu.rs:36-37, 117-121, 491-495).
    - Link items that open a URL (G/menu/popup_menu.rs:212-216, 497-500).
    - Items with custom element rendering (G/menu/popup_menu.rs:49-57, 586-590).
    - The shortcut hint is read from the real key binding of the item's action (G/menu/popup_menu.rs:1119-1150).
    - A choice of which side the check mark sits on (G/menu/popup_menu.rs:452-456).
  - Only ours:
    - Radio items in groups (src/widgets/menu/item.rs:94-100; src/widgets/menu/model.rs:155-175).
    - Separator styles (src/widgets/menu/item.rs:61-79).
    - A visible flag and a description per item (src/widgets/menu/item.rs:117, 128-129).
    - Long press opens a context menu (src/widgets/menu/context_live.rs:201-215; src/widgets/menu/context.rs:284-294).
    - Dialog menus for selection, multi-selection, confirmation and input (src/widgets/menu/dialog.rs:9-21).
    - The mouse wheel moves the selection (src/widgets/menu/model.rs:216-234).
- Builder API:
  - gpui-kit:
    - `PopupMenu::build(window, cx, |menu, window, cx| ..)` (G/menu/popup_menu.rs:358-364). Inside the closure: `menu.menu("Copy", Box::new(Copy))`, `.menu_with_check(..)`, `.separator()`, `.submenu("More", window, cx, |m, ..| ..)`, `.link(..)`, `.label(..)`, `.item(..)` (G/menu/popup_menu.rs:465-500, 564-584, 693-730).
    - `element.context_menu(|menu, ..| ..)` (G/menu/context_menu.rs:19-36).
    - `button.dropdown_menu(|menu, ..| ..)` (G/menu/dropdown_menu.rs:14-19).
    - `AppMenuBar::new(cx)` (G/menu/app_menu_bar.rs:32-44).
  - Ours:
    - `menubar()`, `context_menu()` and `popup_menu()` (src/builder/widgets/menu.rs:316-329).
    - Items: `menu_item(..)`, `action_item(..)`, `submenu_item(..)`, `checkbox_item(..)`, `radio_item(..)` and `separator()` (src/builder/widgets/menu.rs:231-314).
    - `MenuBarBuilder` with `.item()`, `.show_shortcuts()`, `.on_item_selected()` and `.build()` (src/builder/widgets/menu.rs:406-517).
    - `ContextMenuBuilder` with `.trigger_on_right_click()`, `.class()` and others (src/builder/widgets/menu.rs:554-653).
- States:
  - gpui-kit: selected index, bounds, max size, scrollable, parent menu and the focus to restore (G/menu/popup_menu.rs:282-310).
  - Ours:
    - The selected path through nested submenus (src/widgets/menu/model.rs:4-9).
    - Checked and selected state per item (src/widgets/menu/item.rs:89-100), plus enabled and visible (src/widgets/menu/item.rs:114-117).
    - A focused flag per runtime (src/widgets/menu/popup_live.rs:61; src/widgets/menu/menubar_live.rs:52).
    - The context menu's open position (src/widgets/menu/context.rs:62-66).
- Keyboard and screen reader:
  - gpui-kit keys:
    - In a popup menu, Enter confirms, Escape dismisses and Up/Down move with wrap-around. Left and Right open or leave submenus (G/menu/popup_menu.rs:21-30, 913-985).
    - In the menubar, Left/Right wrap and Escape closes (G/menu/app_menu_bar.rs:17-22, 64-92).
    - Dismiss restores the earlier focus unless the item's handler already moved focus (G/menu/popup_menu.rs:1055-1072).
    - A context menu opens only with the right mouse button (G/menu/context_menu.rs:296-300).
  - gpui-kit screen reader:
    - Roles are Menu (G/menu/popup_menu.rs:1466) and MenuBar (G/menu/app_menu_bar.rs:119-123).
    - Items are MenuItem, named from their text and marked aria-selected (G/menu/menu_item.rs:94-101; G/menu/popup_menu.rs:272-279, 1244).
    - No checked or expanded state found (searched G/menu for "toggled", "aria_checked", "MenuItemCheckBox", "MenuItemRadio" and "expanded").
  - Ours keys:
    - In a popup: Up/Down with wrap-around, PageUp/PageDown, Home/End, Right into a submenu, Left or Escape back one level, Escape to close, Enter or Space to activate. Item shortcuts fire while the menu has focus (src/widgets/menu/popup_live.rs:289-333; src/widgets/menu/model.rs:235-255).
    - The menubar works the same way, with Left/Right across the bar and Down to open (src/widgets/menu/menubar_live.rs:267-337).
    - Popups get a focus trap with focus restore (src/widgets/menu/popup_live.rs:257-262; src/component/focus.rs:102-108).
    - A context menu opens by right click or long press only (src/widgets/menu/context_live.rs:191-215).
  - Ours screen reader:
    - Roles are Menu (src/widgets/menu/popup_live.rs:255-256; src/widgets/menu/panels.rs:252) and MenuBar (src/widgets/menu/menubar_live.rs:243).
    - Items are MenuItem, MenuItemCheckBox or MenuItemRadio, with toggled state. Separators are Splitter. Items also expose expanded (for submenus), disabled and a description (src/widgets/menu/view.rs:164-196).
    - Screen-reader focus follows the selected path (src/widgets/menu/view.rs:219-227).
- Worth adopting:
  - Non-interactive label items used as group headings (G/menu/popup_menu.rs:36-37, 117-121, 1255-1260). Long terminal menus read as named groups, both on screen and in a screen reader. Ours only has Action, Submenu, Checkbox, Radio and Separator (src/widgets/menu/item.rs:83-103). Size: small.
  - Build the shortcut hint from the same keys that trigger the item, as gpui-kit reads the real binding (G/menu/popup_menu.rs:1119-1144). Ours stores the display text apart from the keys (src/widgets/menu/item.rs:5-10), so the hint can drift from what actually works. Size: small.
  - Only one line on native menus: G/menu's `AppMenuBar` takes the same `OwnedMenu` list that feeds the native menu (G/menu/app_menu_bar.rs:46-62).

### notification
- Files: ours src/widgets/dialog/toast.rs, src/widgets/dialog/toast/live.rs, src/widgets/dialog/engine.rs, src/widgets/dialog/engine/content.rs, src/builder/widgets/dialog.rs; gpui-kit G/notification.rs, G/window_ext.rs, B/toast.rs.
- Structure:
  - gpui-kit: each `Notification` is an Entity with Render and builder methods (G/notification.rs:106-126, 395-526). A `NotificationList` entity keeps them in a base `ToastManager` and draws one `ToastStack` for each placement (G/notification.rs:692-701, 973-1037; B/toast.rs:110-116).
  - Ours: a toast is a `ToastOptions` value rendered by `LiveToast`. `LiveToast` wraps a non-modal `Modal`, keeps its visibility in a signal and keeps its timer in the scheduler (src/widgets/dialog/toast/live.rs:26-42, 130-188). It is opened with `DialogEngine::show_toast` (src/widgets/dialog/engine.rs:584) or built with `builder::toast()` (src/builder/widgets/dialog.rs:43-45, 367-396).
- Features and options:
  - Both sides:
    - Info, success, warning and error types (G/notification.rs:30-37; ours src/widgets/dialog/toast.rs:18-31, which adds Custom).
    - Auto-dismiss. gpui-kit always waits 5s (G/notification.rs:314-317, 841); ours takes any Duration or None (src/widgets/dialog/toast.rs:40-42).
    - A close button (G/notification.rs:448-465; ours src/widgets/dialog/toast.rs:45-46).
    - An on_close callback (G/notification.rs:328-335; ours src/widgets/dialog/toast.rs:47-48).
    - Placements: gpui-kit handles eight anchors, including left-center and right-center (G/notification.rs:1022-1033), and `placement` takes any gpui Anchor (G/notification.rs:266-268); ours has six (src/widgets/dialog/toast.rs:75-90).
  - Ours starts the timer only after the toast is fully shown (src/widgets/dialog/toast.rs:40-41; src/widgets/dialog/toast/live.rs:62-71).
  - Only gpui-kit:
    - Title, icon and custom content (G/notification.rs:240-260, 376-383).
    - An action button, which turns off auto-hide (G/notification.rs:337-347).
    - An on_click callback (G/notification.rs:319-326) and middle-click dismiss (G/notification.rs:472-476).
    - Pushing a toast with an existing id replaces the old toast (G/notification.rs:108-112, 229-238; B/toast.rs:178-195).
    - Remove by type and clear all (G/window_ext.rs:64-75).
    - At most 10 toasts are visible (G/notification.rs:537-538, 558).
    - Stacking, with collapsed layers that expand (B/toast.rs:22-55).
    - Timers pause while the stack is hovered or focused (B/toast.rs:73-77, 225-256).
    - Delivery to the OS notification center (G/notification.rs:50-73, 849-872).
    - The close button appears only on hover (G/notification.rs:449-454). This is a hover-only affordance and does not carry over to a terminal.
    - Slide and shadow animation (G/notification.rs:477-524) is pixel work.
  - Ours: I found no stacking. Engine toasts get empty bounds (src/widgets/dialog/engine/content.rs:130, 148). Empty bounds leave the position unchanged (src/widgets/dialog/frame.rs:26-33), so toasts with the same position get the same corner (src/widgets/dialog/toast/live.rs:139-146). I searched the toast and engine files for "stack", "offset", "pause" and "max_toasts"; the only hit was a z-index comment (src/widgets/dialog/engine.rs:78).
- Builder API:
  - gpui-kit: `window.push_notification(note, cx)` (G/window_ext.rs:64-65). A note is built with `Notification::success("..")` and `.title() .autohide() .action() .placement() .on_click()` (G/notification.rs:163-347). App-wide defaults live in `NotificationSettings` (G/notification.rs:530-545).
  - Ours:
    - `builder::toast().success("..").duration(ms).persistent().closable(..).position("top-right").class(..).build()` (src/builder/widgets/dialog.rs:248-396). The position is a string, and an unknown string renders error text (src/builder/widgets/dialog.rs:369-377).
    - `engine.show_toast(ToastOptions { .. })` (src/widgets/dialog/engine.rs:584).
- States:
  - gpui-kit: transition phase Starting, Present or Ending (B/toast.rs:584-594); time left before auto-hide (B/toast.rs:87-95); stack hovered or focused (B/toast.rs:64-78).
  - Ours: a visible signal, a timer, and presented and mounted flags (src/widgets/dialog/toast/live.rs:26-42).
- Keyboard and screen reader:
  - gpui-kit:
    - Every toast has the Alert role (B/toast.rs:653-659).
    - The list has a focus handle that is a Tab stop (G/notification.rs:708). Focus inside it expands the stack and pauses the timers (B/toast.rs:327-331, 414-421).
    - No key bindings found (searched G/notification.rs and B/toast.rs for "KeyBinding", "on_action" and "bind_keys").
  - Ours:
    - Role Status for info and success, Alert for warning and error (src/widgets/dialog/toast/live.rs:147-152).
    - The message is a polite or assertive live region (src/widgets/dialog/toast/live.rs:158-161; the class mapping is at src/accessibility/style.rs:227-232, with a test at 330-335).
    - The toast takes no focus (src/widgets/dialog/toast/live.rs:163-164).
    - Escape handling and a focusable Close button are added when the toast is closable (src/widgets/dialog/toast/live.rs:166-167, 182-187; src/widgets/display/modal/live/render.rs:350-368).
- Worth adopting:
  - Stack toasts that share a position, with a cap on how many show. Toasts in one corner would no longer cover each other. Size: medium.
  - Replace a toast by id (B/toast.rs:178-195). For example, "Saving..." becomes "Saved" instead of the toasts piling up. Size: small.
  - Pause the auto-dismiss timer while keyboard focus is in the toast area, or while the mouse is over it where the terminal reports motion (B/toast.rs:225-256, 414-421). Screen-reader users get time to read the message. Size: medium.
  - An action button such as Undo that turns off auto-hide (G/notification.rs:337-347). The user gets a way to reverse an action from the toast itself. Size: small.

### popover
- Files: ours src/widgets/display/popover.rs, src/widgets/display/popover/live.rs, src/widgets/display/popover/live/render.rs, src/widgets/display/popover/live/events.rs, src/widgets/display/overlay.rs, src/builder/widgets/display.rs; gpui-kit G/popover.rs, B/popover.rs, B/popup.rs, B/positioner.rs.
- Structure:
  - gpui-kit: `Popover` is a RenderOnce struct with builder methods (G/popover.rs:104-259, 293-324) over the base `Popover`. Its open state and focus live in a `PopoverState` entity kept in keyed element state (B/popover.rs:27-42, 269-280). `Popup` and `Positioner` place it (B/popup.rs:23-36, 128-145).
  - Ours: a `Popover` component with a `PopoverProps` struct (src/widgets/display/popover.rs:136-191). The handle holds state in `Arc<Mutex<PopoverState>>` plus a live runtime and has `show()` and `hide()` (src/widgets/display/popover.rs:320-325, 565-582). There is also a short Element builder (src/builder/widgets/display.rs:225-291).
- Features and options:
  - Both sides:
    - A click trigger.
    - Controlled open state (G/popover.rs:190-211; ours src/widgets/display/popover.rs:140, 575-582).
    - Close on outside click (G/popover.rs:219-223; ours src/widgets/display/popover.rs:161-162, src/widgets/display/popover/live/render.rs:133-161).
    - Close with Escape (B/popover.rs:324; ours src/widgets/display/popover/live/events.rs:5-19).
  - Only gpui-kit:
    - Choose which mouse button opens it (G/popover.rs:162-166).
    - An unstyled mode (G/popover.rs:240-249).
    - Pass in the focus handle that receives focus on open (G/popover.rs:251-258).
    - Its popover uses corner anchoring, which clamps to the window but does not flip (B/popup.rs:137; B/positioner.rs:27-36). Only dropdown surfaces use side placement, which flips (G/popover.rs:33-39; B/positioner.rs:210-241).
    - The open motion is pixel work (G/popover.rs:82-102).
  - Only ours:
    - 12 positions (src/widgets/display/popover.rs:12-38).
    - Edge behavior Flip, Shift, Hide or Ignore (src/widgets/display/popover.rs:123-134, 447-513).
    - Hover and focus triggers with delays (src/widgets/display/popover.rs:75-86, 165-168; src/widgets/display/popover/live/events.rs:35-89).
    - An arrow with three styles (src/widgets/display/popover.rs:88-121).
    - Min and max size (src/widgets/display/popover.rs:177-184, 515-533).
    - An optional backdrop dim (src/widgets/display/popover/live/render.rs:149-151).
    - An on_position_change callback (src/widgets/display/popover/live/render.rs:128-131).
    - Close on trigger click (src/widgets/display/popover/live/events.rs:30-32).
    - Fade, Scale, Slide and Bounce animation (src/widgets/display/popover/live/render.rs:189-216).
- Builder API:
  - gpui-kit: `Popover::new(id).trigger(button).anchor(..).content(|state, window, cx| ..).open(..).on_open_change(..).overlay_closable(..)` (G/popover.rs:130-258).
  - Ours: `Popover::builder()` with `position`, `trigger`, `content`, `trigger_element`, `boundary_behavior`, `focus_trap` and others; `.build()` returns `(Popover, PopoverProps)` (src/widgets/display/popover.rs:343-345, 621-866). Or `builder::popover().trigger(..).content(..).class(..).build()`, which returns an Element (src/builder/widgets/display.rs:61-63, 237-291).
- States:
  - gpui-kit: open, its own focus handle, the tracked focus handle, and the previous focus (B/popover.rs:32-42).
  - Ours: visible, the position after any flip, rectangles, animation progress, hovered, focused, boundary_adjusted and the last mouse position (src/widgets/display/popover.rs:264-296).
- Keyboard and screen reader:
  - gpui-kit:
    - Enter and Space toggle it from the trigger; Escape closes it (B/popover.rs:17-23, 290-297, 323-324).
    - Opening moves focus into the popover, and closing returns focus to the earlier element (B/popover.rs:99-125).
    - The content has the Dialog role (B/popover.rs:314-319).
    - The trigger is only marked "selected" (B/popover.rs:212-215). No expanded state found (searched B/popover.rs, G/popover.rs and B/popup.rs for "expanded" and "aria").
  - Ours:
    - Enter, Space or a click on the trigger opens it (src/widgets/display/popover/live/events.rs:27-33, 94-110); Escape closes it (src/widgets/display/popover/live/events.rs:5-19).
    - The trigger gets aria-expanded true or false (src/widgets/display/popover/live/render.rs:25-32). A plain-text trigger becomes a button focus stop (src/widgets/display/popover/live/render.rs:33-37).
    - The body has the Dialog role with focus_trap on and Group otherwise (src/widgets/display/popover/live/render.rs:226-231).
    - Focus moves in only when focus_trap or auto_focus is set (src/widgets/display/popover/live/render.rs:232-240). Both are false by default (src/widgets/display/popover.rs:248-249).
- Worth adopting:
  - By default, move focus into a click-opened popover and return it to the trigger on close (B/popover.rs:99-125). Keyboard users reach the content without hunting with Tab and land back where they started. Size: small.

### progress
- Files: ours src/widgets/display/progress_bar.rs, src/widgets/display/progress_bar/live.rs, src/widgets/display/progress_bar/live/motion.rs, src/widgets/dialog/progress.rs, src/widgets/dialog/progress/live.rs, src/widgets/dialog/frame.rs, src/builder/widgets/display.rs, src/builder/dialog_builders.rs, src/builder/macros.rs; gpui-kit G/progress/mod.rs, G/progress/progress.rs, G/progress/progress_circle.rs, B/progress.rs, G/button/button_icon.rs, G/theme/theme_color.rs.
- Structure:
  - gpui-kit: `Progress` and `ProgressCircle` are `RenderOnce` structs with builder methods (G/progress/progress.rs:13-66, G/progress/progress_circle.rs:16-70). `Progress` renders the headless base `Progress` root with `ProgressTrack` and `ProgressIndicator` parts (G/progress/progress.rs:120-170, B/progress.rs:10-48, B/progress.rs:127-128). The animated value is a `transition` keyed by the element id (G/progress/progress.rs:111-118). The circle paints two plot arcs on a canvas (G/progress/progress_circle.rs:74-140).
  - ours: `ProgressBarBuilder` builds `ProgressBarProps`. The `ProgressBar` component renders a `LiveProgress` element (src/widgets/display/progress_bar.rs:18-248, src/widgets/display/progress_bar.rs:534-553). `LiveProgress` lays out cell text runs for its viewport and samples a `Motion` clock on a 16 ms timer (src/widgets/display/progress_bar/live.rs:21-41, src/widgets/display/progress_bar/live/motion.rs:23-27, src/widgets/display/progress_bar/live/motion.rs:79-89). `ProgressDialog` wraps a progress bar in a modal (src/widgets/dialog/progress/live.rs:62-134).
- Features and options:
  - Shared: a value, an indeterminate mode, a color, a value animation, and an indeterminate bar that stands still under reduced motion (src/widgets/display/progress_bar.rs:50-54, src/widgets/display/progress_bar.rs:87-91, src/widgets/display/progress_bar.rs:105-109, src/widgets/display/progress_bar/live/motion.rs:65-94; G/progress/progress.rs:38-59, G/progress/progress.rs:111-118, G/progress/progress.rs:148-164).
  - Only ours:
    - A min and max range (src/widgets/display/progress_bar.rs:56-73).
    - Percent, value or custom-formatted text, and a label row (src/widgets/display/progress_bar.rs:177-181, src/widgets/display/progress_bar.rs:443-475, src/widgets/display/progress_bar/live.rs:70-81, src/widgets/display/progress_bar/live.rs:136-146).
    - Vertical orientation (src/widgets/display/progress_bar.rs:147-157, src/widgets/display/progress_bar/live.rs:84-99).
    - Segments, stripes and pulse (src/widgets/display/progress_bar/live.rs:113-118, src/widgets/display/progress_bar/live.rs:285-322, src/widgets/display/progress_bar/live/motion.rs:95-99).
    - An `on_complete` callback (src/widgets/display/progress_bar.rs:506-518).
    - A red error line for a bad range, size or color (src/widgets/display/progress_bar/live.rs:59-68, src/widgets/display/progress_bar/live.rs:265-283).
    - A modal progress dialog with Cancel, `on_cancel` and a time-remaining estimate (src/widgets/dialog/progress.rs:18-33, src/widgets/dialog/progress/live.rs:76-125).
  - Only gpui-kit:
    - A circular progress (G/progress/progress_circle.rs:16-140) that can be a button's icon (G/button/button_icon.rs:52-58).
    - Five sizes (G/progress/progress.rs:96-102).
    - A theme color token for the bar, with the track at 20% opacity of it (G/theme/theme_color.rs:209, G/progress/progress.rs:83-86, G/progress/progress.rs:135).
    - A separate accessible name (G/progress/progress.rs:61-65).
  - Differences:
    - Fill: ours fills whole cells only (src/widgets/display/progress_bar/live.rs:310), so a 20-cell bar moves in 5% steps. gpui-kit's width is continuous (G/progress/progress.rs:166).
    - Value animation: ours animates only when `animated` is set, linearly over 200 ms (src/widgets/display/progress_bar/live/motion.rs:10, src/widgets/display/progress_bar/live/motion.rs:70-77). gpui-kit always animates, using the theme's motion tokens (G/progress/progress.rs:111-118).
    - Default colors: ours uses the class tokens `bg-blue` and `bg-gray-200` (src/widgets/display/progress_bar.rs:233-234). `grep -rni progress src/theme` found no progress color.
    - Reduced motion: both gpui-kit parts stand still. The bar checks it itself (G/progress/progress.rs:149). The circle animates through gpui's `with_animation` (G/progress/progress_circle.rs:197-208), which gpui draws static under reduced motion; gpui-kit's spinner does the same with no check of its own (G/spinner.rs:60-75), and its test asserts that no frame is requested (G/spinner.rs:90-102).
    - Builder gaps: the smaller `builder::progress_bar()` has no `indeterminate` setter (src/builder/widgets/display.rs:82-200). `ProgressDialogBuilder` has no `on_cancel` or time-remaining setter (src/builder/dialog_builders.rs:12-99).
- Builder API:
  - gpui-kit:
    - `Progress::new(id)` with `value` (0 to 100), `loading`, `color`, `accessibility_label`, `with_size` and style refinement (G/progress/progress.rs:24-79).
    - `ProgressCircle::new(id)` has the same methods and can hold children in its middle (G/progress/progress_circle.rs:28-70, G/progress/progress_circle.rs:156-160).
    - The headless base `Progress::new(id)` has `value`, `indeterminate` and `accessibility_label` (B/progress.rs:21-48).
  - ours:
    - `ProgressBarBuilder::new()` with `value`, `range`, `show_percentage`, `show_value`, `indeterminate`, `animated`, `label`, `color`, `background_color`, `height`, `width`, `vertical`, `segments`, `striped`, `pulse`, `custom_formatter`, `on_complete`, `build` and `render` (src/widgets/display/progress_bar.rs:44-219).
    - Prop helpers such as `ProgressBar::with_value` (src/widgets/display/progress_bar.rs:384-441).
    - `builder::progress_bar()` and the `progress_bar!` macro (src/builder/widgets/display.rs:23-25, src/builder/widgets/display.rs:93-200, src/builder/macros.rs:331-354).
    - `ProgressDialogBuilder::new()` with `title`, `message`, `progress` (0 to 1), `indeterminate`, `cancelable`, `show_percentage`, `class` and `build` (src/builder/dialog_builders.rs:28-99).
- States:
  - gpui-kit: the value clamped to 0 to 100, loading, color and size (G/progress/progress.rs:13-22, G/progress/progress.rs:56-59). The animated value lives in a keyed transition (G/progress/progress.rs:111-118). There is no completed or error state.
  - ours:
    - `ProgressBarState` holds the animation frame, completed, the last value, the indeterminate position and the pulse direction (src/widgets/display/progress_bar.rs:365-379).
    - Completion fires `on_complete` once (src/widgets/display/progress_bar.rs:506-518).
    - `Motion` keeps the start value, the target and the timer (src/widgets/display/progress_bar/live/motion.rs:11-17).
    - An invalid-input error is tracked (src/widgets/display/progress_bar/live.rs:43).
    - The dialog keeps its visibility and its start time for the estimate (src/widgets/dialog/progress/live.rs:39-43, src/widgets/dialog/progress/live.rs:76-87).
- Keyboard and screen reader:
  - ours:
    - The bar ignores input (src/widgets/display/progress_bar.rs:555-563) and sets no focus props in its render (src/widgets/display/progress_bar/live.rs:41-202).
    - Its node has role ProgressIndicator. The name is the visible label, or "Progress" when there is none, and the value is the shown text. When the bar is not indeterminate, it also sets numeric value, min and max (src/widgets/display/progress_bar/live.rs:186-195).
    - The dialog is a modal with role Dialog and an autofocused Cancel button. Escape closes it when it is cancellable and calls `on_cancel` (src/widgets/dialog/progress/live.rs:88-89, src/widgets/dialog/progress/live.rs:101-109, src/widgets/dialog/progress/live.rs:118-133, src/widgets/dialog/frame.rs:36-40).
    - No live announcement: `grep -rn "aria-live\|set_busy\|live-polite\|announce"` over both our progress widgets found nothing.
  - gpui-kit: role ProgressIndicator, an optional name, min 0 and max 100, and a numeric value only when the bar is not indeterminate (B/progress.rs:70-84). Keys: none found. `grep -rn -i "key\|action\|on_click\|focus"` over G/progress and B/progress.rs found nothing.
- Worth adopting:
  - Smooth fill: draw the fill edge with eighth blocks (▏▎▍▌▋▊▉), so the bar moves in 1/8-cell steps. This comes closer to gpui-kit's continuous width (G/progress/progress.rs:166). Ours rounds down to whole cells (src/widgets/display/progress_bar/live.rs:310), while the chart canvas already resolves eighth-block edges (src/widgets/display/charts/mask.rs:8-10). Why: users see steady movement on narrow bars. It needs the ASCII fallback. Size: small.
  - A theme progress color, with the track derived from it (G/theme/theme_color.rs:209, G/progress/progress.rs:83-86, G/progress/progress.rs:135). This would replace the fixed `bg-blue` and `bg-gray-200` defaults (src/widgets/display/progress_bar.rs:233-234). Why: bars stay readable in dark and light presets without setting colors by hand. Size: small.
  - A separate accessible name, like `accessibility_label` (G/progress/progress.rs:61-65). Why: without a visible label, a screen reader hears only "Progress" (src/widgets/display/progress_bar/live.rs:188), and several such bars sound the same. Size: small.
  - A one-cell circular progress, like `ProgressCircle` used as a button icon (G/progress/progress_circle.rs:16-26, G/button/button_icon.rs:52-58). It could use glyphs such as ○◔◑◕●, with an ASCII fallback. Why: shows progress inside buttons, list rows and status lines where a bar does not fit. Size: small.

### radio
- Files: ours src/widgets/input/radio_button.rs, src/widgets/input/named_radio.rs, src/builder/specialized.rs, src/builder/widgets/input.rs; gpui-kit G/radio.rs, B/radio.rs, B/radio_group.rs.
- Structure:
  - gpui-kit: `Radio` is a `RenderOnce` struct over the base `Radio` (G/radio.rs:15-20, 164-166). `RadioGroup` is a `RenderOnce` that gives each child its id, its position, the set size and its checked flag, all taken from `selected_index` (G/radio.rs:385-386, 397-417). The owner controls the value through `on_change` (G/radio.rs:114-124, 325-334).
  - ours: we have two widgets.
    - `RadioButton<T>` is one live component that holds every option. It draws a flex column or row of text rows (src/widgets/input/radio_button.rs:155-160, 203-239).
    - `NamedRadio` is one component per radio. Radios join a group by name through a shared `RadioGroups` registry looked up in component scope, and the selected flag lives in an `Arc<AtomicBool>` (src/widgets/input/named_radio.rs:17-18, 46-53, 93-105).
    - `builder::radio_button()` produces a `NamedRadio` (src/builder/widgets/input.rs:48-50; src/builder/specialized.rs:283-291).
- Features and options:
  - Both sides have label, checked or selected, disabled, and a vertical or horizontal group (src/widgets/input/radio_button.rs:61-77, 95-103; G/radio.rs:69-95, 305-318).
  - Only ours:
    - option values of any type (src/widgets/input/radio_button.rs:105-114)
    - per-option disabled inside a group (src/widgets/input/radio_button.rs:45-53). In gpui-kit's group, the group `disabled` flag overwrites each radio's own flag (G/radio.rs:409).
  - Only gpui-kit:
    - screen-reader name override (G/radio.rs:75-82)
    - tooltip (G/radio.rs:63-67)
    - sizes (G/radio.rs:126-131)
    - child content (G/radio.rs:158-162, 246-248)
    - tab index and tab stop (G/radio.rs:97-107)
    - styles per state (B/radio.rs:38-43, 101-105)
    - radios made directly from strings (G/radio.rs:367-369)
  - Its mark reuses the checkbox's spring fade (G/radio.rs:242-244; G/checkbox.rs:182-191). That fade and the tooltip do not carry over to a terminal.
- Builder API:
  - ours:
    - `builder::radio_button()` has `value`, `label`, `checked`, `disabled`, `group`, `class` and `build` (src/builder/specialized.rs:233-235, 246-296). It has no change callback, and `NamedRadioProps` is `pub(crate)` (src/widgets/input/named_radio.rs:56-63).
    - `widgets::RadioButtonBuilder<T>` has `option`, `disabled_option`, `options`, `selected`, `disabled`, `orientation`, `build` and `render` (src/widgets/input/radio_button.rs:29-31, 45-92).
    - `RadioButton::with_on_change(Fn(T))` is on the component (src/widgets/input/radio_button.rs:162-168).
  - gpui-kit:
    - `Radio::new(id)` has `tooltip`, `label`, `accessibility_label`, `checked`, `disabled`, `tab_index`, `tab_stop` and `on_change(&bool)` (G/radio.rs:39-124).
    - `RadioGroup` has `new`, `vertical`, `horizontal`, `layout`, `on_change(&usize)`, `selected_index`, `disabled`, `child` and `children` (G/radio.rs:290-358).
    - The base adds `styles`, `track_focus` and `set_position` (B/radio.rs:101-105, 136-151), and `RadioGroup::axis` (B/radio_group.rs:29-33).
- States:
  - ours: `RadioButton` keeps `selected` in props and `focused_index` and `hover_index` in state (src/widgets/input/radio_button.rs:118-127, 146-153). `NamedRadio` keeps `focused` and the shared selected flag (src/widgets/input/named_radio.rs:70-79).
  - gpui-kit: checked, disabled, focus from a keyed focus handle (G/radio.rs:174-178), and position and size of set (G/radio.rs:34-35).
- Keyboard and screen reader:
  - ours, `RadioButton`:
    - Up/Left and Down/Right move focus, wrap around, and skip disabled options. They do not change the selection (src/widgets/input/radio_button.rs:349-362).
    - Space or Enter selects the focused option (src/widgets/input/radio_button.rs:363-365).
    - A left click selects the option under the pointer (src/widgets/input/radio_button.rs:410-417).
    - Screen reader: the group is role RadioGroup with no orientation (src/widgets/input/radio_button.rs:211). Each option is role RadioButton with label, toggled, and disabled or Click, plus a focus event so a screen reader can move to one option (src/widgets/input/radio_button.rs:214-239).
  - ours, `NamedRadio`:
    - Space, Enter or a left click selects (src/widgets/input/named_radio.rs:165-178). Each radio is its own focus stop (src/widgets/input/named_radio.rs:145).
    - Screen reader: role RadioButton with label, toggled, and disabled or Click (src/widgets/input/named_radio.rs:124-136).
  - gpui-kit:
    - Enter and Space activate an unchecked radio (B/radio.rs:351-356, test).
    - A checked or disabled radio does nothing and offers no Click action (B/radio.rs:227-234, 373-378, 412).
    - There are no arrow keys: grep for `key_context|on_action|KeyBinding|arrow` in G/radio.rs, B/radio.rs and B/radio_group.rs found none. Each radio is its own Tab stop (G/radio.rs:52-53; B/radio.rs:220-226).
    - Screen reader: role RadioButton, toggled and also selected (B/radio.rs:203-212), plus label, position in set and set size (B/radio.rs:213-219). The group is role RadioGroup with orientation (B/radio_group.rs:58-63).
- Worth adopting:
  - Set orientation on our RadioGroup node (B/radio_group.rs:58-63); the screen reader can then tell the user which arrow keys move between options; small.
  - Mark the chosen radio as selected as well as toggled (B/radio.rs:209-212); some screen readers read only one of the two; small.
  - An `accessibility_label` per radio, for short visible labels; small.
  - A change callback on the builders (like `RadioGroup::on_change`, G/radio.rs:325-334); today `builder::radio_button()` has no way to report a change at all; small.

### scroll
- Files: ours src/widgets/layout/scroll_view.rs, src/widgets/layout/scroll_view/motion.rs; gpui-kit G/scroll/mod.rs, G/scroll/scrollable.rs, B/scrollbar.rs, B/auto_scroll.rs, B/scroll_bounce.rs, B/scrollable_mask.rs.
- Structure:
  - gpui-kit: a trait, `ScrollableElement`, adds scrollbars to any `Div` or `Stateful` element (G/scroll/scrollable.rs:12-63, 189-195).
  - gpui-kit, wrapper: `Scrollable` keeps the caller's element as the scroll area. It stores a `ScrollHandle` in window state, keyed by call site, and lays a base `Scrollbar` over the area (G/scroll/scrollable.rs:65-94, 128-187, 213-225, 265-290). The scrollbar works with any `ScrollbarHandle` (B/scrollbar.rs:68-145).
  - ours: `ScrollViewBuilder` builds `ScrollViewProps`, and `render()` returns `Element::typed::<ScrollView>` (src/widgets/layout/scroll_view.rs:11-112).
  - ours, component: it keeps `ScrollViewState` (src/widgets/layout/scroll_view.rs:162-175). It measures its content with a layout callback (src/widgets/layout/scroll_view.rs:283-293) and moves the content with absolute offsets (src/widgets/layout/scroll_view.rs:259-265). The scrollbars are text columns of █ and ░ (src/widgets/layout/scroll_view.rs:306-341, 455-470).
- Features and options:
  - Both sides: vertical, horizontal or both axes (src/widgets/layout/scroll_view.rs:15-16, 45-55; G/scroll/scrollable.rs:31-62), and wheel scrolling.
  - Only ours:
    - A switch to hide the scrollbars (src/widgets/layout/scroll_view.rs:76-80).
    - Smooth scrolling: 120 ms, linear (src/widgets/layout/scroll_view.rs:82-86; src/widgets/layout/scroll_view/motion.rs:10, 37-68, 84-94).
    - A wheel speed setting (src/widgets/layout/scroll_view.rs:88-92, 443-453).
    - Fixed viewport sizes as props (src/widgets/layout/scroll_view.rs:57-74).
  - Only gpui-kit:
    - Scrollbar modes: Scrolling (fades after idle), Hover and Always (B/scrollbar.rs:46-56).
    - Click on the track to jump, and drag the thumb (B/scrollbar.rs:1716-1750, 1788-1835).
    - Style and motion settings (B/scrollbar.rs:271-370, 641-760).
    - A cap on the update rate while dragging (B/scrollbar.rs:823-827, 929-938).
    - One scrollbar for plain, uniform-list and list handles (B/scrollbar.rs:84-145).
    - `AutoScroll`, which scrolls while a drag is near an edge (B/auto_scroll.rs:6-17, 35-59).
    - `ScrollableMask`, which consumes only its own axis and hands vertical wheel events to the parent at the edge (B/scrollable_mask.rs:56-76).
    - `ScrollBounce`, a touch overscroll (B/scroll_bounce.rs:84-118).
  - Terminal note: the bounce, the fade and the hover-only mode do not carry over to a terminal.
- Builder API:
  - gpui-kit, trait: `.overflow_scrollbar()`, `.overflow_x_scrollbar()`, `.overflow_y_scrollbar()`, `.scrollbar(handle, axis)`, `.vertical_scrollbar(handle)` and `.horizontal_scrollbar(handle)` (G/scroll/scrollable.rs:12-63). `Scrollable::id` (G/scroll/scrollable.rs:86-93).
  - gpui-kit, scrollbar: `Scrollbar::new`, `::horizontal` and `::vertical`, then `.id`, `.mode`, `.scroll_size`, `.viewport_bounds`, `.viewport_from_layout`, `.axis`, `.styles` and `.max_fps` (B/scrollbar.rs:836-938).
  - ours: `ScrollViewBuilder::new(content)`, `.content`, `.scroll_x`, `.scroll_y`, `.viewport_width`, `.viewport_height`, `.viewport_size`, `.show_scrollbars`, `.smooth_scroll`, `.scroll_speed`, `.build` and `.render` (src/widgets/layout/scroll_view.rs:24-112).
  - ours, gap: `ScrollViewProps` has no offset field, so app code cannot set the scroll position (src/widgets/layout/scroll_view.rs:120-139).
- States:
  - gpui-kit: the offset lives in the handle (B/scrollbar.rs:68-100). The scrollbar keeps the hovered axis, the hovered thumb, the dragged axis and drag position, the last scroll time, and the visibility and width animations (B/scrollbar.rs:148-168).
  - ours: scroll_x, scroll_y, content width and height, and focus (src/widgets/layout/scroll_view.rs:162-175). The animation keeps from, target and start time (src/widgets/layout/scroll_view/motion.rs:12-17).
- Keyboard and screen reader:
  - ours, keys: the arrows move by one cell. PageUp and PageDown move by the viewport height. Home goes to the top-left and End to the bottom. Keys work only while the view has focus (src/widgets/layout/scroll_view.rs:378-411). The wheel scrolls, with Shift for horizontal (src/widgets/layout/scroll_view.rs:413-429).
  - ours, screen reader: the root has the ScrollView role (src/widgets/layout/scroll_view.rs:357-359). The bar has no role, and no scroll values are published; a search for `set_scroll_|ScrollBar` in src found no widget use.
  - gpui-kit: none found. `grep role|aria|accesskit|keybinding|focus` gives 0 matches in B/scrollbar.rs, B/auto_scroll.rs and B/scrollable_mask.rs. The only matches elsewhere are a focus pass-through (G/scroll/scrollable.rs:122-125) and a comment that keyboard and focus belong to the child (B/scroll_bounce.rs:93-94).
- Worth adopting:
  - Mouse on our text scrollbar: click the track to jump, and drag the thumb (B/scrollbar.rs:1716-1750, 1788-1835). Terminals report press and drag by cell. Our handler ignores the mouse except for the wheel (src/widgets/layout/scroll_view.rs:369-431). Size: medium.
  - Scroll chaining: when an inner view is already at its edge, pass the wheel event to the parent (B/scrollable_mask.rs:71-75). Ours always returns Consumed (src/widgets/layout/scroll_view.rs:413-433). Size: small.
  - Scroll near the edge during a drag (B/auto_scroll.rs:35-59). Our tree's drag only sets a drop target (src/widgets/display/tree/live.rs:819-825), so a drop target below the visible rows cannot be reached. Size: small.
  - A way for app code to read or set the offset, like `ScrollbarHandle::set_offset` (B/scrollbar.rs:68-75, 93-95), for example to jump to a search hit or to the end of a log (src/widgets/layout/scroll_view.rs:120-139). Size: small.

### select
- Files: ours src/widgets/input/select.rs, src/builder/widgets/input.rs; gpui-kit G/select.rs, G/searchable_list/state.rs, G/searchable_list/delegate.rs, G/list/list.rs, G/combobox.rs (shared state only), B/select.rs.
- Structure:
  - gpui-kit:
    - `SelectState<D>` is an `Entity` that holds a `SearchableListState` (G/select.rs:117-129). That state holds a `List` entity, the selection and the open flag (G/searchable_list/state.rs:21-40).
    - The combobox uses the same `SearchableListState` (G/searchable_list/state.rs:12-17; G/combobox.rs:113). Multi-select lives in the combobox, not the select (G/combobox.rs:116). The select always replaces its one selection (G/select.rs:175-186).
    - Items come from a `SearchableListDelegate` (G/searchable_list/delegate.rs:53-55).
    - `Select` is a `RenderOnce`. Each render it copies its options into the state and wraps them in the base `Select` root (G/select.rs:803-812, 834-852).
  - ours: `Select<T>` is a live component. It draws a header row and, when open, the option rows inline below it in one flex column (src/widgets/input/select.rs:267-304). `builder::select()` wraps it in `ConfiguredSelect`, which adds multi-select (src/builder/widgets/input.rs:559-564, 626-635).
- Features and options:
  - Both sides have placeholder, disabled, disabled options and a selected value (src/widgets/input/select.rs:111-113, 145-160; G/select.rs:669-673, 710-714; G/searchable_list/delegate.rs:47-50).
  - Only ours:
    - multi-select with `[✓]` rows through `builder::select().multiple(true)` (src/builder/widgets/input.rs:500-507; src/widgets/input/select.rs:333-335)
    - type-ahead: typing jumps to the first label that starts with the typed text, and the typed text resets after 1 second (src/widgets/input/select.rs:698-705, 721-723)
    - fixed width, and a limit on visible rows that also fits the space left on screen (src/widgets/input/select.rs:68-78, 394-402)
    - mouse wheel scrolling (src/widgets/input/select.rs:792-794)
    - open and close callbacks (src/widgets/input/select.rs:430-431, 442-443)
  - Only gpui-kit:
    - a search box inside the popup (G/select.rs:297-303, 704-708), with custom matching and async search (G/searchable_list/delegate.rs:40-45, 82-88)
    - grouped items with section headers (G/select.rs:28; G/searchable_list/delegate.rs:53-60, 118-125)
    - a clear button (G/select.rs:438-442, 577-585, 698-702)
    - a title prefix (G/select.rs:690-696)
    - custom icon, empty-state view, menu width and max height, and trigger border on or off (G/select.rs:657-667, 684-688, 716-731)
    - custom row and trigger drawing (G/searchable_list/delegate.rs:18-35, 102-111)
    - a hook that can change or veto a selection before it commits (G/searchable_list/delegate.rs:155-166)
  - gpui-kit's popup is a deferred overlay (G/select.rs:601-604). Ours draws the list inside its own box.
- Builder API:
  - ours:
    - `builder::select()` (src/builder/widgets/input.rs:66-68) has `option(value, label)`, `options`, `selected`, `placeholder`, `disabled`, `multiple`, `class` and `build` (src/builder/widgets/input.rs:452-526).
    - `widgets::SelectBuilder<T>` has `options`, `option`, `add_option`, `selected`, `placeholder`, `disabled`, `width`, `max_visible_items`, `build` and `render` (src/widgets/input/select.rs:19-21, 68-78). `SelectOption::new(..).disabled(..)` marks one option (src/widgets/input/select.rs:124-125, 139-142).
    - `with_on_change`, `with_on_open` and `with_on_close` are on the component only (src/widgets/input/select.rs:418-419, 430-431, 442-443).
  - gpui-kit:
    - `SelectState::new(delegate, selected_index, window, cx)` (G/select.rs:149-154) has `searchable`, `set_selected_index`, `set_selected_value`, `selected_index` and `selected_value` (G/select.rs:297-303, 334-335, 363-371).
    - `Select::new(&state)` (G/select.rs:642-649) has `menu_width`, `menu_max_h`, `placeholder`, `accessibility_label`, `icon`, `title_prefix`, `cleanable`, `search_placeholder`, `disabled`, `empty`, `appearance` (G/select.rs:657-731) and `with_size` (G/select.rs:734-743).
    - Changes arrive as `SelectEvent::Confirm` (G/select.rs:69-75, 201).
- States:
  - ours: `is_open`, `highlighted_index`, `scroll_offset`, `is_focused` (src/widgets/input/select.rs:183-192), plus the type-ahead buffer and its time (src/widgets/input/select.rs:195-203). `ConfiguredSelect` keeps the multi-select list (src/builder/widgets/input.rs:559-564).
  - gpui-kit: open, selection, trigger bounds, disabled and the display options (G/searchable_list/state.rs:21-40). The cursor is the list's selected index (G/select.rs:363-366), and the search text is in the list's query input (G/searchable_list/state.rs:166-171).
- Keyboard and screen reader:
  - ours, keys:
    - Enter or Space opens the list, or chooses the highlighted option (src/widgets/input/select.rs:647-653).
    - Escape closes it (src/widgets/input/select.rs:654).
    - Up and Down move and skip disabled options, or open the list when closed (src/widgets/input/select.rs:655-669).
    - Home, End, PageUp and PageDown work while open (src/widgets/input/select.rs:670-672).
    - Printable keys jump by prefix (src/widgets/input/select.rs:698-705).
    - Losing focus closes the list (src/widgets/input/select.rs:579-582).
  - ours, mouse: clicking the header toggles the list, clicking a row chooses it, and moving the pointer highlights a row (src/widgets/input/select.rs:772-791, 829-837).
  - ours, screen reader:
    - The control is role ComboBox with value, expanded, multiselectable, and disabled or Click (src/widgets/input/select.rs:267-278). It has no name.
    - The Click action opens or closes the list through a custom event (src/widgets/input/select.rs:288-294, 541-548).
    - The list is role ListBox, and each row is ListBoxOption with label and selected (src/widgets/input/select.rs:296-304, 342-350). The highlighted row gets screen-reader focus (src/widgets/input/select.rs:355-362).
  - gpui-kit, keys:
    - The "Select" context binds up, down, enter, secondary-enter and escape (B/select.rs:17-27).
    - Up, Down or Enter on a closed select opens it and moves focus into the list (B/select.rs:230-243, 272-293).
    - Escape closes it and returns focus to the trigger (B/select.rs:178-193, 294-302).
    - The list binds the same keys (G/list/list.rs:26-35). Grep of G/list, G/select.rs, B/select.rs and G/searchable_list found no Home, End, PageUp or PageDown bindings.
    - Blur, or a click outside, closes the popup (G/select.rs:378-388, 625-627).
  - gpui-kit, screen reader:
    - The control is role ComboBox with expanded, label and value (B/select.rs:195-203). Its Click action opens or closes it (B/select.rs:216-228).
    - Rows are role ListItem with position, set size, and selected on the cursor row (G/list/list.rs:481, 488-495), inside role List (G/list/list.rs:788-790).
    - The base asks the app to mark the active option with `aria_active_descendant` (B/select.rs:40-42). Grep found no call to it in G/select.rs, G/list or G/searchable_list.
- Worth adopting:
  - A screen-reader name for the select, separate from its value (G/select.rs:675-682); today the node has only a value (src/widgets/input/select.rs:268-269), so a user hears "Canada" but not "Country"; small.
  - A clear action, like `cleanable`; in a terminal this could be a key such as Delete, plus an optional "x" cell for mouse users; small.
  - A filter row at the top of the open list; our prefix jump does not help with long lists or with matches in the middle of a label; medium.
  - Grouped options with header rows, drawn as rows you cannot select; medium.

### slider
- Files: ours src/widgets/input/slider.rs, src/builder/specialized.rs, src/builder/widgets/input.rs; gpui-kit G/slider.rs, B/slider.rs.
- Structure:
  - gpui-kit: `SliderState` is an `Entity` that holds min, max, step, value, scale and drag state, and emits `SliderEvent` (B/slider.rs:169-183, 779). `Slider` is a `RenderOnce` that builds the base parts: `Slider`, `SliderTrack`, `SliderIndicator` and `SliderThumb` (G/slider.rs:84-92, 261-263; B/slider.rs:413-417).
  - ours: `Slider` is a live component that draws one text element of track glyphs (src/widgets/input/slider.rs:379-391, 448-471). It changes `props.value` in its key and mouse handlers (src/widgets/input/slider.rs:311-318). `builder::slider()` wraps it in `ConfiguredSlider`, which adds a label (src/builder/specialized.rs:409-418).
- Features and options:
  - Both sides have min, max, step, value, disabled, and horizontal or vertical (src/widgets/input/slider.rs:140-161; B/slider.rs:169-183; G/slider.rs:106-122).
  - Only ours:
    - show the value, show min and max labels, and set the track width (src/widgets/input/slider.rs:81-103)
    - a text label that the screen reader also reads (src/builder/specialized.rs:363-367; src/widgets/input/slider.rs:448-457)
    - a bad range draws "Invalid slider range" (src/widgets/input/slider.rs:204-212, 379-382)
  - Only gpui-kit:
    - a range value with two thumbs (B/slider.rs:22-32; G/slider.rs:196-198, 305-308)
    - a logarithmic scale (B/slider.rs:123-130, 294-297)
    - a reversed fill (G/slider.rs:124-136)
    - separate Change and Release events (B/slider.rs:14-20, 383-392)
    - an animated ring around a hovered thumb (G/slider.rs:14-31), which does not carry over to a terminal
- Builder API:
  - ours:
    - `builder::slider()` (src/builder/widgets/input.rs:73-75) has `value`, `min`, `max`, `step`, `label`, `disabled`, `class` and `build` (src/builder/specialized.rs:339-389).
    - `widgets::SliderBuilder` has `min`, `max`, `value`, `step`, `range`, `orientation`, `show_value`, `show_labels`, `disabled`, `width`, `build` and `render` (src/widgets/input/slider.rs:38-40, 81-103, 124-127).
    - The two builders have different options: only the first has `label`, and only the second has orientation and the display switches.
    - `Slider::with_on_change(Fn(f64))` is on the component (src/widgets/input/slider.rs:219-224).
  - gpui-kit: `SliderState::new()` has `min`, `max`, `step`, `scale`, `default_value`, `set_value` and `value` (B/slider.rs:185-201, 253-275). `Slider::new(&state)` has `horizontal`, `vertical`, `disabled` and `reverse` (G/slider.rs:94-97, 106-136).
- States:
  - ours: `is_focused`, `is_dragging`, `is_hover` (src/widgets/input/slider.rs:185-194). They are cleared when the slider is disabled or loses focus (src/widgets/input/slider.rs:361-366, 487-493).
  - gpui-kit: single or range value, fill percentage, bounds, scale and dragging (B/slider.rs:169-183), plus hovered and pressed per thumb for the ring (G/slider.rs:27-31). It has no focus state.
- Keyboard and screen reader:
  - ours:
    - Left or Down lowers the value by one step, and Right or Up raises it by one step. PageDown and PageUp move 10% of the range, at least one step. Home and End jump to min and max (src/widgets/input/slider.rs:514-529).
    - A left click sets the value, and dragging moves it (src/widgets/input/slider.rs:541-559).
    - Screen reader: role Slider, label, value, min, max, step and orientation (src/widgets/input/slider.rs:453-465).
    - The App acts only on Focus and Click requests from a screen reader (src/app.rs:411-417). Increment, decrement and set-value requests do nothing.
  - gpui-kit:
    - No keyboard or focus: grep of G/slider.rs and B/slider.rs for `key_context|KeyBinding|on_key|track_focus|focus_handle|on_action` found none.
    - Pointer: press on the track, or drag a thumb (B/slider.rs:583-588, 613-618, 754-760).
    - Screen reader: role Slider, value (the end value), min, max, step and orientation. Increment and Decrement actions move one step (B/slider.rs:479-508).
    - These actions pass one number to `set_value`, which turns a range slider into a single value. `set_value` also emits no Change event (B/slider.rs:43-47, 260-270).
- Worth adopting:
  - Handle screen-reader Increment, Decrement and SetValue for sliders, as B/slider.rs:489-508 does for the first two; today a screen-reader user can hear the value but not change it that way (src/app.rs:411-417); medium, because it changes how the App dispatches accessibility actions.
  - A range value with two thumbs, for example a time window or a price range, drawn as two markers with a key to switch thumbs; medium.
  - A logarithmic scale for volume, frequency or zoom; small.
  - A release event after a drag, so expensive work runs once instead of on every cell the thumb crosses (ours calls on_change on every change, src/widgets/input/slider.rs:311-318); small.

### tab (tabs)
- Files: ours src/widgets/layout/tabs.rs, src/builder/widgets/layout.rs (a second, smaller `TabsBuilder`); gpui-kit G/tab/mod.rs, G/tab/tab.rs, G/tab/tab_bar.rs, B/tabs.rs.
- Structure:
  - gpui-kit: `TabBar` is a RenderOnce struct of `Tab` items (G/tab/tab_bar.rs:38-55; G/tab/tab.rs:395-419). Each `Tab` wraps the base `Tab`, and the bar wraps the base `Tabs` (G/tab/tab.rs:399; G/tab/tab_bar.rs:42; B/tabs.rs:22-35, 190-195).
  - gpui-kit, ownership: the caller owns the selection through `selected_index` (G/tab/tab_bar.rs:156-160), and the caller draws the panel content. The bar keeps only the indicator animation state, in window keyed state (G/tab/tab_bar.rs:202-209).
  - ours: `TabsBuilder` builds `TabsProps`, and `render()` wraps them in `Element::typed::<Tabs>` (src/widgets/layout/tabs.rs:171-284). `Tabs` is itself the live component. It keeps the authored and live props, tab identities taken from content keys, a set of closed tabs and click targets (src/widgets/layout/tabs.rs:368-399, 401-442). It renders the headers and the panels (src/widgets/layout/tabs.rs:655-791).
- Features and options:
  - Both sides: variants, sizes, disabled tabs, icons and a change callback (src/widgets/layout/tabs.rs:19-43, 109-124, 443-447; G/tab/tab.rs:12-21, 498-559, 612-617).
  - Only ours:
    - Top, Bottom, Left or Right position, and vertical orientation (src/widgets/layout/tabs.rs:10-17, 45-56, 660-661, 765-769).
    - Built-in panels, with lazy loading of inactive panels (src/widgets/layout/tabs.rs:322-323, 739-760).
    - Closable tabs with a close button. Removal is done by the widget or by the parent (src/widgets/layout/tabs.rs:116-117, 449-453, 500-532, 681-697).
    - Badges in five variants (src/widgets/layout/tabs.rs:67-105, 554-563) and tooltips (src/widgets/layout/tabs.rs:122-123, 770-777).
    - Automatic or manual activation from the keyboard (src/widgets/layout/tabs.rs:58-65, 894-899).
    - Keyed identity that keeps the chosen tab when tabs are reordered (src/widgets/layout/tabs.rs:381-442, 937-990).
  - Only gpui-kit:
    - An overflow menu that lists every tab (G/tab/tab_bar.rs:109-113, 554-585).
    - `max_width` with an ellipsis (G/tab/tab_bar.rs:115-121; G/tab/tab.rs:730-742).
    - A horizontally scrolling header row with a tracked handle (G/tab/tab_bar.rs:123-130, 535-551).
    - Prefix and suffix elements on the bar and on each tab (G/tab/tab_bar.rs:132-142; G/tab/tab.rs:534-544).
    - Icon-only tabs (G/tab/tab.rs:719-727).
    - A sliding spring indicator and a text color fade (G/tab/tab_bar.rs:179, 223-240; G/tab/tab.rs:688-698, 750-757). The slide, the shadows and the rounded pill shapes (G/tab/tab.rs:346-356, 747) do not carry over to terminal cells.
  - Ours clips tabs that do not fit. A search for `scroll|overflow|ellipsis|truncat|max_width` in src/widgets/layout/tabs.rs finds only `overflow-hidden` classes (src/widgets/layout/tabs.rs:763, 776, 781, 783).
- Builder API:
  - gpui-kit, bar: `TabBar::new(id)`, the variants (`.with_variant`, `.pill`, `.outline`, `.segmented`, `.underline`), `.menu`, `.max_width`, `.track_scroll`, `.prefix`, `.suffix`, `.child`, `.children`, `.selected_index`, `.last_empty_space` and `.on_click(|ix, ..| ..)` (G/tab/tab_bar.rs:57-177).
  - gpui-kit, tab: `Tab::new()`, `.label`, `.aria_label`, `.icon`, the variants, `.prefix`, `.suffix`, `.disabled` and `.on_click` (G/tab/tab.rs:476-559). The base `Tab::set_position(pos, size)` sets the position in the set (B/tabs.rs:75-81).
  - ours, main builder: `TabsBuilder::new`, `.tab`, `.tabs`, `.add_tab`, `.active_tab`, `.orientation`, `.variant`, `.size`, `.position`, `.closable`, `.disabled`, `.lazy_loading`, `.keyboard_activation`, `.build` and `.render` (src/widgets/layout/tabs.rs:186-284).
  - ours, tab: `Tab::new(label, content)` with `.disabled`, `.closable`, `.with_icon`, `.with_badge` and `.with_tooltip` (src/widgets/layout/tabs.rs:126-169).
  - ours, smaller builder: `tabs()` with `.tab`, `.active`, `.closable`, `.class` and `.build` (src/builder/widgets/layout.rs:38-40, 65-138).
  - ours, gap: the change and close callbacks exist only as `Tabs::with_on_change` and `Tabs::with_on_close` (src/widgets/layout/tabs.rs:443-453). They are reached through `Element::typed_with` (src/component/element.rs:261-279). Neither builder offers them.
- States:
  - gpui-kit: selected and disabled per tab (G/tab/tab.rs:409-410), plus the indicator epoch and bounds (G/tab/tab.rs:411-416; G/tab/tab_bar.rs:202-209, 412-425). Hover is only a style (G/tab/tab.rs:818-833).
  - ours: the active tab (src/widgets/layout/tabs.rs:308-309); the focused tab, the hovered tab and widget focus (src/widgets/layout/tabs.rs:351-360); the closed tabs (src/widgets/layout/tabs.rs:373); disabled per tab and for the whole widget (src/widgets/layout/tabs.rs:114-115, 320-321, 789).
- Keyboard and screen reader:
  - ours, keys: Left and Up go to the previous tab; Right and Down go to the next. Both skip disabled tabs and wrap (src/widgets/layout/tabs.rs:464-484). Home and End jump. Enter and Space activate. Delete or `x` closes. The keys 1 to 9 jump to a tab (src/widgets/layout/tabs.rs:865-901).
  - ours, mouse: click on a tab, click on the close button, and hover are handled (src/widgets/layout/tabs.rs:902-926).
  - ours, screen reader: the header row is a TabList labelled "Tabs" (src/widgets/layout/tabs.rs:729-738). Each header is a Tab with a label, selected state, and a click action or a disabled flag (src/widgets/layout/tabs.rs:698-711). The close control is a Button labelled "Close <label> tab" (src/widgets/layout/tabs.rs:681-697). Each panel is a TabPanel labelled "<label> panel" (src/widgets/layout/tabs.rs:739-760). The root is a Group (src/widgets/layout/tabs.rs:787). Focus and activate events are wired (src/widgets/layout/tabs.rs:712-726, 837-864).
  - gpui-kit, keys: none. The base file says tabs do not take keyboard focus. It lists roving focus, arrow keys, Home/End and Enter/Space as a TODO (B/tabs.rs:15-21). I searched `keybinding|on_action|on_key|focus` in G/tab/tab.rs, G/tab/tab_bar.rs and B/tabs.rs; only that comment matched.
  - gpui-kit, screen reader: each tab is a Tab with an optional aria-label, aria-selected, and an optional position-in-set and size-of-set (B/tabs.rs:149-182). The root is a TabList (B/tabs.rs:227-233). The styled `TabBar` never calls `set_position`; `grep set_position` in G/ finds only G/radio.rs:208.
- Worth adopting:
  - When the tabs do not fit, scroll the header row so the focused tab stays in view, and optionally offer a menu that lists all tabs (G/tab/tab_bar.rs:109-130, 535-585). Today a narrow terminal clips tabs, and the clipped tabs cannot be seen (src/widgets/layout/tabs.rs:729-738, 778-784). Size: medium.
  - A per-tab `max_width` with an ellipsis (G/tab/tab_bar.rs:115-121; G/tab/tab.rs:730-742). It keeps one long label from pushing the others off screen. Size: small.
  - Position in set on each Tab node, so the screen reader says "tab 2 of 5" (B/tabs.rs:75-81, 169-172). Ours sets only label, selected and click (src/widgets/layout/tabs.rs:698-711). Size: small.
  - `on_change` and `on_close` on the builders, like `TabBar::on_click` (G/tab/tab_bar.rs:168-177). Developers would no longer need `Element::typed_with` for these callbacks (src/widgets/layout/tabs.rs:443-453). Size: small.

### table
- **Files:** ours src/widgets/display/table.rs, src/widgets/display/table/live.rs, src/widgets/display/table/border.rs, src/widgets/display/data_table.rs, src/widgets/display/data_table/live.rs, src/widgets/display/data_table/filters.rs, src/builder/widgets/table.rs; gpui-kit G/table/table.rs, G/table/data_table.rs, G/table/state.rs, G/table/delegate.rs, G/table/column.rs, G/table/loading.rs, B/table.rs.
- **Structure:**
  - gpui-kit has two tables.
    - `Table` has no state. It is a tree of RenderOnce parts (TableHeader, TableBody, TableFooter, TableRow, TableHead, TableCell, TableCaption) put together with `.child()` (G/table/table.rs:40-122, 141-352, 354-668). Each part wraps an unstyled semantic part from the base crate (B/table.rs:10-65, 67-152, 153-281).
    - `DataTable` is a RenderOnce shell over an `Entity<TableState<D>>` (G/table/data_table.rs:90-106, 141-174). The app supplies columns, row counts and cell elements through the `TableDelegate` trait (G/table/delegate.rs:16-118). `TableState` keeps selection, widths and scroll (G/table/state.rs:190-260).
  - Ours:
    - `Table` is a Component with `TableProps` and `TableState` (src/widgets/display/table.rs:53-99, 212-238, 536-616). Its render hands both to a private `LiveTable`. `LiveTable` measures the layout, builds only the visible rows, and handles keys and the mouse (src/widgets/display/table/live.rs:47-58, 81-128, 369-566).
    - `DataTable` adds filters, pagination and virtual scroll on top of `TableProps` (src/widgets/display/data_table.rs:134-183). Its `LiveDataTable` keeps a shared `Model` in an `Arc<Mutex<_>>` and passes a filtered, sorted slice to the live table (src/widgets/display/data_table/live.rs:31-49, 367-551, 554-566).
- **Features and options:**
  - **Only ours:**
    - Width modes Fixed, Percent, Auto and Flex (src/widgets/display/table.rs:302-341).
    - Multi-row selection (src/widgets/display/table.rs:414-430, 441-497; src/widgets/display/table/live.rs:701-705).
    - Per-row and per-cell styles and alignment, and clickable cells that carry an action string (src/widgets/display/table.rs:122-150, 848-852; src/widgets/display/table/live.rs:706-716).
    - Rows that cannot be selected (src/widgets/display/table/live.rs:697-699).
    - Bad settings are shown as red text (src/widgets/display/table.rs:14-50; src/widgets/display/table/live.rs:370-373).
    - DataTable adds:
      - global search;
      - five filter types typed as text (src/widgets/display/data_table.rs:19-32; src/widgets/display/data_table/filters.rs:15-43);
      - multi-column sort with a priority number in the header (src/widgets/display/data_table/live.rs:391-403, 489-515);
      - Prev/Next pages and a show/hide column panel (src/widgets/display/data_table/live.rs:279-365);
      - CSV and JSON buttons. These only pass the word "csv" or "json" to `on_export`; the widget builds no file (src/widgets/display/data_table/live.rs:248-259).
  - **Only gpui-kit:**
    - Column spans, a footer and a caption (G/table/table.rs:281-352, 449-453, 608-668).
    - Columns pinned on the left, drag-to-reorder columns, and multi-level group headers (G/table/column.rs:29-34, 51-65, 156-178; G/table/delegate.rs:47-74, 120-130).
    - Row, column and cell selection modes (G/table/state.rs:35-57, 206-227).
    - Right-click context menu and double-click events (G/table/delegate.rs:100-109; G/table/state.rs:61-108, 719-818).
    - An empty view, a skeleton loading view, and loading more rows near the end (G/table/delegate.rs:132-185; G/table/state.rs:1256-1277, 2396-2415).
    - A text dump for export (G/table/delegate.rs:224-230; G/table/state.rs:582-626).
  - **Sorting differs:**
    - gpui-kit hands sorting to the app's `perform_sort`. Its cycle is unsorted, then descending, then ascending, then unsorted (G/table/delegate.rs:28-36; G/table/state.rs:1162-1192).
    - Ours sorts inside the widget with a plain string compare, so "10" sorts before "9" (src/widgets/display/table.rs:343-367; src/widgets/display/data_table/live.rs:144-162). It only flips between ascending and descending (src/widgets/display/table/live.rs:225).
  - **Terminal notes:**
    - gpui-kit's drag preview uses a shadow and opacity (G/table/column.rs:273-289). Rows change color on hover (G/table/state.rs:1982-1988), and the border is rounded (G/table/data_table.rs:168-172). None of this carries over.
    - Our widths are counted in terminal cells (src/widgets/display/table.rs:512). Yet `TableColumn::new` sets `min_width` to 50 (src/widgets/display/table.rs:661) and the builder's `column` sets 100 (src/builder/widgets/table.rs:61).
    - The virtual-scroll defaults are row_height 32 and viewport_height 400, and the docs call them pixels (src/widgets/display/data_table.rs:96-97, 352-358). These look like leftovers from pixel units.
- **Builder API:**
  - gpui-kit:
    - `Table::new().small().child(TableHeader::new().child(TableRow::new().child(TableHead::new().child(..))))`, with `.accessibility_label()`, `.col_span()`, `.text_center()` and `.text_right()` (G/table/table.rs:23-39, 50-81, 449-465, 540-556).
    - `DataTable::new(&state).stripe().bordered().scrollbar_visible()` (G/table/data_table.rs:101-128).
    - `TableState::new(delegate, window, cx)` with `.loop_selection()`, `.col_movable()`, `.col_resizable()`, `.sortable()`, `.row_selectable()`, `.col_selectable()`, `.cell_selectable()` and `.row_header()`, plus `scroll_to_row`, `set_selected_row` and `clear_selection` (G/table/state.rs:267, 315-385, 393-397, 463, 562).
    - `Column::new(key, name).width().sortable().fixed_left().resizable().movable().selectable().min_width().max_width()` (G/table/column.rs:88-225).
  - Ours:
    - `Table::new()` and `Table::with_props(TableProps{..})`, plus static helpers `with_columns`, `with_rows`, `selectable` and `sortable` that return `TableProps` (src/widgets/display/table.rs:264-300).
    - `TableColumn::new(title, key).with_width().with_alignment().sortable().resizable()` (src/widgets/display/table.rs:653-712).
    - `TableRow::new(id).with_cell().with_styled_cell().with_data().with_style().selectable()` (src/widgets/display/table.rs:723-796).
    - `TableCell::new().with_style().with_alignment().clickable(action)` (src/widgets/display/table.rs:807-852).
    - `DataTableProps::new(cols, rows).with_pagination().with_virtual_scroll().with_features()` (src/widgets/display/data_table.rs:336-402).
    - `builder::data_table().column().row().simple_row().pagination().virtual_scroll().features().hide_columns().filter().class().build()` (src/builder/widgets/table.rs:12-188).
    - I found no fluent builder for the plain Table. I searched src for `pub fn table` and `TableBuilder`.
- **States:**
  - gpui-kit:
    - `TableState` tracks the selected row, column and cell, the selection mode, the right-clicked row and cell, the column being resized, the column drag gap and the visible range (G/table/state.rs:190-260).
    - Only one row can be selected (G/table/state.rs:240).
    - Rows get a selected flag, a stripe color and a hover color (G/table/state.rs:1959-1988).
    - Loading comes from the delegate (G/table/delegate.rs:146-149).
  - Ours:
    - `TableState` tracks selected_rows, selected_row, selected_column, scroll, column widths, the column being resized, focus, hover_row, visible_rows and the sort (src/widgets/display/table.rs:212-238).
    - `hover_row` is only declared and set to its default; a search finds no other use (src/widgets/display/table.rs:231, 251).
    - `LiveTable` keeps resized widths, drag state and a full-size scroll offset, so tables over 65,535 rows still scroll correctly (src/widgets/display/table/live.rs:47-58, 146-148).
    - DataTable adds loading, an error message, panel-open flags and the search text (src/widgets/display/data_table.rs:215-240).
    - It keeps the selection by row id, so the selection survives filtering (src/widgets/display/data_table/live.rs:31-49, 405-440).
- **Keyboard and screen reader:**
  - gpui-kit keys (G/table/data_table.rs:15-29; G/table/state.rs:824-1109):
    - Escape clears the selection.
    - Up and Down move the row, and wrap when `loop_selection` is on (G/table/state.rs:865-872, 910-920).
    - Left, Right, Tab and Shift-Tab move by column or cell.
    - PageUp and PageDown move by a page.
    - Home and End select the first or last column in row mode, not the first or last row (G/table/state.rs:925-967).
  - gpui-kit screen reader:
    - The simple Table sets Role::Table, an optional label and optional total row and column counts (B/table.rs:91-110, 138-152).
    - Header and body are RowGroup (B/table.rs:153-158).
    - Rows get a 1-based row index, and header and data cells get a 1-based column index (B/table.rs:206-213, 264-281; G/table/table.rs:60-67, 410, 498, 589).
    - DataTable rows only get Role::Row and a selected flag (G/table/state.rs:1971-1973). A search of state.rs for `role` and `aria` found nothing else.
  - Our keys (src/widgets/display/table.rs:369-439):
    - Up and Down wrap. Home, End, PageUp and PageDown also move the cursor, and all of these skip rows that cannot be selected. Shift extends the selection.
    - Enter calls `on_row_action(row, "select")`.
    - With `multi_select`, Space toggles a row and Ctrl+A selects all.
    - Left and Right scroll one cell sideways (src/widgets/display/table/live.rs:609-616).
    - There is no Escape or Tab handling.
  - Our screen reader:
    - Role::Table with no label (src/widgets/display/table/live.rs:565) and Role::ColumnHeader (src/widgets/display/table/live.rs:395).
    - Role::Row and Role::Cell carry selected and disabled state (src/widgets/display/table/live.rs:435-439, 467-471).
    - Focus follows the cursor cell. A screen-reader focus on a cell moves the cursor (src/widgets/display/table/live.rs:451-462, 580-598).
    - The sort order is only " ↑" or " ↓" appended to the header text (src/widgets/display/table/live.rs:386-388).
    - Our `Node` has no row or column index or count setters (src/accessibility/mod.rs:21-82), and only visible rows are built (src/widgets/display/table/live.rs:118-127, 408).
- **Worth adopting:**
  - **Row and column counts and positions, a table name, and the sort direction in the accessibility tree** (B/table.rs:91-110, 206-213, 264-271; G/table/table.rs:60-67); our table builds only the visible rows, so a screen-reader user cannot hear "row 5 of 200" or which column is sorted; medium (new `Node` setters, then use them in live.rs).
  - **Sorting supplied by the column or by the app, with a third "unsorted" step** (G/table/delegate.rs:28-36; G/table/state.rs:1173-1177); our string compare puts numbers and dates in the wrong order (src/widgets/display/table.rs:353-356); small.
  - **Pinned left columns during sideways scroll** (G/table/column.rs:29-30, 162-166; G/table/state.rs:701-710); in a narrow terminal the key column stays visible, whereas ours shifts every column (src/widgets/display/table/live.rs:205-219); medium.
  - **An empty-state message and a load-more callback near the end** (G/table/delegate.rs:132-185; G/table/state.rs:1256-1277); ours shows only the header for zero rows (I searched the table files for "empty", "no rows" and "no data") and needs every row up front (src/widgets/display/table.rs:57-58); medium.

### theme
- **Files:** ours src/theme/mod.rs, src/theme/variables.rs, src/theme/presets.rs, src/theme/colors.rs, src/theme/ansi.rs (used from src/reactive/hooks.rs, src/app.rs, src/layout/colors.rs); gpui-kit G/theme/mod.rs, G/theme/schema.rs, G/theme/registry.rs, G/theme/color.rs, G/theme/theme_color.rs, G/theme/motion.rs, G/theme/mono_font.rs, G/theme/system_font.rs, G/theme/default-theme.json, G/theme/default-colors.json, B/theme.rs, B/theme_tokens.rs, B/state_style.rs.
- **Structure:**
  - gpui-kit:
    - `Theme` is a gpui Global (G/theme/mod.rs:105-171, 193-200). It holds resolved colors, component tokens, a highlight theme, one light and one dark `ThemeConfig`, a mode, fonts, radius, shadow and motion.
    - `Theme::change(mode)` applies the light or dark config. It then pushes a smaller copy to the base crate's Global: appearance, semantic tokens, and scrollbar and resize-handle styles (G/theme/mod.rs:261-347; B/theme.rs:16-37).
  - Ours:
    - `Theme` is a name, a string map of CSS-style variables and an optional parent theme (src/theme/mod.rs:46-85; src/theme/variables.rs:11-45).
    - It lives in one process-wide RwLock. A change counter clears the color caches (src/theme/mod.rs:23-31, 87-111).
    - Components read it with `use_theme()`, and apps set it with `App::set_theme` (src/reactive/hooks.rs:477-481; src/app.rs:145-149).
    - Utility classes and charts resolve names such as `primary` through it (src/theme/mod.rs:113-153; src/layout/colors.rs:356-361).
- **Features and options:** (all of these carry over to a terminal unless marked pixel-only)
  - **Color roles:**
    - gpui-kit has a background color and a matching foreground color for each role: background, surface, primary, secondary, muted, accent, destructive, plus border, input, ring and selection (B/theme_tokens.rs:19-45).
    - Its JSON also has component keys such as `table.head.background`, `table.even.background`, `table.hover.background` and `button.primary.hover.background` (G/theme/schema.rs:270-351, 576-603).
    - Missing keys fall back to derived colors. For example, table colors come from list colors, and selection comes from primary (G/theme/schema.rs:969-1011).
    - Ours has primary, secondary, accent, background, surface, foreground, text-muted, border, success, warning, error, info, five chart colors, bullish and bearish, and a spacing scale in cells (src/theme/presets.rs:5-42). There is no primary-foreground or selection role.
  - **Presets:**
    - Ours ships dark, light, high contrast, Solarized Dark and Gruvbox Dark (src/theme/presets.rs:5, 45, 85, 125, 165).
    - gpui-kit compiles in "Default Light" and "Default Dark" (G/theme/default-theme.json:10-11, 215-216). Its repository also carries 21 theme files for its ThemeRegistry to load, in the themes directory at the gpui-kit root (ayu, catppuccin, gruvbox, solarized, tokyonight and others).
  - **Light and dark:**
    - gpui-kit keeps a light and a dark config and switches by mode or by the window appearance (G/theme/mod.rs:116-119, 227-237, 701-729).
    - Our `Theme` has no mode (src/theme/mod.rs:46-54). Only the DirectTty backend sends an OSC 11 query for the terminal background (src/platform/mod.rs:1182-1203), and its reply parser reads only the DA1, DA2, Kitty graphics and DECRQM answers (src/platform/mod.rs:1277-1301). The default backend sends no query (src/backend/suprtui.rs:157; src/backend/suprtui/output.rs:171).
  - **Files:**
    - gpui-kit parses a JSON `ThemeSet` of named themes with author and URL (G/theme/schema.rs:21-82). It can load a set from a string, and it can watch a folder and reload on change (G/theme/registry.rs:94-161, 185-262).
    - Ours builds themes in code only. A search of src/theme for `serde`, `Deserialize`, `from_str` and `read_to_string` found nothing.
    - Ours has parent inheritance through `extend` (src/theme/mod.rs:72-85).
  - **Color names:**
    - gpui-kit parses Tailwind-style `name-scale/opacity` and two-stop gradients (G/theme/color.rs:686-770).
    - Ours has the Tailwind palette and `get_color` (src/theme/colors.rs:1-7, 535-548), plus conversion to ANSI 16, 256 and true color (src/theme/ansi.rs:7-44, 52-98, 160-185).
  - **Pixel-only in gpui-kit:** fonts, radius tiers, shadows, a focus ring painted outside the border, scrollbar fade timing, motion curves and springs, and gradients (G/theme/mod.rs:59-103, 120-156, 437-501; G/theme/motion.rs:6-43; B/theme_tokens.rs:108-205; G/theme/color.rs:759-770).
- **Builder API:**
  - gpui-kit:
    - `Theme::change(mode, window, cx)`, `Theme::sync_system_appearance`, `Theme::global` and `global_mut` followed by `Theme::sync_base`, and `cx.theme()` (G/theme/mod.rs:44-53, 195-237, 261, 367-372).
    - `ThemeRegistry::load_themes_from_str`, `watch_dir` and `sorted_themes` (G/theme/registry.rs:98, 126-136, 151-161).
    - `Theme::apply_config` (G/theme/schema.rs:1060).
    - `semantic_tokens()` and `color_tokens()` (G/theme/mod.rs:399-435).
    - The `Colorize` methods `lighten`, `darken`, `mix_oklab`, `opacity` and `parse_hex` (G/theme/color.rs:20-60).
  - Ours:
    - `Theme::new(name).with_variables(ThemeVariables::new().set(k, v)).extend(base)` (src/theme/mod.rs:57-76; src/theme/variables.rs:17-39).
    - `Theme::set_active`, `Theme::active` and `App::set_theme` (src/theme/mod.rs:90-106; src/app.rs:147-149).
    - `get_variable`, `resolve_color` and `apply_classes` (src/theme/mod.rs:79-153).
    - Preset functions such as `dark_theme()` (src/theme/presets.rs:5-202).
- **States:**
  - gpui-kit:
    - A light or dark `mode` and `is_dark()` (G/theme/mod.rs:119, 212-216, 701-720).
    - Hover and active colors for each role in the JSON (G/theme/schema.rs:306-316).
    - The base crate layers styles in a fixed order: the instance style, then value states such as checked or selected, then disabled last (B/state_style.rs:27-53).
  - Ours:
    - The theme holds no state.
    - The App keeps a `focus:`, `hover:` or `disabled:` class only while the element is in that state, and applies `sm:` to `xl:` by width (src/app/event_tree.rs:100-130).
    - A parser with no element state ignores those classes (src/layout/css/variants.rs:36-43).
- **Keyboard and screen reader:** none found on either side.
  - Searches: `Role::|aria_|KeyBinding` in G/theme, B/theme.rs, B/theme_tokens.rs and B/state_style.rs; `Role::|KeyCode|accessib` in src/theme.
  - Ours has a high-contrast preset described as being for accessibility (src/theme/presets.rs:84-122). A search of gpui-kit's theme files for `contrast` and `reduced_motion` found nothing.
- **Worth adopting:**
  - **Theme files (JSON or TOML) loaded into `ThemeVariables`, with optional folder watching** (G/theme/registry.rs:151-161, 185-262; G/theme/schema.rs:21-82); users could share and switch themes without recompiling, and our string map already fits (src/theme/variables.rs:11-14); small for loading, medium with watching.
  - **Picking light or dark from the terminal background**: send the OSC 11 query from the default backend too, read its reply (today only DirectTty sends it, src/platform/mod.rs:1182-1203, and nothing parses the reply, src/platform/mod.rs:1277-1301) and choose a light or dark theme, as gpui-kit does from the window appearance (G/theme/mod.rs:227-237, 261-278); text stays readable on light terminals with no setup; medium, with the input-protocols work.
  - **Selection and row-state roles with derived fallbacks** (G/theme/schema.rs:977, 1002-1011); our table's selected row is fixed to `bg-blue fg-white`, its header to `font-bold`, and striping has no default style (src/widgets/display/table.rs:166-167; src/widgets/display/table/live.rs:391), so a theme change does not reach them; small.
  - **A foreground color paired with each fill role** (B/theme_tokens.rs:20-44); ours has one `--color-foreground` (src/theme/presets.rs:14), so text on a primary or error fill has no theme-chosen contrast color; small.

### tree
- Files: ours src/widgets/display/tree.rs, src/widgets/display/tree/live.rs, src/widgets/display/tree/live/paint.rs; gpui-kit G/tree.rs, B/tree.rs.
- Structure:
  - gpui-kit, state: `TreeState` is an Entity that holds flat entries, the selected index, the right-clicked index and a list scroll handle (B/tree.rs:183-192). `TreeItem`s share their expanded and disabled flags through `Rc<RefCell>` (B/tree.rs:34-46, 98-129).
  - gpui-kit, elements: the base `Tree` is a RenderOnce that binds focus and actions (B/tree.rs:466-531). It draws rows through `uniform_list` (B/tree.rs:420-463). The styled `tree(state, render_item)` takes app-drawn `ListItem` rows and adds a context menu and a scrollbar (G/tree.rs:17-23, 71-112).
  - ours: `TreeBuilder` builds `TreeProps` (src/widgets/display/tree.rs:157-411). The `Tree` component renders `LiveTree` (src/widgets/display/tree.rs:1084-1089).
  - ours, live widget: it owns a copy of the root, the flat rows, a cursor, the scroll offset, the set of lazily loaded nodes and a validation error (src/widgets/display/tree/live.rs:49-61, 130-239). Each row is drawn as an absolute one-line element (src/widgets/display/tree/live/paint.rs:114-254).
- Features and options:
  - Both sides: nested items with expanded and disabled state, single selection, and expand/collapse notification (B/tree.rs:91-96, 320-338; src/widgets/display/tree.rs:14-47, 341-344).
  - Only ours:
    - Multi-select, with Shift and Ctrl and Ctrl+A (src/widgets/display/tree.rs:227-230; src/widgets/display/tree/live.rs:293-334, 588-602).
    - Checkboxes (src/widgets/display/tree.rs:25-26, 257-260; src/widgets/display/tree/live/paint.rs:164-197).
    - Search with highlighting and optional filtering (src/widgets/display/tree.rs:269-278; src/widgets/display/tree/live.rs:132-155, 169-171; src/widgets/display/tree/live/paint.rs:129-131).
    - Lazy loading with a loading mark (src/widgets/display/tree.rs:251-254, 365-371; src/widgets/display/tree/live.rs:336-384; src/widgets/display/tree/live/paint.rs:200-204).
    - Drag and drop to move a node (src/widgets/display/tree.rs:263-266; src/widgets/display/tree/live.rs:451-488, 787-825).
    - Tree lines and an indent size (src/widgets/display/tree.rs:239-248; src/widgets/display/tree/live/paint.rs:12-59).
    - Horizontal scroll (src/widgets/display/tree/live.rs:552-559, 767-781).
    - Border, maximum height and style strings (src/widgets/display/tree.rs:281-326; src/widgets/display/tree/live/paint.rs:328-355).
    - A check for duplicate ids, shown as an error (src/widgets/display/tree/live.rs:64-74; src/widgets/display/tree/live/paint.rs:257-260).
    - Optional windowed drawing (src/widgets/display/tree.rs:329-332; src/widgets/display/tree/live/paint.rs:266-272).
  - Only gpui-kit:
    - A context menu per row and a right-clicked row state (G/tree.rs:54-62, 92-103; B/tree.rs:449-455).
    - `reveal_item` and `set_selected_item`, which expand the ancestors of a hidden item (B/tree.rs:230-241, 268-278, 291-309).
    - `scroll_to_item` with a strategy (B/tree.rs:260-262).
    - Fully app-drawn rows (G/tree.rs:17-23).
- Builder API:
  - gpui-kit, state: `TreeState::new(cx).items(..)`, `set_items`, `selected_index`, `set_selected_index`, `set_selected_item`, `selected_item`, `selected_entry`, `entry`, `scroll_to_item`, `index_of`, `reveal_item` and `focus` (B/tree.rs:196-282).
  - gpui-kit, items and element: `TreeItem::new(id, label)`, `.child`, `.children`, `.expanded` and `.disabled` (B/tree.rs:98-129). `tree(&state, render_item)` and `.context_menu(..)` (G/tree.rs:17-62).
  - ours, builder: `TreeBuilder::new`, `.root`, `.selected`, `.expand`, `.expanded_nodes`, `.selectable`, `.multi_select`, `.show_icons`, `.show_lines`, `.indent_size`, `.lazy_loading`, `.checkable`, `.drag_drop`, `.search`, `.filter_visible`, `.border`, the style setters, `.scrollable`, `.max_height` and `.virtual_scrolling` (src/widgets/display/tree.rs:190-332).
  - ours, callbacks and output: `on_select`, `on_expand`, `on_multi_select`, `on_check`, `on_node_action` and `on_load_children`, then `.build` or `.render` (src/widgets/display/tree.rs:335-411).
  - ours, nodes: `TreeNode::new(id, label)` with `.add_child`, `.children`, `.expanded`, `.selected`, `.checked`, `.icon`, `.with_style` and `find_node` (src/widgets/display/tree.rs:49-155).
- States:
  - gpui-kit: expanded and disabled per item (B/tree.rs:34-37). The state also holds the selected index, the right-clicked index, a focus handle and a scroll handle (B/tree.rs:183-192).
  - ours: selected, expanded and checked ids; scroll; focus; the hovered node; the flat rows; the visible ids; search matches; loading nodes; the drag source; and the drop target (src/widgets/display/tree.rs:578-605). The cursor is kept apart from the selection (src/widgets/display/tree/live.rs:56, 293-308). Each node has selectable, checkable, expandable, lazy and loading flags (src/widgets/display/tree.rs:33-42).
- Keyboard and screen reader:
  - ours, moving: Up, Down, Home, End, PageUp and PageDown move the cursor and skip rows that cannot be selected. Shift extends the selection in multi-select (src/widgets/display/tree/live.rs:505-531).
  - ours, expanding: Left collapses or moves to the parent. Right expands or moves to the first child. Shift+Left and Shift+Right scroll sideways (src/widgets/display/tree/live.rs:533-559).
  - ours, other keys: Enter toggles the row and sends "activate" (src/widgets/display/tree/live.rs:560-566). Space checks the row or toggles its selection (src/widgets/display/tree/live.rs:567-573). `+` and `=` expand, `-` collapses, `*` expands all rows at the same level, and Ctrl+A selects all (src/widgets/display/tree/live.rs:574-602).
  - ours, mouse: wheel, clicks on the expander, checkbox or label, and double-click to activate (src/widgets/display/tree/live.rs:761-866).
  - ours, screen reader: the root is a Tree (src/widgets/display/tree/live/paint.rs:358-359). Each row is a TreeItem with a label, selected state, expanded state when it can expand, and a click action or a disabled flag (src/widgets/display/tree/live/paint.rs:210-221). Checkboxes are CheckBox nodes with a toggled state (src/widgets/display/tree/live/paint.rs:164-197). Focus and activate events are wired (src/widgets/display/tree/live/paint.rs:238-252; src/widgets/display/tree/live.rs:716-747).
  - ours, screen-reader gap: the Tree node does not mark multi-select, while our Select widget does (src/widgets/input/select.rs:271-273, 299-301).
  - gpui-kit, keys: Up and Down wrap at the ends. Left collapses an open folder and Right expands a closed one; they never move to the parent or a child (B/tree.rs:21-26, 365-411).
  - gpui-kit, Enter: a Confirm handler toggles a folder (B/tree.rs:350-363, 523). But the Tree context binds only the four arrows (B/tree.rs:21-26), and I found no Enter binding for that context.
  - gpui-kit, mouse: left click selects and toggles; right click marks the row (B/tree.rs:413-417, 442-456).
  - gpui-kit, screen reader: the root is a Tree (B/tree.rs:520). Rows are TreeItems with a label, selected state, and expanded state for folders (B/tree.rs:432-440).
  - Neither side sets a level or a position in set on tree items (B/tree.rs:432-440; src/widgets/display/tree/live/paint.rs:210-221).
- Worth adopting:
  - Reveal an item: expand its ancestors, then select it and scroll to it (B/tree.rs:230-241, 268-278, 291-309). Today, setting `selected_node` to a node under a closed parent leaves that node selected but hidden, and the cursor falls back to the first row (src/widgets/display/tree/live.rs:223-229, 655-658). Size: small.
  - A row context menu, opened by right click where the terminal reports it, or by a key (G/tree.rs:54-62, 92-103). File-tree apps need actions such as rename and delete next to the row. Size: medium.
  - Level and position in set on each TreeItem, in the same way as gpui-kit's tab position (B/tabs.rs:75-81). A screen reader could then say "level 2, 3 of 7". Size: small.

### virtual_list (virtual list)
- Files: ours (no dedicated widget) src/widgets/display/table/live.rs, src/widgets/display/table.rs, src/widgets/display/data_table.rs, src/widgets/display/data_table/live.rs, src/widgets/display/tree/live/paint.rs, src/widgets/display/file_explorer.rs, src/widgets/display/file_explorer/live.rs, src/widgets/display/file_explorer/live/paint.rs; gpui-kit G/virtual_list.rs (a re-export only, G/virtual_list.rs:1-2), B/virtual_list.rs.
- Structure:
  - gpui-kit: `v_virtual_list` and `h_virtual_list` take a view entity, an id, one size per item and a closure that draws a range of indices (B/virtual_list.rs:129-212). `VirtualListScrollHandle` drives the list (B/virtual_list.rs:38-127).
  - gpui-kit, element: `VirtualList` is a custom gpui Element (B/virtual_list.rs:214-227). It finds the visible range with binary searches over prefix sums (B/virtual_list.rs:346-412) and lays out only those items (B/virtual_list.rs:687-733).
  - ours: there is no standalone virtual list. Windowing is built into several widgets:
    - The table keeps only the rows in view in `visible_rows` (src/widgets/display/table/live.rs:117-127) and builds elements only for them (src/widgets/display/table/live.rs:405-483).
    - The data table passes an offset, an overscan and a revision to the table (src/widgets/display/data_table/live.rs:516-550).
    - The tree skips rows outside the viewport when `virtual_scrolling` is on (src/widgets/display/tree/live/paint.rs:266-272).
    - The file explorer draws a slice of its rows (src/widgets/display/file_explorer/live/paint.rs:294-307; src/widgets/display/file_explorer/live.rs:341-346).
- Features and options:
  - Only gpui-kit:
    - Items of different sizes (B/virtual_list.rs:1-12, 133-143).
    - A vertical or horizontal axis (B/virtual_list.rs:152-171).
    - A cross-axis size taken from one measured item (B/virtual_list.rs:248-252, 314-336).
    - One extra item drawn past the end (B/virtual_list.rs:383-412).
    - `scroll_to_item` with a strategy, including Center, and `scroll_to_bottom` (B/virtual_list.rs:106-126, 264-311).
    - Works with the shared scrollbar (B/virtual_list.rs:61-77).
  - Only ours:
    - Overscan rows in the data table (src/widgets/display/data_table.rs:98-99; src/widgets/display/data_table/live.rs:516-524).
    - A 70,000-row table test where End and Up keep the selected row inside the window (src/widgets/display/table/live.rs:741-785).
  - Limits on our side:
    - Every row is one cell tall, and `VirtualScrollConfig` has a single `row_height` (src/widgets/display/data_table.rs:93-108).
    - The defaults read like pixels: `row_height` 32 and `viewport_height` 400 (src/widgets/display/data_table.rs:352-358). The live code divides them to get 13 rows (src/widgets/display/data_table/live.rs:525-529).
    - `VirtualScrollConfig::visible_range()` is called only from tests (src/widgets/display/data_table.rs:112-130, 437-444; grep `visible_range()` in src). The live table computes its own window.
    - Each time the table recomputes the window, it builds an index list of every row, and sorts it when a sort column is set (src/widgets/display/table/live.rs:118-121; src/widgets/display/table.rs:343-367).
- Builder API:
  - gpui-kit: `v_virtual_list(view, id, sizes, render_range)`, `h_virtual_list(..)`, `.track_scroll(&handle)`, `.with_sizing_behavior(..)` and `.with_item_to_measure_index(ix)` (B/virtual_list.rs:139-252). `VirtualListScrollHandle::new`, `.scroll_to_item(ix, strategy)` and `.scroll_to_bottom()` (B/virtual_list.rs:87-127).
  - ours:
    - `DataTableProps::with_virtual_scroll(enabled, row_height, viewport_height)` (src/widgets/display/data_table.rs:383-394).
    - `TreeBuilder::virtual_scrolling(bool)` (src/widgets/display/tree.rs:329-332).
    - `FileExplorerBuilder::max_visible_items(n)` (src/widgets/display/file_explorer.rs:533-537).
- States:
  - gpui-kit: the handle keeps the item count, the last content size and a pending scroll-to-item (B/virtual_list.rs:30-36). Item origins and sizes are cached across frames (B/virtual_list.rs:346-357).
  - ours: the table keeps `scroll_y`, a u16 public offset and `visible_rows` (src/widgets/display/table/live.rs:102-127). The data table keeps a scroll offset (src/widgets/display/data_table/live.rs:65-71). The tree keeps a scroll value (src/widgets/display/tree/live.rs:241-255).
- Keyboard and screen reader:
  - gpui-kit: none found. `grep role|aria|accesskit|keybinding|focus|on_action` in B/virtual_list.rs has no match. Keys and roles belong to the caller's rows.
  - ours, keys: the host widget owns the keys. The table test shows End and Up moving the window (src/widgets/display/table/live.rs:771-784).
  - ours, screen reader: the table is a Table (src/widgets/display/table/live.rs:565), each row is a Row with selected and disabled (src/widgets/display/table/live.rs:467-471), and each cell is a Cell (src/widgets/display/table/live.rs:435-439). Only rows inside the window exist in the tree (src/widgets/display/table/live.rs:405-483). No row index or row count is set; `grep row_count|row_index|set_row` in the table files finds nothing.
- Worth adopting:
  - A reusable virtual list: a callback that draws a range of rows, row heights in cells, and prefix sums with binary search (B/virtual_list.rs:129-212, 346-412). Apps with long logs or chat views would not need to write their own windowing. Size: medium.
  - `scroll_to_item` with a Top, Center or nearest strategy (B/virtual_list.rs:106-126, 264-311). A search hit shown in the middle is easier to read than one on the last line. Ours only reveals to the nearest edge (src/widgets/display/table/live.rs:130-150). Size: small.
  - A row index and row count on windowed rows, so a screen reader can say "row 500 of 70,000" when only a few rows exist in the tree (src/widgets/display/table/live.rs:467-471). Neither side has this. Size: small.

## gpui-kit components we lack

Each entry says what the component is, what we have that comes closest, what it would be in a terminal, and a build or skip call. Where we already have it under another name, the entry compares the two instead.

### alert
- gpui-kit: An inline message box that sits in the page flow. It is not an overlay. It has five variants: Default, Info, Success, Warning and Error. Each variant has its own text, fill and border color (G/alert.rs:14-54). Options are title, icon, `banner` (full width with no radius, and the title is hidden), `visible`, and `on_close`, which adds a close icon (G/alert.rs:123-157). Shortcut constructors pick an icon for each variant (G/alert.rs:89-115). The root sets `Role::Alert` for every variant (G/alert.rs:190-192). The close control is a clickable `div`, not a Button (G/alert.rs:235-252).
- Ours: absent as an inline widget. The nearest widget is Toast. It has the same four types plus Custom (src/widgets/dialog/toast.rs:18-31). It carries a message but no title (src/widgets/dialog/toast.rs:35-49). It draws as an overlay modal at z-index 2000 (src/widgets/dialog/toast/live.rs:162-171). It uses Role::Alert for Error and Warning and Role::Status for the other types (src/widgets/dialog/toast/live.rs:147-152). Searches: `rg -n -i -l 'alert|banner|callout|notice' src` found only Role::Alert uses, and `rg -n -i 'struct \w*(Alert|Banner|Callout|Notice)\w*' src` found no struct.
- In a terminal: A block in the page flow, for example `▌ ⚠ Disk almost full`, with the bar and icon in the variant color. The title is bold on the first row and the message wraps below it. `[×]` sits at the right when the alert can be closed. The 4% fill tint will not show on 16-color terminals, so the color has to come from the bar and the icon. The close control should be a focusable button (Enter/Space and click). gpui-kit's clickable div cannot take keyboard focus. For screen readers, reuse the toast mapping: Alert for Warning and Error, Status for Info and Success. That way a quiet info notice does not interrupt the reader.
- Call: build, small. Toast already has the types, colors and role mapping. An inline alert is the same content without the overlay.

### attachment
- gpui-kit: A card for a file or image in a chat composer. It has three slots: media, content (title and description) and actions (G/attachment.rs:64-76, 125-141, 609-614). The media slot takes an icon, or an image preview with an overlay (G/attachment.rs:253-258, 280-290). The status can be Pending, Uploading, Processing, Failed or Complete (G/attachment.rs:16-30). Each status has its own look:
  - Pending draws a dashed border (G/attachment.rs:213).
  - Failed draws a red border (G/attachment.rs:209).
  - A loading image is dimmed (G/attachment.rs:389).
  - The title shimmers while work is in progress (G/attachment.rs:520-521).

  Other options: a horizontal or vertical axis (G/attachment.rs:119-123), and a click on the whole card that stays below the action buttons (G/attachment.rs:100-111). `AttachmentGroup` is a row that scrolls sideways (G/attachment.rs:710-717, 749). No accessibility role is set (`grep -n -i 'role' G/attachment.rs` finds none).
- Ours: absent. The searches `rg -n -i -l 'attachment|attach' src` and `rg -n -i 'struct \w*(Attach|FileChip|FileCard|Upload)\w*' src` found only GPU and scheduler uses. The nearest parts:
  - `FileEntry` has a name, size and icon (src/widgets/display/file_explorer.rs:34-52).
  - The Image widget draws images with sixel, Kitty, iTerm2, chafa or viu, or falls back to ASCII art or text (src/widgets/display/image/mod.rs:138-157).
  - `ProgressBar` (src/widgets/display/progress_bar.rs:382).
- In a terminal: A one-row or boxed chip, for example `📄 report.pdf  1.2 MB  ⟳ 40%  [×]`. Pending uses a dashed box line (`┄`). Failed turns red. The shimmer becomes a spinner glyph or a small progress bar. An image preview only works where the terminal has a graphics protocol. The row scrolls with Left and Right. Screen reader: a group named by the file name, with the status as its description.
- Call: skip. It is chat-composer specific, and our text, progress bar and buttons can compose it.

### avatar
- gpui-kit: A round user picture. Without an image it shows initials from the name. The initials sit on a color picked from the initials, so one person always gets the same color (G/avatar/avatar.rs:13-25, 40-60, 140-150). With no name either, it shows a placeholder icon. Sizes are 16, 24, 48 and 80 px (G/avatar/mod.rs:10-19). `AvatarGroup` overlaps avatars. It takes a `limit` and shows a "⋯" avatar for the overflow (G/avatar/avatar_group.rs:8-17, 44-55, 76-107). The base primitive renders the image slot, or the fallback slot when there is no image (B/avatar.rs:9-11, 57-65).
- Ours: absent. `rg -n -i -l 'avatar|initials|profile_pic' src` finds nothing. The Image widget can draw a picture where the terminal supports it (src/widgets/display/image/mod.rs:138-157).
- In a terminal: A 2-cell block such as `AB`, with a background color taken from a hash of the name. An image is only possible with a graphics protocol, so initials are the normal case. There are no circles. A group cannot overlap, so it becomes `AB CD EF +2`. Screen reader: Image role named with the full name. It is not focusable.
- Call: skip. Colored initials are a styled span, and image avatars already go through Image.

### badge
- gpui-kit: Wraps a child and pins a marker to its top-right corner. The marker is a count, a dot or an icon (G/badge.rs:8-14, 29-39, 55-85). A count of 0 hides it (G/badge.rs:103-106). Counts above `max` (default 99) show `99+` (G/badge.rs:47, 131-135). The default color is red (G/badge.rs:125). No role is set.
- Ours: no standalone badge. `rg -n -i 'badge' src` finds only tabs. Tabs have a badge option: `TabBadge` with text and a variant (Default, Success, Warning, Error or Info) (src/widgets/layout/tabs.rs:67-104, 120-121, 158-162). It renders as a mark plus text after the tab label, for example `✗5` (src/widgets/layout/tabs.rs:554-563). This is an option on one widget, not a badge widget. The tab's accessible label comes from `tab.label` only (src/widgets/layout/tabs.rs:699-700). I did not check whether the badge text reaches a screen reader by another path.
- In a terminal: Nothing can overlap a corner reliably, so the marker goes after the content: `Inbox 3`, a reverse-video count ` 3 `, or a dot `Inbox ●`. It keeps the `99+` cap. It is not focusable. Screen reader: the badge text is appended to its owner's label, for example "Inbox, 3 new".
- Call: build, small. Lift `TabBadge` into a shared badge for tabs, menu items, tree rows and list rows, and add count, max and dot.

### bubble
- gpui-kit: A chat message bubble. Its surface variants are Filled, Secondary, Muted, Tinted, Outline, Ghost and Destructive (G/bubble.rs:11-29). It aligns to the start or end and is at most 80% wide (G/bubble.rs:121-141). An optional reaction strip goes above or below and is built from Buttons (G/bubble.rs:31-39, 95-99, 302-320). `BubbleGroup` stacks bubbles from one sender (G/bubble.rs:252-254).
- Ours: absent. `rg -n -i -l 'bubble|chat|speech|conversation|message_list' src` finds only event bubbling (src/event/router.rs:31). The nearest containers are ScrollView and Stack (src/widgets/layout/stack.rs:98-100).
- In a terminal: A block per message. Other people's messages sit on the left and our own on the right, capped at 80% of the columns. Each block gets a background color or a box line. Reactions go on the line below, for example `👍 2  ❤ 1`. Up and Down move between messages inside a scroll view. Screen reader: a list of messages such as "Alice: text".
- Call: skip. A bubble alone adds little. If we want chat UIs, build one message-list widget instead.

### button
- gpui-kit: A Button built on a base primitive. The base handles focus, Enter/Space activation and the Button role (B/button.rs:15-20, 99-100). On top of that, gpui-kit adds:
  - Variants: Default, Primary, Secondary, Danger, Info, Success, Warning, Ghost, Link, Text and Custom (G/button/button.rs:140-155).
  - Icon, tooltip and loading options (G/button/button.rs:382-424).
  - A dropdown caret and a `toggled` option that sets the pressed state for assistive tech (G/button/button.rs:471-486).
  - The Link variant takes the Link role (G/button/button.rs:734-740).
  - The module also has ButtonGroup, a split DropdownButton, Toggle and ToggleGroup (G/button/mod.rs:1-11).
- Ours: `builder::button()` is a styled div with fixed classes (src/builder/core.rs:69-72). `.on_click` makes an element interactive (src/builder/core.rs:193-203). An interactive element is focusable and runs its callbacks on Enter, Space or a left click (src/app/event_tree.rs:306-311, 334-348; src/event/types.rs:41-55). It gets Role::Button by default (src/app/event_tree/accessibility.rs:103-106). `disabled` blocks focus and activation (src/builder/core.rs:280-284). There is also a `primary_button` helper (src/builder/layout.rs:100-109) and `ModalButton` with autofocus (src/widgets/display/modal.rs:141-156).
- Differences:
  - Only gpui-kit has variants (G/button/button.rs:140-155).
  - Only gpui-kit has a loading state that blocks clicks while keeping its look (G/button/button.rs:420-424, 488-495).
  - Only gpui-kit has a tooltip that can show the action's key binding (G/button/button.rs:403-418).
  - Only gpui-kit has a pressed state for assistive tech (G/button/button.rs:477-486).
  - Only gpui-kit has group and multi-select groups (G/button/button_group.rs:16-18, 72-76), split buttons (G/button/dropdown_button.rs:17-25) and segmented toggles (G/button/toggle.rs:220-222, 275).
  - Ours has one look set by utility classes. Its `rounded` and `hover:` classes (src/builder/core.rs:71) come from a pixel world and mean little in cells.
- Worth adopting:
  - Variants that map to theme colors (Primary, Danger, Ghost, Link), because color and reverse video are the main cues in a terminal.
  - A `loading` flag that blocks activation and shows a spinner glyph.
  - A `toggled` pressed state.
- ToggleGroup, which gpui-kit keeps inside its button module (G/button/toggle.rs:220-222, 275):
  - In a terminal: `[ Day │ Week │ Month ]`, with the chosen segment in reverse video. Left and Right move, Space or Enter picks, a click picks. The screen reader gets a radio group.
  - Call: skip as a separate widget. Our horizontal radio group already has the single-choice state and the arrow keys (src/widgets/input/radio_button.rs:73-77). A segmented style for it is small.

### carousel
- gpui-kit: A carousel built from parts that share one `CarouselState` (G/carousel/mod.rs:5-9):
  - The root has Role::Region (G/carousel/carousel.rs:23-28, 128).
  - Each item has Role::Group (G/carousel/carousel.rs:513-515, 572).
  - There are Previous and Next controls (G/carousel/carousel.rs:586-588, 648-650) and pagination dots (G/carousel/carousel.rs:786-788, 841-843).

  The state holds the selection, a horizontal or vertical axis, and looping (G/carousel/state.rs:57-65, 122-133). It handles pointer drags, trackpad and wheel input, and snaps to the nearest item (G/carousel/state.rs:476, 545, 648). The keys are Left, Right, Up, Down, Home and End (G/carousel/mod.rs:17-26).
- Ours: absent. `rg -n -i -l 'carousel|slideshow|pager\b|swiper' src` finds nothing, and `rg -n -i 'struct \w*(Carousel|Slide|Pager|Pages)\w*' src` finds only Slider. Tabs already show one panel at a time and move with Left, Right, Home, End and number keys (src/widgets/layout/tabs.rs:875-891).
- In a terminal: One slide fills the area, with `‹ 2 / 5 ›` or `○ ● ○ ○ ○` under it. The arrow keys change the slide, and clicking a dot jumps to it. Snap animation and swipe do not carry over. Trackpad input arrives as wheel events at most. Screen reader: a region with the slides announced as "slide 2 of 5".
- Call: skip. Tabs plus a dot strip covers the use.

### clipboard
- gpui-kit: A small ghost icon button. A click copies `value`, or the result of `value_fn`. It then swaps the Copy icon for a Check for 2 seconds and calls `on_copied` (G/clipboard.rs:13-22, 50-75, 86-126). A tooltip is optional (G/clipboard.rs:44-48).
- Ours: no widget. The `use_clipboard` hook returns a state signal plus copy and paste functions (src/hooks/clipboard.rs:177-198). Its doc example builds a copy button from `builder::button` (src/hooks/clipboard.rs:183-190). The backends are wl-copy, xsel, xclip, pbcopy/pbpaste and PowerShell; otherwise it reports Unavailable (src/hooks/clipboard.rs:11-21, 49-72). TextInput copies and pastes its selection (src/widgets/input/text_input.rs:675, 700). Search: `rg -n -i -l 'clipboard|osc ?52|osc52' src`. Our only OSC 52 code is the parser for the embedded terminal (src/escape/osc.rs:272).
- In a terminal: `[Copy]` becomes `[✓ Copied]` for 2 seconds, or `[! Copy failed]` when the hook reports an error. Enter, Space or a click activates it. Screen reader: a Button named "Copy", then a polite status message. Over SSH, the command backends run on the remote host and do not reach the user's clipboard. OSC 52, a terminal escape sequence, reaches the user's clipboard in many terminals, and the hook has no OSC 52 backend (src/hooks/clipboard.rs:11-21).
- Call: skip the widget, because the hook's doc example is already this widget in a few lines. The gap worth closing is an OSC 52 write path in the hook. That is hook work, and small.

### collapsible
- gpui-kit: A controlled wrapper. `open(bool)` shows or hides the `content` child. Other children, such as the trigger, always show. An optional spring animation reveals the content (G/collapsible.rs:9-48, 63-79; B/collapsible.rs:12-19, 69-81). It sets no role, no expanded state and no key handling. The caller builds the trigger.
- Ours: Accordion. Each section has a header button, an expanded state and optional custom header, icon and aria label (src/widgets/layout/accordion.rs:55-75). The modes are Single, Multiple and AlwaysOne (src/widgets/layout/accordion.rs:21-31). The header node is a Button with the expanded flag set (src/widgets/layout/accordion/live.rs:139-152). One section in Multiple mode works as a collapsible.
- Differences: gpui-kit is headless. Any element can be the trigger, and the caller owns state and semantics. Ours owns the trigger, the semantics and the keys. It takes an initial `expanded` flag (src/widgets/layout/accordion.rs:69-70) and reports changes through a named CustomEvent (src/widgets/layout/accordion.rs:157-159). Ours has expand and collapse icons, a reduced-motion flag, and no spring (src/widgets/layout/accordion.rs:145-161).
- Worth adopting: nothing.

### color_picker
- gpui-kit: A button that opens a popover (G/color_picker.rs:491) with two tabs (G/color_picker.rs:175-176):
  - Palette: featured colors plus nine theme color ramps (G/color_picker.rs:22-49, 204-217).
  - HSLA: hue, saturation, lightness and alpha sliders (G/color_picker.rs:281, 387).

  It also has a hex field and a hover preview (G/color_picker.rs:151-198). Its options are featured colors, icon, label, accessibility label and anchor (G/color_picker.rs:82-123). The base holds the state and accepts `#rgb`, `#rgba`, `#rrggbb` and `#rrggbbaa` (B/color_picker.rs:22-32, 171-177). It binds Enter and Escape (B/color_picker.rs:27-28), and the root owns focus and the Button role (B/color_picker.rs:460-466, 519-520).
- Ours: absent. `rg -n -i -l 'color_?picker|colorpicker|swatch|palette' src` finds only theme and sixel code. The only matching struct is the sixel `Palette` (src/widgets/display/image/sixel_encode.rs:106). Useful parts:
  - `Slider` (src/widgets/input/slider.rs:197).
  - `hex_to_rgb`, which accepts `#rgb` and `#rrggbb` with no alpha (src/theme/ansi.rs:100-103).
  - `ColorDepth` detection (src/core/capabilities.rs:55-67).
- In a terminal: A grid of `██` swatches limited to what the terminal shows: 16 colors, 256 colors or truecolor. The arrow keys move between swatches and Enter picks one. The HSL rows use our Slider, with a gradient track only on truecolor. The hex value goes in a TextInput. Alpha means nothing for cell colors. Screen reader: each swatch is a button named by its hex value. The ColorWell role exists in our Linux bridge (src/accessibility/platform/translation/node.rs:116).
- Call: skip. It is rare in terminal apps, and the only clear user, a theme editor, does not justify medium work now.

### combobox
- gpui-kit: A trigger plus a popup with a search field over a searchable list. It supports single or multiple selection; with multiple, the popup stays open (G/combobox.rs:282-296, 744-749). Other options: menu width and height, placeholder, check icon, a clear button (`cleanable`), an empty-state builder, a custom trigger and a footer (G/combobox.rs:776-862). The base root owns combobox semantics and binds Up, Down, Enter and Escape (B/combobox.rs:18-26, 33-38).
- Ours:
  - `Select`, a dropdown with hidden type-ahead that keeps typed letters for 1 second (src/widgets/input/select.rs:194-198, 698-725). Its keys are Enter, Space, Escape, arrows, Home, End, PageUp and PageDown (src/widgets/input/select.rs:642-697). The builder has a multiple mode (src/builder/widgets/input.rs:500-507), which sets multi-select semantics (src/widgets/input/select.rs:271-272).
  - `AutocompleteDialog`, a visible query field with suggestions from a static list or an HTTP URL, with debounce, minimum characters and match highlighting (src/widgets/dialog/autocomplete.rs:93-118).
- Differences:
  - Only gpui-kit shows a search field inside an anchored dropdown. Ours has either hidden type-ahead (Select) or a separate dialog (AutocompleteDialog).
  - Only gpui-kit has the clear button (G/combobox.rs:815-820), the custom trigger (G/combobox.rs:844-854), the footer (G/combobox.rs:855-862) and the empty-state builder (G/combobox.rs:827-837).
  - Only ours has HTTP-backed suggestions with debounce (src/widgets/dialog/autocomplete.rs:101-107).
- Worth adopting:
  - A `searchable` Select that shows a filter line at the top of the open list. It can reuse the grapheme-safe `match_ranges` from autocomplete for highlighting (src/widgets/dialog/autocomplete/live.rs:629-631).
  - A clear action: `[×]` on the control, plus Delete when it has focus.
  - A "No matches" empty row.

### command
- gpui-kit: A command palette: a search field over a filtered list of commands (G/command/mod.rs:1-5; G/command/command.rs:42-58).
  - Items have a label, an icon, an Action whose key binding is shown on the row, a checked state and extra search keywords (G/command/item.rs:60-66, 74-86).
  - Groups have headings that hide when nothing in the group matches (G/command/item.rs:150-153).
  - Matching is a case-insensitive substring test on the label and keywords (G/command/item.rs:114-128).
  - Options: `searchable`, `filterable` (turn it off for async search), `on_query`, `on_select`, `on_confirm`, `on_cancel`, placeholder, empty state, header and footer (G/command/command.rs:100-128, 177-222).
  - Keys are Escape, Enter, Up and Down (G/command/state.rs:62-69). The roles are ListBox and ListBoxOption (G/command/state.rs:747, 862).
- Ours: absent. `rg -n -i -l 'command.?palette|palette|fuzzy|quick.?open|command_bar|CommandBar' src` finds only color and sixel code. Nearest parts:
  - `DialogMenu` Selection type, a list in a dialog (src/widgets/menu/dialog.rs:7-21). `rg -n -i 'filter|search|query' src/widgets/menu/*.rs` finds no query filter over that list.
  - `AutocompleteDialog`, a query plus suggestions (src/widgets/dialog/autocomplete.rs:93-118).
  - `MenuShortcut` display text (src/widgets/menu/item.rs:3-10), shown by the menu view (src/widgets/menu/view.rs:153-158).
- In a terminal: A centered overlay about 60% wide. The top row is the query line `> ope█`. Below it, rows are grouped under dim headings, with shortcuts right-aligned, for example `Open file…            Ctrl+O`. The highlighted row is in reverse video. Up, Down, PageUp and PageDown move; Enter runs the command; Escape closes. A click picks a row and the wheel scrolls. Screen reader: the query field as a ComboBox, which our bridge maps (src/accessibility/platform/translation/node.rs:118), and the list as a ListBox that reports the active option.
- Call: build, medium. Keyboard-first TUIs benefit most from a palette, and we already have modal, text input, menu items and match highlighting to build on.

### description_list
- gpui-kit: A list of label and value pairs, with separators (G/description_list.rs:8-27, 208-209). The layout is horizontal (the default, with the label beside the value) or vertical (G/description_list.rs:125-146). Options: label width (default 120 px), `bordered` (default on), and `columns` (default 3, allowed 1 to 10), with per-item `span` (G/description_list.rs:98-101, 147-175). No role is set.
- Ours: absent. The search `rg -n -i -l 'description.?list|key.?value|property.?list|definition.?list|properties_panel|\bdl\b' src` found only chart, markdown and animation files. `rg -n -i 'description_list|DescriptionList|key_value|KeyValue' src` finds only a Markdown parser flag (src/markdown/renderer.rs:27) and chart names. The nearest widgets are Table (src/widgets/display/table.rs:54) and the grid layout (src/layout/grid.rs:1).
- In a terminal: Two columns with a fixed label width in cells:
  ```
  Name      reactive-tui
  Version   1.0.0
  ```
  Labels are muted. The vertical layout puts each label on its own line above its value. `bordered` uses box lines, and extra columns place pairs side by side. It is not focusable. Screen reader: the DescriptionList, Term and Definition roles, which our Linux bridge already maps (src/accessibility/platform/translation/node.rs:123-124, 265).
- Call: build, small. Settings and detail views need it often, and it is layout plus three roles.

### dock
- gpui-kit: A dockable layout: a center area plus optional left, right and bottom docks (B/dock/mod.rs:1-12; B/dock/state.rs:161-170). Each area is a tree of splits and tab groups that the user can rearrange by dragging. The whole layout can be saved and restored (B/dock/mod.rs:31-47). The API includes `set_center`, `set_dock`, `toggle_dock`, `load` and `dump` (B/dock/dock_area.rs:232, 246, 301, 716, 794). The base draws nothing; renderer traits supply the appearance (B/dock/mod.rs:49-58). The component crate is the skin: it re-exports the base types and adds ToggleZoom and ClosePanel actions (G/dock/mod.rs:1-16, 48-59).
- Ours: absent. `rg -n -i -l '\bdock|split.?pane|splitter|resizable|pane_tree|panel_group|SplitView' src` finds only table, modal and dialog resize flags. `rg -n -i 'struct \w*(Dock|Split|Pane|Resiz|Panel)\w*' src` finds only event and menu structs. The nearest widgets are Tabs and Stack (src/widgets/layout/stack.rs:98-100).
- In a terminal: Tiled panes like tmux, with one-cell `│`/`─` dividers, a tab strip per group, and zoom to give one pane the full screen. Drag-to-dock with drop previews and drag ghosts does not carry over well. Use key commands to move a panel left, right or down instead. Screen reader: each pane as a region, each tab strip as a tab list.
- Call: skip. It is large. A resizable split plus our Tabs covers the common need. Revisit if an IDE-like app asks for it.

### empty
- gpui-kit: A presentational empty state. The header holds media, a title and a description, in that order. Media can be an icon in a frame or an unframed image (G/empty.rs:85-89, 107-120, 155-163). A content slot below holds actions (G/empty.rs:326-328). The app decides when to show it (G/empty.rs:8-14).
- Ours: absent. The search `rg -n -i 'empty_state|empty_message|no_data|no_items|no results|No data|EmptyState|placeholder_text|empty_text' src/widgets` found only hard-coded strings:
  - Tree shows "No matching nodes found" or "No data" (src/widgets/display/tree/live/paint.rs:278-286).
  - Charts draw "No data to display" (src/widgets/display/charts/live/canvas.rs:420).
- In a terminal: A centered block:
  ```
      ∅
  No results
  Try a different search.
  [ Clear filters ]
  ```
  The title is bold and the description muted. The action buttons are focusable. Screen reader: a group labelled by the title and described by the description.
- Call: build, small. It also lets tree and charts use one styled empty state instead of their own strings.

### form
- gpui-kit: A layout for fields: `v_form`, `h_form` and `field()` (G/form/mod.rs:7-20).
  - Form options: labels above or beside controls, label width (default 140 px), label text size, footer and column count (G/form/form.rs:12-14, 28-100).
  - Field options: label, description, visible, required and column span/start/end (G/form/field.rs:79-81, 118-174, 203-218). Required draws a red `*` (G/form/field.rs:313-316).

  It has no validation or error display (`grep -n -i 'error\|valid'` finds nothing in G/form/form.rs or G/form/field.rs). It has no accessibility link between a label and its control (`grep -n -i 'labelled\|accessib\|role'` finds nothing in those files).
- Ours: absent. `rg -n -i 'struct \w*(Form|Field|Fieldset)\w*' src` finds only `InputFieldConfig`, which has required, mask and similar fields for a single-field dialog (src/widgets/dialog/input.rs:100-112). `WizardDialog` steps each take a validator (src/widgets/dialog/wizard.rs:28-39). The parts we have are a `label` helper (src/builder/layout.rs:78-81) and the grid layout (src/layout/grid.rs:1).
- In a terminal: A label column of fixed cell width with the control to its right, or the label on the line above. Required fields get a red `*` and the description is a muted line below. Tab order follows the fields. Screen reader: each control is named by its label and described by its description. We should add that link, since gpui-kit does not.
- Call: build, small. It is layout plus a label-to-control link, and it makes our inputs line up.

### group_box
- gpui-kit: A container with an optional title above it (G/group_box.rs:59-70, 92-96, 144-152). The variants are Normal, Fill and Outline (G/group_box.rs:9-16). Fill adds a background and Outline adds a border; both add padding (G/group_box.rs:132-136, 155-158). The title and content styles can be overridden (G/group_box.rs:98-108). No role is set.
- Ours: `card()` and `card_builder()`, bordered containers with fixed classes and no title (src/builder/layout.rs:21-24, 84-90). Our border options have no title slot (src/core/window.rs:129-140).
- Differences: Only gpui-kit has a title slot and the three variants. Ours is a single look set by classes.
- Worth adopting:
  - A title drawn into the top border, `┌─ Network ────────┐`. This is the classic TUI frame, and it saves a row compared with a title above the box.
  - Variants mapped to cells: Normal has no border, Outline uses box lines, Fill uses a background color.
  - Role::Group with the title as the group's name.

### hover_card
- gpui-kit: A popover that opens when the pointer rests on its trigger. The default delays are 600 ms to open and 300 ms to close (G/hover_card.rs:13-52, 81-91). On iOS and Android, a tap toggles it instead (G/hover_card.rs:15-17). The base has no focus or key handling (`grep -c -i 'focus\|keybinding\|on_action' B/hover_card.rs` returns 0).
- Ours: `Popover` with `PopoverTrigger::Hover`. It has `hover_delay` and `hover_leave_delay` (src/widgets/display/popover.rs:75-86, 165-168), with defaults of 100 ms and 300 ms (src/widgets/display/popover.rs:246-247).
- Differences:
  - Only ours has Click, Focus and Manual triggers (src/widgets/display/popover.rs:75-86), close on Escape (on by default; src/widgets/display/popover.rs:159-160, 243), an arrow (src/widgets/display/popover.rs:90) and boundary flipping (src/widgets/display/popover.rs:125).
  - Only gpui-kit has tap-to-toggle on touch devices.
  - Hover in a terminal needs any-motion mouse reporting. Only the DirectTty backend turns it on (src/backend/direct_tty.rs:69); the default backend does not (see "Defects found" in the summary). Keyboard users never hover.
- Worth adopting: nothing. One gap of our own: `trigger` takes a single value (src/widgets/display/popover.rs:143-144), so one card cannot open on both hover and focus.

### icon
- gpui-kit: An SVG icon element. `IconName` names icons from the Lucide catalog in gpui-kit-assets (G/icon.rs:9-38). An `Icon` can use a path or raw SVG bytes, and it can be rotated or transformed (G/icon.rs:84-90, 104-167).
- Ours: absent as a widget. `rg -n -i 'nerd.?font|struct \w*Icon\w*|enum \w*Icon\w*|mod icons?\b|fn icon_for|fn get_icon' src` finds only one widget's map. Each widget uses its own strings:
  - Tabs take an icon string (src/widgets/layout/tabs.rs:118-119), and so do accordion sections (src/widgets/layout/accordion.rs:61-62).
  - The file explorer has an emoji map (src/widgets/display/file_explorer.rs:112-121).
  - Confirmation dialogs map to glyphs (src/widgets/dialog/confirmation/live.rs:130-138).
  - Tab badges map to marks (src/widgets/layout/tabs.rs:555-561).

  The same idea gets different glyphs. Error is `×` in confirmations and `✗` in tab badges.
- In a terminal: One named catalog (Info, Warning, Error, Check, Close, chevrons, Folder, File and so on). It maps to single-width Unicode with an ASCII fallback such as `i ! x v`, and optionally to a Nerd Font set. Emoji are two cells wide and vary across terminals, so they should not be the default. Screen reader: decorative icons are hidden; icons that carry meaning get a label.
- Call: build, small. It makes widgets agree and gives one place to handle terminals with poor fonts.

### kbd
- gpui-kit: An inline tag that shows a keystroke (G/kbd.rs:9-16). Options are `appearance` and `outline` (G/kbd.rs:40-50). It can look up the binding for an action, globally or in a focus context (G/kbd.rs:52-95). `format` renders the keystroke per platform: `⌃⌥⇧⌘` on macOS, and `Ctrl+Shift+…` elsewhere (G/kbd.rs:96-161).
- Ours: absent. `MenuShortcut` is a display string plus keys (src/widgets/menu/item.rs:3-10), shown as plain text by the menu view (src/widgets/menu/view.rs:153-158). `rg -n 'impl (std::)?fmt::Display for (KeyCode|KeyEvent|KeyModifiers|KeyCombination)' src` finds nothing, so nothing formats our `KeyEvent` and `KeyModifiers` (src/event/types.rs:89, 234) as text.
- In a terminal: `Ctrl+S` in dim reverse video, or `[Ctrl+S]`, built from our key types. The macOS symbols are optional, because many terminal fonts lack `⌘⌥`. It is not focusable. Screen reader: a spoken form such as "Control S", not the symbols.
- Call: build, small. Menus, the command palette and help screens all need the same formatter.

### label
- gpui-kit: A text element (G/label.rs:51-59, 62-90) with:
  - Optional muted secondary text.
  - A `masked` mode that draws `•` for each character (G/label.rs:10, 199-205).
  - Case-insensitive highlighting of either a prefix or every match (G/label.rs:12-17, 109-127). Masking turns highlighting off (G/label.rs:153-156).
- Ours: `builder::label()`, a span with fixed classes (src/builder/layout.rs:78-81). Text elements get Role::Label in the accessibility tree (src/app/event_tree/accessibility.rs:107-108). Masking exists only in TextInput's password mode (src/widgets/input/text_input.rs:26-27, 121-124). Match highlighting exists only inside AutocompleteDialog (src/widgets/dialog/autocomplete/live.rs:600-609).
- Differences:
  - Only gpui-kit has secondary text, masking and highlighting on a plain label.
  - gpui-kit applies match offsets found in the lowercased text directly to the original text (G/label.rs:112-125). Our `match_ranges` maps lowercase offsets back to grapheme boundaries (src/widgets/dialog/autocomplete/live.rs:629-631). Ours is safer when lowercasing changes a string's byte length.
- Worth adopting:
  - A highlight option on labels, using our `match_ranges` moved out of autocomplete, for Select, command palette and tree filtering.
  - Muted secondary text, for example `main.rs  src/`.

### link
- **gpui-kit:** G/link.rs:8-17 is a styled, `<a>`-like element. It has a link color, an underline, and hover and active tints (G/link.rs:74-89). A click calls `cx.open_url` on the href, then the `on_click` handler (G/link.rs:94-103). Options are href, on_click and disabled (G/link.rs:32-54). This styled version is pointer-only, and its `disabled` does nothing (tests at G/link.rs:171-188). The base link is the accessible one (B/link.rs:16-37):
  - Role::Link, an accessibility label, and focus with tab index and tab stop.
  - Enter and Space activation.
  - An injected `open_with` strategy; the base never launches a browser itself.
  - A disabled state that blocks the click (B/link.rs:78-133, B/link.rs:163-202; tests at B/link.rs:334-382).
- **Ours:** there is no Link widget. The pieces that exist:
  - Breadcrumb segments are Role::Link nodes (src/widgets/layout/breadcrumb/live.rs:183-196).
  - Markdown links are painted blue and underlined, but the URL is dropped (`NodeValue::Link(_link)` at src/markdown/ast_walker.rs:273-281; src/markdown/converter.rs:110-118).
  - `role="link"` maps to Role::Link (src/accessibility/style.rs:254), and the link role class underlines (src/layout/css/accessibility.rs:228-231).
  - An OSC 8 writer exists (src/platform/mod.rs:464-469, src/platform/mod.rs:775-786, src/backend/direct_tty.rs:80-86), and suprtui cell attributes carry a link id (crates/reactive-tui-suprtui/src/ansi.rs:242-256).
  - `rg -n 'link_id|set_link|hyperlink' src/layout src/backend src/component src/core` hits only src/backend/direct_tty.rs:80-86, so the element paint path carries no link.
- **In a terminal:** underlined text in the link color. It is wrapped in OSC 8 where supported, so the terminal's own Ctrl+click also works. It is a Tab stop. Enter, Space or a click runs an app-supplied open callback. A disabled link is dim and skips focus. The screen reader gets Role::Link with a label and the click action. The hover tint works only where the terminal reports mouse motion.
- **Call:** build, small. The role, the OSC 8 writer and the underline class exist. What is missing is a focusable element, plus a way to carry the URL into painted cells.

### list
- **gpui-kit:** a virtual list, ListState (G/list/list.rs:70) and List (G/list/list.rs:721), driven by a ListDelegate (G/list/delegate.rs:10-170).
  - Sections with headers and footers (G/list/delegate.rs:27, G/list/delegate.rs:52-71).
  - Empty and loading views, with a skeleton as the default loading view (G/list/delegate.rs:74-116).
  - An optional search box on top (G/list/list.rs:122-128) that calls `perform_search` (G/list/delegate.rs:15-22).
  - Load more near the bottom via `has_more`, `load_more_threshold` and `load_more` (G/list/delegate.rs:148-170, G/list/list.rs:327-350).
  - A selectable toggle (G/list/list.rs:135-139), a right-click index (G/list/list.rs:199-212) and `scroll_to_item` (G/list/list.rs:239).
  - Keys are only up, down, enter, secondary-enter and escape (G/list/list.rs:26-35). Events are Select, Confirm and Cancel (G/list/list.rs:38-45).
  - All rows must have the same height (G/list/delegate.rs:41).
  - Rows are Role::ListItem with position, set size and selected state (G/list/list.rs:490-495).
- **Ours:** no standalone list. The nearest pieces:
  - Select's popup list: Role::ListBox, multi-select, type-ahead, and Up/Down/Home/End/PageUp/PageDown (src/widgets/input/select.rs:297-303, src/widgets/input/select.rs:647-705).
  - The autocomplete suggestion ListBox with wheel scrolling (src/widgets/dialog/autocomplete/live.rs:468-480) and a substring filter (src/widgets/dialog/autocomplete.rs:737-757).
  - The file explorer ListBox (src/widgets/display/file_explorer/live/paint.rs:443-447).
  - Virtual scrolling on DataTable (src/widgets/display/data_table.rs:384-394) and on Tree (src/widgets/display/tree.rs:328-332).
- **In a terminal:** one row per item, so the same-height rule is natural. The selected row is in reverse video or marked with `▶`. Section headers are dim rows. An optional one-row filter sits on top, and a scrollbar column on the right.
  - Keys: Up/Down, PageUp/PageDown and Home/End move; Enter confirms; Esc cancels; typing goes to the filter.
  - Mouse: a click selects, a double click confirms, the wheel scrolls.
  - Only visible rows are built.
  - Screen reader: a ListBox whose options carry position, set size and selected state.
- **Call:** build, medium. Select, autocomplete and the file explorer each carry their own list. One shared virtual list with a filter and load-more would serve them and apps.

### marker
- **gpui-kit:** a compact chat status row, for rows like "Thinking…" (G/marker.rs:40-58).
  - Variants: Plain, Separator (divider lines on both sides) and Border (bottom border) (G/marker.rs:12-22, G/marker.rs:187-219).
  - Loading shows a spinner or a text shimmer (G/marker.rs:24-32, G/marker.rs:99-115, G/marker.rs:170-172, G/marker.rs:204-207).
  - It takes an icon slot, a content slot or any child (G/marker.rs:123-133, G/marker.rs:142-147).
  - It has no role by default. `.role(Role::Status)` plus `.id` makes it announce updates (G/marker.rs:82-91, G/marker.rs:223-228).
- **Ours:** absent. `rg -n -i 'spinner|Role::Status|throbber|loading_indicator|\bdivider\b|horizontal_rule|hr\b' src` finds only one-off status texts: "Loading..." and "No suggestions" (src/widgets/dialog/autocomplete/live.rs:402-418) and "Validating..." (src/widgets/dialog/input/live.rs:355).
- **In a terminal:** one row. Separator: `── ⠋ Thinking… ──`. Plain: `⠋ Thinking…`. Border: the text with a `─` row under it. The spinner is a cycling glyph, and the shimmer is a moving brighter color over the text cells. It takes no input. The screen reader gets a Status node whose text changes are announced.
- **Call:** skip as its own widget. Once Separator, Spinner and a shimmer exist, it is three children in a flex row.

### message
- **gpui-kit:** a chat message row with named slots: avatar, header (name, time), content (bubbles, images, code) and footer (delivery state, reactions, actions) (G/message.rs:63-132).
  - Start or end alignment (G/message.rs:8-16). End alignment mirrors the row, so the avatar sits on the right (G/message.rs:147-215).
  - MessageGroup stacks messages from one sender (G/message.rs:18-23).
  - It is layout only: `grep -c 'Role\|KeyBinding\|on_key\|use_keyed_state\|track_focus'` over G/message.rs:1-546 returns 0.
- **Ours:** absent. `rg -n -i 'struct \w*(message|chat|bubble|transcript|log_view|logview)\w*' src` finds only an internal AT-SPI type, and `rg -l -i 'chat|bubble' src/widgets src/builder` finds nothing.
- **In a terminal:** a header row (`alice  12:03`) in bold and dim text, then content indented two cells, optionally inside a box. The user's own messages are right-aligned. A footer row holds status or actions. There is no input unless the footer has buttons. The screen reader gets a group labeled with the sender and time. Avatars become one or two letters.
- **Call:** skip. It has no behavior, only slots; apps can build it from flex rows and borders.

### message_scroller
- **gpui-kit:** a virtual transcript that follows its tail (G/message_scroller.rs:164-212).
  - State:
    - Tail-follow is on by default (G/message_scroller.rs:36-50), with `is_scrolled_up` and `is_following_tail` (G/message_scroller.rs:57-67).
    - `splice`, `append` and `prepend` keep the scroll anchor (G/message_scroller.rs:76-117). Remeasure is available (G/message_scroller.rs:119-136).
    - `scroll_to_item`; `scroll_to_end` resumes following (G/message_scroller.rs:138-157).
  - View:
    - An optional scrollbar, and a jump-to-latest button with its own label, style, renderer and transition (G/message_scroller.rs:214-275, G/message_scroller.rs:409-426).
    - A bottom fade that shows only while away from the live edge (G/message_scroller.rs:277-288).
    - Role::Log on the viewport, so appended rows are announced (G/message_scroller.rs:365-368).
    - A mask that stops vertical wheel scrolling from leaking to an outer scroller (G/message_scroller.rs:404-408).
- **Ours:** absent.
  - ScrollView has axes, viewport size, scrollbars, smooth scroll and speed, but no tail-follow (src/widgets/layout/scroll_view.rs:13-120).
  - `ScrollState::scroll_to_bottom` is a one-shot jump (src/widgets/display/mod.rs:218-223).
  - `rg -n -i 'follow|tail\b|stick|auto_scroll|autoscroll|Role::Log|scroll_to_end|scroll_to_bottom' src/widgets src/builder` finds no follow mode and no Role::Log.
- **In a terminal:** a region that grows at the bottom and sticks to the last row while rows arrive. PageUp or the wheel stops following. A one-row `↓ new` bar, the End key or a click jumps back and resumes. Prepending older history keeps the visible rows still. The bottom fade does not translate; drop it. The screen reader gets Role::Log with polite announcements of appended rows.
- **Call:** build, medium. Logs, chat and build output are common TUI views. The anchor-keeping splice is the hard part.

### native_menu
- **gpui-kit:** a context menu drawn by the operating system, so it can extend past the window (G/native_menu/mod.rs:1-23, G/native_menu/mod.rs:67-74). It has macOS and Windows backends (G/native_menu/mod.rs:39-42).
  - Items dispatch gpui actions and can be disabled, checked, have an icon, or be separators and submenus (G/native_menu/mod.rs:49-65, G/native_menu/mod.rs:82-189). `show` pops it at a position (G/native_menu/mod.rs:196-201).
  - It can be built from existing gpui menus (G/native_menu/mod.rs:321-325).
  - On Linux it falls back to a drawn PopupMenu clipped to the window (G/native_menu/fallback.rs:1-5).
- **Ours:** no OS menu. Our drawn ContextMenu and PopupMenu cover the same item kinds: action, submenu, checkbox, radio and separator (src/widgets/menu/item.rs:81-103, src/widgets/menu/context.rs:6-15).
- **In a terminal:** there is no OS menu to open. The terminal screen is the whole window, so the clipping problem this solves does not exist; a menu is always drawn in cells.
- **Call:** skip. The drawn ContextMenu is the terminal answer.

### pagination
- **gpui-kit:** a page control (G/pagination.rs:19-31).
  - Prev and Next buttons with labels and tooltips (G/pagination.rs:99-139).
  - Numbered page buttons, with the current page outlined (G/pagination.rs:186-208).
  - Ellipsis buttons that open a dropdown of the hidden pages (G/pagination.rs:209-237).
  - `compact` makes prev/next icon-only (G/pagination.rs:85-91). `visible_pages` defaults to 5 (G/pagination.rs:42, G/pagination.rs:93-97). Disabled and size options exist.
  - The page math and Role::Navigation live in the base (B/pagination.rs:12-17, B/pagination.rs:153-160, B/pagination.rs:163-201).
- **Ours:** no standalone widget. DataTable has a PaginationConfig (src/widgets/display/data_table.rs:45-56) and draws Prev and Next buttons plus `n/total (count)` (src/widgets/display/data_table/live.rs:320-364). It has no page numbers, no ellipsis and no navigation role.
- **In a terminal:** one row: `‹ Prev  1 … 4 [5] 6 … 20  Next ›`, with the current page in reverse video.
  - Tab reaches it. Left/Right move between pages, Home/End go to the first or last, and Enter or a click picks a page.
  - The ellipsis opens a small popup list of the hidden pages.
  - The screen reader gets a labeled Navigation landmark with page buttons; the current page is marked current.
- **Call:** build, small. Move the DataTable bar into a widget and add the page math from B/pagination.rs:163-201.

### rating
- **gpui-kit:** a row of stars (G/rating.rs:11-22). Options: `max` (default 5), `value`, size, color (default theme yellow), disabled, and `on_click` with the new value (G/rating.rs:24-81). Hovering lights the stars up to the pointer. Clicking the current value takes one star off (G/rating.rs:142-195). It is mouse-only: no key handling and no role anywhere in G/rating.rs:1-203.
- **Ours:** absent; `rg -n -i 'rating|★|☆|\bstars?\b' src` finds no widget. The nearest is Slider: Role::Slider with value, min, max and step (src/widgets/input/slider.rs:453-461), and Arrow/PageUp/PageDown/Home/End keys (src/widgets/input/slider.rs:510-529).
- **In a terminal:** `★★★☆☆` in five cells, or `*` and `-` where the font lacks stars. Left/Right or `-`/`+` change the value. 0–9 set it directly. Home/End go to min and max. A click on a star sets it. The hover preview works only where the terminal reports mouse motion. The screen reader gets a Slider reading "3 of 5".
- **Call:** build, small. It is a Slider with a star painter and number keys.

### resizable
- gpui-kit: `resizable` is an inline module in the component crate that only re-exports base types (G/lib.rs:67-73, 107-110; B/lib.rs:147-151).
  - `h_resizable` and `v_resizable` build a panel group, and `resizable_panel` builds a panel (B/resizable/mod.rs:16-29).
  - `ResizableState` holds the sizes and can resize, insert, remove or reset a panel (B/resizable/mod.rs:31-33, 69, 95, 221, 233).
  - A panel has `visible`, `size` and `size_range` (B/resizable/panel.rs:269-283). The minimum size is 100 px (B/resizable/mod.rs:14).
  - The group has an axis, `on_resize` and a custom handle look (B/resizable/panel.rs:59, 73, 111).
  - The handle is a pointer drag with a resize cursor (B/resizable/resize_handle.rs:14-16, 165-182). It has no role, focus or key handling (`grep -c -i 'role\|focus\|keybinding\|on_action'` returns 0 for both resize_handle.rs and panel.rs).
- Ours: absent. The searches are the same as for dock. We only have resizing inside single widgets: table column drag (src/widgets/display/table.rs:86; src/widgets/display/table/live.rs:52-53, 641-652) and a `resizable` modal flag (src/widgets/display/modal.rs:49).
- In a terminal: Panes separated by a one-cell `│` or `─` divider that highlights on hover and while dragging. Dragging needs button-motion mouse reporting, which only the DirectTty backend turns on today (src/backend/direct_tty.rs:69; see "Defects found" in the summary). Sizes are whole cells with a minimum. Keyboard: the divider takes focus with Tab. Arrows move it one cell, a modifier moves it further, and Home/End jump to the minimum or maximum. Screen reader: the Splitter role, which our bridge maps to a separator (src/accessibility/platform/translation/node.rs:255), with the size as its value. That is more than gpui-kit offers.
- Call: build, medium. Split panes are basic TUI layout, and the table's drag code shows the mouse part.

### separator
- **gpui-kit:** a 1px horizontal or vertical line (G/separator.rs:15-24). It can be solid or dashed (G/separator.rs:7-13, G/separator.rs:26-59, G/separator.rs:73-77, G/separator.rs:90-117), take a custom color (G/separator.rs:67-71), and show a centered text label (G/separator.rs:61-65, G/separator.rs:142-153). There is no accessibility role anywhere in G/separator.rs:1-155.
- **Ours:** no standalone separator.
  - Menus draw separator rows in Line, ThickLine, DoubleLine, Dashed, Dotted and Space styles, horizontal or vertical, with Role::Splitter (src/widgets/menu/item.rs:61-79, src/widgets/menu/view.rs:82-116).
  - Markdown rules print a fixed 28-cell `─` line (src/markdown/ast_walker.rs:229-234).
- **In a terminal:** a full-width row of `─` (or `╌`, `═`), or a column of `│`. With a label: `──── Label ────`. It takes no input. The screen reader gets Splitter, which our AT-SPI bridge reports as Separator (src/accessibility/platform/translation/node.rs:255).
- **Call:** build, small. The glyph table already exists in the menu view.

### setting
- **gpui-kit:** a full settings screen (G/setting/settings.rs:21-44).
  - A resizable sidebar of pages with a search box (G/setting/settings.rs:62-72, G/setting/settings.rs:136-158). The sidebar is gpui-kit's Sidebar (G/setting/settings.rs:142).
  - Pages with title, icon, description and groups (G/setting/page.rs:20-106).
  - Items with one field each: switch, checkbox, number, text input, dropdown or a custom element (G/setting/fields/mod.rs:60-75, G/setting/fields/mod.rs:156-263).
  - Search hides groups that do not match (G/setting/settings.rs:222-235).
  - Fields reset to a default value or through a custom reset (G/setting/fields/mod.rs:21-24, G/setting/fields/mod.rs:291-316). Pages show a reset button when something changed (G/setting/page.rs:85-91).
- **Ours:** absent; `rg -n -i 'struct \w*(setting|preference|form)\w*|fn form\b|FormBuilder|PropertyGrid' src` finds none. The nearest composites are WizardDialog steps with per-step validation (src/widgets/dialog/wizard.rs:26-56) and Tabs placed on the left or right (src/widgets/layout/tabs.rs:47-56).
- **In a terminal:** a left column of pages with a filter row. The right pane holds rows like `Label ........ [ on]`, each with a dim description line. Tab and the arrows move between rows, Space or Enter edits, and a Reset button restores the default. The screen reader gets a labeled group per row, with the field's own role.
- **Call:** skip. TUIs usually keep settings in a config file. The missing parts, Switch and Sidebar, are on the build list, and with them an app can build this.

### sheet
- **gpui-kit:** a panel that slides in from one edge of the window (G/sheet.rs:40-56).
  - Placement top, right (the default), bottom or left, and a size (default 350px) (G/sheet.rs:59-74).
  - An overlay that closes on click, a title row with a close button, a footer, and a scrolling body (G/sheet.rs:100-117, G/sheet.rs:175-231).
  - A 0.15 s slide from its own edge (G/sheet.rs:232-244) and a top margin that clears the title bar (G/sheet.rs:25-38).
  - `resizable` is stored and settable (G/sheet.rs:47, G/sheet.rs:65, G/sheet.rs:94-97), but grep over G/sheet.rs:1-260 finds no other use of it.
  - The base host owns the focus trap, Escape and overlay dismissal (B/sheet.rs:23-30, B/sheet.rs:128-160).
- **Ours:** Modal (src/widgets/display/modal.rs:9-68).
  - Position Left/Right/Top/Bottom or a corner (src/widgets/display/modal.rs:85-111), and size in cells, percent or viewport fraction (src/widgets/display/modal.rs:72-81).
  - Backdrop-click close (src/widgets/display/modal.rs:25), an Escape close reason (src/widgets/display/modal.rs:130-139), and a focus trap (src/widgets/display/modal.rs:57).
  - A title and a footer (src/widgets/display/modal.rs:12-13, src/widgets/display/modal.rs:52-53).
  - Resizable and draggable (src/widgets/display/modal.rs:48-51), and a Slide animation (src/widgets/display/modal.rs:115-121).
- **Differences:**
  - The sheet spans its whole edge. Our Modal centers its box along the edge (src/widgets/display/modal.rs:442-446), so a full-height side panel needs `Viewport(1.0)` set by hand.
  - Our Slide always moves down from 3 rows above (src/widgets/display/modal/live/render.rs:253-255). The sheet slides in from its own edge (G/sheet.rs:232-244).
  - Ours can be dragged and resized with handles (src/widgets/display/modal/live/events.rs:122-134). The sheet's resize flag has no effect.
- **Worth adopting:**
  - A `Modal` sheet preset that sets the edge position and full length in one call.
  - A slide that starts from the placement edge.

### shimmer
- **gpui-kit:** ShimmerText sweeps a soft bright band across text (G/shimmer.rs:127-144). Options: sweep duration (default 2 s), highlight color, spread as a fraction or a fixed width, reverse, and once (G/shimmer.rs:45-108, G/shimmer.rs:169-197). Under reduced motion it shows plain text and requests no frames (G/shimmer.rs:130-131, G/shimmer.rs:210-214).
- **Ours:** absent; `rg -n -i 'shimmer|skeleton' src` finds nothing. We have per-element opacity animation (`pulse`, src/layout/css/animations.rs:62-72) and reduced-motion handling (src/app/motion.rs:156-159), but no per-character color sweep.
- **In a terminal:** a few cells of the text get a brighter foreground each frame, and that window moves across the text. On 16-color terminals it can use bold instead. It takes no input. The screen reader reads only the text. It needs a repaint roughly every 50 ms while visible, and it stops under reduced motion.
- **Call:** build, small. It is one color function over one text run. It marks busy text without a spinner glyph, and the marker and loading rows can use it.

### sidebar
- **gpui-kit:** a navigation sidebar (G/sidebar/mod.rs:220-297).
  - Collapse modes: Icon, Offcanvas or None (G/sidebar/mod.rs:31-46). Left or right side (G/sidebar/mod.rs:251-257). Header and footer (G/sidebar/mod.rs:275-285). A toggle button (G/sidebar/mod.rs:300-340).
  - Labeled groups (G/sidebar/group.rs:7-34).
  - Items with an icon, active state, nested children, default_open, click-to-toggle, a suffix, disabled and a context menu (G/sidebar/menu.rs:94-109, G/sidebar/menu.rs:133-225).
  - Collapsed items show their tooltip on hover (G/sidebar/menu.rs:379).
  - It has no role and no key handling: `grep -rn 'Role\|on_key\|KeyBinding\|track_focus'` over G/sidebar/mod.rs:1-769, G/sidebar/menu.rs:1-443, G/sidebar/group.rs:1-86, G/sidebar/header.rs:1-100 and G/sidebar/footer.rs:1-88 finds none.
- **Ours:** absent as a widget.
  - `sidebar()` is only a styled aside, `w-64 bg-gray-800 text-white` (src/builder/layout.rs:26-29).
  - Tabs can sit on the left or right (src/widgets/layout/tabs.rs:47-56).
  - Tree has nested nodes that expand (src/widgets/display/tree.rs:21-22, src/widgets/display/tree.rs:84-88).
- **In a terminal:** a column 20–30 cells wide with optional header and footer rows. Group labels are dim. Items show an icon glyph and a label, and the active item is highlighted. Nested items are indented with `▸`/`▾`.
  - Collapsed mode leaves one glyph column.
  - Up/Down move, Right/Left open or close a group, Enter activates, and a key collapses the whole bar. Clicks work.
  - Hover tooltips on collapsed icons do not carry over; show the label on focus instead.
  - The screen reader gets a Navigation landmark holding a Tree or a List.
- **Call:** build, medium. Apps with several views need a navigation column, and today `sidebar()` is only a styled box with no keys, state or roles (src/builder/layout.rs:26-29).

### skeleton
- **gpui-kit:** a full-width, one-line block in the theme's skeleton color, with an optional half-opacity secondary color (G/skeleton.rs:8-29). It pulses opacity from 1 to 0.5 over 2 s (G/skeleton.rs:37-58).
- **Ours:** no Skeleton widget, but the same pulse exists as a class. `pulse` goes from opacity 1.0 to 0.5 over 2000 ms and loops (src/layout/css/animations.rs:62-72). `animate-pulse` applies it (src/layout/css/animations.rs:366, src/layout/css/animations.rs:490-494), and reduced motion stops it (src/app/motion.rs:156-159).
- **In a terminal:** rows of `░` or dim background cells in the shape of the content to come, pulsing between dim and normal. It takes no input. The screen reader should skip it and read a "Loading" status on the parent instead.
- **Call:** build, small. A preset element over `animate-pulse`.

### spinner
- **gpui-kit:** a loader icon rotating in a loop, 0.8 s with ease-in-out (G/spinner.rs:8-28, G/spinner.rs:60-75). Options: icon, color, easing and size (G/spinner.rs:30-58). Under reduced motion it is static and requests no frames (test at G/spinner.rs:90-102).
- **Ours:** absent.
  - The indeterminate ProgressBar moves a filled band (src/widgets/display/progress_bar.rs:87-91, src/widgets/display/progress_bar/live.rs:295-298).
  - `animate-spin` rotates the element's transform (src/layout/css/animations.rs:88-99, src/layout/css/animations.rs:364, src/layout/css/animations.rs:478-482). A rotated glyph cannot look like it is turning inside one cell.
- **In a terminal:** one or two cells cycling frames: `⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏` (braille), `|/-\` in ASCII, or `◐◓◑◒`, at about 10 frames per second.
  - It can fall back to ASCII the way charts already do when the terminal reports no Unicode support (src/core/capabilities.rs:394-402).
  - It takes no input. The screen reader gets a labeled status, announced once, not every frame. Under reduced motion it shows a static glyph.
- **Call:** build, small. Marker rows, list loading and busy buttons would all use it.

### status_bar
- **gpui-kit:** a bottom row with left, center and right regions (G/status_bar.rs:9-37). The center aligns according to which ends are filled (G/status_bar.rs:20-23, G/status_bar.rs:77-106). It has a top border, a status-bar background and small muted text (G/status_bar.rs:86-95). There is no role and no input anywhere in G/status_bar.rs:1-107.
- **Ours:** absent. `footer()` is a plain block container (src/builder/core.rs:39-42), used as a centered footer in the builder example (src/builder/mod.rs:118-121).
- **In a terminal:** the last screen row, vim/tmux style, with reverse video or its own background. Left items pack left, right items pack right, and the center is centered. On overflow, text is cut with `…`, center first. Items can be plain text or Tab-reachable buttons. The screen reader gets a Status region, or a toolbar if it holds buttons.
- **Call:** build, small. Most full-screen terminal apps keep the mode, the position and key hints on the last row, and today each app has to draw that row itself from `footer()` (src/builder/core.rs:39-42).

### stepper
- **gpui-kit:** a row or column of steps with a current step; earlier steps are marked as passed (G/stepper/stepper.rs:10-22, G/stepper/item.rs:117).
  - Options: vertical layout, centered text, disabled and size (G/stepper/stepper.rs:42-82). Each item takes an icon and a disabled flag (G/stepper/item.rs:46-58). A click handler reports the step (G/stepper/stepper.rs:84-93).
  - Items are Role::ListItem with a position (G/stepper/item.rs:125-129).
  - Clicks only: grep for `Role`, `KeyBinding`, `on_key` and `track_focus` over G/stepper/stepper.rs:1-135, G/stepper/item.rs:1-266 and G/stepper/trigger.rs:1-153 finds only that role, at G/stepper/item.rs:128.
- **Ours:** absent as a widget. WizardDialog shows "Step N of M" text and a ProgressBar when `show_progress` is set (src/widgets/dialog/wizard/live.rs:174-191, src/widgets/dialog/wizard.rs:43-51).
- **In a terminal:** `✓ Account ── ● Profile ── ○ Confirm` on one row, or one step per row joined by `│`. The current step is bold or in reverse video. If steps are clickable: Tab focuses the bar, Left/Right move, Enter jumps. The screen reader gets a List whose items read like "Step 2 of 3, Profile, current".
- **Call:** build, small. WizardDialog could then use it in place of its text and bar.

### switch
- **gpui-kit:** a labeled on/off toggle (G/switch.rs:12-46).
  - Options: checked, label, a separate accessibility label, on_change/on_click, color, tooltip, size and disabled (G/switch.rs:48-105).
  - `label_side` defaults to Right and flips the row when Left (G/switch.rs:22, G/switch.rs:41, G/switch.rs:199).
  - The thumb moves with a spring animation (G/switch.rs:9, G/switch.rs:167-176).
  - The base gives Role::Switch with a toggled state, focus with tab index and tab stop, and reports the requested value through on_change (B/switch.rs:15-20, B/switch.rs:345-385).
- **Ours:** Checkbox (src/widgets/input/checkbox.rs:8-60, src/widgets/input/checkbox.rs:127-181). It paints `[✓] Label`, toggles on Space/Enter (src/widgets/input/checkbox.rs:228-240), and reports Role::CheckBox with toggled True, False or Mixed (src/widgets/input/checkbox.rs:164-175).
- **Differences:**
  - Only ours has an indeterminate (Mixed) state (src/widgets/input/checkbox.rs:142-143, src/widgets/input/checkbox.rs:169-170).
  - Only gpui-kit has the Switch role, an accessibility label separate from the visible one, a label on the left, a tooltip, and color and size options.
  - Our checkbox is one string, with `▶` for focus, `🔒` for disabled and `_label_` for hover (src/widgets/input/checkbox.rs:130-161).
  - The spring animation does not translate to cells.
- **Worth adopting:**
  - A switch variant that paints `[● ]`/`[ ●]` or `(on )`/`(off)` and reports Role::Switch.
  - An accessibility label separate from the visible label.
- **In a terminal:** `[ ●] Wi-Fi` when on and `[● ] Wi-Fi` when off, or `(on )` and `(off)` where the font lacks `●`. Space, Enter or a click toggles it. The screen reader gets Role::Switch, which is read as on or off rather than checked.
- **Call:** build, small. It is our checkbox with another paint and role; the state and the keys stay (src/widgets/input/checkbox.rs:228-240). Settings screens and option lists read better with switches.

### tag
- **gpui-kit:** a small status label (G/tag.rs:121-131). Variants: primary, secondary (the default), danger, success, warning, info, a named color, or custom colors (G/tag.rs:8-24, G/tag.rs:146-188). It also has an outline style (G/tag.rs:196-200) and rounded corners (G/tag.rs:202-211). Only Medium and Small sizes are supported (G/tag.rs:122-123). There is no input and no role anywhere in G/tag.rs:1-269.
- **Ours:** no standalone tag. Tabs carry a TabBadge with Default, Success, Warning, Error and Info variants, painted as a glyph plus text in the tab label, such as `●5` or `✗5` (src/widgets/layout/tabs.rs:67-104, src/widgets/layout/tabs.rs:554-563).
- **In a terminal:** padded text on a colored background (` done ` on green), or `[ bug ]`. Outline becomes brackets in the variant color. Rounded corners do not translate. It takes no input. The screen reader reads the text, optionally prefixed with the variant ("Warning: 3").
- **Call:** build, small. Tabs could then use it for their badges.

### text
- **gpui-kit:** G/text/mod.rs:1-16 re-exports the base TextView (B/text/text_view.rs:100-117):
  - Renders Markdown (GFM) or simple HTML (B/text/text_view.rs:183, B/text/text_view.rs:205).
  - Selectable with a copy format, scrollable, and max lines (B/text/text_view.rs:233-290).
  - Code-block and table action slots and a code highlighter (B/text/text_view.rs:291-331), and a link click handler (B/text/text_view.rs:333-340).
  - Markdown extensions, MDX, custom block parsers and renderers, and plugins (B/text/text_view.rs:342-410).
  - A motion policy that fades streamed text in (B/text/text_view.rs:349-356, B/text/stream_fade.rs:1-7).
  - The state has `push_str` for streaming and `selected_text` (B/text/state.rs:343, B/text/state.rs:383).
  - `Text` is a string-or-TextView enum used for labels (B/text/mod.rs:46-50). G adds a front-matter plugin and tree-sitter highlighting (G/text/mod.rs:10, G/text/mod.rs:75-118).
- **Ours:** MarkdownRenderer turns CommonMark/GFM into styled lines, with syntax-highlighted code blocks and source positions (src/markdown/mod.rs:1-23, src/markdown/renderer.rs:9-16, src/markdown/renderer.rs:63-95, src/markdown/ast_walker.rs:158-170).
  - It is not a widget: `rg -n 'MarkdownRenderer|render_markdown|markdown::' src --glob '!markdown/**'` finds no caller outside src/markdown.
  - Links are styled, but the URL is dropped (src/markdown/ast_walker.rs:273-281).
  - Images become `[Image: url]` (src/markdown/ast_walker.rs:283-294).
  - There is no HTML input.
- **In a terminal:** a read-only, scrollable pane of wrapped styled lines.
  - Headings in color and bold; code blocks with a background and highlighting; tables with box-drawing lines.
  - Links are OSC 8 hyperlinks that are also Tab stops; Enter opens them.
  - A mouse drag, or a keyboard selection mode, selects text for copying.
  - Streaming appends text; the fade-in does not translate, so new text shows at once.
  - The screen reader gets a Document with headings, paragraphs, lists and links as nodes.
- **Call:** build, medium. The renderer exists. The missing part is a Markdown view element with scrolling, link activation and append.

### time
- **gpui-kit:** two widgets.
  - **Calendar** (G/time/calendar.rs:36-66, over B/calendar.rs):
    - A single date or a range (B/calendar.rs:12-17), and day, month and year views (B/calendar.rs:139-144).
    - Disabled dates by a list, an interval, a range or a custom function (B/calendar.rs:84-126).
    - Several months side by side, and a first weekday that defaults to Sunday (G/time/calendar.rs:43-66).
    - A year range and month paging (B/calendar.rs:279-357).
  - **DatePicker** (G/time/date_picker.rs:274-380):
    - A trigger with a popup calendar, and a date format that defaults to `%Y/%m/%d` (G/time/date_picker.rs:143-147).
    - Presets such as a named range (G/time/date_picker.rs:41-70, G/time/date_picker.rs:362-366) and a clear button (G/time/date_picker.rs:356-360).
    - Keys: Enter, Escape, Delete and Backspace (G/time/date_picker.rs:26-33). It emits a Change event (G/time/date_picker.rs:35-39).
  - I found no arrow-key day navigation: `grep -n 'KeyBinding\|on_key_down'` over B/calendar.rs:1-1078 finds none.
- **Ours:** absent. `rg -n -i 'calendar|date_?picker|NaiveDate|chrono|weekday|time_?picker' src Cargo.toml` finds only a DataTable ISO date check for column filters (src/widgets/display/data_table.rs:299-315). The `time` crate is already a dependency (Cargo.toml:64).
- **In a terminal:** a 7-column month grid, about 21 cells wide and 8 rows high.
  - A `‹ September 2026 ›` header, weekday initials, and two-cell day numbers.
  - Today is underlined, the selected day is in reverse video, range days get a background, and disabled days are dim.
  - Keys: arrows move by day and week, PageUp/PageDown by month, Shift+PageUp/PageDown by year. Enter selects, Esc closes. A click picks a day.
  - The picker is a one-line input that opens the grid as a popover and also accepts a typed date.
  - The screen reader gets a Grid whose cells are labeled with the full date and selected state.
- **Call:** build, medium. Keyboard grid navigation should be part of it from the start.

### tooltip
- **gpui-kit:** a tooltip with text or a custom element, plus the key binding of an action (G/tooltip.rs:32-81, G/tooltip.rs:94-100).
  - Button, Switch, Checkbox and Radio embed it as `.tooltip()` (G/tooltip.rs:194-197).
  - It opens on hover (G/tooltip.rs:259-275), after a 500 ms delay, with a 300 ms grace period when moving between triggers (B/tooltip.rs:13-14).
  - It has Role::Tooltip (B/tooltip.rs:19-31), a positioner that flips and clamps (B/tooltip.rs:250-262), and slide animations (G/tooltip.rs:148-151).
  - There is no focus trigger: `grep -i focus` over G/tooltip.rs:1-295 and B/tooltip.rs:1-347 finds none.
- **Ours:**
  - Popover, with a Click, Hover, Focus or Manual trigger (src/widgets/display/popover.rs:75-86), hover show/hide delays of 100/300 ms (src/widgets/display/popover.rs:165-168, src/widgets/display/popover.rs:246-247), and flipping at the edge (src/widgets/display/popover.rs:123-128, src/widgets/display/popover.rs:242).
  - Tabs and Breadcrumb each draw their own hover tooltip row (src/widgets/layout/tabs.rs:770-777, src/widgets/layout/breadcrumb/live.rs:394-406). Breadcrumb also puts the tooltip text into the accessibility description (src/widgets/layout/breadcrumb/live.rs:200-202).
- **Differences:**
  - gpui-kit has one shared tooltip that any control can use, the Tooltip role, key-binding display, and a grace period between triggers.
  - Our popover has a Focus trigger, so keyboard users can see it.
  - Our popover body is Dialog or Group, not Tooltip (src/widgets/display/popover/live/render.rs:226-231).
  - Our per-widget tooltips show only on hover.
  - The slide animations do not translate.
- **Worth adopting:**
  - A shared `.tooltip(text)` for focusable elements that shows on focus as well as hover, and sets Role::Tooltip with a described-by link.
  - Showing an action's key binding inside the tooltip.

## Support code

Modules of gpui-kit's component crate that are plumbing, window chrome or re-exports rather than widgets.

### actions (support code)
- An inline module (G/lib.rs:21-23) that re-exports B/actions.rs: the shared keyboard actions Confirm (with a secondary flag), Cancel, SelectUp, SelectDown, SelectLeft, SelectRight, SelectFirst, SelectLast, SelectPrevColumn, SelectNextColumn, SelectPageUp and SelectPageDown (B/actions.rs:6-28). Widgets handle these named actions and the app's keymap binds keys to them, so keys can be rebound in one place; the select and the sheet both use Cancel (G/select.rs:14; G/sheet.rs:9).
- Ours: no shared action set. Each widget matches key codes in its own handler (src/widgets/layout/tabs.rs:876-878; src/widgets/input/select.rs:670-672). A search of src for `KeyBinding`, `keymap` and `keybind` finds only the accessibility action translation. Item 7 of the summary says what adopting this would mean.

### component_traits (support code)
- Re-exports the Collapsible, Disableable and Selectable builder traits (G/component_traits.rs:1-2). They are defined, along with FocusableExt, in B/component_traits.rs:1-30. We have no shared builder traits. Our state lives in StateFlags bits (VISIBLE, ENABLED, SELECTED, CHECKED, EXPANDED and more) with the StateFlagged trait (src/component/state_flags.rs:3-33, src/component/state_flags.rs:98-120), plus a `disabled` field on every element (src/component/element.rs:84-85).

### element_ext (support code)
- `ChildElement` and `AnyChildElement` pass an index and a size into a child before it renders (G/element_ext.rs:13-34). The base `ElementExt` adds `on_prepaint`, which reads the element's final bounds, and a text-selection scope (B/element_ext.rs:5-31). Ours is a crate-internal list of layout callbacks on each element (src/component/element.rs:7, src/component/element.rs:72). Breadcrumb uses it to measure segments (src/widgets/layout/breadcrumb/live.rs:169-175). It is not public.

### global_state (support code)
- App-wide state for base behaviors: touch-press tracking, text-selection suppression, app menus and a deferred-popover registry (G/global_state.rs:1-9, B/global_state.rs:7-15, B/global_state.rs:42-80, B/global_state.rs:108-137). We keep this per App: the App struct owns focus, routing, motion and accessibility (src/app.rs:98-142). The theme is process-wide through `Theme::set_active` (src/app.rs:145-148).

### history (support code)
- Re-exports `History` and `UndoHistory` (G/history.rs:1). `History` is a browser-style back/forward trail with a maximum size (B/history.rs:1-113). `UndoHistory` is grouped undo/redo with a maximum size and a grouping interval (B/undo_history.rs:3-128). We have only TextInput's own undo and redo stacks, capped at 100 (src/widgets/input/text_input.rs:340-342, src/widgets/input/text_input.rs:511-518, src/widgets/input/text_input.rs:547-566). There is no shared history type.

### index_path (support code)
- Re-exports `IndexPath` (section, row, column) (G/index_path.rs:1, B/index_path.rs:5-66). List uses it in its events (G/list/list.rs:38-45). We have no shared type; menus address items by a `Vec<usize>` path (src/widgets/menu/view.rs:82, src/widgets/menu/view.rs:118).

### inspector (support code)
- A developer tool. Ctrl+Shift+I (Cmd+Alt+I on macOS) toggles gpui's element inspector, which has Rust and JSON editors for the picked element's style (G/inspector.rs:28-62, G/inspector.rs:73-83). We have only a debug stats line (FPS, frame, diff and write times), drawn when App debug is on (src/app.rs:223-225, src/core/renderer.rs:290-300).

### root (support code)
- The top view of each window. It owns the dialog, sheet, notification, tooltip and fallback-menu layers, and keeps the handle focused before a sheet or dialog opened so focus can go back when it closes (G/root.rs:34-74, G/root.rs:297-484). Ours: App is the root (src/app.rs:98-142). Dialogs and toasts come from a DialogEngine whose host element the app renders (src/widgets/dialog/engine.rs:316-322, src/widgets/dialog/engine.rs:459-463).

### searchable_list (support code)
- Search and selection plumbing shared by Select and ComboBox, not a standalone widget (G/searchable_list/state.rs:12-17, G/searchable_list/adapter.rs:13-17).
  - Items define `matches`, which defaults to a case-insensitive substring match on the title (G/searchable_list/delegate.rs:8, G/searchable_list/delegate.rs:43).
  - Delegates define `perform_search`, sections, `is_item_checked` and `on_will_change` (G/searchable_list/delegate.rs:54, G/searchable_list/delegate.rs:86, G/searchable_list/delegate.rs:141, G/searchable_list/delegate.rs:167).
  - SearchableVec filters items in memory (G/searchable_list/vec.rs:71-76, G/searchable_list/vec.rs:130-139).
- Our equivalents are per widget: Select's type-ahead (src/widgets/input/select.rs:698-705, src/widgets/input/select.rs:711-755) and the autocomplete substring or custom filter (src/widgets/dialog/autocomplete.rs:737-757).

### sizing (support code)
- A `Size` enum (XSmall, Small, Medium, Large, or pixels) with helpers for table row height and for cell and input padding, plus the `Sizable` trait most components implement (G/sizing.rs:4-13, G/sizing.rs:55-166, G/sizing.rs:175-183). Ours has only per-widget size enums: TabSize (src/widgets/layout/tabs.rs:34-43), ModalSize (src/widgets/display/modal.rs:72-81) and DisplaySize (src/widgets/display/mod.rs:52-63). There is no shared trait. In cells, "size" mostly means padding and row count.

### styled (support code)
- Style helpers:
  - Re-exports of `h_flex`/`v_flex`, StyledExt and the component traits (G/styled.rs:1-7, B/styled.rs:38-48, B/styled.rs:78).
  - Shadow and ring recipes tuned to match shadcn/ui (G/styled.rs:14-56).
  - ThemeStyled, for theme-driven looks such as the focus ring (G/styled.rs:125-130).
- Ours: StyleBuilder (src/layout/style.rs:218), utility classes under src/layout/css, and `flex_row`/`flex_col` helpers (src/builder/layout.rs:119-139). Shadows and blur do not translate to cells.

### title_bar (support code)
- Window chrome: a custom 34px title row that holds app content and the minimize, maximize, restore and close controls. It moves the window and handles double clicks (G/title_bar.rs:15, G/title_bar.rs:38-46, G/title_bar.rs:67-91, G/title_bar.rs:108-118). A terminal app does not own its window. Our equivalent is setting the terminal title with OSC 2 or OSC 0 (src/core/terminal.rs:399-409, src/component/change.rs:109-110, src/component/change.rs:466).

### touch_selection (support code)
- Grab handles and an edit menu drawn over a text selection made by a long press (G/touch_selection/mod.rs:1-7, B/touch_selection.rs:1-10). Terminals do not report touch. Ours has mouse-drag and Shift selection in TextInput (src/widgets/input/text_input.rs:1202-1237), and no touch events: `rg -n -i touch src/event` finds none.

### window_border (support code)
- Window chrome for Linux client-side decorations: a shadow, a frame and resize hit bands (G/window_border.rs:16-35, G/window_border.rs:52-66, G/window_border.rs:87-100). We have no equivalent. The terminal emulator draws the window, and we only follow its size (src/app.rs:221-222).

### window_ext (support code)
- A trait on Window that opens and closes sheets and dialogs, opens alert dialogs, and pushes and removes notifications (G/window_ext.rs:11-78). Ours is the DialogEngine: show_confirmation, show_input, show_toast, show_wizard and other show methods (src/widgets/dialog/engine.rs:550-585), plus `close_dialog` and `close_all` (src/widgets/dialog/engine.rs:451-458) and `active_count` (src/widgets/dialog/engine.rs:416-418). It has no sheet opener.

## Coverage

Every module G/lib.rs declares (80), with the line that declares it and the part of this study that covers it.

- `accordion` (G/lib.rs:25): families both libraries have
- `actions` (G/lib.rs:21): support code
- `alert` (G/lib.rs:26): components we lack
- `attachment` (G/lib.rs:27): components we lack
- `avatar` (G/lib.rs:28): components we lack
- `badge` (G/lib.rs:29): components we lack
- `breadcrumb` (G/lib.rs:30): families both libraries have
- `bubble` (G/lib.rs:31): components we lack
- `button` (G/lib.rs:32): components we lack
- `carousel` (G/lib.rs:33): components we lack
- `chart` (G/lib.rs:34): families both libraries have
- `checkbox` (G/lib.rs:35): families both libraries have
- `clipboard` (G/lib.rs:36): components we lack
- `collapsible` (G/lib.rs:37): components we lack
- `color_picker` (G/lib.rs:38): components we lack
- `combobox` (G/lib.rs:39): components we lack
- `command` (G/lib.rs:40): components we lack
- `component_traits` (G/lib.rs:4): support code
- `description_list` (G/lib.rs:41): components we lack
- `dialog` (G/lib.rs:42): families both libraries have
- `dock` (G/lib.rs:43): components we lack
- `element_ext` (G/lib.rs:5): support code
- `empty` (G/lib.rs:44): components we lack
- `form` (G/lib.rs:45): components we lack
- `global_state` (G/lib.rs:6): support code
- `group_box` (G/lib.rs:46): components we lack
- `highlighter` (G/lib.rs:47): families both libraries have
- `history` (G/lib.rs:48): support code
- `hover_card` (G/lib.rs:49): components we lack
- `icon` (G/lib.rs:7): components we lack
- `index_path` (G/lib.rs:8): support code
- `input` (G/lib.rs:50): families both libraries have
- `inspector` (G/lib.rs:10): support code
- `kbd` (G/lib.rs:51): components we lack
- `label` (G/lib.rs:52): components we lack
- `link` (G/lib.rs:53): components we lack
- `list` (G/lib.rs:54): components we lack
- `marker` (G/lib.rs:55): components we lack
- `menu` (G/lib.rs:56): families both libraries have
- `message` (G/lib.rs:57): components we lack
- `message_scroller` (G/lib.rs:58): components we lack
- `native_menu` (G/lib.rs:59): components we lack
- `notification` (G/lib.rs:60): families both libraries have
- `pagination` (G/lib.rs:61): components we lack
- `plot` (G/lib.rs:62): families both libraries have (with chart)
- `popover` (G/lib.rs:63): families both libraries have
- `progress` (G/lib.rs:64): families both libraries have
- `radio` (G/lib.rs:65): families both libraries have
- `rating` (G/lib.rs:66): components we lack
- `resizable` (G/lib.rs:68): components we lack
- `root` (G/lib.rs:11): support code
- `scroll` (G/lib.rs:74): families both libraries have
- `searchable_list` (G/lib.rs:75): support code
- `select` (G/lib.rs:76): families both libraries have
- `separator` (G/lib.rs:77): components we lack
- `setting` (G/lib.rs:78): components we lack
- `sheet` (G/lib.rs:79): components we lack
- `shimmer` (G/lib.rs:80): components we lack
- `sidebar` (G/lib.rs:81): components we lack
- `sizing` (G/lib.rs:12): support code
- `skeleton` (G/lib.rs:82): components we lack
- `slider` (G/lib.rs:83): families both libraries have
- `spinner` (G/lib.rs:84): components we lack
- `status_bar` (G/lib.rs:85): components we lack
- `stepper` (G/lib.rs:86): components we lack
- `styled` (G/lib.rs:13): support code
- `switch` (G/lib.rs:87): components we lack
- `tab` (G/lib.rs:88): families both libraries have
- `table` (G/lib.rs:89): families both libraries have
- `tag` (G/lib.rs:90): components we lack
- `text` (G/lib.rs:91): components we lack
- `theme` (G/lib.rs:92): families both libraries have
- `time` (G/lib.rs:14): components we lack
- `title_bar` (G/lib.rs:15): support code
- `tooltip` (G/lib.rs:93): components we lack
- `touch_selection` (G/lib.rs:16): support code
- `tree` (G/lib.rs:94): families both libraries have
- `virtual_list` (G/lib.rs:17): families both libraries have
- `window_border` (G/lib.rs:18): support code
- `window_ext` (G/lib.rs:19): support code


## Update for gpui-kit 0.7.0

Date: 2026-09-29. Read at reactive-tui 78b1b5de and gpui-kit 0.7.0, with
gpui-kit 0.6.6 beside it to see what changed.

The developer asked on 2026-09-28 for the widgets to be compared with the
new gpui-kit before the widget work is specified. Four reviewers each read
one part of the two libraries: the charts, the input widgets, the display
and layout widgets, and the overlays, the menus and the theme. They read
code and ran `diff` and `grep`; they built and ran nothing, so each
statement says what the code does, not how it looks. Each report follows
as its reviewer wrote it, with its own path shorthand at its head. The
line numbers are those of the commit named above.

What has changed in reactive-tui since the reports were written: the
commitment layout-cells (done on 2026-09-29) made the padding, margin, gap
and space classes count in cells, made a gap the number of cells it asks
for at every width, corrected `gap-x-N`, `gap-y-N`, `col-span-full` and the
auto-fit classes, and stopped the data table's panels from growing. The
statements about those in the display and layout report describe the code
before that commitment. The order of the widget work that uses these
reports is in docs/spec/roadmap.md.


### Charts: Chart comparison: gpui-kit 0.6.6, gpui-kit 0.7.0 and reactive-tui

Read-only review. I ran `diff`, `grep`, `sed` and `cat -n` only. I did not build, run or render anything.
I cannot see output, so every statement says what the code does, not how it looks.

Path shorthand used below:

- `G7` = ~/workspace2/gpui-kit-0.7.0/crates
- `G6` = ~/workspace2/gpui-kit-0.6.6/crates
- `R` = ~/workspace2/reactive-tui/src/widgets/display/charts
- `RT` = ~/workspace2/reactive-tui
- A bare `charts.rs:N` = RT/src/widgets/display/charts.rs. Other bare names are files under R: `typed.rs`, `live.rs`, `mask.rs`; `canvas.rs` and `motion.rs` are in R/live/; `cartesian.rs`, `pie.rs`, `radar.rs`, `sankey.rs` are in R/live/canvas/; `tick.rs`, `axis.rs`, `scale.rs`, `layout.rs`, `decimate.rs`, `curve.rs`, `polar.rs`, `tooltip.rs` (reactive-tui's) are in R/plot/. G7 files are named with their `G7/` path or `<type>_chart.rs`.
- G7 chart files are `G7/component/src/chart/<type>_chart.rs`. G7 plot primitives are `G7/base/src/plot/`.
- Marks: [obs] = read in code. [assume] = my inference, not checked.

---

#### Part 1. What changed from 0.6.6 to 0.7.0

Method: `diff -r` of `component/src/chart` (both), `component/src/plot` (6.6) against `base/src/plot` (7.0) and the new `component/src/plot` (mod.rs, tooltip.rs). Release notes: `gpui-kit-0.7.0/release-notes.md:25-91`.

##### 1.1 Renamed methods and types (from the release notes and the diff)

| 0.6.6 | 0.7.0 | 0.7.0 place |
|---|---|---|
| `StrokeStyle` (enum) | `Curve` | G7/base/src/plot/mod.rs (enum near line 181) |
| `Line/Area::stroke_style(..)` | `curve(..)` | G7/base/src/plot/shape/line.rs:85-87, shape/area.rs:82-84 |
| `Line/RadialLine::dot_fill_color(Hsla)` | `dot_fill(impl Into<Background>)` | shape/line.rs:103-105, shape/radial_line.rs:136-138 |
| `dot_stroke_color` | `dot_stroke` | shape/line.rs:109-112 |
| `PlotHover::focus`, `Tooltip::focus` | `progress` | base/src/plot/hover.rs:64-71; component/src/plot/tooltip.rs:343-352 |
| `Scale::least_index` | `nearest_index` | base/src/plot/scale.rs:30-33 |
| `AXIS_GAP` const | `axis_gutter(font_size)` | base/src/plot/axis.rs:18-28 |

- Old names stay as `#[deprecated]` aliases for `dot_fill_color`, `dot_stroke_color`, `focus`, `AXIS_GAP` (release-notes.md:82-84). `StrokeStyle` and `stroke_style` are removed (release-notes.md:82).
- `least_index_with_domain` is removed (base/src/plot/scale.rs; the old one was G6/component/src/plot/scale/linear.rs:67-84).
- `Arc::paint`, `paint_cached`, `contains` lose their radius arguments (base/src/plot/shape/arc.rs:90-102, 200-251).
- Scale constructors take two-element arrays and iterators: `ScaleLinear::new(values, [h, 0.])` (scale/linear.rs:15-30).
  - A range with more than two stops (old test `test_scale_linear_multiple_range`) can no longer be expressed. [obs]
- Value bound is now the public `PlotValue` (f32, f64, Decimal) instead of the hidden `Sealed`. `f32` is new (scale.rs:11-24).

**Effect on reactive-tui's spec (CHT-020 and CHT-029, docs/spec/charts.md:103, 125):**

- None of the method names those two requirements list is renamed. The lists use `x y band value stroke fill natural linear step_after dot tick_margin alignment label grid open high low close` and, for radial and Sankey, the names in line 103. All still exist in 0.7.0 (checked with `grep -o "pub fn"` on each chart file). [obs]
- The renames hit only the plot-primitive builders (`Line`, `Area`, `RadialLine`). Search run: `grep -rn "stroke_style\|dot_fill_color\|dot_stroke_color\|StrokeStyle" RT/docs/spec RT/src/widgets/display RT/manual RT/examples/widget_catalog`. One hit only: a doc comment in R/plot/curve.rs:2 that still says "the reference's `StrokeStyle`". [obs]
- reactive-tui already uses the 0.7.0 name: `ChartsBuilder::curve` and type `Curve` (RT/src/widgets/display/charts.rs:240-244; R/plot/curve.rs:6-16). [obs]
- The spec header still says "modeled on gpui-kit 0.6.6" (docs/spec/charts.md:5).

##### 1.2 New chart options in 0.7.0 (methods that did not exist in 0.6.6)

Found by diffing `pub fn` names per chart file.

| Chart | New methods |
|---|---|
| all seven | `interactive(bool)` |
| line, area | `y_axis`, `y_axis_label_placement`, `y_tick_count` (default 5), `y_tick_format`, `y_domain`, `y_padding` (default 10 px top, 0 bottom), `x_tick_count`, `point_count`, `grid_columns`, `grid_dashed`, `reference_line`, `tooltip_title`, `tooltip_value`, `tooltip_value_color`, `tooltip_content` |
| bar | `band_count`, `band_tick_count`, `grid_dashed`, `label_color`, `max_band_width` (30 px), `min_length`, `padding_inner` (0.4), `padding_outer` (0.2), `value_axis_label_placement`, `value_tick_format`, four `tooltip_*` |
| candlestick | `max_band_width`, four `tooltip_*` |
| pie | `tooltip_name`, `tooltip_value(d, value, share)` |
| radar | four `tooltip_*` |
| sankey | `tooltip_name`, `tooltip_value` |

Citations: line G7/component/src/chart/line_chart.rs:99-153, 208-317; area area_chart.rs:99-125 (same set); bar bar_chart.rs:126-200, 303-457; candlestick candlestick_chart.rs:97-146, 185-188; pie pie_chart.rs:110-113, 218-243; radar radar_chart.rs:140-213; sankey sankey_chart.rs:183-196, 279-301. Shared machinery: G7/component/src/chart/mod.rs:215-305 (`TooltipContent`) and 311-339 (`PointAxes` defaults).

##### 1.3 Behavior, default and appearance changes

1. **Tooltips and hover are on by default.** 0.6.6: `id: Option<ElementId>` defaulted to `None`, so no hitbox and no tooltip (G6 line_chart.rs:229-241). 0.7.0: `id` defaults to the construction site via `caller_id()` and `interactive` is `true` (G7 chart/mod.rs:42-51; line_chart.rs:73-74, 91-102, 461-463). [obs]
2. **Hover motion moved out of the charts into a theme-driven layer.**
   - `PlotMotion` (pointer spring, enter and exit transitions) lives in base (G7/base/src/plot/mod.rs:32-88). Base defaults are all `Duration::ZERO`, so with base alone the hover snaps.
   - The component theme projects the "fast" motion tier onto it: pointer spring epsilon 0.1, enter and exit `Transition` with `easing_enter` and `easing_exit` (G7/component/src/theme/mod.rs:90-107). [obs]
   - Hover memory (last datum, cursor, fade) moved from `component/src/plot/tooltip.rs` to `G7/base/src/plot/hover.rs:115-193`. Pointer spring: hover.rs:104-109. `PlotHover::glide`: hover.rs:86-101.
   - `Tooltip` now glides the crosshair and dots itself, default `glide = true` (component/src/plot/tooltip.rs:322-335, 484-512), grows the halo with progress (513-516) and fades the whole overlay with `opacity(progress)` (564-588).
   - Result: line, area, candlestick and radar no longer keep their own hover struct. Bar (`BarHover`, bar_chart.rs:28-35, 1063-1077), pie (`PieHover`, pie_chart.rs:32-39, 438-463) and sankey (sankey_chart.rs:63-70, 727) still do. Pie uses the `spring_control` token (pie_chart.rs:444). [obs]
3. **Grid color is a new theme token `chart_grid`** (G7/component/src/theme/theme_color.rs:146-147). Line and area grid (chart/mod.rs:405-428), bar (bar_chart.rs:857) and radar (radar_chart.rs:485) switched from `border` to `chart_grid`. Candlestick was not switched and still uses `border` (candlestick_chart.rs:304; old G6 candlestick_chart.rs:234). [obs]
4. **Line and area gained a value axis** (off by default), tick count 5, evenly spaced in pixels from baseline to top, both ends included (chart/mod.rs:396-403, 405-428, 455-496; line_chart.rs:233-270). The grid still draws 4 horizontal lines by default: the ticks 0..4 of 5, baseline excluded. Same line positions as the old hard-coded `i/4` (G6 line_chart.rs:205). [obs]
5. **Bar value axis:**
   - `value_tick_count` default went from 4 to 5 and now counts ticks, not steps (G6 bar_chart.rs:92, 247-248, 557 against G7 bar_chart.rs:104, 344-347, 852-853). The tick count on screen is the same by default (5). [obs]
   - The value-label gutter is measured from the label text, with a 32 px minimum (G7 chart/mod.rs:155-168; bar_chart.rs:612-623, 741-743). It was a fixed 32 px (G6 bar_chart.rs:23-29, 285). Horizontal bars still use 32 px (bar_chart.rs:617-619).
   - `AxisLabelPlacement::Inside` puts labels over the plot edge (base/src/plot/axis.rs:39-46; bar_chart.rs:905-922; chart/mod.rs:474-489).
   - Room above the tallest vertical bar is now one text line when the chart has value labels, else 10 px (bar_chart.rs:510-516). Old: fixed 10 px (G6 bar_chart.rs:550-556). [obs]
6. **Band width cap moved.** `ScaleBand::band_width` no longer clamps to 30 px. The charts pass `max_band_width` explicitly, default 30 px (base/src/plot/scale/band.rs:51-63; chart/mod.rs:149-151; bar_chart.rs:437-444, 472-476; candlestick_chart.rs:185-188, 241; release-notes.md:64-66). Same look by default, now configurable. [obs]
7. **Point charts place points by index, not by looking up the x value** (`tick_at`, base scale/point.rs:47-64; line_chart.rs:423-429). The 0.6.6 lookup returned the first match for a repeated x label. The comment at line_chart.rs:423-424 says the old path was O(n^2). [obs; the duplicate-x effect is my reading of the `tick_at` doc, base scale/point.rs:47-52]
8. **Fixed slots for growing data:** `point_count` and `band_count` lay the axis out for N slots and leave the tail empty (chart/mod.rs:56-72; line_chart.rs:220-231; bar_chart.rs:369-378; base scale/band.rs:76-83).
9. **Pinned y domain** clips the line to the plot (`pinned_plot_mask`, chart/mod.rs:498-508; line_chart.rs:438-442). New.
10. **Candlestick wick** is painted as a 1 px quad instead of a stroked path (candlestick_chart.rs:352-358; old G6 candlestick_chart.rs:282-289). The candle highlight band glides through `Tooltip` (candlestick_chart.rs:427-437).
11. **Pie tooltip lost its title.** 0.6.6 titled it with `label(d)`; 0.7.0 shows one row with name, value and share, and says why in a comment (pie_chart.rs:483-497; old diff line 458). The row is still "value (share%)" (pie_chart.rs:495). [obs]
12. **Sankey:** ribbon paths are cached per link by index (sankey_chart.rs:585-616). `tooltip_value` overrides `value_label` for the tooltip only (757-759).
13. **Plot layer internals:**
    - `PlotAxis` records labels and places them at paint time, so builder order no longer matters (base axis.rs:83-90, 181-208, 230-243).
    - Axis `stroke`, `Grid` `stroke` and dot fill take `Background` (gradient capable) (axis.rs:97, grid.rs:8; line.rs:20).
    - `AxisText`, `TooltipState`, `ArcData` and others are `#[non_exhaustive]` (release-notes.md:76-78).
    - `PlotAxis::y_axis` doc now says "Default is false" (base axis.rs:155-160). The old doc said "Default is true" (G6 axis.rs:125-128) while the code defaulted to false; a doc fix only. [obs]
14. **Radar/line dot** fill is `Background` and `dot_stroke` falls back to the solid part of `dot_fill` (shape/line.rs:131-135, radial_line.rs:164-168).

Not changed (checked): pie default outer radius 0.4 x height (pie_chart.rs:246-252); radar default label gap 10, 4 rings, fill opacity 0.3 (radar_chart.rs:29-33, 516-520); area default fill `chart_2` at 0.4 (area_chart.rs:432-433; old G6 area_chart.rs:217); sankey constants (sankey_chart.rs:27-39); band padding 0.4 and 0.2 (bar_chart.rs:112-113). No animation on data change or on first draw anywhere in `G7/component/src/chart` (search: `grep -rn -i "animat" chart/*.rs` found none apart from the words "center" and "enter"). [obs]

##### 1.4 Statements in docs/widget-study.md "### chart, plot" that are no longer true for 0.7.0

The section is at RT/docs/widget-study.md:331-395. Its `G/` refs are 0.6.6 paths.

| Study line | Statement | Status at 0.7.0 |
|---|---|---|
| 332 | "B/ has no chart code. It supplies only the `Spring` ... (G/chart/mod.rs:20, 39-41)" | False. `pointer_spring` left chart/mod.rs. Base now holds the plot layer, `PlotMotion`, hover tracking and `PlotElement` (base/src/plot/*.rs; hover.rs:107-109). Base still has no chart types. |
| 334 | "The hover memory ... lives in keyed element state (G/plot/tooltip.rs:324-401)" | Moved to G7/base/src/plot/hover.rs:115-193. |
| 334 | "Tooltips stay off until `id()` is set (G/chart/line_chart.rs:78-85)" | False. On by default via `caller_id()`; `interactive(false)` turns them off (line_chart.rs:73-74, 91-102). |
| 334 | `Plot` trait at G/plot/mod.rs:25-111 | The trait is now G7/base/src/plot/mod.rs:90-171. The method set is unchanged (`prepaint`, `paint`, `id`, `tooltip_state`, `hover`, `tooltip`; `prepaint` also existed in 0.6.6 at plot/mod.rs:37). `IntoPlot` now generates `type Element = PlotElement<Self>` (release-notes.md:79-80). |
| 340 | "Only ours: Axis title, min, max, custom labels and tick count" | Partly false. The reference now has pinned min/max (`y_domain`), tick count (`y_tick_count`, `value_tick_count`) and tick text (`y_tick_format`). Axis title and custom label lists are still ours only. |
| 341 | "Only ours: Value-axis tick labels on every cartesian chart" | False. Line, area and bar have them, off by default (`y_axis`, `value_axis`). Candlestick has no value axis (search: no `y_axis` in candlestick_chart.rs). |
| 353 | "`value_axis` switch and `value_tick_count` on bars (G/chart/bar_chart.rs:229-250)" | Still exists. Default 4 changed to 5 and it now counts ticks (bar_chart.rs:104, 344-347). Line refs are now bar_chart.rs:324-347. |
| 360 | Hover emphasis line refs (bar 31-32, 628-644; pie 25-29, 412-437) | Behavior unchanged (bar `HOVER_DIM` 0.45; pie lift 6 px and dim 0.35). New refs: bar_chart.rs:25-26, 948-964, 1063-1077; pie_chart.rs:26-30, 276-282, 300-305, 438-463. |
| 362-364 | "gpui-kit keeps a stroke and a fill for each area series (G/chart/area_chart.rs:225-239)" | Still true. Now area_chart.rs:431-455. |
| 364 | "gpui-kit dots off by default (G/chart/line_chart.rs:66)" | Still true. Now line_chart.rs:64. |
| 369-371 | Builder method lists for `LineChart`, `BarChart`, `PieChart` | Incomplete. See 1.2. Line now has 16 more public methods. |
| 381 | "Each chart also keeps a sprung pointer position ... (G/chart/line_chart.rs:23-30 ...)" | False for line, area, candlestick and radar. `Tooltip` glides for them (tooltip.rs:484-512). Bar, pie, sankey keep their own. |
| 381 | "focus value from 0 to 1 ... (G/plot/tooltip.rs:264-332)" | Renamed `progress`, moved to base hover.rs:45-102 (`focus` is a deprecated alias, hover.rs:68-71). |
| 389 | "a11y search of G/chart and G/plot found none" | Still true. Re-ran `grep -rn -i "accessib\|a11y\|aria\|announce\|on_key\|key_down\|KeyBinding\|actions!\|role"` on `component/src/chart`, `component/src/plot`, `base/src/plot`: no matches. |
| 392 | "pie tooltip shows the slice's share (G/chart/pie_chart.rs:447-463)" | Still true, now pie_chart.rs:473-497. The tooltip no longer has a title. |
| 394 | "public `Plot` trait ... hook for custom charts" | Still true; the trait moved to gpui-base (see 334). |
| 396 (sankey cycle) | "gpui-kit paints nothing (G/chart/sankey_chart.rs:378, 484-501, 506-509)" | Still true; now `topology(..).ok()?` at sankey_chart.rs:420-422 and `prepaint` early return at 520-522. |

Study statements that hold as written: area fill replaces stroke color in ours; typed line builder cannot turn dots off (both re-checked in Part 4); scatter places points by index.

---

#### Part 2. Chart by chart: gpui-kit 0.7.0 against reactive-tui today

Terminal widths are 240 to 512 columns. Where it matters I say what the code does at that width. Size classes (R/plot/layout.rs:95-105): Mini under 40 columns or 8 rows; Large needs width >= 200 and height >= 40; else Medium. A 300 x 30 chart is Medium. [obs]

##### 2.0 Cross-cutting defaults

| Topic | gpui-kit 0.7.0 | reactive-tui |
|---|---|---|
| Plot margin | Top 10 px above highest value (`y_padding`, chart/mod.rs:334); bottom axis gutter 18 px (`AXIS_GAP`, mod.rs:147); left gutter measured, min 32 px when value axis on | No top margin: the value scale ends on the top dot of the plot (cartesian.rs:197-210). Left gutter = widest tick label + 1, capped at width/3 (cartesian.rs:133-142). Bottom gutter = 1 row, +1 with an x title (152-162). |
| Tick count | 5 (`y_tick_count`, `value_tick_count`), evenly spaced, both ends included (mod.rs:396-403; bar_chart.rs:1257-1262) | `ChartAxis::tick_count` default 5 (charts.rs:430). `tick_count()` only lowers it to the cell count (canvas.rs:611-613). Ticks land on round values inside the domain (tick.rs:18-95), so the count is "about" and the top of the plot may have no label. Same 5 at 12 rows or 100 rows. [obs] |
| X label spacing | `tick_margin` (every n-th, default 1) or `x_tick_count` (n evenly) (mod.rs:186-211). No text-width check. | `tick_margin` (charts.rs:264-268, default 0 = auto). Auto = `label_skip` on measured widths with a 1-cell gap (tick.rs:156-175; cartesian.rs:288-304, 350-367), plus a second guard in `Axis::draw` (axis.rs:161-163). |
| Axis and grid style | Axis line `border`; labels `muted_foreground`; grid `chart_grid`, dashed 4 px on 2 px off (mod.rs:405-428). `grid_dashed(false)` for solid. y axis line off, x axis line on. | Axis line and labels drawn with `None` color, i.e. no theme token passed (cartesian.rs:322-325, 383-386; axis.rs:114-182). Grid glyph `·` with `None` color (cartesian.rs:311, 319, 373, 380). Grid only at Large (layout.rs:117-120; cartesian.rs:254). Left axis `│` and `└`, bottom `─` (axis.rs:117-123, 141-146). No `chart-grid` preset var (src/theme/presets.rs:24-31 defines chart-1..5, bullish, bearish only). |
| Legend | None in any chart | `ChartLegend` default visible, position Right (charts.rs:447-455); shown at Medium and Large (layout.rs:113-115); Right/Left take up to width/2 (canvas.rs:523-544). Typed builders have no legend method (typed.rs:66-109); only Sankey turns it off (typed.rs:1029). |
| Color choice | Single series: `chart_2` (line 422, area 433, bar 929, pie 267-272). Multi-series area: `strokes[i]` else `chart_2` for every series. Radar and sankey: palette chart_1..5 (radar_chart.rs:326-331; sankey_chart.rs:566-580). | Palette `chart-1..chart-5` by series index (charts.rs:828; canvas.rs:226-250). Pie slices cycle the palette with a wrap fix (pie.rs:53-77). Colors resolve through `Theme::resolve_color` (canvas.rs:220-222). Series 0 gets chart-1, not chart-2. |
| Empty data | Not a message. Line, bar, candlestick still paint axes and grid with empty scales (line_chart.rs:374-459). Pie paints nothing. Radar returns (radar_chart.rs:428, 473). Sankey returns (sankey_chart.rs:520). Search `"No data"`: none in `chart/`. | Text "No data to display" (canvas.rs:419-422; also pie.rs:158-161, radar.rs:33-36, sankey.rs:152-155). NaN or infinite value: an error text, no shapes (canvas.rs:273-281; CHT-026). |
| One point | `ScalePoint` puts it mid-range (base scale/point.rs:47-64). Line draws no stroke for one point; dot only if `dot()`. | `ScalePoint::map` puts it at the centre (scale.rs:240-247). Dot marker is on by default (charts.rs:836; cartesian.rs:679-683). |
| Negative values | Line/area: y domain includes zero (`point_value_scale`, mod.rs:118-139). **Area fills to the plot bottom, not to zero** (`.y0(height)`, area_chart.rs:450). Bars grow from zero; band labels flip to the empty side of each bar (bar_chart.rs:801-826, 1249-1251). Pie: negatives clamped to 0 for the share only (pie_chart.rs:476). | Domain includes zero (`domain_including_zero`, scale.rs:26-48; CHT-011). Area fills to the zero baseline or the stacked base (cartesian.rs:602-609). Bars grow from zero, stacked positives and negatives separately (406-450). No zero line and no flipped labels (see P3.6). Pie with any negative value: whole chart is an error text (canvas.rs:282-284). Radar clamps values to 0..max (radar.rs:120). |
| More points than pixels | No decimation anywhere (`grep -rn "decimat\|downsample\|lttb"` on chart, plot, base/plot: none). | Line/area/scatter: min/max decimation to at most 2 per plot column (cartesian.rs:593-597; decimate.rs:17-60), markers dropped when decimated (681). Bars and candles: not decimated; each bar keeps at least one cell, so more bars than cells overlap (cartesian.rs:419-433, 533-539). [obs] |

##### 2.1 Line

**Options only in the reference (0.7.0):** `y_domain` (clipped), `y_padding`, `point_count`, `reference_line`, `grid_columns`, `grid_dashed`, `x_tick_count`, `y_tick_format`, `y_axis_label_placement` Inside, `interactive`, tooltip title/value/color/content hooks, `name` per chart (line_chart.rs:99-317).
**Options only in reactive-tui:** several series per chart (`y()` adds a series, typed.rs:186-194), per-series `LineStyle` Solid/Dashed/Dotted/None (charts.rs:589-600; cartesian.rs:658-678), axis `title`, `custom_labels`, `min`/`max` (charts.rs:403-420), value labels, legend, size classes, stacking (areas only), decimation, ASCII mode.

| Default | gpui-kit 0.7.0 | reactive-tui |
|---|---|---|
| Curve | Natural (`Curve::default`, line_chart.rs:63) | `Curve::Natural` (charts.rs:835). Ours is Catmull-Rom (plot/curve.rs:1-5, 34-45), not d3 natural. |
| Stroke width | 2 px (line_chart.rs:432) | One dot wide polyline in the mask (cartesian.rs:661) |
| Dots | Off. On: 8 px, filled with stroke (line_chart.rs:64, 434-436) | **On** (charts.rs:836). `Marker::Disc` `●` (mask.rs:46-51, 879-880); only when nothing was decimated (cartesian.rs:681). |
| Fill | none | none |
| Grid | On, dashed, 4 lines (baseline excluded) | On only at Large |
| Legend | none | Right |

**Motion.** Reference: crosshair and dot glide between points on the pointer spring, fade in and out on the fast tier (tooltip.rs:322-335, 484-516, 564-588; theme/mod.rs:90-107). No enter or data-change animation. Ours: no hover motion at all. The overlay is patched into a copy of the grid each frame (live.rs:257-268, 855-918). Reveal and data-change transitions exist and are linear in time (motion.rs:115-118, 196-210); reveal is off by default (`animated: false`, charts.rs:829) while the 200 ms data transition is on (840; motion.rs:165-233). [obs]

**Tooltip and hover.**
- Reference: title = x value, one row per series with swatch, name and value, box hugs the cursor with an 8 px gap, flips to the centre side at the half-way point of the plot, min width 150 px (line_chart.rs:508-531; tooltip.rs:585-605). A vertical dashed crosshair confined to the plot (`cross_line`, line_chart.rs:512-515; tooltip.rs:124-129, 159-165) and a dot with a 20 px halo on the point (line_chart.rs:516-522; tooltip.rs:241-271). Hover is only inside the plot, not over the axis labels or the value gutter (line_chart.rs:474-480).
- Ours: no title, one row per visible series `name  label: value; key=value` (live.rs:685-700; canvas.rs:593-608). Box is a bordered box at the anchor cell, right of it or flipped left or above (tooltip.rs:112-140; live.rs:783-799), at most 8 rows then "+N more, sum X" (tooltip.rs:11-12, 54-91). Crosshair is a `│` written to empty cells only, no color (live.rs:800-803, 879-899). No marker on the hovered point. Keyboard selection and aria-live text exist (live.rs:380-454, 318-324); the reference has neither.

**Edge cases.** Empty, one point, negative, many points: see 2.0. Ours also rejects a NaN with an error (canvas.rs:273-281).

##### 2.2 Area

**Only in the reference:** stroke and fill are separate per series (`strokes`, `fills: Vec<Background>`, area_chart.rs:35-38, 434-455); curve per series (`curves`, 36, 441-444); all the line-only options in 2.1.
**Only in reactive-tui:** stacked series (charts.rs:233-238; cartesian.rs:569-581, 602-609), `FillStyle` None/Solid/Gradient/Pattern (charts.rs:602-613; cartesian.rs:610-657, 705-762).

| Default | gpui-kit 0.7.0 | reactive-tui |
|---|---|---|
| Fill | `chart_2` at 0.4 opacity, per-series `Background` (area_chart.rs:432-440) | `FillStyle::Solid` (charts.rs:566): same color as the stroke, no opacity: `_ => tint` (cartesian.rs:749); stroke uses the same `tint` (598, 658-662) |
| Gradient | User supplies a `Background`; the story fades to the baseline ("Gradient fills fade to the baseline", G7/story/src/stories/chart_story/chart_story.rs:1259) | `FillStyle::Gradient` scales r,g,b by 0.45 at the plot top up to 1.0 at the baseline (cartesian.rs:742-748). No blend toward the background. |
| Baseline | Plot bottom (`.y0(height)`, area_chart.rs:450) | Zero line or stack base (cartesian.rs:602-609, 612-625) |
| Dots | none | Same `props.dots` marker as line: **on by default for area**, typed `AreaChartBuilder` has no way to turn off (see D2) |

Motion, tooltip, hover: as line. Ours draws one crosshair column and one row per series; the reference draws one dot per series (area_chart.rs:509-520, 546-552).

##### 2.3 Bar

**Only in the reference:** `fill(closure)` and `fill_gradient` per datum (bar_chart.rs:213-288); `corner_radii` (412-419); `padding_inner`, `padding_outer`, `max_band_width` (30 px), `min_length`, `band_count`, `band_tick_count`, `label_color`, `value_axis_label_placement`, `value_tick_format`, `grid_dashed`; hover dim of the other bars.
**Only in reactive-tui:** grouped and stacked multi-series bars (cartesian.rs:406-450), `BarGrowth` incl. per-orientation label side (charts.rs:362-381), value labels by size class, decimation none.

| Default | gpui-kit 0.7.0 | reactive-tui |
|---|---|---|
| Band padding | inner 0.4, outer 0.2 (bar_chart.rs:112-113) | inner 0.4 (0.3 when several grouped series), outer 0.2 (0 at Mini) (cartesian.rs:212-218). Grouped lanes use inner 0.15 (424). |
| Max bar width | 30 px (bar_chart.rs:114; chart/mod.rs:151) | **None.** `ScaleBand` has no cap (R/plot/scale.rs:107-183; search `max_band\|bar_width` over R found nothing). Each bar is `bandwidth` wide, snapped to whole cells (cartesian.rs:427-432). At 400 columns and 4 categories a bar is about 0.6 x 100 = 60 columns. [obs, arithmetic from cartesian.rs:212-218] |
| Corner radius | 0 (bar_chart.rs:111) | n/a on cells |
| Fill | `chart_2` solid (929) | palette by series (canvas.rs:226-234) |
| Labels | Band labels at the zero line, on the side each bar leaves empty (801-826). Value labels at the bar tip, `foreground` unless `label_color` (932-934, 1006-1009). | Band labels at the plot bottom (cartesian.rs:339-349). Value labels only at Large or with `.label()`/`value_labels(true)` (391-393; typed.rs:502-507), drawn in the bar color (492-505). |
| Tick count / gutter | 5, measured gutter | 5, measured gutter |
| Grid | On, dashed, 4 lines | Large only |

**Tooltip and hover.** Reference: a translucent band the width of the bar (`foreground` at 0.08) glides between bars, other bars fade by up to 45% (bar_chart.rs:25-26, 948-964, 1092-1114; tooltip.rs:124-129). Ours: a 1-column `│` (vertical bars) or `─` (horizontal) line in empty cells only (live.rs:800-805, 895-903); nothing dims. Selection resolves by nearest band centre (live.rs:669-681).

**Small values.** Reference: `min_length` stub option; default 0 (bar_chart.rs:446-457). Ours: a bar shorter than 1/16 of a cell after snapping to eighths is skipped (`(from - to).abs() < 1e-9` after `snap`, cartesian.rs:455-459). Value 0 draws nothing. [obs]

##### 2.4 Candlestick

**Only in the reference:** `body_width_ratio` (0.8, candlestick_chart.rs:66, 120-123, 348), `max_band_width` (30 px), tooltip hooks, glide band.
**Only in reactive-tui:** per-point color override via `DataPoint::color` (cartesian.rs:523-532); value axis with ticks and labels (the reference candlestick has no value axis; y range is `[height, 10.]`, candlestick_chart.rs:283).

| Default | gpui-kit 0.7.0 | reactive-tui |
|---|---|---|
| Padding | inner 0.4, outer 0.2 (candlestick_chart.rs:242-243) | same via the shared band (cartesian.rs:212-218) |
| Body width | `body_width_ratio` 0.8 of the band (348) | The whole band lane, snapped to cells, at least 1 cell (533-539) |
| Wick | 1 px quad (352-358) | 1 dot wide rect (542-549) |
| Colors | `chart_bullish`, `chart_bearish` (225-229); doji (`close == open`) is bearish (343) | `chart-bullish`/`chart-bearish` (cartesian.rs:14-15, 514-532); `is_bullish` = close > open (charts.rs:398-400), doji is bearish |
| Doji body | zero height | forced to 0.5 dot tall (552-555) |
| Grid | 4 lines, `border` color (303-306) | Large only |

Tooltip: reference shows Open/High/Low/Close rows in the candle color (435-455). Ours shows `label: O .. H .. L .. C ..` in one row (canvas.rs:595-599).

##### 2.5 Pie and donut

**Only in the reference:** `inner_radius_fn`, `outer_radius_fn` per slice (pie_chart.rs:127-168), `label_color`, `label_line_color` (200-209), `tooltip_name`, `tooltip_value(d, value, share)` (225-241), hover lift 6 px and dim 0.35 (26-30, 276-282, 300-305).
**Only in reactive-tui:** Donut as a type (inner default 0.5, charts.rs:617-629; pie.rs:196-205), radii as fractions (`RadialOptions`, charts.rs:644-666), legend that keeps unlabeled slices (pie.rs:100-133), label placement without overlap (pie.rs:288-393), keyboard stepping through slices (live.rs:400-431).

| Default | gpui-kit 0.7.0 | reactive-tui |
|---|---|---|
| Outer radius | 0.4 x height (pie_chart.rs:246-252) | 1.0 of the largest circle that fits (charts.rs:672; pie.rs:190-192): the circle spans the full height of the area. |
| Inner radius | 0 (73) | 0 for pie, 0.5 for donut (pie.rs:196-205) |
| Pad angle | 0 (75) | 0 (charts.rs:673) |
| Label gap | 15 px (24, 81) | 2 columns (charts.rs:674) |
| Label | Leader line + text, `foreground`; line `border` (329-330); labels spread with `spread_labels` (371-376) | Text in the slice color, leader in default color; medium 16 columns max, large full text plus raw value (pie.rs:21-22, 164-175, 385-390) |
| Color | All slices `chart_2` unless `color(f)` (267-272) | Palette cycle (pie.rs:53-77) |
| Legend | none | side, per slice |

**Tooltip and hover.** Reference: one row, slice color swatch, name, `value (share%)` (pie_chart.rs:483-497); hovered slice lifts, others fade; tooltip follows the cursor. Ours: one row in the slice color with `label: value` (live.rs:759-776); a `·` ray along the slice's middle angle (live.rs:569-597, 906-908); nothing lifts or dims. No share percent anywhere (search `share\|percent` in live.rs and canvas.rs: none).

**Edge cases.** Negative slice: error text (canvas.rs:282-284). All zero: "No data to display" (pie.rs:158-161). Slices under 0.5 degree: reference skips their labels (pie_chart.rs:338-341); ours labels only slices whose fill set a sample (pie.rs:236-256).

##### 2.6 Radar

**Only in the reference:** `RadarLabel::Element` (custom label elements, radar_chart.rs:35-68), tooltip hooks.
**Only in reactive-tui:** per-series fill token incl. `"none"` (charts.rs:662-665; radar.rs:124-128), grid at Large only.

| Default | gpui-kit 0.7.0 | reactive-tui |
|---|---|---|
| Rings | 4, always drawn when `grid` (radar_chart.rs:33, 484-496) | 4 (charts.rs:675), drawn only at Large (radar.rs:99-107) |
| Radius | 0.4 x height (341-347) | Largest circle minus label margins, minus one dot (radar.rs:52-68) |
| Fill | series stroke at 0.3 (516-520) | Series color, no opacity, per-series token (radar.rs:124-128) |
| Stroke | 2 px (531) | one dot (radar.rs:138) |
| Dots | off (127); 8 px when on | typed builder off (typed.rs:1057); `ChartsBuilder::radar()` **on** (charts.rs:836; radar.rs:141) |
| Label | `muted_foreground`, gap 10 px, left/centre/right by angle (538-570) | Default color, gap fixed at one dot past the ring (radar.rs:154-171); medium cap 12 columns (19-20) |
| Scale | zero to max data or `max_value` (377-389) | same (radar.rs:88-95), values clamped to 0..max |

**Tooltip.** Reference: title = category label, one row per series, one dot per series at the spoke vertex (radar_chart.rs:611-660). Ours: a row per series with the category label inside the value text; a ray along the spoke (live.rs:569-597).

##### 2.7 Sankey

**Only in the reference:** `node_corner_radius`, `font_size`, `color` (per-line label styling), `tooltip_name`, `tooltip_value` (sankey_chart.rs:279-301). **Only in reactive-tui:** error texts for cycle or missing node (canvas.rs:334-346), keyboard node selection (live.rs:400-431).

| Default | gpui-kit 0.7.0 | reactive-tui |
|---|---|---|
| Node width | 10 px (27) | 2 columns (charts.rs:747) |
| Node padding | 16 px (28) | 1 row (748) |
| Link opacity | 0.3 (29) | 0.3 (749), blended toward the theme background, not alpha (sankey.rs:117-127) |
| Min link width | 1 px (30) | 0.25 row (750) |
| Label gap | 6 px (31) | 1 column (751) |
| Iterations | 6 (157) | 6 (745) |
| Colors | palette chart_1..5 by node (566-580) | palette by node (sankey.rs:38-45) |
| Hover dim | other links x (1 - 0.7 x progress) (37-39, 619-624) | other links x (1 - 0.7) (sankey.rs:18-20, 248-252); selection is drawn into the shapes by the worker (live.rs:205-209) |
| Labels | truncated with an ellipsis, first/last beside, middle centred above (645-705) | same rule (sankey.rs:157-208, 327-374); large adds throughput as raw `f64::to_string()` (74, 102) |

Empty or no links: reference returns (520-522); ours prints "No data to display" (sankey.rs:152-155).

---

#### Part 3. Polish candidates for reactive-tui

Size: small under 200 lines, medium, large. "Contract" means an Agreed requirement would have to change.
Left out on purpose because a cell grid cannot show them: bar corner radii (bar_chart.rs:412-419), pie lift of 6 px (pie_chart.rs:27), a 20 px hover halo (chart/mod.rs:56-58), sub-cell spring glide of the crosshair (theme/mod.rs:90-107).

**P3.1 Cap bar and candle width.** [small to medium]
- Reference: `max_band_width` 30 px default (G7 chart/mod.rs:151; bar_chart.rs:437-444, 472-476; candlestick_chart.rs:185-188, 241; base scale/band.rs:51-63).
- Ours: no cap. Bar lane comes from `band.band(i)` and is rounded to whole cells (cartesian.rs:419-433); candles the same (533-539). `ScaleBand` in R/plot/scale.rs:107-183 has no such field.
- Observed: at 240 to 512 columns, few bars become very wide. [obs for the arithmetic; how it looks is [assume]]
- Cell version: a cap in columns, centred in its band (the reference keeps the tick where it was, scale/band.rs:176-182 test). Also `body_width_ratio` 0.8 for candles (candlestick_chart.rs:66, 348).

**P3.2 Hover emphasis: dim the other bars, slices, candles, radar series.** [medium]
- Reference: bar dim 0.45 x distance to the hovered band (bar_chart.rs:948-964); pie dim 0.35 (pie_chart.rs:276-282).
- Ours: only sankey passes the selection to the worker (live.rs:205-209, 223-232). Other charts get a crosshair or ray patched on a copy of the finished grid (live.rs:855-918). No dimming, and no re-raster on selection.
- Cell version: either pass `selected` to the worker for every type, as sankey does, and multiply shape colors, or blend cell colors in `overlay_grid`. The former re-rasters per hover move; the latter needs per-cell ownership (the mask already tracks `owner` keys, cartesian.rs:461-463).

**P3.3 Highlight band for the hovered bar or candle.** [small to medium]
- Reference: a translucent band as wide as the bar, over the whole plot height (`CrossLine::band`, bar_chart.rs:1092-1114; tooltip.rs:124-129).
- Ours: a `│` in empty cells at the bar centre (live.rs:800-805, 895-899). Above a bar tip that is a thin line; over the bar body it draws nothing because the cell is not free (881-889).
- Cell version: set a background color on every cell in the lane (`CellGrid::set_with_background` exists, canvas.rs:580-583).

**P3.4 Hover marker on the hovered point.** [small]
- Reference: `Dot` on each series at the hovered x, with halo (line_chart.rs:508-522; area_chart.rs:546-552).
- Ours: no marker (live.rs:895-908). The anchor cell of every point is known (`picture.anchors`, cartesian.rs:690).
- Cell version: write `●` or `◉` at the anchor cell for line, area, scatter in the series color.

**P3.5 Tooltip title and formatting.** [small]
- Reference: title = x value or band value, then swatch, name, value rows (chart/mod.rs:279-330; line_chart.rs:524-531). Overrides: `tooltip_title`, `tooltip_value`, `tooltip_value_color`.
- Ours: `Tooltip { title: None, .. }` (live.rs:699, 749, 767); the label repeats inside each row (canvas.rs:593-608); numbers print with `{}` on `f64` (canvas.rs:600), so `0.1 + 0.2` prints `0.30000000000000004`. [obs]
- Cell version: fill `Tooltip::title` from the category label (the field exists, tooltip.rs:29); format values with `format_tick` (tick.rs:40-52). Props hold data, not closures (CHT-020), so a formatter option would be an enum or a precision, not a closure. [assume]

**P3.6 Zero line and label side for mixed-sign bars.** [small to medium]
- Reference: axis line at the zero pixel; each band label on the side its bar leaves empty (bar_chart.rs:771-777, 790-828, 1249-1251).
- Ours: bars grow from `value_scale.map(0)` (cartesian.rs:434-456) but the category axis and its labels sit at the plot bottom (339-349); no zero line at Medium (grid is Large only).
- Cell version: draw a `─` at the zero row when the domain crosses zero; move labels to the free side. Assumes a bar chart with negatives is a common case. [assume]

**P3.7 Colors for axis, grid and labels.** [small]
- Reference: axis `border`, labels `muted_foreground`, grid `chart_grid` (new token, theme_color.rs:146-147; chart/mod.rs:405-428, 455-496).
- Ours: axes, tick labels and grid pass color `None` (cartesian.rs:311, 322-325, 373, 383-386). Presets define no chart-grid (src/theme/presets.rs:24-31); `--color-border` exists (presets.rs:18, 58, 98, 138).
- Cell version: add `--color-chart-grid` to the presets and resolve it via `Theme::resolve_color`; use `border` for axis lines. [assume: default-color rendering is the same as data text color; I did not render]

**P3.8 Separate area fill and stroke, with blended fill.** [small to medium]
- Reference: `strokes[i]` and `fills[i]`, default fill `chart_2` at 0.4 (area_chart.rs:432-455).
- Ours: one `tint` per series for fill and stroke (cartesian.rs:598, 611-625, 658-662); typed builder overwrites the series color with the fill (typed.rs:344-353).
- Cell version: add a fill token per series; blend it toward the theme background at about 0.4 with the existing `blend` helper (sankey.rs:117-127); draw the stroke on top in full color. Fix the gradient to fade toward the baseline (see D7).

**P3.9 Reference lines.** [small]
- Reference: `reference_line(value)`, dashed, darker than the grid (line_chart.rs:300-308; chart/mod.rs:430-452).
- Ours: none.
- Cell version: extra rows in the under layer at `value_scale.map(v)`.

**P3.10 Headroom above the highest value and a round top tick.** [small]
- Reference: 10 px top padding by default (chart/mod.rs:334).
- Ours: the max value maps to the top dot (cartesian.rs:197-210); ticks only inside the domain (tick.rs:66-93), so the top row often has no label. [obs]
- Cell version: one row of headroom, or extend the domain to the next tick step. [assume: "looks better"; the reference itself does not round to a tick]

**P3.11 Tick count that follows the plot height.** [small]
- Reference: fixed 5 (mod.rs:329).
- Ours: `tick_count(axis, cells)` = min(axis.tick_count, cells) (canvas.rs:611-613); default 5 (charts.rs:430).
- Cell version: derive the default from rows, for example one tick per 4 to 6 rows. In the study's 240-512 wide, tall terminals, 5 ticks over 60 rows leaves about 12 rows between labels. [assume for the row count]

**P3.12 Grid and rings below the Large class.** [small; contract]
- Reference: grid on by default at every size (line_chart.rs:69; bar_chart.rs:108; radar_chart.rs:125).
- Ours: grid and radar rings only at Large (layout.rs:117-120; cartesian.rs:254; radar.rs:99). Large needs height >= 40, so a 300 x 30 chart has neither. [obs]
- Contract: CHT-024 (docs/spec/charts.md:145) says medium has "axes, ticks" and large has "grid". Changing it means editing an Agreed requirement.

**P3.13 Bar `min_length` stub.** [small]
- Reference: `min_length` (bar_chart.rs:446-457, 1140-1180).
- Ours: bars under 1/16 cell are skipped (cartesian.rs:455-459).
- Cell version: a minimum of one eighth block.

**P3.14 Fixed slots for growing data (`point_count`, `band_count`).** [medium]
- Reference: line_chart.rs:220-231; bar_chart.rs:369-378; chart/mod.rs:56-72.
- Ours: none; `count` is the longest series (cartesian.rs:87-91), so the axis rescales every time data grows.

**P3.15 Pie share percent, per-slice hooks, smaller default radius.** [small]
- Reference: `value (share%)` (pie_chart.rs:476-497); default outer 0.4 x height (246-252).
- Ours: `label: value` (live.rs:759-776); large label appends raw `data[i].value` (pie.rs:164-171); outer 1.0 (charts.rs:672).
- Cell version: add the share to the tooltip and the Large label. A smaller default radius is a taste call; the label placement already needs side room. [assume]

**P3.16 Tick number format for large values.** [small]
- Reference: `format_tick` prints whole numbers bare and the rest to one decimal (chart/mod.rs:176-183); `y_tick_format` hook (line_chart.rs:263-270).
- Ours: from 1e6 up it prints scientific notation (tick.rs:44-46). With 240 or more columns there is room for `2.5M` or `2500000`. [assume]
- Also `ChartAxis` has no format field (charts.rs:403-420); props must stay `PartialEq`.

**P3.17 Legend control in typed builders.** [small]
- Reference: no legend.
- Ours: legend on at Medium and Large, Right (canvas.rs:496-544); typed builders cannot hide or move it (typed.rs:66-109).

**P3.18 Ease-out on reveal and data transitions.** [small; not from the reference]
- Ours: linear in time (motion.rs:115-118, 196-210).
- The reference has no data animation, but uses theme easing for hover fades (theme/mod.rs:90-107). [assume: an easing curve reads better; not checked]

**P3.19 Bar gradients (per-row color ramp).** [medium; low priority]
- Reference: `fill_gradient` (bar_chart.rs:246-288, 983-997).
- Cells carry one color each, so a per-row ramp along a bar is possible, but the mask draws one tint per rectangle (cartesian.rs:461-463). I list it for completeness, not as a recommendation.

---

#### Part 4. Defects found by reading reactive-tui's chart code

##### D1 (still holds). Area fill replaces the stroke color
- `AreaChartBuilder::stroke` sets `SeriesSpec.color` (R/typed.rs:304-310). `fill` stores a token in `fills` (312-318). `build()` then writes that token into `series.color` (344-353), so the fill wins whatever the call order.
- The canvas draws fill and stroke from one `tint = point_color(props, s, 0)` (R/live/canvas/cartesian.rs:598, 611-625, 658-662).
- Result: stroke and fill cannot differ. There is no way to set only the stroke color on an area chart that also has a fill token.

##### D2 (still holds, wider than the study says). Typed builders cannot turn dots off
- `LineChartBuilder::dot()` only sets `true` (typed.rs:231-235). Default is `dots: true` (charts.rs:836). No `dots(false)` on any typed builder. `ChartsBuilder::dots(bool)` exists (charts.rs:246-250), so only the typed route is affected.
- Wider: the marker branch is `_ if props.dots && samples.len() == values.len() => Some(Marker::Disc)` (cartesian.rs:679-683). It applies to every non-scatter chart type that reaches this code, so `AreaChartBuilder` (no `dot` method at all, typed.rs:244-354) always draws a disc at every point. The doc string says "line-chart point" (charts.rs:246, 793). [obs]
- `ChartsBuilder::radar()` also has dots on (charts.rs:836; radar.rs:141); the typed radar builder defaults them off (typed.rs:1057).

##### D3. State fields that are written and never read
- `ChartState::tooltip`, `animation_progress`, `animating` (charts.rs:848-859).
- `tooltip` is written at live.rs:159, 374-377, 393, 429, 452 and never read. Render builds the tooltip from `hovered_point` (live.rs:257-262). `tooltip_text` (702-714) exists only to fill it.
- `animation_progress` and `animating`: search `grep -rn "animation_progress\|\.animating" src` shows the chart never touches them (the only other hits are popover.rs). They are public API (src/widgets/mod.rs:36, display/mod.rs:27).

##### D4. `tick_margin` also thins the value axis and its grid, and never thins horizontal-bar categories
- Contract text and doc: "Show every n-th axis label" (charts.rs:264; typed.rs:168).
- Vertical charts: `stride` is applied to the value axis (`Axis::vertical(..).with_skip(stride)`, cartesian.rs:328-338) and to the category axis (350-367). Value grid rows come from `vertical.shown()` (369-372), so grid lines also disappear.
- Horizontal bars: `stride` goes to the value axis (289, 305); the category axis `vertical` has no `with_skip` (266-276, 321-323), so category labels are never thinned and its grid rows use all ticks (313-319).
- The reference's `tick_margin` is a stride over the category axis only (bar_chart.rs:340-342 doc; chart/mod.rs:186-211).

##### D5. Typed bar `.label()` leaks into tooltips
- `BarChartBuilder::build` writes the label into `point.metadata["label"]` for every point (typed.rs:517-523).
- `point_text` appends every metadata pair as `; key=value` (canvas.rs:602-606). Result: the tooltip and announcement text carry `; label=<value label>`. [obs from reading; not run]

##### D6. Labels that can overlap or leave the plot
- Bar value labels (cartesian.rs:477-507): no check against neighbours or lane width. In grouped or dense bar charts, labels wider than a lane write over each other (later write wins in `TextLayer::put`, canvas.rs:141-145). Vertical labels are centred with only a left clamp `.max(plot.x)` (504), no right clamp; the text layer only clips at the chart width (171), and the legend is drawn after and can overwrite (canvas.rs:462-484).
- Radar category labels (radar.rs:154-171): no collision test; text cut by `text.text(.., width)` without an ellipsis (169), unlike pie labels that use `fit_label` (pie.rs:382).
- Pie labels and Sankey labels are guarded (pie.rs:344-381, sankey.rs:199-208). Cartesian x labels are guarded (axis.rs:161-163; tick.rs:156-175).

##### D7. Area gradient direction (possible inversion; not rendered)
- `FillStyle::Gradient`: `t = (y - near) / (far - near)` with `near` the top of the plot; `k = 0.45 + 0.55 * t`; rgb is multiplied by `k` (cartesian.rs:742-748). Top of the plot is the darkest, baseline is full color.
- Manual says "shades the fill toward the baseline" (RT/manual/display-widgets.md:88-89). The reference's gradient story says "fade to the baseline" (chart_story.rs:1259). Ours reaches full brightness at the baseline. I did not render, so I cannot say which reading the author meant. The multiply darkens toward black instead of blending toward the theme background (compare sankey.rs:117-127). [obs for the math; the mismatch with intent is [assume]]

##### D8. Stale reference names
- R/plot/curve.rs:2 still says "the reference's `StrokeStyle`" (renamed `Curve` in 0.7.0).
- docs/spec/charts.md:5 says "gpui-kit 0.6.6".

##### D9. Typed builders cannot switch off value-axis labels
- `Common::finish` sets `x_axis.show_labels` from the builder and leaves `y_axis.show_labels` at the default true (typed.rs:53-61). `common_methods!` has only `x_axis(bool)` (typed.rs:86-90). Ours cannot reproduce the reference's value axis being off by default through typed builders. `ChartsBuilder::y_axis(ChartAxis{..})` can (charts.rs:178-182).

##### Checked and found fine
- `tick_count` cannot be 0 (`max(1)`, canvas.rs:613); `PolarGrid` with 0 levels draws spokes only (polar.rs:33-34, 42).
- `Axis::draw` clamps a tick at `row == plot.h` to the last row (axis.rs:130-133); consistent with the bottom axis row being the last plot row (cartesian.rs:176-187).
- Reveal never scales the value domain: `value_domain` reads `props.series`, not animated `job.values` (cartesian.rs:18-25).
- Pie and radar with `label_gap` or `grid_levels` at 0 do not divide by zero (pie.rs:339-343; polar.rs:42-43).

##### Not checked
- I did not run any test, golden or demo. `tests/charts_goldens.rs:522` lists a 600 x 160 large golden, but I did not read the goldens.
- The catalog page uses 20x5, 60x12 and 60x14 (examples/widget_catalog/catalog.rs:501-506). It shows no chart at 240 to 512 columns. [obs]
- I did not read R/mask.rs beyond the marker enum, so stroke thickness and blitter behavior are from comments and call sites only.
- I did not read the gpui-kit `stories` beyond the list of cards and the "fade to the baseline" note.


### Input widgets: Input widgets: reactive-tui vs gpui-kit 0.7.0

Read-only review. No file in either repo was changed. Nothing was built or run. I read source only, so I make no claim about how anything looks on screen.

#### Path aliases

- `R/` = ~/workspace2/reactive-tui/
- `C7/` = ~/workspace2/gpui-kit-0.7.0/crates/component/src/
- `B7/` = ~/workspace2/gpui-kit-0.7.0/crates/base/src/
- `C6/`, `B6/` = the same paths under ~/workspace2/gpui-kit-0.6.6/crates/
- `study` = R/docs/widget-study.md

#### How the comparison was done

- `diff -rq` and `diff` between C6/B6 and C7/B7 for the matching paths.
- Then a full read of R/src/widgets/input/*.rs, R/src/widgets/input/text_input/*.rs, R/src/builder/widgets/input.rs and the radio and slider parts of R/src/builder/specialized.rs.
- I did not read R/src/widgets/dialog/input*. It is outside `widgets/input`.
- I did not read the App's event routing or the layout engine. Where a claim depends on them I say so.

---

#### Part 1. What changed in gpui-kit between 0.6.6 and 0.7.0

#### 1.0 The release notes do not cover these families

- release-notes.md (164 lines) lists only Root overlays, `SettingGroup::variant`, the Plot move and its breaking changes, and `gpui_base::Root` / `open_window` (gpui-kit-0.7.0/release-notes.md:5-164).
- I searched it with grep for input, token, checkbox, radio, select, combobox, slider, switch, rating, form, label, time, questionnaire, otp and mask. The only hits were unrelated words (`Root` "selection", chart "label", "tokens" for motion). So every change below comes from `diff`, not from notes.
- The breaking change that touches this group and is not in the notes: `DatePickerEvent::Change` now carries `DateTime`, not `Date` (C7/time/date_picker.rs, the `Change(DateTime)` variant; C6 had `Change(Date)`).

#### 1.1 Files that are byte-identical in 0.6.6 and 0.7.0

Checked with `diff -q`. No change of any kind:

- C7/checkbox.rs, C7/radio.rs, C7/slider.rs, C7/label.rs
- B7/checkbox.rs, B7/radio.rs, B7/radio_group.rs, B7/slider.rs, B7/select.rs, B7/combobox.rs, B7/number_input.rs, B7/otp_input.rs
- C7/input/number_input.rs, C7/input/otp_input.rs, C7/input/content_type.rs, C7/input/clear_button.rs, C7/input/group.rs, B7/input/base/mask_pattern.rs, B7/input/base/selection.rs

So for checkbox, radio, slider, label, number input and OTP input there is no change. The `study` text for them is still true for 0.7.0, apart from line numbers that moved in the files that did change.

#### 1.2 Changes by family

##### Text input (C7/input/, B7/input/)

Size of diff: input.rs 197 lines, base/state.rs 1201, base/element.rs 1525.

1. New inline tokens. An atomic chip inside the text.
   - Types: `InlineToken` (id, text, label; validated: id not blank, text and label not empty, no control chars or U+2028/2029), `InlineTokenSpan`, `InputContent` (text plus token spans, validated on attach), `InlineTokenError` (B7/input/base/inline_tokens.rs:9-170).
   - `InputContent::with_token` rejects: empty range, range off a grapheme boundary, text that differs from the token text, overlap (B7/input/base/inline_tokens.rs:99-128).
   - Ranges are UTF-8 byte offsets (B7/input/base/inline_tokens.rs:1).
   - `set_value` now takes plain text or `InputContent`, clears history, emits no Change (B7/input/base/state.rs:925-952).
   - Presentation types: `InlineTokenContext` (selected, disabled, readonly, line height, available width), `InlineTokenClickEvent`, renderer and click listener aliases (B7/input/base/token_presentation.rs:6-70).
   - Builders on the styled controls: `Input::token(render)` and `Input::on_token_click(listener)` (C7/input/input.rs:178-195); the same two on `Textarea` (C7/input/textarea.rs, `token`, `on_token_click`).
   - Default token look is `InputToken`: `gap_1`, `px_1`, `border_1`, radius from the theme, `bg` = `theme.muted` and border = `theme.border`, or `theme.selection` when selected, opacity 0.5 when disabled, ellipsis on overflow, optional icon (C7/input/token.rs:1-69).
   - Screen reader: a token gets role `Button`, its label and a Click action only when a click listener is set and the token is not disabled (B7/input/base/element.rs:1458-1480). Without a listener the token has no role of its own.
   - Keyboard: an `ActivateToken` action opens the selected token (B7/input/base/state.rs:4484-4500). I grepped `KeyBinding` and `ActivateToken` under B7/ and found no default key bound to it. The application must bind it. (Assumption: none in C7 either; I searched B7 only.)
   - A press on a token selects it; opening is the listener's job (B7/input/base/element.rs, comments at the `token_element` function).
2. New range decorations (`RangeDecoration`, `RangeDecorationStyle`, `range_decorations`): geometric highlights over ranges, clipped to the viewport, folds and wraps (C7/input/mod.rs:31-38 exports; B7/input/base/kind.rs:83-90; B7/input/base/element.rs `layout_range_decorations`). Not tokens. Editor-oriented.
3. Behavior changes on `Input` (C7/input/input.rs):
   - SetValue from a screen reader is ignored unless the input is editable (`is_editable`), so it no longer edits a read-only or disabled input (lines 488-495 and 754-758).
   - New screen-reader Focus action, refused when disabled (lines 497-501, 750-753).
   - A disabled input swallows every mouse down (lines 732-734).
   - Right-click menu: Paste is offered whenever the text can change; it no longer checks the clipboard (lines 631-640). A custom `context_menu` shows only while the state's context menu is enabled (lines 355-358; B7/input/base/state.rs:778-781). Turning the menu off now also hides a custom menu.
   - Code-editor left padding is capped at 6 px (lines 582-590).
   - `Textarea` gets `Sizable`/`with_size` and passes the size to its input (C7/input/textarea.rs, `impl Sizable for Textarea`, `into_input`).
4. Engine changes in B7/input/base/state.rs:
   - Paste is a single atomic edit, drops newlines in a single-line input, is skipped when the clipboard has no text (image), and a late clipboard read is compared with the target captured at Paste time (`PasteTarget`, `paste_target`, `insert_clipboard`, lines 2709-2760).
   - Vertical selection at the first or last visual row now extends to the document edge (B7/input/base/movement.rs, `vertical_selection_target`).
   - Undo merges a run of single-cursor keystrokes into one recorded change (B7/input/base/undo_manager.rs:24-28, 187-215). Limits unchanged: 1000 transactions (`MAX_UNDO_TRANSACTIONS`, line 6).
   - Masking is not re-applied while history is replayed (undo/redo) (B7/input/base/state.rs:4096, `!self.replaying_history && !self.mask_pattern.is_none()`).
   - Blink cursor `stop` now resets the blink state; stale tasks from an older blink cycle no longer change state (B7/input/base/blink_cursor.rs:42-63).
   - A single line is centered in a taller frame (test `single_line_is_centered_in_a_taller_frame`, B7/input/base/state.rs:4919).
   - Line-number gutter starts at 3 digits and grows to 7 (B7/input/base/element.rs, `line_number_len`, `displayed_line_number`).

##### Number input, OTP input, mask

- Number input and OTP input files: no change (section 1.1).
- Mask: only the replay guard above. `MaskPattern` file is unchanged.
- `Input` password reveal, `mask_toggle`, `cleanable`, `prefix`, `suffix`: no change to the option set (C7/input/input.rs:265-315).

##### Checkbox, radio, slider, label

No change. See 1.1. Consequence for the two claims you asked about:

- Radios have no arrow keys in 0.7.0 either. B7/radio.rs and B7/radio_group.rs are identical to 0.6.6. I grepped both plus C7/radio.rs for `key_context`, `on_action`, `KeyBinding`, `on_key`, `arrow` and found only focus and `tab_stop` plumbing (B7/radio.rs:32-33, 136-166, 221-224). Enter and Space come from the click path (test at B7/radio.rs:359-377).
- Sliders have no keyboard handling in 0.7.0 either. B7/slider.rs and C7/slider.rs are identical. I grepped both for `key_context`, `KeyBinding`, `on_key`, `track_focus`, `focus_handle`, `on_action`. Only `on_a11y_action` Increment and Decrement exist (B7/slider.rs:489-508).
- New in 0.7.0 that does add arrow keys for radios: the questionnaire module, not the Radio widget. When a single-choice question has a focused choice, Up/Down/Left/Right move to the previous/next choice (B7/questionnaire/keyboard.rs:56-71, `move_current_radio` at B7/questionnaire/state.rs:774). This is a separate widget.

##### Select and combobox

- `SelectState` emits `DismissEvent` whenever an open menu closes, including after a confirmed pick (C7/select.rs:119-120, 431-441).
- Popup width: `bounds.width + 2px` became `bounds.width` for both select and combobox (C7/select.rs:612-618; C7/combobox.rs:1049-1053).
- List role: `Role::List` moved from the end of the list render to line 670 (C7/list/list.rs:670; was at line 790 in C6). Row content now shrinks with `min_w_0` and truncates (C7/list/list_item.rs:125-140; C7/searchable_list/adapter.rs:133-134). Old rows used `whitespace_nowrap`.
- `SearchableVec` keeps an index list instead of cloned items (C7/searchable_list/vec.rs:73-97). Internal.
- Keyboard: no new keys. Base select is identical (B7/select.rs:17-27).
- Combobox: nothing else changed (4-line diff).

##### Switch

The largest change among the small widgets (C7/switch.rs, 202 diff lines; B7/switch.rs, 23):
- New options `tab_stop`, `tab_index` (C7/switch.rs:112-124) and `focus_ring` via `FocusableExt` (C7/switch.rs:147-156). The switch now owns a keyed focus handle and tracks it (C7/switch.rs:166-170, 229-233).
- The focus ring is drawn on the track only, not on the label row (C7/switch.rs:268-272).
- The track no longer shrinks with a long label: `flex_shrink_0` on the track, `min_w_0` on the label (C7/switch.rs:246-250, 286-290). Track inset is now a 1px border plus padding so the ring stays visible (C7/switch.rs:252-262).
- Base switch accepts a caller focus handle: `track_focus` (B7/switch.rs:319-331).

##### Rating

Hover updates are guarded: no `notify` when the hovered value does not change (C7/rating.rs:143-146, 172-176). No option change.

##### Form and label

- Form: a `Field` marked not visible is now hidden and skipped by the form (C7/form/field.rs:170-173, 284; C7/form/form.rs:139, 151).
- Form: `Form` now applies its own style refinements (`.refine_style`, C7/form/form.rs:151; test at C7/form/tests.rs:156-246).
- Label: no change.

##### Time

- New `time/time_field.rs`, plus base logic in `B7/time_field.rs` (775 lines).
- `DatePicker` can edit a time of day: `time_precision`, `hour_cycle`, `default_time`, `DateTime` value, `date_time()`, `set_date_time()` (C7/time/date_picker.rs, the `+` lines of the diff; 459 diff lines). `date_format` default becomes date plus time when a precision is set. Selecting a date keeps the popup open when a precision is set. A range picker edits dates only.
- Calendar: cell text is `text_xs` at `Size::Small`, `text_sm` otherwise (C7/time/calendar.rs:118-127).

##### Questionnaire (new)

See Part 5.

#### 1.3 Statements in `study` that are no longer true for 0.7.0

Only statements about files that changed can be stale. Findings:

1. study "input (text)", Keyboard and screen reader: "The SetValue action replaces the text". Now true only for an editable input. Read-only and disabled refuse it (C7/input/input.rs:488-495, 754-758).
2. study "input (text)", Only gpui-kit: "a custom context menu". Now conditional on the state's context menu being enabled (see 1.2 item 3).
3. study "input (text)", keys: right-click "opens a native menu with Cut, Copy, Paste and Select All". Paste is now enabled without checking the clipboard (C7/input/input.rs:634-640).
4. study lists "undo that merges a run of typing into one step and keeps 1000 steps (B/undo_manager.rs:6, 176-203)". Still true. The merge code moved to B7/input/base/undo_manager.rs:187-215. Update the line numbers.
5. study "select", Keyboard: "Rows are role ListItem ... inside role List (G/list/list.rs:481, 488-495), (G/list/list.rs:788-790)". Still true, but the `Role::List` line moved to C7/list/list.rs:670.
6. study gives all `G/input/input.rs` line numbers from 0.6.6. Every one is off in 0.7.0 (the file grew by the token fields and accessibility code).
7. study "input (text)", "Only gpui-kit" list has no tokens, no range decorations and no `Textarea` sizing. These are new.
8. study "gpui-kit components we lack: time" and "switch" and "rating" sections: `time` now has a time field and date-time value; switch has focus ring and tab options. The study's statements about them describe 0.6.6. (I did not read those study sections; I checked only the summary and the sections you named. I infer this from the diffs. Unverified against the text.)

Statements that are still true:

- "Radios: ... no arrow keys" (study line 178). True for Radio and RadioGroup. See 1.2. Not true of the new questionnaire's choice list.
- "Sliders: ... no keyboard handling" (study line 181). True.
- Study Summary item 3 (names, positions and counts for screen readers) and item 7 (named key actions): the files it cites are unchanged.
- "gpui-kit's popup is a deferred overlay (G/select.rs:601-604)": still true; the code now sits at C7/select.rs:604-632.

---

#### Part 2. Family by family: gpui-kit 0.7.0 vs reactive-tui today

Sizes in gpui-kit are pixels; in reactive-tui they are terminal cells. Do not equate numbers. I give the number and the unit.

#### 2.0 Cross-cutting: colors and states

Theme source.
- gpui-kit: every color comes from `cx.theme()`: `input`, `input_background()`, `primary`, `primary_foreground`, `foreground`, `muted_foreground`, `ring`, `selection`, `danger`, `tokens.slider_bar`, `tokens.slider_thumb` (C7/checkbox.rs:238-246; C7/radio.rs:185-192; C7/slider.rs:164-170; C7/select.rs:519, 552-560; C7/input/input.rs:98-107, 694-701).
- reactive-tui: `grep -rn -i theme R/src/widgets/input` finds nothing. Checkbox, radio, select and slider set no color at all; they emit text plus layout classes (R/src/widgets/input/checkbox.rs:127-185; radio_button.rs:203-243; select.rs:279-304; slider.rs:448-471). Only text input sets colors, and they are literal palette classes: `text-gray-500`, `bg-white text-black`, `bg-blue-600 text-white`, `text-red-500` (R/src/widgets/input/text_input/paint.rs:275, 300, 304, 306, 324, 354).
- The theme can supply roles: `--color-primary`, `--color-foreground`, `--color-background`, `--color-surface`, `--color-text-muted`, `--color-border`, `--color-error` etc. (R/src/theme/presets.rs:5-23), and `Theme::resolve_color` maps `primary`, `muted`, `chart-1` before falling back to palette names (R/src/theme/mod.rs:112-131). The presets have no `selection`, `input`, `ring`, `disabled` or `focus` role (`grep -n "selection\|focus\|--color-input\|disabled\|--color-ring" R/src/theme/presets.rs` = no hits).

State table (what each side draws):

| State | gpui-kit 0.7.0 | reactive-tui |
|---|---|---|
| focus | ring from `focus_ring_style` on checkbox, radio, select, input, switch (C7/checkbox.rs:281-285; C7/radio.rs:216-218; C7/select.rs:556-559; C7/input/input.rs:775-777) | a `▶ ` glyph in front, no color (checkbox.rs:133-134; radio_button.rs:299-302; named_radio.rs:139; select.rs:244-245; slider.rs:392-396; paint.rs:62-63). Input cursor is a `bg-white text-black` cell (paint.rs:300, 324) |
| hover | slider thumb ring only (C7/slider.rs:220-230). Checkbox, radio, select: none found (`grep -i hover` in those three = no hits) | checkbox and radio: label becomes `_label_` (checkbox.rs:157-158; radio_button.rs:311-312). Slider: marker `◉` (slider.rs:387-388). Select: mouse move highlights a row (select.rs:827-835) |
| disabled | opacity 0.5 and `muted_foreground` text (C7/checkbox.rs:246, 336-339; C7/select.rs:546; C7/input/input.rs:700-703, 788, 804) and mouse blocked on input (C7/input/input.rs:732-734) | `🔒 ` glyph (checkbox.rs:131-132; select.rs:240-241; paint.rs:60-61), label in `(...)` (checkbox.rs:155-156; radio_button.rs:309-310), slider marker `○` (slider.rs:385-386). No dim color |
| invalid | Input: none (no `invalid` in C7/input/input.rs; `grep -n -i invalid` = no hits). `InputGroup::invalid` and `TimeField::invalid` exist (C7/input/group.rs:88-90; C7/time/time_field.rs:69-73). Checkbox, radio, select, slider: none | text input only: `❌ ` glyph and optional red error line (paint.rs:58-59, 352-356) |
| read-only | `Input::readonly`; same look as normal, edits refused (C7/input/input.rs:342-348, 597) | text input only, and only when a caller sets the private closure. Not drawn differently (text_input.rs:381-391; only user: R/src/widgets/dialog/input/live.rs:293). Builder `.readonly()` does not call it, see Part 4 D2 |
| loading | Input: spinner in the suffix (C7/input/input.rs:805-807). Others: none | none |

#### 2.1 Text input

Layout defaults.

| Item | gpui-kit 0.7.0 | reactive-tui |
|---|---|---|
| Horizontal padding | `input_px`: 4/8/10/12 px for XSmall/Small/Medium/Large (C7/sizing.rs:147-155) | none. The row is `status + [ + text + ]`; status is 2 cells, or 3 with a 2-cell emoji (paint.rs:57-67, 268-278, 333) |
| Vertical padding | `input_py`: 0/2/8/10 px (C7/sizing.rs:158-165); not applied to multi-line input (C7/input/input.rs:762-764) | none. Each row is `h-1` (paint.rs:340) |
| Gap between parts | `gap_x` 4/6/8 px by size (C7/input/input.rs:688-692, 772) | prefix is status (2 or 3 cells) plus 1 cell for `[`, then the text, then `]` (paint.rs:77-79) |
| Height | `input_h`: 20/24/32/44 px (C7/sizing.rs:268-276) | 1 row, or `MultiLine{height}` rows (paint.rs:87-90) |
| Width | root is `.size_full()` (C7/input/input.rs:760): takes the width its parent gives | fixed: `width.unwrap_or(30)` cells of text (paint.rs:86, 97-100), default `Some(30)` (text_input.rs:67, 244). The outer box says `w-full` (paint.rs:377) but the painted row is `prefix + min(width, available) + ]` |
| Minimum width | none set | none, but `available_width` uses `saturating_sub` so it can reach 0 (paint.rs:92-93, 98) |
| Alignment | `text_align` from style, default Left (C7/input/input.rs:527) | left only |
| Wide parent (240+ cols) | grows to fill | stays at 30 text cells. In the catalog the input has `class("w-full")` (R/examples/widget_catalog/catalog.rs:359-363) and the field is still 30 cells |

Options and builder methods.
- Only gpui-kit: `prefix`, `suffix`, `cleanable`, `mask_toggle`, `content_type`, `role`, `aria_label`, `appearance`, `bordered`, `focus_bordered`, `h`, `tab_index`, `context_menu`, `on_paste`, `token`, `on_token_click` (C7/input/input.rs:178-361); state: `masked`, `mask_pattern`, `pattern`, `validate`, `clean_on_escape`, loading (study lines 596-606, unchanged).
- Only reactive-tui: `max_length` (graphemes), `numeric` mode, `validator_pattern` and named validators, `error_message`, `suggestions`, `show_line_numbers`, `wrap_text`, `tab_size`, `auto_indent`, `width` (R/src/widgets/input/text_input.rs:45-204). Builder `text_input()` has only `value`, `placeholder`, `disabled`, `readonly`, `max_length`, `input_type`, `class` (R/src/builder/widgets/input.rs:96-197). It has no `width`, no change or submit callback (input.rs:230-232).

Keyboard.
- gpui-kit: full set from actions; bindings are in B7/input/base/state.rs (unchanged list except tokens). `Escape` clears when `clean_on_escape`.
- reactive-tui: Ctrl+A/C/X/V/Z/Y, Ctrl+arrows and Home/End, Ctrl+Backspace/Delete (text_input.rs:830-884); Enter, Tab, Escape, arrows, Home/End/Page (text_input.rs:1039-1164). Alt keys are ignored (text_input.rs:890-899, 1036-1038). Super/Meta is not checked (see D5).

Screen reader.
- gpui-kit: role by content type: TextInput, PasswordInput, EmailInput, UrlInput, PhoneNumberInput, DateTimeInput, DateInput, MultilineTextInput (C7/input/input.rs:27-87). Label from `aria_label`, else the placeholder unless it is a mask placeholder (C7/input/input.rs:717-723). Value withheld when masked or password (C7/input/input.rs:89-96). Actions: Focus and SetValue (C7/input/input.rs:750-758).
- reactive-tui: role PasswordInput, MultilineTextInput, NumberInput, TextInput; `read_only` flag; placeholder as description; value, caret and selection as text runs (R/src/widgets/input/text_input/accessibility.rs:13-42). No label from the placeholder. No Focus/SetValue handling here; the study says the App handles only Focus and Click (study lines 61-66; R/src/app.rs:411-417 as cited there; I did not re-read app.rs).

#### 2.2 Checkbox

| Item | gpui-kit 0.7.0 | reactive-tui |
|---|---|---|
| Box size | 12/14/16/18 px (`0.75/0.875/1/1.125` rem) (C7/checkbox.rs:219-224) | `[x]` = 3 cells, fixed (checkbox.rs:139-150) |
| Gap box to label | `gap_2` = 8 px, label column `flex_1`, inner gap `gap_1` (C7/checkbox.rs:271, 327-331) | 1 cell (checkbox.rs:153-154) |
| Leading area | none | 2 or 3 cells: `  `, `▶ `, or `🔒 ` (checkbox.rs:130-137) |
| Width | root `h_flex`, no width; label column `flex_1` `overflow_hidden` (C7/checkbox.rs:270, 326-328) | one text node; no width or wrap class (checkbox.rs:181) |
| Sizes | XSmall..Large (C7/checkbox.rs:219-224) | one size |
| Label spoken | `accessibility_label` override, else label (C7/checkbox.rs:229-231) | label only (checkbox.rs:166-168) |

Options. Only gpui-kit: size, tooltip, child content, tab index/stop, focus ring switch, role override, label override. Only reactive-tui: `indeterminate` in the styled widget (gpui-kit has it only in the base part, B7/checkbox.rs:16-22). Builder: no change callback (R/src/builder/widgets/input.rs:319-405). The component has `with_on_change` (checkbox.rs:105-108).

Keyboard. gpui-kit: Space/Enter from the click path, one Tab stop. reactive-tui: Space or Enter toggles when focused; left mouse down toggles (checkbox.rs:198-273). Indeterminate goes to checked (checkbox.rs:231-234).

Screen reader. gpui-kit: role CheckBox, toggled (True/False/Mixed), label (B7/checkbox.rs:374-379). reactive-tui: role CheckBox, label, toggled incl. Mixed, Click or disabled (checkbox.rs:164-180). Equivalent, except no name override.

#### 2.3 Radio

| Item | gpui-kit 0.7.0 | reactive-tui |
|---|---|---|
| Circle size | same rem sizes as checkbox (C7/radio.rs:168-172) | `( )` = 3 cells (radio_button.rs:316; named_radio.rs:137-142) |
| Gap circle to label | `gap_x_2` = 8 px (C7/radio.rs:211) | 1 cell |
| Gap between radios | `gap_3` = 12 px (C7/radio.rs:402) | vertical: 0 (rows, radio_button.rs:209). horizontal: class `gap-0.5` (radio_button.rs:207); the mouse hit test assumes 2 cells (radio_button.rs:398). The class table maps `0.5` to 2 (R/src/layout/css/parsers.rs:44); I did not verify the px-to-cells conversion |
| Group width | horizontal group: `w_full().flex_wrap()` (C7/radio.rs:394); vertical: `v_flex` | `flex flex-row overflow-hidden`: no wrap (radio_button.rs:207) |
| Leading area | none | 2 cells `▶ ` or blanks (radio_button.rs:299-303) |

Options. Only reactive-tui: values of any type, per-option `disabled` (in gpui-kit the group `disabled` overwrites each radio's flag, C7/radio.rs:409). Only gpui-kit: size, tooltip, child content, tab options, label override, position in set. Builder `radio_button()` makes a `NamedRadio`; no change callback and `NamedRadioProps` is `pub(crate)` (R/src/builder/specialized.rs:233-296; R/src/widgets/input/named_radio.rs:56-63).

Keyboard.
- gpui-kit: Enter/Space on a radio; each radio is its own Tab stop (C7/radio.rs:198-201). No arrows (see 1.2).
- reactive-tui `RadioButton`: arrows move focus and skip disabled options, but do not select; Space/Enter select (radio_button.rs:349-365). All four arrow keys work in both orientations (radio_button.rs:349). `NamedRadio`: Space/Enter/click only; each radio is a Tab stop (named_radio.rs:165-178).

Screen reader.
- gpui-kit: radio has toggled and selected, label, position in set, size of set; group has orientation (B7/radio.rs:203-219; B7/radio_group.rs:59-63).
- reactive-tui: group is `RadioGroup` with no label and no orientation (radio_button.rs:211); options are `RadioButton` with label and toggled, plus a focus event (radio_button.rs:214-239). No selected flag, no position in set. `NamedRadio`: same node without the group (named_radio.rs:124-136).

#### 2.4 Select

| Item | gpui-kit 0.7.0 | reactive-tui |
|---|---|---|
| Trigger size | `input_size`: height 20/24/32/44 px, `input_px` 4..12 px (C7/select.rs:550) | 1 row |
| Trigger width | `size_full`, inner `w_full().min_w_0()` truncates (C7/select.rs:526, 566-567, 577-580) | root class `w-{width}` with `width.unwrap_or(30)`, and `max-w-full` (select.rs:279-283). Default 30 (select.rs:27, 170) |
| Gap title to caret | `gap_1` (C7/select.rs:572) | none |
| Popup width | trigger width, or a set width (C7/select.rs:612-618) | inline list under the header, same 30 cells |
| Popup height | `menu_max_h` default `rems(20)` (C7/select.rs:106) | `max_visible_items` default 5, limited by room left (select.rs:169, 394-402) |
| Popup padding | `Edges::all(4px)` (C7/select.rs:629) | 2 leading cells per row (select.rs:319) |
| Truncation | CSS truncate | cut by grapheme, no ellipsis (select.rs:250-261) |

Options. Only gpui-kit: search box, groups, `cleanable`, `title_prefix`, icon, `empty`, `menu_width`, `menu_max_h`, size, `appearance`, focus ring switch, selection veto hook (study lines 1108-1119, unchanged). Only reactive-tui: multi-select in the builder, type-ahead, `width`, `max_visible_items`, wheel scroll, open and close callbacks. Builder `select()` has `option`, `options`, `selected`, `placeholder`, `disabled`, `multiple`, `class` (R/src/builder/widgets/input.rs:430-533); it cannot set `width`, `max_visible_items` or a per-option disabled flag (input.rs:542-556 uses `..Default::default()`).

Keyboard.
- gpui-kit: up, down, enter, secondary-enter, escape in a "Select" context (B7/select.rs:17-27); no Home/End/Page (unchanged; list.rs did not change key bindings in the diff I read).
- reactive-tui: Enter/Space open or choose; Escape; Up/Down; Home/End/PageUp/ PageDown when open; printable keys jump by prefix (select.rs:646-705).

Screen reader.
- gpui-kit: ComboBox with expanded, label, value (B7/select.rs:195-203); Click action (B7/select.rs:216); rows `ListItem` with position and set size.
- reactive-tui: ComboBox with value, expanded, multiselectable, no label (select.rs:267-278); list `ListBox`, rows `ListBoxOption` with label and selected (select.rs:296-304, 342-350).

#### 2.5 Slider

| Item | gpui-kit 0.7.0 | reactive-tui |
|---|---|---|
| Track thickness | `h_6` hit area, `h_1p5` bar (C7/slider.rs:279, 288) | 1 row |
| Thumb size | `size_4` = 16 px (C7/slider.rs:218) | 1 cell |
| Length | horizontal `w_full` and `flex_1`; vertical `h(120px)` (C7/slider.rs:265-269) | `width` cells, default 20 (slider.rs:33, 174); `available.min(props.width)` never lets it grow (slider.rs:261, 270) |
| Decorations | none | focus glyph 2 cells, optional min/max labels, value text (slider.rs:238-273, 398-421) |
| Builder length | n/a | `builder::slider()` cannot set width, show_value or show_labels; it uses `..Default::default()` (R/src/builder/specialized.rs:391-400) |

Options. Only gpui-kit: range with two thumbs, log scale, `reverse`, Change and Release events (study lines 1170-1176; C7/slider.rs:124-135). Only reactive-tui: `show_value`, `show_labels`, `width`, label, invalid-range message.

Keyboard. gpui-kit: none. reactive-tui: arrows, PageUp/Down at 10% of range, Home/End (slider.rs:514-529); click and drag (slider.rs:541-559).

Screen reader. gpui-kit: Slider role, value, min, max, step, orientation, and Increment/Decrement actions (B7/slider.rs:479-508). reactive-tui: Slider role, label, value, min, max, step, orientation (slider.rs:453-465); no Increment/Decrement handling (study line 66 cites the App).

---

#### Part 3. Clean-up candidates for reactive-tui's input widgets

Size: small = under 200 lines, medium = 200 to 800, large = more. "Ref" is where gpui-kit 0.7.0 does the matching thing. Ordered by how likely each is to matter at 240-512 columns.

#### A. Fixed numbers that ignore a wide parent

A1. Text input does not fill its parent. Text width defaults to 30 and `None` also means 30. The `w-full` on the outer box does nothing to the row.
- Ours: R/src/widgets/input/text_input/paint.rs:86, 97-100, 377; R/src/widgets/input/text_input.rs:67, 244.
- Ref: `size_full` root, C7/input/input.rs:760.
- Fix idea: let `width: None` mean "fill", use `layout.content_size()` for the text width, and keep `Some(n)` for a fixed size. Builder needs a `width`.
- Size: small.

A2. Select is 30 cells wide by default and `None` also means 30.
- Ours: R/src/widgets/input/select.rs:27, 170, 279-283, 404-409.
- Ref: `w_full().min_w_0()` trigger, popup follows trigger, C7/select.rs:566-567, 612-618.
- Size: small. Same fix as A1.

A3. Slider track stops at 20 cells. `available.min(props.width)` blocks growth.
- Ours: R/src/widgets/input/slider.rs:33, 174, 261, 270.
- Ref: `w_full` horizontal, C7/slider.rs:269.
- Fix idea: an "auto" width that uses the parent's content width minus labels.
- Size: small.

A4. Builders cannot set the sizes at all: `builder::text_input()`, `builder::select()`, `builder::slider()` all end in `..Default::default()` (30, 30/5, 20).
- Ours: R/src/builder/widgets/input.rs:206-221, 542-556; R/src/builder/specialized.rs:391-400.
- Ref: `w()`/`h()` on the gpui-kit builders through `Styled`.
- Size: small.

A5. Horizontal radio group does not wrap and clips at the edge (`overflow-hidden`).
- Ours: R/src/widgets/input/radio_button.rs:207.
- Ref: `h_flex().w_full().flex_wrap()`, C7/radio.rs:394.
- Cost: wrap changes `option_at` hit-testing (radio_button.rs:385-399), which assumes a single row.
- Size: medium.

A6. Gap and hit-test both hold the number 2 in separate places (class `gap-0.5` and `start = end + 2`). If one changes the mouse target drifts.
- Ours: R/src/widgets/input/radio_button.rs:207 and 398.
- Size: small.

A7. Select rows are cut with no ellipsis; long labels lose their tail silently.
- Ours: R/src/widgets/input/select.rs:250-261.
- Ref: `truncate`, C7/select.rs:577-580; C7/searchable_list/adapter.rs:133-134.
- Size: small.

#### B. Colors written as literals

B1. Text input: replace palette classes with theme roles.
- Ours: R/src/widgets/input/text_input/paint.rs:275, 306 (`text-gray-500` -> `text-muted`), 354 (`text-red-500` -> `text-error`), 300 and 324 (`bg-white text-black` cursor -> `bg-foreground text-background`), 304 (`bg-blue-600 text-white` selection -> needs a `selection` role).
- Ref: C7/input/input.rs:98-107 and `theme.selection`, `theme.ring`, `theme.input`, `theme.danger`.
- The presets have `--color-text-muted`, `--color-error`, `--color-foreground`, `--color-background`, `--color-primary` (R/src/theme/presets.rs:5-23) but no `selection`, `input` or `ring`. Adding them is part of this item.
- Size: small for the swap, medium with new theme variables in every preset.

B2. Checkbox, radio, select and slider draw no color for focus, hover, disabled or selected. State is shown by glyph and text only. Adding roles (for example focus row `bg-primary`, disabled `text-muted`) is a design change, not a bug.
- Ours: R/src/widgets/input/checkbox.rs:127-185; radio_button.rs:297-317; named_radio.rs:137-146; select.rs:236-304, 311-364; slider.rs:379-471.
- Ref: C7/checkbox.rs:238-246, 299-320; C7/radio.rs:185-192; C7/select.rs:519-560.
- Size: medium.

#### C. State drawn by changing text width

C1. The state glyph is 2 cells wide for `▶ ` and blanks but 3 cells for `🔒 ` and `❌ ` (2-cell emoji plus a space). Label and text start shift by 1 cell between states. The checkbox and radio also add `_` around the label on hover and `(...)` when disabled, so the label grows by 2 cells.
- Ours: checkbox.rs:130-137, 155-158; radio_button.rs:299-315; select.rs:240-248, 253; paint.rs:57-67.
- Ref: gpui-kit changes opacity and color, not the size.
- Fix idea: keep a fixed 2-cell state column and change color, not text.
- Size: small per widget.

C2. Radio unselected mark differs between the two radio widgets: `( )` in `RadioButton`, `(○)` in `NamedRadio`.
- Ours: radio_button.rs:304-308; named_radio.rs:140.
- Size: small.

#### D. Options that exist but have no effect

D1. `input_type` values other than password, number and email are dropped (`tel`, `url`, `search`, `date`, and so on become plain single-line). The doc comment says "text, password, email, number, etc.".
- Ours: R/src/builder/widgets/input.rs:164-171, 213-218.
- Ref: `content_type` picks role and hints, C7/input/input.rs:321-327 and 27-87.
- Size: small to map to roles; medium if new validators are added.

D2. `NamedRadioProps.checked` is read once, in `new` (named_radio.rs:96). Later prop changes are ignored; `update` only re-joins the group (named_radio.rs:107-119).
- Size: small.

D3. Builder `.readonly(true)` never reaches the inner `TextInput::with_read_only`, so the read-only flag is not set on the accessibility node, and `execute_command` does not know it (see D2 in Part 4).
- Ours: R/src/builder/widgets/input.rs:230-232, 260-290; R/src/widgets/input/text_input.rs:381-391.
- Size: small.

D4. Dead fields: `Checkbox.state` and `Select.state` are written in `update` and never read.
- Ours: R/src/widgets/input/checkbox.rs:93, 123; R/src/widgets/input/select.rs:200, 503.
- Size: small.

#### E. Gaps in what the reference does

E1. Prefix and suffix cells, clear button, password reveal inside the text input frame.
- Ref: C7/input/input.rs:265-315, 788-807.
- Ours: frame at R/src/widgets/input/text_input/paint.rs:268-278, 333.
- Size: medium.

E2. Change callbacks on all builders (`text_input`, `checkbox`, `radio_button`, `slider`, `select`). The manual says callbacks can be set "through the matching builder" (R/manual/input-widgets.md:26-28); they cannot.
- Ours: R/src/builder/widgets/input.rs:230-232; R/src/builder/specialized.rs:283-296, 409-445.
- Ref: gpui-kit builders take `on_change` (study Summary item 1).
- Size: medium (touches five builders and the way props are stored).

E3. Radio group: name, orientation and selected flag on the accessibility nodes.
- Ours: R/src/widgets/input/radio_button.rs:211-231.
- Ref: B7/radio_group.rs:59-63; B7/radio.rs:203-219.
- Size: small.

E4. Select needs a name for screen readers apart from its value.
- Ours: R/src/widgets/input/select.rs:267-278.
- Ref: B7/select.rs:195-203.
- Size: small.

E5. `invalid` and `disabled` looks as color, not glyph, on text input, select and slider. gpui-kit has `invalid` only on group and time field, so there is no reference for select and slider invalid state.
- Size: medium.

E6. Slider Increment/Decrement/SetValue from screen readers.
- Ours: R/src/widgets/input/slider.rs:453-465 (no handler). Ref: B7/slider.rs:489-508.
- Size: medium (App dispatch change, study item 2).

---

#### Part 4. Defects found by reading

Each item is from reading the code, not from running it. "Confirmed by reading" means the lines say so; nothing was executed. Items marked (assumption) depend on code I did not read.

D1. An empty email field is drawn invalid before the user types.
- `initial_state` and `update` call `validate(value, pattern)` (R/src/widgets/input/text_input.rs:918-923, 946). For `"email"` the regex is `^[^\s@]+@[^\s@]+\.[^\s@]+$` (text_input.rs:482-483), which does not match `""`. So `is_valid` is false at once and `status` returns `❌ ` (paint.rs:57-59). The `numeric`, `alpha`, `alphanumeric` checks use `.all(...)`, which is true for the empty string (text_input.rs:474-480). So they behave differently.
- `builder::text_input().input_type("email")` sets this pattern (R/src/builder/widgets/input.rs:218).

D2. Builder `.readonly(true)` misses the accessibility flag.
- `ConfiguredTextInput::new` builds `TextInput::new(...)` with no read-only closure (R/src/builder/widgets/input.rs:230-232). `with_read_only` has one caller, the dialog (R/src/widgets/dialog/input/live.rs:293). Read-only is enforced only by a key filter in `handle_event` (input.rs:260-290). The accessibility node checks `is_read_only()` (accessibility.rs:20-22), which is false here. The filter also swallows Enter (Consumed) while read-only (input.rs:270-283).

D3. Static suggestions are not filtered by the typed text.
- After any text change from a Char key or paste, the suggestion callback runs only if set (text_input.rs:1012-1014). Then `show_suggestions = !props.suggestions.is_empty()` (text_input.rs:1015). The builder's `suggestions(...)` list is static (text_input.rs:146-153, 192) and there is no `starts_with`/filter anywhere in text_input.rs (I searched `suggestions` in the file; every use is listed in lines 54-1213). Result: every typed character shows the full list, up to 5 rows (paint.rs:244-246).
- Backspace and Delete change the text but skip this block (the condition needs `KeyCode::Char` or a paste, text_input.rs:1009-1011), so a stale list stays open.

D4. CRLF pasted into a single-line input becomes two spaces.
- `paste.content.replace(['\r', '\n'], " ")` replaces each character (R/src/widgets/input/text_input.rs:985). The multi-line branch handles CRLF correctly (line 983). Windows clipboard text `a\r\nb` becomes `a  b`.

D5. Super/Meta+letter inserts the letter (assumption: the terminal reports the modifier, for example through the Kitty keyboard protocol).
- `handle_key_event` checks `ctrl` and `alt` only, then inserts any `KeyCode::Char(c)` (R/src/widgets/input/text_input.rs:1033-1040). `select.rs` checks `meta` (select.rs:698-702). Text input does not.

D6. Select: Space always chooses, so type-ahead cannot match labels with spaces.
- `KeyCode::Enter | Char(' ') | Space` is matched before the char search (R/src/widgets/input/select.rs:647-653, 698-705). A label like "New York" cannot be typed past "New". Typing the space while open chooses the current row.

D7. Select header can overflow by one cell when disabled.
- The header prefix is `🔒 ` (3 cells) when disabled, `▼ `/`▶ `/blanks (2 cells) otherwise (select.rs:240-248). The text budget is a fixed `width - 4` (select.rs:253) and assumes 2. (Assumption: the layout engine measures the emoji as 2 cells like `unicode_width`. The root has `overflow-hidden`, so the effect would be a clipped `]`.)

D8. Slider shows one decimal whatever the step is.
- Value text is `{value:.1}` (R/src/widgets/input/slider.rs:419, 443). Min/max labels use `{:.0}` (slider.rs:241-242, 402, 416, 426, 440). With `step(0.05)` the shown value moves in visible jumps of 0.1 or 0.0, and a `min` of `0.5` shows as `0` (`{:.0}` of 0.5 is "0" in Rust's round-half-even formatting; I did not run it).

D9. Slider keys do not snap and accumulate float error.
- Left/Right add or subtract `props.step` from the current value and only clamp (slider.rs:516-517). PageUp/Down snap (slider.rs:518-525); Home/End set exact ends. With `step 0.1` the value can reach `0.30000000000000004` and go to `on_change` (slider.rs:311-318). The manual says sliders "clamp values to their range and step" (R/manual/input-widgets.md:38).

D10. Slider builder default value differs from the widget builder.
- `builder::slider()` starts at 0.0 (R/src/builder/specialized.rs:329); the widget's `SliderBuilder`/`SliderProps` start at 50.0 (R/src/widgets/input/slider.rs:27, 168). Same crate, two defaults.

D11. Numeric mode accepts more than numbers.
- The filter checks only the inserted text: `is_numeric()` or `.` or `-` (R/src/widgets/input/text_input.rs:627-633). `is_numeric()` is true for non-ASCII digits and fractions such as `½`. `-` and `.` are allowed anywhere and more than once (`1-2.3.4`). A paste with one bad character is rejected whole.

D12. `max_length` rejects a paste that is too long instead of cutting it.
- `insert_text` returns without inserting when the candidate is over the limit (text_input.rs:621-626).

D13. An invalid user regex marks every value invalid, with no message.
- `Regex::new(pattern).ok()` is cached (text_input.rs:492); a `None` regex makes `is_some_and` false (text_input.rs:494-497). The input shows `❌` for every value. Also a custom pattern uses `is_match`, which is not anchored; the built-in email pattern is anchored (text_input.rs:482, 497).

D14. Undo entry pushed before the edit is known to apply.
- `execute_command` pushes the inverse and clears redo (text_input.rs:511-518) and then `apply_command` may return early if the text at `position` differs (text_input.rs:536-539). A failed apply leaves a dead undo entry.

D15. `NamedRadio` label and disabled look differ from `RadioButton`.
- No `(...)` around a disabled label, no `_..._` on hover, `○` for off (named_radio.rs:137-146 vs radio_button.rs:297-317). Same family, two looks. (Listed as C2 in Part 3 for the mark; here for the rest.)

D16. `NamedRadio` group state lives in a shared `AtomicBool`; I could not verify that sibling radios repaint when one is chosen.
- `RadioGroups::select` flips every member's flag (named_radio.rs:46-53) but the handler returns `Consumed` only for the clicked radio (named_radio.rs:187). Whether the App redraws the whole tree on `Consumed` is in code I did not read. Not confirmed as a defect. Worth a test.

D17. Enter is consumed by a single-line text input that has no `on_submit`.
- `KeyCode::Enter` falls through to `EventResult::Consumed` (text_input.rs:1063-1077, 1167) even when `on_submit` is `None`. A parent's default action for Enter is blocked. (Assumption: parents would otherwise get the key.)

Not defects, noted so they are not re-reported:
- `Checkbox` and `Select` write a `state` field they never read (Part 3 D4).
- Left/Right with a selection moves the cursor one step from its old position instead of jumping to the selection edge (text_input.rs:1114-1121). This is a behavior choice.

---

#### Part 5. New gpui-kit 0.7.0 modules

#### 5.1 Questionnaire

What it is. A multi-question form controller: items with single or multiple choices plus an optional free-text input; required flags; validators; Previous/Next/Skip/Submit navigation state; progress; answers snapshot. Logic is in base, the look in component (B7/questionnaire/state.rs 1703 lines, types.rs 698, keyboard.rs 104, control.rs 121; C7/questionnaire/components.rs 1984 lines).
- Keyboard: arrows move between answers and items, Enter confirms a filled answer, Cmd/Ctrl+Enter confirms, bare letter or digit activates a shortcut (`Letters` or `Numbers` mode) (B7/questionnaire/keyboard.rs:36-81; B7/questionnaire/types.rs:247-250).
- Screen reader: form root `Role::Form`, item `Role::Group`, choices group `Role::Group`, progress `Role::ProgressIndicator`, errors `Role::Alert` (C7/questionnaire/components.rs:351, 587, 662, 420, 1191); choices carry description, position and set size (B7/questionnaire/control.rs:77-80).
- Layout: `w_full` on root, item, choices, actions (C7/questionnaire/components.rs:361, 598, 666-675, 1287).

In a terminal. One question at a time: a title row, a radio or checkbox list where each row shows a shortcut key (`1)`, `a)`), an optional text row, a required marker, an error line, and a `Back | Next | Skip | Submit` row. Progress as `2/5`.

Call: skip as a new widget. Reason: reactive-tui already has a wizard dialog (study line 189-191 cites R/src/widgets/dialog/wizard.rs:43-56) and an input dialog with validation (R/src/widgets/dialog/input.rs:100-128); a controller would add a third path. Take two ideas into the existing widgets instead: shortcut keys on choice rows and the radio arrow keys with wrap (RadioButton already has the arrows).

#### 5.2 Time field (`time_field.rs`)

What it is. One segmented editor, `09:30`, `09:30:15` or `09:30 PM` (B7/time_field.rs:33-52). Precision Minute or Second; hour cycle H23 or H12.
- Keys: Up/Down step the selected segment and wrap inside it, Left/Right and Tab/Shift-Tab move between segments, digits typed with a two-digit buffer, `a`/`p` set AM/PM, Backspace/Delete reset the segment (B7/time_field.rs:291-299). The field is one Tab stop; Tab leaves the last segment by design of the bindings (B7/time_field.rs:22-29). (Assumption: Tab leaves at the ends; I did not read `on_next_column`.)
- Screen reader: `Role::TimeInput`, value is the formatted time (B7/time_field.rs:587-588).
- Styled: border, radius, `input_h`, `px_1`, `flex_none` (does not stretch), `theme.selection` on the active segment, `invalid` in `danger` border, tabular digits (C7/time/time_field.rs:39-131).
- The date picker uses it for `time_precision`.

In a terminal. `[09:30 PM]` with the active segment in reverse video, in 10 to 13 cells. It is a fixed-width widget, so no wide-layout problem.

Call: build, small to medium (about 300 lines). It is a pure state machine (SegmentEditor is already testable without a window, B7/time_field.rs:104-289) and fits one text node with a per-segment cursor. It also unblocks time entry in the future date picker (study "time" row: medium). Needs a `TimeInput` role check in the platform accessibility map (I did not check whether it exists).

#### 5.3 Input tokens (`input/token.rs`, `inline_tokens.rs`)

What it is. Atomic chips inside a text field: `@alice`, `#tag`, file references. The cursor treats each as one unit; delete removes it whole; undo keeps it; the app gets click or keyboard activation.

In a terminal. A token is a run of cells with a different style, for example ` @alice ` on a muted background, one grapheme-like unit for the cursor. Width is `unicode_width` of the label; wrapping needs whole-token moves.

Call: skip for now. Reason: the text input stores `lines: Vec<String>` and byte offsets (R/src/widgets/input/text_input.rs:348, 665) with no annotation layer, and undo, selection, paint and mouse hit-testing would all need span awareness (paint.rs:212-238). That is a large change with no stated user. Revisit if a chat composer or mention field is planned.

#### 5.4 Not asked, seen in the diff

- Range decorations (`RangeDecoration`): editor highlight layer. Skip; the code editor is a separate roadmap item.
- `DatePicker` with time editing (`DateTime`): part of the medium "calendar and date picker" row in the study; do not build before the time field.

---

#### Limits of this review

- No code was run and nothing was built. Every "the code does X" is from reading.
- I did not read: R/src/app.rs, the layout engine, the event router, the accessibility platform map, R/src/widgets/dialog/input*, the study sections other than those you named (except the Summary tables), or gpui-kit combobox and rating beyond their diffs.
- I did not verify how `gap-0.5` and `w-30` convert to cells at paint time. I found that `w-` uses `parse_spacing_terminal` (cells) and `gap-` uses `parse_spacing` (a px table) in R/src/layout/css/sizing.rs:14 and R/src/layout/css/spacing.rs:77, and I did not follow the conversion.
- The 0.6.6 tree was compared with `diff` only. I did not check its git history.
- Line numbers for gpui-kit 0.7.0 come from `grep -n` and `sed -n` on the 0.7.0 files. Where I cite a range that I only saw un-numbered in a diff, the range is approximate to within a few lines (marked by a function name).


### Display and layout widgets: Display and layout widgets: reactive-tui vs gpui-kit 0.7.0

Read-only review. I read files and ran ls, grep, sed and diff. I ran no cargo, rustc, tests or builds. I cannot see rendered output, so every statement says what the code does, not how it looks.

Short names used below:

- `R/` = ~/workspace2/reactive-tui/src
- `G7/` = ~/workspace2/gpui-kit-0.7.0/crates/component/src
- `B7/` = ~/workspace2/gpui-kit-0.7.0/crates/base/src
- `G6/`, `B6/` = the same paths in ~/workspace2/gpui-kit-0.6.6
- `study` = ~/workspace2/reactive-tui/docs/widget-study.md (last commit 9340fcc2, 2026-09-26)
- `catalog` = ~/workspace2/reactive-tui/examples/widget_catalog/catalog.rs

Not covered: the charts directory (another reviewer), image, popover, modal, file_explorer and dialog widgets beyond the greps named in the text.

---

#### Part 1. What changed in gpui-kit between 0.6.6 and 0.7.0

Method: `diff -r` and `diff` on the matching paths in crates/component/src and crates/base/src. The file ~/workspace2/gpui-kit-0.7.0/release-notes.md (164 lines) mentions none of the families below. It lists Root overlays, `SettingGroup::variant`, the Plot move to base, and the Plot and Root breaking changes (release-notes.md:1-164). The diffs are the only source for what follows.

##### 1.1 Files with no difference (searched with `diff -rq` and `diff -q`)

- Accordion, base: B7/accordion.rs is identical to B6.
- Collapsible: G7/collapsible.rs and B7/collapsible.rs are identical.
- Breadcrumb: G7/breadcrumb.rs is identical. There is no base part in either version.
- Progress: G7/progress/progress.rs, G7/progress/mod.rs and B7/progress.rs are identical.
- Scroll: everything under G7/scroll/, and B7/scrollbar.rs, B7/scrollable_mask.rs and B7/auto_scroll.rs, are identical.
- Tab: all of G7/tab/ (mod.rs, tab.rs, tab_bar.rs) and B7/tabs.rs are identical.
- Table: G7/table/column.rs, delegate.rs, data_table.rs, table.rs, loading.rs and B7/table.rs are identical.
- Virtual list: G7/virtual_list.rs and B7/virtual_list.rs are identical.
- Also unchanged among the "other" families you listed: badge, tag, skeleton, spinner, separator, description_list, empty, status_bar, stepper. `diff -rq` on component/src did not list them.

##### 1.2 Changes that affect behavior, options, keyboard, roles, defaults or appearance

Accordion (G7/accordion.rs)

1. Disabled state. An item now uses `self.disabled || accordion.disabled` (G7/accordion.rs:123-129). In 0.6.6 the item passed only its own flag (G6 line 128). Before, `Accordion::disabled(true)` blocked only `on_toggle_click` (G7/accordion.rs:147). Now it also disables each item trigger.
2. Closed panels unmount. `.keep_mounted(true)` became `.keep_mounted(progress != 0. || cx.reduce_motion())` (G7/accordion.rs:353-359). A settled closed panel no longer stays in layout and paint. A new test covers it (G7/accordion.rs:430-483).
3. No change to the base accordion, so roles are unchanged: Group root, Button trigger with expanded, Heading with level, Region panel.
4. No key handling was added (`grep -n -i "key_context\|on_action\|KeyBinding" G7/accordion.rs B7/accordion.rs` finds nothing).

Progress (G7/progress/progress_circle.rs)

5. Only the circle changed. It builds arcs with `ArcData::new(&(), 0, 100., 0., TAU)` (G7/progress/progress_circle.rs:102 and 112) and no longer passes two `None` radius overrides. This follows the Plot API change in the release notes. No visible or keyboard change.

Table (G7/table/state.rs)

6. New `TableSelection` enum (None, Row, Column, Cell) at G7/table/state.rs:59-77. New `selection()` (476-491) and `set_selection()` (603-615).
7. `selected_row()`, `selected_col()` and `selected_cell()` now return `Some` only in their own selection mode (497, 541-546, 572). Before, `selected_row()` returned the stored value whatever the mode (G6 line 459).
8. Highlight rules now go through these accessors. A selected cell never leaves its column highlighted (1411-1413) and never highlights its row (2010-2012, 2054-2055, 2159-2160, 2243). Before, the code read the fields and tested `selection_mode` separately.
9. The row-selection "reselect" test no longer requires cell mode (859).
10. The fake trailing cells were replaced by one spacer `div().flex_shrink_0().h_full().w(cols_width)` (2275-2281, 2301-2304). Before, one h_flex element per column.
11. No role change: Row is still set at G7/table/state.rs:2024. The styled table sets no row or column count (see Part 2.6).

Tree (G7/tree.rs, B7/tree.rs)

12. Context menu. The row closure keeps only the item id and looks the entry up again when the menu opens. It returns the menu unchanged if the id at that index has changed or the entry is disabled (G7/tree.rs:84-110). Before, it cloned the entry (with its whole subtree) for each row on each frame and tested `is_disabled` on the stale clone. Behavior change: a menu opened after the tree changed no longer builds for the wrong row.
13. Performance in the base tree: `ancestor_refs` borrows instead of deep-cloning (B7/tree.rs:147-160). `push_root` and a reference-taking `add_entry` flatten without cloning the root subtree (321-346). `reveal_item` searches only root entries (299-305). The collapse path moves entries instead of cloning them (370-373).
14. A new test asserts that revealing an item under a later root keeps the other roots' subtrees (B7/tree.rs:645-677). This suggests a bug fixed in `reveal_item`. I did not check whether 0.6.6 lost subtrees.
15. Keys and roles are unchanged: arrows through the Tree key context, TreeItem with label, selected and expanded (B7/tree.rs:463-467, 548-551).

List (G7/list/)

16. `Role::List` moved from the outer `List` wrapper (G6 line 790) to `ListState::render` (G7/list/list.rs:670). A bare `ListState` is now exposed as a list. A test checks it (884-901).
17. `ListItem::accessibility_label` is new (G7/list/list_item.rs:41, 61, 89-97, 210). Before, rows had no accessible name from their children.
18. `hover` is registered always, and returns no change when the row is active (G7/list/list_item.rs:225-232). This fixes a stale hover style.
19. `secondary_selected` used to suppress the selected fill (G6 lines 234-245). It now draws a 1 px outline in `theme().selection` with the row's corner radius, independent of `selected` (G7/list/list_item.rs:194-197, 257-274). Appearance change for a right-clicked row.

Resizable (G7/resizable.rs, B7/resizable/)

20. In 0.6.6 the component `resizable` was an inline re-export module (G6 lib.rs:67-71). In 0.7.0 it is a real file, G7/resizable.rs (136 lines), with a new `resize_handle_appearance()` (39). It draws a 1 px `theme().border` line and a 3 px indicator (25, 93-120). The indicator uses `muted_foreground` and grows on hover, press and drag (50-53, 108). It takes motion timing from `motion_tokens()` (65).
21. New handle states Idle, Hovered, Pressed and Dragging (B7/resizable/resize_handle.rs:60-91, 76). The old bool `active` is gone.
22. Handle hit-testing uses a hitbox (B7/resizable/resize_handle.rs:214, 364). A handle covered by an occluding element now stays idle. New tests at B7/resizable/mod.rs:633-757.
23. Breaking rename: `placement(Side)` became `inside(HandleEdge)` (B7/resizable/resize_handle.rs:119, 171). `HandleEdge` and `ResizeHandleState` are exported (B7/lib.rs:152-155).
24. Still no role, focus or key handling on the handle: `grep -c -i "role\|focus\|keybinding\|on_action"` returns 0 for B7/resizable/resize_handle.rs and B7/resizable/panel.rs. `PANEL_MIN_SIZE` is still `px(100.)` (B7/resizable/mod.rs:14).

Toolbar (new): see Part 6.

Other changes found while diffing (not in your list, but in the same crate)

25. B7/scroll_bounce.rs (touch overscroll only): a short drag that catches a moving fling no longer starts a new fling (`CATCH_DRAG_SLOP` = 8 px, B7/scroll_bounce.rs:145-153, 253-342). Touch only. Not relevant to a terminal.
26. B7/component_traits.rs adds `Selectable::open` and `is_open`, which default to `selected` (B7/component_traits.rs:2-34).
27. G7/group_box.rs adds `footer()` under the surface, 8 px gap, small muted text (G7/group_box.rs:111-119, 165-187). The content gap under the title is now `gap_2`. The surface still has `p_4` and `gap_4`.
28. G7/pagination.rs caps the ellipsis menu at 100 pages, near the current page (G7/pagination.rs:19-36, 239).
29. G7/sidebar/mod.rs adds `accessibility_label` for the toggle button (G7/sidebar/mod.rs:334-342).
30. B7/dock/tab_group.rs: `is_panel_closable` and constraints (B7/dock/tab_group.rs:281-282, 823-838). This is the dock's tab group, not `TabBar`.

##### 1.3 Study statements that are no longer true for 0.7.0

The study cites G/ and B/ lines in 0.6.6. Every line number in these files moved:

- G/accordion.rs: shifts of +1 from line 123, up to +7 after 352.
- G/table/state.rs: shifts of +19 after line 58, and more further down. The study's "G/table/state.rs:1959-1988" is now about 2010-2040.
- G/tree.rs from line 84.
- G/list/list.rs and list_item.rs.
- B/tree.rs from line 147, by +7 to about +30. The study's "B/tree.rs:230-241, 268-278, 291-309" is now shifted.
- G/lib.rs resizable lines.

Statements that changed meaning:

- Study line 274, accordion: "`keep_mounted` on the base panel, so closed content can be unmounted." In 0.6.6 the styled accordion always passed `keep_mounted(true)`, so it never unmounted. In 0.7.0 it unmounts when settled (G7/accordion.rs:353-359).
- Study line 271: "Disabling the whole accordion at once (G/accordion.rs:56-60, 128)." In 0.6.6 the flag did not reach the item triggers (only `on_toggle_click`). It does in 0.7.0 (G7/accordion.rs:123-129).
- Study "table" section: selection accessors. `selected_row()` and the others no longer return a stale value from another mode (item 7). If the study says a caller can read `selected_row` in any mode, that is no longer true. I did not find that exact sentence.
- Study "tree" section: "Context menu per row" (G/tree.rs:54-62, 92-103). The lookup and the disabled check changed (item 12).
- Study "resizable" section (study lines 1816-1826): "`resizable` is an inline module in the component crate that only re-exports base types (G/lib.rs:67-73, 107-110)". False for 0.7.0. It is now a module file with its own handle look (items 20-23). Also "It has no role, focus or key handling" is still true.
- Study "list" section (study lines 1728-1749): "Rows are Role::ListItem with position, set size and selected state (G/list/list.rs:490-495)". Still true (now G7/list/list.rs:492). It does not say that `ListState` now carries `Role::List`, and that ListItem can now take an accessible name.
- Study "Summary" item 3 (study lines 63-73, "gpui-kit also sets position in set on radios and tabs (B/radio.rs:213-219; B/tabs.rs:75-81)"). The tab half is misleading in both versions. B7/tabs.rs:75-81 defines `set_position`, but the styled TabBar never calls it: `grep -rn "set_position" G7/` finds only G7/radio.rs:208, and the study's own tab section says the same. The same holds for table counts: B7/table.rs:91-110 defines `row_count` and `column_count`, but `grep -rn "row_count(\|column_count(\|aria_row\|aria_col" G7/table/` finds nothing.

##### 1.4 The "tabs keyboard is a TODO" statement (study line 177)

Still true in 0.7.0. B7/tabs.rs is byte-identical to B6/tabs.rs (`diff -q` prints nothing). The comment at B7/tabs.rs:15-20 still says tabs do not take keyboard focus and lists roving focus, arrows, Home/End and Enter/Space as a TODO. G7/tab/ is identical. `grep -n -i "key_context\|on_action\|KeyBinding\|on_key\|track_focus\|focus_handle" G7/tab/*.rs B7/tabs.rs` finds nothing.

What 0.7.0 did add is keyboard roving for the new toolbar (B7/toolbar.rs:114-131). It is a separate primitive. It does not touch tabs.

---

#### Part 2. Family-by-family comparison

Notation: "ours" = reactive-tui today. Cell counts for ours. "px" for gpui-kit. I did not convert px to cells.

Unit background used by several rows below (details in Part 3): utility spacing classes map to numbers as `p-1`=4, `p-4`=16, `gap-0.25`=1, `gap-0.5`=2, and the number is used as cells (R/layout/css/parsers.rs:27-79). `w-N` and `h-N` use the number directly as cells (R/layout/css/sizing.rs:14, 52). So `w-4` is 4 cells wide, but `p-4` is 16 cells of padding.

##### 2.1 Accordion

Layout defaults

- Ours: no padding, border or gap. Sections are `flex flex-col shrink-0 min-w-0` (R/widgets/layout/accordion/live.rs:216-221). The root is `flex flex-col min-w-0` (R/widgets/layout/accordion/live.rs:226-231), so it fills a column parent and has no width of its own. Header row: `flex flex-row shrink-0 min-w-0` (137). It holds a 2-cell focus marker `"▶ "` or `"  "` (116-118), the title, and the glyph `" ▼"` or `" ▲"` right after the title (121-133). The glyph is not pushed to the right edge.
- gpui-kit: `size_full()` and a bordered rounded card by default (G7/accordion.rs:105-111, 37). Header `py_2 px_3`, `gap_3` (298-304). Content `pb_2 px_3` (366-369). Item background `tokens.accordion`, `border_b_1` between items (378-381). Sizes XSmall to Large.
- Wide parent: both fill the parent width. Ours has no maximum. Content is `width_percent(100)` (R/widgets/layout/accordion/live.rs:172-180).

Content that does not fit

- Ours: the body is `overflow_hidden` with an animated height (R/widgets/layout/accordion/live.rs:190-206). Text wrapping depends on the child's own class. I did not check wrapping in the catalog's accordion.
- gpui-kit: GUI text wraps. The chevron is a fixed icon at the end (G7/accordion.rs:322-331).

States and colors

- Ours: focus = the `▶ ` glyph (116). Disabled = literal class `text-gray-500` (113). No hover. Expanded = glyph change. No theme lookup in any of these.
- gpui-kit: open uses `theme().foreground` (306). The chevron uses `muted_foreground` (330). Hover is off by default (`hover()` doc, G7/accordion.rs:227-231). Disabled comes from the base.

Options

- Ours only: modes Single, Multiple, AlwaysOne; glyphs; stagger; `persist_state`; per-section `aria_label`; presets (study lines 260-270).
- gpui-kit only: `bordered`, whole-accordion `disabled` (now working, Part 1 item 1), sizes, title, hover and content styles, `keep_mounted` behavior (now used).

Keyboard and roles

- Ours: Up, Down (wrap), Home, End, Enter, Space (R/widgets/layout/accordion.rs:333-337). Header is `Role::Button` with a label, expanded, click, disabled (R/widgets/layout/accordion/live.rs:139-152). A collapsed body is inert and a hidden Group (207-214).
- gpui-kit: no keys. Roles Heading (level) and Region come from the base (study lines 288-289 cite B/accordion.rs:188-203, 255-270).

##### 2.2 Breadcrumb

Layout defaults

- Ours: each segment has class `px-1` unless `compact` (R/widgets/layout/breadcrumb/live.rs:154-157). `px-1` is 4 cells on each side, so a segment is its label plus 8 cells. The separator is `" / "` (3 cells) unless compact (43-49). The root is `flex flex-col min-w-0 w-full` (416). Default separator `/`, default strategy MiddleEllipsis, `max_width` None (R/widgets/layout/breadcrumb.rs:145-147). Home icon default is the emoji `🏠` and `show_home_icon` is true (R/widgets/layout/breadcrumb.rs:153-154). An emoji may be two cells wide; I did not check how the code measures it.
- gpui-kit: `h_flex().gap_1p5().text_sm()` (G7/breadcrumb.rs:171-176). Separator is a 3.5 icon (145). No item padding.
- Wide parent: ours fills (`w-full`). The trail is left-aligned. Nothing stretches.

Content that does not fit

- Ours: five strategies. MiddleEllipsis (default) keeps first and last and fills from the end (R/widgets/layout/breadcrumb/overflow.rs:8-60). Available width is the smaller of the viewport and `max_width` (R/widgets/layout/breadcrumb/live.rs:52-60). Scroll mode scrolls to the focused segment (92-120).
- gpui-kit: nothing. No wrap, no clip, no menu.

States and colors

- Ours: current = `font-bold`, clickable = `underline`, disabled = `opacity-50` (204-211). No color from the theme. Hover only shows a tooltip line with the literal `bg-gray-800 text-white` (404).
- gpui-kit: `muted_foreground`; last item `foreground`; disabled `muted_foreground` (G7/breadcrumb.rs:101-105). Cursor pointer for links.

Options

- Ours only: separator text, compact, max_width, five strategies, icons, home icon, tooltips, presets.
- gpui-kit only: a click closure per item, `disabled` per item.

Keyboard and roles

- Ours: Left and Right (no wrap), Home, End, Enter, Space (R/widgets/layout/breadcrumb.rs:274-278). Root `Navigation`. Segment `Link` with label or aria_label, description, `aria-current=page` (R/widgets/layout/breadcrumb/live.rs:183-202, 419). Hidden segments are unreachable (study line 316-317).
- gpui-kit: no keys. Item is `Link` when clickable, else `ListItem` (G7/breadcrumb.rs:94-99). The row has no role.

##### 2.3 Progress

Layout defaults

- Ours: height 1 cell (R/widgets/display/progress_bar.rs:235). `width` None gives `width_percent(100)`, so it fills the parent (R/widgets/display/progress_bar/live.rs:171-178). The error line is fixed `width_px(40.0)` (173). An intrinsic-width helper uses 24 cells when `width` is None (180-184). A label adds a row above, and the text adds a row below (160-165).
- gpui-kit: `w_full()`, height 4, 6, 8 or 10 px by size, default Medium 8 px (G7/progress/progress.rs:97-102, 126-129). Pill radius unless the theme has no radius (104-111).
- Wide parent: both fill the parent when width is not set. Ours steps in whole cells (R/widgets/display/progress_bar/live.rs:310), so at 240 columns the step is small.

Content that does not fit: ours has no wrapping. The label and text rows are `whitespace-pre truncate` (231-236 in the helper `text_at`). gpui: not applicable.

States and colors

- Ours: default fill `bg-blue`, track `bg-gray-200` (R/widgets/display/progress_bar.rs:233-234, 308-309). The error line is literal `text-red-500` (R/widgets/display/progress_bar/live.rs:62-66). Both are palette names, not theme roles. Theme roles that exist: primary, secondary, accent, surface, border, foreground, background, error, success, warning, info, text-muted (R/theme/presets.rs, `grep -o '"--color-[a-z0-9-]*"'`).
- gpui-kit: `theme().tokens.progress_bar`, track at 20% opacity (G7/progress/progress.rs:85-86, 137-139). Value transition uses `motion_tokens().duration_normal` (114). No focus or hover.

Options

- Ours only: range, percent/value/custom text, label, vertical, segments, stripes, pulse, `on_complete`, dialog (study lines 949-957).
- gpui-kit only: circle, five sizes, accessible name, theme color token.

Keyboard and roles

- Ours: no keys. `ProgressIndicator`, name = label or the fixed English `"Progress"`, value text, numeric value and min and max when determinate (R/widgets/display/progress_bar/live.rs:186-195).
- gpui-kit: same role, optional name, min 0, max 100 (B7/progress.rs:70-84; the study cites the same lines).

##### 2.4 Scroll

Layout defaults

- Ours: root size is the props `viewport_width` x `viewport_height`, default 80 x 24 (R/widgets/layout/scroll_view.rs:31-32, 147-148), with `max_width_percent(100)` and `max_height_percent(100)` (351-352). It never grows past 80 cells wide unless a class overrides it. The scrollbar takes one column or row from the visible area whenever it is enabled and that axis scrolls, even with no overflow (185-193 subtracts `usize::from(show_scrollbars && scroll_y)`). Bar glyphs `█` and `░` are plain text (R/widgets/layout/scroll_view.rs:455-470). The content is absolutely positioned with `inset_left(-offset)` (262-265).
- gpui-kit: the caller's element is the scroll area (G7/scroll/scrollable.rs:142-165). The scrollbar is an overlay (B7/scrollbar.rs:22-31, width 16 px, thumb 6 px, minimum thumb 48 px). It takes no layout space.
- Wide parent: ours stays 80 cells wide and does not fill (see cleanup C1). gpui fills because the element is the caller's.

Content that does not fit: that is the purpose of both. Ours scrolls by cells. Horizontal scroll only if `scroll_x`. With `scroll_x` false the content is forced to the viewport width (R/widgets/layout/scroll_view.rs:266-271).

States and colors

- Ours: focus flag only. No color from the theme (bar text has no color class).
- gpui-kit: theme scrollbar tokens, modes Scrolling, Hover, Always (B7/scrollbar.rs:46-56).

Options

- Ours only: `scroll_speed`, `smooth_scroll`, fixed viewport size, `show_scrollbars` (R/widgets/layout/scroll_view.rs:24-112).
- gpui-kit only: modes, drag on the thumb, click on the track, `max_fps`, auto-scroll near an edge, bounce.
- Two builders exist with different options. `builder::scroll_view()` (R/builder/specialized.rs:456-540) has `horizontal_scroll`, `vertical_scroll`, `show_scrollbars`, `class`, `contents`. It has no viewport or speed setters, so it always uses the 80 x 24 default (R/builder/specialized.rs:520-528).

Keyboard and roles

- Ours: arrows by one cell, PageUp and PageDown by the viewport height, Home and End; only while the view has focus (R/widgets/layout/scroll_view.rs:378-411). The wheel scrolls, with Shift for horizontal (413-429). At an edge the wheel returns `Ignored` so a parent can scroll (430-433, comment INP-005). Role `ScrollView` (358). The bar has no role or values.
- gpui-kit: no keys, no roles (B7/scrollbar.rs: 0 matches for role, aria, keybinding).

##### 2.5 Tabs

Layout defaults

- Ours: header text is `<focus glyph><icon ><label><badge>`, where the glyph is `▶` when focused, `→` when hovered, else a space (R/widgets/layout/tabs.rs:541-563). Each header is `flex flex-row items-center shrink-0` plus `px-0` (Small), `px-0.5` (Medium, default) or `px-1` (Large) (675-678, 709). Those are 0, 2 and 4 cells each side. Tab bar row is `flex flex-row shrink-0` (729-736). Root is `flex flex-col min-w-0 overflow-hidden` (778-787). Panels are `flex flex-col min-w-0`, the content area `flex-1 min-w-0 overflow-hidden` (751, 763).
- gpui-kit: padding by variant and size, 8, 10, 12 or 16 px (G7/tab/tab.rs:71-77, tab_bar.rs:362-364). Underline gap 16 px. Fixed heights 20 to 44 px (G7/tab/tab.rs:27-40).
- Wide parent: the bar is content-wide and left-aligned. The root fills a column parent.

Content that does not fit

- Ours: clipped by the root's `overflow-hidden` (R/widgets/layout/tabs.rs:781-783). No scroll, no ellipsis, no menu. I searched `scroll|overflow|ellipsis|truncat|max_width` in R/widgets/layout/tabs.rs. It finds only `overflow-hidden` classes (763, 776, 781, 783). Tabs beyond the width cannot be seen. Keyboard focus can still move to them.
- gpui-kit: horizontal scroll (G7/tab/tab_bar.rs:545-546), an overflow menu with `.menu(true)` (default false, 109-113), and per-tab `max_width` with an ellipsis (118-121).

States and colors

- Ours: selected = variant class: `underline`, `border`, `bg-gray-700`, `bg-blue-600 text-white` (R/widgets/layout/tabs.rs:667-673). `border` adds no border, see Part 3.6. Disabled = the label is wrapped in `~...~` (564-566). Focus and hover = the glyphs above. Tooltip uses `bg-gray-800 text-white` (776). All literal, none from the theme.
- gpui-kit: `tab_foreground`, `foreground`, `border`, `tab_bar` and `tab_bar_segmented` tokens (G7/tab/tab.rs:131-149, tab_bar.rs:369-391).

Options

- Ours only: positions Top, Bottom, Left, Right; closable; badges; tooltips; lazy panels; keyboard activation; keyed identity.
- gpui-kit only: overflow menu, `max_width`, scroll handle, prefix and suffix, icon-only tabs, sliding indicator.

Keyboard and roles

- Ours: Left, Up, Right, Down (wrap, skip disabled), Home, End, Enter, Space, Delete or `x` (close, only if the tab is closable, R/widgets/layout/tabs.rs:500-507), keys 1 to 9 (876-891). Roles: `TabList` labelled with the fixed English "Tabs" (737 and the label after it), `Tab` with label, selected, click or disabled (699-706), `TabPanel` labelled "<label> panel" (756), root `Group` (787). No position in set or size of set.
- gpui-kit: no keys (Part 1.4). `Tab` role with optional position (B7/tabs.rs:157-172). The styled bar never sets it.

##### 2.6 Table

Layout defaults

- Ours: root `w-full min-w-0 min-h-0`, plus `border` and 1-cell padding when a border is enabled (R/widgets/display/table/live.rs:539-560). Height = rows + header (+2 with border), capped by `max_height` and 100% of the parent (539-556). Rows are one cell tall. Cells have no padding and no gap between columns (R/widgets/display/table/live.rs:172-199). Text is `whitespace-pre truncate` (194). Column defaults: `TableColumn::new` has width Auto and `min_width` 50 (R/widgets/display/table.rs:653-664). `builder::data_table().column` has width Flex(1.0) and `min_width` 100 (R/builder/widgets/table.rs:52-64). Flex columns start at their minimum, then share the spare width by weight (R/widgets/display/table.rs:302-341).
- gpui-kit: column width 100 px, `min_width` 20 px (G7/table/column.rs:75-80). A table cell has `min_w(100 px * col_span)` (G7/table/table.rs:14, 505). About 10 characters wide, not 100.
- Wide parent (240+ columns): ours fills the width. With the builder's 100-cell minimum, 3 columns need 300 cells and the table scrolls sideways on a 240-column screen (R/widgets/display/table/live.rs:75-76, 92-100 clamp offset_x to content width).

Content that does not fit: ours truncates a cell (194) and scrolls horizontally when `scrollable` (R/widgets/display/table/live.rs:89-100). Header sort arrows ` ↑` and ` ↓` are appended to the title and can be cut off (385-388).

States and colors

- Ours: selected row class from `selected_style`, default literal `bg-blue fg-white` (R/widgets/display/table.rs:166). Header default `font-bold`. Zebra and row styles are caller strings. `hover_row` exists in `TableState` (R/widgets/display/table.rs:231, 251) but nothing sets or reads it (`grep -rn hover_row src` finds only those two lines). Focus = accessibility focus on the selected cell (R/widgets/display/table/live.rs:456-462). No visible focus style.
- gpui-kit: selected, stripe and hover colors from theme (study line 1300; G7/table/state.rs:2010-2040).

Options

- Ours only: Fixed, Percent, Auto, Flex widths; multi-select; per-row and per-cell styles; clickable cells; data-table filters, pages, hidden columns, multi-sort.
- gpui-kit only: pinned and movable columns, group headers, column spans, footer and caption, row, column and cell modes (now with `TableSelection`), context menu, loading and empty views.

Keyboard and roles

- Ours: Down wraps to the first row and Up wraps to the last (R/widgets/display/table.rs:395-398). Home, End, PageUp, PageDown, Enter, Space (multi), Ctrl+A (399-421). Roles: `Table`, `ColumnHeader`, `Row` (selected, disabled), `Cell` (R/widgets/display/table/live.rs:395, 435, 467, 565). No row index or count (`grep -n "row_count\|row_index\|column_count\|set_position" R/widgets/display/table/live.rs table.rs` finds nothing).
- gpui-kit: keys not checked. Roles: `Row` only in the styled table (G7/table/state.rs:2024). Base counts exist but are not called (Part 1.3).

##### 2.7 Tree

Layout defaults

- Ours: indent 2 cells per level (R/widgets/display/tree.rs:423, 519). Expander `"▶ "` or `"▼ "` is 2 cells. A checkbox is 2 cells. With `show_lines`, 3 more cells per nested row (R/widgets/display/tree/live/paint.rs:76-86, 141-145). Rows are absolute one-line elements, `width = max(row width, viewport)` (paint.rs:206-208, 227-234), so they fill. Root `w-full min-w-0 min-h-0` (paint.rs:349-357).
- gpui-kit: the styled tree draws no rows itself. The app draws `ListItem`s (G7/tree.rs:84-92). Indent is the app's.
- Wide parent: fills. Nothing stretches.

Content that does not fit: label text is `whitespace-pre truncate` (paint.rs:101). Horizontal scroll with Shift+Left or Shift+Right (study line 1430). Unwindowed by default: every row is built and painted (paint.rs:266-272 skips only when `virtual_scrolling` is on).

States and colors

- Ours: selected = literal `bg-blue fg-white` (R/widgets/display/tree.rs:431, 527). Tree lines = literal `fg-gray` (434, 530). Match = `underline`. Drop target = `underline font-bold` (paint.rs:116-140). `hover_node` is set on mouse move (R/widgets/display/tree/live.rs:818, 835) but never used when painting (`grep -rn hover_node src` shows the field, the two setters, nothing else). Focus = accessibility focus on the cursor row (paint.rs:243).
- gpui-kit: `ListItem` uses `list_hover`, `list_active` or `accent`, and `selection` (G7/list/list_item.rs:225-274).

Options

- Ours only: multi-select, checkboxes, search, lazy load, drag and drop, lines, horizontal scroll, borders, windowing.
- gpui-kit only: context menu, `reveal_item`, `scroll_to_item`, app-drawn rows.

Keyboard and roles

- Ours: Up, Down, Home, End, PageUp, PageDown, Left, Right (collapse, expand, move), Enter, Space, `+ = - *`, Ctrl+A (study lines 1422-1428, unchanged: R/widgets/display/tree/live.rs:505-602). `Tree` root and `TreeItem` with label, selected, expanded (paint.rs:210-221, 359). No level or position.
- gpui-kit: arrows through actions; Up and Down wrap; Left and Right only collapse and expand (study line 1435). `TreeItem` with label, selected, expanded (B7/tree.rs:463-467).

##### 2.8 Virtual list

- Ours: no widget. Windowing lives in the table (R/widgets/display/table/live.rs:117-127), the tree (paint.rs:266-272), the file explorer and the data table.
- Data table defaults: `row_height` 32, `viewport_height` 400 (R/widgets/display/data_table.rs:352-358). When virtual scroll is on (more than 100 rows, R/widgets/display/data_table.rs:358), the table is capped at ceil(400/32) = 13 rows plus header plus border, whatever the terminal height (R/widgets/display/data_table/live.rs:525-545).
- gpui-kit: `v_virtual_list` and `h_virtual_list`, per-item sizes, no role, no keys. Both files unchanged (Part 1.1).
- Layout, overflow, states, options, roles: not applicable to ours. gpui-kit has none of its own (the caller's rows carry them).

---

#### Part 3. The gap problem

##### 3.1 From class to Taffy value

1. `w-full grid grid-cols-3 gap-0.25` is split by whitespace and applied token by token (R/layout/css/optimizer.rs:106-132, 146-200).
2. `gap-0.25` reaches `apply_gap` through `delegate_to_existing_modules` (R/layout/css/optimizer.rs:194) and `apply_spacing_utilities` (R/layout/css/spacing.rs:113-134). `apply_gap` calls `parse_spacing(token, "gap-")` (R/layout/css/spacing.rs:75-79).
3. `parse_spacing` is a table lookup. `"0.25"` returns `1.0`, `"0.5"` returns `2.0`, `"1"` returns `4.0` (R/layout/css/parsers.rs:27-79, `"0.25" => Some(1.0)` at line 47). Its doc comment says "pixel values" (24-26). A value not in the table returns `None` (parsers.rs:73, "No fallback").
4. `apply_gap` then calls `StyleBuilder::gap_px(1.0, 1.0)` (R/layout/css/spacing.rs:78; R/layout/style.rs:908-913). It stores `LengthPercentage::length(1.0)` in the Taffy `gap`. The layout unit is one terminal cell (a test checks `gap-4` becomes `length(16.0)`, R/layout/css/spacing.rs:196-216). So `gap-0.25` is exactly 1 cell, not a fraction of a cell. Nothing rounds a gap before layout.
5. `grid-cols-3` sets `grid_cols = Some(3)` (R/layout/css/layout.rs:113-115, R/layout/style.rs:1095-1098). `build()` turns it into three tracks made with `TrackSizingFunction::from_fr(1.0)` (R/layout/style.rs:1530-1534). In Taffy 0.9.2 that is `minmax(auto, 1fr)` (taffy-0.9.2/src/style/grid.rs:1120-1123). So a track can grow past its equal share if an item's minimum content is larger. Catalog cards and span cells set `min-w-0`, which removes the automatic minimum (R/layout/css/sizing.rs:85).

##### 3.2 Where Taffy's float result becomes whole cells

- Every layout call uses `TaffyTree::new()` with default settings: R/layout/paint_tree.rs:84 and 124, R/layout/paint_tree/suprtui.rs:365, R/layout/mod.rs:62, R/layout/manager.rs:131. Cargo.toml pins `taffy = "0.9.1"` (line 41). Cargo.lock has 0.9.2. The default is `use_rounding: true` (taffy-0.9.2/src/tree/taffy_tree.rs:79-84). I searched `use_rounding|disable_rounding|enable_rounding` in src and crates and found no override.
- Taffy's rounding pass is `round_layout` (taffy-0.9.2/src/compute/mod.rs:207-247; the same code is in 0.13.0 at compute/mod.rs:230-232). For each node:
  - `location = round(unrounded location relative to the parent)` (line 218-219);
  - `size = round(cumulative_start + size) - round(cumulative_start)` (line 220), where `cumulative_start` is the sum of the unrounded relative positions from the root.
- So the position is rounded relative to the parent, and the size is rounded from the absolute position. They do not use the same rounding origin.
- reactive-tui then reads those rounded values. `collect` uses `layout.size` and `layout.location` from `tree.layout(id)` (R/layout/paint_tree/suprtui.rs:665-685), then adds parent transforms and takes `ceil` of the corners (R/layout/paint_tree/transform.rs:61-97, 98-124). With identity motion and integer positions, this adds no new rounding. The older non-suprtui painter truncates: `location.x.max(0.0) as usize` (R/layout/paint_tree.rs:495-509).
- Grid track widths: Taffy computes `(container - gaps) / n` in floats and rounds only at the end, as above. For 3 equal tracks in width W with gap g, track width is `(W - 2g)/3`.

##### 3.3 Every place where a requested gap can become 0 cells

1. Class value not in the table. `gap-0.75`, `gap-1.25`, `gap-13` and others return `None` (R/layout/css/parsers.rs:73). The token is ignored without error and the gap stays 0 (optimizer.rs:249-251, "Token not recognized").
2. `gap-x-N` and `gap-y-N` overwrite the other axis. They call `gap_px(px, 0.0)` and `gap_px(0.0, px)` (R/layout/css/spacing.rs:82-89), and `gap_px` replaces the whole `gap` (R/layout/style.rs:908-913). `gap-4 gap-x-2` leaves the row gap at 0, and `gap-x-2 gap-y-2` keeps only the last. The same for `space-x-N` and `space-y-N` (R/layout/css/spacing.rs:96-106).
3. `DeclarativeGrid`. `grid_to_node_spec` writes `gap-{gap}` with a cell count as if it were a scale step. `Grid::gap(1)` becomes `gap-1`, which is 4 cells. `gap: 13` becomes `gap-13`, which is not in the table, so 0 (R/layout/renderer.rs:84-93, R/layout/grid.rs:175-176, 207-208). The field docs say "in cells" (188-190). Also `column_gap` and `row_gap` default to 1 but are used only when `gap` is 0 and are turned into 4 cells each.
4. Animation. `gap` in the motion property table is set from a float and clamped at 0 (R/app/motion/property.rs:317). A tween can pass through fractional values. Taffy then rounds the resulting positions. I did not read the tween code.
5. Inline styles. `gap:` with a negative number returns `None` and drops the declaration (R/layout/style/inline.rs:256-262).
6. Rounding (next section). By the derivation below, an integer gap `g` becomes `g`, `g-1` or `g+1` cells between two neighbours only when the grid's cumulative origin is fractional. This can give 0 for `g = 1`.

##### 3.4 Where two neighbouring tracks can be rounded so they touch

Let a container have cumulative unrounded origin `c` (the sum of unrounded relative positions from the root to the grid). Track k starts at relative position `r_k` and ends at `r_k + w`, and the next track starts at `r_{k+1} = r_k + w + g`. Using Taffy's rule (Part 3.2):

- painted start of k+1 = `round(r_{k+1})`
- painted end of k = `round(r_k) + round(c + r_k + w) - round(c + r_k)`
- Let `e_k = round(r_k) - round(c + r_k) + round(c)`. Then the painted gap is `g + e_{k+1} - e_k` when `g` is an integer. Each `e_k` is in {-1, 0, +1}.

So (derivation done by hand, not run):

- If `c` is an integer, `e_k` is 0 for all k, and the painted gap equals `g` exactly (1 for `gap-0.25`).
- If `c` has a fractional part, the painted gap between neighbours can be `g-1`, `g` or `g+1`. For `g = 1` that is 0, 1 or 2 cells, in the same row of tracks. This matches "first and second touch, third keeps its gap" and "gaps of two cells and one cell". It also explains why gap 4 (16 cells) would not show it: 15 to 17 cells looks the same.
- Any node whose absolute x is fractional gives such a `c`. Sources in reactive-tui: `w-1/3` or `width_percent` on an ancestor, `flex-1` sharing with unequal siblings, or an outer grid track that is itself fractional. A grid nested inside a grid track inherits that fractional origin.

Rounding also affects sizes, not only gaps: an item spanning tracks (`col-span-2`) has width `2w + g`, rounded from its own start (R/layout/css/layout.rs:130-131, 137).

##### 3.5 Which explains the reproduction (hypothesis)

Hypothesis, since I cannot run the code: the relative-location vs cumulative-size rounding in `taffy::round_layout` (3.2), applied to a grid whose cumulative origin is fractional (3.4). Reasons:

- It gives exactly the reported pattern: some gaps 0, some 1, some 2 in one grid, at a width not divisible by the column count.
- The catalog's gaps are all 1 cell (`gap-0.25`, next section), the smallest value where +/-1 matters.
- The catalog's cards have no visible border: `border` only sets a gray background if none is set (R/layout/css/effects.rs:51-60). A card is a colored block, so a 0-cell gap fuses two cards.

What I could not confirm, and it weakens the hypothesis:

- Reading the catalog structure at 160 columns (sidebar `w-24`, stage `flex-1`, `p-0.25`, scroll view with a fixed `width_px`, no percent widths), I could not find a fractional origin. Every ancestor width and offset I traced is an integer (catalog.rs:302, 995, 1004-1020). I traced them by hand: sidebar 24, stage 136, stage content 134, scroll content 133, grid at x = 25. If all these are integers, the formula gives gap 1 everywhere and the reproduction is not explained by rounding alone.
- So the reproduction is either (a) caused by a fractional origin I did not find (for example inside the scroll view's measured size or a component that reports fractional layout), or (b) caused by something else in the path. Candidates I did not rule out: the retained-tree update path reusing stale layout (R/layout/paint_tree/suprtui.rs:365-420); `minmax(auto, 1fr)` tracks becoming unequal.
- A direct check, which I could not run: print `tree.layout(id)` (both `location` and `size`, unrounded via `disable_rounding`) for the grid's children at width 160 and compare.

##### 3.6 Gap and padding classes used by the catalog

Card grids

- `card_grid_class(width)` (catalog.rs:240-250) returns `w-full grid grid-cols-N gap-0.25` with N = 1 below 80 columns, 2 below 150, 3 below 200, 4 from 200 up. It is used at catalog.rs:356, 416, 670 and 693.
- Layout page span grid: `w-full grid grid-cols-4 gap-0.25` (catalog.rs:466). Cells use `min-w-0 px-0.25 text-white` (456) and `col-span-4`, `col-span-2`, `col-span-3` (467-476).

Other gap and padding classes (all 1 cell)

- `w-full flex-col gap-0.25` (catalog.rs:331, 462, 861, 954), `flex-col gap-0.25` (509, 847).
- Card: `flex-col min-w-0 shrink-0 border border-gray-700 bg-gray-900 p-0.25 gap-0.25` (318).
- Sidebar: `w-24 shrink-0 h-full flex-col border-r border-gray-700 bg-gray-950 p-0.25` (302). Compact bar `h-3 ... px-0.25` (306).
- Stage: `flex-col flex-1 min-w-0 min-h-0 h-full p-0.25 gap-0.25 bg-black` (995). Header and footer bars `px-0.25` (1045, 1064).

Effects on wide terminals

- Grid columns stop at 4 (from 200 columns). At 512 columns each card is about 120 wide, and cards never use more than 4 columns.
- The catalog picks `0.25` because `gap-1` would be 4 cells. This is the unit problem in Part 3.1 step 3. A developer writing `p-1` or `gap-1` gets 4 cells.
- `border-*` classes have no effect (`border-r`, `border-b`, `border-gray-700` are not layout or paint: `grep -n '"border-r"\|"border-b"' R/layout/css/*.rs` finds nothing, and R/layout/css/effects.rs:81-91 accepts `border-gray*` as a no-op).

---

#### Part 4. Clean-up candidates

Sizes: small under 200 lines, medium, large. Sizes are guesses.

**C1. ScrollView does not fill its parent.** Root size is the fixed props 80 x 24 with a 100% cap (R/widgets/layout/scroll_view.rs:31-32, 347-352). `builder::scroll_view()` has no way to set a size (R/builder/specialized.rs:520-528). The catalog works around it by computing the viewport by hand (catalog.rs:1004-1020). Reference: gpui's scroll area is the caller's element (G7/scroll/scrollable.rs:142-165). Fix: default to fill the parent (100% and `min-h-0`) and make the fixed size optional. Size: small.

**C2. Table column minimums of 50 and 100 cells.** R/widgets/display/table.rs:661 and R/builder/widgets/table.rs:61, enforced at R/widgets/display/table/live.rs:75-76 and 666-667. Reference: gpui column `min_width` 20 px and default 100 px (G7/table/column.rs:75-80), cell min 100 px (G7/table/table.rs:14). Fix: minimum of about 8 to 10 cells, and equal weights so 4 columns fit 240 columns. Size: small.

**C3. Data table row cap of 13.** `row_height` 32 and `viewport_height` 400 are read as cells (R/widgets/display/data_table.rs:352-358). They cap the table at 13 rows, header and border (R/widgets/display/data_table/live.rs:525-545). Reference: gpui virtual list takes item sizes and fills the parent (B7/virtual_list.rs:129-212). Fix: derive the row count from the measured height. Size: small to medium.

**C4. Spacing scale used as cells.** `p-1` is 4 cells, `gap-1` is 4, `gap-4` is 16 (R/layout/css/parsers.rs:27-79, R/layout/css/spacing.rs:196-216). `w-4` is 4 cells (sizing.rs:14). Consumers: breadcrumb segment `px-1` = 4 cells each side (R/widgets/layout/breadcrumb/live.rs:156); tabs `px-1` = 4 (R/widgets/layout/tabs.rs:678); `responsive_grid` has `gap-4` = 16 cells (R/builder/layout.rs:38); `primary_button` `px-4 py-2` = 16 cells and 8 rows (R/builder/layout.rs:105). Reference: gpui uses px on a 4 px scale (G7/tab/tab.rs:71-77; G7/breadcrumb.rs:172). Fix: one terminal spacing scale (1 unit = 1 cell or 2 cells), and check every widget class. Size: medium, because it touches every class string.

**C5. Gap parsing gaps.** (a) values not in the table are dropped silently; (b) `gap-x`, `gap-y` and `space-*` overwrite the other axis (R/layout/css/spacing.rs:82-106; R/layout/style.rs:908-913); (c) `DeclarativeGrid` writes cells as scale steps (R/layout/renderer.rs:84-93). Fix: accept any number and `px`, keep the other axis, and add a `gap_x` and `gap_y` setter. Size: small.

**C6. Rounding of fractional grid tracks.** Part 3.2 to 3.4. Options: give Taffy integer-friendly tracks (compute column widths in the grid setup so gaps stay integral), or post-process children so consecutive tracks keep at least `gap` cells. Size: medium. Needs a test at 160 columns first.

**C7. Colors written as literals instead of theme roles.**
- Progress `bg-blue`, `bg-gray-200`, `text-red-500` (R/widgets/display/progress_bar.rs:233-234, 308-309; live.rs:62-66).
- Table and tree selected `bg-blue fg-white` (R/widgets/display/table.rs:166; R/widgets/display/tree.rs:431, 527).
- Tree lines `fg-gray` (tree.rs:434, 530).
- Tabs `bg-gray-700`, `bg-blue-600 text-white`, `bg-gray-800` (R/widgets/layout/tabs.rs:670-671, 776).
- Accordion `text-gray-500` (R/widgets/layout/accordion/live.rs:113). Breadcrumb tooltip `bg-gray-800 text-white` (R/widgets/layout/breadcrumb/live.rs:404).
- Error lines `text-red-500` in table, tree, data table, modal and popover (R/widgets/display/table/live.rs:372; tree/live/paint.rs:259; data_table/live.rs:685, 701; modal/live/render.rs:41, 248; popover/live/render.rs:21).
- Modal default `bg-white text-black`, backdrop `bg-black/50` (R/widgets/display/modal.rs:184-189). The file explorer selection uses `bg-blue-600 text-white` (R/widgets/display/file_explorer/live/paint.rs:340).
- `builder::card()` is `bg-white ... border-gray-200` (R/builder/layout.rs:85-89).
- Reference: gpui reads `theme().foreground`, `muted_foreground`, `border`, `accent`, `list_active`, tokens (G7/accordion.rs:306, 330; G7/breadcrumb.rs:101-105; G7/tab/tab.rs:131-149; G7/list/list_item.rs:225-274; G7/progress/progress.rs:85-86).
- Theme roles available: `--color-primary`, `secondary`, `accent`, `surface`, `border`, `foreground`, `background`, `error`, `success`, `warning`, `info`, `text-muted` (R/theme/presets.rs). A class such as `bg-primary` resolves through the theme (R/layout/colors.rs:357-361). Size: small per widget, medium in total.

**C8. Defaults that name a class the parser does not know.** `fg-white` and `fg-gray` (Part 5.5). Size: small.

**C9. Tabs that do not fit.** Root clips (R/widgets/layout/tabs.rs:778-787). Reference: horizontal scroll and menu (G7/tab/tab_bar.rs:109-121, 545-546). Fix: scroll the bar to keep the focused tab in view, then a menu, then optional `max_width` with an ellipsis. Size: medium.

**C10. Breadcrumb segments hidden by overflow cannot be reached** (R/widgets/layout/breadcrumb.rs:256-265; the `…` is hidden text, live.rs:322-343). Reference: gpui tab menu (G7/tab/tab_bar.rs:554-585). Size: medium.

**C11. Position, count and level for screen readers.** Tabs (R/widgets/layout/tabs.rs:699-706), tree rows (R/widgets/display/tree/live/paint.rs:210-221), table rows and cells (R/widgets/display/table/live.rs:435, 467). Reference: B7/tabs.rs:75-81 (defined; not called by the styled bar), B7/table.rs:91-110 (defined; not called). Ours would go beyond the reference. Size: small each.

**C12. Fixed English accessible names.** "Tabs" (tabs.rs:738 area), "Progress" (R/widgets/display/progress_bar/live.rs:188), "<label> panel" (R/widgets/layout/tabs.rs:757), "Close <label> tab" (683). Reference: gpui takes an optional label (G7/progress/progress.rs:62; G7/list/list_item.rs:89-97). Fix: an optional label. Size: small.

**C13. Whole-accordion disable, and unmount closed content.** Reference: G7/accordion.rs:123-129, 353-359. Ours: disabled is per section (R/widgets/layout/accordion.rs:65-66). Closed bodies stay in the tree with zero height (R/widgets/layout/accordion/live.rs:190-214). Size: small.

**C14. Accordion and breadcrumb have no visual separation.** No border, padding or gap between sections (R/widgets/layout/accordion/live.rs:216-221). Reference: item padding and a border between items (G7/accordion.rs:298-304, 378-381). The glyph is next to the title, not at the right edge (121-133). Size: small.

**C15. Progress: fractional fill.** Whole cells only (R/widgets/display/progress_bar/live.rs:310). Study line 984 proposes eighth blocks. At 240+ columns the step is already about 0.4% per cell, so the value is low there. Size: small.

**C16. Scrollbar takes a column even with no overflow** (R/widgets/layout/scroll_view.rs:185-193). Reference: gpui overlays the bar (B7/scrollbar.rs:22-31). Fix: subtract only when the content overflows. Size: small.

**C17. Mouse on the scrollbar.** No click on the track and no thumb drag (R/widgets/layout/scroll_view.rs:369-431). Study line 1083. Size: medium.

**C18. Tab labels: `~label~` for disabled.** Tabs draw disabled as literal tildes (R/widgets/layout/tabs.rs:564-566). Use a dim class from the theme. Size: small.

**C19. Modal `Auto` size is half the viewport.** 120 cells at 240 columns, 256 at 512 (R/widgets/display/modal.rs:405-406). I did not check whether the live modal uses this helper. Size: small.

**C20. `col-span-full` spans 12** (R/layout/css/layout.rs:125). In a 4-column grid, Taffy adds implicit tracks. Fix: span to the last line (`grid-column: 1 / -1`). Size: small.

**C21. `grid-cols-auto-fit-N` and `auto-fill-N` set N columns, not a minimum width** (R/layout/style.rs:1205-1217, R/layout/css/layout.rs:169-190). `grid-cols-auto-fit-20` makes 20 columns. Fix: use Taffy `repeat(auto-fit, minmax(N, 1fr))`. Size: small.

**C22. Dead state and options.** `TableState::hover_row` (R/widgets/display/table.rs:231, 251) and `TreeState::hover_node` (R/widgets/display/tree.rs:592; R/widgets/display/tree/live.rs:818, 835) are tracked but not drawn. Either draw a hover style (theme `list_hover` equivalent) or remove them. Size: small.

**C23. Two builders per widget, with different options.** `builder::tabs()` vs `widgets::layout::TabsBuilder` (study line 1235); `builder::scroll_view()` vs `widgets::layout::ScrollViewBuilder`. Size: medium.

---

#### Part 5. Defects found by reading

Study items first.

**5.1 Table sorts every column as text. Still holds.** `content_a.cmp(content_b)` on the cell strings (R/widgets/display/table.rs:343-360, the compare at 356). The data table's multi-column sort compares strings the same way (R/widgets/display/data_table/live.rs:143-159). So "10" sorts before "9".

**5.2 Table column minimums of 50 and 100 cells. Still holds.** `TableColumn::new` has `min_width: 50` (R/widgets/display/table.rs:661). `TableBuilder::column` has `min_width: 100` (R/builder/widgets/table.rs:61). Both are enforced in `LiveTable::widths` (R/widgets/display/table/live.rs:75-76) and in the column drag clamp (666-667). Data table virtual defaults 32 and 400 also still hold (R/widgets/display/data_table.rs:352-358).

**5.3 Tree node selected under a collapsed parent stays hidden. Still holds, in a narrower form.** The selection itself survives: `selected_nodes` is retained against all selectable nodes, not only visible ones (R/widgets/display/tree/live.rs:223, and 160). The cursor does not: if the cursor id is not among the visible rows, it is reset to the first row (R/widgets/display/tree/live.rs:227-233). The row is not painted, and there is no reveal or auto-expand. Setting `selected_node` puts the cursor on it (lines 638, 655, 662), and the next `rebuild` moves it away. Collapsing an ancestor of the cursor row has the same effect (401-410 then `rebuild`).

**5.4 Study-listed items that are fixed or changed.** The study's "always returns Consumed" for the scroll wheel (study line 1084) is no longer true. The wheel returns `Ignored` at an edge (R/widgets/layout/scroll_view.rs:430-433). The study's line references for the other items in the earlier sections match the current code for tabs (R/widgets/layout/tabs.rs:763, 776, 781, 783 are still the `overflow-hidden` lines). I did not re-verify every study line number.

New defects (not in the study)

**5.5 `fg-white` and `fg-gray` are not parsed.** Defaults use them: `selected_style: "bg-blue fg-white"` (R/widgets/display/table.rs:166; R/widgets/display/tree.rs:431, 527) and `line_style: "fg-gray"` (tree.rs:434, 530). I found no parser for the `fg-` prefix: `grep -rn '"fg-\|fg-"' src` finds only these strings, and `grep -rn 'strip_prefix("fg' src` finds nothing. The color modules handle only `text-` and `bg-` (R/layout/css/colors.rs:17, 47). An unknown token is ignored (R/layout/css/optimizer.rs:249-251). So the selected row gets `bg-blue` and keeps its inherited text color, and tree lines get no color. This is a static reading. I did not run it.

**5.6 A `gap-x-N` or `gap-y-N` class clears the other axis.** (Part 3.3, item 2.) R/layout/css/spacing.rs:82-89 and R/layout/style.rs:908-913.

**5.7 `grid-cols-auto-fit-N` makes N columns.** R/layout/style.rs:1205-1210 stores `min_size` in `grid_cols`. The comment at 1206-1207 says it "would need special handling". So a "minimum 20 chars" class asks for 20 tracks of `1fr`.

**5.8 `col-span-full` is `col-span-12`.** R/layout/css/layout.rs:125.

**5.9 `DeclarativeGrid` gap units.** Part 3.3 item 3. R/layout/renderer.rs:84-93. The struct says "in cells" (R/layout/grid.rs:188-190), the class scale multiplies by 4.

**5.10 Accordion glyph meaning.** A collapsed section shows `▼` and an expanded one shows `▲` (R/widgets/layout/accordion/live.rs:121-130, defaults `expand_icon: ▼`, `collapse_icon: ▲` at R/widgets/layout/accordion.rs:171-172). The glyph shows the action, not the state. The focus marker is a separate `▶ ` (116). Not a bug by code, but a reader may take `▼` as "open". Style opinion; listed for awareness.

**5.11 Tabs `Delete` and `x` close a tab only if closable** (R/widgets/layout/tabs.rs:500-507). No defect. Noted because `x` is easy to hit and there is no confirm step. Style opinion.

**5.12 ScrollView subtracts a scrollbar column when there is no overflow** (R/widgets/layout/scroll_view.rs:189-190, `usize::from(props.show_scrollbars && props.scroll_y)`). The content width is 1 cell narrower than the space even when no bar is drawn (bar drawn only if `limits.1 > 0`, line 306). This is a layout defect: unused column.

**5.13 `truncate` on cells and rows.** I did not open `R/layout/css/typography.rs:68`. I did not check whether `truncate` draws an ellipsis or a hard cut.

**5.14 Text nodes get `min-height: 1` only if the class has no `h-`.** `spec.class.contains("h-")` (R/layout/paint_tree.rs:235 in `node_parts`). It matches any class containing `h-`, such as `max-h-4`, `min-h-0` or `sm:h-2`. So a text node with `min-h-0` gets no minimum. I did not test this.

Checked, no defect found

- Data table search and filters read cell text only. No rounding issue.
- Tabs `close` guards `disabled` and `closable` (R/widgets/layout/tabs.rs:500-507).

---

#### Part 6. Toolbar (new in gpui-kit 0.7.0)

What it is (B7/toolbar.rs, 460 lines; G7/toolbar.rs, 379 lines)

- A container that groups controls and owns roving arrow-key focus (B7/toolbar.rs:10-45).
- Left and Right move focus to the previous or next focusable child and wrap at the ends (B7/toolbar.rs:114-131, 87-105). No Up, Down, Home or End. It is capped at 100 hops (14). Disabled turns navigation off (65).
- Role `Toolbar` with horizontal orientation (B7/toolbar.rs:184-185). `ToolbarGroup` is a `Group` with an accessible label (B7/toolbar.rs:260-261).
- It is not a tab stop (`tab_stop(false)`, B7/toolbar.rs:171-173), so Tab enters through the items.
- The styled version adds a size (default Small, Large maps to Medium, G7/toolbar.rs:58, 109, 172, 235) and a `child` method that sizes controls (G7/toolbar.rs:169-200). It draws no background or border; the surrounding surface does (G7/toolbar.rs:140-157 doc).
- Text inputs inside keep their arrow keys (B7/toolbar.rs:40-43).
- The release notes do not mention it.

In a terminal

- A one-row strip of buttons. Tab reaches the first button. Left and Right move between buttons (wrap), and Tab leaves. The screen reader gets a Toolbar role and named groups.
- Ours today has no `Role::Toolbar` use in any widget. The role exists only in the accessibility translation (R/accessibility/platform/translation/node.rs:284). The data table's toolbar is a plain wrapping row of buttons with no role and no arrow-key roving (R/widgets/display/data_table/live.rs:226-261, row at 179-186). I searched `-i toolbar` in src and found only those and one doc comment (R/builder/widgets/breadcrumb.rs:116).

Call: **build, small.** It is one keyed container plus a Left and Right handler over the existing focus order, and it adds a named role for a pattern the data table already draws. Do it after the input-protocol work if the roadmap requires. The study's status bar entry (study line 1909) says it can use the same role for a bar of buttons.

---

#### Appendix. What I did not check

- Rendered output. Every layout statement is from code.
- Whether `truncate` produces an ellipsis, and how the emoji home icon is measured.
- How `ElementBuilder::styles(...)` and `.class(...)` merge when both set a width (I assumed class wins because the catalog relies on `with_class("w-full ...")` on the scroll view, catalog.rs:1020).
- Keyboard handling in gpui-kit's table state (G7/table/state.rs). Not searched.
- Image, file explorer, popover and modal widgets beyond the greps named above.
- The retained-tree update path (R/layout/paint_tree/suprtui.rs:290-420) for stale layout as a source of the gap defect.
- Whether Taffy 0.10 to 0.13 (present in ~/.cargo/registry) change grid gap handling. I compared only `round_layout` between 0.9.2 and 0.13.0 (same rule).


### Overlays, menus and theme: Overlays, menus and theme: reactive-tui against gpui-kit 0.7.0

Read-only review. No file in either repository was changed. No build or test was run. I could not see rendered output, so every statement below says what the code does, not how it looks.

Path shorthand used in this report:

- `R/` = ~/workspace2/reactive-tui/
- `G0/` = ~/workspace2/gpui-kit-0.7.0/crates/component/src/
- `B0/` = ~/workspace2/gpui-kit-0.7.0/crates/base/src/
- `G6/`, `B6/` = the same directories in ~/workspace2/gpui-kit-0.6.6/crates/
- `study` = R/docs/widget-study.md

Method: `diff -rq` and `diff -u` of matching paths between 0.6.6 and 0.7.0, then reading the 0.7.0 files. `release-notes.md` exists only in 0.7.0 (no 0.6.6 copy to diff).

Limits I hit:

- The `text` family diff in `B0/text` is about 6900 lines. I read the Markdown parser hunk, `component/src/text/*.rs` and the file list. I did not read every hunk of `B0/text/*`.
- I did not read gpui-kit tests except where their names state a behavior.
- Study line numbers point into 0.6.6. Files that changed in 0.7.0 have shifted lines. Files with an empty diff (sheet.rs, hover_card.rs, alert.rs, link.rs, highlighter/language_name.rs, theme/registry.rs, themes/*.json) keep their numbers.

---

#### Part 1. What changed in gpui-kit between 0.6.6 and 0.7.0

##### Overall

- `diff -rq G6 G0`: 0.7.0 adds `input/token.rs`, `questionnaire/`, `resizable.rs`, `time/time_field.rs`, `toolbar.rs`. It removes the `plot/` internals (moved to `B0/plot/`, see release-notes.md "Plot moves to gpui-base"). None of these are in the families asked about.
- `diff -rq gpui-kit-0.6.6/themes gpui-kit-0.7.0/themes` prints nothing. The 21 theme files are identical.
- Files with an empty diff: `G0/sheet.rs`, `G0/hover_card.rs`, `G0/alert.rs`, `G0/link.rs`. Their own code did not change. Sheets are still affected by the Root change below.

##### Root (release-notes.md "Root owns window overlays", "Root layers", "Added: gpui_base::Root")

- `Root` now always mounts the dialog, sheet and notification layers itself. `Root::render_dialog_layer`, `render_sheet_layer` and `render_notification_layer` are removed. The application must delete those calls. There is no replacement switch. (release-notes.md, "Root layers" section; `G0/root.rs:129-200` are now private `notification_layer`, `sheet_layer`, `dialog_layer` on a `pub(crate) WindowState` struct at `G0/root.rs:25`.)
- A new `gpui_base::Root` type owns overlay hosting, Tab traversal and copy. `G0/root.rs:434` implements `gpui_base::RootPlugin for WindowState`. The plugin trait is at `B0/root.rs:20-70`. Plugins must be registered before a window is created.
- `gpui_kit::open_window` (`crates/kit/src/lib.rs:144`) wraps content in `Root`. Applications must not return a `Root` from its builder.
- The window methods `open_dialog`, `open_sheet`, `open_sheet_at`, `push_notification` remain (`G0/window_ext.rs:94-170`). They now call `WindowState::update` instead of `Root::update`.
- Removed from `WindowExt`: `selected_text`, `has_text_selection`, `clear_text_selection`, `end_text_selection` (use `gpui_base::TextSelection`). Root fields such as `notification` and `active_dialogs` are no longer public.
- Tab and Shift-Tab bindings and the focus-trap logic moved from `G0/root.rs` (`Tab`, `TabPrev` actions) to `B0/root.rs:13-21` (bindings) and `B0/root.rs:194-260` (`on_action_tab`, `on_action_tab_prev`, using `active_focus_trap`). Behavior is the same code moved.

##### Dialog (`G0/dialog/`, `B0/dialog.rs`, `B0/alert_dialog.rs`)

- `DialogButtonProps` fields are now `Option`s that are unset until a builder sets them. Defaults are applied at render time (`G0/dialog/dialog.rs:31-140`; `show_cancel` is `Option<bool>` at line 51). `Dialog::button_props` and `AlertDialog::button_props` now merge instead of replacing. Before, `AlertDialog::confirm().button_props(...)` could drop the Cancel button, and `on_ok` set earlier was lost. Now call order does not matter. Four tests in `G0/dialog/alert_dialog.rs` (end of file) state this.
- New `AlertDialog` builders: `ok_text`, `ok_variant`, `cancel_text`, `cancel_variant` (`G0/dialog/alert_dialog.rs:209-230`). Defaults documented there: OK text `OK`, OK variant Primary, Cancel text `Cancel`.
- Placement now goes through `gpui_base::Positioner::corner(Anchor::TopLeft, ...)` with `.margin(margin)` (`G0/dialog/dialog.rs:570-600`, diff hunk "gpui_base::Positioner::corner"). Window paddings are added to the offset.
- Height limit changed. 0.6.6: `max_height = view.height - y - margin`. 0.7.0: `max_height = view.height - margin*2 - layer_offset` (`G0/dialog/dialog.rs:584`). The stack step is still 16px per layer (`G0/dialog/dialog.rs:577`). Width is `props.width.min(view.width - margin*2)` (`G0/dialog/dialog.rs:580-582`), default width `px(448.)` (`G0/dialog/dialog.rs:219`). The edge margin is `spacing_tokens().lg` (`G0/dialog/dialog.rs:575`). An overflowing dialog is snapped up to the edge margin (comment at `G0/dialog/dialog.rs:570-574`).
- Base dialog popup and alert popup now call `.occlude()` (`B0/dialog.rs:190,231`; `B0/alert_dialog.rs:16,50`), so a press on the popup does not reach the backdrop and dismiss the dialog. The base host centers its popup by default (test `the_popup_is_centered_by_default`, end of `B0/dialog.rs`). `AlertDialog` now implements `Styled` (`B0/alert_dialog.rs:275`), so a caller can change its host layout.
- Screen-reader roles unchanged: `Role::Dialog` (`B0/dialog.rs:428`), `Role::AlertDialog` (`B0/alert_dialog.rs:199`). Escape still binds `Cancel` (`B0/dialog.rs:91`). I found no diff hunk that adds a label from the title (search: `diff -ru` of both dialog dirs filtered for `aria_label`, `Role::`; only a `Role::Button` test appears).

##### Sheet

- `G0/sheet.rs` has no diff. Study "### sheet" still describes 0.7.0 code, except that the sheet layer is mounted by Root (see Root above).

##### Notification (`G0/notification.rs`)

- Lifecycle clock: a new `autohide_ids` set (`G0/notification.rs:706,718`) and `needs_clock` (`G0/notification.rs:757`) stop the tick timer when only persistent notifications are at rest. Dismiss requests now restart the clock (`start_advancing`).
- `TopCenter` and `BottomCenter` placement changed from `left_0().right_0().mx_auto()` to `left(relative(0.5)).ml(-width/2)` (`G0/notification.rs:1060-1066`).
- The notification layer now uses the Root's full bounds (release-notes.md, "Root owns window overlays").
- Defaults are unchanged: width `px(382.)` (`G0/notification.rs:33`), margins 16px plus title bar on top, `max_items: 10` (`G0/notification.rs:555-568`).

##### Popover (`G0/popover.rs`, `B0/popover.rs`, `B0/popup.rs`, `B0/positioner.rs`)

- New options: `Popover::offset(px)` (`G0/popover.rs:169`; gap from trigger, default 0.25rem) and `Popover::arrow(bool)` (`G0/popover.rs:177`; default false; drawn with a path, sized 0.375rem; `G0/popover.rs:318-360`, `arrow_anchor` at `:419`).
- `Popover` now implements `Styled` on the trigger container (`B0/popover.rs`, hunk "impl Styled for Popover"). The doc says `w_full` or `flex_1` must go there.
- Open state and selected state are now separate: the popover calls `trigger.open(..)`, not `trigger.selected(..)` (`B0/popover.rs` hunk at lines ~212-215; new trait methods `open` and `is_open` at `B0/component_traits.rs:25,32`; `Button` stores `open` apart from `selected` at `G0/button/button.rs:548-556`, shown selected by `shows_selected_style` at `:501`). The `AppMenuBar` uses `.open(is_open)` for the same reason (`G0/menu/app_menu_bar.rs` hunk).
- The popup's state subscription holds a weak handle so the state does not outlive its trigger (`B0/popover.rs` hunk at line ~102).
- `Popup` and `Positioner` gain `offset`, `on_position` and `tracked_corner_position`, so an open popup follows a moving trigger (`B0/popup.rs`, `B0/positioner.rs` hunks).
- Anchoring doc changed: the anchor names the popover's own anchor, "Legacy anchoring clamps without flipping" (`G0/popover.rs` doc on `anchor`).
- Role unchanged: content `Role::Dialog` (`B0/popover.rs:356`). Escape unchanged (`B0/popover.rs:19`). I found no `aria_expanded` in `B0/popover.rs` (search: `grep -n "aria_expanded\|expanded" B0/popover.rs` returned nothing).

##### Hover card, tooltip

- `G0/hover_card.rs`: no diff.
- `G0/tooltip.rs`: only the name change `Root::tooltip_overlay` to `WindowState::tooltip_overlay` (`G0/tooltip.rs:260,283`). `Role::Tooltip` is unchanged (`B0/tooltip.rs:31`).

##### Menu (`G0/menu/`)

- `ContextMenu` now draws its menu through a new `DeferredMenu` element (`G0/menu/context_menu.rs:167`). The menu is drawn only when the click position is inside the element's bounds (`draws_menu`, same file). Tests: a click fires `on_click` once for rows without ids; an opened menu without dismiss releases its entity.
- `DropdownMenu`: the `trigger_style` builder is removed (`G0/menu/dropdown_menu.rs`, hunk lines 24-69). The trigger's own style is used through `Popover`'s new `Styled`.
- `AppMenuBar`: `is_selected` became `is_open` (`G0/menu/app_menu_bar.rs:162`).
- Key bindings unchanged: `enter`, `escape`, `up`, `down`, `left`, `right` on `PopupMenu` (`G0/menu/popup_menu.rs:23-28`); `escape`, `left`, `right` on `AppMenuBar` (`G0/menu/app_menu_bar.rs:19-21`). Roles unchanged: `Role::Menu` at `G0/menu/popup_menu.rs:1466`; item label via `aria_label` at `:1244`.
- I searched `G0/menu` and `G0/command/state.rs` for `typeahead`, `type_ahead`, `mnemonic`, `"home"`, `"end"`, `page_up`. Nothing found. So gpui-kit has no type-ahead and no Home/End in menus.

##### Command (`G0/command/`)

- Row measuring changed: `install_model` keeps measured rows when the new model has the same layout (`same_layout` in `G0/command/item.rs`, `set_options` at `G0/command/state.rs:175`, `measure_rows` at `:643`). Row shortcut hints (`Kbd`) are re-measured on the next frame when the binding changes (`item_binding` at `:774`). `Kbd::keystroke()` and `Icon::same_layout` were added for this.
- Roles unchanged: `Role::ListBox` (`G0/command/state.rs:945`), `Role::ListBoxOption` with `aria_selected` (`:830-831`). Keys unchanged: `escape`, `enter`, `up`, `down` (`:65-68`).

##### Highlighter (`G0/highlighter/`)

- Injection layers (embedded languages such as a Rust fence inside Markdown) are now edited in place with each text edit and reused when the ranges still match (`edit_injection_layers`, `update_edits`, new `combined` flag; hunks in `G0/highlighter/highlighter.rs`). Old layers are moved into `ReusableInjectionLayer`. A test `test_incremental_injection_layers_match_fresh_parse` covers edits inside a Markdown document.
- `parse_input_bytes` fixes parsing at an offset inside a multibyte character.
- `LanguageRegistry::generation()` (`G0/highlighter/registry.rs:554`) is a counter bumped on each registration so caches can retry languages that did not resolve before.
- `input_adapter.rs`: sync parse timeout 2 ms, sync limit 256 KiB, debounce 150 ms are now module constants (`G0/highlighter/input_adapter.rs` top). The logic was split into `finish_update`.
- Language aliases: `G0/highlighter/language_name.rs` has no diff, so the alias table still applies.

##### Text (`G0/text/`, `B0/text/`)

- Markdown parsing: prose such as "$5 and $10" that parses as inline math and is not claimed by a plugin is re-parsed as ordinary Markdown (`B0/text/format/markdown.rs:40`, `flatten_unclaimed_math`).
- New `RangeHighlight`, `RangeHighlightError`, `RenderedText` exports (`G0/text/mod.rs:13-14`; new file `B0/text/range_highlight.rs`). New `TextView::on_reveal` (`G0/text/compat.rs` hunk).
- Code blocks use `shared_code_block_highlighter` instead of `component_code_block_highlighter` (`G0/text/mod.rs:121` and `G0/text/compat.rs` hunk).
- Stream fade changed from 350 ms per chunk to 280 ms per word with a 10 ms stagger between words (`G0/text/compat.rs` constants `STREAM_FADE`, `STREAM_FADE_STAGGER`).
- Heading sizes are now set through `with_heading(level -> StyleRefinement)`.

##### Theme (`G0/theme/`, `B0/theme.rs`)

- New `Theme::update(cx, |theme| ..)` (`G0/theme/mod.rs:268`). It reconciles tokens with edited colors, re-applies the light or dark config when the mode changes, resolves fonts and re-syncs the base theme. `Theme::change` and `set_scrollbar_mode` now go through it (`G0/theme/mod.rs:369`). `Theme::sync_base` remains (`:453`) but its doc no longer tells callers to call it after `global_mut`.
- `default-theme.json` gains a `chart.grid` color: `neutral-200/60` in light (`G0/theme/default-theme.json:28`) and `neutral-800/60` in dark (`:234`).
- `gpui_base::Theme` gains a `plot: PlotTheme` field (`B0/theme.rs`), filled from the motion tokens (`G0/theme/mod.rs:85-107`, `:432`).
- Theme files and folder watching are unchanged: `load_themes_from_str` `G0/theme/registry.rs:151`, `watch_dir` `:98`, `sorted_themes` `:126`, `apply_config` `G0/theme/schema.rs:1064`, `sync_system_appearance` `G0/theme/mod.rs:334`.

##### Button (`G0/button/button.rs`)

- New `open` state stored apart from `selected` (`:548-556`), and `prepare_for_toolbar` returns `ghost().compact()` (`:564`). Used by popover triggers, menu bar and the new toolbar. No key-handling or role change found in the diff (search: filtered `diff -u` for `Role::`, `KeyBinding`, `aria`; none).

##### Study statements that are no longer true for 0.7.0

The study was written against 0.6.6.

1. study "### dialog", Structure: "Open dialogs are a stack in `Root.active_dialogs`" (G/root.rs:297-323). It is now `WindowState.active_dialogs` and private to the crate. And study "### root (support code)": "The top view of each window. It owns the dialog, sheet, notification, tooltip and fallback-menu layers" is now stronger: Root mounts them itself and the render-layer functions no longer exist.
2. study "### dialog", Builder API line for `DialogButtonProps` (`ok_text`, `cancel_text`, `show_cancel`): partly stale. The fields are `Option` and merge; `AlertDialog` also has `ok_text`, `ok_variant`, `cancel_text`, `cancel_variant`. Also the "size is clamped to the window" line (G/dialog/dialog.rs:524-535) now reads `G0/dialog/dialog.rs:575-584` with a different height rule.
3. study "### popover", "Only ours ... An arrow with three styles": gpui-kit now has `arrow(bool)` and `offset(px)` (`G0/popover.rs:169,177`). Ours still has three arrow styles and the 12 positions; gpui-kit has one arrow style and eight anchors.
4. study "### popover", Keyboard: "The trigger is only marked 'selected' (B/popover.rs:212-215)". False now. The trigger is told `open(..)`, not `selected(..)`. Still true that no expanded state is set (search above).
5. study "### notification" placements: `TopCenter` and `BottomCenter` are computed differently now (`G0/notification.rs:1060-1066`). The count of eight anchors is unchanged.
6. study "### theme", "Builder API": "`Theme::global` and `global_mut` followed by `Theme::sync_base`" is now `Theme::update` (`G0/theme/mod.rs:268`). Also the study line "G/theme/mod.rs:261-347" for `Theme::change` is now `:369`.
7. study "### highlighter", "Worth adopting: Keeping the parse tree and reparsing with edits (G/highlighter/highlighter.rs:502-600)": gpui-kit now also keeps and edits the embedded-language trees. The rest of that item holds.
8. Line citations `G/...` for changed files are all shifted by 0.7.0 (see Limits).
9. Study statements about reactive-tui that are stale at the current code (not gpui-kit changes):
   - study "Defects found", first bullet (default backend never turns on mouse): commit c01145ea "Turn on terminal mouse and paste input on the default backend". `R/src/backend/suprtui/output.rs:213` now queues `EnableMouseCapture, EnableBracketedPaste`.
   - study Summary item 4 ("The default backend ... sends no query at all"): `R/src/backend/suprtui.rs:952-977` (`follow_terminal_background`) sets `light_theme()` when the reply to the OSC 11 query has luminance above 0.5 and the app has set no theme. `R/src/backend/suprtui/input_pty.rs:48,628` is the test fixture that counts that query. I did not verify where the query bytes are written for the real backend; I only read the consumer and the fixture.

---

#### Part 2. Family comparison: gpui-kit 0.7.0 against reactive-tui today

Units: gpui-kit uses pixels and rem. reactive-tui uses cells. The utility-class scale in reactive-tui is Tailwind's pixel numbers used as cells: `p-4` is 16 cells, `px-2` is 8 cells, `px-1` is 4 cells (`R/src/layout/css/parsers.rs:27-52`; test `R/src/layout/css/spacing.rs:151-170`). Width and height classes such as `w-1` are cells (`R/src/layout/css/parsers.rs:84-89`). This decides which fixed numbers below are large.

##### 2.1 Dialog and modal

**Who mounts it**

- gpui-kit: Root mounts the layer. The app calls `window.open_dialog(cx, |dialog, ..| ..)` (`G0/window_ext.rs:124`). The app renders nothing for it (release-notes.md "Root layers").
- reactive-tui `Modal`: the app puts the modal element in its own tree, for example `builder::modal().visible(true).build()` inside a card (`R/examples/widget_catalog/catalog.rs:788-793,853-865`). The modal is a `position: absolute` 0x0 owner node whose viewport is that node's inherited `clip` rectangle (`R/src/widgets/display/modal/live/render.rs:49-62,313-327`), not necessarily the whole screen. I did not run it, so whether that clip equals the screen depends on the ancestors.
- reactive-tui typed dialogs (confirmation, input, autocomplete, progress, toast, wizard): the app creates a `DialogEngine`, calls `show_*`, and must render `engine.render()` in its tree (`R/src/widgets/dialog/engine.rs:459-463`; the doc says "Produce a keyed App host"). A second mount shows the text "DialogEngine is already mounted" (`R/src/widgets/dialog/engine/live.rs:66-71`). So yes, the app must render the layer itself.
- The catalog's "Menus and dialogs" page does not use the engine. It builds each dialog element directly (`R/examples/widget_catalog/catalog.rs:756-870`) with `Rect::default()` bounds and inside a card.

**Layout defaults**

| Item | gpui-kit 0.7.0 | reactive-tui |
|---|---|---|
| Width | `px(448.)` default, clamped to view minus 2x margin, optional `max_w` (`G0/dialog/dialog.rs:219,466-468,580-582`) | Auto by default: width is the widest of content, title (+2 with close button), footer, buttons, plus insets (`R/src/widgets/display/modal/live/render.rs:79-96`). Only cap is the viewport (`:119-121`). `ModalSize::Auto` without measured content is half the viewport (`R/src/widgets/display/modal.rs:405-410`). |
| Placement | Centered horizontally; top offset `view.height/10` plus 16px per stacked layer (`G0/dialog/dialog.rs:577-583`) | `ModalPosition::Center` centers both ways (`R/src/widgets/display/modal.rs:433-436`). No margin from the edges in any position (`:432-452`). |
| Stacking of several | Each layer 16px lower (`G0/dialog/dialog.rs:577`) | z-index only: `base_z_index + rank*2`, base 1000 (`R/src/widgets/dialog/engine/live.rs:105-114`; `R/src/widgets/dialog/mod.rs:247`). No offset. |
| Padding | 16px each side, `gap` = max(top padding, 8px) (`G0/dialog/dialog.rs:589,676`) | No content padding by default. Border adds a 1-cell inset when a border is on (`R/src/widgets/display/modal/live/render.rs:63-68,240-242`). Header style `border-b font-bold` (`R/src/widgets/display/modal.rs:186`). Engine dialogs use title style `font-bold border-b border-gray-200 px-1` = 4 cells each side (`R/src/widgets/dialog/mod.rs:262`). |
| Border and radius | 1px border, `radius_lg`, theme colors (`G0/dialog/dialog.rs`, chain in render) | `border border-gray-300 rounded-lg shadow-lg` as literal classes (`R/src/widgets/dialog/mod.rs:261`). |
| Max height | `view.height - 2*margin - layer_offset` (`G0/dialog/dialog.rs:584`) | Height auto from content, min with viewport (`R/src/widgets/display/modal/live/render.rs:97-121`). |
| Wide screen (240+ columns) | Fixed 448px wide; does not stretch. Placed in the middle. | Content-sized. A short message gives a narrow box. A long message (text is one unwrapped run: `R/src/widgets/dialog/confirmation/live.rs:139-144`) gives a box as wide as the message up to the viewport, because no maximum width is set on the auto path. I did not run it at 240 columns, so this is from reading the sizing code only. |

**States drawn, colors**

- gpui-kit draws open dialog, layer index, topmost (`B0/dialog.rs`); colors from `cx.theme().tokens.background`, `.border`, `.radius_lg` (`G0/dialog/dialog.rs:670-676`).
- reactive-tui `Modal` draws hidden, showing, visible, hiding, drag, resize, focused button (study "### dialog", States). Colors are literal class strings, not theme lookups: `bg-white text-black shadow-lg`, backdrop `bg-black/50`, close button `text-gray-500 hover:text-gray-700` (`R/src/widgets/display/modal.rs:184-189`). Engine dialogs use `DialogTheme` strings (`R/src/widgets/dialog/mod.rs:256-282`): `bg-white`, buttons `bg-blue-500 text-white px-1 py-0 rounded`, and so on. Neither reads `Theme::active()`.

**Options each lacks**

- Only gpui-kit: `trigger()` element, controlled `DialogHandle`, `DialogChangeReason`, `AlertDialog` preset with icon and description, merge semantics for button props, `ok_variant`/`cancel_variant` (`G0/dialog/alert_dialog.rs:209-230`).
- Only reactive-tui: confirmation/input/autocomplete/progress/wizard dialogs, drag and resize, anchored positions (`R/src/widgets/dialog/frame.rs:80-137`), async completion, live update. (From study; not re-verified except frame.rs and engine.rs.)

**Keyboard and screen reader**

- gpui-kit: Escape and Enter bound to Cancel and Confirm in the Dialog context (`B0/dialog.rs:91`); Tab loop through `B0/root.rs:194-260`; `Role::Dialog` (`B0/dialog.rs:428`), `Role::AlertDialog` (`B0/alert_dialog.rs:199`).
- reactive-tui: Escape closes when `closable && escape_closable && keyboard_navigation`; the handler consumes every Escape press while visible even when it does not close (`R/src/widgets/display/modal/live/events.rs:10-34`). Focus trap via `FocusProps::modal()` (`R/src/widgets/display/modal/live/render.rs:282-287`). Dialog role and label from the title (`:273-280`). Close button labeled "Close" (`:366-368`). Confirmation dialog uses `Role::Dialog`, not `AlertDialog` (`R/src/widgets/dialog/confirmation/live.rs:191-195`). Still open from the study.

##### 2.2 Notification and toast

**Who mounts it**

- gpui-kit: `window.push_notification(note, cx)` (`G0/window_ext.rs:163`). Root mounts the layer. App renders nothing.
- reactive-tui: two ways. (a) `builder::toast()...build()` returns an element the app must place in its tree (`R/src/builder/widgets/dialog.rs:43,359-395`). (b) `DialogEngine::show_toast` and the app renders `engine.render()` (`R/src/widgets/dialog/engine.rs:584`). Either way the application renders the layer.

**Layout defaults**

| Item | gpui-kit | reactive-tui |
|---|---|---|
| Width | fixed `px(382.)`, settable (`G0/notification.rs:33,566`) | Content-sized. Toast is a non-modal `Modal` with `ModalSize::Auto` (`R/src/widgets/dialog/toast/live.rs:156-177`); message is one `Element::text` (`:158-161`). Width follows message length up to the viewport. |
| Placement | eight anchors; margins 16px, top adds the title bar (`G0/notification.rs:555-568,1058-1068`) | six positions (`R/src/widgets/dialog/toast.rs:75-90`, mapped at `R/src/widgets/dialog/toast/live.rs:139-146`). Corner positions use offset 0 (`R/src/widgets/display/modal.rs:447-452`): no gap to the edge. |
| Stacking | Several toasts stack; collapsed layers expand; `max_items: 10` (`G0/notification.rs:565`; `B0/toast.rs:22-55`) | None. Engine toasts get `Rect::default()` (`R/src/widgets/dialog/engine/content.rs:130,148`); empty bounds leave position unchanged (`R/src/widgets/dialog/frame.rs:26-33`). Two toasts with the same position use the same corner. |
| Wide screen | Fixed 382px in the corner | Corner-anchored, content-sized. Long message: see 2.1. |
| Auto-dismiss | 5s fixed | `Duration` or none, default 3 s (`R/src/widgets/dialog/toast.rs:188-199`); timer starts after the first presented frame (`R/src/widgets/dialog/toast/live.rs:179-187`). |

**States and colors**

- gpui-kit: Starting, Present, Ending; timers pause on hover or focus (`B0/toast.rs`); type colors from theme.
- reactive-tui: info/success/warning/error and `Custom`. Colors are literals: `bg-green-700 text-white`, `bg-red-700 text-white`, `bg-yellow-700 text-white`, `bg-blue-700 text-white` (`R/src/widgets/dialog/toast/live.rs:147-152`). `Custom` falls into the blue branch (`:151`).

**Options**

- Only gpui-kit: title, icon, custom content, action button, on_click, replace by id, remove by type, clear all, OS delivery (study "### notification").
- Only reactive-tui: `duration(None)` (persistent), six string positions in the builder. Builder position is a string; an unknown string returns the text "Invalid toast position" (`R/src/builder/widgets/dialog.rs:363-372`).

**Keyboard and screen reader**

- gpui-kit: `Role::Alert` for every toast (`B0/toast.rs:656`); the list is a Tab stop; no key bindings (study).
- reactive-tui: `Status` for info/success, `Alert` for warning/error, live-region classes on the text (`R/src/widgets/dialog/toast/live.rs:147-161`); no focus (`:163-164`); Escape and close button only when closable (`:166-167`).

##### 2.3 Popover

**Who mounts it**

- gpui-kit: the `Popover` is placed in the parent as a trigger; the content is a deferred overlay through `Popup` and `Positioner` (`B0/popup.rs`). No layer to render.
- reactive-tui: `popover().trigger(..).content(..).build()` returns an element that holds the trigger and the popup in one container (`R/src/widgets/display/popover/live/render.rs:283-291`). The app places it. No separate layer. Bounds come from the container's `clip` (`:78-86`).

**Layout defaults**

| Item | gpui-kit | reactive-tui |
|---|---|---|
| Gap to trigger | `offset` default 0.25rem (`G0/popover.rs:169`, `:330`) | `offset: (0, 8)` (`R/src/widgets/display/popover.rs:241`). `calculate_rect_for_position` adds `offset_y` as rows (`R/src/widgets/display/popover.rs:372-406`). So the default gap below or above the trigger is 8 rows. |
| Arrow | off by default; 0.375rem deep (`G0/popover.rs:177,318-322`) | on by default: `PopoverArrow { enabled: true, size: 8, .. }` (`R/src/widgets/display/popover.rs:97-105`). The arrow is a filled triangle of `size` rows: `for depth in 0..size` puts `2*depth+1` cells on each row (`R/src/widgets/display/popover/live/render.rs:357-380`). Size 8 = up to 8 rows deep and 15 cells wide. |
| Padding | `p_3` when appearance is on (`G0/popover.rs:341`) | none: the body node has no padding, border or background (`R/src/widgets/display/popover/live/render.rs:167-175,196-200`). Style comes from the content element. |
| Width and height | `popover_style` (theme) | max = parent clip unless `Ignore` (`:158-163`), optional min/max props (`R/src/widgets/display/popover.rs:252-255`). |
| Flip | corner anchoring, no flip ("Legacy anchoring clamps without flipping", `G0/popover.rs` doc) | `BoundaryBehavior::Flip` default (`R/src/widgets/display/popover.rs:242`). |
| Wide screen | Same 8 anchors in any window | Follows the trigger; the 8-row gap and 8-row arrow do not scale. |

**Colors**: gpui-kit uses `cx.theme().popover` and a ring color (`G0/popover.rs:330-335`). reactive-tui: body has no color of its own; the outside-click shield uses `bg-black/30` when `backdrop_filter` is on (`R/src/widgets/display/popover/live/render.rs:148-151`); the arrow cells carry no color style (`:388-398`), so they take the inherited text color. Error text `text-red-500` (`:21`).

**Options**

- Only gpui-kit: mouse button choice, unstyled mode, focus handle to receive focus, `open` state for the trigger.
- Only reactive-tui: 12 positions, hover/focus/manual triggers with delays, three arrow styles, four boundary behaviors, min/max size, backdrop, four animations, `on_position_change` (study "### popover"; props at `R/src/widgets/display/popover.rs:230-260`).

**Keyboard and screen reader**

- gpui-kit: Enter/Space toggle from trigger; Escape closes (`B0/popover.rs:19`); focus moves in on open and back on close; content `Role::Dialog` (`B0/popover.rs:356`).
- reactive-tui: Escape closes by default (`R/src/widgets/display/popover.rs:243`); `focus_trap` and `auto_focus` are both false by default (`:248-249`), so focus does not move into the popover on open. Trigger gets `aria-expanded-true/false` classes (`R/src/widgets/display/popover/live/render.rs:23-27`). Body role is `Dialog` only with `focus_trap`, otherwise `Group` (`:209-215`).

##### 2.4 Menu

**Who mounts it**

- gpui-kit: `PopupMenu` is built with `PopupMenu::build`; `context_menu` and `dropdown_menu` wrap elements; drawn as a deferred overlay (`G0/menu/context_menu.rs:167-270`). No layer to render.
- reactive-tui: `menubar()`, `context_menu()`, `popup_menu()` return elements the app places (`R/examples/widget_catalog/catalog.rs:777-782`). Panels are `position_absolute` nodes inside that element, with z-index `1000 + depth*2` (`R/src/widgets/menu/panels.rs:168-172`). A popup menu's z-index of 1000 equals the modal default (`R/src/widgets/display/modal.rs:191`), the popover default (`R/src/widgets/display/popover.rs:250`) and the engine base z-index (`R/src/widgets/dialog/mod.rs:247`). I did not check whether that causes a wrong stacking order when a menu opens inside a modal.

**Layout defaults**

- reactive-tui `MenuStyle::default()`: `min_width: 10`, `max_width: Some(50)`, `padding: 1`, border on, shadow on (`R/src/widgets/menu/style.rs:41-53`). Applied in `R/src/widgets/menu/panels.rs:162-190`. So a menu is capped at 50 cells wide unless the caller changes it, on any terminal width. Menu height default `max_visible_items: 10` (`R/src/widgets/menu/popup.rs:105`; `R/src/widgets/menu/context.rs:45`).
- The shadow is a second node offset by 1 cell with `bg_rgba(0,0,0,0.4)` (`R/src/widgets/menu/panels.rs:262-274`).
- gpui-kit sets height with `max_h` and `scrollable` (study "### menu"); item sizes come from theme spacing tokens.

**States and colors**

- reactive-tui default menu colors are literal classes: `bg-gray-800 text-white`, selected `bg-blue-600 text-white font-bold`, focused `bg-cyan-500 text-black font-bold`, disabled `text-gray-500`, separator `text-gray-400`, shortcut `text-yellow-400`, icon `text-green-400`, border `border border-gray-600` (`R/src/widgets/menu/style.rs:41-53`). Three preset variants also use literals (`:227-259`).
- The builder path has a different default. `convert_menu_style` uses `white` background and `black` text when the caller sets none, and `blue-500` for the selected row (`R/src/builder/widgets/menu.rs:913-935`). So `menubar().build()` and `MenuBarProps::default()` do not agree on colors. The builder's own `MenuStyle` struct has string fields for colors, padding and margin (`R/src/builder/widgets/menu.rs:196-212`).
- Dialog-menu backdrop is `bg_rgba(0,0,0,0.3)` at z-index 998 (`R/src/widgets/menu/dialog_live.rs:395-406`).
- gpui-kit draws item, hover, selected, disabled, checked with theme colors.

**Options**

- Only gpui-kit: label items, link items, custom-element items, shortcut hint read from the real key binding, check side.
- Only reactive-tui: radio items, separator styles, visible flag, per-item description, long-press context menu, dialog menus, wheel selection (study "### menu"). `MenuShortcut` keeps `display` and `keys` as two separate strings (`R/src/widgets/menu/item.rs:3-9`).

**Keyboard and screen reader**

- gpui-kit: Enter, Escape, Up, Down, Left, Right (`G0/menu/popup_menu.rs:23-28`); `Role::Menu` (`:1466`); item name from text (`:1244`). No type-ahead, no Home/End (search above).
- reactive-tui: Up, Down with wrap, PageUp, PageDown, Home, End, Right into submenu, Left or Escape back, Enter or Space activate, item shortcuts (`R/src/widgets/menu/popup_live.rs:289-333`). No type-ahead: the only `Char` match is the space character (`:325`). `Role::Menu` (`:255-256`), items as MenuItem/MenuItemCheckBox/MenuItemRadio with toggled and expanded (study; `R/src/widgets/menu/view.rs:164-196`). Reactive-tui's keyboard handling is fuller than gpui-kit's here.

##### 2.5 Theme

**Color roles**

- gpui-kit: `ColorTokens` has background, foreground, surface, surface_foreground, primary, primary_foreground, secondary, secondary_foreground, muted, muted_foreground, accent, accent_foreground, destructive, destructive_foreground, border, input, ring, selection (`B0/theme_tokens.rs:19-45`). `default-theme.json` has 232 keyed entries for both modes (`G0/theme/default-theme.json`; count from `grep -c '^        "'`), including component keys and `chart.grid` (new, lines 28 and 234).
- reactive-tui: `--color-primary`, `secondary`, `accent`, `background`, `surface`, `foreground`, `text-muted`, `border`, `success`, `warning`, `error`, `info`, `chart-1` to `chart-5`, `chart-bullish`, `chart-bearish`, and `--spacing-xs` to `2xl` (`R/src/theme/presets.rs:5-43`). No foreground-on-primary, no selection role, no hover role, no input or ring. Unchanged since the study.

**Light and dark**

- gpui-kit: a light and a dark config; `Theme::change(mode)` (`G0/theme/mod.rs:369`); `sync_system_appearance` reads the window appearance (`:334-340`); `Theme::update` re-applies config on a mode change (`:268`).
- reactive-tui: `Theme` has no mode field (`R/src/theme/mod.rs:46-54`). Five presets: dark, light, high contrast, Solarized Dark, Gruvbox Dark (`R/src/theme/presets.rs:5,45,85,125,165`). The default backend switches to `light_theme()` once, at startup, when the terminal background is light and the app has set no theme (`R/src/backend/suprtui.rs:952-977`). There is no switch back and no dark-variant lookup.

**Loading from a file**

- gpui-kit: `load_themes_from_str` (`G0/theme/registry.rs:151`), `watch_dir` (`:98`), 21 shipped files in `~/workspace2/gpui-kit-0.7.0/themes/`.
- reactive-tui: code only. Search `grep -rn "serde\|Deserialize\|from_str\|read_to_string\|from_file" R/src/theme` returned nothing. `Theme::new(name).with_variables(..).extend(base)` (`R/src/theme/mod.rs:57-76`).
- Syntax themes do load from files: `SYNTAX_RESOURCES.load_theme_from_file` (`R/src/syntax/resources.rs`, per study), but `Theme::set_active` does not change the syntax theme (default `onedark`, `R/src/syntax/resources.rs:14-15`).

**Who reads the theme**

- In `R/src/widgets`, I found no class that names a theme role. Search: `grep -rEc '(bg|text|border|ring)-(primary|secondary|accent|surface|foreground|background|muted|success|warning|error|info)\b|var\(--' R/src/widgets` gave zero for every file. The only widget code that reads the active theme is charts (`R/src/widgets/display/charts/live/canvas.rs:221`) and image cells (`R/src/widgets/display/image/live/cells.rs:60-61,184-185`). Theme classes are supported by `Theme::apply_classes` (`R/src/theme/mod.rs:146-153`) and utility classes such as `text-primary` resolve (test at `R/src/theme/mod.rs:184-191`), but no overlay, menu or dialog uses them.
- gpui-kit widgets read `cx.theme()` roles (for example `G0/dialog/dialog.rs:670-676`, `G0/popover.rs:330-335`).

##### 2.6 Highlighter and Markdown code blocks

- gpui-kit: tree-sitter with a per-language registry, language aliases (`G0/highlighter/language_name.rs:5-37`, unchanged), embedded languages with edit reuse (Part 1), sync parse 2 ms up to 256 KiB, then a 150 ms debounced background parse (`G0/highlighter/input_adapter.rs` constants). Plain text gets a default style so the caller's color applies (study, `G/highlighter/highlighter.rs:1096-1099`; the function is unchanged in structure, I did not re-read it).
- reactive-tui: Lumis wrapper. Exact, case-sensitive language name match (`R/src/syntax/resources.rs:101-108`). 15 grammars. 1 MiB input limit (study; `R/src/syntax/highlighter.rs:16-17`). Runs carry a transparent background (`:286-291`). The plain fallback is black (Part 4).
- Colors: gpui-kit `HighlightTheme` is part of each app theme config and follows light or dark (study; `G0/theme/schema.rs`). reactive-tui keeps a separate syntax theme, default `onedark`, not tied to `Theme` mode.
- Keyboard and roles: none on either side (study).

---

#### Part 3. Clean-up candidates for reactive-tui

Sizes: small under 200 lines, medium, large. These are guesses from reading, not estimates from a trial.

##### A. Fixed numbers that do not fit wide layouts

| # | What | Reference | reactive-tui code | Size |
|---|---|---|---|---|
| A1 | Popover default gap is 8 rows, default arrow 8 rows deep | gpui-kit gap 0.25rem, arrow 0.375rem (`G0/popover.rs:169,318-322`) | `R/src/widgets/display/popover.rs:241` (`offset: (0, 8)`), `:97-105` (arrow size 8), `R/src/widgets/display/popover/live/render.rs:357-380` | small: change defaults to 1 row gap and a 1 to 2 row arrow; check tests that assert the numbers |
| A2 | `DialogThemes::light/dark/...` presets use `px-4 py-2` buttons and `p-4` titles: 16 cells by 8 rows per button, 16 cells on each side of the title | gpui-kit padding 16px (`G0/dialog/dialog.rs:589`) | `R/src/widgets/dialog/dialog_types.rs:199-293` (exported at `R/src/widgets/mod.rs:30`) | small |
| A3 | Auto-width dialogs and toasts have no maximum width; long single-line messages grow to the viewport | `width.min(view - 2*margin)` and `max_w` (`G0/dialog/dialog.rs:219,466,580-582`); toast `width` 382px (`G0/notification.rs:33`) | `R/src/widgets/display/modal/live/render.rs:79-96,119-121`; `R/src/widgets/dialog/confirmation/live.rs:139-144`; `R/src/widgets/dialog/toast/live.rs:158-161` | small: add a default `max_width` (in cells or as a share of the viewport) and wrap the message text |
| A4 | Menu width cap of 50 cells and minimum 10 | menu width follows content, `max_h`/scrollable (study) | `R/src/widgets/menu/style.rs:52-53`; `R/src/widgets/menu/panels.rs:162-177` | small: make the cap a share of the viewport or none |
| A5 | Corner-anchored modals and toasts touch the screen edge (no margin) | notification margins 16px (`G0/notification.rs:555-568`); dialog edge margin `spacing_tokens().lg` (`G0/dialog/dialog.rs:575`) | `R/src/widgets/display/modal.rs:447-452` (`TopLeft` = (0,0) etc.) | small |
| A6 | Popover and modal `z_index` fixed at 1000, engine base 1000, menu panels `1000 + depth*2`, toast default 2000 | Root layers in one fixed order (`B0/root.rs` plugin order, comment "later plugins appear above earlier ones") | `R/src/widgets/display/modal.rs:191`; `R/src/widgets/display/popover.rs:250`; `R/src/widgets/dialog/mod.rs:247`; `R/src/widgets/menu/panels.rs:171`; `R/src/widgets/dialog/toast/live.rs:171` (overridden for engine toasts by `R/src/widgets/dialog/frame.rs:23`) | medium: define one z-index ladder; unchecked whether a menu opened inside a modal is hidden |
| A7 | `DialogBounds` default `min_size` 200x100, `margin` 20 each side; typed dialogs set 300x150, 400x200, `max_size` 600x400 or 800x600. These read as pixels | gpui-kit sizes are pixels on a pixel screen | `R/src/widgets/dialog/dialog_component.rs:439-462`; `R/src/widgets/dialog/confirmation.rs:263-270`; `R/src/widgets/dialog/autocomplete.rs:209-215`; `R/src/widgets/dialog/input.rs:280-286` | small, but see C1: they have no effect today |
| A8 | Stacked engine dialogs are not offset | one layer = 16px lower (`G0/dialog/dialog.rs:577`) | `R/src/widgets/dialog/engine/live.rs:105-114` | small |
| A9 | `DialogUtils::calculate_size` assumes 60 characters per line and a height `lines*2 + 8` capped at 40 | none | `R/src/widgets/dialog/dialog_types.rs:116-125` (no non-test caller found: `grep -rn calculate_size R/src`) | small: delete or replace |

##### B. Colors written as literals (no theme lookup)

Search: `grep -rEc '(bg|text|border|ring|fg)-(white|black|<palette name>(-NNN)?)\b' R/src/widgets --include=*.rs`, non-zero files only. The counts include test code and tests' expected strings, so treat them as upper bounds. Zero files use a theme role class (Part 2.5).

| File | Count | Lines and notes |
|---|---|---|
| `R/src/widgets/menu/style.rs` | 44 | defaults `:41-53`, presets `:227-259`, tests from `:275` |
| `R/src/widgets/dialog/dialog_types.rs` | 26 | `DialogThemes` presets `:196-293` |
| `R/src/widgets/dialog/mod.rs` | 7 | `DialogTheme::default` `:259-275` |
| `R/src/widgets/input/text_input/paint.rs` | 6 | `:275,300,304,306,324,354` |
| `R/src/widgets/display/progress_bar.rs` | 6 | `:233-234,308-309`, tests `:775-776` |
| `R/src/widgets/dialog/toast/live.rs` | 5 | `:135,148-151` |
| `R/src/widgets/display/tree.rs` | 4 | `:431,434,527,530` |
| `R/src/widgets/layout/tabs.rs` | 3 | `:670-671,776` |
| `R/src/widgets/display/modal.rs` | 3 | `:184-189` |
| `R/src/widgets/display/popover/live/render.rs` | 2 | `:21,150` |
| `R/src/widgets/display/modal/live/render.rs` | 2 | `:41,248` |
| `R/src/widgets/display/file_explorer/live/paint.rs` | 2 | `:340,358` |
| `R/src/widgets/display/data_table/live.rs` | 2 | `:685,701` |
| `R/src/widgets/dialog/input/live.rs` | 2 | `:342,349` |
| `R/src/widgets/dialog/autocomplete/live.rs` | 2 | `:408,441` |
| `R/src/widgets/layout/breadcrumb/live.rs` | 1 | `:404` |
| `R/src/widgets/layout/accordion/live.rs` | 1 | `:113` |
| `R/src/widgets/display/tree/live/paint.rs` | 1 | `:259` |
| `R/src/widgets/display/table/live.rs` | 1 | `:372` |
| `R/src/widgets/display/table.rs` | 1 | `:166` (`bg-blue fg-white`, the selected row) |
| `R/src/widgets/display/progress_bar/live.rs` | 1 | `:65` |
| `R/src/widgets/display/image/live/blocks.rs` | 1 | `:188` |
| `R/src/widgets/display/data_table/filters.rs` | 1 | `:144` |
| `R/src/widgets/dialog/wizard/live.rs` | 1 | `:223` |

Color-value literals (`Rgba::new`, `Rgba::black()`, `bg_rgba`, `rgb(`), same tool, count per file, non-zero only: `display/image/decoded.rs` 5, `dialog/dialog_buffer.rs` 5, `terminal/paint.rs` 4, `display/image/live/cells.rs` 3 (plus 2 `#` hex hits), `display/image/protocol_renderer.rs` 2, `display/charts/live/canvas/sankey.rs` 2, and 1 each in `menu/panels.rs` (`:270`), `menu/dialog_live.rs` (`:405`), `display/progress_bar/live.rs`, `display/image/sixel_renderer.rs`, `display/image/live/blocks.rs`, `display/image/image_processor.rs`, `display/image/external_renderer.rs`, `display/charts/mask.rs`. Most of the image and terminal ones are pixel data, not styling. The overlay ones are `menu/panels.rs:270` (shadow 0.4 black), `menu/dialog_live.rs:405` (backdrop 0.3 black), `dialog/dialog_buffer.rs`.

Also, `R/src/syntax/highlighter.rs` has 7 `Rgba::black()` uses (`:94,140,177,213,257,327,342,357`), outside `src/widgets`.

| # | Candidate | Reference | reactive-tui | Size |
|---|---|---|---|---|
| B1 | Replace overlay literals with theme roles: dialog bg, border, backdrop, toast types, menu states, error text `text-red-500`. Needs class names or variables that resolve through `Theme::active()` | `cx.theme().tokens.background`, `.border`, `.popover` (`G0/dialog/dialog.rs:670-676`, `G0/popover.rs:330-335`) | files in the table above; theme presets `R/src/theme/presets.rs` | medium (many files, one pattern) |
| B2 | Add foreground-on-fill roles (primary-foreground, error-foreground), selection and hover roles, so `text-white` on `bg-blue-*` can be `text-primary-foreground` on `bg-primary` | `B0/theme_tokens.rs:19-45` | `R/src/theme/presets.rs:5-43` | small |
| B3 | Progress bar and table selected row: use roles | study "### theme" | `R/src/widgets/display/progress_bar.rs:233-234`; `R/src/widgets/display/table.rs:166` | small |
| B4 | Make `MenuStyle::default` and the builder default agree | one theme | `R/src/widgets/menu/style.rs:41-53` against `R/src/builder/widgets/menu.rs:913-935` | small |
| B5 | Popover arrow and body take the popover surface color | `G0/popover.rs:330-335` | `R/src/widgets/display/popover/live/render.rs:167-175,388-398` | small |

##### C. Options with no effect

| # | Option | Evidence | Size |
|---|---|---|---|
| C1 | `DialogBounds` fields `size`, `min_size`, `max_size`, `position`, `resizable`, `draggable`, `margin`, and `DialogComponent::get_bounds` | `get_bounds` is defined in the trait (`R/src/widgets/dialog/dialog_component.rs:32`) and in six implementations (`confirmation.rs:577`, `input.rs:598`, `autocomplete.rs:580`, `toast.rs:150`, `progress.rs:125`, `wizard.rs:216`). Search `grep -rn "get_bounds" R/src R/examples R/tests --include=*.rs` found no call other than these definitions. So `min_size`, `max_size`, `margin`, `draggable: true` (`confirmation.rs:269`) and `resizable` are set and never read. Size and position reach the modal through `options.size` and `options.position` in the live files instead (`R/src/widgets/dialog/confirmation/live.rs:162-168`). | small: delete the struct and the trait method, or wire it in |
| C2 | `Toast` `ToastType::Custom(String)` | it is matched only by the `_` branch, so its string changes nothing about the look (`R/src/widgets/dialog/toast/live.rs:147-152`) | small |
| C3 | `ToastOptions::on_close` set through the builder | the builder sets `on_close: None` and offers no setter (`R/src/builder/widgets/dialog.rs:387-393`) | small |

I checked every field of `ModalProps` (`R/src/widgets/display/modal.rs:171-205`) and `PopoverProps` (`R/src/widgets/display/popover.rs:230-260`) with `grep -rn "props\.<field>\|config\.<field>"` under the modal, popover and dialog directories. Each field is read at least once. That check shows a reference, not that the effect is correct. `DialogEngineConfig` fields (`R/src/widgets/dialog/mod.rs:242-252`) are each read in `engine.rs` or `engine/live.rs`.

##### D. Behavior gaps found in the comparison (not defects)

| # | Candidate | Reference | reactive-tui | Size |
|---|---|---|---|---|
| D1 | Toast stacking with a cap and a gap between toasts | `B0/toast.rs:22-55`, `G0/notification.rs:565` | `R/src/widgets/dialog/engine/content.rs:128-131`; `R/src/widgets/dialog/frame.rs:19-34` | medium |
| D2 | Replace a toast by id; action button; title | `G0/notification.rs` (study lines 108-112, 229-238, 337-347 in 0.6.6) | `R/src/widgets/dialog/toast.rs:188-199` (no such fields) | small each |
| D3 | `AlertDialog` role for confirmation dialogs | `B0/alert_dialog.rs:199` | `R/src/widgets/dialog/confirmation/live.rs:191-195` | small |
| D4 | Popover moves focus in on open and back on close by default | `B0/popover.rs` | `R/src/widgets/display/popover.rs:248-249` | small |
| D5 | Merge-style options for dialog buttons (unset stays default) | `G0/dialog/dialog.rs:31-140` | `R/src/widgets/dialog/confirmation.rs`/`R/src/widgets/dialog/mod.rs:263-278` (button styles keyed by variant string) | medium |
| D6 | Language aliases and case-insensitive lookup | `G0/highlighter/language_name.rs:5-37` | `R/src/syntax/resources.rs:106-108` | small (see Part 4) |
| D7 | Theme file loading | `G0/theme/registry.rs:98,151` | `R/src/theme/mod.rs` | small to load, medium with watching |
| D8 | Theme mode and dark/light pair with a way to switch after start | `G0/theme/mod.rs:334,369` | `R/src/backend/suprtui.rs:952-977` sets light once | medium |
| D9 | Label (heading) items in menus | `G0/menu/popup_menu.rs` (study lines 36-37, 117-121) | `R/src/widgets/menu/item.rs:83-103` (five kinds) | small |
| D10 | Shortcut hint built from the keys that trigger it | `G0/menu/popup_menu.rs` (study 1119-1150) | `R/src/widgets/menu/item.rs:3-9` | small |
| D11 | Tie syntax theme to `Theme` and use the theme text color for plain text | `G0/theme/schema.rs` (study 1060-1073) | `R/src/syntax/resources.rs:14-15`; `R/src/syntax/highlighter.rs:88-96` | small |
| D12 | Overlays follow a moving trigger and offset API (`offset`, `on_position`) | `B0/popup.rs`, `B0/positioner.rs` | `R/src/widgets/display/popover.rs` has `offset` and `on_position_change` already (`:241,258`) | none: already present |

---

#### Part 4. Defects

##### 4.1 Study defects: does each still hold at the current code?

1. Markdown fence ```` ```rust ```` is not highlighted. **Still holds.**
   - `R/src/syntax/resources.rs:106-108`: `Language::iter().find(|language| language.name() == name)`. The doc at `:101-105` says "exact and case-sensitive".
   - `R/src/syntax/highlighter.rs:48-50`: `SyntaxHighlighter::new` calls `find_language_by_name(language)` and returns `None` when it does not match.
   - `R/src/markdown/ast_walker.rs:161-166`: passes the first word of the fence info string unchanged. When `new` returns `None`, the code falls to the plain path (`:172-178`).
   - `R/src/markdown/tests.rs:56-71` (`test_code_block_rendering`) uses a lowercase `rust` fence and only checks that some run has a background. `:76` uses `Rust`.
   - The file-name path is case-insensitive for extensions (`R/src/syntax/resources.rs:129-138`). `R/src/editor/syntax_editor.rs:97` uses `from_extension`, and `:58,80` use `new(language)` with the caller's string.
2. Toasts at the same position draw over each other. **Still holds.**
   - `R/src/widgets/dialog/engine/content.rs:129-130` sets `let bounds = Rect::default();` and passes it at `:133,148` (all six kinds).
   - `R/src/widgets/dialog/frame.rs:26-33`: `apply_bounds` changes width, height and position only `if !bounds.size.is_empty()`. An empty rectangle changes nothing except `z_index` and `focus_trap` (`:20-25`).
   - `R/src/widgets/dialog/toast/live.rs:139-146,178` maps the position to a corner and offset 0. Two toasts with the same `ToastPosition` get the same corner.
   - Search for a fix: `grep -n -i "stack\|toast" R/src/widgets/dialog/engine.rs` shows only `:78` (a priority comment) and `:584` (`show_toast`).
3. The highlighter's plain fallback text is painted black on a near-black editor background. **Still holds, with a narrower scope than the study states.**
   - Black is used at `R/src/syntax/highlighter.rs:177` (uncached line in `highlight_lines`), `:327` (`plain_line`), `:342` (`unhighlighted_lines`), `:357` (`fallback_range`). Also `:94,140,213` use black when no theme resolves, and `:257` returns black for a light theme with no `normal` color.
   - Editor background: `R/src/editor/syntax_editor.rs:201` (`0.05` in `render`) and `:217-221` (`get_styled_lines`); `painting::render` fills every cell with that background (`R/src/editor/painting.rs:116-135`).
   - Runs from the highlighter carry a transparent background (`R/src/syntax/highlighter.rs:286-291`, per study), so the editor background shows.
   - When the editor has no highlighter, it uses a `0.9` gray foreground (`R/src/editor/syntax_editor.rs:210-216`, `:238`) and is fine. Black appears only through the highlighter's fallback lines (unknown language for `from_extension`? `find_language_for_file` returns `PlainText`, which does not fall back; the fallback paths are lock failure, over-size input, a failed highlight, or an unresolvable theme). I did not run these paths, so I cannot say how often a user reaches one.

##### 4.2 New items found by reading

1. **`DialogComponent::get_bounds` and `DialogBounds` are dead.** See C1. Options that look like they limit dialog size and margin (`min_size` 300x150, `max_size` 600x400, `margin` 20 each) do nothing. Evidence: `R/src/widgets/dialog/confirmation.rs:263-270`, `R/src/widgets/dialog/dialog_component.rs:439-462`, and the zero-caller search in C1.
2. **Popover default arrow and gap are large in cells.** `R/src/widgets/display/popover.rs:97-105,241`. With arrow on and size 8, the arrow loop emits `2*depth+1` cells for each of 8 rows (`R/src/widgets/display/popover/live/render.rs:357-380`). Combined with the 8-row default gap, the popover sits 8 rows from the trigger. This is what the code does; whether it is wrong depends on intent. The catalog demo (`R/examples/widget_catalog/catalog.rs:795-798`) uses the defaults.
3. **`DialogThemes` presets use pixel-scale utility classes.** `px-4 py-2` on buttons is 16 cells by 8 rows (`R/src/widgets/dialog/dialog_types.rs:204-212,231-239,285-293`); `p-4` titles are 16 cells (`:199,226,280`). Basis: `R/src/layout/css/parsers.rs:45-52` and the test at `R/src/layout/css/spacing.rs:151-170`. I did not find a caller that applies these presets (`grep -rn DialogThemes R/src R/examples R/tests` shows only the export at `R/src/widgets/mod.rs:30`), so the defect is in the public preset values only.
4. **Menu defaults differ between direct and builder use.** `R/src/widgets/menu/style.rs:41-53` (dark `bg-gray-800 text-white`) against `R/src/builder/widgets/menu.rs:913-935` (`white` background, `black` text, `blue-500` selected). `builder::menubar().build()` goes through `convert_menu_style` (called from `:528` per grep).
5. **Modal Escape handler consumes Escape while visible even when the modal does not close.** `R/src/widgets/display/modal/live/events.rs:20-34` returns `EventResult::Consumed` after the `if`. An outer handler (for example a parent popover) will not see Escape while a non-closable modal is visible. This may be intended. Marked as a read-only observation.
6. **Toast `ToastType::Custom` has no styling of its own.** `R/src/widgets/dialog/toast/live.rs:147-152` (falls to `bg-blue-700`); the builder turns any unknown type string into `Custom` (`R/src/builder/widgets/dialog.rs:381-386`).
7. **Confirmation dialogs report `Role::Dialog`** (`R/src/widgets/dialog/confirmation/live.rs:194`). Study lists it as "worth adopting". Still open.

##### 4.3 Things I checked that are fine or unverified

- Fine: the toast auto-dismiss timer starts after the first presented frame (`R/src/widgets/display/modal/live/render.rs:304-316`; `R/src/widgets/dialog/toast/live.rs:179-187`).
- Fine: the modal position subtraction `(viewport - size)` cannot underflow on the auto path, because `dimensions` are clamped to the viewport first (`R/src/widgets/display/modal/live/render.rs:119-127`). `ModalPosition::Custom` is not clamped inside `calculate_position` but is clamped right after (`:124-127`).
- Unverified: whether the catalog overlay demos (`R/examples/widget_catalog/catalog.rs:788-865`) are sized against the card or the screen. It depends on the `clip` of the ancestors. I did not run the example.
- Unverified: the popover arrow's `arrow_position` and the `TopEnd`/`BottomEnd` signs of `offset_x` (`R/src/widgets/display/popover.rs:384-387,402-406`). They subtract the x offset on the End positions. That looks like intentional mirroring; I did not check the tests.
- Not checked: input, wizard, autocomplete and progress dialogs beyond their default class strings.
