# Widget acceptance inventory

API-011 covers both `widgets::*` component/props builders and `builder::*`
Element builders, including public submodule paths and convenience macros.
The following rows are acceptance obligations, not readiness claims. PENDING
rows make the mechanism fail. Reviewed App coverage below refers to the recorded
editing checks; Cairn still reruns the complete mechanism against committed inputs.
A passing unit test or a painted description
does not establish an App workflow. Each interactive row needs keyboard and
mouse input at two viewport sizes, resized hit bounds, disabled/empty cases,
state retention and the advertised callbacks. Display-only rows need actual
content and layout assertions at two sizes. Extra advertised options discovered
during source review are added before their implementation.

| Public family and builder routes | Advertised controls and required observations | App evidence |
| --- | --- | --- |
| Core ElementBuilder; button/input and primary_button; macros | Children, text, classes, focus, activation once, disabled state; core input edits | App coverage reviewed; App cases in tests/api_widget_behavior/core_builders.rs cover editable text seeds, visible placeholders, all four input macro forms, measured clicks after resize, disabled editing/callback suppression and exactly one pointer callback. Existing button activation/disabled probes remain in api_widget_behavior.rs; helper and macro construction, colors and reader paths are covered by core_builders.rs, macros.rs and the Orca workflows |
| TextInput; input::TextInputBuilder; builder::text_input and macro | Value, placeholder, password/email/number modes, readonly/disabled/max length, Unicode editing, selection, history, suggestions, validation and callbacks | App coverage reviewed; editable builder and nineteen input App cases cover modes, Unicode selection/history, suggestions, validation, callbacks and resize. Text runs expose actual values and caret/selection with password masking. Consumer checks cover empty, Unicode, multiline, long grapheme and Select All; real Orca/GNOME Terminal in the Tabs workflow at 100x32 and 100x60 verifies initial value speech, keyboard editing and caret movement |
| Checkbox; input::CheckboxBuilder; builder::checkbox and macro | Toggle, indeterminate transition, label, disabled, change callback, hover/focus | App coverage reviewed; builder/macro App toggles and four checkbox unit tests pass. Real Orca/GNOME Terminal at 32x10 and 60x16 verifies distinct labels, checkbox role, mixed/checked/unchecked announcements, focus, assistive and keyboard toggles, disabled actions and removal |
| Select; input::SelectBuilder; builder::select and macro | Open/close, option navigation, single/multiple selection, disabled options/control, search, empty options, change callback | App coverage reviewed; typed and generic App cases cover single/multiple choices, disabled skipping, callbacks, padding and resized bounds. Prefix-search and parent-authored multiple-selection replacement cases pass at 24x6 and 48x12; a deterministic unit test verifies expiry and bounded input. Real Orca/GNOME Terminal at 32x10 and 60x16 verifies combo/list/option roles, labels, expansion, focus, disabled skipping and assistive/keyboard selection with selected-state speech. Expanded combo assistive close/reopen also passes at both sizes |
| RadioButton; input::RadioButtonBuilder; specialized::RadioButtonBuilder | Single-choice group, horizontal/vertical options, selection, disabled, label/value, change callback; named groups use distinct keyed owners and stay inside one App | App coverage reviewed; generic and named radio App cases in tests/api_widget_behavior/radio.rs; cleanup unit case in named_radio.rs. Real Orca/GNOME Terminal at 32x10 and 60x16 verifies both construction families, group/option roles, labels, focus, disabled skipping, selected-state speech and exclusive assistive/keyboard selection |
| Slider; input::SliderBuilder; specialized::SliderBuilder | Range, step, keyboard/mouse position and dragging, horizontal/vertical tracks, value/end labels, disabled, builder label/classes, change callback, invalid/degenerate bounds | App coverage reviewed; six App cases in tests/api_widget_behavior/slider.rs cover range, builders, measured padding, vertical rows, resizing, callbacks, inert states and dragging/release/focus loss. Real Orca/GNOME Terminal at 32x10 and 60x16 verifies label, slider role, focus, numeric bounds, keyboard step/end and changed-value speech |
| Table and TableProps | Actual columns/rows, fixed/percent/auto/flex widths and min/max bounds, column/cell alignment, border styles/color, row/cell/header/selection/zebra styles, sort, selection and multi-selection, row/cell actions, scrolling, resizing, empty data and stable row IDs | App coverage reviewed; thirteen App cases in tests/api_widget_behavior/table.rs cover named/typed construction, measured sorting/selection/actions, resize/drag/release, scrolling, stable row IDs, widths/alignment/styles, borders/clipping, initial props, intrinsic width in an auto-sized padded parent, disabled/empty input and invalid configurations; retained-offset unit case covers more than 65535 rows. Real Orca/GNOME Terminal at 32x10 and 60x16 verifies table/cell roles, labels, enabled-row keyboard focus and speech, selected/disabled states and assistive cell focus without selection changes |
| DataTable; builder::data_table and macro | Table behavior plus all filter types, global search, pagination, column visibility, export callbacks, virtual scrolling, advertised multi-column sorting and each advertised panel | App coverage reviewed; fourteen App cases in tests/api_widget_behavior/data_table.rs cover builder/macro construction, filter types and editing, search, multi-column sorting before pagination, source-index callbacks, selection across pages, virtual rows/overscan, prop updates, export, column visibility and disabled/empty/error states; public range arithmetic has boundary unit tests. Real Orca/GNOME Terminal at 32x10 and 60x16 verifies search entry role/label, pagination actions and disabled controls, and spoken focus on paged/filtered cells |
| Tree; display::TreeBuilder; specialized::TreeBuilder; builder::tree | Hierarchy, expand/collapse, selection/multi-selection, per-node selectable/checkable/expandable flags, checking, icons/lines/styles, search and filtering, lazy-loaded children, in-memory drag/drop, node actions, measured scrolling/virtual rows, builder classes, changed props and empty/invalid roots | App coverage reviewed; fifteen App cases in tests/api_widget_behavior/tree.rs cover both builders and typed construction, expansion/actions, lazy children and errors, selection/checking, filtering, virtual scrolling, safe in-memory drops, stable IDs, authored prop changes, borders/classes, measured padding/resize and inert input. Real Orca/GNOME Terminal at 32x10 and 60x16 verifies tree/item/checkbox roles, labels, expansion/collapse speech, child and assistive focus, keyboard/assistive checked-state speech and disabled skipping |
| FileExplorer; display::FileExplorerBuilder; builder::file_explorer, simple_file_browser, code_project_explorer, media_gallery_explorer, document_browser, compact_file_picker, system_file_manager, current_directory_explorer, home_directory_explorer and filtered_file_explorer | Asynchronous directory loading and file operations (copy, move, delete, rename), rooted navigation and measured breadcrumbs, distinct list/grid/hierarchical tree views, hidden/extension/search filters, all sort criteria/orders, single/multiple/no selection, keyboard and measured mouse activation, selection/activation/navigation callback IDs, bounded previews, icons and file details, symlink/non-UTF8 paths and filesystem errors, virtual rows and resize, changed props, disabled/empty states and worker cleanup; filesystem mutations use disposable fixtures | App coverage reviewed; seventeen App cases in tests/api_widget_behavior/file_explorer.rs cover all construction routes, navigation, views, filtering/sorting, callbacks with native path identities, preview completion, confirmed/cancelled operations, errors, selection, changed props, measured padding/resize and bounded rows; eight worker cases cover confinement, cancellation, no-overwrite publication, directory permissions, previews, stale reads and owner cleanup. Real Orca/GNOME Terminal at 32x10 and 60x16 verifies file/directory labels and roles, focus speech, navigation and search controls. Windows and macOS filesystem workers and all seventeen FileExplorer App cases pass in native run 34505587248; all hosted native groups also pass in run 34531816183; final committed receipts remain required |
| Chart; ChartsBuilder; builder::chart and macro | All seven chart types; visible series and finite values; indexed X coordinates, explicit/auto axis ranges, tick counts/custom labels/grid/titles; legend positions/width, point/series/palette colors, line/fill styles, hover tooltip metadata, finite animation and live updates, empty/invalid data and measured viewport bounds | App coverage reviewed; real Orca/GNOME Terminal at 60x16 and 100x32 verifies title/image role, focus and keyboard-selected point speech including metadata; fifteen App cases in tests/api_widget_behavior/charts.rs cover all modes and builder routes, exact values/colors, line and fill styles, axis clipping, pie sectors/donut hole, tooltip metadata and keyboard access, signed bars, legend placement, changed props, padded resize, finite animation, reduced motion and empty/invalid/disabled states; owned-deadline unit test in charts/live/motion.rs |
| ProgressBar; display::ProgressBarBuilder; builder::progress_bar and macro | Finite value/range and clamping; measured horizontal thickness/vertical height, segments, stripes, pulse and smooth changes, indeterminate animation, labels/value/percentage/custom formatter, colors/container/bar/text styles, changed formatter/callback props, completion once per transition, disabled input and invalid configurations | App coverage reviewed; real Orca/GNOME Terminal at 60x16 and 100x32 reads the label and changed percentage through object navigation and verifies range/clamping/indeterminate state; ten App cases in tests/api_widget_behavior/progress.rs cover both builders/macros/public factories, precise horizontal/vertical fill and colors, segments/stripes/styles, own padding and resize, formatter/callback replacement, completion transitions, invalid/indeterminate/disabled input, reduced motion and intermediate frames; twelve unit cases include deadline interpolation/reversal/completion/cancellation |
| Popover; display::PopoverBuilder; builder::popover | Retained trigger and actual child content; click/hover/focus/manual control, imperative wakeup, twelve measured positions and four boundary modes, cell offsets and size constraints, three arrow styles, four animations and owned delays, dismissal flags, backdrop, callbacks, prop replacement, autofocus/traps/restoration, disabled/empty/invalid states and resizing | App coverage reviewed; tests/api_widget_behavior/popover.rs covers construction, all positions/boundary modes, arrow direction/styles and mouse targets, nested focus, manual anchors and invalid fallback, content/callback replacement, hover and Escape dismissal, clipping and intermediate animation. Runtime unit tests cover reversal, delayed-hover cancellation, owner replacement and unmount cleanup. Real Orca/GNOME Terminal at 60x16 and 100x32 verifies assistive trigger activation, content focus, input value speech/editing, Escape removal and spoken focus restoration. Typed input triggers retain their own text semantics; the shared App construction and behavior paths are reconciled with this row |
| Modal; Modal::element/with_props and prop helpers; builder::modal | Actual content/footer children; Auto/Fixed/Percent/Viewport dimensions, ten positions and measured resize; border and region styles, backdrop, close/Escape/backdrop dismissal, all button actions/callbacks/disabled/autofocus states, focus traps/restoration, real scrolling, title dragging and eight resize handles, four owned animations, prop replacement and invalid/empty states; dialog engine lifecycle also API-012 | App coverage reviewed; twenty-three App cases in tests/api_widget_behavior/modal.rs cover geometry, actual children/scrolling, callbacks/focus, all resize edges, dragging/clamping/focus-loss cancellation, animation, prop replacement and region colors, dismissal flags, generic/public helpers, empty content, nested Escape, overlapping z-order, inert closing fades, reopening and invalid-prop recovery. Owned transition and duplicate-action unit cases are in modal/live.rs. Real Orca/GNOME Terminal at 60x16 and 100x32 verifies dialog/title speech, input value speech/editing, trapped assistive focus, Escape and Close dismissal, spoken opener restoration and removal; the shared App construction and behavior paths are reconciled with this row |
| ScrollView; layout::ScrollViewBuilder; specialized::ScrollViewBuilder | Real child content/events, horizontal/vertical offsets, signed wheel/keyboard, scroll speed, smooth movement, scrollbars, clipping, classes and resized viewport | App coverage reviewed; real Orca/GNOME Terminal at 60x16 and 100x32 verifies scroll-pane role, label, clipping, scrolled child focus/value speech and editing; tests/api_widget_behavior/scroll.rs covers both builders, Unicode cells, scrolling and clipping of nested controls, signed wheels, offsets/resize, bars, empty/disabled axes and smooth intermediate frames; motion unit test checks deadlines and cleanup |
| Stack; layout::StackBuilder; specialized::StackBuilder | Real children and their events, direction/reversal/wrapping, alignment, justification, cell spacing, padding and classes | App coverage reviewed; App cases in tests/api_widget_behavior/stack.rs cover both builders, nested editing/clicks, padding, alignment, all justification modes, reversal and wrapping at two sizes |
| Tabs; layout::TabsBuilder; builder::tabs and macro | Active child panels, lazy/eager mounting and retained state, keyboard activation modes, positions/orientation, size/variant, icons/badges/tooltips, disabled tabs, close callbacks (or local removal without a callback), resize and empty tabs | App coverage reviewed; tests/api_widget_behavior/tabs.rs covers both builders, panel editing, all positions/orientations, Unicode measured targets, resizing, change/close callbacks, disabled/empty controls, lazy/eager mounting, hidden child events and retained input. A fresh-child root regression now verifies local selection, editing and close survive reconstructed child Elements; keyed reorder and explicit authored selection have unit coverage. Ten App cases include closing an earlier tab without losing unkeyed editor state. Real Orca/GNOME Terminal at 100x32 and 60x16 verifies tab/list roles, labels, disabled skipping, selected/focused speech, active-panel text/caret, assistive activation and labelled Close actions |
| Accordion; layout::AccordionBuilder; builder::accordion and convenience variants | Single/multiple/always-one expansion, measured custom headers, retained nested content, disabled/empty sections, callback IDs, persistence, icons/classes, duration/stagger/reduced motion, and advertised ARIA labels/screen-reader support | App coverage reviewed; eight App cases in tests/api_widget_behavior/accordion.rs cover expansion, headers, notifications, disabled/empty/release, retained editing/resize, persistence on changed props and intermediate animation; motion unit case covers spring timing, stagger, reversal and cancellation. Real App/Orca workflows verify distinct labels, roles, focus, disabled actions, expansion, nested input and removal at 32x10 and 60x16. The closing-body case also verifies immediate removal from mouse activation and Tab order while the animation is still painted. Shared transport failure tests and the catalog Orca workflows cover delivery and owner removal. |
| Breadcrumb; layout::BreadcrumbBuilder; builder::breadcrumb and convenience variants | Navigation callback IDs, current/nonclickable items, measured overflow strategies, separators, icons/home/compact options, tooltips, keyboard navigation, ARIA labels and empty path | App coverage reviewed; fourteen App cases in tests/api_widget_behavior/breadcrumb.rs cover named/typed builders, callbacks, measured Unicode/padding, overflow, wrapping, scrolling/wheels, tooltips, icons/classes, resize, prop reorder, replacement focus and inert input. Real Orca workflows at 32x10 and 60x16 verify labels, links, focus, current-page announcement, inert actions, callbacks and removal |
| MenuBar; menu::MenuBarBuilder; builder::menubar | Dropdowns, nested actions, shortcuts, checkbox/radio entries, separators, enabled/visible flags, selection/dropdown callbacks | App coverage reviewed; App workflows in tests/api_widget_behavior/menus.rs cover nested selection, retained checkbox values, shortcuts, measured clicks, outside dismissal, nested hover, wheel/keyboard scrolling, disabled/empty menus, callback order/replacement, prop reorder, initial public state and shared styles/separators. Orca/GNOME Terminal menu workflows pass at 32x10 and 60x16, including item roles, assistive and keyboard focus, checked-state announcements, nested activation and removal. Public state and callback updates, authored dropdown scroll offsets and resized open-dropdown pointer targets now pass at both sizes; the App construction and behavior paths are reconciled with this row |
| ContextMenu; menu::ContextMenuBuilder; builder::context_menu | Actual items, trigger position, viewport constraints, nested navigation, actions, close and disabled/empty entries | App coverage reviewed; three App workflows cover right-click opening, idle long-press opening and generic builder callback order at two sizes. Two timer tests cover cancellation and owner isolation. Shared nested hover, wheel and disabled/empty App cases also pass. The public props builder, offset-parent trigger areas, pointer activation and Orca focus/nested activation/removal now pass. Public state/callback updates and resized pointer targets pass at both sizes. App long-press cancellation after release, move and drag outside an offset owner waits beyond the deadline and passes; the App construction and behavior paths are reconciled with this row |
| PopupMenu; menu::PopupMenuBuilder; builder::popup_menu | Placement, open/close, navigation, item actions, callback and disabled/empty entries | App coverage reviewed; App workflows cover painting, nested keyboard and hover selection, measured clicks, outside dismissal after submenu closure, wheel navigation, separator-aware scrolling, numeric padding/fixed-width caps, generic styles and builder callbacks at two sizes. Repeated open/cancel/select restores trigger focus. All seven public placement modes and resized mouse targets pass. Public scroll seeds, callback replacement, prop reorder, repeated remounts, open-owner removal and disabled/empty states pass at both sizes. Public props builders and real Orca focus, checked-state announcements, nested activation and removal pass at 32x10 and 60x16. The App construction and advertised controls are reconciled with this row |
| DialogMenu; menu::DialogMenuBuilder | Each menu type, selection, confirmation/cancellation, callbacks and empty/disabled entries | App coverage reviewed; App workflows cover painting, selection/hide callbacks, multi-selection confirmation, confirmation/cancellation, measured clicks, outside dismissal and wheel navigation. Unicode input includes paste, horizontal scrolling, backspace/forward delete and exact submission. Public-state tests cover grapheme editing and invalid/extreme positions. Hover and measured-page navigation pass. Repeated open/cancel/select restores trigger focus for modal and nonmodal menus. Public scroll seeds, callback replacement, prop reorder, repeated remounts, open-owner removal, nested multi-selection and disabled/empty states pass at both sizes. Public props builders and real Orca focus, checked-state announcements, nested multi-selection confirmation and removal pass at 32x10 and 60x16. Resized pointer targets and public state/callback updates now pass at both sizes; the styled selected/disabled-row case and measured outline cases also pass |
| DialogBuilder; specialized::DialogBuilder; dialog_builders; builder::dialog | Functional content and control construction; completion, stacking, async and focus lifecycle additionally API-012 | App coverage reviewed; two App workflows at 32x12 and 60x20 verify retained editable children, Escape dismissal, nonmodal background input and non-closable behavior. Orca/GNOME Terminal at 60x16 and 100x32 verifies dialog and entry roles, spoken input, editing and Escape dismissal |
| ConfirmationDialog and builders | Buttons and configured actions; result delivery additionally API-012 | App coverage reviewed; public and generic builder App cases at 32x12 and 60x20 cover disabled/default buttons, ordered Tab navigation, action veto and exactly one close result. Relative positioning now has App coverage for all nine anchors, offsets, changed parent layout, resize, removal and ambiguous keys. Orca/GNOME Terminal at 60x16 and 100x32 verifies dialog speech, default focus, disabled action and assistive completion. All seven position modes and explicit render bounds track viewport resize; updated content, enabled buttons and replacement callbacks pass before resized activation. Native engine lifecycle checks remain unfinished under API-012 |
| InputDialog and builders | Editing, password, validation, submit/cancel callbacks; result delivery additionally API-012 | App coverage reviewed; twenty-six integration cases cover required/rule and mask validation, Unicode limits/deletion, password masking, multiline editing, read-only/callback/seed updates, pointer editing/submission after resize, blur validation, debounce coalescing/rescheduling/cancellation, warnings and App remote HTTP validation. See [input formats and updates](dialog-input.md). Six transport unit tests and the Linux HTTPS fixture cover request escaping, cancellation, deadlines, bounds, errors and trusted/untrusted certificates; see [HTTP evidence limits](dialog-http.md). Windows/macOS HTTP and TLS trust cases pass in run 34505587248. Native engine HTTP/results remain unfinished under API-012. Orca/GNOME Terminal at 60x16 and 100x32 verifies entry speech, required-error and nonblocking warning announcements, editing and confirmed result |
| AutocompleteDialog and DialogBuilder::autocomplete | Static and HTTP suggestions, grapheme minimum length, debounce, request headers and errors, maximum/empty results, custom filtering/rendering, display/description/icon/metadata, matching styles, keyboard/wheel navigation, measured selection after resize, selection veto, draft and callback updates; results additionally API-012 | App coverage reviewed; App cases in tests/api_widget_behavior/autocomplete.rs and dialog_http.rs cover retained input, Unicode editing, static/custom filtering, selection and cancellation, resized clicks, long-list selection, callback replacement and real HTTP requests. Native unit cases cover Unicode editing and selection veto regressions. See [suggestion behavior and transport limits](dialog-http.md). App removal cancels a live HTTP request; query replacement and debounce checks pass. Additional two-size cases verify exact input/content colors, matched-only bold/underline, disabled highlighting, grapheme minimum length, placeholders and description visibility. Native engine HTTP/results remain unfinished under API-012. Orca/GNOME Terminal at 60x16 and 100x32 verifies input/list/option roles, selected-option speech and assistive selection with the correct result |
| ProgressDialog and builders | Updates, cancellation, labels and progress; lifecycle additionally API-012 | App coverage reviewed; public and generic builder App cases at 32x12 and 60x20 cover actual percentage/message, cancellation once, changed progress and callback, non-cancellable Escape and estimated-time text. Orca/GNOME Terminal at 60x16 and 100x32 verifies dialog/progress roles, numeric value speech and cancellation. Resized pointer cancellation emits one callback; indeterminate progress advances without input or a false percentage. Native engine lifecycle checks remain unfinished under API-012 |
| WizardDialog and builders | Stable step IDs and retained child input, Next/Back/Skip/Finish, step validation with explicit named data, completion veto and cancellation, progress, builder initial step/can_proceed/cancelable/classes, prop updates and empty/invalid configurations; lifecycle additionally API-012 | App coverage reviewed; twelve App cases in tests/api_widget_behavior/wizard.rs cover measured navigation after resize, keyboard navigation, validation/Skip, authored data, completion veto, cancellation, builder controls, retained editing and reordered steps with new callbacks/data. Four native regression cases cover empty/duplicate configurations, validation and named values. Removing and reinserting a visited step clears its previous child edits. Noncancelable wizards still accept keyboard Finish after ignoring Escape. Exact builder background color also passes with reduced motion. Native engine lifecycle/results remain unfinished under API-012. Orca/GNOME Terminal at 60x16 and 100x32 verifies validation speech, step navigation, nested editing and completion |
| Toast and builders/macros | Actual message/type/position, timeout and dismissal; lifecycle additionally API-012 | App coverage reviewed; public/generic App cases at 32x12 and 60x20 cover painted message, expiry without input, all six measured positions and continued background editing. Unit coverage verifies cancellation of deadlines on manual close and unmount with retained output. Orca/GNOME Terminal at 60x16 and 100x32 verifies message speech, expiry and retained background focus. Resized pointer close emits one callback; updated message, duration and callback retain the existing owner and expire correctly. Native engine lifecycle checks remain unfinished under API-012 |
| Image; specialized::ImageBuilder; builder::image | Functional source/placement construction; decoding, protocols and URL decision additionally API-014 | Native image and twelve App workflows pass; Kitty, Sixel, inline, Chafa and Viu have real-host update/removal captures (see image details below); Surface host checks pass; Orca/GNOME Terminal at 60x16 and 100x32 verifies image role, alternative/changed labels and hidden fallback cells; App coverage reviewed; macOS and Windows native image groups pass in hosted run 34531816183, supplemented by MacBook and Windows 11/MSVC runs. The approved iTerm 3.7 color/transparency limit is stated below; committed native receipts remain required |
| TerminalWidget and TerminalProps | Actual styled grapheme screen, title and scrollbar; focused keyboard/paste, signed scrollback and measured resize; executable/environment/directory props, retained session on display changes and replacement on launch changes, idle output wakeup, process exit/errors, bounded queues and stop/reap on removal; real Unix PTY and claimed native Windows behavior; retained terminal entry path additionally API-016 | App coverage reviewed; 16 App workflows in tests/api_widget_behavior/terminal.rs cover real colored child output, measured resize at two sizes, launch props, focus and release suppression, signed history scrolling, display-prop retention, launch replacement/errors and child reaping. Unicode, extended colors, dim/hidden text, alternate screens, child cursor/paste modes, wide block cursors and silent-child blinking also pass. Native underline/bar preserves text, follows clipping and image/text coverage, retries failed output and resets host shape/color. Screen tests cover width reflow across retained history and the visible grid, bounded history, active/saved cursors, parser recovery, saved cursor bounds, whole-glyph erasure, modified keys, character/line edits, insert mode, scrolling margins and tab stops. Fallible screen/terminal constructors reject invalid dimensions before allocation; a widget reports invalid size and starts only after valid measured/explicit resize. Scrollbar press/drag uses measured padded bounds and cancels on release/leave/blur/stop/hide. Orca/GNOME Terminal at 60x16 and 100x32 verifies terminal role/title, focus, readable screen, real child input/output speech and adapter removal. Shared text-run tests cover Unicode, hidden-cell privacy and caret/scrollback mapping. The Windows input/resize App workflow, ten native terminal tests and 37 screen tests pass on both hosted GNU and Windows 11/MSVC. The complete ConPTY probe passes after independently accounting for Windows initialization; a deliberate leaked handle still fails. Committed native receipts remain required |
| Core/layout convenience builders; MixedElementBuilder/from_vdom; macros | Real child content, layout, style and event preservation across every public construction route | App coverage reviewed; core helper and VDOM App cases cover default containers, headings, labels, editable/styled/search inputs, primary button, fixed and responsive grid/flex placement, native payloads, disabled and resized activation, inline styles/errors, registered component props/attributes and replacement handlers, and native control round trips. VDOM menu actions and all public builder macro families now have App coverage at both sizes; all public construction families are reconciled with the App cases |

