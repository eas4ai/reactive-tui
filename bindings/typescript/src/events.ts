/**
 * Event System for Reactive TUI
 * 
 * Comprehensive event handling for keyboard, mouse, and custom events.
 */

/**
 * Key codes for keyboard events
 */
export enum KeyCode {
    // Letters
    A = 'a', B = 'b', C = 'c', D = 'd', E = 'e', F = 'f', G = 'g', H = 'h',
    I = 'i', J = 'j', K = 'k', L = 'l', M = 'm', N = 'n', O = 'o', P = 'p',
    Q = 'q', R = 'r', S = 's', T = 't', U = 'u', V = 'v', W = 'w', X = 'x',
    Y = 'y', Z = 'z',
    
    // Numbers
    Num0 = '0', Num1 = '1', Num2 = '2', Num3 = '3', Num4 = '4',
    Num5 = '5', Num6 = '6', Num7 = '7', Num8 = '8', Num9 = '9',
    
    // Function keys
    F1 = 'F1', F2 = 'F2', F3 = 'F3', F4 = 'F4', F5 = 'F5', F6 = 'F6',
    F7 = 'F7', F8 = 'F8', F9 = 'F9', F10 = 'F10', F11 = 'F11', F12 = 'F12',
    
    // Navigation
    ArrowUp = 'ArrowUp',
    ArrowDown = 'ArrowDown',
    ArrowLeft = 'ArrowLeft',
    ArrowRight = 'ArrowRight',
    Home = 'Home',
    End = 'End',
    PageUp = 'PageUp',
    PageDown = 'PageDown',
    
    // Editing
    Backspace = 'Backspace',
    Delete = 'Delete',
    Insert = 'Insert',
    Enter = 'Enter',
    Tab = 'Tab',
    Escape = 'Escape',
    Space = ' ',
    
    // Modifiers (as keys)
    Shift = 'Shift',
    Control = 'Control',
    Alt = 'Alt',
    Meta = 'Meta',
}

/**
 * Modifier keys state
 */
export interface Modifiers {
    shift: boolean;
    ctrl: boolean;
    alt: boolean;
    meta: boolean;
}

/**
 * Base event interface
 */
export interface Event {
    type: string;
    timestamp: number;
    target?: any;
    currentTarget?: any;
    bubbles: boolean;
    cancelable: boolean;
    defaultPrevented: boolean;
    propagationStopped: boolean;
    
    preventDefault(): void;
    stopPropagation(): void;
    stopImmediatePropagation(): void;
}

/**
 * Keyboard event
 */
export interface KeyboardEvent extends Event {
    key: string;
    code: KeyCode;
    modifiers: Modifiers;
    repeat: boolean;
}

/**
 * Mouse button types
 */
export enum MouseButton {
    Left = 0,
    Middle = 1,
    Right = 2,
    Back = 3,
    Forward = 4,
}

/**
 * Mouse event
 */
export interface MouseEvent extends Event {
    x: number;
    y: number;
    button: MouseButton;
    buttons: number;
    modifiers: Modifiers;
    clickCount: number;
}

/**
 * Scroll event
 */
export interface ScrollEvent extends Event {
    deltaX: number;
    deltaY: number;
    deltaMode: 'pixel' | 'line' | 'page';
}

/**
 * Focus event
 */
export interface FocusEvent extends Event {
    relatedTarget?: any;
}

/**
 * Resize event
 */
export interface ResizeEvent extends Event {
    width: number;
    height: number;
    oldWidth: number;
    oldHeight: number;
}

/**
 * Custom event
 */
export interface CustomEvent<T = any> extends Event {
    detail: T;
}

/**
 * Event type strings
 */
export enum EventType {
    // Keyboard events
    KeyDown = 'keydown',
    KeyUp = 'keyup',
    KeyPress = 'keypress',
    
    // Mouse events
    Click = 'click',
    DoubleClick = 'dblclick',
    MouseDown = 'mousedown',
    MouseUp = 'mouseup',
    MouseMove = 'mousemove',
    MouseEnter = 'mouseenter',
    MouseLeave = 'mouseleave',
    MouseOver = 'mouseover',
    MouseOut = 'mouseout',
    ContextMenu = 'contextmenu',
    
    // Scroll events
    Scroll = 'scroll',
    Wheel = 'wheel',
    
