import { lib, output, readHandle, NativeHandle } from './ffi';
import { checkError } from './error';
import { Component } from './component';
import { ForeignComponent } from './foreign-component';
import { callbackBoundary } from './native-controller';

/** A blocking native App. This wrapper disposes its retained C handle after run. */
export class NativeApp {
  private handle: NativeHandle | null = null;
  private root: ForeignComponent<null, null>;
  constructor(render: () => Component) {
    this.root = new ForeignComponent({ props: null, state: null, render });
    let builder: NativeHandle | null = null;
    try {
      const out = output('void *');
      checkError(lib.rtui_app_builder_create(out)); builder = readHandle(out);
      checkError(lib.rtui_app_builder_backend_suprtui(builder));
      const element = this.root.element();
      try {
        checkError(lib.rtui_app_builder_root_element(builder, element.getNativeHandle()));
        element.release();
      } finally { element.dispose(); }
      const consumed = builder; builder = null;
      checkError(lib.rtui_app_builder_build(consumed, out)); this.handle = readHandle(out);
    } catch (error) {
      if (builder !== null) lib.rtui_app_builder_destroy(builder);
      this.root.dispose();
      throw error;
    }
  }
  run(): void {
    if (this.handle === null) throw new Error('App is disposed or consumed');
    const handle = this.handle; this.handle = null;
    callbackBoundary(() => {
      try { checkError(lib.rtui_app_run(handle)); }
      finally { lib.rtui_app_destroy(handle); this.root.dispose(); }
    });
  }
  dispose(): void {
    if (this.handle !== null) { lib.rtui_app_destroy(this.handle); this.handle = null; }
    this.root.dispose();
  }
}