The overlapping requirements above do not remove these entries from API-011.
Their final checks must agree with the dedicated lifecycle/protocol/entry-point
mechanisms. App evidence does not establish DialogEngine readiness: engine
stacking, synchronous/asynchronous results and lifecycle remain unfinished under
API-012. API-014 and API-016 also retain their dedicated obligations. All remain
inside the current 54-requirement remediation commitment, and API-020 requires
their reconciliation before Done. See
`docs/decisions/record-app-widget-evidence-while-retaining-the-engine-lifecycle-requirements.md`.

## Table controls under acceptance

Table now uses terminal-cell widths and measured content bounds. Explicit column
minima and maxima apply to fixed, percent, auto and flex widths; overflowing
columns remain reachable with Left/Right or Shift-wheel when scrolling is enabled.
Up/Down, Home/End and PageUp/PageDown select rows in displayed sort order and skip
nonselectable rows. Shift-navigation adds rows in multi-select mode; Space and
Ctrl-click toggle a row, and Ctrl-A selects all selectable rows. Enter emits the
row's `select` action. Clickable cells emit their configured action. Selection and
action callbacks report source indices, and row IDs retain selection across reorder.
Dragging the last cell of a resizable header changes that column's width until
release or focus loss. The public unit components and named construction routes
remain available.

