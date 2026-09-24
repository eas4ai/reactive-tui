import { lib, output, decode, readHandle, NativeHandle } from './ffi';
import { checkError } from './error';

export interface Capabilities {
  rgb: boolean; color_256: boolean; unicode_level: number;
  kitty_keyboard: boolean; mouse: boolean; pixel_mouse: boolean;
  hyperlinks: boolean; images: boolean; synchronized_output: boolean; bracketed_paste: boolean;
}

/** Uses the host terminal dimensions. Dispose restores the terminal. */
export class Terminal {
  private handle: NativeHandle | null;
  private cursor = { x: 1, y: 1, visible: true };
  constructor() {
    const out = output('void *');
    checkError(lib.rtui_terminal_create(out));
    this.handle = readHandle(out);
  }
  /** The existing native setup API has no error result. */
  init(): void { lib.setupTerminal(this.getNativeHandle(), false); }
  /** Shutdown releases this handle. Construct a new Terminal to reopen it. */
  shutdown(): void { this.dispose(); }
  clear(): void { lib.clearTerminal(this.getNativeHandle()); }
  getSize(): { width: number; height: number } {
    const out = output('RTuiDimensions');
    checkError(lib.rtui_terminal_get_dimensions(this.getNativeHandle(), out));
    return decode(out, 'RTuiDimensions');
  }
  getCapabilities(): Capabilities {
    const out = output('RTuiCapabilities');
    lib.getTerminalCapabilities(this.getNativeHandle(), out);
    return decode(out, 'RTuiCapabilities');
  }
  /** Cursor coordinates are one-based. */
  setCursor(x: number, y: number): void {
    if (![x, y].every(value => Number.isInteger(value) && value >= 1 && value <= 65535)) {
      throw new RangeError('Cursor coordinates must be in 1..65535');
    }
    lib.setCursorPosition(this.getNativeHandle(), x, y, this.cursor.visible);
    this.cursor = { x, y, visible: this.cursor.visible };
  }
  hideCursor(): void {
    lib.setCursorPosition(this.getNativeHandle(), this.cursor.x, this.cursor.y, false);
    this.cursor.visible = false;
  }
  showCursor(): void {
    lib.setCursorPosition(this.getNativeHandle(), this.cursor.x, this.cursor.y, true);
    this.cursor.visible = true;
  }
  dispose(): void {
    if (this.handle === null) return;
    lib.rtui_terminal_destroy(this.handle);
    this.handle = null;
  }
  getNativeHandle(): NativeHandle {
    if (this.handle === null) throw new Error('Terminal is disposed');
    return this.handle;
  }
}
