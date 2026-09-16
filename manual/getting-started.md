# Getting started

Crate modules: `error`

Cargo features: `default`, `debug_patches`, `debug`, `simd`, `async-capabilities`, `ffi`, `embedded-terminal`, `wgpu-graphics`

## Purpose

This chapter shows the smallest supported application shape and explains crate
features that change the build.

## Main API

- `reactive_tui::prelude::*` imports `App`, builders, components, hooks,
  reactive types, common virtual DOM helpers, macros, and common Taffy values.
- `ReactiveError` is the crate error type.
- `Result<T>` is the crate result alias.
- `App::builder()` starts application construction.
- `RootComponent` supplies the root `Element`.

## Basic use

```rust,no_run
use reactive_tui::app::RootComponent;
use reactive_tui::backend::DebugBackend;
use reactive_tui::prelude::*;

struct Root;

impl RootComponent for Root {
    fn render(&self) -> Element {
        div().class("p-1").text("Hello").build()
    }
}

fn main() -> Result<()> {
    let _app = App::builder()
        .backend(DebugBackend::new(80, 24))
        .root(Root)
        .build()?;
    Ok(())
}
```

## Behavior

`AppBuilder::build` requires both a backend and a root component. The debug
backend renders into memory and is useful for tests. A terminal application can
use `SuprTuiBackend` and call `App::run`.

The default feature enables Tokio support. `async-capabilities` also enables
Tokio. `ffi` exports the C ABI. `embedded-terminal` enables the Unix
libghostty-based embedded session. The two debug features add diagnostic paths.
`simd` enables nightly Rust portable SIMD code.
`wgpu-graphics` enables the optional offscreen canvas with CPU fallback; see
[Offscreen graphics](wgpu-graphics.md).

## Limits

- `simd` requires a nightly compiler because the crate enables
  `portable_simd` when that feature is active.
- `embedded-terminal` is exported only on Unix.
- The embedded terminal dependency is fetched from a pinned Git revision.
- The crate currently builds both an `rlib` and a `cdylib`.

## Source map

- Crate exports and prelude: [`src/lib.rs`](../src/lib.rs)
- Features and package metadata: [`Cargo.toml`](../Cargo.toml)
- Application construction: [`src/app.rs`](../src/app.rs)
- Compiling entry-point contract: [`tests/api_documentation_contract.rs`](../tests/api_documentation_contract.rs)
- Runnable rendering example: [`examples/gradient_blocks.rs`](../examples/gradient_blocks.rs)

## Related chapters

- [Applications and components](app-and-components.md)
- [Rendering and backends](rendering-and-backends.md)
- [Terminal and embedded sessions](terminal-and-embedded-sessions.md)

[Back to the manual](README.md)
