# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive image support with multi-backend rendering
- Sixel graphics protocol support for native terminal graphics
- External tool integration (chafa, viu) for broad compatibility
- Terminal protocol support (Kitty graphics, iTerm2 inline images)
- ASCII art fallback with Floyd-Steinberg dithering
- Image utility CSS classes (`image-fit-cover`, `aspect-ratio-16-9`, etc.)
- Automatic terminal capability detection for images
- Quality settings for image rendering (Fast, Balanced, High)
- Size constraints with aspect ratio preservation
- Base64 image data support
- Raw pixel data support for images

### Changed
- Updated base64 API usage to modern Engine-based approach
- Improved error handling with image-specific error types
- Enhanced documentation with comprehensive examples
- Organized project structure with proper documentation hierarchy

### Fixed
- Resolved all clippy warnings for clean, idiomatic code
- Fixed aspect ratio calculation in image renderers
- Corrected test expectations for image sizing
- Eliminated unused imports and dead code warnings

## [0.0.7] - 2024-XX-XX

### Added
- Initial release with core TUI functionality
- CSS-like layout system with Tailwind-inspired utility classes
- Double-buffered rendering with diff-based updates
- Component system with React-like API
- Event handling for mouse and keyboard
- Animation support with spring physics
- Syntax highlighting with customizable themes
- Markdown rendering with GFM extensions
- Multi-screen application support
- Adaptive performance management
- FFI bindings for C interoperability

### Features
- **Layout System**: Flexbox and grid layouts using Taffy
- **Styling**: Comprehensive utility classes for spacing, colors, typography
- **Components**: Reusable UI widgets (modals, tables, trees, progress bars)
- **Hooks**: React-like hooks for state management and effects
- **Animation**: Smooth transitions with easing functions
- **Terminal Support**: 24-bit color with modern terminal compatibility

### Technical
- Built on crossterm for terminal control
- Uses Taffy for layout computation
- Supports wezterm, kitty, alacritty, iTerm2
- Requires Rust 1.70+

## [0.0.6] - Previous versions

Previous versions focused on core architecture development and are not documented in detail.

---

## Release Notes

### Image Support (0.0.7+)

The major addition in this release is comprehensive image support:

#### Rendering Backends
- **Sixel Graphics**: Native terminal graphics for high-quality images
- **External Tools**: Integration with chafa and viu
- **Terminal Protocols**: Kitty graphics and iTerm2 inline images
- **ASCII Art**: Fallback with advanced dithering algorithms

#### Usage Example
```rust
use reactive_tui::widgets::Image;
use reactive_tui::core::terminal::Terminal;

let image = Image::from_file("photo.jpg")
    .with_max_size(80, 24)
    .with_preserve_aspect(true)
    .with_quality(ImageQuality::High);

let capabilities = Terminal::detect_image_capabilities();
let output = image.render(&capabilities)?;
```

#### Terminal Compatibility
| Terminal | Sixel | Kitty Graphics | iTerm2 Inline | External Tools |
|----------|-------|----------------|---------------|----------------|
| WezTerm  | ✅     | ❌              | ❌             | ✅              |
| Kitty    | ❌     | ✅              | ❌             | ✅              |
| iTerm2   | ❌     | ❌              | ✅             | ✅              |
| Alacritty| ❌     | ❌              | ❌             | ✅              |

### Breaking Changes

None in this release. All changes are additive and backward compatible.

### Migration Guide

No migration required. New image features are opt-in and don't affect existing code.

### Performance Improvements

- Optimized diff-based rendering reduces terminal output
- Efficient image processing with lazy loading
- Smart capability detection minimizes runtime overhead

### Known Issues

- Some terminals may require specific configuration for optimal image display
- Large images may impact performance on slower terminals
- External tools (chafa/viu) must be installed separately for best experience
