# Reactive-TUI Visual Demos

This directory contains **actual visual demonstrations** of reactive-tui features. These are interactive applications that showcase the framework's capabilities.

**🎯 Important**: Tests have been moved to `tests/integration/`. Only visual demos remain here.

## Running Visual Demos

To run any visual demo:

```bash
cargo run --example <demo_name>
```

## 🎨 Available Visual Demos

These are interactive applications that showcase reactive-tui's capabilities:

### 🪟 Hierarchical Windows Demo
**File**: `hierarchical_windows_demo.rs`
**Command**: `cargo run --example hierarchical_windows_demo`

Demonstrates the advanced window system with libvaxis feature parity:
- Parent-child window relationships
- Automatic constraint handling
- Advanced text printing with segments
- Cursor management and scrolling
- Mouse event handling
- Border rendering with different styles

**Visual Features**: Interactive window hierarchy with nested panels, borders, and real-time updates.

### 🖥️ Terminal Widget Demo
**File**: `terminal_widget_demo.rs`
**Command**: `cargo run --example terminal_widget_demo`

Showcases the embedded terminal widget - reactive-tui's "killer feature":
- Complete terminal emulator integration
- ANSI escape sequence parsing
- Virtual screen with scrollback
- Cross-platform PTY spawning
- Component system integration

**Visual Features**: Full terminal emulator running inside a TUI application with status display.

### 🎨 Grid Showcase Demo
**File**: `grid_showcase.rs`
**Command**: `cargo run --example grid_showcase`

Interactive demonstration of the unified grid system:
- CSS Grid and Flexbox layouts
- Responsive design patterns
- Dynamic grid manipulation
- Real-time layout updates

**Visual Features**: Interactive grid layouts with live editing and responsive behavior.

### 🎬 Integrated Animations Demo
**File**: `integrated_animations.rs`
**Command**: `cargo run --example integrated_animations`

Showcases the coordinated animation system:
- Smooth transitions and effects
- Timeline-based animations
- Easing functions
- Performance optimization

**Visual Features**: Smooth animated UI elements with coordinated timing.

### 🖼️ Image Placement Demo
**File**: `image_placement_demo.rs`
**Command**: `cargo run --example image_placement_demo`

Demonstrates image rendering capabilities:
- Multiple image formats
- Placement and scaling
- Terminal capability detection
- Fallback handling

**Visual Features**: Images displayed in terminal with proper scaling and positioning.

### 📱 Responsive Grid Demo
**File**: `responsive_grid_demo.rs`
**Command**: `cargo run --example responsive_grid_demo`

Interactive responsive layout demonstration:
- Breakpoint-based layouts
- Dynamic grid reconfiguration
- Mobile-first design patterns
- Real-time responsiveness

**Visual Features**: Grid layouts that adapt to terminal size changes in real-time.

### 🎨 Syntax Highlighting Demos
**File**: `simple_syntax_highlight.rs` / `syntax_with_theme.rs`
**Commands**:
- `cargo run --example simple_syntax_highlight`
- `cargo run --example syntax_with_theme`

Code syntax highlighting demonstrations:
- Multiple language support
- Theme system integration
- Color scheme variations
- Performance optimization

**Visual Features**: Beautifully highlighted code with multiple themes and languages.

## 🧪 Tests

**Framework tests have been moved to `tests/integration/`**

To run tests:
```bash
cargo test
# or for integration tests specifically:
cargo test --test integration
```

Tests include:
- Backend comparison and validation
- Grid system functionality
- Terminal emulation testing
- Performance benchmarking
- Mouse and keyboard input validation
- Animation system verification
- And much more...

## 🎯 Demo vs Test Guidelines

**Visual Demos** (in `examples/`):
- ✅ Interactive applications you can see and use
- ✅ Showcase framework capabilities visually
- ✅ Demonstrate real-world usage patterns
- ✅ User-facing feature demonstrations

**Tests** (in `tests/integration/`):
- ✅ Validate functionality works correctly
- ✅ Performance and benchmark testing
- ✅ Error handling verification
- ✅ Framework internal testing

---

**🎯 Remember**: This directory is for **visual demonstrations only**. All tests have been moved to `tests/integration/` where they belong!
