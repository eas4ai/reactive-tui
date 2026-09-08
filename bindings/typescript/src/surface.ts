/** Owned surface storage, or a renderer-owned view with checked lifetime. */
import { lib, output, decode, encode, readHandle, NativeHandle, unsigned } from './ffi';
import { checkError } from './error';

export interface Cell { content: string; fg: number; bg: number; }
interface Color { r: number; g: number; b: number; }
interface NativeCell { ch: number; fg: Color; bg: Color; attrs: Record<string, boolean>; }
function color(value: number): Color {
  if (!Number.isInteger(value) || value < 0 || value > 0xFFFFFF) throw new RangeError('Color must be RGB24');
  return { r: value >>> 16, g: (value >>> 8) & 255, b: value & 255 };
}
function packed(value: Color): number { return (value.r << 16) | (value.g << 8) | value.b; }

export class Surface {
  private handle: NativeHandle | null;
  private ownerAlive?: () => void;

  constructor(width: number, height: number) {
    unsigned(width, 16, 'width'); unsigned(height, 16, 'height');
    const out = output('void *');
    checkError(lib.rtui_surface_create(width, height, out));
    this.handle = readHandle(out);
  }

  /** Internal borrowed view. The renderer must stay alive and must not resize. */
  static borrowed(handle: NativeHandle, ownerAlive: () => void): Surface {
    const view = Object.create(Surface.prototype) as Surface;
    view.handle = handle;
    view.ownerAlive = ownerAlive;
    return view;
  }

  /** Recreate owned storage; resizing discards cells. */
  resize(width: number, height: number): void {
    this.getNativeHandle();
    if (this.ownerAlive) throw new Error('Resize the owning renderer instead');
    const replacement = new Surface(width, height);
    this.dispose();
    this.handle = replacement.handle;
    replacement.handle = null;
  }

  clear(r = 0, g = 0, b = 0): void {
    checkError(lib.rtui_surface_clear(this.getNativeHandle(), unsigned(r, 8, 'r'), unsigned(g, 8, 'g'), unsigned(b, 8, 'b')));
  }

  private position(x: number, y: number): void {
    unsigned(x, 16, 'x'); unsigned(y, 16, 'y');
    const size = this.getSize();
    if (x >= size.width || y >= size.height) throw new RangeError('Cell position outside surface');
  }

  setCell(x: number, y: number, content: string, fg = 0xFFFFFF, bg = 0): void {
    this.position(x, y);
    const chars = Array.from(content);
    const ch = content.codePointAt(0);
    if (chars.length !== 1 || ch === undefined || ch === 0 || (ch >= 0xD800 && ch <= 0xDFFF)) {
      throw new TypeError('Cell content must be one non-NUL Unicode scalar');
    }
    const cell = encode('RTuiCell', { ch, fg: color(fg), bg: color(bg), attrs: {
      bold: false, hidden: false, italic: false, underline: false,
      blink: false, reverse: false, strikethrough: false,
    }});
    checkError(lib.rtui_surface_set_cell(this.getNativeHandle(), x, y, cell));
  }

  getCell(x: number, y: number): Cell {
    this.position(x, y);
    const out = output('RTuiCell');
    checkError(lib.rtui_surface_get_cell(this.getNativeHandle(), x, y, out));
    const cell = decode<NativeCell>(out, 'RTuiCell');
    return { content: String.fromCodePoint(cell.ch), fg: packed(cell.fg), bg: packed(cell.bg) };
  }

  getSize(): { width: number; height: number } {
    const out = output('RTuiDimensions');
    checkError(lib.rtui_surface_get_dimensions(this.getNativeHandle(), out));
    return decode(out, 'RTuiDimensions');
  }

  dispose(): void {
    if (this.handle === null) return;
    if (!this.ownerAlive) lib.rtui_surface_destroy(this.handle);
    this.handle = null;
  }

  getNativeHandle(): NativeHandle {
    if (this.handle === null) throw new Error('Surface is disposed');
    this.ownerAlive?.();
    return this.handle;
  }
}
