/**
 * Layout System and CSS Utilities for Reactive TUI
 * 
 * Provides Tailwind-style utility classes and layout helpers
 * for building responsive terminal UIs.
 */

import { Theme } from './theme';

/**
 * Display types
 */
export type Display = 'none' | 'block' | 'flex' | 'grid';

/**
 * Position types
 */
export type Position = 'static' | 'relative' | 'absolute' | 'fixed' | 'sticky';

/**
 * Flexbox direction
 */
export type FlexDirection = 'row' | 'row-reverse' | 'column' | 'column-reverse';

/**
 * Flexbox wrap
 */
export type FlexWrap = 'nowrap' | 'wrap' | 'wrap-reverse';

/**
 * Alignment options
 */
export type AlignItems = 'start' | 'end' | 'center' | 'stretch' | 'baseline';
export type JustifyContent = 'start' | 'end' | 'center' | 'between' | 'around' | 'evenly';
export type AlignContent = 'start' | 'end' | 'center' | 'between' | 'around' | 'stretch';

/**
 * Box model properties
 */
export interface BoxModel {
    margin?: Spacing | number;
    padding?: Spacing | number;
    border?: Border;
}

/**
 * Spacing values (can be different for each side)
 */
export interface Spacing {
    top?: number;
    right?: number;
    bottom?: number;
    left?: number;
}

/**
 * Border definition
 */
export interface Border {
    width?: number | Spacing;
    style?: 'solid' | 'dashed' | 'dotted' | 'double' | 'none';
    color?: string;
    radius?: number | BorderRadius;
}

/**
 * Border radius for corners
 */
export interface BorderRadius {
    topLeft?: number;
    topRight?: number;
    bottomRight?: number;
    bottomLeft?: number;
}

/**
 * Size constraints
 */
export interface SizeConstraints {
    width?: number | string;
    height?: number | string;
    minWidth?: number;
    minHeight?: number;
    maxWidth?: number;
    maxHeight?: number;
}

/**
 * Grid layout properties
 */
export interface GridLayout {
    columns?: number | string[];
    rows?: number | string[];
    gap?: number | { row?: number; column?: number };
    autoFlow?: 'row' | 'column' | 'dense';
    autoColumns?: string;
    autoRows?: string;
}

/**
 * Flexbox layout properties
 */
export interface FlexLayout {
    direction?: FlexDirection;
    wrap?: FlexWrap;
    alignItems?: AlignItems;
    justifyContent?: JustifyContent;
    alignContent?: AlignContent;
    gap?: number;
    grow?: number;
    shrink?: number;
    basis?: number | string;
}

/**
 * Complete layout style definition
 */
export interface LayoutStyle {
    display?: Display;
    position?: Position;
    box?: BoxModel;
    size?: SizeConstraints;
    flex?: FlexLayout;
    grid?: GridLayout;
    zIndex?: number;
    overflow?: 'visible' | 'hidden' | 'scroll' | 'auto';
    opacity?: number;
}

/**
 * CSS utility class generator
 */
export class CSSUtilities {
    private theme: Theme;
    
    constructor(theme: Theme) {
        this.theme = theme;
    }
    
    /**
     * Generate display utilities
     */
    display(type: Display): string {
        const map: Record<Display, string> = {
            none: 'hidden',
            block: 'block',
            flex: 'flex',
            grid: 'grid',
        };
        return map[type];
    }
    
    /**
     * Generate position utilities
     */
    position(type: Position): string {
        return type === 'static' ? '' : type;
    }
    
    /**
     * Generate flexbox utilities
     */
    flex(layout: FlexLayout): string {
        const classes: string[] = ['flex'];
        
        if (layout.direction) {
            const dirMap: Record<FlexDirection, string> = {
                'row': 'flex-row',
                'row-reverse': 'flex-row-reverse',
                'column': 'flex-col',
                'column-reverse': 'flex-col-reverse',
            };
            classes.push(dirMap[layout.direction]);
        }
        
        if (layout.wrap) {
            const wrapMap: Record<FlexWrap, string> = {
                'nowrap': 'flex-nowrap',
                'wrap': 'flex-wrap',
                'wrap-reverse': 'flex-wrap-reverse',
            };
            classes.push(wrapMap[layout.wrap]);
        }
        
        if (layout.alignItems) {
            classes.push(`items-${layout.alignItems}`);
        }
        
        if (layout.justifyContent) {
            classes.push(`justify-${layout.justifyContent}`);
        }
        
        if (layout.gap !== undefined) {
            classes.push(`gap-${layout.gap}`);
        }
        
        if (layout.grow !== undefined) {
            classes.push(layout.grow === 0 ? 'flex-grow-0' : 'flex-grow');
        }
        
        if (layout.shrink !== undefined) {
            classes.push(layout.shrink === 0 ? 'flex-shrink-0' : 'flex-shrink');
        }
        
        return classes.join(' ');
    }
    
