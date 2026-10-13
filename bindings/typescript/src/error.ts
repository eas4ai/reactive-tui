/**
 * Error handling utilities
 */

import { ReactiveError } from './ffi';

export class ReactiveTUIError extends Error {
  constructor(public code: ReactiveError, message?: string) {
    super(message || `Reactive TUI Error: ${ReactiveError[code]}`);
    this.name = 'ReactiveTUIError';
  }
}

/**
 * Check an FFI return code and throw if it's an error
 */
export function checkError(code: number): void {
  if (code !== ReactiveError.Success) {
    throw new ReactiveTUIError(code as ReactiveError);
  }
}