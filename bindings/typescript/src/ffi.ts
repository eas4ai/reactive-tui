/** Native declarations generated from the compiler-audited C API. */
import * as koffi from 'koffi';
import * as path from 'path';
import * as fs from 'fs';
import schema from './native-api.json';
import type { NativeFunctionName } from './native-types';
export type { NativeFunctionName } from './native-types';

export enum ReactiveError {
  Success = 0,
  InvalidParameter = -1,
  NullPointer = -2,
  BufferTooSmall = -3,
  OutOfMemory = -4,
  InvalidUtf8 = -5,
  TerminalNotAvailable = -6,
  NotSupported = -7,
  AlreadyExists = -8,
  NotFound = -9,
  InvalidState = -10,
  InvalidPointer = -11,
  InternalError = -12,
  Panic = -99,
  Unknown = -100,
}

// Layouts and signatures are checked against independent rustc and clang probes.
for (const name of schema.opaque) koffi.opaque(name);
for (const name of schema.enums) koffi.alias(name, 'int');
for (const [name, record] of Object.entries(schema.records)) {
  if (record.kind === 'union') koffi.union(name, record.fields);
  else koffi.struct(name, record.fields);
}
for (const [name, signature] of Object.entries(schema.callbacks)) {
  const declaration = signature.replace('(*)', name + 'Callback');
  koffi.alias(name, koffi.pointer(koffi.proto(declaration)));
}

function findNativeLibrary(): string {
  if (process.env.RTUI_LIBRARY_PATH) return path.resolve(process.env.RTUI_LIBRARY_PATH);
  const filename = process.platform === 'win32' ? 'reactive_tui.dll'
    : process.platform === 'darwin' ? 'libreactive_tui.dylib' : 'libreactive_tui.so';
  const candidates = [
    path.join(__dirname, '..', 'native', filename),
    path.join(__dirname, '..', '..', '..', 'target', 'release', filename),
    path.join(__dirname, '..', '..', '..', 'target', 'debug', filename),
  ];
  const found = candidates.find(candidate => fs.existsSync(candidate));
  if (!found) throw new Error(`Native library missing. Build with cargo build --features ffi, or set RTUI_LIBRARY_PATH. Tried: ${candidates.join(', ')}`);
  return found;
}

const library = koffi.load(findNativeLibrary());
type NativeFunction = ReturnType<typeof library.func>;
function parameterType(type: string): string {
  return Object.entries(schema.callbacks).find(([, signature]) => signature === type)?.[0] ?? type;
}
// Koffi converts char* results to JS strings. Free the owned allocation after
// conversion with Rust's allocator, not Koffi's default C free().
const ownedString = koffi.disposable('char *', (pointer: NativeHandle) => lib.rtui_string_free(pointer));
export const lib = Object.fromEntries(Object.entries(schema.functions).map(([name, entry]) =>
  [name, library.func(name, name === 'rtui_signal_get_string_owned' ? ownedString : entry.result,
    entry.parameters.map(parameterType))]
)) as Record<NativeFunctionName, NativeFunction>;

// Buffer-backed output storage stays alive for each synchronous call. Native
// pointers returned in it remain owned by their caller until explicitly freed.
export function output(type: string): Buffer { return Buffer.alloc(koffi.sizeof(type)); }
export function decode<T>(buffer: Buffer, type: string): T { return koffi.decode(buffer, type) as T; }
export function encode(type: string, value: unknown): Buffer {
  const buffer = output(type);
  koffi.encode(buffer, type, value);
  return buffer;
}
export type NativeHandle = bigint;
export function readHandle(buffer: Buffer): NativeHandle {
  const handle = decode<NativeHandle | null>(buffer, 'void *');
  if (handle === null || handle === 0n) throw new Error('Native function returned a null handle');
  return handle;
}
export function nativeString(value: string): string {
  if (value.includes('\0')) throw new TypeError('Native strings cannot contain NUL');
  return value;
}
export function unsigned(value: number, bits: 8 | 16 | 32, name: string): number {
  if (!Number.isInteger(value) || value < 0 || value > 2 ** bits - 1) {
    throw new RangeError(`${name} must be an unsigned ${bits}-bit integer`);
  }
  return value;
}
export { koffi };
