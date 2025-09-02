# Reactive TUI TypeScript SDK

TypeScript/JavaScript bindings for the Reactive TUI library, providing a high-level API for building terminal user interfaces.

## Installation

```bash
npm install @reactive-tui/core
```

## Features

- **Terminal Management**: Initialize and control terminal settings
- **Rendering System**: Efficient double-buffered rendering with frame management
- **Surface API**: Low-level drawing operations for cells and colors
- **Component System**: Declarative UI components with builder pattern
- **Animation Framework**: Smooth animations with easing functions
- **Dialog System**: Built-in dialogs (alert, confirm, prompt, select)
- **Type Safety**: Full TypeScript support with comprehensive type definitions

## Quick Start

```typescript
import { initialize, cleanup, Terminal, Renderer, Surface } from '@reactive-tui/core';

// Initialize the library
initialize();

// Create terminal and renderer
const terminal = new Terminal(80, 24);
terminal.init();

const renderer = new Renderer(80, 24);
const surface = new Surface(80, 24);

// Draw something
surface.clear(0, 0, 0);
surface.setCell(10, 10, 'H', 0xFFFFFF, 0x000000);

// Render frame
renderer.frame(() => {
  // Rendering logic here
});

// Cleanup when done
surface.dispose();
renderer.dispose();
terminal.dispose();
cleanup();
```

## API Overview

### Core Functions

- `initialize()` - Initialize the library (called automatically on import)
- `cleanup()` - Clean up resources
- `getVersion()` - Get library version information

### Terminal Class

```typescript
const terminal = new Terminal(width, height);
terminal.init();                    // Initialize terminal
terminal.getSize();                 // Get terminal dimensions
terminal.shutdown();                // Shutdown terminal
terminal.dispose();                 // Free resources
```

### Renderer Class

```typescript
const renderer = new Renderer(width, height);
renderer.resize(newWidth, newHeight);     // Resize renderer
renderer.clear(r, g, b);                  // Clear with color
renderer.beginFrame();                     // Start frame
renderer.endFrame();                       // End frame
renderer.frame(callback);                  // Render complete frame
```

### Surface Class

```typescript
const surface = new Surface(width, height);
surface.clear(r, g, b);                          // Clear surface
surface.setCell(x, y, char, fg, bg);            // Set cell
surface.getCell(x, y);                          // Get cell
surface.getSize();                               // Get dimensions
```

### Component System

```typescript
import { div, text, button, flex, grid } from '@reactive-tui/core';

const ui = div()
  .prop('style', { padding: 2 })
  .children(
    text('Hello World'),
    button('Click me').prop('onClick', handleClick),
    flex().children(
      text('Item 1'),
      text('Item 2')
    )
  );

const component = ui.build();
component.render(surface);
```

### Animation

```typescript
import { Animation, AnimationType, AnimationProperty } from '@reactive-tui/core';

const animation = new Animation(
  AnimationProperty.X,
  0,    // from
  100,  // to
  {
    duration: 1000,
    type: AnimationType.EaseInOut,
    onComplete: () => console.log('Done!')
  }
);

animation.start();
animation.update(deltaTime);
```

### Dialogs

```typescript
import { alert, confirm, prompt, select } from '@reactive-tui/core';

await alert('Hello!', 'Title');
const result = await confirm('Are you sure?');
const name = await prompt('Enter your name:');
const choice = await select(['Option 1', 'Option 2'], 'Choose one:');
```

## Examples

See the `examples/` directory for complete examples:

- `hello-world.ts` - Basic rendering example
- `animation-demo.ts` - Animation showcase
- `component-demo.ts` - Component system demonstration

Run examples with:

```bash
npm run example:hello
npm run example:animation
npm run example:component
```

## Requirements

- Node.js 14+
- Native library (`libreactive_tui.so` / `.dylib` / `.dll`)

## Building from Source

```bash
# Build the Rust library
cd ../..
cargo build --release

# Install TypeScript dependencies
cd bindings/typescript
npm install

# Build TypeScript
npm run build
```

## License

MIT License - See LICENSE file for details