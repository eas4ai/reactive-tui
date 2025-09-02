/**
 * Terminal management API
 */

import * as ref from 'ref-napi';
import { lib, VoidPtr, ReactiveError } from './ffi';
import { checkError } from './error';

export class Terminal {
  private handle: Buffer;
  private isSetup: boolean = false;

  constructor(width: number = 80, height: number = 24) {
    const handlePtr = ref.alloc('pointer');
    checkError(lib.rtui_terminal_create(width, height, handlePtr));
    this.handle = handlePtr.deref();
    if (this.handle.isNull()) {
      throw new Error('Failed to create terminal');
    }
  }

  /**
   * Initialize the terminal
   */
  init(): void {
    checkError(lib.rtui_terminal_init(this.handle));
    this.isSetup = true;
  }

  /**
   * Shutdown the terminal
   */
  shutdown(): void {
    if (this.isSetup) {
      checkError(lib.rtui_terminal_shutdown(this.handle));
      this.isSetup = false;
    }
  }

  /**
   * Clear the terminal screen
   * Note: Clear functionality may be provided through renderer API
   */
  clear(): void {
    // Terminal clear is handled through renderer
    console.warn('Terminal clear should be done through renderer API');
  }

  /**
   * Flush any buffered output to the terminal
   * Note: Flush functionality may be provided through renderer API
   */
  flush(): void {
    // Terminal flush is handled through renderer
    console.warn('Terminal flush should be done through renderer API');
  }

  /**
   * Get the terminal size
   */
  getSize(): { width: number; height: number } {
    const dimensionsPtr = ref.alloc(ref.types.uint32);
    
    checkError(lib.rtui_terminal_get_size(this.handle, dimensionsPtr));
    
    const dimensions = dimensionsPtr.deref();
    // Dimensions are packed as width in lower 16 bits, height in upper 16 bits
    return {
      width: dimensions & 0xFFFF,
      height: (dimensions >> 16) & 0xFFFF,
    };
  }

  /**
   * Set the cursor position
   * Note: Cursor control may be provided through renderer API
   */
  setCursor(x: number, y: number): void {
    // Cursor control is handled through renderer
    console.warn('Cursor control should be done through renderer API');
  }

  /**
   * Hide the cursor
   * Note: Cursor control may be provided through renderer API
   */
  hideCursor(): void {
    // Cursor control is handled through renderer
    console.warn('Cursor control should be done through renderer API');
  }

  /**
   * Show the cursor
   * Note: Cursor control may be provided through renderer API
   */
  showCursor(): void {
    // Cursor control is handled through renderer
    console.warn('Cursor control should be done through renderer API');
  }

  /**
   * Free the terminal resources
   */
  dispose(): void {
    if (!this.handle.isNull()) {
      this.shutdown();
      lib.rtui_terminal_destroy(this.handle);
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