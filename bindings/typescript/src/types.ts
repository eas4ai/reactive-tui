/**
 * Common type definitions for Reactive TUI
 */

/**
 * RGB color representation
 */
export interface Color {
  r: number;
  g: number;
  b: number;
  a?: number;
}

/**
 * Position in 2D space
 */
export interface Position {
  x: number;
  y: number;
}

/**
 * Size dimensions
 */
export interface Size {
  width: number;
  height: number;
}

/**
 * Rectangle bounds
 */
export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

/**
 * Padding values
 */
export interface Padding {
  top: number;
  right: number;
  bottom: number;
  left: number;
}

/**
 * Margin values
 */
export interface Margin {
  top: number;
  right: number;
  bottom: number;
  left: number;
}

/**
 * Border configuration
 */
export interface Border {
  style: BorderStyle;
  width: number;
  color: Color | string;
  radius?: number;
}

/**
 * Border styles
 */
export enum BorderStyle {
  None = 'none',
  Solid = 'solid',
  Double = 'double',
  Rounded = 'rounded',
  Thick = 'thick',
  Dashed = 'dashed',
  Dotted = 'dotted',
}

/**
 * Text alignment options
 */
export enum TextAlign {
  Left = 'left',
  Center = 'center',
  Right = 'right',
  Justify = 'justify',
}

/**
 * Vertical alignment options
 */
export enum VerticalAlign {
  Top = 'top',
  Middle = 'middle',
  Bottom = 'bottom',
}

/**
 * Layout direction
 */
export enum Direction {
  Horizontal = 'horizontal',
  Vertical = 'vertical',
}

/**
 * Flex alignment options
 */
export enum FlexAlign {
  Start = 'start',
  Center = 'center',
  End = 'end',
  Stretch = 'stretch',
  SpaceBetween = 'space-between',
  SpaceAround = 'space-around',
  SpaceEvenly = 'space-evenly',
}

/**
 * Input event types
 */
export enum EventType {
  KeyPress = 'keypress',
  KeyDown = 'keydown',
  KeyUp = 'keyup',
  MouseMove = 'mousemove',
  MouseDown = 'mousedown',
  MouseUp = 'mouseup',
  MouseClick = 'click',
  MouseDoubleClick = 'dblclick',
  MouseWheel = 'wheel',
  Focus = 'focus',
  Blur = 'blur',
  Resize = 'resize',
}

/**
 * Key codes for keyboard events
 */
export enum KeyCode {
  Backspace = 8,
  Tab = 9,
  Enter = 13,
  Shift = 16,
  Ctrl = 17,
  Alt = 18,
  Escape = 27,
  Space = 32,
  PageUp = 33,
  PageDown = 34,
  End = 35,
  Home = 36,
  ArrowLeft = 37,
  ArrowUp = 38,
  ArrowRight = 39,
  ArrowDown = 40,
  Insert = 45,
  Delete = 46,
  F1 = 112,
  F2 = 113,
  F3 = 114,
  F4 = 115,
  F5 = 116,
  F6 = 117,
  F7 = 118,
  F8 = 119,
  F9 = 120,
  F10 = 121,
  F11 = 122,
  F12 = 123,
}

/**
 * Mouse button codes
 */
export enum MouseButton {
  Left = 0,
  Middle = 1,
  Right = 2,
}

/**
 * Keyboard event
 */
export interface KeyEvent {
  type: EventType;
  key: string;
  keyCode: KeyCode;
  char?: string;
  shift: boolean;
  ctrl: boolean;
  alt: boolean;
  meta: boolean;
}

/**
 * Mouse event
 */
export interface MouseEvent {
  type: EventType;
  x: number;
  y: number;
  button: MouseButton;
  buttons: number;
  shift: boolean;
  ctrl: boolean;
  alt: boolean;
  meta: boolean;
  deltaX?: number;
  deltaY?: number;
}

/**
 * Resize event
 */
export interface ResizeEvent {
  type: EventType.Resize;
  width: number;
  height: number;
  oldWidth: number;
  oldHeight: number;
}

/**
 * Generic event
 */
export type Event = KeyEvent | MouseEvent | ResizeEvent;

/**
 * Style properties for components
 */
export interface Style {
  // Layout
  display?: 'block' | 'inline' | 'flex' | 'grid' | 'none';
  position?: 'static' | 'relative' | 'absolute' | 'fixed';
  width?: number | string;
  height?: number | string;
  minWidth?: number | string;
  minHeight?: number | string;
  maxWidth?: number | string;
  maxHeight?: number | string;
  
  // Spacing
  padding?: Padding | number | string;
  margin?: Margin | number | string;
  
  // Border
  border?: Border | string;
  borderRadius?: number;
  
  // Colors
  color?: Color | string;
  backgroundColor?: Color | string;
  
  // Text
  fontSize?: number;
  fontWeight?: 'normal' | 'bold';
  textAlign?: TextAlign;
  textDecoration?: 'none' | 'underline' | 'line-through';
  
  // Flex container
  flexDirection?: 'row' | 'column';
  justifyContent?: FlexAlign;
  alignItems?: FlexAlign;
  flexWrap?: 'nowrap' | 'wrap';
  gap?: number;
  
  // Flex item
  flex?: number | string;
  flexGrow?: number;
  flexShrink?: number;
  flexBasis?: number | string;
  alignSelf?: FlexAlign;
  
  // Grid
  gridTemplateColumns?: string;
  gridTemplateRows?: string;
  gridGap?: number;
  gridColumn?: string;
  gridRow?: string;
  
