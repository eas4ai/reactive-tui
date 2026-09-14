# Images and clipboard

Crate modules: `hooks`, `platform`, `widgets`

## Purpose

Image APIs display raster and animated image content in capable terminals.
Clipboard hooks copy and read text through platform clipboard commands.

## Main API

- `Image`, `ImageSource`, `ImageFormat`, `ImageDisplayMode`, `ImageQuality`, and
  `ImageCapabilities` configure image content and output.
- `ImageProcessor` decodes and resizes image frames.
- Protocol and Sixel renderers encode host-terminal output.
- Cell painting provides a fallback when a direct image protocol is not used.
- `SuprTuiBackend::new_with_images` enables terminal image output options.
- `use_clipboard` and `use_simple_clipboard` expose clipboard state and
  operations through hooks.
- Platform image APIs detect and transmit Kitty, Sixel, and iTerm2 forms.

## Basic use

Create an image from a supported source and set its display mode and quality.
Use an image-enabled backend for direct protocols. Use clipboard hooks inside a
component hook scope and handle unavailable clipboard programs as an operation
error.

## Behavior

Image work runs through an owned worker. The processor decodes frames, applies
size and quality settings, and publishes results. The renderer selects direct
protocol output or cell painting from detected capabilities and configuration.
Animated images schedule frame changes and wake the application.

Clipboard operations select a platform command, run it as an owned process,
capture bounded output, and publish completion into hook state.

## Limits

- Direct image output depends on host-terminal protocol support.
- Image dimensions and decoded allocation are bounded by checked entry points.
- Cell fallback has lower visual resolution than a direct protocol.
- Clipboard tools differ by operating system and desktop session and may be
  absent.
- Clipboard and image operations can fail after the UI has requested them.

## Source map

- Image widget exports: [`src/widgets/display/image/mod.rs`](../src/widgets/display/image/mod.rs)
- Image worker: [`src/widgets/display/image/live/worker.rs`](../src/widgets/display/image/live/worker.rs)
- Clipboard hooks: [`src/hooks/clipboard.rs`](../src/hooks/clipboard.rs)
- Platform image support: [`src/platform/image.rs`](../src/platform/image.rs)
- Image behavior tests: [`tests/api_widget_behavior/image.rs`](../tests/api_widget_behavior/image.rs)
- Clipboard platform tests: [`tests/clipboard_platform.rs`](../tests/clipboard_platform.rs)

## Related chapters

- [Display widgets](display-widgets.md)
- [Rendering and backends](rendering-and-backends.md)
- [Reactive state and hooks](reactive-state-and-hooks.md)

[Back to the manual](README.md)
