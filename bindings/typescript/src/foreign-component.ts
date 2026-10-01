import { lib, output, readHandle, decode, koffi, NativeHandle } from './ffi';
import { checkError } from './error';
import { Component } from './component';
import { NativeController, callbackBoundary, callbackFailed, json } from './native-controller';

export interface NativeModifiers { shift: boolean; ctrl: boolean; alt: boolean; meta: boolean; }
export type NativeInput =
  | { type: 'key'; key: string; kind?: 'press' | 'release' | 'repeat'; repeat?: boolean; modifiers?: NativeModifiers }
  | { type: 'mouse'; kind: string; button: string; x: number; y: number; unit: 'cells' | 'pixels'; modifiers: NativeModifiers; wheel: { unit: 'lines' | 'pixels'; x: number; y: number; phase: string } | null }
  | { type: 'paste'; text: string }
  | { type: 'resize'; width: number; height: number; pixelWidth?: number | null; pixelHeight?: number | null }
  | { type: 'focus'; kind: string }
  | { type: 'custom'; name: string; data: unknown };
export interface ForeignComponentOptions<P, S> {
  props: P;
  state: S;
  render: (props: P, state: S) => Component;
  event?: (input: NativeInput, props: P, state: S) => boolean;
  dispose?: () => void;
}

/** One stateful foreign controller. Call dispose after its App has stopped. */
export class ForeignComponent<P, S> extends NativeController {
  private callbacks: NativeHandle[];
  constructor(options: ForeignComponentOptions<P, S>) {
    const callbacks: NativeHandle[] = [];
    const register = (callback: (...args: never[]) => unknown, type: string): NativeHandle => {
      const handle = koffi.register(callback, type); callbacks.push(handle); return handle;
    };
    try {
      const render = register((props: string, state: string, _userdata: NativeHandle, out: NativeHandle) => {
        try {
          const element = options.render(JSON.parse(props) as P, JSON.parse(state) as S);
          if (!(element instanceof Component)) throw new TypeError('Foreign render must return an owned Component');
          koffi.encode(out, 'void *', element.getNativeHandle());
          element.release();
          return 0;
        } catch (error) { callbackFailed(error); return -12; }
      }, 'RTuiForeignRenderCallback');
      const event = options.event ? register((input: string, props: string, state: string, _userdata: NativeHandle, out: NativeHandle) => {
        try {
          const handled = options.event!(JSON.parse(input) as NativeInput, JSON.parse(props) as P, JSON.parse(state) as S);
          if (typeof handled !== 'boolean') throw new TypeError('Foreign event must return a boolean');
          koffi.encode(out, 'bool', handled); return 0;
        } catch (error) { callbackFailed(error); return -12; }
      }, 'RTuiForeignEventCallback') : null;
      const dispose = register(() => {
        try { options.dispose?.(); } catch (error) { callbackFailed(error); }
      }, 'RTuiForeignDisposeCallback');
      super(out => lib.rtui_foreign_component_create(json(options.props), json(options.state), render, event, dispose, null, out));
      this.callbacks = callbacks;
    } catch (error) {
      for (const callback of callbacks) koffi.unregister(callback);
      throw error;
    }
  }
  get props(): P { return JSON.parse(this.readRequiredString(out => lib.rtui_foreign_component_get_props(this.getNativeHandle(), out))) as P; }
  set props(value: P) { checkError(lib.rtui_foreign_component_set_props(this.getNativeHandle(), json(value))); }
  get state(): S { return JSON.parse(this.readRequiredString(out => lib.rtui_foreign_component_get_state(this.getNativeHandle(), out))) as S; }
  set state(value: S) { checkError(lib.rtui_foreign_component_set_state(this.getNativeHandle(), json(value))); }
  /** Native callback error code; successful explicit render clears it. */
  get lastError(): number {
    const out = output('int32_t');
    checkError(lib.rtui_foreign_component_last_error(this.getNativeHandle(), out));
    return decode<number>(out, 'int32_t');
  }
  element(): Component {
    const out = output('void *'); checkError(lib.rtui_foreign_component_element(this.getNativeHandle(), out));
    return Component.fromOwned(readHandle(out));
  }
  /** Synchronously render a standalone snapshot. Recursive entry is rejected. */
  render(): Component {
    return callbackBoundary(() => {
      const out = output('void *'); checkError(lib.rtui_foreign_component_render(this.getNativeHandle(), out));
      return Component.fromOwned(readHandle(out));
    });
  }
  dispatch(input: NativeInput): boolean {
    return callbackBoundary(() => {
      const out = output('bool'); checkError(lib.rtui_foreign_component_dispatch(this.getNativeHandle(), json(input), out));
      return decode<boolean>(out, 'bool');
    });
  }
  dispose(): void {
    if (this.handle === null) return;
    callbackBoundary(() => {
      checkError(lib.rtui_foreign_component_destroy(this.getNativeHandle()));
      this.handle = null;
      for (const callback of this.callbacks) koffi.unregister(callback);
      this.callbacks = [];
    });
  }
}
