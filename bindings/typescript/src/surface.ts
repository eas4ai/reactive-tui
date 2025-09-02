/**
 * Surface (rendering buffer) API
 */

import * as ref from 'ref-napi';
import { lib, VoidPtr } from './ffi';
import { checkError } from './error';

export interface Cell {
  content: string;
  fg: number;
  bg: number;
}

export class Surface {
  private handle: Buffer;
  private width: number;
  private height: number;

  constructor(width: number, height: number) {
    this.width = width;
    this.height = height;
    const handlePtr = ref.alloc('pointer');
    checkError(lib.rtui_surface_create(width, height, handlePtr));
    this.handle = handlePtr.deref();
    
    if (this.handle.isNull()) {
      throw new Error(`Failed to create surface of size ${width}x${height}`);
    }
  }

  /**
   * Resize the surface
   * Note: Surface resizing may need to be done by creating a new surface
   */
  resize(width: number, height: number): void {
    // Surface resize is not directly available in C API
    // Need to recreate the surface
    console.warn('Surface resize not directly supported, consider creating new surface');
    this.width = width;
    this.height = height;
  }

  /**
   * Clear the surface with a color
   */
  clear(r: number = 0, g: number = 0, b: number = 0): void {
    checkError(lib.rtui_surface_clear(this.handle, r, g, b));
  }

  /**
   * Set a cell at the given position
   */
  setCell(x: number, y: number, content: string, fg: number = 0xFFFFFF, bg: number = 0x000000): void {
    // Create a cell structure
    const cellSize = 8 + 3 + 3 + 1; // char[8] + RGB + RGB + attributes
    const cellPtr = Buffer.alloc(cellSize);
    
    // Write character (UTF-8, max 8 bytes)
    const charBuffer = Buffer.from(content, 'utf8');
    charBuffer.copy(cellPtr, 0, 0, Math.min(charBuffer.length, 7));
    cellPtr[Math.min(charBuffer.length, 7)] = 0; // null terminator
    
    // Write foreground color (RGB)
    cellPtr[8] = (fg >> 16) & 0xFF; // R
    cellPtr[9] = (fg >> 8) & 0xFF;  // G
    cellPtr[10] = fg & 0xFF;        // B
    
    // Write background color (RGB)
    cellPtr[11] = (bg >> 16) & 0xFF; // R
    cellPtr[12] = (bg >> 8) & 0xFF;  // G
    cellPtr[13] = bg & 0xFF;         // B
    
    // Attributes (0 for now)
    cellPtr[14] = 0;
    
    checkError(lib.rtui_surface_set_cell(this.handle, x, y, cellPtr));
  }

  /**
   * Get a cell at the given position
   */
  getCell(x: number, y: number): Cell {
    const cellSize = 8 + 3 + 3 + 1; // char[8] + RGB + RGB + attributes
    const cellPtr = Buffer.alloc(cellSize);
    
    checkError(lib.rtui_surface_get_cell(this.handle, x, y, cellPtr));
    
    // Read character
    let contentEnd = 0;
    while (contentEnd < 8 && cellPtr[contentEnd] !== 0) {
      contentEnd++;
    }
    const content = cellPtr.toString('utf8', 0, contentEnd);
    
    // Read foreground color
    const fg = (cellPtr[8] << 16) | (cellPtr[9] << 8) | cellPtr[10];
    
    // Read background color
    const bg = (cellPtr[11] << 16) | (cellPtr[12] << 8) | cellPtr[13];
    
    return {
      content,
      fg,
      bg,
    };
  }

  /**
   * Get the surface dimensions
   */
  getSize(): { width: number; height: number } {
    return {
      width: this.width,
      height: this.height,
    };
  }

  /**
   * Free the surface resources
   */
  dispose(): void {
    if (!this.handle.isNull()) {
      lib.rtui_surface_destroy(this.handle);
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