    /**
     * Generate grid utilities
     */
    grid(layout: GridLayout): string {
        const classes: string[] = ['grid'];
        
        if (layout.columns) {
            if (typeof layout.columns === 'number') {
                classes.push(`grid-cols-${layout.columns}`);
            } else {
                // Custom grid template
                classes.push('grid-cols-custom');
            }
        }
        
        if (layout.rows) {
            if (typeof layout.rows === 'number') {
                classes.push(`grid-rows-${layout.rows}`);
            } else {
                classes.push('grid-rows-custom');
            }
        }
        
        if (layout.gap !== undefined) {
            if (typeof layout.gap === 'number') {
                classes.push(`gap-${layout.gap}`);
            } else {
                if (layout.gap.row !== undefined) {
                    classes.push(`gap-y-${layout.gap.row}`);
                }
                if (layout.gap.column !== undefined) {
                    classes.push(`gap-x-${layout.gap.column}`);
                }
            }
        }
        
        if (layout.autoFlow) {
            classes.push(`grid-flow-${layout.autoFlow}`);
        }
        
        return classes.join(' ');
    }
    
    /**
     * Generate spacing utilities (margin/padding)
     */
    spacing(type: 'margin' | 'padding', value: Spacing | number): string {
        const prefix = type === 'margin' ? 'm' : 'p';
        
        if (typeof value === 'number') {
            return `${prefix}-${value}`;
        }
        
        const classes: string[] = [];
        
        if (value.top !== undefined) {
            classes.push(`${prefix}t-${value.top}`);
        }
        if (value.right !== undefined) {
            classes.push(`${prefix}r-${value.right}`);
        }
        if (value.bottom !== undefined) {
            classes.push(`${prefix}b-${value.bottom}`);
        }
        if (value.left !== undefined) {
            classes.push(`${prefix}l-${value.left}`);
        }
        
        return classes.join(' ');
    }
    
    /**
     * Generate size utilities
     */
    size(constraints: SizeConstraints): string {
        const classes: string[] = [];
        
        if (constraints.width !== undefined) {
            if (typeof constraints.width === 'number') {
                classes.push(`w-${constraints.width}`);
            } else {
                classes.push(`w-[${constraints.width}]`);
            }
        }
        
        if (constraints.height !== undefined) {
            if (typeof constraints.height === 'number') {
                classes.push(`h-${constraints.height}`);
            } else {
                classes.push(`h-[${constraints.height}]`);
            }
        }
        
        if (constraints.minWidth !== undefined) {
            classes.push(`min-w-${constraints.minWidth}`);
        }
        
        if (constraints.minHeight !== undefined) {
            classes.push(`min-h-${constraints.minHeight}`);
        }
        
        if (constraints.maxWidth !== undefined) {
            classes.push(`max-w-${constraints.maxWidth}`);
        }
        
        if (constraints.maxHeight !== undefined) {
            classes.push(`max-h-${constraints.maxHeight}`);
        }
        
        return classes.join(' ');
    }
    
    /**
     * Generate border utilities
     */
    border(border: Border): string {
        const classes: string[] = [];
        
        if (border.width !== undefined) {
            if (typeof border.width === 'number') {
                classes.push(border.width === 0 ? 'border-0' : `border-${border.width}`);
            } else {
                // Different widths for each side
                if (border.width.top !== undefined) {
                    classes.push(`border-t-${border.width.top}`);
                }
                if (border.width.right !== undefined) {
                    classes.push(`border-r-${border.width.right}`);
                }
                if (border.width.bottom !== undefined) {
                    classes.push(`border-b-${border.width.bottom}`);
                }
                if (border.width.left !== undefined) {
                    classes.push(`border-l-${border.width.left}`);
                }
            }
        } else {
            classes.push('border');
        }
        
        if (border.style && border.style !== 'solid') {
            classes.push(`border-${border.style}`);
        }
        
        if (border.color) {
            classes.push(`border-${border.color}`);
        }
        
        if (border.radius !== undefined) {
            if (typeof border.radius === 'number') {
                classes.push(`rounded-${border.radius}`);
            } else {
                // Different radius for each corner
                classes.push('rounded-custom');
            }
        }
        
        return classes.join(' ');
    }
    
