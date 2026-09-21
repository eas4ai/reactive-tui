# Rust API remediation

Status: Agreed 2026-09-08
Prefix: API

The developer requested a commitment to remediate all findings in
`docs/api-audit.md`. This specification turns that audit into requirements.
The audit is diagnostic evidence, not an acceptance suite.

Existing working APIs and inherited contracts remain binding. Removing features,
retiring APIs, narrowing promised semantics, or declaring a finding out of scope
requires an explicit developer decision; documentation alone cannot repair broken
behavior. Routine implementation choices remain with the agent. Judged choices
are recorded before implementation. Platform claims require platform evidence;
unavailable verification is reported and escalated, never fabricated.

## Signal ownership

[API-001]
Every legacy and tagged native signal constructor/getter/setter/destructor MUST use matching allocation types and documented ownership. Preserve existing ABI; reject invalid family use where validation is supported without dereferencing an invalid allocation.
Audit mapping: RAPI-01.
Falsifier: a supported typed lifecycle miscasts or leaks an allocation, or consumer ownership becomes ambiguous.
Mechanism: typed Rust/C lifecycle probes with a suitable memory-safety checker.

## Component expansion and identity

[API-002]
App MUST recursively render registered components before layout and painting, retain one live instance per stable key, deliver prop changes, and unmount removed instances exactly once. Unknown component names MUST have defined observable behavior.
Audit mapping: RAPI-02.
Falsifier: a nested component fails to paint, keyed reorder loses state, props remain stale, or lifecycle callbacks duplicate.
Mechanism: real App/SuprTUI captured-frame and lifecycle tests.

## Hook state and memoization

[API-003]
Generated components MUST reset hook indexing for each render and retain existing slots without growth. Memo values MUST follow documented dependency semantics. Invalid hook ordering MUST have defined behavior.
Audit mapping: RAPI-03.
Falsifier: repeated renders allocate new state slots, state resets, or a changed dependency returns a stale memo.
Mechanism: repeated generated-component renders with state, slot-count and memo assertions.

## Effects context and runtime

[API-004]
App MUST own component effect/timer lifecycle. Effects MUST run and clean up at documented dependency changes and unmount. Removed components MUST leave no runnable timer or effect. Descendants MUST inherit context with scoped overrides and isolation between Apps.
Audit mapping: RAPI-03.
Falsifier: cleanup runs immediately instead of at its lifecycle boundary, work survives removal, parent context is absent, or Apps share component context.
Mechanism: mount/update/unmount tests with controlled scheduler and nested providers.

## Callbacks and event routing

[API-005]
Builder callbacks MUST be retained and invoked through App keyboard/mouse routing exactly once for the intended target. Event bounds and ordering MUST agree with the painted layout after resize and updates.
Audit mapping: RAPI-04.
Falsifier: a supplied handler is discarded, an event hits the wrong node, or resized bounds are stale.
Mechanism: App input-to-callback tests using captured layout and frames.

## Stable focus and traps

[API-006]
Focus identity MUST survive keyed redraw/reorder. Tab and reverse Tab MUST work with and without traps. Nested dialogs MUST autofocus, confine focus and restore it on close or removal.
Audit mapping: RAPI-04.
Falsifier: focus changes solely because of redraw, navigation stalls, or trapping/restoration targets a different node.
Mechanism: keyboard sequences through real rendered trees including nested and removed traps.

## Unicode editor positions

[API-007]
Both plain and syntax editors MUST consistently convert text positions and terminal display columns. Insertion, deletion, movement and selection MUST respect documented grapheme boundaries and multiline behavior for ASCII, CJK, emoji and combining text.
Audit mapping: RAPI-05.
Falsifier: mixed insert_char/insert_text operations corrupt cursor position, backspace leaves the inserted grapheme, or selection/display bounds split a grapheme.
Mechanism: editor operation and rendered-cell tests across Unicode and multiline cases.

## Bounded clipboard operations

