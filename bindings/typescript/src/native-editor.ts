import { lib, output, readHandle, nativeString, unsigned } from './ffi';
import { checkError } from './error';
import { Component } from './component';
import { NativeController } from './native-controller';

const movements = { left: 0, right: 1, up: 2, down: 3, home: 4, end: 5, documentStart: 6, documentEnd: 7, wordBackward: 8, wordForward: 9 };
export type EditorMovement = keyof typeof movements;

/** Native Unicode editor. Rendering returns an owned snapshot; route input explicitly. */
export class NativeTextEditor extends NativeController {
  constructor() { super(out => lib.rtui_text_editor_create(out)); }
  get content(): string { return this.readRequiredString(out => lib.rtui_text_editor_get_content_owned(this.getNativeHandle(), out)); }
  set content(value: string) { checkError(lib.rtui_text_editor_set_content(this.getNativeHandle(), nativeString(value))); }
  insert(value: string): void { checkError(lib.rtui_text_editor_insert_text(this.getNativeHandle(), nativeString(value))); }
  delete(backward = true): void { checkError(lib.rtui_text_editor_delete(this.getNativeHandle(), backward)); }
  move(movement: EditorMovement, select = false): void {
    if (!Object.hasOwnProperty.call(movements, movement)) throw new RangeError('Unknown editor movement');
    checkError(lib.rtui_text_editor_move(this.getNativeHandle(), movements[movement], select));
  }
  setSize(width: number, height: number): void {
    checkError(lib.rtui_text_editor_set_size(this.getNativeHandle(), unsigned(width, 16, 'width'), unsigned(height, 16, 'height')));
  }
  showLineNumbers(show: boolean): void { checkError(lib.rtui_text_editor_set_show_line_numbers(this.getNativeHandle(), show)); }
  element(): Component {
    const out = output('void *'); checkError(lib.rtui_text_editor_element(this.getNativeHandle(), out));
    return Component.fromOwned(readHandle(out));
  }
  dispose(): void {
    if (this.handle === null) return;
    checkError(lib.rtui_text_editor_destroy(this.handle)); this.handle = null;
  }
}
