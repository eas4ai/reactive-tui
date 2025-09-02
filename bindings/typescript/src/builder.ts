/**
 * Builder Pattern API for Reactive TUI
 * 
 * Provides fluent APIs for building UI components matching the Rust builder pattern.
 * This gives TypeScript developers the same ergonomic API as Rust developers.
 */

import { Component } from './component';
import { ElementHandle, ComponentHandle } from './types';
import { FFI } from './ffi';

/**
 * Base builder class that all widget builders extend
 */
abstract class BaseBuilder<T> {
    protected props: Map<string, any> = new Map();
    
    /**
     * Set a CSS class string on the element
     */
    class(className: string): this {
        this.props.set('class', className);
        return this;
    }
    
    /**
     * Set an ID for the element
     */
    id(id: string): this {
        this.props.set('id', id);
        return this;
    }
    
    /**
     * Set a data attribute
     */
    data(key: string, value: any): this {
        const dataAttrs = this.props.get('data') || {};
        dataAttrs[key] = value;
        this.props.set('data', dataAttrs);
        return this;
    }
    
    /**
     * Build the component
     */
    abstract build(): T;
}

/**
 * Button Builder
 * 
 * @example
 * ```typescript
 * const button = new ButtonBuilder()
 *     .text('Click me')
 *     .primary()
 *     .onClick(() => console.log('Clicked!'))
 *     .build();
 * ```
 */
export class ButtonBuilder extends BaseBuilder<Component> {
    private text: string = '';
    private variant: 'primary' | 'secondary' | 'danger' | 'success' = 'secondary';
    private onClick?: () => void;
    private disabled: boolean = false;
    
    /**
     * Set the button text
     */
    label(text: string): this {
        this.text = text;
        return this;
    }
    
    /**
     * Make this a primary button
     */
    primary(): this {
        this.variant = 'primary';
        return this;
    }
    
    /**
     * Make this a secondary button
     */
    secondary(): this {
        this.variant = 'secondary';
        return this;
    }
    
    /**
     * Make this a danger button
     */
    danger(): this {
        this.variant = 'danger';
        return this;
    }
    
    /**
     * Make this a success button
     */
    success(): this {
        this.variant = 'success';
        return this;
    }
    
    /**
     * Set the click handler
     */
    onPress(handler: () => void): this {
        this.onClick = handler;
        return this;
    }
    
    /**
     * Set disabled state
     */
    setDisabled(disabled: boolean): this {
        this.disabled = disabled;
        return this;
    }
    
    build(): Component {
        const handle = FFI.componentCreate('Button');
        const component = new Component(handle);
        
        component.setState({
            text: this.text,
            variant: this.variant,
            disabled: this.disabled,
            ...Object.fromEntries(this.props)
        });
        
        if (this.onClick) {
            component.on('click', this.onClick);
        }
        
        return component;
    }
}

/**
 * List Builder
 * 
 * @example
 * ```typescript
 * const list = new ListBuilder()
 *     .items(['Item 1', 'Item 2', 'Item 3'])
 *     .selectable()
 *     .onSelect((index) => console.log(`Selected: ${index}`))
 *     .build();
 * ```
 */
export class ListBuilder extends BaseBuilder<Component> {
    private items: any[] = [];
    private selectable: boolean = false;
    private multiSelect: boolean = false;
    private onSelect?: (index: number | number[]) => void;
    
    /**
     * Set the list items
     */
    setItems(items: any[]): this {
        this.items = items;
        return this;
    }
    
    /**
     * Add a single item
     */
    addItem(item: any): this {
        this.items.push(item);
        return this;
    }
    
    /**
     * Enable selection
     */
    enableSelection(): this {
        this.selectable = true;
        return this;
    }
    
    /**
     * Enable multi-selection
     */
    enableMultiSelect(): this {
        this.multiSelect = true;
        this.selectable = true;
        return this;
    }
    
    /**
     * Set selection handler
     */
    onItemSelect(handler: (index: number | number[]) => void): this {
        this.onSelect = handler;
        return this;
    }
    
    build(): Component {
        const handle = FFI.componentCreate('List');
        const component = new Component(handle);
        
        component.setState({
            items: this.items,
            selectable: this.selectable,
            multiSelect: this.multiSelect,
            ...Object.fromEntries(this.props)
        });
        
        if (this.onSelect) {
            component.on('select', this.onSelect);
        }
        
        return component;
    }
}

