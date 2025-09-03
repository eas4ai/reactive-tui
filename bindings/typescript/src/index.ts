/**
 * Reactive TUI - TypeScript SDK
 * High-level API for terminal UI applications
 */

export * from './terminal';
export * from './renderer';
export * from './surface';
export * from './component';
export * from './animation';
export * from './dialog';
export * from './data-table';
export * from './builder';
export * from './hooks';
export * from './events';
export * from './theme';
export * from './layout';
export * from './input-widgets';
export * from './types';
export * from './error';

import { lib, ReactiveError } from './ffi';

/**
 * Initialize the Reactive TUI library
 * Must be called before using any other functions
 */
export function initialize(): void {
  const result = lib.rtui_init();
  if (result !== ReactiveError.Success) {
    throw new Error(`Failed to initialize Reactive TUI: ${ReactiveError[result]}`);
  }
}

/**
 * Cleanup the Reactive TUI library
 * Should be called when done using the library
 */
export function cleanup(): void {
  lib.rtui_cleanup();
}

/**
 * Get the library version information
 */
export interface Version {
  major: number;
  minor: number;
  patch: number;
  abiVersion: number;
}

export function getVersion(): Version {
  // Note: This would need proper struct handling
  // For now, return a mock version
  return {
    major: 0,
    minor: 1,
    patch: 0,
    abiVersion: 1,
  };
}

// Auto-initialize on import (can be disabled if needed)
if (typeof process !== 'undefined' && !process.env.RTUI_NO_AUTO_INIT) {
  try {
    initialize();
    
    // Register cleanup on exit
    process.on('exit', cleanup);
    process.on('SIGINT', () => {
      cleanup();
      process.exit(0);
    });
    process.on('SIGTERM', () => {
      cleanup();
      process.exit(0);
    });
  } catch (error) {
    console.error('Failed to auto-initialize Reactive TUI:', error);
  }
}