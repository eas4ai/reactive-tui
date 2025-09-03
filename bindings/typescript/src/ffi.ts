/**
 * Low-level FFI bindings to the reactive-tui Rust library
 */

import * as ffi from 'ffi-napi';
import * as ref from 'ref-napi';
import * as path from 'path';
import * as fs from 'fs';

// Basic FFI types
export const VoidPtr = ref.refType(ref.types.void);
export const StringPtr = ref.refType(ref.types.CString);

// Error codes matching C header RTuiError enum
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
  Panic = -99,
  Unknown = -100,
}

// Version struct
export const RTuiVersion = ref.types.void; // Will be defined as struct

// Find the native library
function findNativeLibrary(): string {
  const libName = process.platform === 'win32' 
    ? 'reactive_tui.dll'
    : process.platform === 'darwin'
    ? 'libreactive_tui.dylib'
    : 'libreactive_tui.so';

  // Try multiple paths
  const possiblePaths = [
    // Local development path
    path.join(__dirname, '..', '..', '..', 'target', 'release', libName),
    // Installed path
    path.join(__dirname, '..', 'native', libName),
    // Alternative development path
    path.join(__dirname, '..', '..', '..', 'target', 'debug', libName),
  ];

  for (const libPath of possiblePaths) {
    if (fs.existsSync(libPath)) {
      return libPath;
    }
  }

  throw new Error(`Could not find native library ${libName}. Tried paths: ${possiblePaths.join(', ')}`);
}

// Load the native library
const libPath = findNativeLibrary();
export const lib = ffi.Library(libPath, {
  // Core functions
  'rtui_init': ['int', []],
  'rtui_cleanup': ['void', []],
  'rtui_version': [RTuiVersion, []],

  // Terminal API - matching C header
  'rtui_terminal_create': ['int', ['uint16', 'uint16', 'pointer']],
  'rtui_terminal_destroy': ['void', [VoidPtr]],
  'rtui_terminal_init': ['int', [VoidPtr]],
  'rtui_terminal_shutdown': ['int', [VoidPtr]],
  'rtui_terminal_get_size': ['int', [VoidPtr, 'pointer']],

  // Renderer API - matching C header
  'rtui_renderer_create': ['int', ['uint16', 'uint16', 'pointer']],
  'rtui_renderer_destroy': ['void', [VoidPtr]],
  'rtui_renderer_resize': ['int', [VoidPtr, 'uint16', 'uint16']],
  'rtui_renderer_clear': ['int', [VoidPtr, 'uint8', 'uint8', 'uint8']],
  'rtui_renderer_frame': ['int', [VoidPtr, 'bool']],

  // Surface API - matching C header
  'rtui_surface_create': ['int', ['uint16', 'uint16', 'pointer']],
  'rtui_surface_destroy': ['void', [VoidPtr]],
  'rtui_surface_get_dimensions': ['int', [VoidPtr, 'pointer']],
  'rtui_surface_clear': ['int', [VoidPtr, 'uint8', 'uint8', 'uint8']],
  'rtui_surface_set_cell': ['int', [VoidPtr, 'uint16', 'uint16', 'pointer']],
  'rtui_surface_get_cell': ['int', [VoidPtr, 'uint16', 'uint16', 'pointer']],

  // Component functions
  'rtui_component_create': [VoidPtr, ['string', 'string']],
  'rtui_component_free': ['void', [VoidPtr]],
  'rtui_component_render': ['int', [VoidPtr, VoidPtr]],
  'rtui_component_update': ['int', [VoidPtr, 'string']],
  'rtui_component_handle_event': ['int', [VoidPtr, 'pointer']],
  'rtui_component_get_state': ['string', [VoidPtr]],
  'rtui_component_set_state': ['int', [VoidPtr, 'string']],

  // Animation functions
  'rtui_animation_create': [VoidPtr, ['int', 'double', 'double', 'uint32', 'int', 'uint32', 'int', 'bool']],
  'rtui_animation_free': ['void', [VoidPtr]],
  'rtui_animation_start': ['int', [VoidPtr]],
  'rtui_animation_stop': ['int', [VoidPtr]],
  'rtui_animation_pause': ['int', [VoidPtr]],
  'rtui_animation_resume': ['int', [VoidPtr]],
  'rtui_animation_reset': ['int', [VoidPtr]],
  'rtui_animation_update': ['int', [VoidPtr, 'double']],
  'rtui_animation_is_running': ['bool', [VoidPtr]],
  'rtui_animation_is_complete': ['bool', [VoidPtr]],
  'rtui_animation_get_progress': ['double', [VoidPtr]],
  'rtui_animation_set_progress': ['int', [VoidPtr, 'double']],
  
  // Animation group functions
  'rtui_animation_group_create': [VoidPtr, []],
  'rtui_animation_group_free': ['void', [VoidPtr]],
  'rtui_animation_group_add': ['int', [VoidPtr, VoidPtr]],
  'rtui_animation_group_remove': ['int', [VoidPtr, VoidPtr]],
  'rtui_animation_group_start': ['int', [VoidPtr]],
  'rtui_animation_group_stop': ['int', [VoidPtr]],
  'rtui_animation_group_pause': ['int', [VoidPtr]],
  'rtui_animation_group_resume': ['int', [VoidPtr]],
  'rtui_animation_group_update': ['int', [VoidPtr, 'double']],
  'rtui_animation_group_is_running': ['bool', [VoidPtr]],

  // Dialog functions
  'rtui_dialog_create': [VoidPtr, ['int', 'string']],
  'rtui_dialog_free': ['void', [VoidPtr]],
  'rtui_dialog_show': [VoidPtr, [VoidPtr]],
  'rtui_dialog_show_async': [VoidPtr, [VoidPtr, 'string', 'pointer']], // async with callback
  'rtui_dialog_close': ['int', [VoidPtr]],
  'rtui_dialog_update': ['int', [VoidPtr, 'string']],
  'rtui_dialog_is_visible': ['bool', [VoidPtr]],
  'rtui_dialog_focus': ['int', [VoidPtr]],
  'rtui_dialog_get_result': ['string', [VoidPtr]],
  'rtui_dialog_result_free': ['void', [VoidPtr]],
});