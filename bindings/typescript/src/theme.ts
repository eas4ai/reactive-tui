/**
 * Theme System for Reactive TUI
 * 
 * Provides theming capabilities including color schemes, typography,
 * spacing, and custom properties support.
 */

import { EventEmitter } from './events';

/**
 * Color definition with RGB values
 */
export interface Color {
    r: number;
    g: number;
    b: number;
    a?: number;
}

/**
 * Color palette for a theme
 */
export interface ColorPalette {
    // Primary colors
    primary: Color;
    primaryLight: Color;
    primaryDark: Color;
    
    // Secondary colors
    secondary: Color;
    secondaryLight: Color;
    secondaryDark: Color;
    
    // Semantic colors
    success: Color;
    warning: Color;
    error: Color;
    info: Color;
    
    // Neutral colors
    background: Color;
    surface: Color;
    text: Color;
    textSecondary: Color;
    border: Color;
    
    // State colors
    hover: Color;
    active: Color;
    disabled: Color;
    focus: Color;
}

/**
 * Typography settings
 */
export interface Typography {
    // Font families
    fontFamily: string;
    monoFamily: string;
    
    // Font sizes
    sizes: {
        xs: number;
        sm: number;
        base: number;
        lg: number;
        xl: number;
        '2xl': number;
        '3xl': number;
    };
    
    // Font weights
    weights: {
        thin: number;
        light: number;
        normal: number;
        medium: number;
        semibold: number;
        bold: number;
        extrabold: number;
    };
    
    // Line heights
    lineHeights: {
        none: number;
        tight: number;
        snug: number;
        normal: number;
        relaxed: number;
        loose: number;
    };
}

/**
 * Spacing scale
 */
export interface Spacing {
    0: number;
    px: number;
    0.5: number;
    1: number;
    1.5: number;
    2: number;
    2.5: number;
    3: number;
    3.5: number;
    4: number;
    5: number;
    6: number;
    7: number;
    8: number;
    9: number;
    10: number;
    12: number;
    14: number;
    16: number;
    20: number;
    24: number;
    28: number;
    32: number;
    36: number;
    40: number;
    44: number;
    48: number;
    52: number;
    56: number;
    60: number;
    64: number;
    72: number;
    80: number;
    96: number;
}

/**
 * Border radius values
 */
export interface BorderRadius {
    none: number;
    sm: number;
    base: number;
    md: number;
    lg: number;
    xl: number;
    '2xl': number;
    '3xl': number;
    full: number;
}

/**
 * Shadow definitions
 */
export interface Shadows {
    none: string;
    sm: string;
    base: string;
    md: string;
    lg: string;
    xl: string;
    '2xl': string;
    inner: string;
}

/**
 * Animation durations
 */
export interface Animations {
    durations: {
        instant: number;
        fast: number;
        normal: number;
        slow: number;
    };
    
    easings: {
        linear: string;
        easeIn: string;
        easeOut: string;
        easeInOut: string;
        bounce: string;
    };
}

/**
 * Complete theme definition
 */
export interface Theme {
    name: string;
    mode: 'light' | 'dark';
    colors: ColorPalette;
    typography: Typography;
    spacing: Spacing;
    borderRadius: BorderRadius;
    shadows: Shadows;
    animations: Animations;
    customProperties?: Record<string, any>;
}

/**
 * Default light theme
 */
