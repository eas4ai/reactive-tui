/**
 * Component system API
 */

import { lib, VoidPtr } from './ffi';
import { checkError } from './error';
import { Surface } from './surface';

export interface ComponentProps {
  [key: string]: any;
}

export interface ComponentState {
  [key: string]: any;
}

export class Component {
  private handle: Buffer;

  constructor(type: string, props?: ComponentProps) {
    const propsJson = JSON.stringify(props || {});
    this.handle = lib.rtui_component_create(type, propsJson);
    
    if (this.handle.isNull()) {
      throw new Error(`Failed to create component of type: ${type}`);
    }
  }

  /**
   * Render the component to a surface
   */
  render(surface: Surface): void {
    checkError(lib.rtui_component_render(this.handle, surface.getNativeHandle()));
  }

  /**
   * Update component props
   */
  update(props: ComponentProps): void {
    const propsJson = JSON.stringify(props);
    checkError(lib.rtui_component_update(this.handle, propsJson));
  }

  /**
   * Handle an event
   */
  handleEvent(event: any): void {
    // Event handling would need proper struct marshaling
    // This is a simplified version
    const eventPtr = Buffer.from(JSON.stringify(event));
    checkError(lib.rtui_component_handle_event(this.handle, eventPtr));
  }

  /**
   * Get component state
   */
  getState(): ComponentState {
    const stateJson = lib.rtui_component_get_state(this.handle);
    return JSON.parse(stateJson);
  }

  /**
   * Set component state
   */
  setState(state: ComponentState): void {
    const stateJson = JSON.stringify(state);
    checkError(lib.rtui_component_set_state(this.handle, stateJson));
  }

  /**
   * Free the component resources
   */
  dispose(): void {
    if (!this.handle.isNull()) {
      lib.rtui_component_free(this.handle);
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
 * Component builder for declarative UI
 */
export class ComponentBuilder {
  private type: string;
  private props: ComponentProps = {};
  private children: ComponentBuilder[] = [];

  constructor(type: string) {
    this.type = type;
  }

  /**
   * Set a prop
   */
  prop(key: string, value: any): this {
    this.props[key] = value;
    return this;
  }

  /**
   * Set multiple props
   */
  props(props: ComponentProps): this {
    Object.assign(this.props, props);
    return this;
  }

  /**
   * Add a child component
   */
  child(child: ComponentBuilder): this {
    this.children.push(child);
    return this;
  }

  /**
   * Add multiple children
   */
  children(...children: ComponentBuilder[]): this {
    this.children.push(...children);
    return this;
  }

  /**
   * Build the component
   */
  build(): Component {
    // Include children in props
    if (this.children.length > 0) {
      this.props.children = this.children.map(c => c.toJSON());
    }
    
    return new Component(this.type, this.props);
  }

  /**
   * Convert to JSON representation
   */
  toJSON(): any {
    return {
      type: this.type,
      props: this.props,
      children: this.children.map(c => c.toJSON()),
    };
  }
}

// Helper functions for common components
export const div = () => new ComponentBuilder('div');
export const text = (content: string) => new ComponentBuilder('text').prop('content', content);
export const button = (label: string) => new ComponentBuilder('button').prop('label', label);
export const input = (placeholder?: string) => new ComponentBuilder('input').prop('placeholder', placeholder || '');
export const flex = () => new ComponentBuilder('flex');
export const grid = () => new ComponentBuilder('grid');