    // Focus events
    Focus = 'focus',
    Blur = 'blur',
    FocusIn = 'focusin',
    FocusOut = 'focusout',
    
    // Form events
    Change = 'change',
    Input = 'input',
    Submit = 'submit',
    Reset = 'reset',
    Invalid = 'invalid',
    
    // Window events
    Resize = 'resize',
    Load = 'load',
    Unload = 'unload',
    BeforeUnload = 'beforeunload',
    
    // Custom events
    Custom = 'custom',
}

/**
 * Event listener function type
 */
export type EventListener<E extends Event = Event> = (event: E) => void;

/**
 * Event listener options
 */
export interface EventListenerOptions {
    once?: boolean;
    passive?: boolean;
    capture?: boolean;
}

/**
 * Event emitter for managing events
 */
export class EventEmitter {
    private listeners: Map<string, Set<EventListener>> = new Map();
    private captureListeners: Map<string, Set<EventListener>> = new Map();
    
    /**
     * Add an event listener
     */
    addEventListener(
        type: string,
        listener: EventListener,
        options?: EventListenerOptions
    ): void {
        const map = options?.capture ? this.captureListeners : this.listeners;
        
        if (!map.has(type)) {
            map.set(type, new Set());
        }
        
        const listeners = map.get(type)!;
        
        if (options?.once) {
            const onceListener: EventListener = (event) => {
                listener(event);
                this.removeEventListener(type, onceListener, options);
            };
            listeners.add(onceListener);
        } else {
            listeners.add(listener);
        }
    }
    
    /**
     * Remove an event listener
     */
    removeEventListener(
        type: string,
        listener: EventListener,
        options?: EventListenerOptions
    ): void {
        const map = options?.capture ? this.captureListeners : this.listeners;
        const listeners = map.get(type);
        
        if (listeners) {
            listeners.delete(listener);
            if (listeners.size === 0) {
                map.delete(type);
            }
        }
    }
    
    /**
     * Dispatch an event
     */
    dispatchEvent(event: Event): boolean {
        // Capture phase
        const captureListeners = this.captureListeners.get(event.type);
        if (captureListeners && !event.propagationStopped) {
            for (const listener of captureListeners) {
                listener(event);
                if (event.propagationStopped) break;
            }
        }
        
        // Bubble phase
        const bubbleListeners = this.listeners.get(event.type);
        if (bubbleListeners && !event.propagationStopped) {
            for (const listener of bubbleListeners) {
                listener(event);
                if (event.propagationStopped) break;
            }
        }
        
        return !event.defaultPrevented;
    }
    
    /**
     * Emit a custom event
     */
    emit<T = any>(type: string, detail?: T): void {
        const event = createCustomEvent(type, detail);
        this.dispatchEvent(event);
    }
    
    /**
     * Shorthand for addEventListener
     */
    on(type: string, listener: EventListener, options?: EventListenerOptions): void {
        this.addEventListener(type, listener, options);
    }
    
    /**
     * Shorthand for removeEventListener
     */
    off(type: string, listener: EventListener, options?: EventListenerOptions): void {
        this.removeEventListener(type, listener, options);
    }
    
    /**
     * Add a one-time event listener
     */
    once(type: string, listener: EventListener): void {
        this.addEventListener(type, listener, { once: true });
    }
    
    /**
     * Remove all listeners for a specific event type
     */
    removeAllListeners(type?: string): void {
        if (type) {
            this.listeners.delete(type);
            this.captureListeners.delete(type);
        } else {
            this.listeners.clear();
            this.captureListeners.clear();
        }
    }
}

/**
 * Create a base event
 */
export function createEvent(type: string, options?: Partial<Event>): Event {
    const event: Event = {
        type,
        timestamp: Date.now(),
        bubbles: options?.bubbles ?? true,
        cancelable: options?.cancelable ?? true,
        defaultPrevented: false,
        propagationStopped: false,
        target: options?.target,
        currentTarget: options?.currentTarget,
        
        preventDefault() {
            if (this.cancelable) {
                this.defaultPrevented = true;
            }
        },
        
        stopPropagation() {
            this.propagationStopped = true;
        },
        
        stopImmediatePropagation() {
            this.propagationStopped = true;
        },
    };
    
    return event;
}

/**
 * Create a keyboard event
 */