export const lightTheme: Theme = {
    name: 'Light',
    mode: 'light',
    colors: {
        primary: { r: 59, g: 130, b: 246 },
        primaryLight: { r: 96, g: 165, b: 250 },
        primaryDark: { r: 37, g: 99, b: 235 },
        
        secondary: { r: 139, g: 92, b: 246 },
        secondaryLight: { r: 167, g: 139, b: 250 },
        secondaryDark: { r: 124, g: 58, b: 237 },
        
        success: { r: 34, g: 197, b: 94 },
        warning: { r: 251, g: 191, b: 36 },
        error: { r: 239, g: 68, b: 68 },
        info: { r: 59, g: 130, b: 246 },
        
        background: { r: 255, g: 255, b: 255 },
        surface: { r: 249, g: 250, b: 251 },
        text: { r: 17, g: 24, b: 39 },
        textSecondary: { r: 107, g: 114, b: 128 },
        border: { r: 229, g: 231, b: 235 },
        
        hover: { r: 243, g: 244, b: 246 },
        active: { r: 229, g: 231, b: 235 },
        disabled: { r: 156, g: 163, b: 175 },
        focus: { r: 59, g: 130, b: 246, a: 0.5 },
    },
    typography: {
        fontFamily: 'system-ui, -apple-system, sans-serif',
        monoFamily: 'Menlo, Monaco, Consolas, monospace',
        sizes: {
            xs: 12,
            sm: 14,
            base: 16,
            lg: 18,
            xl: 20,
            '2xl': 24,
            '3xl': 30,
        },
        weights: {
            thin: 100,
            light: 300,
            normal: 400,
            medium: 500,
            semibold: 600,
            bold: 700,
            extrabold: 800,
        },
        lineHeights: {
            none: 1,
            tight: 1.25,
            snug: 1.375,
            normal: 1.5,
            relaxed: 1.625,
            loose: 2,
        },
    },
    spacing: {
        0: 0,
        px: 1,
        0.5: 2,
        1: 4,
        1.5: 6,
        2: 8,
        2.5: 10,
        3: 12,
        3.5: 14,
        4: 16,
        5: 20,
        6: 24,
        7: 28,
        8: 32,
        9: 36,
        10: 40,
        12: 48,
        14: 56,
        16: 64,
        20: 80,
        24: 96,
        28: 112,
        32: 128,
        36: 144,
        40: 160,
        44: 176,
        48: 192,
        52: 208,
        56: 224,
        60: 240,
        64: 256,
        72: 288,
        80: 320,
        96: 384,
    },
    borderRadius: {
        none: 0,
        sm: 2,
        base: 4,
        md: 6,
        lg: 8,
        xl: 12,
        '2xl': 16,
        '3xl': 24,
        full: 9999,
    },
    shadows: {
        none: 'none',
        sm: '0 1px 2px 0 rgba(0, 0, 0, 0.05)',
        base: '0 1px 3px 0 rgba(0, 0, 0, 0.1), 0 1px 2px 0 rgba(0, 0, 0, 0.06)',
        md: '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)',
        lg: '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)',
        xl: '0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04)',
        '2xl': '0 25px 50px -12px rgba(0, 0, 0, 0.25)',
        inner: 'inset 0 2px 4px 0 rgba(0, 0, 0, 0.06)',
    },
    animations: {
        durations: {
            instant: 0,
            fast: 150,
            normal: 300,
            slow: 500,
        },
        easings: {
            linear: 'linear',
            easeIn: 'cubic-bezier(0.4, 0, 1, 1)',
            easeOut: 'cubic-bezier(0, 0, 0.2, 1)',
            easeInOut: 'cubic-bezier(0.4, 0, 0.2, 1)',
            bounce: 'cubic-bezier(0.68, -0.55, 0.265, 1.55)',
        },
    },
};

/**
 * Default dark theme
 */
export const darkTheme: Theme = {
    ...lightTheme,
    name: 'Dark',
    mode: 'dark',
    colors: {
        ...lightTheme.colors,
        background: { r: 17, g: 24, b: 39 },
        surface: { r: 31, g: 41, b: 55 },
        text: { r: 243, g: 244, b: 246 },
        textSecondary: { r: 156, g: 163, b: 175 },
        border: { r: 55, g: 65, b: 81 },
        
        hover: { r: 55, g: 65, b: 81 },
        active: { r: 75, g: 85, b: 99 },
        disabled: { r: 107, g: 114, b: 128 },
    },
};

/**
 * Theme manager for handling theme switching and custom properties
 */
export class ThemeManager extends EventEmitter {
    private currentTheme: Theme;
    private themes: Map<string, Theme> = new Map();
    private cssVariables: Map<string, string> = new Map();
    
    constructor(initialTheme: Theme = lightTheme) {
        super();
        this.currentTheme = initialTheme;
        this.registerTheme(lightTheme);
        this.registerTheme(darkTheme);
        this.applyTheme(initialTheme);
    }
    
    /**
     * Register a new theme
     */
    registerTheme(theme: Theme): void {
        this.themes.set(theme.name, theme);
    }
    
    /**
     * Get a registered theme by name
     */
    getTheme(name: string): Theme | undefined {
        return this.themes.get(name);
    }
    
    /**
     * Get all registered themes
     */
    getAllThemes(): Theme[] {
        return Array.from(this.themes.values());
    }
    
    /**
     * Get the current theme
     */
    getCurrentTheme(): Theme {
        return this.currentTheme;
    }
    
    /**
     * Switch to a different theme
     */
    switchTheme(nameOrTheme: string | Theme): void {
        const theme = typeof nameOrTheme === 'string' 
            ? this.themes.get(nameOrTheme)
            : nameOrTheme;
        
        if (!theme) {
            throw new Error(`Theme not found: ${nameOrTheme}`);
        }
        
        const previousTheme = this.currentTheme;
        this.currentTheme = theme;
        this.applyTheme(theme);
        
        this.emit('themeChanged', {
            previous: previousTheme,
            current: theme,
        });
    }
    