/**
 * Table Builder
 * 
 * @example
 * ```typescript
 * const table = new TableBuilder()
 *     .columns([
 *         { key: 'id', title: 'ID', width: 10 },
 *         { key: 'name', title: 'Name', width: 30 }
 *     ])
 *     .rows(data)
 *     .sortable()
 *     .build();
 * ```
 */
export class TableBuilder extends BaseBuilder<Component> {
    private columns: any[] = [];
    private rows: any[] = [];
    private sortable: boolean = false;
    private filterable: boolean = false;
    private paginated: boolean = false;
    private pageSize: number = 20;
    
    /**
     * Set table columns
     */
    setColumns(columns: any[]): this {
        this.columns = columns;
        return this;
    }
    
    /**
     * Set table rows
     */
    setRows(rows: any[]): this {
        this.rows = rows;
        return this;
    }
    
    /**
     * Enable sorting
     */
    enableSorting(): this {
        this.sortable = true;
        return this;
    }
    
    /**
     * Enable filtering
     */
    enableFiltering(): this {
        this.filterable = true;
        return this;
    }
    
    /**
     * Enable pagination
     */
    enablePagination(pageSize: number = 20): this {
        this.paginated = true;
        this.pageSize = pageSize;
        return this;
    }
    
    build(): Component {
        const handle = FFI.componentCreate('Table');
        const component = new Component(handle);
        
        component.setState({
            columns: this.columns,
            rows: this.rows,
            sortable: this.sortable,
            filterable: this.filterable,
            paginated: this.paginated,
            pageSize: this.pageSize,
            ...Object.fromEntries(this.props)
        });
        
        return component;
    }
}

/**
 * Input Builder
 * 
 * @example
 * ```typescript
 * const input = new InputBuilder()
 *     .placeholder('Enter your name')
 *     .value(currentValue)
 *     .onChange((value) => console.log(value))
 *     .build();
 * ```
 */
export class InputBuilder extends BaseBuilder<Component> {
    private value: string = '';
    private placeholder: string = '';
    private type: string = 'text';
    private disabled: boolean = false;
    private readonly: boolean = false;
    private maxLength?: number;
    private onChange?: (value: string) => void;
    private onSubmit?: (value: string) => void;
    
    /**
     * Set the input value
     */
    setValue(value: string): this {
        this.value = value;
        return this;
    }
    
    /**
     * Set placeholder text
     */
    setPlaceholder(placeholder: string): this {
        this.placeholder = placeholder;
        return this;
    }
    
    /**
     * Set input type
     */
    setType(type: 'text' | 'password' | 'email' | 'number'): this {
        this.type = type;
        return this;
    }
    
    /**
     * Set disabled state
     */
    setDisabled(disabled: boolean): this {
        this.disabled = disabled;
        return this;
    }
    
    /**
     * Set readonly state
     */
    setReadonly(readonly: boolean): this {
        this.readonly = readonly;
        return this;
    }
    
    /**
     * Set max length
     */
    setMaxLength(length: number): this {
        this.maxLength = length;
        return this;
    }
    
    /**
     * Set change handler
     */
    onValueChange(handler: (value: string) => void): this {
        this.onChange = handler;
        return this;
    }
    
    /**
     * Set submit handler
     */
    onValueSubmit(handler: (value: string) => void): this {
        this.onSubmit = handler;
        return this;
    }
    
    build(): Component {
        const handle = FFI.componentCreate('Input');
        const component = new Component(handle);
        
        component.setState({
            value: this.value,
            placeholder: this.placeholder,
            type: this.type,
            disabled: this.disabled,
            readonly: this.readonly,
            maxLength: this.maxLength,
            ...Object.fromEntries(this.props)
        });
        
        if (this.onChange) {
            component.on('change', this.onChange);
        }
        
        if (this.onSubmit) {
            component.on('submit', this.onSubmit);
        }
        
        return component;
    }
}

/**
 * Container Builder for layout components
 * 
 * @example
 * ```typescript
 * const container = new ContainerBuilder()
 *     .direction('row')
 *     .gap(2)
 *     .padding(4)
 *     .children([child1, child2])
 *     .build();
 * ```
 */
export class ContainerBuilder extends BaseBuilder<Component> {
    private direction: 'row' | 'column' = 'column';
    private gap: number = 0;
    private padding: number = 0;
    private margin: number = 0;
    private children: Component[] = [];
    private align?: 'start' | 'center' | 'end' | 'stretch';
    private justify?: 'start' | 'center' | 'end' | 'between' | 'around';
    
    /**
     * Set flex direction
     */
    setDirection(direction: 'row' | 'column'): this {
        this.direction = direction;
        return this;
    }
    
