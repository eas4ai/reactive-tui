import { lib, output, readHandle, decode, koffi, NativeHandle } from './ffi';
import { checkError } from './error';

/** Shared ownership primitives for controller wrappers, without finalizer threads. */
export abstract class NativeController {
  protected handle: NativeHandle | null;
  protected constructor(create: (out: Buffer) => number) {
    const out = output('void *');
    checkError(create(out));
    this.handle = readHandle(out);
  }
  getNativeHandle(): NativeHandle {
    if (this.handle === null) throw new Error('Native controller is disposed');
    return this.handle;
  }
  protected readString(getter: (out: Buffer) => number): string | null {
    const out = output('void *');
    checkError(getter(out));
    const pointer = decode<NativeHandle | null>(out, 'void *');
    if (pointer === null || pointer === 0n) return null;
    try { return koffi.decode.string(pointer); } finally { lib.rtui_string_free(pointer); }
  }
  protected readRequiredString(getter: (out: Buffer) => number): string {
    const result = this.readString(getter);
    if (result === null) throw new Error('Native controller returned a missing string');
    return result;
  }
}

interface CallbackFrame { failure?: { error: unknown }; }
const callbackFrames: CallbackFrame[] = [];

/** Keep exceptions within JS, then rethrow after native ownership is reconciled. */
export function callbackBoundary<T>(action: () => T): T {
  const frame: CallbackFrame = {};
  callbackFrames.push(frame);
  try {
    const result = action();
    if (frame.failure) throw frame.failure.error;
    return result;
  } catch (error) {
    throw frame.failure ? frame.failure.error : error;
  } finally {
    callbackFrames.pop();
  }
}

export function callbackFailed(error: unknown): void {
  const frame = callbackFrames[callbackFrames.length - 1];
  if (frame && !frame.failure) frame.failure = { error };
}

export function json(value: unknown): string {
  const encoded = JSON.stringify(value, (_key, item: unknown) => {
    if (typeof item === 'number' && !Number.isFinite(item)) throw new TypeError('Native JSON numbers must be finite');
    return item;
  });
  if (encoded === undefined) throw new TypeError('A native value must be JSON serializable');
  if (Buffer.byteLength(encoded) > 1024 * 1024) throw new RangeError('Native JSON exceeds 1 MiB');
  return encoded;
}
