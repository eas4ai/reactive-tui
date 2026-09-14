# FFI and TypeScript

Crate modules: `ffi`

## Purpose

The foreign-function interface exposes selected framework operations through a
C ABI. TypeScript packages wrap native functions and provide SDK types and
builders.

## Main API

- The `ffi` Cargo feature enables the Rust C ABI module.
- Opaque handles represent applications, terminals, surfaces, renderers,
  elements, editors, dialogs, animations, signals, effects, Markdown renderers,
  syntax highlighters, themes, and foreign components.
- `RTuiVersion` and `rtui_version` report ABI compatibility information.
- Error enums and owned-string functions define cross-language failure and
  memory handling.
- The C headers declare the consumer ABI.
- TypeScript modules expose native application, layout, component, widget,
  reactive, terminal, editor, dialog, Markdown, syntax, theme, and platform
  wrappers.

## Basic use

Enable `ffi`, build the `cdylib`, include the matching public header, initialize
the library, create handles, check every returned error, and destroy owned
handles with their matching function. A TypeScript host should use the package
entry points and load the matching native library.

## Behavior

Handle-based entry points check null pointers before dereferencing them. A basic
pointer check can reject null, misaligned, or implausible addresses. It does not
prove that it is allocated or live, owned by the library, readable, or writable.
Tracked handle families also require a matching live allocation before access.

Panic boundaries convert Rust failures into ABI error values. Functions that
return allocated strings or arrays provide matching release functions. Buffer
array snapshots keep their allocation metadata inside the library. The
caller-supplied length does not control deallocation. Interior, misaligned, and
repeated releases are ignored. Optimized-buffer and text-buffer destruction is
single-use; later destroy calls do not reach the allocator.

Element child insertion transfers child ownership after validation.
Self-parenting is rejected. App run borrows its retained handle while blocking,
so another thread can request quit; destruction happens after run returns.
Optional root and cleanup callbacks may be null. Foreign component callbacks
cross a guarded boundary and preserve the last error for diagnostics.

The TypeScript layer maps native values into typed classes and interfaces. Its
public declarations must match the exported C ABI names, ownership, arguments,
and result values.

## Limits

- The C ABI is absent unless the `ffi` feature is enabled.
- A handle must be destroyed by the API that created it and must not be used
  after release.
- The native library, header, and language package must use compatible ABI
  versions.
- Rust lifetimes are represented by explicit ownership rules across the ABI.
- TypeScript code does not replace the native shared library.

## Source map

- FFI exports: [`src/ffi/mod.rs`](../src/ffi/mod.rs)
- FFI pointer tracking: [`src/ffi/pointer.rs`](../src/ffi/pointer.rs)
- Public C header: [`include/reactive_tui.h`](../include/reactive_tui.h)
- TypeScript package source: [`bindings/typescript/src/index.ts`](../bindings/typescript/src/index.ts)
- ABI consumer test: [`tests/binding_abi_consumer.c`](../tests/binding_abi_consumer.c)
- FFI integration tests: [`tests/test_ffi_integration.rs`](../tests/test_ffi_integration.rs)

## Related chapters

- [Getting started](getting-started.md)
- [Applications and components](app-and-components.md)
- [Text editing, Markdown, and syntax](text-editing-markdown-and-syntax.md)

[Back to the manual](README.md)
