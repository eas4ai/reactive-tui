/**
 * Dialog API for terminal UI dialogs
 */

import { lib, VoidPtr } from './ffi';
import { checkError } from './error';

export enum DialogType {
  Alert = 0,
  Confirm = 1,
  Prompt = 2,
  Select = 3,
  MultiSelect = 4,
  FileOpen = 5,
  FileSave = 6,
}

export enum DialogButton {
  Ok = 0,
  Cancel = 1,
  Yes = 2,
  No = 3,
  Retry = 4,
  Abort = 5,
}

export interface DialogOptions {
  title?: string;
  message?: string;
  defaultValue?: string;
  placeholder?: string;
  buttons?: DialogButton[];
  items?: string[];
  multiSelect?: boolean;
  modal?: boolean;
  width?: number;
  height?: number;
}

export interface DialogResult {
  button: DialogButton;
  value?: string;
  selectedItems?: string[];
  selectedIndices?: number[];
}

export class Dialog {
  private handle: Buffer;
  private type: DialogType;
  private options: DialogOptions;

  constructor(type: DialogType, options?: DialogOptions) {
    this.type = type;
    this.options = options || {};
    
    const optionsJson = JSON.stringify(this.options);
    this.handle = lib.rtui_dialog_create(type, optionsJson);
    
    if (this.handle.isNull()) {
      throw new Error(`Failed to create dialog of type ${DialogType[type]}`);
    }
  }

  /**
   * Show the dialog and wait for user response
   */
  show(): DialogResult {
    const resultPtr = lib.rtui_dialog_show(this.handle);
    
    if (resultPtr.isNull()) {
      throw new Error('Failed to show dialog');
    }
    
    const resultJson = lib.rtui_dialog_get_result(resultPtr);
    lib.rtui_dialog_result_free(resultPtr);
    
    return JSON.parse(resultJson);
  }

  /**
   * Show the dialog asynchronously
   */
  async showAsync(): Promise<DialogResult> {
    return new Promise((resolve, reject) => {
      // In a real implementation, this would use async FFI calls
      // For now, we'll simulate async behavior
      setTimeout(() => {
        try {
          const result = this.show();
          resolve(result);
        } catch (error) {
          reject(error);
        }
      }, 0);
    });
  }

  /**
   * Update dialog options
   */
  update(options: DialogOptions): void {
    const optionsJson = JSON.stringify(options);
    checkError(lib.rtui_dialog_update(this.handle, optionsJson));
    Object.assign(this.options, options);
  }

  /**
   * Close the dialog programmatically
   */
  close(): void {
    checkError(lib.rtui_dialog_close(this.handle));
  }

  /**
   * Check if the dialog is currently visible
   */
  isVisible(): boolean {
    return lib.rtui_dialog_is_visible(this.handle) !== 0;
  }

  /**
   * Set focus to the dialog
   */
  focus(): void {
    checkError(lib.rtui_dialog_focus(this.handle));
  }

  /**
   * Free the dialog resources
   */
  dispose(): void {
    if (!this.handle.isNull()) {
      lib.rtui_dialog_free(this.handle);
      this.handle = Buffer.alloc(0);
    }
  }

  /**
   * Get the native handle for low-level operations
   */
  getNativeHandle(): Buffer {
    return this.handle;
  }
}

/**
 * Alert dialog - shows a message with OK button
 */
export async function alert(message: string, title?: string): Promise<void> {
  const dialog = new Dialog(DialogType.Alert, {
    title: title || 'Alert',
    message,
    buttons: [DialogButton.Ok],
  });
  
  try {
    await dialog.showAsync();
  } finally {
    dialog.dispose();
  }
}

/**
 * Confirm dialog - shows a message with Yes/No buttons
 */
