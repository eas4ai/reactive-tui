# Reactive-TUI Examples

This directory contains working examples demonstrating various features of the Reactive-TUI library.

## Running Examples

To run any example:

```bash
cargo run --example <example_name>
```

## Available Examples

### 🖼️ Image Widget Demo
**File**: `image_widget_demo.rs`
**Command**: `cargo run --example image_widget_demo`

Demonstrates comprehensive image support including:
- Multiple rendering backends (sixel, chafa, viu, terminal protocols)
- Different image sources (files, base64, raw data)
- Quality settings and size constraints
- Error handling and fallbacks
- Terminal capability detection

**Requirements**: Modern terminal with 24-bit color support. For best experience, install `chafa` or `viu`.

### 🎬 Animation Integration Demo
**File**: `animation_integration_demo.rs`
**Command**: `cargo run --example animation_integration_demo`

Shows animation capabilities:
- Spring physics animations
- Easing functions and transitions
- Staggered animations
- Performance optimization
- Timeline management

### 🖥️ Multi-Screen Demo
**File**: `multi_screen_demo.rs`
**Command**: `cargo run --example multi_screen_demo`

Demonstrates screen management:
- Multiple application screens
- Screen transitions and navigation
- State management across screens
- Hotkey handling

### 🎨 Syntax Highlighting
**File**: `syntax_highlight.rs`
**Command**: `cargo run --example syntax_highlight`

Basic syntax highlighting example:
- Code syntax highlighting
- Language detection
- Theme application
- Performance optimization

### 🌈 Syntax with Themes
**File**: `syntax_with_theme.rs`
**Command**: `cargo run --example syntax_with_theme`

Advanced syntax highlighting with custom themes:
- Multiple color themes
- Theme switching
- Custom syntax definitions
- Optimized rendering

### ⚡ Patch-Aware Rendering
**File**: `patch_aware_rendering.rs`
**Command**: `cargo run --example patch_aware_rendering`

Demonstrates efficient rendering:
- Diff-based updates
- Minimal terminal output
- Performance monitoring
- Render optimization techniques

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

### Image Support by Terminal
| Terminal | Sixel | Kitty Graphics | iTerm2 Inline | External Tools |
|----------|-------|----------------|---------------|----------------|
| WezTerm  | ✅     | ❌              | ❌             | ✅              |
| Kitty    | ❌     | ✅              | ❌             | ✅              |
| iTerm2   | ❌     | ❌              | ✅             | ✅              |
| Alacritty| ❌     | ❌              | ❌             | ✅              |

## Troubleshooting

### Image Examples Not Working?
1. **Check terminal compatibility**: Ensure your terminal supports 24-bit colors
2. **Install external tools**: `brew install chafa` or `cargo install viu`
3. **Verify terminal settings**: Some terminals require specific configuration
4. **Check file paths**: Ensure image files exist and are accessible

### Performance Issues?
1. **Use recommended terminals**: WezTerm and Kitty offer best performance
2. **Reduce animation complexity**: Lower frame rates for slower systems
3. **Optimize image sizes**: Large images may impact performance
4. **Check system resources**: Monitor CPU and memory usage

### Display Problems?
1. **Terminal size**: Ensure terminal is large enough for examples
2. **Font support**: Use fonts with good Unicode coverage
3. **Color support**: Verify 24-bit color is enabled
4. **Refresh issues**: Try resizing terminal window

## Example Development

When creating new examples:

1. **Focus on one feature**: Each example should demonstrate a specific capability
2. **Include error handling**: Show proper error handling patterns
3. **Add documentation**: Comment complex sections thoroughly
4. **Test across terminals**: Verify compatibility with major terminals
5. **Keep dependencies minimal**: Only add necessary dependencies

## Contributing Examples

We welcome new examples! Please:

1. Follow the existing code style
2. Add comprehensive comments
3. Update this README with your example
4. Test on multiple terminals
5. Include any special requirements

See [CONTRIBUTING.md](../CONTRIBUTING.md) for detailed guidelines.