    /**
     * Generate complete layout style as CSS classes
     */
    generateClasses(style: LayoutStyle): string {
        const classes: string[] = [];
        
        if (style.display) {
            classes.push(this.display(style.display));
        }
        
        if (style.position) {
            classes.push(this.position(style.position));
        }
        
        if (style.box) {
            if (style.box.margin) {
                classes.push(this.spacing('margin', style.box.margin));
            }
            if (style.box.padding) {
                classes.push(this.spacing('padding', style.box.padding));
            }
            if (style.box.border) {
                classes.push(this.border(style.box.border));
            }
        }
        
        if (style.size) {
            classes.push(this.size(style.size));
        }
        
        if (style.flex) {
            classes.push(this.flex(style.flex));
        }
        
        if (style.grid) {
            classes.push(this.grid(style.grid));
        }
        
        if (style.zIndex !== undefined) {
            classes.push(`z-${style.zIndex}`);
        }
        
        if (style.overflow) {
            classes.push(`overflow-${style.overflow}`);
        }
        
        if (style.opacity !== undefined) {
            classes.push(`opacity-${Math.round(style.opacity * 100)}`);
        }
        
        return classes.join(' ');
    }
}

/**
 * Layout builder for creating complex layouts
 */
export class LayoutBuilder {
    private style: LayoutStyle = {};
    
    /**
     * Set display type
     */
    display(type: Display): this {
        this.style.display = type;
        return this;
    }
    
    /**
     * Set position type
     */
    position(type: Position): this {
        this.style.position = type;
        return this;
    }
    
    /**
     * Set margin
     */
    margin(value: Spacing | number): this {
        if (!this.style.box) this.style.box = {};
        this.style.box.margin = value;
        return this;
    }
    
    /**
     * Set padding
     */
    padding(value: Spacing | number): this {
        if (!this.style.box) this.style.box = {};
        this.style.box.padding = value;
        return this;
    }
    
    /**
     * Set border
     */
    border(value: Border): this {
        if (!this.style.box) this.style.box = {};
        this.style.box.border = value;
        return this;
    }
    
    /**
     * Set size constraints
     */
    size(constraints: SizeConstraints): this {
        this.style.size = constraints;
        return this;
    }
    
    /**
     * Configure as flexbox
     */
    flex(layout?: FlexLayout): this {
        this.style.display = 'flex';
        this.style.flex = layout || {};
        return this;
    }
    
    /**
     * Configure as grid
     */
    grid(layout?: GridLayout): this {
        this.style.display = 'grid';
        this.style.grid = layout || {};
        return this;
    }
    
    /**
     * Set z-index
     */
    zIndex(value: number): this {
        this.style.zIndex = value;
        return this;
    }
    
    /**
     * Set overflow
     */
    overflow(value: 'visible' | 'hidden' | 'scroll' | 'auto'): this {
        this.style.overflow = value;
        return this;
    }
    
    /**
     * Set opacity
     */
    opacity(value: number): this {
        this.style.opacity = value;
        return this;
    }
    
    /**
     * Build the layout style
     */
    build(): LayoutStyle {
        return { ...this.style };
    }
    
    /**
     * Build as CSS classes
     */
    toClasses(theme: Theme): string {
        const utils = new CSSUtilities(theme);
        return utils.generateClasses(this.style);
    }
}

/**
 * Responsive breakpoints
 */
export const breakpoints = {
    sm: 640,
    md: 768,
    lg: 1024,
    xl: 1280,
    '2xl': 1536,
};

/**
 * Media query helper
 */
export function mediaQuery(breakpoint: keyof typeof breakpoints): string {
    return `@media (min-width: ${breakpoints[breakpoint]}px)`;
}

/**
 * Common layout presets
 */
export const layouts = {
    /**
     * Center content both horizontally and vertically
     */
    center: (): LayoutStyle => ({
        display: 'flex',
        flex: {
            alignItems: 'center',
            justifyContent: 'center',
        },
    }),
    
    /**
     * Stack items vertically
     */
    stack: (gap?: number): LayoutStyle => ({
        display: 'flex',
        flex: {
            direction: 'column',
            gap,
        },
    }),
    
    /**
     * Arrange items horizontally
     */
    row: (gap?: number): LayoutStyle => ({
        display: 'flex',
        flex: {
            direction: 'row',
            alignItems: 'center',
            gap,
        },
    }),
    
    /**
     * Create a grid layout
     */
    gridLayout: (columns: number, gap?: number): LayoutStyle => ({
        display: 'grid',
        grid: {
            columns,
            gap,
        },
    }),
    
    /**
     * Absolute positioning that fills parent
     */
    absoluteFill: (): LayoutStyle => ({
        position: 'absolute',
        size: {
            width: '100%',
            height: '100%',
        },
    }),
    
    /**
     * Container with max width and centered
     */
    container: (maxWidth?: number): LayoutStyle => ({
        size: {
            width: '100%',
            maxWidth: maxWidth || 1280,
        },
        box: {
            margin: { left: 0, right: 0 }, // auto margins for centering
        },
    }),
};