DataTable applies search, active filters and sorting before pagination. Click a
header to replace the sort order; Shift-click adds or toggles a secondary column.
The displayed arrows and priority numbers show that order. The filter panel cycles
between contains, equals, numeric range, date range and boolean modes. Numeric
ranges use `min..max`; date ranges use `YYYY-MM-DD..YYYY-MM-DD`. Invalid edits show
an error and preserve the last valid filter. On/Off toggles a configured filter;
editing a column replaces that column's filters. Search and filter changes return
to the first page. Column controls preserve data while hiding its rendered column.
CSV and JSON buttons deliver the configured export-request callback.

The virtual-scroll configuration retains its pixel-based row-count calculation:
`ceil(viewport_height / row_height)`. App caps that request to its measured terminal
area and paints each data row on one terminal line. Overscan retains extra rows
outside the clipped body without painting them over the header or exposing hidden
mouse targets. Query and page changes reset the visible row window. Zero page size
or virtual row height produces a visible configuration error. These descriptions
document the implementation under test; the catalog rows remain pending final
coverage and mechanism review.

## Tree controls under acceptance

Tree retains user expansion, selection, checking and loaded children across redraws.
The root follows its expanded flag, like other nodes. Changed node flags and explicit
selection/expansion props replace the corresponding authored state; reorder preserves
state by node ID. Duplicate IDs produce a visible error. The display builder, specialized
builder and named/typed construction routes render the same control. Builder classes
reach its painted elements.

