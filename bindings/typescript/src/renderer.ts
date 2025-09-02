/**
 * Renderer API for drawing to terminal
 */

import * as ref from 'ref-napi';
import { lib, VoidPtr } from './ffi';
import { checkError } from './error';

export class Renderer {
  private handle: Buffer;
  private width: number;
  private height: number;

  constructor(width: number, height: number) {
    this.width = width;
    this.height = height;
    const handlePtr = ref.alloc('pointer');
    checkError(lib.rtui_renderer_create(width, height, handlePtr));
    this.handle = handlePtr.deref();
    
    if (this.handle.isNull()) {
      throw new Error(`Failed to create renderer of size ${width}x${height}`);
    }
  }

  /**
   * Resize the renderer
   */
  resize(width: number, height: number): void {
    checkError(lib.rtui_renderer_resize(this.handle, width, height));
    this.width = width;
    this.height = height;
  }

  /**
   * Clear the renderer with a background color
   */
  clear(r: number = 0, g: number = 0, b: number = 0): void {
    checkError(lib.rtui_renderer_clear(this.handle, r, g, b));
  }

  /**
   * Begin a new frame
   */
  beginFrame(): void {
    checkError(lib.rtui_renderer_frame(this.handle, true));
  }

  /**
   * End the current frame and flush to terminal
   */
  endFrame(): void {
    checkError(lib.rtui_renderer_frame(this.handle, false));
  }

  /**
   * Render a complete frame with a callback
   */
  frame(callback: () => void): void {
    this.beginFrame();
    try {
      callback();
    } finally {
      this.endFrame();
    }
  }

  /**
   * Get the renderer dimensions
   */
  getSize(): { width: number; height: number } {
    return {
      width: this.width,
      height: this.height,
    };
  }

  /**
   * Free the renderer resources
   */
  dispose(): void {
    if (!this.handle.isNull()) {
      lib.rtui_renderer_destroy(this.handle);
      this.handle = Buffer.alloc(0);
    }
  }

  /**
   * Get the native handle for low-level operations
   */
  getNativeHandle(): Buffer {
    return this.handle;
  }
}