  // Visibility
  opacity?: number;
  visibility?: 'visible' | 'hidden';
  zIndex?: number;
  
  // Overflow
  overflow?: 'visible' | 'hidden' | 'scroll' | 'auto';
  overflowX?: 'visible' | 'hidden' | 'scroll' | 'auto';
  overflowY?: 'visible' | 'hidden' | 'scroll' | 'auto';
}

/**
 * Theme configuration
 */
export interface Theme {
  name: string;
  colors: {
    primary: Color | string;
    secondary: Color | string;
    background: Color | string;
    foreground: Color | string;
    border: Color | string;
    text: Color | string;
    error: Color | string;
    warning: Color | string;
    success: Color | string;
    info: Color | string;
    [key: string]: Color | string;
  };
  fonts?: {
    default?: string;
    mono?: string;
  };
  spacing?: {
    unit: number;
    small: number;
    medium: number;
    large: number;
  };
  borderRadius?: {
    small: number;
    medium: number;
    large: number;
  };
}

/**
 * Component lifecycle hooks
 */
export interface LifecycleHooks {
  onMount?: () => void;
  onUnmount?: () => void;
  onUpdate?: (oldProps: any, newProps: any) => void;
  onBeforeRender?: () => void;
  onAfterRender?: () => void;
}

/**
 * Render context
 */
export interface RenderContext {
  width: number;
  height: number;
  theme: Theme;
  isDirty: boolean;
  parentContext?: RenderContext;
}

/**
 * Layout constraints
 */
export interface LayoutConstraints {
  minWidth?: number;
  minHeight?: number;
  maxWidth?: number;
  maxHeight?: number;
  preferredWidth?: number;
  preferredHeight?: number;
}

/**
 * Animation state
 */
export interface AnimationState {
  isRunning: boolean;
  progress: number;
  currentValue: number;
  startValue: number;
  endValue: number;
  duration: number;
  elapsed: number;
}

/**
 * Focus management
 */
export interface FocusOptions {
  preventScroll?: boolean;
  focusVisible?: boolean;
}

/**
 * Scroll options
 */
export interface ScrollOptions {
  behavior?: 'auto' | 'smooth';
  block?: 'start' | 'center' | 'end' | 'nearest';
  inline?: 'start' | 'center' | 'end' | 'nearest';
}

/**
 * Common component props
 */
export interface CommonProps {
  id?: string;
  className?: string;
  style?: Style;
  hidden?: boolean;
  disabled?: boolean;
  tabIndex?: number;
  role?: string;
  ariaLabel?: string;
  ariaDescribedBy?: string;
  ariaLabelledBy?: string;
  onKeyDown?: (event: KeyEvent) => void;
  onKeyUp?: (event: KeyEvent) => void;
  onKeyPress?: (event: KeyEvent) => void;
  onClick?: (event: MouseEvent) => void;
  onDoubleClick?: (event: MouseEvent) => void;
  onMouseDown?: (event: MouseEvent) => void;
  onMouseUp?: (event: MouseEvent) => void;
  onMouseMove?: (event: MouseEvent) => void;
  onMouseEnter?: (event: MouseEvent) => void;
  onMouseLeave?: (event: MouseEvent) => void;
  onFocus?: () => void;
  onBlur?: () => void;
}

/**
 * Utility type for deep partial
 */
export type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P];
};

/**
 * Utility type for required keys
 */
export type RequireKeys<T, K extends keyof T> = T & Required<Pick<T, K>>;

/**
 * Utility type for optional keys
 */
export type OptionalKeys<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>;

/**
 * Helper function to convert hex color to RGB
 */
export function hexToRgb(hex: string): Color {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  if (!result) {
    throw new Error(`Invalid hex color: ${hex}`);
  }
  return {
    r: parseInt(result[1], 16),
    g: parseInt(result[2], 16),
    b: parseInt(result[3], 16),
    a: 1.0,
  };
}

/**
 * Helper function to convert RGB to hex
 */
export function rgbToHex(color: Color): string {
  const toHex = (n: number) => {
    const hex = Math.round(n).toString(16);
    return hex.length === 1 ? '0' + hex : hex;
  };
  return `#${toHex(color.r)}${toHex(color.g)}${toHex(color.b)}`;
}

/**
 * Helper function to create padding from a single value
 */
export function createPadding(value: number | string | Padding): Padding {
  if (typeof value === 'object') {
    return value;
  }
  const numValue = typeof value === 'string' ? parseInt(value) : value;
  return {
    top: numValue,
    right: numValue,
    bottom: numValue,
    left: numValue,
  };
}

/**
 * Helper function to create margin from a single value
 */
export function createMargin(value: number | string | Margin): Margin {
  if (typeof value === 'object') {
    return value;
  }
  const numValue = typeof value === 'string' ? parseInt(value) : value;
  return {
    top: numValue,
    right: numValue,
    bottom: numValue,
    left: numValue,
  };
}

/**
 * Helper to check if a point is inside a rectangle
 */
export function pointInRect(x: number, y: number, rect: Rect): boolean {
  return x >= rect.x && 
         x < rect.x + rect.width && 
         y >= rect.y && 
         y < rect.y + rect.height;
}

/**
 * Helper to check if two rectangles intersect
 */
export function rectsIntersect(a: Rect, b: Rect): boolean {
  return !(a.x + a.width <= b.x || 
           b.x + b.width <= a.x || 
           a.y + a.height <= b.y || 
           b.y + b.height <= a.y);
}

/**
 * Helper to clamp a value between min and max
 */
export function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value));
}