[API-008]
Clipboard subprocess operations MUST have bounded deadlines, propagate nonzero exits and unavailable-backend errors, and reap owned children on timeout/cancellation. Claimed desktop backends MUST have explicit verification coverage.
Audit mapping: RAPI-06.
Falsifier: a stalled tool blocks indefinitely, a failed or unavailable copy reports success, or a child survives timeout.
Mechanism: isolated subprocess fixtures plus recorded integration checks for each claimed backend.

## Dynamic styling and text tokens

[API-009]
Focus/hover/disabled variants MUST depend on actual node state. Accepted typography and overflow tokens MUST change rendered output according to their documented terminal semantics.
Audit mapping: RAPI-07.
Falsifier: unfocused styling applies unconditionally, uppercase/truncation is accepted without effect, or a state change fails to repaint.
Mechanism: App state transitions and independent expected cell/text output.

## Gradient and animation painting

[API-010]
Supported gradient and animation properties MUST survive component expansion and layout and affect the supported painter. Terminal approximations MUST be explicit and tested.
Audit mapping: RAPI-07.
Falsifier: properties disappear in the bridge or configured changes never affect frames.
Mechanism: captured intermediate frames with gradients and scheduled property changes.

## Widget behavior and builders

[API-011]
An inventory MUST enumerate public widgets/builders and advertised controls. Existing advertised behavior MUST work through App, including input, checkbox/select, tables/trees, menus, scrolling, selection, callbacks and disabled/empty states. Builders MUST produce functional controls rather than descriptive placeholders.
Audit mapping: RAPI-08.
Falsifier: a catalog entry lacks behavioral coverage, a builder paints a description instead of its control, or interaction relies on fixed/default bounds.
Mechanism: per-widget acceptance matrix and App workflows at multiple viewport sizes.

The acceptance matrix below is the authoritative obligation list this requirement names. It moved here verbatim from docs/widget-acceptance.md under the approved DOC-001 retention (escalation doc-001); only its home changed. Rows are acceptance obligations, not readiness claims; a row marked PENDING makes the mechanism fail. The deleted file's dated run narratives stay in git history.

<!-- WIDGET-ACCEPTANCE-START -->
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
<!-- WIDGET-ACCEPTANCE-END -->

Screen-reader guarantee (approved 2026-09-09, escalation api-011-api-018): verify
Orca with GNOME Terminal on Linux, including actual delivery of labels, roles,
focus and state changes. Preserve public label APIs. Other terminal/screen-reader
pairs are explicitly unverified; their existing rendering/input guarantees remain.
Retained metadata or painted text alone is not screen-reader acceptance evidence.

Reader-client lifetime guarantee (approved 2026-09-13, escalation
api-011-api-020): the repository-owned Linux fixture MUST build the official
libatspi 2.60.6 source archive pinned by its published SHA-256 and load the
corrected library only inside the fixture. The installed desktop library MUST
remain unchanged. An independent reentrant state-query reproducer MUST expose
the original `AtspiStateSet` use-after-free under memory checking and MUST pass
for both `contains` and `get_states` after the repair, with balanced object
lifetime and an unchanged dynamic symbol surface. The unchanged complete Orca
workflow matrix MUST pass with the isolated repaired library. A clean corrected
run without the original memory failure, use of the system library by mistake,
or a reduced reader workflow falsifies this guarantee.

## Dialog lifecycle and results

[API-012]
Dialogs MUST paint, accept events, update and deliver completion/cancellation results synchronously or asynchronously as advertised. Stacking MUST honor z-order and limits. Closing MUST emit documented events, release resources and restore focus.
Audit mapping: RAPI-09.
Falsifier: a result is discarded, async mode has no completion, z-order is ignored, or close leaves focus/resources behind.
Mechanism: nested dialog workflows including limits, cancellation, result delivery and cleanup.

## Animation semantics and screens

