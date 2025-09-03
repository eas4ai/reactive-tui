/**
 * Hello World Example
 * Demonstrates basic usage of the Reactive TUI TypeScript SDK
 */

import { 
  initialize, 
  cleanup, 
  Terminal, 
  Renderer, 
  Surface,
  hexToRgb,
  rgbToHex,
} from '@reactive-tui/core';

async function main() {
  // Initialize the library
  initialize();
  
  let terminal: Terminal | null = null;
  let renderer: Renderer | null = null;
  let surface: Surface | null = null;
  
  try {
    // Create and initialize terminal
    terminal = new Terminal(80, 24);
    terminal.init();
    
    // Get actual terminal size
    const size = terminal.getSize();
    console.log(`Terminal size: ${size.width}x${size.height}`);
    
    // Create renderer and surface
    renderer = new Renderer(size.width, size.height);
    surface = new Surface(size.width, size.height);
    
    // Clear with a dark blue background
    const bgColor = hexToRgb('#1a1b26');
    surface.clear(bgColor.r, bgColor.g, bgColor.b);
    
    // Draw "Hello, World!" in the center
    const message = 'Hello, World!';
    const startX = Math.floor((size.width - message.length) / 2);
    const startY = Math.floor(size.height / 2);
    
    const fgColor = hexToRgb('#7aa2f7'); // Light blue
    const fgPacked = (fgColor.r << 16) | (fgColor.g << 8) | fgColor.b;
    const bgPacked = (bgColor.r << 16) | (bgColor.g << 8) | bgColor.b;
    
    // Draw each character
    for (let i = 0; i < message.length; i++) {
      surface.setCell(startX + i, startY, message[i], fgPacked, bgPacked);
    }
    
    // Draw a border
    const borderColor = hexToRgb('#9ece6a'); // Green
    const borderPacked = (borderColor.r << 16) | (borderColor.g << 8) | borderColor.b;
    
    // Top and bottom borders
    for (let x = 0; x < size.width; x++) {
      surface.setCell(x, 0, '─', borderPacked, bgPacked);
      surface.setCell(x, size.height - 1, '─', borderPacked, bgPacked);
    }
    
    // Left and right borders
    for (let y = 1; y < size.height - 1; y++) {
      surface.setCell(0, y, '│', borderPacked, bgPacked);
      surface.setCell(size.width - 1, y, '│', borderPacked, bgPacked);
    }
    
    // Corners
    surface.setCell(0, 0, '┌', borderPacked, bgPacked);
    surface.setCell(size.width - 1, 0, '┐', borderPacked, bgPacked);
    surface.setCell(0, size.height - 1, '└', borderPacked, bgPacked);
    surface.setCell(size.width - 1, size.height - 1, '┘', borderPacked, bgPacked);
    
    // Render the frame
    renderer.frame(() => {
      // Blit the surface to the renderer
      for (let y = 0; y < size.height; y++) {
        for (let x = 0; x < size.width; x++) {
          const cell = surface.getCell(x, y);
          if (cell) {
            renderer.setCell(x, y, cell.char, cell.fg, cell.bg);
          }
        }
      }
      renderer.flush();
    });
    
    // Add instructions
    const instructions = 'Press Ctrl+C to exit';
    const instrX = Math.floor((size.width - instructions.length) / 2);
    const instrY = startY + 2;
    
    const instrColor = hexToRgb('#bb9af7'); // Purple
    const instrPacked = (instrColor.r << 16) | (instrColor.g << 8) | instrColor.b;
    
    for (let i = 0; i < instructions.length; i++) {
      surface.setCell(instrX + i, instrY, instructions[i], instrPacked, bgPacked);
    }
    
    // Keep the application running
    console.log('Application is running. Press Ctrl+C to exit.');
    
    // Handle graceful shutdown
    process.on('SIGINT', () => {
      console.log('\nShutting down...');
      process.exit(0);
    });
    
    // Keep the process alive
    await new Promise(() => {});
    
  } catch (error) {
    console.error('Error:', error);
  } finally {
    // Cleanup resources
    if (surface) surface.dispose();
    if (renderer) renderer.dispose();
    if (terminal) terminal.dispose();
    cleanup();
  }
}

// Run the application
main().catch(console.error);