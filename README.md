# Reactive-TUI

Reactive-TUI builds Rust terminal applications from retained components, reactive
state and CSS-like layout. App paints complete grapheme-aware frames through the
locally maintained SuprTUI renderer. Crossterm supplies host input and raw mode.

The [supported-API matrix](docs/supported-api.md) lists behavior checks, manual
adapters and platform limits. API remediation is still in progress: documentation
builds do not certify the remaining gesture, theme, Markdown integration and
legacy-platform review. Historical findings remain in [the audit](docs/api-audit.md).

## Run the current checkout

```sh
cargo run --locked --example suprtui_counter
```

Space changes the counter; Escape or Ctrl+C quits. For an application depending on
this checkout, use a Cargo path dependency pointing to the repository. The source
version is 0.0.7; these recovery changes do not establish what a published package
contains.

```toml
[dependencies]
reactive-tui = { path = "../reactive-tui" }
```

```rust,no_run
use reactive_tui::prelude::*;
use reactive_tui::app::RootComponent;
use reactive_tui::backend::SuprTuiBackend;

struct Hello;
impl RootComponent for Hello {
    fn render(&self) -> Element {
        div().class("p-2 bg-blue-500 text-white")
            .child(Element::text("Hello, Reactive-TUI!"))
            .build()
    }
}

fn main() -> Result<()> {
    App::builder()
        .backend(SuprTuiBackend::new()?)
        .root(Hello)
        .build()?
        .run()
}
```

App owns component lifecycle, routed keyboard/mouse input and stable focus.
[Application entry points](docs/application-entry-points.md) explains the retained
backend and native routes. Use the [widget inventory](docs/widget-acceptance.md)
for each input, table, tree, menu and dialog contract.

## Images

Return an image Element from a root or component so App owns its placement,
clipping, replacement, animation and removal:

```rust
use reactive_tui::component::Element;
use reactive_tui::widgets::Image;

fn photo() -> Element {
    Image::from_file("photo.jpg")
        .with_max_size(40, 12)
        .with_preserve_aspect(true)
        .into_element()
}
```

Local files, encoded memory and base64 data URLs are supported source routes.
HTTP/HTTPS image URLs return an explicit unsupported-source error. Image output
uses the selected host protocol, Chafa/Viu or ASCII fallback. See
[image acceptance](docs/image-acceptance.md) for the captured routes and exact limits:

- Kitty graphics acceptance requires the pinned Kitty 0.45.0 patch; this repository
  does not repair users' stock Kitty installations.
- Ghostty graphics, Xterm Sixel and WezTerm inline have captured Linux workflows.
- iTerm2 3.7 has an approved color/transparency exception; placement, replacement
  and removal remain required.
- GNOME Terminal has fallback coverage. Screen-reader acceptance is specifically
  Orca with GNOME Terminal, not every host or assistive tool.

## Layout, animation and native access

Utility classes include flex/grid layout, spacing, colors and state variants.
Terminal transforms move cells and upright glyphs; they do not rotate glyph bitmaps
or change the host font size. [Text styling](docs/text-styling.md) and
[animation integration](docs/ANIMATION_INTEGRATION.md) describe the supported painter.

C headers and TypeScript expose stateful foreign components, editor/layout/dialog
controllers and App ownership. Read the [native component guide](docs/native-components.md),
[C ABI policy](docs/FFI_ABI_POLICY.md) and [TypeScript SDK](bindings/typescript/README.md).
The migration records distinguish retained native functions from retired wrappers.

## Embedded shells and build configurations

```sh
cargo run --locked --features embedded-terminal --example embedded_shell
cargo doc --locked --lib --no-deps
cargo check --locked --no-default-features
```

Embedded sessions use libghostty on Unix and require the pinned Zig toolchain
specified in [embedded sessions](docs/embedded-terminal.md). Ctrl+C reaches the
child; Ctrl+Q exits the host. The `simd` feature requires nightly Rust. Other
feature combinations and native platform evidence are listed in the API matrix.
Builds and checks on this development machine use at most 12 workers per pool.

## Contributors

- **Shawn McAllister** ([@entrepeneur4lyf](https://github.com/entrepeneur4lyf)) - Creator and Lead Developer
- **Auggie** (Claude Sonnet 4) - Systems Architecture and Memory Management Engineering

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## Acknowledgments

- [Taffy](https://github.com/DioxusLabs/taffy) supplies layout.
- [Crossterm](https://github.com/crossterm-rs/crossterm) supplies terminal input and control.
- Comrak and Syntect supply Markdown parsing and syntax highlighting.

### Renderer ownership

The renderer is maintained locally in `src/backend/engine`, imported from
SuprTUI revision `7793deb80c5bceecc5d8ed9fc6bc6d2530531774`.
`SuprTuiBackend` remains the application adapter. The internal crate keeps its
own tests and source notices; `scripts/check-renderer.sh` runs those tests
alongside the application and PTY checks.

### Shared App wakeups

Signal-driven roots can sleep until input or background work arrives. See
[App wakeups](docs/app-wakeups.md) for signal subscriptions, scheduler deadlines,
compatibility behavior, and `cargo run --locked --example wake_counter`.