[API-013]
Typed/untyped keyframes MUST preserve values and interpolate according to documented type/easing semantics. Relative values MUST use actual current properties. Active screens MUST receive input. Transition progress MUST visibly affect output.
Audit mapping: RAPI-10.
Falsifier: a numeric midpoint steps to the previous endpoint, conversion substitutes defaults, relative values start from zero incorrectly, or fade/input is ineffective.
Mechanism: deterministic clock tests plus intermediate rendered screen frames and input delivery.

## Image decoding and terminal output

[API-014]
Advertised file and encoded-memory image paths MUST decode actual image data and share correct placement, clipping and cleanup semantics. Claimed graphics protocols MUST emit valid output. Fallback MUST render the decoded image. URL loading support MUST be explicitly decided before implementation.
Audit mapping: RAPI-11.
Falsifier: a file yields a synthetic pattern, encoded bytes are mistaken for pixels, a claimed protocol only paints fallback cells, or removal leaves stale placement.
Mechanism: known-image pixel comparisons, protocol captures and integration evidence on each claimed host/protocol.

iTerm2 3.7 color/transparency limit (approved 2026-09-10, escalation
api-011-api-014-api-020): inline-image color accuracy and transparency are
unsupported on this host/version. Preserve the public inline-image APIs and
all other host requirements, including placement, update and removal. Retain
its failing color evidence. WezTerm remains the verified inline-protocol host
for color acceptance. This exception does not apply to other hosts or protocols.

Kitty host repair (approved 2026-09-12, escalation api-014-api-020): this
commitment includes the narrow repair that prepares image placements before
choosing Kitty's image-layer paint path. Build and verify an isolated, pinned
host with the existing image checks and an independent sender that exposes
the stale placement-count decision. Preserve the failing stock-host evidence.
The corrected host MUST show new placements without an extra repaint request
or retransmission and retain update and removal behavior. Record the build's
provenance and required host version/patch; do not claim that this updates
users' unmodified Kitty installations. All other API-014 requirements remain.

## Feature configurations

[API-015]
The no-default-features configuration MUST compile and have defined runtime behavior. Optional dependencies MUST be correctly gated or made mandatory by a recorded compatibility decision. The supported feature matrix MUST pass build and applicable behavior checks.
Audit mapping: RAPI-12.
Falsifier: no-default compilation references an absent optional dependency or a claimed feature combination fails.
Mechanism: locked feature-matrix builds and applicable behavior tests.

## Backend and terminal entry points

[API-016]
Public application/backend/terminal entry points MUST have a documented supported route, with functional adapters or an explicitly approved migration. Retained legacy App paths MUST initialize and update prior render state correctly without synthetic patches. Native App construction MUST expose the recovered rendering path.
Audit mapping: RAPI-13.
Falsifier: a retained entry point repeatedly takes first-render behavior, uses fake patches, fails updates/restoration, or has no functional supported route.
Mechanism: Rust and native entry-point workflows with update, error and terminal-restoration assertions.

## Complete native binding access

[API-017]
After native behavior works, editor/layout/dialog APIs and a stateful foreign component/event bridge MUST be exposed through audited C and TypeScript interfaces. Define callback lifetime, reentry, prop/state updates, errors and ownership; preserve existing ABI guarantees.
Audit mapping: RAPI-14.
Falsifier: a required family remains excluded or a consumer cannot paint, interact, observe state/results and clean up safely.
Mechanism: compiled C and TypeScript end-to-end consumers plus independent ABI/layout checks.

## Accurate public API documentation

[API-018]
Publish a complete supported-API matrix linked to behavior evidence and limits. Public Markdown APIs MUST be visible consistently during documentation builds. Documented examples MUST compile. Props validation MUST enforce specified constraints or have an explicitly approved narrower contract.
Audit mapping: RAPI-15.
Falsifier: readiness claims exceed evidence, docs hide public APIs, examples fail, or validation accepts every value despite promised constraints.
Mechanism: documentation/example builds, invalid/valid Props cases and evidence-linked inventory review.

