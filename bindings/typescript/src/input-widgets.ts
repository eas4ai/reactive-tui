/**
 * Input Widgets for Reactive TUI
 * 
 * Complete set of input components including radio buttons, sliders,
 * date/time pickers, color pickers, and file selectors.
 */

import { Component } from './component';
import { EventEmitter } from './events';
import { FFI } from './ffi';
import { ComponentHandle } from './types';

/**
 * Base class for all input widgets
 */
abstract class InputWidget<T = any> extends Component {
    protected value: T;
    protected disabled: boolean = false;
    protected readonly: boolean = false;
    
    constructor(handle: ComponentHandle, initialValue: T) {
        super(handle);
        this.value = initialValue;
    }
    
    /**
     * Get the current value
     */
    getValue(): T {
        return this.value;
    }
    
    /**
     * Set the value
     */
    setValue(value: T): void {
        const oldValue = this.value;
        this.value = value;
        this.setState({ value });
        this.emit('change', { oldValue, newValue: value });
    }
    
    /**
     * Enable/disable the input
     */
    setDisabled(disabled: boolean): void {
        this.disabled = disabled;
        this.setState({ disabled });
    }
    
    /**
     * Set readonly state
     */
    setReadonly(readonly: boolean): void {
        this.readonly = readonly;
        this.setState({ readonly });
    }
    
    /**
     * Check if input is valid
     */
    abstract validate(): boolean;
}

/**
 * Radio button group configuration
 */
export interface RadioOption<T = string> {
    value: T;
    label: string;
    disabled?: boolean;
}

/**
 * Radio Button Group Widget
 * 
 * @example
 * ```typescript
 * const radio = new RadioButtonGroup({
 *     options: [
 *         { value: 'small', label: 'Small' },
 *         { value: 'medium', label: 'Medium' },
 *         { value: 'large', label: 'Large' }
 *     ],
 *     value: 'medium',
 *     orientation: 'vertical'
 * });
 * 
 * radio.on('change', ({ newValue }) => {
 *     console.log('Selected:', newValue);
 * });
 * ```
 */
export class RadioButtonGroup<T = string> extends InputWidget<T> {
    private options: RadioOption<T>[];
    private orientation: 'horizontal' | 'vertical';
    
    constructor(config: {
        options: RadioOption<T>[];
        value?: T;
        orientation?: 'horizontal' | 'vertical';
        disabled?: boolean;
    }) {
        const handle = FFI.componentCreate('RadioButtonGroup');
        super(handle, config.value || config.options[0]?.value);
        
        this.options = config.options;
        this.orientation = config.orientation || 'vertical';
        this.disabled = config.disabled || false;
        
        this.initialize();
    }
    
    private initialize(): void {
        this.setState({
            options: this.options,
            value: this.value,
            orientation: this.orientation,
            disabled: this.disabled
        });
    }
    
    /**
     * Update options
     */
    setOptions(options: RadioOption<T>[]): void {
        this.options = options;
        this.setState({ options });
    }
    
    /**
     * Select an option by value
     */
    select(value: T): void {
        const option = this.options.find(opt => opt.value === value);
        if (option && !option.disabled) {
            this.setValue(value);
        }
    }
    
    /**
     * Get the selected option
     */
    getSelectedOption(): RadioOption<T> | undefined {
        return this.options.find(opt => opt.value === this.value);
    }
    
    validate(): boolean {
        return this.options.some(opt => opt.value === this.value);
    }
}

/**
 * Slider/Range Control Widget
 * 
 * @example
 * ```typescript
 * const slider = new Slider({
 *     min: 0,
 *     max: 100,
 *     value: 50,
 *     step: 1,
 *     showValue: true,
 *     marks: [0, 25, 50, 75, 100]
 * });
 * 
 * slider.on('change', ({ newValue }) => {
 *     console.log('Slider value:', newValue);
 * });
 * ```
 */
export class Slider extends InputWidget<number> {
    private min: number;
    private max: number;
    private step: number;
    private showValue: boolean;
    private showMarks: boolean;
    private marks?: number[];
    private orientation: 'horizontal' | 'vertical';
    
