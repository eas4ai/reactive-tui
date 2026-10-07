# Reactive TUI TypeScript SDK

The package wraps Rust terminal, surface, renderer and Element APIs, plus
NativeApp, ForeignComponent, NativeTextEditor, NativeLayoutStyle and NativeDialogEngine. It uses Koffi 3.2.1 and compiler-audited native declarations.

## Build and verify

Use Node.js 20 or later, the repository's Rust toolchain, and a compatible native
library. The acceptance checks currently run on Linux with clang and rustc.
Other platform names are recognized by the loader but are not certified by these checks.

```sh
# From the repository root:
cargo build --locked --features ffi
cd bindings/typescript
npm ci
npm run build
npm test
```

`npm run build` checks all remaining TypeScript source and recreates dist.
`npm test` audits signatures/layouts before executing real C-boundary calls in an
isolated terminal, with a deadline and terminal-restoration checks.
`npm run lint` runs the configured ESLint correctness and unused-variable checks.

Set `RTUI_LIBRARY_PATH` to select a particular shared library. Otherwise the loader
looks in package/native, then the repository release and debug target directories.
`npm run build:rust` builds a release library. Native libraries are not downloaded
or included automatically: distribute a matching library in package/native or set
the environment variable. A package must be rebuilt with the same native ABI schema.

## Draw a frame

```typescript
import { initialize, cleanup, Renderer } from '@reactive-tui/core';

initialize();
try {
  const renderer = new Renderer(40, 10);
  try {
    const surface = renderer.getSurface();
    renderer.frame(() => {
      surface.clear(15, 20, 30);
      surface.setCell(2, 2, '界', 0xFFFFFF, 0x0F141E);
    });
    surface.dispose(); // Releases this view, not the renderer's storage.
  } finally { renderer.dispose(); }
} finally { cleanup(); }
```

Use `new Terminal()` to query the host size and capabilities. Its shutdown/dispose
restores the terminal and releases the handle. Native setup currently returns no
error value; the TypeScript wrapper does not invent a success/error result.
Initialize and dispose terminal/renderer owners in sequence so their terminal modes
do not overlap. Importing this package installs no signal or exit handlers.

An independently created `Surface` owns its storage. Its resize recreates an empty
surface. A renderer's surface is borrowed; renderer resize or disposal invalidates
the TypeScript view. A cell stores one Unicode scalar, not a grapheme cluster.

## Build a native Element tree

```typescript
import { div, text } from '@reactive-tui/core';

const root = div().class('flex-col').key('root').child(text('Hello')).build();
try {
  const child = root.getChild(0);
  try { console.log(child.getText()); }
  finally { child.dispose(); }
} finally { root.dispose(); }
```

Child insertion consumes the child owner. Build consumes its builder. Child
getters return owned clones. Explicitly dispose owners that are not consumed.
Native builder `.text()` converts its result to a text Element; use `.child(text())`
when you want a container with text children.

Static trees and explicit stateful controllers are supported. `ForeignComponent`
owns JSON props/state and routed callbacks; `NativeApp` owns the retained native
App. Editor, layout and dialog controllers expose their recovered native behavior.
See the [FFI manual](../../manual/ffi-and-typescript.md) for callback lifetimes,
consuming calls, errors and disposal order. The old widget-specific JSON wrappers
remain deliberately retired; the new controllers do not recreate those methods.
[TypeScript migration](MIGRATION.md) records the complete before/after inventory.
The low-level `lib` export requires valid pointers and caller-managed ownership.

The examples `hello-world.ts` and `component-demo.ts` use the supported API.
The old animation/widget demos are retired along with their unsupported wrappers.

The low-level owned signal-string getter converts to a JS string and releases the native allocation automatically. See the migration guide before using either native signal family.
