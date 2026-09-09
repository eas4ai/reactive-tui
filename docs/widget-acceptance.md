# Widget acceptance inventory

API-011 covers both `widgets::*` component/props builders and `builder::*`
Element builders, including public submodule paths and convenience macros.
The following rows are acceptance obligations, not readiness claims. PENDING
rows make the mechanism fail. A passing unit test or a painted description
does not establish an App workflow. Each interactive row needs keyboard and
mouse input at two viewport sizes, resized hit bounds, disabled/empty cases,
state retention and the advertised callbacks. Display-only rows need actual
content and layout assertions at two sizes. Extra advertised options discovered
during source review are added before their implementation.

| Public family and builder routes | Advertised controls and required observations | App evidence |
| --- | --- | --- |
| Core ElementBuilder; button/input, primary/secondary/danger buttons; macros | Children, text, classes, focus, activation once, disabled state; core input edits | PENDING |
| TextInput; input::TextInputBuilder; builder::text_input and macro | Value, placeholder, password/email/number modes, readonly/disabled/max length, Unicode editing, selection, history, suggestions, validation and callbacks | PENDING; initial editable builder probe in api_widget_behavior |
| Checkbox; input::CheckboxBuilder; builder::checkbox and macro | Toggle, indeterminate transition, label, disabled, change callback, hover/focus | PENDING; initial builder toggle probe in api_widget_behavior |
| Select; input::SelectBuilder; builder::select and macro | Open/close, option navigation, single/multiple selection, disabled options/control, search, empty options, change callback | PENDING; initial builder selection probe in api_widget_behavior |
| RadioButton; input::RadioButtonBuilder; specialized::RadioButtonBuilder | Single-choice group, selection, disabled, label/value, change callback | PENDING |
| Slider; input::SliderBuilder; specialized::SliderBuilder | Range, step, keyboard/mouse position, disabled, label, change callback, invalid/degenerate bounds | PENDING |
| Table and TableProps | Actual columns/rows, sort, selection and multi-selection, row actions, scrolling, resizing, empty data | PENDING |
| DataTable; builder::data_table and macro | Table behavior plus all filter types, global search, pagination, column visibility, export callbacks, virtual scrolling and each advertised panel | PENDING |
| Tree; display::TreeBuilder; specialized::TreeBuilder; builder::tree | Hierarchy, expand/collapse, selection/multi-selection, checking, icons/lines, node actions, scrolling and empty root | PENDING |
| FileExplorer; display::FileExplorerBuilder; builder::file_explorer and convenience variants | Directory navigation, view modes, filtering, sorting, selection, file operations and callbacks using disposable fixtures | PENDING |
| Chart; ChartsBuilder; builder::chart and macro | Each chart type, series values, axes, legend, styles, updates, empty data and viewport bounds | PENDING |
| ProgressBar; display::ProgressBarBuilder; builder::progress_bar and macro | Value/range, horizontal/vertical, formatting, indeterminate animation and updates | PENDING |
| Popover; builder::popover | Trigger, placement, clipping, open/close, content, callback and focus behavior | PENDING |
| Modal; builder::modal | Content, size/position, backdrop, dismissal, disabled close behavior and callbacks; shared engine lifecycle also API-012 | PENDING |
| ScrollView; layout::ScrollViewBuilder; specialized::ScrollViewBuilder | Content, horizontal/vertical offsets, wheel/keyboard, scrollbars, clipping and resized viewport | PENDING |
| Stack; layout::StackBuilder; specialized::StackBuilder | Children, direction, alignment, justification, spacing, padding and classes | PENDING |
| Tabs; layout::TabsBuilder; builder::tabs and macro | Active panel, keyboard activation modes, positions/orientation, disabled tabs, close callbacks and empty tabs | PENDING |
| Accordion; layout::AccordionBuilder; builder::accordion and convenience variants | Single/multiple expansion, keyboard/mouse headers, content, disabled sections, callbacks and empty sections | PENDING |
| Breadcrumb; layout::BreadcrumbBuilder; builder::breadcrumb and convenience variants | Navigation callbacks, current/nonclickable items, overflow strategies, separators, icons and empty path | PENDING |
| MenuBar; menu::MenuBarBuilder; builder::menubar | Dropdowns, nested actions, shortcuts, checkbox/radio entries, separators, enabled/visible flags, selection/dropdown callbacks | PENDING |
| ContextMenu; builder::context_menu | Actual items, trigger position, viewport constraints, nested navigation, actions, close and disabled/empty entries | PENDING |
| PopupMenu; builder::popup_menu | Placement, open/close, navigation, item actions, callback and disabled/empty entries | PENDING |
| DialogMenu; builder::dialog_menu | Each menu type, selection, confirmation/cancellation, callbacks and empty/disabled entries | PENDING |
| DialogBuilder; specialized::DialogBuilder; dialog_builders; builder::dialog | Functional content and control construction; completion, stacking, async and focus lifecycle additionally API-012 | PENDING |
| ConfirmationDialog and builders | Buttons and configured actions; result delivery additionally API-012 | PENDING |
| InputDialog and builders | Editing, password, validation, submit/cancel callbacks; result delivery additionally API-012 | PENDING |
| AutocompleteDialog and builders | Suggestions, filtering, navigation, selection/change callbacks; results additionally API-012 | PENDING |
| ProgressDialog and builders | Updates, cancellation, labels and progress; lifecycle additionally API-012 | PENDING |
| WizardDialog and builders | Steps, navigation, validators, completion callbacks; lifecycle additionally API-012 | PENDING |
| Toast and builders/macros | Actual message/type/position, timeout and dismissal; lifecycle additionally API-012 | PENDING |
| Image; specialized::ImageBuilder; builder::image | Functional source/placement construction; decoding, protocols and URL decision additionally API-014 | PENDING |
| TerminalWidget and TerminalProps | Content, input, resize, session ownership and cleanup; retained terminal entry path additionally API-016 | PENDING |
| Core/layout convenience builders; MixedElementBuilder/from_vdom; macros | Real child content, layout, style and event preservation across every public construction route | PENDING |

The overlapping requirements above do not remove these entries from API-011.
Their final checks must agree with the dedicated lifecycle/protocol/entry-point
mechanisms. All remain inside the current remediation commitment.