    constructor(config: {
        min: number;
        max: number;
        value?: number;
        step?: number;
        showValue?: boolean;
        showMarks?: boolean;
        marks?: number[];
        orientation?: 'horizontal' | 'vertical';
        disabled?: boolean;
    }) {
        const handle = FFI.componentCreate('Slider');
        const initialValue = config.value ?? config.min;
        super(handle, initialValue);
        
        this.min = config.min;
        this.max = config.max;
        this.step = config.step || 1;
        this.showValue = config.showValue ?? true;
        this.showMarks = config.showMarks ?? false;
        this.marks = config.marks;
        this.orientation = config.orientation || 'horizontal';
        this.disabled = config.disabled || false;
        
        this.initialize();
    }
    
    private initialize(): void {
        this.setState({
            min: this.min,
            max: this.max,
            value: this.value,
            step: this.step,
            showValue: this.showValue,
            showMarks: this.showMarks,
            marks: this.marks,
            orientation: this.orientation,
            disabled: this.disabled
        });
    }
    
    /**
     * Set the minimum value
     */
    setMin(min: number): void {
        this.min = min;
        if (this.value < min) {
            this.setValue(min);
        }
        this.setState({ min });
    }
    
    /**
     * Set the maximum value
     */
    setMax(max: number): void {
        this.max = max;
        if (this.value > max) {
            this.setValue(max);
        }
        this.setState({ max });
    }
    
    /**
     * Set the step value
     */
    setStep(step: number): void {
        this.step = step;
        this.setState({ step });
    }
    
    /**
     * Increment the value by one step
     */
    increment(): void {
        const newValue = Math.min(this.value + this.step, this.max);
        this.setValue(newValue);
    }
    
    /**
     * Decrement the value by one step
     */
    decrement(): void {
        const newValue = Math.max(this.value - this.step, this.min);
        this.setValue(newValue);
    }
    
    /**
     * Get the value as a percentage
     */
    getPercentage(): number {
        return ((this.value - this.min) / (this.max - this.min)) * 100;
    }
    
    validate(): boolean {
        return this.value >= this.min && this.value <= this.max;
    }
}

/**
 * Date Picker Widget
 * 
 * @example
 * ```typescript
 * const datePicker = new DatePicker({
 *     value: new Date(),
 *     min: new Date('2020-01-01'),
 *     max: new Date('2030-12-31'),
 *     format: 'YYYY-MM-DD'
 * });
 * 
 * datePicker.on('change', ({ newValue }) => {
 *     console.log('Selected date:', newValue);
 * });
 * ```
 */
export class DatePicker extends InputWidget<Date> {
    private min?: Date;
    private max?: Date;
    private format: string;
    private showCalendar: boolean;
    private disabledDates?: Date[];
    private highlightedDates?: Date[];
    
    constructor(config: {
        value?: Date;
        min?: Date;
        max?: Date;
        format?: string;
        showCalendar?: boolean;
        disabledDates?: Date[];
        highlightedDates?: Date[];
        disabled?: boolean;
    }) {
        const handle = FFI.componentCreate('DatePicker');
        super(handle, config.value || new Date());
        
        this.min = config.min;
        this.max = config.max;
        this.format = config.format || 'YYYY-MM-DD';
        this.showCalendar = config.showCalendar ?? true;
        this.disabledDates = config.disabledDates;
        this.highlightedDates = config.highlightedDates;
        this.disabled = config.disabled || false;
        
        this.initialize();
    }
    
    private initialize(): void {
        this.setState({
            value: this.value.toISOString(),
            min: this.min?.toISOString(),
            max: this.max?.toISOString(),
            format: this.format,
            showCalendar: this.showCalendar,
            disabledDates: this.disabledDates?.map(d => d.toISOString()),
            highlightedDates: this.highlightedDates?.map(d => d.toISOString()),
            disabled: this.disabled
        });
    }
    
    /**
     * Set the date format
     */
    setFormat(format: string): void {
        this.format = format;
        this.setState({ format });
    }
    
    /**
     * Set the minimum date
     */
    setMinDate(date: Date): void {
        this.min = date;
        if (this.value < date) {
            this.setValue(date);
        }
        this.setState({ min: date.toISOString() });
    }
    