export async function confirm(message: string, title?: string): Promise<boolean> {
  const dialog = new Dialog(DialogType.Confirm, {
    title: title || 'Confirm',
    message,
    buttons: [DialogButton.Yes, DialogButton.No],
  });
  
  try {
    const result = await dialog.showAsync();
    return result.button === DialogButton.Yes;
  } finally {
    dialog.dispose();
  }
}

/**
 * Prompt dialog - shows an input field
 */
export async function prompt(message: string, defaultValue?: string, title?: string): Promise<string | null> {
  const dialog = new Dialog(DialogType.Prompt, {
    title: title || 'Prompt',
    message,
    defaultValue,
    buttons: [DialogButton.Ok, DialogButton.Cancel],
  });
  
  try {
    const result = await dialog.showAsync();
    if (result.button === DialogButton.Ok) {
      return result.value || null;
    }
    return null;
  } finally {
    dialog.dispose();
  }
}

/**
 * Select dialog - shows a list of items to choose from
 */
export async function select(items: string[], message?: string, title?: string): Promise<number | null> {
  const dialog = new Dialog(DialogType.Select, {
    title: title || 'Select',
    message,
    items,
    buttons: [DialogButton.Ok, DialogButton.Cancel],
  });
  
  try {
    const result = await dialog.showAsync();
    if (result.button === DialogButton.Ok && result.selectedIndices) {
      return result.selectedIndices[0] || null;
    }
    return null;
  } finally {
    dialog.dispose();
  }
}

/**
 * Multi-select dialog - shows a list with checkboxes
 */
export async function multiSelect(items: string[], message?: string, title?: string): Promise<number[] | null> {
  const dialog = new Dialog(DialogType.MultiSelect, {
    title: title || 'Select Multiple',
    message,
    items,
    multiSelect: true,
    buttons: [DialogButton.Ok, DialogButton.Cancel],
  });
  
  try {
    const result = await dialog.showAsync();
    if (result.button === DialogButton.Ok) {
      return result.selectedIndices || null;
    }
    return null;
  } finally {
    dialog.dispose();
  }
}

/**
 * File open dialog
 */
export async function fileOpen(title?: string): Promise<string | null> {
  const dialog = new Dialog(DialogType.FileOpen, {
    title: title || 'Open File',
    buttons: [DialogButton.Ok, DialogButton.Cancel],
  });
  
  try {
    const result = await dialog.showAsync();
    if (result.button === DialogButton.Ok) {
      return result.value || null;
    }
    return null;
  } finally {
    dialog.dispose();
  }
}

/**
 * File save dialog
 */
export async function fileSave(defaultName?: string, title?: string): Promise<string | null> {
  const dialog = new Dialog(DialogType.FileSave, {
    title: title || 'Save File',
    defaultValue: defaultName,
    buttons: [DialogButton.Ok, DialogButton.Cancel],
  });
  
  try {
    const result = await dialog.showAsync();
    if (result.button === DialogButton.Ok) {
      return result.value || null;
    }
    return null;
  } finally {
    dialog.dispose();
  }
}

/**
 * Progress dialog for long-running operations
 */
export class ProgressDialog {
  private dialog: Dialog;
  private currentProgress: number = 0;

  constructor(title: string, message?: string) {
    this.dialog = new Dialog(DialogType.Alert, {
      title,
      message: message || 'Processing...',
      modal: true,
    });
  }

  /**
   * Update the progress (0-100)
   */
  setProgress(progress: number): void {
    this.currentProgress = Math.min(100, Math.max(0, progress));
    const progressBar = '█'.repeat(Math.floor(this.currentProgress / 5)) + 
                       '░'.repeat(20 - Math.floor(this.currentProgress / 5));
    
    this.dialog.update({
      message: `${this.dialog['options'].message}\n[${progressBar}] ${this.currentProgress}%`,
    });
  }

  /**
   * Update the message
   */
  setMessage(message: string): void {
    this.dialog.update({ message });
  }

  /**
   * Close the progress dialog
   */
  close(): void {
    this.dialog.close();
    this.dialog.dispose();
  }
}