# Reactive-TUI C API Headers

This directory contains the modular C API headers for Reactive-TUI, organized for maintainability and ease of use.

## 📁 Header Structure

### Main Header
- **`reactive_tui.h`** - Main header that includes all modules (194 lines)
  - Include this single file to access the complete API
  - Provides convenience macros and common constants
  - Documents future modules to be implemented

### Modular Headers (`reactive_tui/` directory)

#### Core Infrastructure
- **`core.h`** - Core types, error handling, and library initialization
  - Version information and library lifecycle
  - Error codes and basic types (Position, Rect, Color, Cell)
  - Memory management functions

- **`events.h`** - Event system for input handling
  - Event types (keyboard, mouse, resize, focus, paste)
  - Event structures and handler callbacks
  - Event polling functions

#### Terminal & Rendering
- **`terminal.h`** - Terminal control and management
  - Terminal creation, configuration, and cleanup
  - Raw mode control and cursor management
  - Basic terminal operations (clear, write, flush)

- **`surface.h`** - Surface and buffer operations
  - Surface creation and management
  - Cell manipulation and text operations
  - Rectangle operations and surface copying

- **`render.h`** - Rendering system and frame management
  - Renderer creation and frame control
  - Drawing operations (surfaces, text, rectangles)
  - Efficient terminal output

#### UI Components
- **`dialogs.h`** - Dialog system for user interactions
  - Dialog engine management
  - Dialog types (confirmation, input, toast, progress)
  - Dialog lifecycle and interaction

- **`animation.h`** - Animation system with easing
  - Animation manager and creation
  - Easing functions and loop modes
  - Property animation and spring physics

## 🎯 Benefits of Modular Structure

### For Developers
1. **Selective Inclusion** - Include only needed modules
2. **Clear Organization** - Easy to find specific functionality
3. **Reduced Compile Time** - Smaller headers compile faster
4. **Better Documentation** - Each module is self-contained

### For Maintainers
1. **Easier Updates** - Modify specific modules without affecting others
2. **Clear Boundaries** - Well-defined module responsibilities
3. **Scalable Growth** - Easy to add new modules
4. **Reduced Conflicts** - Smaller files reduce merge conflicts

### For Language Bindings
1. **Granular Binding** - Generate bindings for specific modules
2. **Incremental Support** - Implement modules progressively
3. **Clear Dependencies** - Understand module relationships
4. **Easier Testing** - Test individual modules in isolation

## 📋 Usage Examples

### Simple Usage (All Features)
```c
#include <reactive_tui.h>  // Include everything

int main() {
    rtui_init();
    // Use any API...
    rtui_cleanup();
}
```

### Selective Usage (Specific Modules)
```c
#include <reactive_tui/core.h>
#include <reactive_tui/terminal.h>
#include <reactive_tui/render.h>

int main() {
    rtui_init();
    
    RTuiTerminal* terminal;
    RTuiRenderer* renderer;
    
    rtui_terminal_create(&terminal);
    rtui_renderer_create(terminal, &renderer);
    
    // Basic rendering only...
    
    rtui_renderer_destroy(renderer);
    rtui_terminal_destroy(terminal);
    rtui_cleanup();
}
```

### Language Binding Example (Python)
```python
# Generate bindings for specific modules
from cffi import FFI

ffi = FFI()

# Load only core and terminal modules
ffi.cdef(open('reactive_tui/core.h').read())
ffi.cdef(open('reactive_tui/terminal.h').read())

lib = ffi.dlopen('libreactive_tui.so')
```

## 🚀 Future Expansion

The modular structure is designed to accommodate future API additions:

### Planned Modules
- `app.h` - Application framework and lifecycle
- `builder.h` - Element builder API with CSS styling
- `layout.h` - CSS-like layout with flexbox/grid
- `widgets.h` - Widget library (input, display, layout)
- `reactive.h` - Reactive system (hooks, signals)
- `theme.h` - Theming and styling system
- `editor.h` - Text editor components
- `syntax.h` - Syntax highlighting
- `markdown.h` - Markdown rendering
- `platform.h` - Platform-specific features

### Adding New Modules
1. Create `reactive_tui/new_module.h`
2. Add include to main `reactive_tui.h`
3. Update this documentation
4. Implement corresponding Rust FFI functions

## 📊 Current Status

**Total Lines**: ~1,400 lines (vs 1,157 lines in monolithic header)
**Modules**: 7 implemented, 10 planned
**Coverage**: Core functionality complete, UI framework in progress

The modular structure provides a clean, maintainable foundation for the complete Reactive-TUI C API while keeping individual files manageable and focused.