export function createKeyboardEvent(
    type: string,
    key: string,
    modifiers?: Partial<Modifiers>
): KeyboardEvent {
    const event = createEvent(type) as KeyboardEvent;
    
    event.key = key;
    event.code = key as KeyCode;
    event.modifiers = {
        shift: modifiers?.shift ?? false,
        ctrl: modifiers?.ctrl ?? false,
        alt: modifiers?.alt ?? false,
        meta: modifiers?.meta ?? false,
    };
    event.repeat = false;
    
    return event;
}

/**
 * Create a mouse event
 */
export function createMouseEvent(
    type: string,
    x: number,
    y: number,
    button?: MouseButton,
    modifiers?: Partial<Modifiers>
): MouseEvent {
    const event = createEvent(type) as MouseEvent;
    
    event.x = x;
    event.y = y;
    event.button = button ?? MouseButton.Left;
    event.buttons = 1 << event.button;
    event.modifiers = {
        shift: modifiers?.shift ?? false,
        ctrl: modifiers?.ctrl ?? false,
        alt: modifiers?.alt ?? false,
        meta: modifiers?.meta ?? false,
    };
    event.clickCount = type === EventType.DoubleClick ? 2 : 1;
    
    return event;
}

/**
 * Create a custom event
 */
export function createCustomEvent<T = any>(type: string, detail?: T): CustomEvent<T> {
    const event = createEvent(type) as CustomEvent<T>;
    event.detail = detail!;
    return event;
}

/**
 * Key combination parser
 */
export class KeyCombo {
    constructor(
        public key: string,
        public modifiers: Modifiers
    ) {}
    
    /**
     * Parse a key combination string
     * @example "Ctrl+Shift+A", "Alt+Enter", "F1"
     */
    static parse(combo: string): KeyCombo {
        const parts = combo.split('+').map(p => p.trim().toLowerCase());
        const modifiers: Modifiers = {
            shift: false,
            ctrl: false,
            alt: false,
            meta: false,
        };
        
        let key = '';
        
        for (const part of parts) {
            switch (part) {
                case 'shift':
                    modifiers.shift = true;
                    break;
                case 'ctrl':
                case 'control':
                    modifiers.ctrl = true;
                    break;
                case 'alt':
                    modifiers.alt = true;
                    break;
                case 'meta':
                case 'cmd':
                case 'win':
                    modifiers.meta = true;
                    break;
                default:
                    key = part;
            }
        }
        
        return new KeyCombo(key, modifiers);
    }
    
    /**
     * Check if a keyboard event matches this combo
     */
    matches(event: KeyboardEvent): boolean {
        return event.key.toLowerCase() === this.key.toLowerCase() &&
            event.modifiers.shift === this.modifiers.shift &&
            event.modifiers.ctrl === this.modifiers.ctrl &&
            event.modifiers.alt === this.modifiers.alt &&
            event.modifiers.meta === this.modifiers.meta;
    }
    
    toString(): string {
        const parts = [];
        if (this.modifiers.ctrl) parts.push('Ctrl');
        if (this.modifiers.shift) parts.push('Shift');
        if (this.modifiers.alt) parts.push('Alt');
        if (this.modifiers.meta) parts.push('Meta');
        parts.push(this.key);
        return parts.join('+');
    }
}

/**
 * Global keyboard shortcuts manager
 */
export class KeyboardShortcuts {
    private shortcuts: Map<string, EventListener<KeyboardEvent>> = new Map();
    private emitter = new EventEmitter();
    
    /**
     * Register a keyboard shortcut
     */
    register(combo: string, handler: EventListener<KeyboardEvent>): void {
        this.shortcuts.set(combo, handler);
    }
    
    /**
     * Unregister a keyboard shortcut
     */
    unregister(combo: string): void {
        this.shortcuts.delete(combo);
    }
    
    /**
     * Handle a keyboard event
     */
    handleKeyEvent(event: KeyboardEvent): void {
        for (const [comboStr, handler] of this.shortcuts) {
            const combo = KeyCombo.parse(comboStr);
            if (combo.matches(event)) {
                handler(event);
                if (event.defaultPrevented) break;
            }
        }
    }
    
    /**
     * Clear all shortcuts
     */
    clear(): void {
        this.shortcuts.clear();
    }
}