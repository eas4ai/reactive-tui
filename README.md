# Reactive-TUI

A modern, reactive terminal user interface library for Rust with CSS-like styling and comprehensive image support.

## Features

- **CSS-like Layout System**: Use familiar CSS properties like `flex`, `grid`, `padding`, `margin`
- **Utility Classes**: Tailwind-inspired classes like `p-4`, `bg-blue-500`, `text-white`
- **Modern Terminal Support**: Requires 24-bit color terminals (wezterm, kitty, alacritty, iTerm2)
- **Double-buffered Rendering**: Efficient diff-based updates minimize terminal output
- **Component System**: Build reusable UI components with a React-like API
- **Event Handling**: Mouse and keyboard event support with focus management
- **Animation Support**: Smooth transitions and animations with spring physics
- **Image Support**: Multi-backend image rendering with sixel, external tools, and terminal protocols
- **Syntax Highlighting**: Built-in syntax highlighting with customizable themes
- **Markdown Rendering**: Rich markdown support with GFM extensions

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
reactive-tui = "0.0.7"
```

Basic example:

```rust
use reactive_tui::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::builder()
        .title("My App")
        .size(80, 24)
        .component(|| {
            Element::new("div")
                .class("flex items-center justify-center bg-blue-500 text-white p-4")
                .text("Hello, Reactive-TUI!")
        })
        .build()?;
    
    app.run()
}
```

## Image Support

Display images in your terminal applications:

```rust
use reactive_tui::widgets::Image;
use reactive_tui::core::terminal::Terminal;

let image = Image::from_file("photo.jpg")
    .with_max_size(80, 24)
    .with_preserve_aspect(true);

let capabilities = Terminal::detect_image_capabilities();
let output = image.render(&capabilities)?;
println!("{}", output);
```

### Supported Image Backends

- **Sixel Graphics**: Native terminal graphics (xterm, wezterm, mlterm)
- **External Tools**: chafa and viu for broad compatibility
- **Terminal Protocols**: Kitty graphics and iTerm2 inline images
- **ASCII Art**: Fallback with Floyd-Steinberg dithering

## CSS Utility Classes

The library supports a comprehensive set of utility classes:

### Layout
- `flex`, `grid`, `flex-row`, `flex-col`
- `justify-center`, `items-center`, `place-items-center`
- `gap-2`, `gap-4`, `gap-8`

### Spacing
- `p-2`, `p-4`, `px-2`, `py-4` (padding)
- `m-2`, `m-4`, `mx-2`, `my-4` (margin)

### Sizing
- `w-full`, `h-full`, `w-32`, `h-16`
- `min-w-0`, `max-w-full`

### Colors
- `bg-red-500`, `text-blue-300`, `border-green-600`
- Named colors: `bg-primary`, `text-secondary`

### Typography
- `font-bold`, `italic`, `underline`
- `text-left`, `text-center`, `text-right`

### Images
- `image-fit-cover`, `image-fit-contain`
- `aspect-ratio-16-9`, `aspect-ratio-4-3`
- `image-quality-high`, `image-rendering-smooth`

## Examples

Run the examples to see the library in action:

```bash
# Image widget demo
cargo run --example image_widget_demo

# Animation integration
cargo run --example animation_integration_demo

# Multi-screen application
cargo run --example multi_screen_demo

# Syntax highlighting
cargo run --example syntax_highlight
```

## Documentation

- [API Documentation](https://docs.rs/reactive-tui)
- [Animation Integration Guide](docs/ANIMATION_INTEGRATION.md)
- [Adaptive Performance Guide](docs/ADAPTIVE_PERFORMANCE.md)
- [FFI ABI Policy](docs/FFI_ABI_POLICY.md)

## Terminal Compatibility

### Recommended Terminals
- **WezTerm** - Full feature support including images
- **Kitty** - Excellent performance with graphics protocol
- **Alacritty** - Fast rendering with good color support
- **iTerm2** - macOS with inline image support

### Image Support by Terminal
| Terminal | Sixel | Kitty Graphics | iTerm2 Inline | External Tools |
|----------|-------|----------------|---------------|----------------|
| WezTerm  | ✅     | ❌              | ❌             | ✅              |
| Kitty    | ❌     | ✅              | ❌             | ✅              |
| iTerm2   | ❌     | ❌              | ✅             | ✅              |
| Alacritty| ❌     | ❌              | ❌             | ✅              |

## Requirements

- Rust 1.70+
- 24-bit color terminal
- For images: chafa, viu, or compatible terminal

## Contributors

- **Shawn McAllister** ([@entrepeneur4lyf](https://github.com/entrepeneur4lyf)) - Creator and Lead Developer
- **Auggie** (Claude Sonnet 4) - Systems Architecture and Memory Management Engineering

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## Acknowledgments

- Built on [Taffy](https://github.com/DioxusLabs/taffy) for layout
- Uses [crossterm](https://github.com/crossterm-rs/crossterm) for terminal control
- Image support powered by [sixel-rs](https://github.com/saitoha/sixel-rs)
- Inspired by modern web frameworks and Tailwind CSS
