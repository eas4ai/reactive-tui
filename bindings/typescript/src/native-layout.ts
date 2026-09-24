import { lib, nativeString } from './ffi';
import { checkError } from './error';
import { Component } from './component';
import { NativeController } from './native-controller';

/** A validated native inline style snapshot, copied when applied to an Element. */
export class NativeLayoutStyle extends NativeController {
  constructor(css: string) { super(out => lib.rtui_native_style_create(nativeString(css), out)); }
  apply(element: Component): Component {
    checkError(lib.rtui_native_style_apply(this.getNativeHandle(), element.getNativeHandle()));
    return element;
  }
  dispose(): void {
    if (this.handle === null) return;
    checkError(lib.rtui_native_style_destroy(this.handle)); this.handle = null;
  }
}
