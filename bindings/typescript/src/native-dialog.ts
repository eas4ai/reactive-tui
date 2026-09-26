import { lib, output, readHandle, decode, unsigned } from './ffi';
import { checkError } from './error';
import { Component } from './component';
import { NativeController, json } from './native-controller';

export type NativeDialogOptions =
  | { kind: 'confirmation'; title: string; message?: string }
  | { kind: 'input'; title: string; prompt?: string; value?: string; placeholder?: string }
  | { kind: 'toast'; title: string; message?: string; duration?: number }
  | { kind: 'progress'; title: string; message?: string; progress?: number }
  | { kind: 'autocomplete'; title: string; prompt?: string; suggestions?: string[] }
  | { kind: 'wizard'; title: string; steps?: { id: string; title: string; content: string; optional?: boolean }[] };
export type NativeDialogUpdate = { progress: number } | { input: string } | { wizardData: Record<string, string> } | { zIndex: number };
export type NativeDialogResult = { kind: 'confirmed'; data?: string | null } | { kind: 'cancelled'; data?: never } | { kind: 'selected' | 'custom' | 'error'; data: string };
export interface NativeDialogEvent { kind: 'opened' | 'closed'; id: number; result?: NativeDialogResult; }

/** Controller for native dialogs; render element() inside one running App. */
export class NativeDialogEngine extends NativeController {
  constructor() { super(out => lib.rtui_dialog_engine_create(out)); }
  open(options: NativeDialogOptions): number {
    const out = output('uint32_t');
    checkError(lib.rtui_dialog_engine_open(this.getNativeHandle(), json(options), out));
    return decode<number>(out, 'uint32_t');
  }
  update(id: number, update: NativeDialogUpdate): void {
    checkError(lib.rtui_dialog_engine_update(this.getNativeHandle(), unsigned(id, 32, 'dialog id'), json(update)));
  }
  close(id: number, result: NativeDialogResult): void {
    checkError(lib.rtui_dialog_engine_close(this.getNativeHandle(), unsigned(id, 32, 'dialog id'), json(result)));
  }
  takeEvent(): NativeDialogEvent | null {
    const event = this.readString(out => lib.rtui_dialog_engine_take_event(this.getNativeHandle(), out));
    return event === null ? null : JSON.parse(event) as NativeDialogEvent;
  }
  element(): Component {
    const out = output('void *'); checkError(lib.rtui_dialog_engine_element(this.getNativeHandle(), out));
    return Component.fromOwned(readHandle(out));
  }
  dispose(): void {
    if (this.handle === null) return;
    checkError(lib.rtui_dialog_engine_destroy(this.handle)); this.handle = null;
  }
}
