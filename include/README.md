# Reactive-TUI C headers

Include `reactive_tui.h` for the current native interface. Modular compatibility
headers under `reactive_tui/` include the compiler-generated `native.h`; historical
type names in `compat.h` do not imply that a similarly named function exists.
The generated declarations are checked against compiled exports and layouts.

Current interfaces include terminal/surface/renderer, Elements/builders,
signals/hooks, animations, App, native text editor, validated layout styles,
dialog sessions and stateful foreign components. Read the
[FFI manual](../manual/ffi-and-typescript.md) and
[migration guide](../bindings/typescript/MIGRATION.md) before managing owners.

## Initialize and release

```c
#include <reactive_tui.h>

int main(void) {
    RTuiError status = rtui_init();
    if (status != R_TUI_ERROR_SUCCESS) return 1;
    rtui_cleanup();
    return 0;
}
```

## An independently owned surface

A surface does not require a live terminal. A renderer's surface, by contrast, is
borrowed and must not be destroyed by the caller.

```c
#include <reactive_tui/core.h>
#include <reactive_tui/surface.h>

int main(void) {
    RTuiSurface *surface = NULL;
    RTuiError status = rtui_init();
    if (status != R_TUI_ERROR_SUCCESS) return 1;
    status = rtui_surface_create(40, 10, &surface);
    if (status == R_TUI_ERROR_SUCCESS) {
        status = rtui_surface_clear(surface, 15, 20, 30);
        rtui_surface_destroy(surface);
    }
    rtui_cleanup();
    return status == R_TUI_ERROR_SUCCESS ? 0 : 1;
}
```

## Loading from Python

This small `ctypes` example loads the current library and declares its lifecycle
signatures. It is not a packaged Python widget binding. Set `RTUI_LIBRARY_PATH`
to the matching shared library. Raw headers contain preprocessor directives;
do not pass them directly to `cffi.FFI.cdef`.

```python
import ctypes
import os

lib = ctypes.CDLL(os.environ["RTUI_LIBRARY_PATH"])
lib.rtui_init.argtypes = []
lib.rtui_init.restype = ctypes.c_int
lib.rtui_cleanup.argtypes = []
lib.rtui_cleanup.restype = None
```

API-018 compiles and links the C examples and checks the Python loader against
the current built library. ABI and native component mechanisms execute their
own consumers with terminal ownership, failure and cleanup assertions.