Props contract (approved 2026-09-12, escalation api-018-api-020): keep
`#[derive(Props)]`, fluent builders and caller-invoked `validate() -> bool`.
`#[prop(validate = rule)]` names a predicate receiving a shared reference to its
field. Validation succeeds only if every declared rule succeeds; fields without
rules add no constraints. Construction and App mounting do not call validation.
Reject bare or malformed validation annotations and unsupported prop options.
`#[prop(optional)]` requires an explicitly written `Option<T>` and defaults to
`None`; the derive does not rewrite fields. Preserve documented defaults and
builders, and migrate existing bare validation annotations to named rules.

## Residual audit concerns

[API-019]
The audit coverage table concerns MUST be resolved or verified explicitly: gesture hooks, ui::Updater dispatch, theme propagation and multi-App isolation, Markdown/syntax integration and bounded large-input behavior, editor undo/selection claims, and claimed platform/legacy terminal behavior. Each MUST have an inventory entry with a concrete contract and falsifier before implementation.
Audit mapping: RAPI-03, RAPI-15 and coverage table.
Falsifier: a named concern disappears from the inventory or an advertised operation remains a marker/no-op without approved contract change.
Mechanism: focused behavior checks and final cross-reference review of the complete audit.

The inventory below is the authoritative entry list this requirement names. It moved here verbatim from docs/residual-api-inventory.md under the approved DOC-001 retention (escalation doc-001); only its home changed.

