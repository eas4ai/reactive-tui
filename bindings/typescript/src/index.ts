export * from './terminal';
export * from './renderer';
export * from './surface';
export * from './component';
export * from './builder';
export * from './events';
export * from './theme';
export * as layout from './layout';
export * as types from './types';
export * from './error';
export * from './native-editor';
export * from './native-layout';
export * from './native-dialog';
export * from './foreign-component';
export * from './native-app';
export { lib, ReactiveError } from './ffi';

import { lib } from './ffi';
import { checkError } from './error';

/** Explicit initialization; importing this package installs no process handlers. */
export function initialize(): void { checkError(lib.rtui_init()); }
/** Dispose all native handles before cleanup. */
export function cleanup(): void { lib.rtui_cleanup(); }
export interface Version { major: number; minor: number; patch: number; abiVersion: number; }
export function getVersion(): Version {
  const version = lib.rtui_version();
  return { major: version.major, minor: version.minor, patch: version.patch, abiVersion: version.abi_version };
}
