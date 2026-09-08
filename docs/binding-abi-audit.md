# Binding ABI audit

Status: Declarations reconciled, 2026-09-08

The repaired header and TypeScript loader agree with 210 Rust exports. The
current checks compare signatures, layouts and enum values, then run real C/C++
and TypeScript consumers. Migration guides record every corrected or retired
consumer declaration. See the reconciled result below and the commitment review
for acceptance evidence and the limits of the runtime coverage.

## Historical baseline

The first committed static check found 198 C function declarations, 58 symbols
requested by the TypeScript loader and 194 exported Rust functions. Of those
consumer declarations, 75 C names and 38 TypeScript names were absent from the
shared library. The original headers compiled together as C; that did not
prove matching signatures or layouts. No mismatched native call was executed.

The complete initial inventory is in binding-abi-baseline.json. The failing
receipt is ABI-001/20260908T201846770Z.

## Baseline compatibility distinction

Some missing names are aliases for existing operations, such as the older
element-builder setters. Others describe API families with no compiled native
implementation. The uncompiled dialog module also uses a different interface
from both the C header and TypeScript wrapper, and contains a success-returning
placeholder for an unsupported engine update. Enabling that file alone cannot
establish compatibility. TypeScript component creation/state/event functions are
absent; its builder source also has method/property name collisions.

The recorded compatibility decision uses existing exported Rust functions as the
baseline. The repair restores aliases with implemented behavior and explicitly
retires unsupported consumer declarations. The migration inventories preserve
the old declarations and name replacements or unsupported behavior. No missing
API was replaced with a success-returning stub.

## C symbols missing at the baseline

- `createRenderingContext`
- `destroyRenderingContext`
- `getBufferDims`
- `getRendererSurface`
- `logMessage`
- `rtui_apply_utility_classes`
- `rtui_button`
- `rtui_card`
- `rtui_computed_style_destroy`
- `rtui_dialog_close`
- `rtui_dialog_create_confirmation`
- `rtui_dialog_create_input`
- `rtui_dialog_create_progress`
- `rtui_dialog_create_toast`
- `rtui_dialog_engine_create`
- `rtui_dialog_engine_destroy`
- `rtui_dialog_engine_has_active_dialogs`
- `rtui_dialog_engine_update`
- `rtui_dialog_get_confirmation_result`
- `rtui_dialog_get_input_text`
- `rtui_dialog_hide`
- `rtui_dialog_is_visible`
- `rtui_dialog_set_progress`
- `rtui_dialog_set_progress_message`
- `rtui_dialog_show`
- `rtui_div`
- `rtui_element_builder_child`
- `rtui_element_builder_children`
- `rtui_element_builder_class`
- `rtui_element_builder_key`
- `rtui_element_builder_text`
- `rtui_element_empty`
- `rtui_element_text`
- `rtui_free_string`
- `rtui_h1`
- `rtui_h2`
- `rtui_h3`
- `rtui_p`
- `rtui_primary_button`
- `rtui_renderer_begin_frame`
- `rtui_renderer_clear_with_color`
- `rtui_renderer_draw_rect`
- `rtui_renderer_draw_surface`
- `rtui_renderer_draw_surface_rect`
- `rtui_renderer_draw_text`
- `rtui_renderer_end_frame`
- `rtui_renderer_fill_rect`
- `rtui_renderer_get_dimensions`
- `rtui_span`
- `rtui_style_builder_align_items`
- `rtui_style_builder_background_color`
- `rtui_style_builder_build`
- `rtui_style_builder_color`
- `rtui_style_builder_create`
- `rtui_style_builder_destroy`
- `rtui_style_builder_display`
- `rtui_style_builder_flex_direction`
- `rtui_style_builder_height`
- `rtui_style_builder_justify_content`
- `rtui_style_builder_margin`
- `rtui_style_builder_padding`
- `rtui_style_builder_width`
- `rtui_surface_clear_default`
- `rtui_surface_copy_rect`
- `rtui_surface_resize`
- `rtui_surface_set_text`
- `rtui_terminal_clear`
- `rtui_terminal_enter_raw_mode`
- `rtui_terminal_exit_raw_mode`
- `rtui_terminal_flush`
- `rtui_terminal_set_cursor`
- `rtui_terminal_set_cursor_visible`
- `rtui_terminal_write`
- `textBufferAppendText`
- `writeToBuffer`