Up/Down, Home/End and PageUp/PageDown move selection through visible selectable nodes.
Left collapses or selects the parent; Right expands or selects the first child. Enter
also sends the `activate` node action. Plus/minus expand/collapse, and asterisk expands
branches at the current level. Shift-navigation extends multi-selection, Ctrl-click
and Space toggle it, and Ctrl-A selects visible selectable nodes. Space checks the
current node when checking is enabled. Global `checkable` enables all checkboxes;
individual node `checkable` flags can enable them without the global option. A supplied
checked value without either flag paints a read-only checkbox. Nonselectable nodes can
still expose expansion or checking. Disabling the entire Element suppresses all input.

Search opens matching paths for display and underlines matches. With filtering enabled,
only matches and their ancestors remain in the visible/input order. Clearing search
restores the retained expansion state. Wheel direction and measured viewport height
control scrolling; Shift-wheel or Shift-Left/Right reaches horizontally clipped labels.
Virtual scrolling limits constructed rows to the visible window. Auto-sized parents
receive the tree's intrinsic width, and connectors occupy explicit cell positions.

Expanding a lazy node uses its configured synchronous child callback once, including
an initially expanded node. Returned children become real rows; missing callbacks or
duplicate returned IDs produce visible errors. Replacing the authored root clears the
loaded-child cache. Dragging a label onto an expandable node moves that subtree in the
owned in-memory tree and reports `drop:<target-id>` through the source node's action
callback. Root moves and descendant cycles are rejected. This operation does not alter
files. Release or focus loss ends a drag. These are implementation checks; final catalog
acceptance and the remaining widget families are still pending.

## FileExplorer controls under acceptance

FileExplorer owns one filesystem worker per mounted control. Directory reads and
bounded previews wake App when they finish. A new path rejects stale responses;
removal cancels queued work and joins the worker. Cancellation is checked between
filesystem calls and copy chunks. An operating-system call already running cannot
be preempted, so worker shutdown does not have a hard deadline. Reads and the
retained directory cache each have a 100,000-entry limit; painting is bounded by
the measured viewport and configured visible-item limit.

