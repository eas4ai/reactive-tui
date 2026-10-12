import { lib, output, readHandle, NativeHandle, unsigned } from './ffi';
import { checkError } from './error';
import { Surface } from './surface';

export class Renderer {
  private handle: NativeHandle | null;
  private generation = 0;
  constructor(width: number, height: number) {
    const out = output('void *');
    checkError(lib.rtui_renderer_create(unsigned(width, 16, 'width'), unsigned(height, 16, 'height'), out));
    this.handle = readHandle(out);
  }
  resize(width: number, height: number): void {
    checkError(lib.rtui_renderer_resize(this.getNativeHandle(), unsigned(width, 16, 'width'), unsigned(height, 16, 'height')));
    this.generation++;
  }
  clear(r = 0, g = 0, b = 0): void {
    checkError(lib.rtui_renderer_clear(this.getNativeHandle(), unsigned(r, 8, 'r'), unsigned(g, 8, 'g'), unsigned(b, 8, 'b')));
  }
  beginFrame(): void { checkError(lib.rtui_renderer_frame(this.getNativeHandle(), true)); }
  endFrame(): void { checkError(lib.rtui_renderer_frame(this.getNativeHandle(), false)); }
  frame(callback: () => void): void {
    this.beginFrame();
    try { callback(); } finally { this.endFrame(); }
  }
  getSurface(): Surface {
    const out = output('void *');
    checkError(lib.rtui_renderer_get_surface(this.getNativeHandle(), out));
    const generation = this.generation;
    return Surface.borrowed(readHandle(out), () => {
      this.getNativeHandle();
      if (generation !== this.generation) throw new Error('Surface view expired after renderer resize');
    });
  }
  getSize(): { width: number; height: number } {
    const surface = this.getSurface();
    try { return surface.getSize(); } finally { surface.dispose(); }
  }
  dispose(): void {
    if (this.handle === null) return;
    lib.rtui_renderer_destroy(this.handle);
    this.handle = null;
  }
  getNativeHandle(): NativeHandle {
    if (this.handle === null) throw new Error('Renderer is disposed');
    return this.handle;
  }
}
