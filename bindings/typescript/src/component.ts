/** Static native Element trees. Dynamic JSON widget state is not a native API. */
import { lib, output, decode, readHandle, nativeString, NativeHandle, koffi } from './ffi';
import { checkError } from './error';

export interface ComponentProps { class?: string; key?: string; }
export enum ElementType { Component = 0, Text = 1, Layout = 2, Fragment = 3, Empty = 4 }

export class Component {
  private handle: NativeHandle | null;
  constructor(name: string, props: ComponentProps = {}) {
    const out = output('void *');
    checkError(lib.rtui_element_create_component(nativeString(name), out));
    this.handle = readHandle(out);
    try { this.update(props); } catch (error) { this.dispose(); throw error; }
  }
  /** Adopt a newly allocated native element; never adopt a borrowed handle. */
  static fromOwned(handle: NativeHandle): Component {
    const component = Object.create(Component.prototype) as Component;
    component.handle = handle;
    return component;
  }
  static text(content: string): Component {
    const out = output('void *');
    checkError(lib.rtui_element_text(nativeString(content), out));
    return Component.fromOwned(readHandle(out));
  }
  update(props: ComponentProps): void {
    for (const key of Object.keys(props)) {
      if (key !== 'class' && key !== 'key') throw new Error(`Unsupported native component property: ${key}`);
    }
    if (props.class !== undefined) checkError(lib.rtui_element_set_class(this.getNativeHandle(), nativeString(props.class)));
    if (props.key !== undefined) checkError(lib.rtui_element_set_key(this.getNativeHandle(), nativeString(props.key)));
  }
  /** Configure the normal App keyboard focus target. */
  focus(focusable = true, autoFocus = false): this {
    checkError(lib.rtui_element_set_focus(this.getNativeHandle(), focusable, autoFocus));
    return this;
  }
  /** Transfer child ownership to this element. */
  addChild(child: Component): this {
    const parent = this.getNativeHandle();
    if (child === this) throw new Error('An element cannot own itself');
    const handle = child.release();
    checkError(lib.rtui_element_add_child(parent, handle));
    return this;
  }
  getType(): ElementType {
    const out = output('RTuiElementType');
    checkError(lib.rtui_element_get_type(this.getNativeHandle(), out));
    return decode(out, 'RTuiElementType');
  }
  private string(getter: typeof lib.rtui_element_get_key): string | null {
    const out = output('void *');
    checkError(getter(this.getNativeHandle(), out));
    const pointer = decode<NativeHandle | null>(out, 'void *');
    if (pointer === null || pointer === 0n) return null;
    try { return koffi.decode.string(pointer); } finally { lib.rtui_string_free(pointer); }
  }
  getKey(): string | null { return this.string(lib.rtui_element_get_key); }
  getClass(): string | null { return this.string(lib.rtui_element_get_class); }
  getText(): string | null { return this.string(lib.rtui_element_get_text_content); }
  getName(): string | null { return this.string(lib.rtui_element_get_component_name); }
  getChildCount(): number {
    const out = output('size_t');
    checkError(lib.rtui_element_get_child_count(this.getNativeHandle(), out));
    return Number(decode<number | bigint>(out, 'size_t'));
  }
  /** Returns an owned clone that remains valid after parent disposal. */
  getChild(index: number): Component {
    if (!Number.isSafeInteger(index) || index < 0) throw new RangeError('Invalid child index');
    const out = output('void *');
    checkError(lib.rtui_element_get_child(this.getNativeHandle(), index, out));
    return Component.fromOwned(readHandle(out));
  }
  /** Transfer ownership to a consuming native API. */
  release(): NativeHandle {
    const handle = this.getNativeHandle();
    this.handle = null;
    return handle;
  }
  dispose(): void {
    if (this.handle === null) return;
    lib.rtui_element_destroy(this.handle);
    this.handle = null;
  }
  getNativeHandle(): NativeHandle {
    if (this.handle === null) throw new Error('Component is disposed or consumed');
    return this.handle;
  }
}

const factories = {
  div: lib.rtui_div, span: lib.rtui_span, button: lib.rtui_button,
  p: lib.rtui_p, h1: lib.rtui_h1, h2: lib.rtui_h2, h3: lib.rtui_h3,
};
export type BuilderType = keyof typeof factories;

/** Owns a native builder. build() consumes it; dispose() abandons it. */
export class ComponentBuilder {
  private handle: NativeHandle | null;
  constructor(type: BuilderType = 'div') {
    const factory = factories[type];
    if (!factory) throw new Error(`Unsupported builder type: ${type}`);
    const out = output('void *');
    checkError(factory(out));
    this.handle = readHandle(out);
  }
  private live(): NativeHandle {
    if (this.handle === null) throw new Error('Builder is disposed or consumed');
    return this.handle;
  }
  class(value: string): this {
    checkError(lib.rtui_element_builder_class(this.live(), nativeString(value))); return this;
  }
  key(value: string): this {
    checkError(lib.rtui_element_builder_key(this.live(), nativeString(value))); return this;
  }
  text(value: string): this {
    checkError(lib.rtui_element_builder_text(this.live(), nativeString(value))); return this;
  }
  prop(key: string, value: string): this {
    if (key === 'class') return this.class(value);
    if (key === 'key') return this.key(value);
    if (key === 'content' || key === 'label' || key === 'text') return this.text(value);
    throw new Error(`Unsupported native builder property: ${key}`);
  }
  props(values: Record<string, string>): this {
    for (const [key, value] of Object.entries(values)) this.prop(key, value);
    return this;
  }
  child(value: Component | ComponentBuilder): this {
    if (value === this) throw new Error('A builder cannot own itself');
    const builder = this.live();
    const child = value instanceof ComponentBuilder ? value.build() : value;
    checkError(lib.rtui_element_builder_child(builder, child.release()));
    return this;
  }
  children(...values: (Component | ComponentBuilder)[]): this {
    for (const child of values) this.child(child);
    return this;
  }
  build(): Component {
    const builder = this.live();
    const out = output('void *');
    this.handle = null;
    checkError(lib.rtui_element_builder_build(builder, out));
    return Component.fromOwned(readHandle(out));
  }
  dispose(): void {
    if (this.handle === null) return;
    lib.rtui_element_builder_destroy(this.handle);
    this.handle = null;
  }
}
export const div = () => new ComponentBuilder('div');
export const text = (content: string) => Component.text(content);
export const button = (label: string) => new ComponentBuilder('button').text(label);
export const flex = () => new ComponentBuilder('div');
