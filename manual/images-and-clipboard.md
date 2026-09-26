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
- `Blitter`, `set_image_blitter` and `image_blitter` choose the block glyphs
  that cell fallback draws an image with.
- `SuprTuiBackend::new_with_images` enables terminal image output options.
- `use_clipboard` and `use_simple_clipboard` expose clipboard state and
  operations through hooks.
- Platform image APIs detect and transmit Kitty, Sixel, and iTerm2 forms.

## Basic use

Create an image from a supported source and set its display mode and quality.
Use an image-enabled backend for direct protocols. Use clipboard hooks inside a
component hook scope and handle unavailable clipboard programs as an operation
error.

## Image widget

`image()` builds the image widget. `.source_file(path)`,
`.source_raw_bytes(bytes, width, height, format)`, `.source_base64(data)` or
`.source_url(url)` gives it pixels; `.display_mode(mode)`, `.quality(quality)`,
`.format(format)` and `.class(classes)` configure it before `.build()`.

- It fills the rectangle its parent allots unless its classes size it, and
  fits the picture inside that rectangle with its aspect ratio kept.
- After a resize it draws the picture again at the new size. Until the image
  worker has drawn it, the image area is empty and its node is marked busy;
  it never shows the cells drawn for the old size. The worker draws every
  form of the picture, ASCII art included, so none of that work runs on the
  App's thread.
- Its screen-reader node has the image role, the image's fallback text as
  its label, and a description of its state: `Loading image` while the
  worker prepares it, which also marks the node busy; its size in pixels once
  decoded; or the error when loading failed.
- It has no pointer actions of its own: its content is inert, so it needs no
  keyboard equivalent.
- The only colors it draws are the picture's own. Where a chafa or viu
  rendering leaves the terminal's default color, the cell takes the active
  theme's foreground or background.

## Behavior

Image work runs through an owned worker. The processor decodes frames, applies
size and quality settings, and publishes results. The renderer selects direct
protocol output or cell painting from detected capabilities and configuration.
Animated images schedule frame changes and wake the application.

In `Auto` mode without a direct protocol, an image falls back to an installed
Chafa, then Viu, and otherwise to block glyphs. When the application or the
`REACTIVE_TUI_BLITTER` environment variable names a blitter, `Auto` draws with
that blitter instead of an external tool, which would ignore the choice. Each cell shows the split of
its pixels into two colors with the least color error, and a transparent pixel
keeps what is below the cell. The blitter is chosen by tier: ASCII when the
terminal answers that it has no Unicode, octant on kitty, Ghostty and foot,
quadrant on Apple Terminal and VS Code, half blocks on the Linux console, and
sextant on any other terminal. `set_image_blitter` and the
`REACTIVE_TUI_BLITTER` environment variable (`braille`, `octant`, `sextant`,
`quadrant`, `half-block` or `ascii`) replace that choice; the environment
variable wins. `AsciiArt` keeps the character ramp.

Clipboard operations select a platform command, run it as an owned process,
capture bounded output, and publish completion into hook state.

## Limits

- Direct image output depends on host-terminal protocol support.
- Encoded image input is limited to 64 MiB. Static decoding has one 256 MiB
  total memory budget for the retained encoded input, decoder allocations, and
  any required RGBA output copy. Animated GIF storage is limited to 256 MiB and
  4,096 frames.
- Chafa and Viu receive `--` before the image path, so a relative path beginning
  with a hyphen remains image data instead of becoming a command option.
- Cell fallback has lower visual resolution than a direct protocol.
- No terminal reports which block glyphs its font draws. The per-terminal
  table is a judgment from each terminal's identity; a font without the chosen
  glyphs shows empty boxes until `REACTIVE_TUI_BLITTER` names a lower tier.
- Clipboard tools differ by operating system and desktop session and may be
  absent.
- Clipboard and image operations can fail after the UI has requested them.

## Source map

- Image widget exports: [`src/widgets/display/image/mod.rs`](../src/widgets/display/image/mod.rs)
- Image worker: [`src/widgets/display/image/live/worker.rs`](../src/widgets/display/image/live/worker.rs)
- Block fallback blitters: [`crates/reactive-tui-suprtui/src/blit.rs`](../crates/reactive-tui-suprtui/src/blit.rs)
- Clipboard hooks: [`src/hooks/clipboard.rs`](../src/hooks/clipboard.rs)
- Platform image support: [`src/platform/image.rs`](../src/platform/image.rs)
- Image behavior tests: [`tests/api_widget_behavior/image.rs`](../tests/api_widget_behavior/image.rs)
- Clipboard platform tests: [`tests/clipboard_platform.rs`](../tests/clipboard_platform.rs)

## Related chapters

- [Display widgets](display-widgets.md)
- [Rendering and backends](rendering-and-backends.md)
- [Reactive state and hooks](reactive-state-and-hooks.md)

[Back to the manual](README.md)
