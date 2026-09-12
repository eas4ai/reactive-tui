# C ABI and ownership

The compatibility baseline is the compiled Rust export inventory, not the old
handwritten declarations. Include `reactive_tui.h`, which includes the generated
`reactive_tui/native.h` and compatibility names. Existing compiled signatures and
layouts are preserved by ABI-001 through ABI-004; API-017 adds the owned editor,
layout, dialog and foreign-component interfaces. Recompile callers against the
matching headers. [ABI migration](binding-abi-migration.md) records corrected and
retired declarations; [native components](native-components.md) defines the new
controllers and callback contract.

## Build and link

```sh
cargo build --locked --features ffi
cc -std=c11 -Iinclude example.c -Ltarget/debug -lreactive_tui -o example
LD_LIBRARY_PATH="$PWD/target/debug${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" ./example
```

The command above is for Linux. C/TypeScript ABI and consumer acceptance currently
runs on Linux; macOS/Windows native Rust workflows are separate evidence. Loader
recognition of a platform is not certification of every binding or host terminal.
See the [supported-API matrix](supported-api.md).

```c
#include <reactive_tui.h>
```

## Errors and valid inputs

Functions returning `RTuiError` use the constants in `native.h`, including
`R_TUI_ERROR_SUCCESS` (0), invalid-parameter/null-pointer errors and
`R_TUI_ERROR_PANIC` (-99). Other functions return pointers, booleans or void;
check each declaration rather than interpreting all results as an error code.
Status-returning guarded entry points convert caught panics to errors. This does
not make arbitrary pointers safe or promise that every legacy entry point catches
all failures.

A non-null pointer must still reference the correct live allocation. Keep strings
NUL-terminated and valid for the documented call or callback lifetime. Pass valid
enum variants, buffer lengths and non-overlapping owners where required. Null
checks cannot validate stale, forged or mismatched pointers.

## Ownership and threading

- Pair each owned handle with its matching release function. Functions that
  consume a builder or child transfer ownership; do not reuse or free it afterward.
- A renderer surface is borrowed. Renderer resize, shutdown or destruction ends
  the borrow. Independently created surfaces have their own destructor.
- Owned returned strings use `rtui_string_free`; buffer snapshots use the matching
  `bufferRelease*` function and original count. Borrowed strings remain borrowed.
- Both ordinary `RTuiSignal` families now share typed ownership; either ordinary
  destructor is valid. Their integer widths differ. `RTuiThreadSafeSignal` remains
  a separate family with its own operations and destructor.
- Serialize calls on a handle and respect its creating-thread restrictions.
  Retaining native APIs need stable callbacks and user data until destruction.
  Dispose Apps before their foreign controllers. See the native guide for
  recursive callback rejection, permitted JSON setter reentry and consumed owners.

For example, initialize an output owner and release it only after successful creation:

```c
RTuiTerminal *terminal = NULL;
RTuiError status = rtui_terminal_create(&terminal);
if (status == R_TUI_ERROR_SUCCESS) {
    rtui_terminal_destroy(terminal);
}
```

## Draw one frame

This complete example uses the retained renderer route. Run it in a terminal;
API-018 compiles and links it without taking over the developer's desktop.
It releases the borrowed surface through its renderer owner.

```c
#include <reactive_tui.h>

int main(void) {
    RTuiRenderer *renderer = NULL;
    RTuiSurface *surface = NULL;
    RTuiError status = rtui_init();
    if (status != R_TUI_ERROR_SUCCESS) return 1;

    status = rtui_renderer_create(40, 10, &renderer);
    if (status != R_TUI_ERROR_SUCCESS) goto cleanup;
    status = rtui_renderer_get_surface(renderer, &surface);
    if (status != R_TUI_ERROR_SUCCESS) goto cleanup;
    status = rtui_renderer_frame(renderer, true);
    if (status != R_TUI_ERROR_SUCCESS) goto cleanup;
    status = rtui_surface_clear(surface, 15, 20, 30);
    if (status != R_TUI_ERROR_SUCCESS) goto cleanup;
    const RTuiColor white = {255, 255, 255};
    const RTuiColor background = {15, 20, 30};
    status = rtui_surface_draw_text(surface, 2, 2, "Hello from C", &white, &background);
    if (status != R_TUI_ERROR_SUCCESS) goto cleanup;
    status = rtui_renderer_frame(renderer, false);

cleanup:
    if (renderer) {
        RTuiError shutdown = rtui_renderer_shutdown(renderer);
        if (status == R_TUI_ERROR_SUCCESS) status = shutdown;
        rtui_renderer_destroy(renderer);
    }
    rtui_cleanup();
    return status == R_TUI_ERROR_SUCCESS ? 0 : 1;
}
```

Use the native App route for retained components, focus and event routing.
`rtui_terminal_poll_event` takes a timeout and output event, with no terminal
handle argument. Old frame/terminal method names that have no native symbol are
listed in the migration record rather than provided as successful no-ops.

## Verification

ABI-001 checks compiler-derived headers; ABI-002 checks TypeScript signatures and
layouts; ABI-003 checks native consumers. API-001 verifies typed signal ownership,
and API-017 verifies editor/layout/dialog and foreign component workflows through
compiled C and TypeScript consumers. These checks establish the named paths and
failure cases, not blanket memory safety for invalid caller pointers.