## TypeScript loader symbols missing at the baseline

- `rtui_animation_free`
- `rtui_animation_group_add`
- `rtui_animation_group_create`
- `rtui_animation_group_free`
- `rtui_animation_group_is_running`
- `rtui_animation_group_pause`
- `rtui_animation_group_remove`
- `rtui_animation_group_resume`
- `rtui_animation_group_start`
- `rtui_animation_group_stop`
- `rtui_animation_group_update`
- `rtui_animation_is_complete`
- `rtui_animation_is_running`
- `rtui_animation_reset`
- `rtui_animation_resume`
- `rtui_animation_set_progress`
- `rtui_animation_start`
- `rtui_animation_update`
- `rtui_component_create`
- `rtui_component_free`
- `rtui_component_get_state`
- `rtui_component_handle_event`
- `rtui_component_render`
- `rtui_component_set_state`
- `rtui_component_update`
- `rtui_dialog_close`
- `rtui_dialog_create`
- `rtui_dialog_focus`
- `rtui_dialog_free`
- `rtui_dialog_get_result`
- `rtui_dialog_is_visible`
- `rtui_dialog_result_free`
- `rtui_dialog_show`
- `rtui_dialog_show_async`
- `rtui_dialog_update`
- `rtui_terminal_get_size`
- `rtui_terminal_init`
- `rtui_terminal_shutdown`

## Baseline static type audit

All 13 original headers also compiled together as C++ without native invocation. The
baseline JSON includes C typedefs, record fields and enum variant inventories.
Known matching-name mismatches include createRenderer (four header arguments,
two Rust arguments), destroyRenderer (one header argument, three Rust arguments),
render (one header argument versus a renderer and a force flag), and capabilities
(the header colors/size record versus the Rust capability-flag record).
The TypeScript rtui_version return is void although Rust returns a four-u32
record. Its animation constructor uses eight unrelated arguments while Rust
accepts an ID string, duration, easing/loop configuration and an output handle.
These are declaration defects; no attempt was made to invoke them.


## Reconciled result

The canonical native header and TypeScript schema now contain 210 Rust exports:
the original 194 plus 16 compatibility aliases. Independent rustc/clang probes
compare all signatures, 13 record/union layouts and field types, eight enum layouts
and 59 enum values. Koffi's actual loaded signatures, record layouts, field types
and error constants are compared against those compiler results before any consumer
call. A separate snapshot recovered from the original Rust revision protects the
existing native signatures, records and enum values.

The migration inventory covers all 198 original C declarations, 58 original
TypeScript loader declarations, 52 original C type aliases and public declarations
from all 17 original TypeScript modules. All original C type names still compile;
legacy-only types are explicitly distinguished from native records. The invalid
adaptive enum value is not advertised. Current package sources and examples
strictly typecheck; retired modules are removed from generated package output.

Development C/C++ and TypeScript consumer checks pass with real native values,
raw-mode changes and restoration, capabilities canaries, RGB/Unicode cells,
renderer-owned views, legacy builder aliases, consuming children/builders,
owned child clones, null/error results and string release. Nine safe failure
demonstrations and their restored controls are in binding-abi-demonstrations/.
The committed Cairn receipts record passing checks for ABI-001 through ABI-004
and all 30 inherited requirements. The commitment review records the final audit
and production self-assessment.

The preexisting legacy signal destructor mismatch is recorded in the backlog and
migration guidance. No mismatched signal-family call was executed. This ABI
commitment does not claim that every legacy native feature is behaviorally complete.