    /**
     * Set gap between children
     */
    setGap(gap: number): this {
        this.gap = gap;
        return this;
    }
    
    /**
     * Set padding
     */
    setPadding(padding: number): this {
        this.padding = padding;
        return this;
    }
    
    /**
     * Set margin
     */
    setMargin(margin: number): this {
        this.margin = margin;
        return this;
    }
    
    /**
     * Set alignment
     */
    setAlign(align: 'start' | 'center' | 'end' | 'stretch'): this {
        this.align = align;
        return this;
    }
    
    /**
     * Set justification
     */
    setJustify(justify: 'start' | 'center' | 'end' | 'between' | 'around'): this {
        this.justify = justify;
        return this;
    }
    
    /**
     * Add children
     */
    addChildren(children: Component[]): this {
        this.children.push(...children);
        return this;
    }
    
    /**
     * Add a single child
     */
    addChild(child: Component): this {
        this.children.push(child);
        return this;
    }
    
    build(): Component {
        const handle = FFI.componentCreate('Container');
        const component = new Component(handle);
        
        const cssClasses = [];
        
        // Build CSS classes based on properties
        if (this.direction === 'row') {
            cssClasses.push('flex-row');
        } else {
            cssClasses.push('flex-col');
        }
        
        if (this.gap > 0) {
            cssClasses.push(`gap-${this.gap}`);
        }
        
        if (this.padding > 0) {
            cssClasses.push(`p-${this.padding}`);
        }
        
        if (this.margin > 0) {
            cssClasses.push(`m-${this.margin}`);
        }
        
        if (this.align) {
            cssClasses.push(`items-${this.align}`);
        }
        
        if (this.justify) {
            cssClasses.push(`justify-${this.justify}`);
        }
        
        component.setState({
            class: cssClasses.join(' '),
            children: this.children.map(c => c.handle),
            ...Object.fromEntries(this.props)
        });
        
        return component;
    }
}

/**
 * Text Builder
 * 
 * @example
 * ```typescript
 * const text = new TextBuilder()
 *     .content('Hello, World!')
 *     .bold()
 *     .color('blue')
 *     .build();
 * ```
 */
export class TextBuilder extends BaseBuilder<Component> {
    private content: string = '';
    private bold: boolean = false;
    private italic: boolean = false;
    private underline: boolean = false;
    private color?: string;
    private bgColor?: string;
    private size?: 'sm' | 'base' | 'lg' | 'xl';
    
    /**
     * Set text content
     */
    setContent(content: string): this {
        this.content = content;
        return this;
    }
    
    /**
     * Make text bold
     */
    setBold(): this {
        this.bold = true;
        return this;
    }
    
    /**
     * Make text italic
     */
    setItalic(): this {
        this.italic = true;
        return this;
    }
    
    /**
     * Make text underlined
     */
    setUnderline(): this {
        this.underline = true;
        return this;
    }
    
    /**
     * Set text color
     */
    setColor(color: string): this {
        this.color = color;
        return this;
    }
    
    /**
     * Set background color
     */
    setBgColor(color: string): this {
        this.bgColor = color;
        return this;
    }
    
    /**
     * Set text size
     */
    setSize(size: 'sm' | 'base' | 'lg' | 'xl'): this {
        this.size = size;
        return this;
    }
    
    build(): Component {
        const handle = FFI.componentCreate('Text');
        const component = new Component(handle);
        
        const styles = [];
        
        if (this.bold) styles.push('font-bold');
        if (this.italic) styles.push('italic');
        if (this.underline) styles.push('underline');
        if (this.color) styles.push(`text-${this.color}`);
        if (this.bgColor) styles.push(`bg-${this.bgColor}`);
        if (this.size) styles.push(`text-${this.size}`);
        
        component.setState({
            content: this.content,
            class: styles.join(' '),
            ...Object.fromEntries(this.props)
        });
        
        return component;
    }
}

/**
 * Factory functions for creating builders
 */
export const builders = {
    button: () => new ButtonBuilder(),
    list: () => new ListBuilder(),
    table: () => new TableBuilder(),
    input: () => new InputBuilder(),
    container: () => new ContainerBuilder(),
    text: () => new TextBuilder(),
    
    // Shortcuts for common patterns
    primaryButton: (text: string) => new ButtonBuilder().label(text).primary(),
    dangerButton: (text: string) => new ButtonBuilder().label(text).danger(),
    row: () => new ContainerBuilder().setDirection('row'),
    column: () => new ContainerBuilder().setDirection('column'),
};