<!-- RESIDUAL-INVENTORY-START -->
| Concern | Required contract | Falsifier | State |
| --- | --- | --- | --- |
| Hover, drag, drag-and-drop, mouse position, clicks, long press, swipe and wheel hooks | App-routed component events update retained hook state; component-local coordinates, long-press and drag thresholds, wheel deltas, keyed handle/drop-zone identifiers and `allow_drag_outside` are respected; component removal unregisters every owned hook | An advertised hook remains a default signal, an option is ignored, a sibling receives another component's state, or a registration survives its owner | App route, option failure and owner-removal checks wired into API-019 |
| Public reference hooks | References retain identity/value through component renders without adding redraws; callback updates are reentrant and cleanup has explicit ownership; local references retain arbitrary non-Send values in the approved with_local_hooks scope, supplied by App::run; foreign cleanup waits for creator sweep or scope exit | Rerender replaces stored values, callback reentry deadlocks, or a local reference crosses threads | Shared and scoped local-reference positive, reentry, ownership and cleanup checks pass in API-019 |
| ui::Updater | Registered updates reach the intended state/component and preserve defined ordering and ownership | Updater remains a marker or an update is discarded | App-owned dispatch and positive/violating ownership checks pass in API-019 |
| Theme propagation | Public `Theme` parents supply inherited variables; child overrides and later mutation resolve immediately; independent Theme instances remain isolated; CSS utility resolution consumes the selected Theme | An inherited or changed variable is stale, CSS ignores it, or an independent Theme sees another instance's value | Public Theme inheritance/isolation and CSS resolution check wired into API-019; no App theme-provider API is claimed |
| Performance context | Each App owns metrics and mode requests through its explicit handle and inherited hooks; standalone globals remain separate; completed timing, mode reporting, stable slots and cleanup follow the approved migration | Another App changes its metrics or requests, mode/timing is inaccurate, or publication causes idle redraws | App ownership, cross-App isolation, stable-slot and cleanup checks pass in API-019 |
| Markdown/Lumis integration | Parsed Markdown reaches styled output; fenced code uses Lumis when enabled and retains the ordinary code style when disabled; checked entry points report conversion errors | Conversion compiles but loses rendered content, ignores the highlighting option, or drops styles | Enabled/disabled fenced-code behavior check wired into API-019 |
| Large Markdown/syntax input | Checked Markdown and syntax entry points reject sources larger than 1 MiB with an actionable error before parsing or highlighting | An oversized source enters the parser/highlighter, hangs, overflows, or allocates output without the documented bound | Oversized Markdown and syntax failure check wired into API-019 |
| Editor undo/selection | `TextEditor` and `SyntaxEditor` replace whole selected graphemes, move on valid Unicode boundaries and keep painted selection aligned with text offsets; these public editors expose no undo/redo operation | Mixed Unicode selection replacement corrupts text or a cursor/selection lands inside an invalid boundary | Existing plain/syntax Unicode selection checks mapped into API-019; undo/redo is not a shipped contract |
| Unix input worker | Raw and parsed async input keep every descriptor owned and queues bounded; dropping the receiver or final session owner stops and joins owned workers, including while idle, before descriptor reuse | A worker survives consumer/session removal, reads a reused descriptor or queues without a bound | Receiver lifetime, descriptor-reuse, idle cleanup and backpressure checks pass in API-019 |
| SIGWINCH ownership | The signal action performs only a self-pipe write; an owned dispatcher invokes callbacks outside signal context; the final reset unregisters library delivery while preserving another handler | A callback runs while the signaling thread holds its mutex, another handler is lost, or library callbacks continue after cleanup | Prior-handler, mutex and cleanup falsifier wired into API-019 |
| Legacy input parsing | Streaming parsing retains split UTF-8, escape and mouse sequences; lone Escape has an explicit flush; buffered incomplete input is capped at 4096 bytes; native buttons, modifiers, drag and wheel direction survive translation | A supported input becomes Unknown, is discarded/corrupted at a read boundary, or an incomplete sequence grows without bound | Split-input, overflow and mouse-detail checks wired into API-019 |
| DebugBackend boundaries | Checked resize accepts empty/bounded dimensions and rejects axis overflow, multiplication overflow and more than 262144 cells without changing the last valid screen; the infallible trait method preserves that screen on rejection | Dimensions truncate to `u16`, an invalid allocation occurs, or rejected resize leaves stale dimensions with a new surface | Positive and oversized-resize checks wired into API-019 |
| Raw-mode ownership | Library sessions count raw-mode owners, disable only after the final library owner, and preserve raw mode established by the caller | One owner's drop disrupts another live session or the library disables caller-owned raw mode | Ownership state-machine check wired into API-019 |
| Public RenderTree | Fragments, custom nodes, 2048-wide trees and 256-deep element trees retain keys/output; conversion, indexing, element reconstruction and destruction use explicit work stacks | Advertised nodes disappear, deep conversion/reconstruction/drop overflows, or wide lookup loses a node | Public depth/width/custom/fragment check wired into API-019 |
| Nested legacy events | Input reaches retained nested targets exactly once in capture/target/bubble order and removed handlers release their captures | Root-only dispatch loses a nested target, changes routing order, duplicates activation or calls a stale handler | Existing positive, ordering and removal checks mapped into API-019 |
| Legacy backend test reachability | Key, focus, paste, resize and mouse mapping tests are registered and executed by the Rust test harness | A named mapping test is missing from discovery or an intentional mismatch does not fail its executed test | Discovery and execution remain explicit API-019 checker steps |
| Transition integration metadata | `TransitionConfig` retains animation ID, custom properties and hardware preference as compatibility metadata; the terminal transition painter ignores them and derives progress only from duration/easing | Documentation presents a GPU or animation-hook bridge, or metadata changes terminal painting | Contract decision and metadata-invariance check wired into API-019 |
| CSS property diagnostics | Checked animation conversion requires a named property, rejects incompatible units/value loss and reports unknown style properties | Unknown/mismatched properties silently succeed despite the promised diagnostic | Internal error and public checked-conversion checks mapped into API-019 |
| Full claimed native platform surface | Every retained macOS/Windows/Unix path is tied to native evidence or an explicitly approved scope choice; reconcile the four recorded Windows warnings and six FFI warnings against current diagnostics | A platform claim relies only on Linux compilation or an untested placeholder, or a recorded diagnostic disappears without review | Current Linux checks and hash-verified macOS/Windows native records pass API-019; recorded diagnostics are reconciled |
| Image capture timeout ownership | The actual check runner reaps its private terminal/Xvfb descendants on timeout and cancellation, including separate sessions | The driver dies but an owned host or Xvfb process survives | API-020 supervises each private session and checks normal and forced-timeout cleanup with no surviving group member |