    /**
     * Toggle between light and dark themes
     */
    toggleMode(): void {
        const newMode = this.currentTheme.mode === 'light' ? 'dark' : 'light';
        const newTheme = newMode === 'light' ? lightTheme : darkTheme;
        this.switchTheme(newTheme);
    }
    
    /**
     * Apply a theme by generating CSS variables
     */
    private applyTheme(theme: Theme): void {
        this.cssVariables.clear();
        
        // Generate color variables
        Object.entries(theme.colors).forEach(([key, color]) => {
            this.cssVariables.set(
                `--color-${key}`,
                `rgb(${color.r}, ${color.g}, ${color.b})`
            );
            if (color.a !== undefined) {
                this.cssVariables.set(
                    `--color-${key}-alpha`,
                    `rgba(${color.r}, ${color.g}, ${color.b}, ${color.a})`
                );
            }
        });
        
        // Generate typography variables
        Object.entries(theme.typography.sizes).forEach(([key, value]) => {
            this.cssVariables.set(`--text-${key}`, `${value}px`);
        });
        
        Object.entries(theme.typography.weights).forEach(([key, value]) => {
            this.cssVariables.set(`--font-${key}`, value.toString());
        });
        
        // Generate spacing variables
        Object.entries(theme.spacing).forEach(([key, value]) => {
            const varKey = key.replace('.', '-');
            this.cssVariables.set(`--spacing-${varKey}`, `${value}px`);
        });
        
        // Generate border radius variables
        Object.entries(theme.borderRadius).forEach(([key, value]) => {
            this.cssVariables.set(`--radius-${key}`, `${value}px`);
        });
        
        // Generate shadow variables
        Object.entries(theme.shadows).forEach(([key, value]) => {
            this.cssVariables.set(`--shadow-${key}`, value);
        });
        
        // Add custom properties
        if (theme.customProperties) {
            Object.entries(theme.customProperties).forEach(([key, value]) => {
                this.cssVariables.set(`--${key}`, value.toString());
            });
        }
    }
    
    /**
     * Get a CSS variable value
     */
    getCSSVariable(name: string): string | undefined {
        return this.cssVariables.get(name);
    }
    
    /**
     * Get all CSS variables as an object
     */
    getAllCSSVariables(): Record<string, string> {
        return Object.fromEntries(this.cssVariables);
    }
    
    /**
     * Set a custom property on the current theme
     */
    setCustomProperty(key: string, value: any): void {
        if (!this.currentTheme.customProperties) {
            this.currentTheme.customProperties = {};
        }
        this.currentTheme.customProperties[key] = value;
        this.cssVariables.set(`--${key}`, value.toString());
        
        this.emit('customPropertyChanged', { key, value });
    }
    
    /**
     * Create a new theme based on an existing one
     */
    createTheme(name: string, base: Theme, overrides: Partial<Theme>): Theme {
        const newTheme: Theme = {
            ...base,
            ...overrides,
            name,
            colors: { ...base.colors, ...(overrides.colors || {}) },
            typography: { ...base.typography, ...(overrides.typography || {}) },
            spacing: { ...base.spacing, ...(overrides.spacing || {}) },
            borderRadius: { ...base.borderRadius, ...(overrides.borderRadius || {}) },
            shadows: { ...base.shadows, ...(overrides.shadows || {}) },
            animations: { ...base.animations, ...(overrides.animations || {}) },
        };
        
        this.registerTheme(newTheme);
        return newTheme;
    }
}

/**
 * Global theme manager instance
 */
export const themeManager = new ThemeManager();

/**
 * Utility function to get color value from theme
 */
export function getColor(theme: Theme, colorName: keyof ColorPalette): string {
    const color = theme.colors[colorName];
    return `rgb(${color.r}, ${color.g}, ${color.b})`;
}

/**
 * Utility function to get spacing value from theme
 */
export function getSpacing(theme: Theme, size: keyof Spacing): number {
    return theme.spacing[size];
}

/**
 * Create a color from RGB values
 */
export function rgb(r: number, g: number, b: number, a?: number): Color {
    return { r, g, b, a };
}

/**
 * Create a color from hex string
 */
export function hex(hexString: string): Color {
    const hex = hexString.replace('#', '');
    const r = parseInt(hex.substring(0, 2), 16);
    const g = parseInt(hex.substring(2, 4), 16);
    const b = parseInt(hex.substring(4, 6), 16);
    const a = hex.length === 8 ? parseInt(hex.substring(6, 8), 16) / 255 : undefined;
    return { r, g, b, a };
}