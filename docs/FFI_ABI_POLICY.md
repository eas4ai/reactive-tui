# FFI and ABI Policy

## Overview

The reactive-tui FFI layer provides a stable C ABI for using the library from other languages while maintaining zero overhead for Rust users.

## ABI Versioning

- **Current ABI Version**: 1
- **Semantic Versioning**: The ABI follows semantic versioning
  - Major: Breaking changes to existing functions/structures
  - Minor: New functions/fields added (backward compatible)
  - Patch: Bug fixes with no ABI changes

## Stability Guarantees

### Stable (ABI v1)
- All functions in `rtui.h`
- All structures marked `#[repr(C)]`
- Error codes will not be renumbered
- Existing function signatures will not change

### Unstable
- Internal structures not exposed through FFI
- Rust-only APIs
- Debug/development features

## Memory Management

### Ownership Rules
1. **Create/Destroy Pattern**: Every `rtui_*_create` has a corresponding `rtui_*_destroy`
2. **Caller Owns Output**: Output parameters allocated by caller
3. **Library Owns Handles**: Opaque handles must be freed via destroy functions
4. **String Lifetime**: Strings passed to functions are copied internally

### Example
```c
ReactiveTerminal* terminal = NULL;
ReactiveError err = rtui_terminal_create(&terminal);
if (err == RTUI_SUCCESS) {
    // Use terminal
    rtui_terminal_destroy(terminal);
}
```

## Thread Safety

### Thread-Safe Functions
- `rtui_version()`
- `rtui_init()` (call once before any other functions)
- `rtui_cleanup()` (call once at program end)

### NOT Thread-Safe
- All terminal operations must be called from the same thread
- Surface modifications require external synchronization
- Renderer operations must be sequential

## Error Handling

### Error Codes
All functions returning `ReactiveError` follow these conventions:
- `RTUI_SUCCESS (0)`: Operation succeeded
- Negative values: Errors (check specific code)
- `RTUI_ERROR_PANIC (-99)`: Internal panic caught (bug in library)

### Panic Safety
All FFI functions catch Rust panics and return `RTUI_ERROR_PANIC`. This prevents undefined behavior but indicates a bug in the library.

## Platform Support

### Tier 1 (Fully Supported)
- Linux (x86_64, aarch64)
- macOS (x86_64, aarch64)
- Windows 10+ (x86_64)

### Tier 2 (Best Effort)
- FreeBSD
- Other Unix-like systems

## Language Bindings

### Official
- C/C++ (via header)
- Rust (native, no FFI overhead)

### Community
- Python (via cffi)
- Node.js (via N-API)
- Go (via cgo)

## Deprecation Policy

1. **Deprecation Notice**: Functions marked deprecated in minor release
2. **Migration Period**: Deprecated functions maintained for 2 major releases
3. **Removal**: Only in major version with migration guide

## Building with FFI

### Rust Side
```bash
cargo build --features ffi
```

### C Side
```c
#include <rtui.h>
// Link with -lreactive_tui
```

### pkg-config
```bash
pkg-config --cflags --libs reactive-tui
```

## Example Usage

### Basic Terminal Application (C)
```c
#include <rtui.h>
#include <stdio.h>

int main() {
    ReactiveError err;
    ReactiveTerminal* terminal = NULL;
    RTuiRenderer* renderer = NULL;
    RTuiSurface* surface = NULL;
    
    // Initialize library
    err = rtui_init();
    if (err != RTUI_SUCCESS) return 1;
    
    // Create terminal
    err = rtui_terminal_create(&terminal);
    if (err != RTUI_SUCCESS) goto cleanup;
    
    // Enter raw mode
    err = rtui_terminal_enter_raw_mode(terminal);
    if (err != RTUI_SUCCESS) goto cleanup;
    
    // Create renderer
    err = rtui_renderer_create(terminal, &renderer);
    if (err != RTUI_SUCCESS) goto cleanup;
    
    // Main loop
    while (1) {
        // Begin frame
        rtui_renderer_begin_frame(renderer);
        
        // Get surface for drawing
        rtui_renderer_get_surface(renderer, &surface);
        
        // Draw something
        RTuiColor white = {255, 255, 255};
        RTuiColor black = {0, 0, 0};
        rtui_surface_draw_text(surface, 10, 10, "Hello from C!", &white, &black);
        
        // End frame
        rtui_renderer_end_frame(renderer, terminal);
        
        // Check for exit
        RTuiEvent event;
        if (rtui_terminal_poll_event(terminal, 100, &event) == RTUI_SUCCESS) {
            if (event.event_type == RTUI_EVENT_KEY && event.data.key.key_code == 27) {
                break; // ESC pressed
            }
        }
    }
    
cleanup:
    if (renderer) {
        rtui_renderer_shutdown(renderer, terminal);
        rtui_renderer_destroy(renderer);
    }
    if (terminal) {
        rtui_terminal_exit_raw_mode(terminal);
        rtui_terminal_destroy(terminal);
    }
    rtui_cleanup();
    
    return err == RTUI_SUCCESS ? 0 : 1;
}
```

## Testing

FFI tests are in `tests/ffi/` and can be run with:
```bash
cargo test --features ffi
```

## Security Considerations

1. **Input Validation**: All inputs validated at FFI boundary
2. **Buffer Overrun Protection**: Size parameters checked
3. **UTF-8 Validation**: String inputs validated
4. **Integer Overflow**: Checked arithmetic where applicable
5. **Null Pointer Checks**: All pointer parameters validated

## Contact

Report FFI issues to: https://github.com/reactive-tui/reactive-tui/issues
Tag with `ffi` label.