List and grid show the current directory. Tree expands directories lazily. The
view, sort, order, hidden-file, search and preview toolbar controls also have
keyboard shortcuts: V, S, O, period, slash and P. F5 refreshes cached contents.
Up/Down, Home/End and PageUp/PageDown move the cursor; grid also uses Left/Right.
Enter activates an entry; directory activation navigates. Space and Ctrl-click
toggle selection, Shift-navigation extends it, and Ctrl-A selects visible entries
in multiple-selection mode. Root and path breadcrumbs use measured mouse targets.
Filename display may be lossy; identity and filesystem operations retain native
paths. Callback payloads contain `paths` display strings, `native_paths` integer
arrays and `encoding` (`bytes` on Unix, `utf16` on Windows).

F7 copies, F6 moves, F2 renames and Delete requests deletion. The same operations
are available on the toolbar. Destination forms accept typing and paste; Enter
confirms and Escape cancels. Rename takes one basename. Delete requires explicit
confirmation. Copy publishes a privately staged result without replacing an
existing destination. Move and rename use atomic rename without replacement;
cross-filesystem moves return an error and preserve the source. Errors identify
partial work when earlier selected entries may already have changed. Cancel stops
remaining work; it does not undo completed changes. All mutation tests use
disposable directories.

Filesystem access is relative to a retained root directory capability. Traversal
and external symlink reads fail; copying a symlink preserves the link itself.
Previews read at most 16 KiB plus one byte to detect truncation, preserve a valid
UTF-8 prefix, and summarize binary content. Unix directory and file permission
bits survive copy, and failed publication removes private staging. Linux behavior
has executed tests. Windows compilation and native platform execution are distinct
checks; platform and screen-reader acceptance remain pending in the catalog row.

## Chart controls under acceptance

Chart and Charts construction routes share a retained, measured cell renderer.
Line and scatter use point indices as X coordinates and point values as Y.
Vertical bars group series at each point index; horizontal bars use values on X
and list the visible points down Y. Finite explicit axis limits clip data; auto
ranges include zero. Ticks, custom labels, grid lines and axis titles use the
remaining space after title and legend placement. Point labels appear at matching
category ticks. Invalid dimensions, nonfinite values, invalid colors and impossible
ranges display errors. A chart canvas has a one-million-cell allocation ceiling.

Line styles draw solid, dashed, dotted or no connecting line while retaining point
markers. New DataSeries values default to solid area fill; explicit FillStyle::None
leaves the area unfilled. Gradient fill uses terminal shading characters. Pattern
fill supports `diagonal`, `dots`, or a repeating sequence of printable one-cell
graphemes. Pie and donut draw proportional sectors, with a half-radius hole for
donut and terminal-cell aspect correction. These modes reject negative values and
show an empty message for a zero total. Point colors override series colors, then
the palette; pie/donut palette entries distinguish points without explicit colors.
Legend position and maximum width control actual placement and clipping.

Hovering a painted point, line, bar or sector shows its label, value and metadata;
long tooltips wrap within the chart. Left/Right, Home/End and Escape also browse or
clear point details when tooltips are enabled. Disabled controls suppress input.
Changing data updates the plot and clears stale tooltips. Animation reveals values
from zero (or sweeps circular sectors) over the configured duration. Its App-owned
timer stops on completion and is cancelled on removal. A `reduced-motion` class in
ChartProps applies final values immediately. Actual screen-reader acceptance for
this catalog row remains pending.

## ProgressBar controls under acceptance

Both ProgressBar builder families and the macro paint the actual control.
Horizontal bars fill left to right; vertical bars fill bottom to top. Height
controls horizontal thickness or vertical length. Width is an optional container
constraint; without it the bar uses the measured available width. Segments fit
within that length and include gaps where cells permit. Values clamp visually to
the finite minimum/maximum range, while custom formatters receive the supplied
value and bounds. Labels, percentage/value text and their styles remain separate
from bar styling. Colors accept hex, named colors, RGB/RGBA values and the existing
`bg-`/`text-` color tokens. Invalid ranges, dimensions, segment counts or colors
produce a visible error and cannot trigger completion.

Completion fires once when a valid determinate value first reaches its maximum,
including an initially complete value, and again only after it drops below the
maximum and reaches it anew. Replacing a completion callback or formatter takes
effect without changing the numeric value. Indeterminate mode never completes.
The control is display-only; keyboard and mouse input do not change its value.

Animated value changes interpolate from the displayed fraction over 200 ms.
Indeterminate movement and pulse use an owned App scheduler deadline, independent
of unrelated input. Stripes use terminal pattern characters; pulse changes filled
cell opacity. Deadlines stop after a finite transition and are cancelled on removal.
The `reduced-motion` token in the container style applies final values and a static
indeterminate segment without scheduling animation. Orca/GNOME Terminal at 60x16 and 100x32 reads the label and initial/updated
percentage through object navigation. The delivered progress role and numeric
range track valid determinate values, clamped to their bounds. Invalid and
indeterminate controls expose no numeric value; indeterminate state is delivered.
Automatic progress announcements depend on reader settings and are not asserted.

## Approved accessibility contract

The Accordion module advertises full ARIA and screen-reader support.
The audit found AccordionSection::aria_label and BreadcrumbSegment::aria_label
storing strings without a reader. The repaired Accordion and Breadcrumb paths now have real Orca
workflow evidence, including nested input and the current-page announcement.
Shared accessibility styles now retain values through StyleBuilder snapshots
and reach App input routing and reader publication. The Orca workflow verifies
CSS reference labels, descriptions, toggle roles, focus, pressed changes, polite
live announcements, hidden content and nested screen-reader-only text. A separate
negative run removes the semantic styles and must fail to find the control.
Keyboard navigation and reduced motion also have App and clock tests; they do
not substitute for the reader workflow. The catalog as a whole remains pending.

Use `layout::css::focus::apply_aria_attribute` to supply label/property values and
`apply_role` to supply roles in explicit styles. Bare `aria-label`,
`aria-labelledby` and `aria-describedby` tokens require a supplied value;
`aria-label` also preserves a label supplied by an existing Element label API.
An incomplete marker fails before frame presentation. Reference values name
space-separated IDs supplied with `Element::with_accessibility_id`; IDs are unique
within an App. Duplicate, missing and cyclic references are errors. `sr-only`
removes visual layout space while retaining reader content; `not-sr-only` restores
the authored size. `tabindex-*` changes App focus order, `keyboard-only` suppresses
mouse hit targets, and `reduced-motion` uses static style values without starting
an animation clock for that element.

Approved in api-011-api-018 on 2026-09-09: verify widget screen-reader
support with Orca in GNOME Terminal on Linux. Preserve the public label APIs and
make focus, accessible labels, roles, expanded/selected/disabled state and dynamic
changes reach that screen reader. Verify that a custom accessibility label reaches
the reader even when it differs from painted text. Keyboard navigation must reach
nested controls, collapsed/hidden content must not remain navigable, and removing
a control must remove its accessible representation. The test must observe what
Orca receives or announces, with the terminal, Orca and desktop versions recorded;
painting expected text or retaining metadata is insufficient.

This approval narrows the former blanket screen-reader claim to one verified
terminal/reader pair. Other pairs, including screen readers with Kitty or Ghostty,
would be explicitly unverified. It does not change their existing rendering/input
acceptance. Screen-reader implementation and integration evidence remain required before
acceptance; this approval is not evidence that the path already works.

