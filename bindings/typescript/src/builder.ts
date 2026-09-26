/** Static builders backed by the implemented native Element API. */
import { Component, ComponentBuilder } from './component';

export class ButtonBuilder extends ComponentBuilder {
  constructor() { super('button'); }
  label(value: string): this { return this.text(value); }
  primary(): this { return this.class('bg-blue-500 text-white'); }
  secondary(): this { return this.class('bg-gray-500 text-white'); }
  danger(): this { return this.class('bg-red-500 text-white'); }
  success(): this { return this.class('bg-green-500 text-white'); }
}

export class ContainerBuilder extends ComponentBuilder {
  constructor() { super('div'); }
  setDirection(value: 'row' | 'column'): this { return this.class(value === 'row' ? 'flex-row' : 'flex-col'); }
  private spacing(prefix: string, value: number): this {
    if (!Number.isSafeInteger(value) || value < 0) throw new RangeError('Spacing must be a nonnegative integer');
    return this.class(`${prefix}-${value}`);
  }
  setGap(value: number): this { return this.spacing('gap', value); }
  setPadding(value: number): this { return this.spacing('p', value); }
  setMargin(value: number): this { return this.spacing('m', value); }
  setAlign(value: 'start' | 'center' | 'end' | 'stretch'): this { return this.class(`items-${value}`); }
  setJustify(value: 'start' | 'center' | 'end' | 'between' | 'around'): this { return this.class(`justify-${value}`); }
  addChild(child: Component): this { return this.child(child); }
  addChildren(children: Component[]): this { return this.children(...children); }
}

export class TextBuilder extends ComponentBuilder {
  constructor() { super('span'); }
  setContent(value: string): this { return this.text(value); }
  setBold(): this { return this.class('font-bold'); }
  setItalic(): this { return this.class('italic'); }
  setUnderline(): this { return this.class('underline'); }
  setColor(value: string): this { return this.class(`text-${value}`); }
  setBgColor(value: string): this { return this.class(`bg-${value}`); }
  setSize(value: 'sm' | 'base' | 'lg' | 'xl'): this { return this.class(`text-${value}`); }
}

export const builders = {
  button: () => new ButtonBuilder(), container: () => new ContainerBuilder(), text: () => new TextBuilder(),
  primaryButton: (text: string) => new ButtonBuilder().label(text).primary(),
  dangerButton: (text: string) => new ButtonBuilder().label(text).danger(),
  row: () => new ContainerBuilder().setDirection('row'), column: () => new ContainerBuilder().setDirection('column'),
};