    /**
     * Set the maximum date
     */
    setMaxDate(date: Date): void {
        this.max = date;
        if (this.value > date) {
            this.setValue(date);
        }
        this.setState({ max: date.toISOString() });
    }
    
    /**
     * Navigate to today
     */
    today(): void {
        this.setValue(new Date());
    }
    
    /**
     * Navigate to next day
     */
    nextDay(): void {
        const next = new Date(this.value);
        next.setDate(next.getDate() + 1);
        this.setValue(next);
    }
    
    /**
     * Navigate to previous day
     */
    previousDay(): void {
        const prev = new Date(this.value);
        prev.setDate(prev.getDate() - 1);
        this.setValue(prev);
    }
    
    /**
     * Format the date as a string
     */
    formatDate(): string {
        // Simple format implementation
        const year = this.value.getFullYear();
        const month = String(this.value.getMonth() + 1).padStart(2, '0');
        const day = String(this.value.getDate()).padStart(2, '0');
        
        return this.format
            .replace('YYYY', year.toString())
            .replace('MM', month)
            .replace('DD', day);
    }
    
    validate(): boolean {
        if (this.min && this.value < this.min) return false;
        if (this.max && this.value > this.max) return false;
        if (this.disabledDates?.some(d => 
            d.toDateString() === this.value.toDateString()
        )) return false;
        return true;
    }
}

/**
 * Time Picker Widget
 * 
 * @example
 * ```typescript
 * const timePicker = new TimePicker({
 *     value: { hours: 14, minutes: 30 },
 *     format: '24h',
 *     showSeconds: false,
 *     step: 15 // 15-minute increments
 * });
 * ```
 */
export interface TimeValue {
    hours: number;
    minutes: number;
    seconds?: number;
}

export class TimePicker extends InputWidget<TimeValue> {
    private format: '12h' | '24h';
    private showSeconds: boolean;
    private step: number; // minute step
    
    constructor(config: {
        value?: TimeValue;
        format?: '12h' | '24h';
        showSeconds?: boolean;
        step?: number;
        disabled?: boolean;
    }) {
        const handle = FFI.componentCreate('TimePicker');
        const defaultValue = { hours: 0, minutes: 0, seconds: 0 };
        super(handle, config.value || defaultValue);
        
        this.format = config.format || '24h';
        this.showSeconds = config.showSeconds || false;
        this.step = config.step || 1;
        this.disabled = config.disabled || false;
        
        this.initialize();
    }
    
    private initialize(): void {
        this.setState({
            value: this.value,
            format: this.format,
            showSeconds: this.showSeconds,
            step: this.step,
            disabled: this.disabled
        });
    }
    
    /**
     * Set hours
     */
    setHours(hours: number): void {
        const maxHours = this.format === '24h' ? 23 : 12;
        hours = Math.max(0, Math.min(hours, maxHours));
        this.setValue({ ...this.value, hours });
    }
    
    /**
     * Set minutes
     */
    setMinutes(minutes: number): void {
        minutes = Math.max(0, Math.min(minutes, 59));
        this.setValue({ ...this.value, minutes });
    }
    
    /**
     * Set seconds
     */
    setSeconds(seconds: number): void {
        seconds = Math.max(0, Math.min(seconds, 59));
        this.setValue({ ...this.value, seconds });
    }
    
    /**
     * Format the time as a string
     */
    formatTime(): string {
        const { hours, minutes, seconds } = this.value;
        let h = hours;
        let period = '';
        
        if (this.format === '12h') {
            period = hours >= 12 ? ' PM' : ' AM';
            h = hours % 12 || 12;
        }
        
        let time = `${String(h).padStart(2, '0')}:${String(minutes).padStart(2, '0')}`;
        
        if (this.showSeconds && seconds !== undefined) {
            time += `:${String(seconds).padStart(2, '0')}`;
        }
        
        return time + period;
    }
    
    validate(): boolean {
        const { hours, minutes, seconds } = this.value;
        const maxHours = this.format === '24h' ? 23 : 12;
        
        if (hours < 0 || hours > maxHours) return false;
        if (minutes < 0 || minutes > 59) return false;
        if (seconds !== undefined && (seconds < 0 || seconds > 59)) return false;
        
        return true;
    }
}