Orca is GNOME's desktop screen reader; its documentation describes speech and
braille access, not proof of this library's integration:
[Orca introduction](https://help.gnome.org/orca/introduction.html).

## Popover controls under acceptance

Popover retains its trigger while closed and positions expanded child content from
measured terminal-cell bounds. Both component and Element builders use the retained
owner. Public show/hide requests wake App; callbacks run after state locks are
released. Unchanged visibility props preserve user interaction, while changed
visibility props set the next state. Cloned public handles refer to the same owner.

Flip chooses the opposite position when it reduces overflow, then shifts any
remaining overflow toward the viewport. Shift retains the requested side. Hide
suppresses a body that does not fit. Ignore preserves content size and placement;
the terminal still clips output. Explicit min/max dimensions constrain content,
and a zero maximum can hide its body. Contradictory minimum/maximum dimensions or
durations outside the clock range show a configuration error. Nonfinite or reversed
manual anchor rectangles restore measurement of the rendered trigger.

Arrow size and offsets use cells. Arrows point toward the trigger, use solid,
outline or double-line glyphs, and share the body animation. Their generated depth
is bounded by the viewport. Fade changes cell opacity; Scale and Bounce move
upright glyph cells around the body center; Slide travels three cells. The backdrop
filter is a translucent black cell overlay. These are terminal approximations.

Hover delays cancel on leave/reentry and stop on unmount. Escape is consumed while
the overlay is visible, including when Escape dismissal is disabled, so it does
not reach App's default quit action. Closing content becomes inert immediately.
Autofocus targets expanded descendants; nontrapping overlays allow Tab to leave
and restore focus only if it was still inside. Nested traps restore their opener.
Orca/GNOME Terminal at 60x16 and 100x32 verifies trigger activation, expanded
content focus, input speech/editing and spoken opener restoration after dismissal.

Menu follow-up: nested DialogMenu multi-selection checkbox painting, retention across prop updates, and removal under disabled parents are covered by `dialog_menu_nested_multi_selection_paints_and_survives_prop_updates` at 32x12 and 60x20. Other outstanding menu acceptance work remains as recorded above.

## Wizard controls under acceptance

WizardDialog renders retained step panels through the shared Modal control. Next
and Finish validate the active step. Skip bypasses validation only on an optional
step; on the last step it still respects completion veto. Back preserves visited
child input. Step IDs must be nonempty and unique, and remain the identity across
reorder. Removing a step unmounts its panel. An empty or invalid configuration
shows an error. The builder uses generated positional IDs, honors its initial step
and can_proceed flag, and supplies actual buttons and progress.

Callers provide named string values with WizardDialog::set_data; data() exposes
the authored map. Child callbacks update caller state, and the next render supplies
that map to validators and completion callbacks. The wizard does not infer field
names from arbitrary child Elements. Changing callbacks or data preserves local
navigation; changing the authored initial step resets it. Native engine result
delivery, remaining lifecycle cases and reader acceptance are still pending.

## Relative dialog placement under acceptance

DialogPosition::RelativeTo resolves element_id against a unique presented Element
key. ElementBuilder::id already maps to that key. The selected anchor point uses
the target rectangle after screen clipping, plus the supplied cell offset; Modal
then applies its viewport limits. All nine anchor points are exercised through App.

App replaces its named geometry after acknowledged presentation. A changed target
or viewport schedules the next placement update. Hidden, removed and missing
targets show an unresolved-anchor error; repeated visible keys in separate branches
show an ambiguous-anchor error. These local keys remain legal elsewhere in the tree.
Geometry and requested names belong to one App and are released when unused.
Native engine positioning and the remaining dialog controls retain their separate
acceptance obligations.

## Core input construction under acceptance

The core input() builder and input! macro forms construct TextInput. text() seeds
the editable value; placeholder() displays its exact text while empty. Rebuilding
with the same seed preserves local edits through TextInput ownership. Classes,
children, keys, styles and focus settings remain on the resulting control. An
on_click callback observes a pointer click before the editor handles it; typing
does not activate that callback. Disabled controls suppress editing and callbacks.

## Image recovery in progress

The native image tests decode actual file, base64, data-URL, encoded-memory and
raw RGB/RGBA sources. PNG, JPEG, GIF, BMP and TIFF have pixel comparisons.
Malformed raw extents and encoded payloads return errors. Encoded inputs have a
64 MiB limit; decoded RGBA storage has a 256 MiB limit. The decoder library also
receives an allocation limit, which its API documents as best effort. HTTP image
loading remains explicitly unsupported; data URLs and local-path Url values are
retained by the recorded URL decision.

Kitty captures verify RGBA format, chunk limits and one display action. iTerm2
captures decode the PNG payload and verify byte size and pixel dimensions.
ASCII fallback uses decoded pixels and honors both requested cell dimensions.
Opaque renderers composite transparency over the selected background, or black
when none is supplied. Explicit Fallback mode still displays the caller's text;
unavailable graphics modes fall back to the decoded image.

External renderer tests preserve caller files through success, failure and missing
tools, verify independent owned temporary files, and decode the PNG prepared from
raw memory. Command fixtures verify the five-second deadline, 16 MiB output limit,
invalid UTF-8 and nonzero exit errors, and child cleanup. The library reuses its
owned-process runner for these commands and capability probes.

Logs: `api-011-image-ownership-baseline.log`, `api-011-image-raw-baseline.log`,
`api-011-image-protocol-baseline.log`, `api-011-image-fallback-baseline.log` record
safe violating cases. `api-011-image-fallback-corrected.log` records all 26 native
image tests passing. `api-011-image-native-clippy.log` records strict all-target
Clippy passing. These files are under `.cairn/reviews/`; they are editing checks,
not committed Cairn acceptance evidence.

Eight App workflows now verify the retained builder and native Image-to-Element
route: file/raw/base64/data-URL sources, format hints, classes, errors, empty state,
keyed source updates, resize, parent clipping, independent images, alpha/background
options, quality, text fallback and removal. Worker tests verify latest-source
results and release of shared ownership after shutdown. See
`api-011-image-retained-tests.log` (38 filtered library tests and eight App tests)
and `api-011-image-retained-clippy.log` (strict all-target Clippy).

Kitty output now uses the owned App frame path. Ten App image workflows include
raw, file and encoded-memory transmission, source updates and removal at two sizes.
Backend captures compare RGBA pixels, placement, clipping, layering and cell sizes,
exercise text/background occlusion and verify failed-flush retries and shutdown
cleanup (`api-011-image-graphics-combined.log`). The writer defaults to no graphics;
`with_writer_and_images` accepts explicit Kitty support and physical cell pixels.
Native construction uses environment detection and terminal pixel dimensions when
available, with an 8x16 fallback. The App projection and per-frame encoded output
each have a 64 MiB limit; total cell coverage also has a bounded allocation.
The real-host fixture in `tests/api_widget_behavior/image_host_probe.rs` and its
X11 capture driver verify image pixels, source changes, movement and removal in
Kitty 0.45.0 and Ghostty 1.3.1. Captures and measured color bounds are retained in
`.cairn/reviews/api-011-image-host-{kitty,ghostty}-corrected/`. The Ghostty baseline
rendered ASCII because its environment name was unrecognized; adding `ghostty`
to capability detection makes the same check pass. These are Linux/Xvfb checks
using system Mesa software rendering, not macOS or hardware-GPU evidence.

Sixel and inline images now share the owned App frame output. Actual Xterm and
WezTerm captures verify source changes, movement and removal. Full-height Xterm
captures expose and correct a scrolling defect: saved/restored DECSDM and bounded
origin padding keep the image and overlaid label in place. Native protocol output
now respects background and resize quality. The platform adapter decodes files and
encoded memory, applies source clipping/scaling/offsets, and shares bounded Sixel
encoding and decoding. Platform Sixel/inline/Kitty outputs have actual host captures.

Explicit Chafa and Viu App modes run off the App thread, parse captured symbol
output into a bounded grid, and compose styled runs through normal layout and
clipping. No child escape sequence is forwarded to the host. Requests reuse decoded
pixels on resize and cancel obsolete external processes. Chafa 1.18.1 and Viu 1.6.1
have Kitty host captures for source changes, movement, enlargement and removal
(`api-011-image-host-{chafa,viu}-resize/`). A forced ASCII case fails the same color
assertions (`api-011-image-host-external-ascii-negative/`). Worker fixtures verify
resize after deletion of the original file and prompt cancellation/reaping on
removal. Viu receives preprocessed pixels to honor aspect, background and quality.

Auto retains graphics priority and probes Chafa then Viu when environment detection
has no graphics protocol. A GNOME Terminal App capture passes real image colors,
source changes, movement and removal (`api-011-image-host-auto-chafa/`); isolated
worker tests cover tool priority, missing tools and graphics priority.

Animated GIFs now retain bounded decoded frames and advance on the image worker's
condition-variable clock. Limits are 4096 frames and 256 MiB cumulative RGBA storage;
zero delays use a ten-millisecond minimum. Disposal, repetition and frame deadlines
have pixel/time tests. Finite animations stop, resize keeps the clock, and replacing
or removing the source releases its frames. The App workflow advances and repeats
without input; disabling frame wakeups makes it fail. Real unchanged-GIF captures
advance, move and disappear through Kitty, Xterm Sixel, WezTerm inline, Chafa and Viu
(`api-011-image-host-gif-{kitty,sixel,inline,chafa,viu}/`). Standalone one-shot
serializers remain snapshots; App owns timed playback.

Surface and DiffWriter now validate RGBA bytes, preserve source-region remainder
pixels, bound destination work to visible cells, and remove dangling placements.
Checked registration and placement methods report invalid input; legacy registration
returns reserved ID zero on error. A zero source extent uses the remaining image.
DiffWriter defaults to decoded two-sample half-block cells, and accepts explicit
host graphics options. It shares App's protocol encoding and cleanup owner. Its
checked preparation emits no output on invalid placements; callers acknowledge only
after write and flush succeed. The legacy Renderer uses this path, propagates output
errors, retries a complete frame, and clears its graphics before terminal restoration.

Surface output has actual Kitty, Xterm/Sixel, WezTerm/inline and GNOME fallback
captures for source changes, movement and removal, including full-screen Sixel
without scrolling (`api-011-image-host-surface-*/`). Kitty uses native negative
z-index for images behind text. Sixel and inline have no text z-index: they retain
glyphs over a sampled image background in text cells and real pixels elsewhere.
Kitty and Xterm captures verify text over colored image backgrounds without black
holes. The Kitty retry capture redirects the isolated fixture's stdout to
`/dev/full`, verifies a buffered frame fails, restores stdout and displays the same
frame successfully. A foreground fallback clears the complete footprint of a wide
grapheme when an image overlaps it; fully transparent samples preserve text.

The Surface tests and host captures are editing checks. Remaining platform acceptance
is still pending, so this is not an image-family readiness claim.

### Mixed VDOM composition

`mixed_container`, `from_vdom`, `IntoElement`, and composition macros must preserve
VDOM children, keys, component props, inline styles and supplied event callbacks.
App must deliver the native `Event` payload at measured targets after resize;
`click` means unmodified Enter/Space press or left mouse down/click. Disabled
nodes must not activate, and replacement handlers must replace prior handlers.
Generic key/mouse/focus/paste/custom observers follow native bubbling. Inline
styles use terminal cell lengths and must report invalid declarations as layout
errors instead of silently accepting them. A dropped callback, stale props,
ignored style, or callback on a disabled node falsifies this entry.

Inline declarations accept width/height and their min/max forms (cells, `px`,
`ch`, percentages or `auto`), display, relative/absolute position, cell insets,
flex direction/wrap/grow/shrink/basis, alignment and justification, padding and
margin (one to four cell lengths; margin also accepts `auto`), gap/row-gap/column-gap,
foreground/background palette or hex colors, opacity, z-index, font weight/style,
underline/line-through decoration and overflow per axis. One `px` or `ch` is one
terminal cell. They apply after utility classes, including resolved state classes.
Unsupported properties/values and non-finite lengths return an App layout error.
Inline styles do not change terminal font size or create a separate scroll owner.

The responsive grid is one column below 80 terminal columns and uses its requested
column count at 80 or more, updating after resize. App resolves `sm:`, `md:`,
`lg:` and `xl:` at 40, 80, 120 and 160 columns respectively. A standalone style
parser has no viewport and leaves these variants inactive. Fixed grid/flex helpers
may size to their contents unless the caller gives them a width. Default search
inputs must expose their placeholder and accept editing at 40 and 80 columns.

`VNode::element` delivers `VElementProps` to registered components; it retains
typed values (`get::<T>(name)`) and string attributes. `VNode::component` delivers
its caller-defined typed props unchanged. `VNode::from_element` uses an opaque
VComponent payload and separate children so `to_element` restores the native
control, its factory, props, metadata, focus, style and key. `class`, `style`,
`disabled`, `autofocus` and `id` attributes have native bridge meanings; all
attributes remain available to the component. Event payloads downcast to native
`Event`; supported names are `click`, `key`, `mouse`, `focus`, `paste` and `custom`.
Generic callbacks use normal App bubbling and may be stopped by a consuming
native handler; activation consumes its event after invoking the callback.

### Native dialog and terminal verification

The macOS development run 34491471836 passed six HTTP transport cases and
13 of 14 HTTP App workflows. Its submit-veto/retry timeout also reproduced on
Linux: the test progress marker was underneath the modal backdrop and became
unreadable when the opening fade finished. The marker now has an explicit cell
size and paints above the backdrop while retaining its row in normal layout.
A delayed response exercises the fully opened dialog. Lowering the marker behind
the backdrop reproduces the timeout; restoring it passes the same delayed case.
Logs are `api-011-veto-backdrop-{negative,corrected}.log` in `.cairn/reviews/`.
All 14 HTTP App cases then passed on Linux. Native reruns remain required.

The native platform script also runs TerminalWidget App workflows and installs
the matched ConPTY runtime beside Windows test executables. The Windows case
submits arithmetic through paste and keyboard input at two viewport sizes,
resizes, and requires calculated answers in captured frames. Expected answers do
not occur in submitted commands, so command echo cannot satisfy the check.
This case is pending native execution; the 15 Linux terminal App cases passed.

### Confirmation controls and Escape policy

Confirmation shortcuts ignore key releases and Ctrl/Alt/Meta combinations.
ASCII shortcut letters retain case-insensitive matching, and non-character
shortcuts use their configured key code. App and direct dialog routes share
this matching and the button-result mapping. Standard IDs keep their existing
results, including the distinct `no` result. Custom buttons marked `is_cancel`
return cancellation, including after a resized pointer click.

The dialog `escape_closable` option only controls Escape dismissal. Confirmation,
input and autocomplete buttons remain reachable by Tab and Enter when it is
false. The private modal adapter carries that policy separately from the public
Modal keyboard-navigation option. Autocomplete still handles Escape first to
collapse its suggestions. App tests cover these paths at 32x12 and 60x20.

The Windows development run 34491471836 passed clipboard and ConPTY verification
but exposed two HTTP fixture assumptions. Accepted Windows sockets inherited
nonblocking mode; the bounded fixture now explicitly restores blocking IO before
applying read/write deadlines. Cancelling the HTTP child produced ConnectionReset,
which also proves the connection closed; the test accepts that result or EOF,
and still rejects timeouts and any remaining bytes. The original failure is
`api-011-windows-http-fixture-baseline.log` in `.cairn/reviews/`. Six HTTP transport
cases pass on Linux after the fixture correction; native reruns remain required.

The macOS run 34494618226 passes all six HTTP transport and fourteen HTTP App
cases. Its filesystem fixture failed before invoking FileExplorer: APFS rejected
an invalid UTF-8 filename with EILSEQ. The symlink safety test now runs separately.
A portable test compares Unicode entry identity with the filesystem's own name;
Linux additionally checks raw non-UTF-8 bytes, and Windows will check an unpaired
UTF-16 surrogate. Ten filesystem worker tests pass on Linux. The original macOS
failure is `api-011-macos-filename-fixture-baseline.log` in `.cairn/reviews/`.
The native driver now captures every independent case even after another fails,
requires all to pass before writing its record, and includes image decoding,
image App, platform-image and Surface checks. These additions need native runs.

Confirmation and notification follow-up: the App checks in
`api-011-confirmation-updated-controls.log` and
`api-011-confirmation-position-modes.log` verify changed confirmation content,
button enablement and callbacks, all position modes and explicit render bounds,
including resizing in both directions between 32x12 and 60x20. The four cases in
`api-011-dialog-notifications-ready-controls.log` verify resized Progress Cancel
and Toast Close targets, idle indeterminate progress, and retained Toast updates
followed by expiry. These are editing checks; native dialog-engine acceptance
remains separate and pending.

A new noncancelable Wizard keyboard case failed with Finish still painted after
Enter (`api-011-wizard-keyboard-negative.log`). The retained adapter now keeps
keyboard navigation enabled and passes cancellation policy separately to Modal.
All eleven then-existing Wizard App cases pass in
`api-011-wizard-keyboard-corrected.log`; a twelfth case verifies removal and
reinsertion discard the removed step's edited child
(`api-011-wizard-removed-step.log`). The broader pre-fix App run passed all 392
cases in `api-011-catalog-app-reconciliation.log`; that run did not include the
new Wizard failure case and is not evidence for the correction.

The macOS development run 34500402680 passes all nine filesystem worker cases
and seventeen FileExplorer App workflows after correcting native path fixtures.
It also passes HTTP/HTTPS, nine native Terminal cases, thirteen TerminalWidget
App cases, twelve Image App cases, twelve platform-image cases and seventeen
Surface cases. Image resource cleanup still fails with EPERM, and iTerm2 starts
but does not create the scripted probe window. Neither failure is acceptance.
The Windows run 34497209141 passes the HTTP/HTTPS and image groups but fails
FileExplorer rename, absolute-target symlink copy and TerminalWidget startup.
Its canonical-path assertion failures now have corrected fixtures; directory-relative
rename and path-normalization changes require native reruns. Raw terminal startup
diagnostics have been added to distinguish missing output from parsing or App delivery.

Menu outlines now use the same measured cell-glyph renderer as tables and modals.
An enabled outline reserves one cell on each edge independently of CSS padding;
numeric menu padding includes that edge. Border color classes color the outline
glyphs without replacing the panel background. `show_border: false` and
`border-0` suppress the outline. Glyphs remain hidden from screen readers.
The captured-frame baseline in `api-011-menu-border-negative.log` fails because
the outline is absent. The corrected two-size case also checks zero CSS padding
and distinct border/panel colors (`api-011-menu-border-padding.log`). The earlier
395-case App run passed in `api-011-catalog-current.log`; it predates this outline
repair and does not establish its correctness.

The final local run passes all 396 App cases in
`api-011-catalog-final-local.log`. The first run exposed an autocomplete fixture
race: it clicked Beta as soon as Alpha appeared during a resize animation. It
now waits for Beta to be painted before clicking it; the selection-veto and
single-result assertions remain. The corrected target case passes separately in
`api-011-autocomplete-visible-target.log`. Updated Orca/GNOME Terminal workflows,
including all four menu families, pass at 32x10 and 60x16; regular diagnostic
files are in `api011-orca-menu-border32/` and `api011-orca-menu-border60/`.

The macOS snapshot run 34503199020 passes every native widget group, including
image resource cleanup. Its remaining iTerm host failure is a first-launch
relocation prompt, visible in `api-011-iterm-relocation-baseline/startup-24.png`.
The host driver now extracts the pinned bundle inside its private temporary
`Applications` directory. iTerm 3.7.0 recognizes this directory component in
its [relocation check](https://github.com/gnachman/iTerm2/blob/v3.7.0/ThirdParty/LetsMove/PFMoveApplication.m).
Actual image captures still require a native rerun.

The macOS snapshot run 34505587248 passes all HTTP/HTTPS, filesystem, terminal,
image and Surface groups. iTerm2 now opens a normal shell, but its legacy
AutoLaunch task does not write the probe's startup marker. The owned-window
capture and diagnostics are in `api-011-iterm-autolaunch-baseline/`. The pinned
iTerm source silently ignores legacy AppleScript task errors. The fixture now
uses the pinned version's direct `--command` option, waits for the child's actual
terminal-size marker, and identifies its unique title in a visible window owned
by the launched process. Actual image acceptance still requires native execution.

Wizard's twelve App cases now include an exact authored background-color check
with reduced motion (`api-011-wizard-style.log`). Autocomplete's two-size style
case checks configured input/content backgrounds and bold/underline only on
matching characters, including disabled highlighting. The initial style-test
failure used byte offsets past a Unicode border; it was a fixture error, not a
widget defect. The corrected fixture measures terminal columns. Disabling the
actual match-rendering branch fails at the expected matched cell
(`api-011-autocomplete-highlighting-disabled-negative.log`); normal rendering
passes in `api-011-autocomplete-style.log`.

Autocomplete's combining-grapheme minimum and empty placeholder pass in
`api-011-autocomplete-grapheme-minimum.log`. The HTTP suggestion-object case
now runs with descriptions enabled and disabled at both sizes, preserving
icon/display labels, submitted identity, request headers and query assertions
(`api-011-autocomplete-description-toggle.log`).

Run 34507954693 confirms iTerm's direct launch starts the probe at a real terminal
size. It also opens an extra shell window. The capture driver now gives the
probe a unique OSC window title and requires that title in a window owned by
its child PID. The two owned startup windows are retained in
`api-011-iterm-direct-launch-baseline/`; image capture remains unverified.


### Approved iTerm2 3.7 image limitation

The developer approved `.cairn/escalations/api-011-api-014-api-020.md` on
2026-09-10. Inline-image color accuracy and transparency are unsupported on
this host/version. The API remains available; placement, replacement, movement
and removal still require native evidence. WezTerm inline acceptance retains
its exact color checks. Other hosts and protocols keep their existing contracts.

The iTerm fixture records exact sRGB counts in `pixels.json` and uses separately
recorded dominant-channel regions in `geometry-pixels.json` for geometry only.
`--require-exact-srgb` retains the failing strict diagnostic. Neither region
classification nor successful geometry establishes color or transparency support.
Independent PNG/AppKit evidence remains in
`.cairn/reviews/api-011-iterm-native-reference`; host renderer diagnostics remain
in `.cairn/reviews/api-011-iterm-renderer-diagnostic`.


### Current widget implementation review

All catalog rows now have reviewed App coverage. Current editing checks passed
970 library tests, 399 App workflows and the broader default suite (1,766 passing
tests across 64 targets). After removing the unused Sixel build dependency,
87 image-filtered tests and the HTTP group passed. Hosted run 34531816183 passed
both platforms. The developer's MacBook passed 188 selected tests; Windows 11/MSVC
passed 171 selected tests and the complete ConPTY probe. Its corrected failed-launch
check stayed at 135 handles; a deliberate leak reached 145 and was rejected.

This closes implementation coverage for API-011, not the commitment. Cairn must
still check committed inputs and refresh their native receipts. DialogEngine
lifecycle/results (API-012), dedicated image acceptance (API-014), retained entry
points (API-016), bindings, documentation and residual audit concerns retain their
separate obligations. Historical pending notes above are superseded only for this
widget implementation coverage.