API-018 repaired the responsive macro's compile failure and discarded breakpoint
values under the API-009 style contract, including boundary, resize and failure
tests. No concern in this inventory is deferred.
<!-- RESIDUAL-INVENTORY-END -->

Unix async input ownership (approved 2026-09-12, escalation api-019-api-020):
`UnixTty::spawn_input_thread` and `DirectTty::start_async_events` return an owned
`platform::InputReceiver<T>` instead of the standard library's concrete receiver.
Dropping the receiver or final terminal owner MUST stop and join its input work,
including idle and full-queue waits. Queues MUST be bounded and readers MUST NOT
access closed or reused descriptors. Preserve ordinary receive methods and iteration
with standard receive errors. Explicit standard receiver/iterator types require
documented migration. This approval does not weaken other platform/input contracts.

Local reference scopes (approved 2026-09-13, escalation api-004-api-019-api-020):
`hooks::with_local_hooks` MUST own retained arbitrary non-Send local values for
related synchronous renders. `App::run` supplies a scope or reuses the caller's
active scope. Manual renders using local hooks require the wrapper. Hooks remain
Send + Sync and local handles remain non-Send. Local owners MUST reject missing,
different-thread, different-scope or expired-scope use. Same-thread cleanup releases
retained local shares; foreign-thread cleanup defers their destruction until the
creator next uses its local scope or exits it. Scope exit, including unwinding,
MUST release retained shares; escaped LocalRef handles retain ordinary ownership.
App cleanup MUST finish inside its scope and MUST NOT close another owner's scope.

App performance ownership (approved 2026-09-13, escalation
api-004-api-016-api-019-api-020): each App MUST expose its own PerformanceContext
handle and provide it to component hooks with scoped overrides. Mode requests
MUST wake and affect only that App, coalesce with bounded storage, and become
inert after App exit. Snapshots MUST report the selected mode and completed-frame
timing. Keep the public context fields and ordinary hook call forms. Legacy
global context and mode functions serve standalone callers only; Apps MUST NOT
publish into or consume that state. External global callers must migrate to the
App handle or component hooks. Optional context changes MUST preserve hook slots.

## Regression and closure

[API-020]
Every inherited requirement MUST retain current passing evidence. Final review MUST reconcile every audit finding and named subcase with implementation, tests and any explicit developer-approved contract change. Mechanisms MUST demonstrate safe violating and corrected cases. Historical defect-confirming probes MUST not be counted as acceptance passes.
Audit mapping: RAPI-01 through RAPI-15.
Falsifier: an inherited check regresses, an audit item is silently deferred, coverage is weakened, or a defect-confirming assertion is counted as correctness.
Mechanism: all inherited mechanisms and independent final scope/behavior review.

## Approved Updater migration

Approved 2026-09-13, escalation api-019-api-020-2: ui::Updater MUST require
an update method. Existing empty implementations must implement that method.
Each App owns its registered updaters. Request handles wake only their App,
coalesce repeated pending requests, and dispatch in registration order before
rendering on the App thread. Requests during callbacks run on a later turn.
Removed registrations and closed Apps make old handles inert. Callback errors
propagate through App cleanup. Preserve bounded pending storage per registration.

## Approved gesture coordinate units

Approved 2026-09-13, escalation api-019-api-020-3: preserve both Position
variants and hook signatures. Gesture distances and thresholds MUST use cells
for Cell input and pixels for Pixel input. Velocity MUST use the corresponding
units per second. A threshold of 5 means five cells or five pixels. Restart
a gesture when its coordinate units change; never subtract cells from pixels.