/**
 * Color Picker Widget
 * 
 * @example
 * ```typescript
 * const colorPicker = new ColorPicker({
 *     value: { r: 255, g: 0, b: 0 },
 *     format: 'rgb',
 *     showAlpha: true,
 *     presets: [
 *         { r: 255, g: 0, b: 0 },    // Red
 *         { r: 0, g: 255, b: 0 },    // Green
 *         { r: 0, g: 0, b: 255 }     // Blue
 *     ]
 * });
 * ```
 */
export interface ColorValue {
    r: number;
    g: number;
    b: number;
    a?: number;
}

export class ColorPicker extends InputWidget<ColorValue> {
    private format: 'rgb' | 'hex' | 'hsl';
    private showAlpha: boolean;
    private presets?: ColorValue[];
    private showInput: boolean;
    
    constructor(config: {
        value?: ColorValue;
        format?: 'rgb' | 'hex' | 'hsl';
        showAlpha?: boolean;
        presets?: ColorValue[];
        showInput?: boolean;
        disabled?: boolean;
    }) {
        const handle = FFI.componentCreate('ColorPicker');
        const defaultValue = { r: 0, g: 0, b: 0, a: 1 };
        super(handle, config.value || defaultValue);
        
        this.format = config.format || 'rgb';
        this.showAlpha = config.showAlpha || false;
        this.presets = config.presets;
        this.showInput = config.showInput ?? true;
        this.disabled = config.disabled || false;
        
        this.initialize();
    }
    
    private initialize(): void {
        this.setState({
            value: this.value,
            format: this.format,
            showAlpha: this.showAlpha,
            presets: this.presets,
            showInput: this.showInput,
            disabled: this.disabled
        });
    }
    
    /**
     * Set color from hex string
     */
    setHex(hex: string): void {
        const color = this.hexToRgb(hex);
        if (color) {
            this.setValue(color);
        }
    }
    
    /**
     * Get color as hex string
     */
    getHex(): string {
        return this.rgbToHex(this.value);
    }
    
    /**
     * Set color from HSL values
     */
    setHsl(h: number, s: number, l: number): void {
        const color = this.hslToRgb(h, s, l);
        this.setValue(color);
    }
    
    /**
     * Get color as HSL values
     */
    getHsl(): { h: number; s: number; l: number } {
        return this.rgbToHsl(this.value);
    }
    
    /**
     * Convert hex to RGB
     */
    private hexToRgb(hex: string): ColorValue | null {
        const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
        return result ? {
            r: parseInt(result[1], 16),
            g: parseInt(result[2], 16),
            b: parseInt(result[3], 16),
            a: 1
        } : null;
    }
    
    /**
     * Convert RGB to hex
     */
    private rgbToHex(color: ColorValue): string {
        const toHex = (n: number) => {
            const hex = n.toString(16);
            return hex.length === 1 ? '0' + hex : hex;
        };
        return `#${toHex(color.r)}${toHex(color.g)}${toHex(color.b)}`;
    }
    
    /**
     * Convert HSL to RGB
     */
    private hslToRgb(h: number, s: number, l: number): ColorValue {
        h /= 360;
        s /= 100;
        l /= 100;
        
        let r, g, b;
        
        if (s === 0) {
            r = g = b = l;
        } else {
            const hue2rgb = (p: number, q: number, t: number) => {
                if (t < 0) t += 1;
                if (t > 1) t -= 1;
                if (t < 1/6) return p + (q - p) * 6 * t;
                if (t < 1/2) return q;
                if (t < 2/3) return p + (q - p) * (2/3 - t) * 6;
                return p;
            };
            
            const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
            const p = 2 * l - q;
            r = hue2rgb(p, q, h + 1/3);
            g = hue2rgb(p, q, h);
            b = hue2rgb(p, q, h - 1/3);
        }
        
        return {
            r: Math.round(r * 255),
            g: Math.round(g * 255),
            b: Math.round(b * 255),
            a: 1
        };
    }
    
