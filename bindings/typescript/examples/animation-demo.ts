/**
 * Animation Demo
 * Demonstrates animation capabilities of the Reactive TUI TypeScript SDK
 */

import {
  initialize,
  cleanup,
  Terminal,
  Renderer,
  Surface,
  Animation,
  AnimationProperty,
  AnimationType,
  AnimationGroup,
  fadeIn,
  fadeOut,
  bounce,
  spring,
  hexToRgb,
} from '@reactive-tui/core';

class AnimatedBox {
  x: number;
  y: number;
  width: number;
  height: number;
  color: string;
  animation: Animation | null = null;

  constructor(x: number, y: number, width: number, height: number, color: string) {
    this.x = x;
    this.y = y;
    this.width = width;
    this.height = height;
    this.color = color;
  }

  draw(surface: Surface): void {
    const rgb = hexToRgb(this.color);
    const packed = (rgb.r << 16) | (rgb.g << 8) | rgb.b;
    const bgPacked = 0x000000;

    // Draw filled box
    for (let dy = 0; dy < this.height; dy++) {
      for (let dx = 0; dx < this.width; dx++) {
        const char = (dy === 0 || dy === this.height - 1 || 
                     dx === 0 || dx === this.width - 1) ? '█' : '░';
        surface.setCell(
          Math.floor(this.x) + dx,
          Math.floor(this.y) + dy,
          char,
          packed,
          bgPacked
        );
      }
    }
  }

  animatePosition(toX: number, toY: number, duration: number = 1000): void {
    // Create horizontal animation
    const xAnim = new Animation(
      AnimationProperty.X,
      this.x,
      toX,
      {
        duration,
        type: AnimationType.EaseInOut,
        onProgress: (progress) => {
          this.x = this.x + (toX - this.x) * progress;
        }
      }
    );

    // Create vertical animation
    const yAnim = new Animation(
      AnimationProperty.Y,
      this.y,
      toY,
      {
        duration,
        type: AnimationType.EaseInOut,
        onProgress: (progress) => {
          this.y = this.y + (toY - this.y) * progress;
        }
      }
    );

    // Group animations to run together
    const group = new AnimationGroup();
    group.add(xAnim);
    group.add(yAnim);
    group.start();

    this.animation = xAnim; // Keep reference for cleanup
  }

  dispose(): void {
    if (this.animation) {
      this.animation.dispose();
    }
  }
}

async function main() {
  initialize();

  let terminal: Terminal | null = null;
  let renderer: Renderer | null = null;
  let surface: Surface | null = null;
  const boxes: AnimatedBox[] = [];

  try {
    // Setup terminal and renderer
    terminal = new Terminal(100, 30);
    terminal.init();
    
    const size = terminal.getSize();
    renderer = new Renderer(size.width, size.height);
    surface = new Surface(size.width, size.height);

    // Create animated boxes
    boxes.push(new AnimatedBox(10, 5, 8, 4, '#ff79c6'));  // Pink
    boxes.push(new AnimatedBox(30, 10, 10, 5, '#50fa7b')); // Green
    boxes.push(new AnimatedBox(50, 15, 6, 3, '#f1fa8c'));  // Yellow
    boxes.push(new AnimatedBox(70, 8, 12, 6, '#8be9fd'));  // Cyan

    // Start animations
    boxes[0].animatePosition(70, 20, 2000);
    boxes[1].animatePosition(10, 20, 2500);
    boxes[2].animatePosition(40, 5, 1800);
    boxes[3].animatePosition(20, 5, 2200);

    // Animation loop
    let lastTime = Date.now();
    const animationLoop = setInterval(() => {
      const currentTime = Date.now();
      const deltaTime = (currentTime - lastTime) / 1000.0;
      lastTime = currentTime;

      // Clear surface
      surface.clear(26, 27, 38); // Dark background

      // Update and draw boxes
      for (const box of boxes) {
        if (box.animation) {
          box.animation.update(deltaTime);
        }
        box.draw(surface);
      }

      // Draw title
      const title = '✨ Animation Demo ✨';
      const titleX = Math.floor((size.width - title.length) / 2);
      const titleColor = hexToRgb('#bd93f9');
      const titlePacked = (titleColor.r << 16) | (titleColor.g << 8) | titleColor.b;
      
      for (let i = 0; i < title.length; i++) {
        surface.setCell(titleX + i, 1, title[i], titlePacked, 0x000000);
      }

      // Render frame
      renderer.frame(() => {
        // Surface would be blitted to renderer here
      });

      // Check if animations are complete
      let allComplete = true;
      for (const box of boxes) {
        if (box.animation && !box.animation.isComplete()) {
          allComplete = false;
          break;
        }
      }

      if (allComplete) {
        // Restart animations with new targets
        boxes[0].animatePosition(10, 5, 2000);
        boxes[1].animatePosition(30, 10, 2500);
        boxes[2].animatePosition(50, 15, 1800);
        boxes[3].animatePosition(70, 8, 2200);
      }
    }, 16); // ~60 FPS

    // Handle shutdown
    process.on('SIGINT', () => {
      clearInterval(animationLoop);
      console.log('\nAnimation stopped.');
      process.exit(0);
    });

    console.log('Animation demo running. Press Ctrl+C to exit.');
    
    // Keep process alive
    await new Promise(() => {});

  } catch (error) {
    console.error('Error:', error);
  } finally {
    // Cleanup
    for (const box of boxes) {
      box.dispose();
    }
    if (surface) surface.dispose();
    if (renderer) renderer.dispose();
    if (terminal) terminal.dispose();
    cleanup();
  }
}

main().catch(console.error);