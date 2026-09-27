# Input widgets

Crate modules: `widgets`

Widget module: `input`

## Purpose

Input widgets collect text, Boolean choices, one choice from a set, or a value
from a numeric range.

## Main API

- `TextInput`, `TextInputProps`, and `TextInputState` manage editable text,
  cursor position, selection, placeholder text, password display, and callbacks.
- `Checkbox`, `CheckboxProps`, and `CheckboxState` manage a Boolean value.
- `RadioButton` and its radio types manage one option. Named radio groups share
  selection by group name.
- `Select`, `SelectOption`, `SelectProps`, and `SelectState` manage a list and an
  open selection panel.
- `Slider`, `SliderProps`, and `SliderState` manage bounded numeric input.
- Each family exports a builder or is available through builder helpers.

## Basic use

Create props, set controlled or initial state as required by the widget, and
return the widget element from a root or component. Register change and submit
callbacks in props or through the matching builder.

## Behavior

Input widgets receive routed key and mouse events. They update retained widget
state, invoke callbacks once for accepted actions, and request redraws when the
visible state changes. Text input uses grapheme-aware cursor and selection data
and exposes accessible text without revealing password content.

Select and radio controls keep selection within the available options. Sliders
clamp values to their range and step. Disabled controls remain visible but do
not accept actions.

## Limits

- Controlled values must be updated by the owning application after callbacks.
- Empty option lists and zero-size layouts do not provide a selectable item.
- Text positions are based on Unicode text boundaries, not raw terminal bytes.
- Password mode changes exposed and painted text; it does not encrypt the
  stored value.

## Source map

- Input exports: [`src/widgets/input/mod.rs`](../src/widgets/input/mod.rs)
- Text input implementation: [`src/widgets/input/text_input.rs`](../src/widgets/input/text_input.rs)
- Select implementation: [`src/widgets/input/select.rs`](../src/widgets/input/select.rs)
- Input widget tests: [`tests/api_widget_behavior/input.rs`](../tests/api_widget_behavior/input.rs)
- Select tests: [`tests/api_widget_behavior/select.rs`](../tests/api_widget_behavior/select.rs)

## Related chapters

- [Events, focus, and input](events-focus-and-input.md)
- [Layout, style, and themes](layout-style-and-themes.md)
- [Accessibility](accessibility.md)

[Back to the manual](README.md)