    /**
     * Convert RGB to HSL
     */
    private rgbToHsl(color: ColorValue): { h: number; s: number; l: number } {
        const r = color.r / 255;
        const g = color.g / 255;
        const b = color.b / 255;
        
        const max = Math.max(r, g, b);
        const min = Math.min(r, g, b);
        let h = 0, s = 0, l = (max + min) / 2;
        
        if (max !== min) {
            const d = max - min;
            s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
            
            switch (max) {
                case r: h = ((g - b) / d + (g < b ? 6 : 0)) / 6; break;
                case g: h = ((b - r) / d + 2) / 6; break;
                case b: h = ((r - g) / d + 4) / 6; break;
            }
        }
        
        return {
            h: Math.round(h * 360),
            s: Math.round(s * 100),
            l: Math.round(l * 100)
        };
    }
    
    validate(): boolean {
        const { r, g, b, a } = this.value;
        if (r < 0 || r > 255) return false;
        if (g < 0 || g > 255) return false;
        if (b < 0 || b > 255) return false;
        if (a !== undefined && (a < 0 || a > 1)) return false;
        return true;
    }
}

/**
 * File Selector Widget
 * 
 * @example
 * ```typescript
 * const fileSelector = new FileSelector({
 *     multiple: true,
 *     accept: ['.txt', '.md', '.json'],
 *     directory: false,
 *     maxSize: 10 * 1024 * 1024 // 10MB
 * });
 * 
 * fileSelector.on('change', ({ newValue }) => {
 *     console.log('Selected files:', newValue);
 * });
 * ```
 */
export interface FileInfo {
    name: string;
    path: string;
    size: number;
    type: string;
    lastModified: Date;
}

export class FileSelector extends InputWidget<FileInfo[]> {
    private multiple: boolean;
    private accept?: string[];
    private directory: boolean;
    private maxSize?: number;
    private maxFiles?: number;
    
    constructor(config: {
        multiple?: boolean;
        accept?: string[];
        directory?: boolean;
        maxSize?: number;
        maxFiles?: number;
        disabled?: boolean;
    }) {
        const handle = FFI.componentCreate('FileSelector');
        super(handle, []);
        
        this.multiple = config.multiple || false;
        this.accept = config.accept;
        this.directory = config.directory || false;
        this.maxSize = config.maxSize;
        this.maxFiles = config.maxFiles;
        this.disabled = config.disabled || false;
        
        this.initialize();
    }
    
    private initialize(): void {
        this.setState({
            multiple: this.multiple,
            accept: this.accept,
            directory: this.directory,
            maxSize: this.maxSize,
            maxFiles: this.maxFiles,
            disabled: this.disabled
        });
    }
    
    /**
     * Open file dialog
     */
    open(): void {
        // Trigger file dialog
        this.emit('open');
    }
    
    /**
     * Clear selected files
     */
    clear(): void {
        this.setValue([]);
    }
    
    /**
     * Remove a file by index
     */
    removeFile(index: number): void {
        const files = [...this.value];
        files.splice(index, 1);
        this.setValue(files);
    }
    
    /**
     * Get total size of selected files
     */
    getTotalSize(): number {
        return this.value.reduce((total, file) => total + file.size, 0);
    }
    
    /**
     * Format file size for display
     */
    formatSize(bytes: number): string {
        const units = ['B', 'KB', 'MB', 'GB'];
        let size = bytes;
        let unitIndex = 0;
        
        while (size >= 1024 && unitIndex < units.length - 1) {
            size /= 1024;
            unitIndex++;
        }
        
        return `${size.toFixed(2)} ${units[unitIndex]}`;
    }
    
    /**
     * Check if a file type is accepted
     */
    isAccepted(filename: string): boolean {
        if (!this.accept || this.accept.length === 0) return true;
        
        const ext = filename.substring(filename.lastIndexOf('.'));
        return this.accept.includes(ext);
    }
    
    validate(): boolean {
        // Check file count
        if (this.maxFiles && this.value.length > this.maxFiles) return false;
        
        // Check file sizes
        if (this.maxSize) {
            for (const file of this.value) {
                if (file.size > this.maxSize) return false;
            }
        }
        
        // Check file types
        for (const file of this.value) {
            if (!this.isAccepted(file.name)) return false;
        }
        
        return true;
    }
}