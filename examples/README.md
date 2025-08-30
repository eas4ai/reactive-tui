# Reactive-TUI Examples & Tests

This directory contains both **visual examples** and **framework tests** for the Reactive-TUI library.

## Running Examples & Tests

To run any example or test:

```bash
cargo run --example <name>
```

## 🧪 Framework Tests

These validate that the framework works correctly. They focus on **functionality verification** rather than visual demonstration.

### Terminal Capability Detection Test
**File**: `real_capability_detection.rs`
**Command**: `cargo run --example real_capability_detection`

Validates the terminal capability detection framework:
- API functionality testing (5 test categories)
- Data integrity validation
- Performance benchmarking
- Error handling verification
- Cross-terminal compatibility

**Purpose**: Ensures the capability detection system works correctly across different terminals.

### Image Widget Test
**File**: `image_widget_demo.rs`
**Command**: `cargo run --example image_widget_demo`

Tests image rendering capabilities:
- Multiple rendering backend validation
- Error handling and fallback testing
- Format support verification
- Terminal capability detection
- Resource cleanup validation

**Purpose**: Validates that image rendering works correctly and safely.

### Mouse Tracking Test
**File**: `mouse_tracking_demo.rs`
**Command**: `cargo run --example mouse_tracking_demo`

Tests mouse input capabilities:
- Mouse level detection
- Coordinate system validation
- Event handling verification
- Capability probing

**Purpose**: Ensures mouse tracking works across different terminals.

### Performance Test
**File**: `enhanced_performance_demo.rs`
**Command**: `cargo run --example enhanced_performance_demo`

Validates rendering performance:
- Render operation benchmarking
- Memory usage monitoring
- Frame rate testing
- Optimization verification

**Purpose**: Ensures the framework meets performance requirements.

### Screen Management Test
**File**: `multi_screen_demo.rs`
**Command**: `cargo run --example multi_screen_demo`

Tests screen management functionality:
- Screen creation and navigation
- Transition system validation
- Event handling verification
- State management testing

**Purpose**: Validates that screen management and transitions work correctly.

### Animation System Test
**File**: `animation_integration_demo.rs`
**Command**: `cargo run --example animation_integration_demo`

Tests animation system functionality:
- Animation pipeline validation
- Easing function testing
- Performance measurement
- Integration verification

**Purpose**: Ensures animation system works correctly and performs well.

## 🎨 Visual Examples

These demonstrate features visually so users can **see** what the library can do.

### Multi-Screen Navigation
**File**: `multi_screen_demo.rs`
**Command**: `cargo run --example multi_screen_demo`

**What you'll see**: Interactive screen transitions and navigation
- Multiple application screens with smooth transitions
- Hotkey-based navigation between screens
- State management demonstration
- Visual transition effects

### Animation Showcase
**File**: `animation_integration_demo.rs`
**Command**: `cargo run --example animation_integration_demo`

**What you'll see**: Various animation effects and transitions
- Spring physics animations in action
- Different easing functions demonstrated
- Staggered animation sequences
- Performance optimization examples

### Syntax Highlighting Display
**File**: `syntax_highlight.rs`
**Command**: `cargo run --example syntax_highlight`

**What you'll see**: Live syntax highlighting of code
- Real-time code syntax highlighting
- Multiple language support
- Theme application
- Interactive editing (press ESC to exit)

### Advanced Syntax with Themes
**File**: `syntax_with_theme.rs`
**Command**: `cargo run --example syntax_with_theme`

**What you'll see**: Themed syntax highlighting
- Multiple color themes in action
- Theme switching demonstration
- Custom syntax definitions
- Optimized rendering display

### Efficient Rendering Demo
**File**: `patch_aware_rendering.rs`
**Command**: `cargo run --example patch_aware_rendering`

**What you'll see**: Optimized rendering in action
- Diff-based updates visualization
- Performance metrics display
- Render optimization techniques
- Real-time performance monitoring

### Statistics Dashboard
**File**: `stats_demo.rs`
**Command**: `cargo run --example stats_demo`

**What you'll see**: Live performance statistics
- Real-time performance metrics
- Memory usage visualization
- Render statistics display
- System resource monitoring



## Terminal Requirements

### Minimum Requirements
- 24-bit color support
- Unicode support
- Modern terminal emulator

### Recommended Terminals
- **WezTerm**: Full feature support including sixel graphics
- **Kitty**: Excellent performance with graphics protocol
- **Alacritty**: Fast rendering with good color support
- **iTerm2**: macOS with inline image support

### Feature Support by Terminal
| Terminal | Sixel | Kitty Graphics | iTerm2 Inline | External Tools | Mouse | Animations |
|----------|-------|----------------|---------------|----------------|-------|------------|
| WezTerm  | ✅     | ❌              | ❌             | ✅              | ✅     | ✅          |
| Kitty    | ❌     | ✅              | ❌             | ✅              | ✅     | ✅          |
| iTerm2   | ❌     | ❌              | ✅             | ✅              | ✅     | ✅          |
| Alacritty| ❌     | ❌              | ❌             | ✅              | ✅     | ✅          |

## Troubleshooting

### Tests Failing?
1. **Framework tests**: Should pass on any modern terminal
2. **Capability detection**: May show different results per terminal (this is expected)
3. **Performance tests**: Results vary by system performance
4. **Mouse tests**: Require interactive terminal (not CI/automated environments)

### Visual Examples Not Working?
1. **Check terminal compatibility**: Ensure your terminal supports required features
2. **Install external tools**: `brew install chafa` or `cargo install viu` for image examples
3. **Verify terminal settings**: Some features require specific configuration
4. **Terminal size**: Ensure terminal is large enough for visual examples

### Performance Issues?
1. **Use recommended terminals**: WezTerm and Kitty offer best performance
2. **Reduce complexity**: Lower animation frame rates for slower systems
3. **Check system resources**: Monitor CPU and memory usage during tests

## Development Guidelines

### Creating New Tests
1. **Focus on validation**: Tests should verify functionality works correctly
2. **Include error cases**: Test both success and failure scenarios
3. **Make them deterministic**: Tests should produce consistent results
4. **Add comprehensive assertions**: Validate all important aspects

### Creating New Visual Examples
1. **Focus on demonstration**: Examples should show features visually
2. **Include user interaction**: Let users see features in action
3. **Add clear instructions**: Tell users what they should see/do
4. **Handle graceful exit**: Provide clear exit mechanisms (ESC key, etc.)

### General Guidelines
1. **Test across terminals**: Verify compatibility with major terminals
2. **Keep dependencies minimal**: Only add necessary dependencies
3. **Add documentation**: Comment complex sections thoroughly
4. **Follow code style**: Use `cargo fmt` and follow project conventions

## Contributing

We welcome new examples and tests! Please:

1. **Choose the right category**: Tests for validation, Examples for demonstration
2. **Follow existing patterns**: Look at similar files for structure
3. **Update this README**: Add your new test/example to the appropriate section
4. **Test thoroughly**: Verify on multiple terminals and systems
5. **Include requirements**: Document any special dependencies or setup

See [CONTRIBUTING.md](../CONTRIBUTING.md) for detailed guidelines.
