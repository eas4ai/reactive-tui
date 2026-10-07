#ifndef REACTIVE_TUI_NATIVE_H
#define REACTIVE_TUI_NATIVE_H

/* Generated from Rust by cbindgen 0.29.4. Run scripts/generate-native-header.py. */

#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
typedef struct RTuiRenderer RTuiRenderer;
typedef struct RTuiBuffer RTuiBuffer;
typedef struct RTuiTerminal RTuiTerminal;
typedef struct RTuiTextBuffer RTuiTextBuffer;
typedef struct RTuiAnimationManager RTuiAnimationManager;
typedef struct RTuiAnimation RTuiAnimation;
typedef struct RTuiAppBuilder RTuiAppBuilder;
typedef struct RTuiElement RTuiElement;
typedef struct RTuiApp RTuiApp;
typedef struct RTuiElementBuilder RTuiElementBuilder;
typedef struct RTuiSignal RTuiSignal;
typedef struct RTuiThreadSafeSignal RTuiThreadSafeSignal;
typedef struct RTuiEffect RTuiEffect;
typedef struct RTuiHooks RTuiHooks;
typedef struct RTuiSurface RTuiSurface;
typedef struct RTuiTextEditor RTuiTextEditor;
typedef struct RTuiNativeStyle RTuiNativeStyle;
typedef struct RTuiDialogEngine RTuiDialogEngine;
typedef struct RTuiForeignComponent RTuiForeignComponent;

/* Signal ownership: each constructor and hook lookup returns an owning handle.
 * Release it exactly once with either rtui_signal_destroy or
 * rtui_signal_destroy_new. Handles are confined to their creating thread;
 * use RTuiThreadSafeSignal only with its separate functions/destructor.
 * Both RTuiSignal function families use the same typed allocation. String,
 * boolean and float access interoperates; legacy int is int64_t while the
 * improved int is C int. Wrong live types return InvalidParameter (or the
 * documented getter default). Arbitrary/stale pointers are invalid.
 * Owned getter strings require rtui_string_free independently of the signal.
 *
 * Snapshot arrays returned by bufferGet*Ptr remain owned by the library until
 * their matching bufferRelease*Ptr call. The library retains their allocation
 * metadata; the length argument is accepted for ABI compatibility and does not
 * control deallocation. Interior, misaligned, and repeated releases are ignored.
 *
 * createOptimizedBuffer and createTextBuffer return tracked owning handles.
 * Their destroy functions ignore NULL and already-destroyed handles. AppBuilder
 * build consumes every non-NULL builder, including on failure. Animation-manager
 * add consumes the animation on success. rtui_app_run borrows its retained App
 * handle while it blocks; another thread may call rtui_app_quit, and the owner
 * destroys the handle only after run returns. Optional root and effect-cleanup
 * callbacks may be NULL.
 */


/**
 * Error codes for FFI functions
 */
typedef enum RTuiError {
  /**
   * Operation succeeded
   */
  R_TUI_ERROR_SUCCESS = 0,
  /**
   * Invalid parameter passed
   */
  R_TUI_ERROR_INVALID_PARAMETER = -1,
  /**
   * Null pointer passed where not allowed
   */
  R_TUI_ERROR_NULL_POINTER = -2,
  /**
   * Buffer too small for operation
   */
  R_TUI_ERROR_BUFFER_TOO_SMALL = -3,
  /**
   * Out of memory
   */
  R_TUI_ERROR_OUT_OF_MEMORY = -4,
  /**
   * Invalid UTF-8 string
   */
  R_TUI_ERROR_INVALID_UTF8 = -5,
  /**
   * Terminal not available or not supported
   */
  R_TUI_ERROR_TERMINAL_NOT_AVAILABLE = -6,
  /**
   * Operation not supported
   */
  R_TUI_ERROR_NOT_SUPPORTED = -7,
  /**
   * Resource already exists
   */
  R_TUI_ERROR_ALREADY_EXISTS = -8,
  /**
   * Resource not found
   */
  R_TUI_ERROR_NOT_FOUND = -9,
  /**
   * Invalid state for operation
   */
  R_TUI_ERROR_INVALID_STATE = -10,
  /**
   * Invalid pointer (use-after-free or corrupted)
   */
  R_TUI_ERROR_INVALID_POINTER = -11,
  /**
   * Internal error occurred
   */
  R_TUI_ERROR_INTERNAL_ERROR = -12,
  /**
   * Panic occurred (bug in library)
   */
  R_TUI_ERROR_PANIC = -99,
  /**
   * Unknown error
   */
  R_TUI_ERROR_UNKNOWN = -100,
} RTuiError;

/**
 * Event type
 */
typedef enum RTuiEventType {
  /**
   * Keyboard event
   */
  R_TUI_EVENT_TYPE_KEY = 0,
  /**
   * Mouse event
   */
  R_TUI_EVENT_TYPE_MOUSE = 1,
  /**
   * Terminal resize event
   */
  R_TUI_EVENT_TYPE_RESIZE = 2,
  /**
   * Focus change event
   */
  R_TUI_EVENT_TYPE_FOCUS = 3,
  /**
   * Paste event
   */
  R_TUI_EVENT_TYPE_PASTE = 4,
} RTuiEventType;

/**
 * Easing function types
 */
typedef enum RTuiEasingType {
  /**
   * Linear interpolation
   */
  R_TUI_EASING_TYPE_LINEAR = 0,
  /**
   * Ease in (slow start)
   */
  R_TUI_EASING_TYPE_EASE_IN = 1,
  /**
   * Ease out (slow end)
   */
  R_TUI_EASING_TYPE_EASE_OUT = 2,
  /**
   * Ease in and out
   */
  R_TUI_EASING_TYPE_EASE_IN_OUT = 3,
  /**
   * Bounce effect
   */
  R_TUI_EASING_TYPE_BOUNCE = 4,
  /**
   * Elastic effect
   */
  R_TUI_EASING_TYPE_ELASTIC = 5,
  /**
   * Back effect (overshoot)
   */
  R_TUI_EASING_TYPE_BACK = 6,
  /**
   * Exponential easing
   */
  R_TUI_EASING_TYPE_EXPO = 7,
  /**
   * Circular easing
   */
  R_TUI_EASING_TYPE_CIRC = 8,
  /**
   * Sine wave easing
   */
  R_TUI_EASING_TYPE_SINE = 9,
  /**
   * Quadratic easing
   */
  R_TUI_EASING_TYPE_QUAD = 10,
  /**
   * Cubic easing
   */
  R_TUI_EASING_TYPE_CUBIC = 11,
  /**
   * Quartic easing
   */
  R_TUI_EASING_TYPE_QUART = 12,
  /**
   * Quintic easing
   */
  R_TUI_EASING_TYPE_QUINT = 13,
  /**
   * Spring effect
   */
  R_TUI_EASING_TYPE_SPRING = 14,
} RTuiEasingType;

/**
 * Loop behavior types
 */
typedef enum RTuiLoopMode {
  /**
   * No looping
   */
  R_TUI_LOOP_MODE_NONE = 0,
  /**
   * Loop infinitely
   */
  R_TUI_LOOP_MODE_INFINITE = 1,
  /**
   * Loop a specific number of times
   */
  R_TUI_LOOP_MODE_COUNT = 2,
  /**
   * Ping-pong back and forth
   */
  R_TUI_LOOP_MODE_PING_PONG = 3,
} RTuiLoopMode;

/**
 * Animation property types
 */
typedef enum RTuiAnimatedProperty {
  /**
   * Opacity animation
   */
  R_TUI_ANIMATED_PROPERTY_OPACITY = 0,
  /**
   * X-axis translation
   */
  R_TUI_ANIMATED_PROPERTY_TRANSLATE_X = 1,
  /**
   * Y-axis translation
   */
  R_TUI_ANIMATED_PROPERTY_TRANSLATE_Y = 2,
  /**
   * X-axis scaling
   */
  R_TUI_ANIMATED_PROPERTY_SCALE_X = 3,
  /**
   * Y-axis scaling
   */
  R_TUI_ANIMATED_PROPERTY_SCALE_Y = 4,
  /**
   * Rotation animation
   */
  R_TUI_ANIMATED_PROPERTY_ROTATION = 5,
  /**
   * Width animation
   */
  R_TUI_ANIMATED_PROPERTY_WIDTH = 6,
  /**
   * Height animation
   */
  R_TUI_ANIMATED_PROPERTY_HEIGHT = 7,
} RTuiAnimatedProperty;

/**
 * Performance mode enumeration
 */
typedef enum RTuiPerformanceMode {
  /**
   * Power saving mode (lower FPS)
   */
  R_TUI_PERFORMANCE_MODE_POWER_SAVE = 0,
  /**
   * Balanced mode (moderate FPS)
   */
  R_TUI_PERFORMANCE_MODE_BALANCED = 1,
  /**
   * Performance mode (high FPS)
   */
  R_TUI_PERFORMANCE_MODE_PERFORMANCE = 2,
} RTuiPerformanceMode;

/**
 * Layout types
 */
typedef enum RTuiLayoutType {
  /**
   * Flexbox layout
   */
  R_TUI_LAYOUT_TYPE_FLEX = 0,
  /**
   * Grid layout
   */
  R_TUI_LAYOUT_TYPE_GRID = 1,
  /**
   * Stack layout
   */
  R_TUI_LAYOUT_TYPE_STACK = 2,
  /**
   * Absolute positioning
   */
  R_TUI_LAYOUT_TYPE_ABSOLUTE = 3,
} RTuiLayoutType;

/**
 * Element types
 */
typedef enum RTuiElementType {
  /**
   * Component element
   */
  R_TUI_ELEMENT_TYPE_COMPONENT = 0,
  /**
   * Text element
   */
  R_TUI_ELEMENT_TYPE_TEXT = 1,
  /**
   * Layout element
   */
  R_TUI_ELEMENT_TYPE_LAYOUT = 2,
  /**
   * Fragment element
   */
  R_TUI_ELEMENT_TYPE_FRAGMENT = 3,
  /**
   * Empty element
   */
  R_TUI_ELEMENT_TYPE_EMPTY = 4,
} RTuiElementType;

/**
 * Version information for ABI compatibility
 */
typedef struct RTuiVersion {
  /**
   * Major version number
   */
  uint32_t major;
  /**
   * Minor version number
   */
  uint32_t minor;
  /**
   * Patch version number
   */
  uint32_t patch;
  /**
   * ABI version for compatibility checking
   */
  uint32_t abi_version;
} RTuiVersion;

/**
 * Terminal capabilities structure
 */
typedef struct RTuiCapabilities {
  /**
   * RGB color support
   */
  bool rgb;
  /**
   * 256 color support  
   */
  bool color_256;
  /**
   * Unicode support level (0=basic, 1=extended, 2=full)
   */
  uint8_t unicode_level;
  /**
   * Kitty keyboard protocol support
   */
  bool kitty_keyboard;
  /**
   * Mouse support
   */
  bool mouse;
  /**
   * Pixel mouse support
   */
  bool pixel_mouse;
  /**
   * Hyperlinks support
   */
  bool hyperlinks;
  /**
   * Image support
   */
  bool images;
  /**
   * Synchronized output support
   */
  bool synchronized_output;
  /**
   * Bracketed paste support
   */
  bool bracketed_paste;
} RTuiCapabilities;

/**
 * Terminal dimensions
 */
typedef struct RTuiDimensions {
  /**
   * Width in characters
   */
  uint16_t width;
  /**
   * Height in characters
   */
  uint16_t height;
} RTuiDimensions;

/**
 * Key event
 */
typedef struct RTuiKeyEvent {
  /**
   * Key code
   */
  uint32_t key_code;
  /**
   * Modifier keys (bit flags: 1=Shift, 2=Ctrl, 4=Alt, 8=Meta)
   */
  uint8_t modifiers;
} RTuiKeyEvent;

/**
 * Mouse event
 */
typedef struct RTuiMouseEvent {
  /**
   * X coordinate
   */
  uint16_t x;
  /**
   * Y coordinate
   */
  uint16_t y;
  /**
   * Mouse button
   */
  uint8_t button;
  /**
   * Modifier keys
   */
  uint8_t modifiers;
  /**
   * Event type (0=Press, 1=Release, 2=Move, 3=ScrollUp, 4=ScrollDown)
   */
  uint8_t event_type;
} RTuiMouseEvent;

/**
 * Resize event
 */
typedef struct RTuiResizeEvent {
  /**
   * New width
   */
  uint16_t width;
  /**
   * New height
   */
  uint16_t height;
} RTuiResizeEvent;

/**
 * Event union
 */
typedef union RTuiEventData {
  /**
   * Key event data
   */
  struct RTuiKeyEvent key;
  /**
   * Mouse event data
   */
  struct RTuiMouseEvent mouse;
  /**
   * Resize event data
   */
  struct RTuiResizeEvent resize;
} RTuiEventData;

/**
 * Event structure
 */
typedef struct RTuiEvent {
  /**
   * Type of event
   */
  enum RTuiEventType event_type;
  /**
   * Event data
   */
  union RTuiEventData data;
} RTuiEvent;

/**
 * Root component callback function type.
 *
 * Pass `Some(callback)` to register a callback or `None` to leave it unset.
 * A bare function pointer is not a nullable callback:
 *
 * ```compile_fail
 * use reactive_tui::ffi::RTuiElement;
 * use std::ffi::c_void;
 *
 * type BareRootCallback = extern "C" fn(*mut c_void) -> *mut RTuiElement;
 *
 * fn nullable(
 *     callback: BareRootCallback,
 * ) -> Option<extern "C" fn(*mut c_void) -> *mut RTuiElement> {
 *     callback
 * }
 * ```
 */
typedef RTuiElement *(*RTuiRootComponentCallback)(void *user_data);

/**
 * Performance metrics structure
 */
typedef struct RTuiPerformanceMetrics {
  /**
   * Current frames per second
   */
  float current_fps;
  /**
   * Average render time in milliseconds
   */
  float avg_render_time_ms;
  /**
   * Frame drop rate as percentage
   */
  float drop_rate_percent;
  /**
   * Whether performance is stable
   */
  bool is_stable;
} RTuiPerformanceMetrics;

/**
 * Return zero and an owned Element, or a native error code. Strings are borrowed.
 */
typedef int32_t (*RTuiForeignRenderCallback)(const char*, const char*, void*, RTuiElement**);

/**
 * Return zero and set handled, or return a native error code. Strings are borrowed.
 */
typedef int32_t (*RTuiForeignEventCallback)(const char*, const char*, const char*, void*, bool*);

/**
 * Called once by successful explicit destruction, never by a renderer worker.
 */
typedef void (*RTuiForeignDisposeCallback)(void*);

/**
 * Effect callback function type
 */
typedef void (*RTuiEffectCallback)(void *user_data);

/**
 * Effect cleanup callback function type.
 *
 * Pass `Some(callback)` to register cleanup or `None` when no cleanup is needed.
 * A bare function pointer is not a nullable callback:
 *
 * ```compile_fail
 * use std::ffi::c_void;
 *
 * type BareCleanupCallback = extern "C" fn(*mut c_void);
 *
 * fn nullable(
 *     callback: BareCleanupCallback,
 * ) -> Option<extern "C" fn(*mut c_void)> {
 *     callback
 * }
 * ```
 */
typedef void (*RTuiEffectCleanupCallback)(void *user_data);

/**
 * RGB color
 */
typedef struct RTuiColor {
  /**
   * Red component (0-255)
   */
  uint8_t r;
  /**
   * Green component (0-255)
   */
  uint8_t g;
  /**
   * Blue component (0-255)
   */
  uint8_t b;
} RTuiColor;

/**
 * Text attributes flags
 */
typedef struct RTuiTextAttributes {
  /**
   * Bold text
   */
  bool bold;
  /**
   * Italic text
   */
  bool italic;
  /**
   * Underlined text
   */
  bool underline;
  /**
   * Strikethrough text
   */
  bool strikethrough;
  /**
   * Reverse video
   */
  bool reverse;
  /**
   * Blinking text
   */
  bool blink;
  /**
   * Hidden text
   */
  bool hidden;
} RTuiTextAttributes;

/**
 * Cell in the terminal
 */
typedef struct RTuiCell {
  /**
   * Unicode codepoint
   */
  uint32_t ch;
  /**
   * Foreground color
   */
  struct RTuiColor fg;
  /**
   * Background color
   */
  struct RTuiColor bg;
  /**
   * Text attributes
   */
  struct RTuiTextAttributes attrs;
} RTuiCell;

/**
 * Rectangle area
 */
typedef struct RTuiRect {
  /**
   * X coordinate of top-left corner
   */
  uint16_t x;
  /**
   * Y coordinate of top-left corner
   */
  uint16_t y;
  /**
   * Width of the rectangle
   */
  uint16_t width;
  /**
   * Height of the rectangle
   */
  uint16_t height;
} RTuiRect;

/**
 * Log callback function type (matching OpenTUI)
 */
typedef void (*LogCallback)(uint8_t level, const uint8_t *msg_ptr, size_t msg_len);

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * Get the library version
 */
struct RTuiVersion rtui_version(void);

/**
 * Initialize the library (must be called before any other functions)
 */
enum RTuiError rtui_init(void);

/**
 * Cleanup the library
 */
void rtui_cleanup(void);

/**
 * Create a new renderer
 */
RTuiRenderer *createRenderer(uint32_t width, uint32_t height);

/**
 * Destroy a renderer
 *
 * # Safety
 *
 * `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, no other call may use it until this one returns, and it is not used again after this call.
 */
void destroyRenderer(RTuiRenderer *renderer,
                     bool _use_alternate_screen,
                     uint32_t _split_height);

/**
 * Set renderer background color
 *
 * # Safety
 *
 * `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 *
 * `color` must be null or point to four readable `f32` values: red, green, blue and alpha, each from 0 to 1.
 */
void setBackgroundColor(RTuiRenderer *renderer,
                        const float *color);

/**
 * Render the current frame
 *
 * # Safety
 *
 * `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
void render(RTuiRenderer *renderer,
            bool force);

/**
 * Resize renderer
 *
 * # Safety
 *
 * `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
void resizeRenderer(RTuiRenderer *renderer,
                    uint32_t width,
                    uint32_t height);

/**
 * Create an optimized buffer.
 * `width_method` is accepted for compatibility; one width policy applies to all buffers.
 */
RTuiBuffer *createOptimizedBuffer(uint32_t width,
                                  uint32_t height,
                                  bool _respect_alpha,
                                  uint8_t _width_method,
                                  const uint8_t *_id_ptr,
                                  size_t _id_len);

/**
 * Destroy an optimized buffer
 *
 * # Safety
 *
 * `buffer` must be null or a live `RTuiBuffer` handle that this library returned and has not destroyed, no other call may use it until this one returns, and it is not used again after this call.
 */
void destroyOptimizedBuffer(RTuiBuffer *buffer);

/**
 * Get buffer width
 *
 * # Safety
 *
 * `buffer` must be null or a live `RTuiBuffer` handle that this library returned and has not destroyed, and no call may destroy it until this one returns.
 */
uint32_t getBufferWidth(const RTuiBuffer *buffer);

/**
 * Get buffer height
 *
 * # Safety
 *
 * `buffer` must be null or a live `RTuiBuffer` handle that this library returned and has not destroyed, and no call may destroy it until this one returns.
 */
uint32_t getBufferHeight(const RTuiBuffer *buffer);

/**
 * Clear buffer with background color
 *
 * # Safety
 *
 * `buffer` must be null or a live `RTuiBuffer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 *
 * `bg` must be null or point to four readable `f32` values: red, green, blue and alpha, each from 0 to 1.
 */
void bufferClear(RTuiBuffer *buffer,
                 const float *bg);

/**
 * Draw text to buffer
 *
 * # Safety
 *
 * `buffer` must be null or a live `RTuiBuffer` handle this library returned
 * and has not destroyed, and no other call may use it until this one
 * returns. `text` must be null or the start of as many readable bytes as
 * `text_len` says. `fg` must be null or four readable `f32` color
 * components. `bg` must be null or four readable `f32` color components.
 */
void bufferDrawText(RTuiBuffer *buffer,
                    const uint8_t *text,
                    size_t text_len,
                    uint32_t x,
                    uint32_t y,
                    const float *fg,
                    const float *bg,
                    uint8_t attributes);

/**
 * Set a single cell with alpha blending
 *
 * # Safety
 *
 * `buffer` must be null or a live `RTuiBuffer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 *
 * `fg` must be null or point to four readable `f32` values: red, green, blue and alpha, each from 0 to 1.
 *
 * `bg` must be null or point to four readable `f32` values: red, green, blue and alpha, each from 0 to 1.
 */
void bufferSetCellWithAlphaBlending(RTuiBuffer *buffer,
                                    uint32_t x,
                                    uint32_t y,
                                    uint32_t character,
                                    const float *fg,
                                    const float *bg,
                                    uint8_t attributes);

/**
 * Fill rectangle with background color
 *
 * # Safety
 *
 * `buffer` must be null or a live `RTuiBuffer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 *
 * `bg` must be null or point to four readable `f32` values: red, green, blue and alpha, each from 0 to 1.
 */
void bufferFillRect(RTuiBuffer *buffer,
                    uint32_t x,
                    uint32_t y,
                    uint32_t width,
                    uint32_t height,
                    const float *bg);

/**
 * Get direct pointer to character buffer
 *
 * Returns a pointer to a contiguous array of u32 characters.
 * The array has width * height elements in row-major order.
 *
 * # Safety
 *
 * `buffer` must be null or a live `RTuiBuffer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 *
 * - The returned pointer is valid until its matching release call
 * - The caller must not access beyond width * height elements
 * - Concurrent access must be synchronized by the caller
 * - The caller must call bufferReleaseCharPtr to free the memory
 */
uint32_t *bufferGetCharPtr(RTuiBuffer *buffer);

/**
 * Release character buffer pointer
 *
 * **REQUIRED:** Must be called for every pointer returned by bufferGetCharPtr()
 * **SAFETY:** Only call this once per pointer, and only with pointers from bufferGetCharPtr()
 *
 * # Safety
 *
 * `ptr` must be null or a pointer that `bufferGetCharPtr` returned and that has not been released; it is not used again after this call.
 */
void bufferReleaseCharPtr(uint32_t *ptr,
                          size_t _length);

/**
 * Get direct pointer to foreground color buffer
 *
 * Returns a pointer to a contiguous array of f32 RGBA values.
 * The array has width * height * 4 elements (RGBA per cell) in row-major order.
 *
 * # Safety
 *
 * `buffer` must be null or a live `RTuiBuffer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 *
 * - The returned pointer is valid until its matching release call
 * - The caller must not access beyond width * height * 4 elements
 * - The caller must call bufferReleaseFgPtr to free the memory
 */
float *bufferGetFgPtr(RTuiBuffer *buffer);

/**
 * Release foreground color buffer pointer
 *
 * **REQUIRED:** Must be called for every pointer returned by bufferGetFgPtr()
 * **SAFETY:** Only call this once per pointer, and only with pointers from bufferGetFgPtr()
 *
 * # Safety
 *
 * `ptr` must be null or a pointer that `bufferGetFgPtr` returned and that has not been released; it is not used again after this call.
 */
void bufferReleaseFgPtr(float *ptr,
                        size_t _length);

/**
 * Get direct pointer to background color buffer
 *
 * Returns a pointer to a contiguous array of f32 RGBA values.
 * The array has width * height * 4 elements (RGBA per cell) in row-major order.
 *
 * # Safety
 *
 * `buffer` must be null or a live `RTuiBuffer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 *
 * - The returned pointer is valid until its matching release call
 * - The caller must not access beyond width * height * 4 elements
 * - The caller must call bufferReleaseBgPtr to free the memory
 */
float *bufferGetBgPtr(RTuiBuffer *buffer);

/**
 * Release background color buffer pointer
 *
 * **REQUIRED:** Must be called for every pointer returned by bufferGetBgPtr()
 * **SAFETY:** Only call this once per pointer, and only with pointers from bufferGetBgPtr()
 *
 * # Safety
 *
 * `ptr` must be null or a pointer that `bufferGetBgPtr` returned and that has not been released; it is not used again after this call.
 */
void bufferReleaseBgPtr(float *ptr,
                        size_t _length);

/**
 * Get direct pointer to attributes buffer
 *
 * Returns a pointer to a contiguous array of u8 attribute flags.
 * The array has width * height elements in row-major order.
 *
 * # Safety
 *
 * `buffer` must be null or a live `RTuiBuffer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 *
 * - The returned pointer is valid until its matching release call
 * - The caller must not access beyond width * height elements
 * - The caller must call bufferReleaseAttributesPtr to free the memory
 */
uint8_t *bufferGetAttributesPtr(RTuiBuffer *buffer);

/**
 * Release attributes buffer pointer
 *
 * **REQUIRED:** Must be called for every pointer returned by bufferGetAttrPtr()
 * **SAFETY:** Only call this once per pointer, and only with pointers from bufferGetAttrPtr()
 *
 * # Safety
 *
 * `ptr` must be null or a pointer that `bufferGetAttrPtr` returned and that has not been released; it is not used again after this call.
 */
void bufferReleaseAttrPtr(uint8_t *ptr,
                          size_t _length);

/**
 * Get buffer respect alpha setting
 */
bool bufferGetRespectAlpha(const RTuiBuffer *buffer);

/**
 * Set buffer respect alpha setting
 *
 * Controls whether alpha blending is respected when rendering.
 * When enabled, transparent colors will blend with background.
 * When disabled, alpha values are ignored and colors are rendered opaque.
 *
 * # Safety
 *
 * `buffer` must be null or a live `RTuiBuffer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
void bufferSetRespectAlpha(RTuiBuffer *buffer,
                           bool respect_alpha);

/**
 * Resize buffer
 *
 * # Safety
 *
 * `buffer` must be null or a live `RTuiBuffer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
void bufferResize(RTuiBuffer *buffer,
                  uint32_t width,
                  uint32_t height);

/**
 * Paint the complete surface through the caller's terminal, preserving graphemes and pictures.
 * No renderer or terminal session is created, and no terminal mode is changed or restored.
 *
 * # Safety
 *
 * `surface` must be null or a live `RTuiBuffer` handle that this library returned and has not destroyed, and no call may destroy it until this one returns.
 *
 * `terminal` must be null or a live `RTuiTerminal` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
bool renderSurfaceToTerminal(const RTuiBuffer *surface,
                             RTuiTerminal *terminal);

/**
 * Complete rendering pipeline: TextBuffer → Surface → Renderer → Terminal
 *
 * This integrates all systems in a single high-level operation
 *
 * # Safety
 *
 * `text_buffer` must be null or a live `RTuiTextBuffer` handle that this library returned and has not destroyed, and no call may destroy it until this one returns.
 *
 * `terminal` must be null or a live `RTuiTerminal` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
bool renderTextToTerminal(const RTuiTextBuffer *text_buffer,
                          RTuiTerminal *terminal,
                          uint32_t x,
                          uint32_t y,
                          uint32_t width,
                          uint32_t height);

/**
 * Integrated renderer with stats collection
 *
 * Render the frame through the renderer's own terminal, with optional statistics collection.
 * The terminal argument is validated but is not written to.
 *
 * # Safety
 *
 * `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 *
 * `terminal` must be null or a live `RTuiTerminal` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
bool renderWithStats(RTuiRenderer *renderer,
                     RTuiTerminal *terminal,
                     bool collect_stats);

/**
 * Keep the host time, FPS and frame callback time for the debug overlay and buffer dump.
 * Also enable detailed measured frame statistics.
 *
 * # Safety
 *
 * `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
void updateStats(RTuiRenderer *renderer,
                 double time,
                 uint32_t fps,
                 double frame_callback_time);

/**
 * Keep the host heap-used, heap-total and array-buffer values for the overlay and buffer dump.
 *
 * # Safety
 *
 * `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
void updateMemoryStats(RTuiRenderer *renderer,
                       uint32_t heap_used,
                       uint32_t heap_total,
                       uint32_t array_buffers);

/**
 * Add `offset` to every emitted cursor row, including the debug overlay.
 * Rows beyond the terminal are still written; changing the offset forces a repaint.
 *
 * # Safety
 *
 * `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
void setRenderOffset(RTuiRenderer *renderer,
                     uint32_t offset);

/**
 * Set debug overlay
 *
 * # Safety
 *
 * `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
void setDebugOverlay(RTuiRenderer *renderer,
                     bool enabled,
                     uint8_t _corner);

/**
 * Write `id` over a region clipped to the renderer in the grid being built.
 * Negative origins clip; outside regions are ignored; later registrations overwrite earlier ones.
 * A successful completed render publishes this grid and starts an empty one.
 *
 * # Safety
 *
 * `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
void addToHitGrid(RTuiRenderer *renderer,
                  int32_t x,
                  int32_t y,
                  uint32_t width,
                  uint32_t height,
                  uint32_t id);

/**
 * Return the registered id at (x, y) in the last completed frame, or 0 for an empty or outside cell.
 *
 * # Safety
 *
 * `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
uint32_t checkHit(RTuiRenderer *renderer,
                  uint32_t x,
                  uint32_t y);

/**
 * Log each distinct nonzero id in the shown hit grid with its cells' bounding box.
 *
 * # Safety
 *
 * `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
void dumpHitGrid(RTuiRenderer *renderer);

/**
 * Write `rtui-buffers-<timestamp>.txt` in the current directory.
 * The text contains dimensions, front and back surfaces row by row (each cell's
 * grapheme, or a space for an empty cell), last frame statistics and kept host statistics.
 * Log the path on success or the write error on failure, without panicking.
 *
 * # Safety
 *
 * `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
void dumpBuffers(RTuiRenderer *renderer,
                 int64_t timestamp);

/**
 * Dump stdout buffer for debugging
 *
 * # Safety
 *
 * `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
void dumpStdoutBuffer(RTuiRenderer *renderer,
                      int64_t timestamp);

/**
 * Set log callback for debugging
 */
void setLogCallback(void (*callback)(uint8_t level, const uint8_t *msg_ptr, size_t msg_len));

/**
 * Start profiling session
 *
 * # Safety
 *
 * `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
void startProfiling(RTuiRenderer *renderer);

/**
 * Stop profiling session
 *
 * # Safety
 *
 * `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
void stopProfiling(RTuiRenderer *renderer);

/**
 * Get frame timing statistics
 *
 * # Safety
 *
 * `renderer` must be null or a live `RTuiRenderer` handle this library
 * returned and has not destroyed, and no call may destroy it until this one
 * returns. `out_avg_frame_time` must be null or a `f32` the caller owns,
 * which this call may write. `out_min_frame_time` must be null or a `f32`
 * the caller owns, which this call may write. `out_max_frame_time` must be
 * null or a `f32` the caller owns, which this call may write.
 * `out_frame_count` must be null or a `u32` the caller owns, which this call
 * may write.
 */
void getFrameStats(const RTuiRenderer *renderer,
                   float *out_avg_frame_time,
                   float *out_min_frame_time,
                   float *out_max_frame_time,
                   uint32_t *out_frame_count);

/**
 * Reset performance counters
 *
 * # Safety
 *
 * `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
void resetPerformanceCounters(RTuiRenderer *renderer);

/**
 * Create terminal instance
 */
RTuiTerminal *createTerminal(void);

/**
 * Destroy terminal instance
 *
 * # Safety
 *
 * `terminal` must be null or a live `RTuiTerminal` handle that this library returned and has not destroyed, no other call may use it until this one returns, and it is not used again after this call.
 */
void destroyTerminal(RTuiTerminal *terminal);

/**
 * Setup terminal for TUI mode
 *
 * # Safety
 *
 * `terminal` must be null or a live `RTuiTerminal` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
void setupTerminal(RTuiTerminal *terminal,
                   bool use_alternate_screen);

/**
 * Clear terminal
 */
void clearTerminal(RTuiTerminal *terminal);

/**
 * Get terminal capabilities
 *
 * # Safety
 *
 * `terminal` must be null or a live `RTuiTerminal` handle this library
 * returned and has not destroyed, and no call may destroy it until this one
 * returns. `caps_ptr` must be null or a `Capabilities` the caller owns,
 * which this call may write.
 */
void getTerminalCapabilities(const RTuiTerminal *terminal, struct RTuiCapabilities *caps_ptr);

/**
 * Process capability response
 *
 * # Safety
 *
 * `terminal` must be null or a live `RTuiTerminal` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns. `response_ptr` must be null or the start of as many readable
 * bytes as `response_len` says.
 */
void processCapabilityResponse(RTuiTerminal *terminal,
                               const uint8_t *response_ptr,
                               size_t response_len);

/**
 * Set cursor position
 */
void setCursorPosition(RTuiTerminal *terminal, int32_t x, int32_t y, bool visible);

/**
 * Set cursor style
 *
 * # Safety
 *
 * `terminal` must be null or a live `RTuiTerminal` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns. `style_ptr` must be null or the start of as many readable
 * bytes as `style_len` says.
 */
void setCursorStyle(RTuiTerminal *terminal,
                    const uint8_t *style_ptr,
                    size_t style_len,
                    bool blinking);

/**
 * Set cursor color
 *
 * # Safety
 *
 * `color` must be null or point to four readable `f32` values: red, green, blue and alpha, each from 0 to 1.
 */
void setCursorColor(RTuiTerminal *terminal,
                    const float *color);

/**
 * Set terminal title
 *
 * # Safety
 *
 * `terminal` must be null or a live `RTuiTerminal` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns. `title_ptr` must be null or the start of as many readable
 * bytes as `title_len` says.
 */
void setTerminalTitle(RTuiTerminal *terminal, const uint8_t *title_ptr, size_t title_len);

/**
 * Enable mouse support
 */
void enableMouse(RTuiTerminal *terminal, bool enable_movement);

/**
 * Disable mouse support
 */
void disableMouse(RTuiTerminal *terminal);

/**
 * Enable Kitty keyboard protocol
 */
void enableKittyKeyboard(RTuiTerminal *terminal, uint8_t flags);

/**
 * Disable Kitty keyboard protocol
 */
void disableKittyKeyboard(RTuiTerminal *terminal);

/**
 * Create a terminal handle without entering raw mode.
 *
 * # Safety
 *
 * `out_terminal` must be null or a pointer slot the caller owns, which this
 * call may write.
 */
enum RTuiError rtui_terminal_create(RTuiTerminal **out_terminal);

/**
 * Release a terminal handle. Null and unregistered handles are ignored.
 *
 * # Safety
 *
 * `terminal` must be null or a live `ReactiveTerminal` handle that this library returned and has not destroyed, no other call may use it until this one returns, and it is not used again after this call.
 */
void rtui_terminal_destroy(RTuiTerminal *terminal);

/**
 * Read the host terminal size. No synthetic dimensions are returned on error.
 *
 * # Safety
 *
 * `terminal` must be null or a live `ReactiveTerminal` handle this library
 * returned and has not destroyed, and no call may destroy it until this one
 * returns. `out_dimensions` must be null or a `RTuiDimensions` the caller
 * owns, which this call may write.
 */
enum RTuiError rtui_terminal_get_dimensions(const RTuiTerminal *terminal,
                                            struct RTuiDimensions *out_dimensions);

/**
 * Begin or end a synchronized terminal update.
 *
 * # Safety
 *
 * `terminal` must be null or a live `ReactiveTerminal` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
enum RTuiError rtui_terminal_sync(RTuiTerminal *terminal,
                                  bool begin);

/**
 * Poll once; zero is nonblocking. Unsupported payloads leave output unchanged.
 *
 * # Safety
 *
 * `out_event` must be null or a `RTuiEvent` the caller owns, which this call
 * may write.
 */
enum RTuiError rtui_terminal_poll_event(uint32_t timeout_ms, struct RTuiEvent *out_event);

/**
 * Create a new text buffer.
 *
 * `width_method` is accepted for compatibility and does not change the width
 * policy: text is painted using the crate's grapheme display width.
 */
RTuiTextBuffer *createTextBuffer(uint32_t length, uint8_t _width_method);

/**
 * Destroy a text buffer
 *
 * # Safety
 *
 * `tb` must be null or a live `RTuiTextBuffer` handle that this library returned and has not destroyed, no other call may use it until this one returns, and it is not used again after this call.
 */
void destroyTextBuffer(RTuiTextBuffer *tb);

/**
 * Get direct pointer to character data
 *
 * # Safety
 *
 * `tb` must be null or a live `RTuiTextBuffer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
uint32_t *textBufferGetCharPtr(RTuiTextBuffer *tb);

/**
 * Get text buffer length
 *
 * # Safety
 *
 * `tb` must be null or a live `RTuiTextBuffer` handle that this library returned and has not destroyed, and no call may destroy it until this one returns.
 */
uint32_t textBufferGetLength(const RTuiTextBuffer *tb);

/**
 * Get text buffer capacity
 *
 * # Safety
 *
 * `tb` must be null or a live `RTuiTextBuffer` handle that this library returned and has not destroyed, and no call may destroy it until this one returns.
 */
uint32_t textBufferGetCapacity(const RTuiTextBuffer *tb);

/**
 * Resize text buffer
 *
 * # Safety
 *
 * `tb` must be null or a live `RTuiTextBuffer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
void textBufferResize(RTuiTextBuffer *tb,
                      uint32_t new_length);

/**
 * Reset text buffer
 *
 * # Safety
 *
 * `tb` must be null or a live `RTuiTextBuffer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
void textBufferReset(RTuiTextBuffer *tb);

/**
 * Write a chunk of text
 *
 * # Safety
 *
 * `tb` must be null or a live `RTuiTextBuffer` handle this library returned
 * and has not destroyed, and no other call may use it until this one
 * returns. `text_bytes` must be null or the start of as many readable bytes
 * as `text_len` says. `fg` must be null or four readable `f32` color
 * components. `bg` must be null or four readable `f32` color components.
 * `attr` must be null or a readable `u8`.
 */
uint32_t textBufferWriteChunk(RTuiTextBuffer *tb,
                              const uint8_t *text_bytes,
                              uint32_t text_len,
                              const float *fg,
                              const float *bg,
                              const uint8_t *attr);

/**
 * Set selection range
 *
 * # Safety
 *
 * `tb` must be null or a live `RTuiTextBuffer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 *
 * `bg_color` must be null or point to four readable `f32` values: red, green, blue and alpha, each from 0 to 1.
 *
 * `fg_color` must be null or point to four readable `f32` values: red, green, blue and alpha, each from 0 to 1.
 */
void textBufferSetSelection(RTuiTextBuffer *tb,
                            uint32_t start,
                            uint32_t end,
                            const float *bg_color,
                            const float *fg_color);

/**
 * Render text buffer to surface
 *
 * # Safety
 *
 * `tb` must be null or a live `RTuiTextBuffer` handle that this library returned and has not destroyed, and no call may destroy it until this one returns.
 *
 * `buffer` must be null or a live `RTuiBuffer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
uint32_t renderTextBufferToSurface(const RTuiTextBuffer *tb,
                                   RTuiBuffer *buffer,
                                   uint32_t x,
                                   uint32_t y,
                                   uint32_t max_width);

/**
 * Render text buffer to renderer surface
 *
 * # Safety
 *
 * `tb` must be null or a live `RTuiTextBuffer` handle that this library returned and has not destroyed, and no call may destroy it until this one returns.
 *
 * `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
uint32_t renderTextBufferToRenderer(const RTuiTextBuffer *tb,
                                    RTuiRenderer *renderer,
                                    uint32_t x,
                                    uint32_t y,
                                    uint32_t max_width);

/**
 * Integrated text rendering with automatic surface management
 *
 * Creates a temporary surface, renders text to it, then renders to terminal
 * This provides a complete TextBuffer → Surface → Renderer → Terminal pipeline
 *
 * # Safety
 *
 * `tb` must be null or a live `RTuiTextBuffer` handle that this library returned and has not destroyed, and no call may destroy it until this one returns.
 *
 * `terminal` must be null or a live `RTuiTerminal` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
bool renderTextBufferDirect(const RTuiTextBuffer *tb,
                            RTuiTerminal *terminal,
                            uint32_t x,
                            uint32_t y,
                            uint32_t width,
                            uint32_t height);

/**
 * Reset selection
 *
 * # Safety
 *
 * `tb` must be null or a live `RTuiTextBuffer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
void textBufferResetSelection(RTuiTextBuffer *tb);

/**
 * Get selection info as packed u64: `[start:u32][end:u32]`.
 * Returns 0xFFFFFFFF_FFFFFFFF if no selection
 *
 * # Safety
 *
 * `tb` must be null or a live `RTuiTextBuffer` handle that this library returned and has not destroyed, and no call may destroy it until this one returns.
 */
uint64_t textBufferGetSelectionInfo(const RTuiTextBuffer *tb);

/**
 * Set default foreground color
 *
 * # Safety
 *
 * `tb` must be null or a live `RTuiTextBuffer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 *
 * `fg` must be null or point to four readable `f32` values: red, green, blue and alpha, each from 0 to 1.
 */
void textBufferSetDefaultFg(RTuiTextBuffer *tb,
                            const float *fg);

/**
 * Set default background color
 *
 * # Safety
 *
 * `tb` must be null or a live `RTuiTextBuffer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 *
 * `bg` must be null or point to four readable `f32` values: red, green, blue and alpha, each from 0 to 1.
 */
void textBufferSetDefaultBg(RTuiTextBuffer *tb,
                            const float *bg);

/**
 * Set default attributes
 *
 * # Safety
 *
 * `tb` must be null or a live `RTuiTextBuffer` handle this library returned
 * and has not destroyed, and no other call may use it until this one
 * returns. `attr` must be null or a readable `u8`.
 */
void textBufferSetDefaultAttributes(RTuiTextBuffer *tb, const uint8_t *attr);

/**
 * Reset all defaults
 *
 * # Safety
 *
 * `tb` must be null or a live `RTuiTextBuffer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
void textBufferResetDefaults(RTuiTextBuffer *tb);

/**
 * Create a new animation manager
 *
 * # Safety
 *
 * `out_manager` must be null or a pointer slot the caller owns, which this
 * call may write.
 */
enum RTuiError rtui_animation_manager_create(RTuiAnimationManager **out_manager);

/**
 * Destroy an animation manager
 *
 * # Safety
 *
 * `manager` must be null or a live `RTuiAnimationManager` handle that this library returned and has not destroyed, no other call may use it until this one returns, and it is not used again after this call.
 */
void rtui_animation_manager_destroy(RTuiAnimationManager *manager);

/**
 * Update the animation manager
 *
 * # Safety
 *
 * `manager` must be null or a live `RTuiAnimationManager` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
enum RTuiError rtui_animation_manager_update(RTuiAnimationManager *manager);

/**
 * Create a new animation
 *
 * # Safety
 *
 * `id` must be null or a NUL-terminated string that stays readable during
 * the call. `out_animation` must be null or a pointer slot the caller owns,
 * which this call may write.
 */
enum RTuiError rtui_animation_create(const char *id,
                                     uint32_t duration_ms,
                                     enum RTuiEasingType easing,
                                     enum RTuiLoopMode loop_mode,
                                     uint32_t loop_count,
                                     RTuiAnimation **out_animation);

/**
 * Destroy an animation
 *
 * # Safety
 *
 * `animation` must be null or a live `RTuiAnimation` handle that this library returned and has not destroyed, no other call may use it until this one returns, and it is not used again after this call.
 */
void rtui_animation_destroy(RTuiAnimation *animation);

/**
 * Set animation property
 *
 * # Safety
 *
 * `animation` must be null or a live `RTuiAnimation` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
enum RTuiError rtui_animation_set_property(RTuiAnimation *animation,
                                           enum RTuiAnimatedProperty property,
                                           float from_value,
                                           float to_value);

/**
 * Add animation to manager
 *
 * # Safety
 *
 * `manager` must be null or a live `RTuiAnimationManager` handle this
 * library returned and has not destroyed, and no other call may use it until
 * this one returns. `animation` must be null or a live `RTuiAnimation`
 * handle this library returned and has not destroyed, and no other call may
 * use it until this one returns. `out_id` must be null or a pointer slot the
 * caller owns; this call stores a string there that the caller releases with
 * `rtui_string_free`, keeping every byte and the terminating NUL where this
 * library put them until then.
 */
enum RTuiError rtui_animation_manager_add(RTuiAnimationManager *manager,
                                          RTuiAnimation *animation,
                                          char **out_id);

/**
 * Remove animation from manager
 *
 * # Safety
 *
 * `manager` must be null or a live `RTuiAnimationManager` handle this
 * library returned and has not destroyed, and no other call may use it until
 * this one returns. `animation_id` must be null or a NUL-terminated string
 * that stays readable during the call.
 */
enum RTuiError rtui_animation_manager_remove(RTuiAnimationManager *manager,
                                             const char *animation_id);

/**
 * Play an animation
 *
 * # Safety
 *
 * `animation` must be null or a live `RTuiAnimation` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
enum RTuiError rtui_animation_play(RTuiAnimation *animation);

/**
 * Pause an animation
 *
 * # Safety
 *
 * `animation` must be null or a live `RTuiAnimation` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
enum RTuiError rtui_animation_pause(RTuiAnimation *animation);

/**
 * Stop an animation
 *
 * # Safety
 *
 * `animation` must be null or a live `RTuiAnimation` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
enum RTuiError rtui_animation_stop(RTuiAnimation *animation);

/**
 * Check if animation is playing
 *
 * # Safety
 *
 * `animation` must be null or a live `RTuiAnimation` handle this library
 * returned and has not destroyed, and no call may destroy it until this one
 * returns. `out_playing` must be null or a `bool` the caller owns, which
 * this call may write.
 */
enum RTuiError rtui_animation_is_playing(const RTuiAnimation *animation, bool *out_playing);

/**
 * Get animation progress (0.0 to 1.0)
 *
 * # Safety
 *
 * `animation` must be null or a live `RTuiAnimation` handle this library
 * returned and has not destroyed, and no call may destroy it until this one
 * returns. `out_progress` must be null or a `f32` the caller owns, which
 * this call may write.
 */
enum RTuiError rtui_animation_get_progress(const RTuiAnimation *animation, float *out_progress);

/**
 * Create a spring animation
 *
 * # Safety
 *
 * `id` must be null or a NUL-terminated string that stays readable during
 * the call. `out_animation` must be null or a pointer slot the caller owns,
 * which this call may write.
 */
enum RTuiError rtui_animation_create_spring(const char *id,
                                            float stiffness,
                                            float damping,
                                            float mass,
                                            RTuiAnimation **out_animation);

/**
 * Retain an Element root, consuming it on success. Nested foreign components
 * use the normal fallible component runtime without an infallible root callback.
 *
 * # Safety
 *
 * `builder` must be null or a live `RTuiAppBuilder` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 *
 * `element` must be null or a live `RTuiElement` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
enum RTuiError rtui_app_builder_root_element(RTuiAppBuilder *builder,
                                             RTuiElement *element);

/**
 * Create a new app builder
 *
 * # Safety
 *
 * `out_builder` must be null or a pointer slot the caller owns, which this
 * call may write.
 */
enum RTuiError rtui_app_builder_create(RTuiAppBuilder **out_builder);

/**
 * Destroy an app builder
 *
 * # Safety
 *
 * `builder` must be null or a live `RTuiAppBuilder` handle that this library returned and has not destroyed, no other call may use it until this one returns, and it is not used again after this call.
 */
void rtui_app_builder_destroy(RTuiAppBuilder *builder);

/**
 * Set debug mode for app builder (safe in-place modification)
 *
 * # Safety
 *
 * `builder` must be null or a live `RTuiAppBuilder` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
enum RTuiError rtui_app_builder_debug(RTuiAppBuilder *builder,
                                      bool debug);

/**
 * Set performance mode for app builder (safe in-place modification)
 *
 * # Safety
 *
 * `builder` must be null or a live `RTuiAppBuilder` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
enum RTuiError rtui_app_builder_performance_mode(RTuiAppBuilder *builder,
                                                 enum RTuiPerformanceMode mode);

/**
 * Set backend for app builder (creates debug backend with specified size)
 *
 * # Safety
 *
 * `builder` must be null or a live `RTuiAppBuilder` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
enum RTuiError rtui_app_builder_backend_debug(RTuiAppBuilder *builder,
                                              uint16_t width,
                                              uint16_t height);

/**
 * Select the complete-frame SuprTUI renderer and enter its terminal session.
 * The builder owns that session until build transfers it to the App, or the
 * builder is destroyed. Setup errors leave the existing builder value intact.
 * Re-selecting either native terminal route retains its current session.
 *
 * # Safety
 *
 * `builder` must be null or a live `RTuiAppBuilder` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
enum RTuiError rtui_app_builder_backend_suprtui(RTuiAppBuilder *builder);

/**
 * Set backend for app builder (creates crossterm backend)
 *
 * # Safety
 *
 * `builder` must be null or a live `RTuiAppBuilder` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
enum RTuiError rtui_app_builder_backend_crossterm(RTuiAppBuilder *builder);

/**
 * Set root component for app builder (safe in-place modification)
 *
 * # Safety
 *
 * `builder` must be null or a live `RTuiAppBuilder` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 *
 * `user_data` must stay valid for as long as the callback it is passed back to can run.
 */
enum RTuiError rtui_app_builder_root_component(RTuiAppBuilder *builder,
                                               RTuiRootComponentCallback callback,
                                               void *user_data);

/**
 * Build the app from the builder
 *
 * # Safety
 *
 * `builder` must be null or a live `RTuiAppBuilder` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns. `out_app` must be null or a pointer slot the caller owns,
 * which this call may write.
 */
enum RTuiError rtui_app_builder_build(RTuiAppBuilder *builder, RTuiApp **out_app);

/**
 * Destroy an app
 *
 * # Safety
 *
 * `app` must be null or a live `RTuiApp` handle that this library returned and has not destroyed, no other call may use it until this one returns, and it is not used again after this call.
 */
void rtui_app_destroy(RTuiApp *app);

/**
 * Run the app as a blocking call. The handle remains valid until destroy.
 * While this call is active, only quit is available; other app calls return
 * InvalidState.
 *
 * # Safety
 *
 * `app` must be null or a live `RTuiApp` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
enum RTuiError rtui_app_run(RTuiApp *app);

/**
 * Stop the app
 *
 * # Safety
 *
 * `app` must be null or a live `RTuiApp` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
enum RTuiError rtui_app_quit(RTuiApp *app);

/**
 * Get app terminal size
 *
 * # Safety
 *
 * `app` must be null or a live `RTuiApp` handle this library returned and
 * has not destroyed, and no call may destroy it until this one returns.
 * `out_dimensions` must be null or a `RTuiDimensions` the caller owns, which
 * this call may write.
 */
enum RTuiError rtui_app_get_size(const RTuiApp *app, struct RTuiDimensions *out_dimensions);

/**
 * Set app performance mode
 *
 * # Safety
 *
 * `app` must be null or a live `RTuiApp` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
enum RTuiError rtui_app_set_performance_mode(RTuiApp *app,
                                             enum RTuiPerformanceMode mode);

/**
 * Get current FPS
 *
 * # Safety
 *
 * `app` must be null or a live `RTuiApp` handle this library returned and
 * has not destroyed, and no call may destroy it until this one returns.
 * `out_fps` must be null or a `u32` the caller owns, which this call may
 * write.
 */
enum RTuiError rtui_app_get_current_fps(const RTuiApp *app, uint32_t *out_fps);

/**
 * Get performance metrics
 *
 * # Safety
 *
 * `app` must be null or a live `RTuiApp` handle this library returned and
 * has not destroyed, and no call may destroy it until this one returns.
 * `out_metrics` must be null or a `RTuiPerformanceMetrics` the caller owns,
 * which this call may write.
 */
enum RTuiError rtui_app_get_performance_metrics(const RTuiApp *app,
                                                struct RTuiPerformanceMetrics *out_metrics);

/**
 * Create a div element builder
 *
 * # Safety
 *
 * `out_builder` must be null or a pointer slot the caller owns, which this
 * call may write.
 */
enum RTuiError rtui_element_builder_div(RTuiElementBuilder **out_builder);

/**
 * Create a span element builder
 *
 * # Safety
 *
 * `out_builder` must be null or a pointer slot the caller owns, which this
 * call may write.
 */
enum RTuiError rtui_element_builder_span(RTuiElementBuilder **out_builder);

/**
 * Create a button element builder
 *
 * # Safety
 *
 * `out_builder` must be null or a pointer slot the caller owns, which this
 * call may write.
 */
enum RTuiError rtui_element_builder_button(RTuiElementBuilder **out_builder);

/**
 * Add CSS classes to element builder (REAL in-place modification)
 *
 * # Safety
 *
 * `builder` must be null or a live `RTuiElementBuilder` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns. `classes` must be null or a NUL-terminated string that stays
 * readable during the call.
 */
enum RTuiError rtui_element_builder_add_class(RTuiElementBuilder *builder, const char *classes);

/**
 * Set text content for element builder (REAL in-place modification)
 *
 * # Safety
 *
 * `builder` must be null or a live `RTuiElementBuilder` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns. `text` must be null or a NUL-terminated string that stays
 * readable during the call.
 */
enum RTuiError rtui_element_builder_set_text(RTuiElementBuilder *builder, const char *text);

/**
 * Set key for element builder (REAL in-place modification)
 *
 * # Safety
 *
 * `builder` must be null or a live `RTuiElementBuilder` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns. `key` must be null or a NUL-terminated string that stays
 * readable during the call.
 */
enum RTuiError rtui_element_builder_set_key(RTuiElementBuilder *builder, const char *key);

/**
 * Add a child element to builder (REAL in-place modification)
 *
 * # Safety
 *
 * `builder` must be null or a live `RTuiElementBuilder` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 *
 * `child` must be null or a live `RTuiElement` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
enum RTuiError rtui_element_builder_add_child(RTuiElementBuilder *builder,
                                              RTuiElement *child);

/**
 * Build the final element (consumes the builder)
 *
 * # Safety
 *
 * `builder` must be null or a live `RTuiElementBuilder` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns. `out_element` must be null or a pointer slot the caller owns,
 * which this call may write.
 */
enum RTuiError rtui_element_builder_build(RTuiElementBuilder *builder, RTuiElement **out_element);

/**
 * Destroy an element builder
 *
 * # Safety
 *
 * `builder` must be null or a live `RTuiElementBuilder` handle that this library returned and has not destroyed, no other call may use it until this one returns, and it is not used again after this call.
 */
void rtui_element_builder_destroy(RTuiElementBuilder *builder);

/**
 * Destroy an element
 *
 * # Safety
 *
 * `element` must be null or a live `RTuiElement` handle that this library returned and has not destroyed, no other call may use it until this one returns, and it is not used again after this call.
 */
void rtui_element_destroy(RTuiElement *element);

/**
 * Create a legacy div builder; the caller owns the returned builder.
 *
 * # Safety
 *
 * `out` must be null or a pointer slot the caller owns, which this call may
 * write.
 */
enum RTuiError rtui_div(RTuiElementBuilder **out);

/**
 * Create a legacy inline span builder; the caller owns the returned builder.
 *
 * # Safety
 *
 * `out` must be null or a pointer slot the caller owns, which this call may
 * write.
 */
enum RTuiError rtui_span(RTuiElementBuilder **out);

/**
 * Create a legacy styled button builder without a click callback.
 *
 * # Safety
 *
 * `out` must be null or a pointer slot the caller owns, which this call may
 * write.
 */
enum RTuiError rtui_button(RTuiElementBuilder **out);

/**
 * Create a paragraph builder using the native paragraph classes.
 *
 * # Safety
 *
 * `out` must be null or point to a writable `*mut RTuiElementBuilder`.
 */
enum RTuiError rtui_p(RTuiElementBuilder **out);

/**
 * Create a level-one heading builder using the native heading classes.
 *
 * # Safety
 *
 * `out` must be null or point to a writable `*mut RTuiElementBuilder`.
 */
enum RTuiError rtui_h1(RTuiElementBuilder **out);

/**
 * Create a level-two heading builder using the native heading classes.
 *
 * # Safety
 *
 * `out` must be null or point to a writable `*mut RTuiElementBuilder`.
 */
enum RTuiError rtui_h2(RTuiElementBuilder **out);

/**
 * Create a level-three heading builder using the native heading classes.
 *
 * # Safety
 *
 * `out` must be null or point to a writable `*mut RTuiElementBuilder`.
 */
enum RTuiError rtui_h3(RTuiElementBuilder **out);

/**
 * Append classes through the legacy non-consuming builder name.
 *
 * # Safety
 *
 * `builder` must be null or a live `RTuiElementBuilder` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns. `classes` must be null or a NUL-terminated string that stays
 * readable during the call.
 */
enum RTuiError rtui_element_builder_class(RTuiElementBuilder *builder, const char *classes);

/**
 * Set text through the legacy non-consuming builder name.
 *
 * # Safety
 *
 * `builder` must be null or a live `RTuiElementBuilder` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns. `text` must be null or a NUL-terminated string that stays
 * readable during the call.
 */
enum RTuiError rtui_element_builder_text(RTuiElementBuilder *builder, const char *text);

/**
 * Set the key through the legacy non-consuming builder name.
 *
 * # Safety
 *
 * `builder` must be null or a live `RTuiElementBuilder` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns. `key` must be null or a NUL-terminated string that stays
 * readable during the call.
 */
enum RTuiError rtui_element_builder_key(RTuiElementBuilder *builder, const char *key);

/**
 * Append and consume a live child through the legacy builder name.
 *
 * # Safety
 *
 * `builder` must be null or a live `RTuiElementBuilder` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 *
 * `child` must be null or a live `RTuiElement` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
enum RTuiError rtui_element_builder_child(RTuiElementBuilder *builder,
                                          RTuiElement *child);

/**
 * Append children, consuming every child after successful argument validation.
 *
 * # Safety
 *
 * `builder` must be null or a live `RTuiElementBuilder` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns. `children` must be null or the start of as many `RTuiElement`
 * handles as `children_count` says, each one this library returned and has
 * not destroyed, and no other call may use or destroy them until this one
 * returns.
 */
enum RTuiError rtui_element_builder_children(RTuiElementBuilder *builder,
                                             RTuiElement *const *children,
                                             size_t children_count);

/**
 * Create a caller-owned text element through the legacy name.
 *
 * # Safety
 *
 * `text` must be null or a NUL-terminated string that stays readable during
 * the call. `out` must be null or a pointer slot the caller owns, which this
 * call may write.
 */
enum RTuiError rtui_element_text(const char *text, RTuiElement **out);

/**
 * Create a caller-owned empty element through the legacy name.
 *
 * # Safety
 *
 * `out` must be null or a pointer slot the caller owns, which this call may
 * write.
 */
enum RTuiError rtui_element_empty(RTuiElement **out);

/**
 * Create a native card, consuming its children after argument validation.
 *
 * # Safety
 *
 * `children` must be null or the start of as many `RTuiElement` handles as
 * `children_count` says, each one this library returned and has not
 * destroyed, and no other call may use or destroy them until this one
 * returns. `out` must be null or a pointer slot the caller owns, which this
 * call may write.
 */
enum RTuiError rtui_card(RTuiElement *const *children, size_t children_count, RTuiElement **out);

/**
 * Free a string allocated by native string getters; null is allowed.
 *
 * # Safety
 *
 * `string` must be null or a string this library returned and has not freed,
 * with every byte and the terminating NUL where the library put them: the
 * caller may read it but must not shorten it or write a NUL into it.
 */
void rtui_free_string(char *string);

/**
 * Create a component element
 *
 * # Safety
 *
 * `name` must be null or a NUL-terminated string that stays readable during
 * the call. `out_element` must be null or a pointer slot the caller owns,
 * which this call may write.
 */
enum RTuiError rtui_element_create_component(const char *name, RTuiElement **out_element);

/**
 * Create a text element
 *
 * # Safety
 *
 * `text` must be null or a NUL-terminated string that stays readable during
 * the call. `out_element` must be null or a pointer slot the caller owns,
 * which this call may write.
 */
enum RTuiError rtui_element_create_text(const char *text, RTuiElement **out_element);

/**
 * Create a layout element
 *
 * # Safety
 *
 * `out_element` must be null or a pointer slot the caller owns, which this
 * call may write.
 */
enum RTuiError rtui_element_create_layout(enum RTuiLayoutType layout_type,
                                          RTuiElement **out_element);

/**
 * Create a fragment element
 *
 * # Safety
 *
 * `out_element` must be null or a pointer slot the caller owns, which this
 * call may write.
 */
enum RTuiError rtui_element_create_fragment(RTuiElement **out_element);

/**
 * Create an empty element
 *
 * # Safety
 *
 * `out_element` must be null or a pointer slot the caller owns, which this
 * call may write.
 */
enum RTuiError rtui_element_create_empty(RTuiElement **out_element);

/**
 * Set element key
 *
 * # Safety
 *
 * `element` must be null or a live `RTuiElement` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns. `key` must be null or a NUL-terminated string that stays
 * readable during the call.
 */
enum RTuiError rtui_element_set_key(RTuiElement *element, const char *key);

/**
 * Set element class
 *
 * # Safety
 *
 * `element` must be null or a live `RTuiElement` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns. `class` must be null or a NUL-terminated string that stays
 * readable during the call.
 */
enum RTuiError rtui_element_set_class(RTuiElement *element, const char *class_);

/**
 * Add child element
 *
 * # Safety
 *
 * `parent` must be null or a live `RTuiElement` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 *
 * `child` must be null or a live `RTuiElement` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
enum RTuiError rtui_element_add_child(RTuiElement *parent,
                                      RTuiElement *child);

/**
 * Get element type
 *
 * # Safety
 *
 * `element` must be null or a live `RTuiElement` handle this library
 * returned and has not destroyed, and no call may destroy it until this one
 * returns. `out_type` must be null or a `RTuiElementType` the caller owns,
 * which this call may write.
 */
enum RTuiError rtui_element_get_type(const RTuiElement *element, enum RTuiElementType *out_type);

/**
 * Get element key
 *
 * # Safety
 *
 * `element` must be null or a live `RTuiElement` handle this library
 * returned and has not destroyed, and no call may destroy it until this one
 * returns. `out_key` must be null or a pointer slot the caller owns; this
 * call stores a string there that the caller releases with
 * `rtui_string_free`, keeping every byte and the terminating NUL where this
 * library put them until then.
 */
enum RTuiError rtui_element_get_key(const RTuiElement *element, char **out_key);

/**
 * Get element class
 *
 * # Safety
 *
 * `element` must be null or a live `RTuiElement` handle this library
 * returned and has not destroyed, and no call may destroy it until this one
 * returns. `out_class` must be null or a pointer slot the caller owns; this
 * call stores a string there that the caller releases with
 * `rtui_string_free`, keeping every byte and the terminating NUL where this
 * library put them until then.
 */
enum RTuiError rtui_element_get_class(const RTuiElement *element, char **out_class);

/**
 * Get number of children
 *
 * # Safety
 *
 * `element` must be null or a live `RTuiElement` handle this library
 * returned and has not destroyed, and no call may destroy it until this one
 * returns. `out_count` must be null or a `usize` the caller owns, which this
 * call may write.
 */
enum RTuiError rtui_element_get_child_count(const RTuiElement *element, size_t *out_count);

/**
 * Get child element at index
 *
 * # Safety
 *
 * `element` must be null or a live `RTuiElement` handle this library
 * returned and has not destroyed, and no call may destroy it until this one
 * returns. `out_child` must be null or a pointer slot the caller owns, which
 * this call may write.
 */
enum RTuiError rtui_element_get_child(const RTuiElement *element,
                                      size_t index,
                                      RTuiElement **out_child);

/**
 * Get component name (if element is a component)
 *
 * # Safety
 *
 * `element` must be null or a live `RTuiElement` handle this library
 * returned and has not destroyed, and no call may destroy it until this one
 * returns. `out_name` must be null or a pointer slot the caller owns; this
 * call stores a string there that the caller releases with
 * `rtui_string_free`, keeping every byte and the terminating NUL where this
 * library put them until then.
 */
enum RTuiError rtui_element_get_component_name(const RTuiElement *element, char **out_name);

/**
 * Get text content (if element is text)
 *
 * # Safety
 *
 * `element` must be null or a live `RTuiElement` handle this library
 * returned and has not destroyed, and no call may destroy it until this one
 * returns. `out_text` must be null or a pointer slot the caller owns; this
 * call stores a string there that the caller releases with
 * `rtui_string_free`, keeping every byte and the terminating NUL where this
 * library put them until then.
 */
enum RTuiError rtui_element_get_text_content(const RTuiElement *element, char **out_text);

/**
 * Create an isolated native dialog engine with default limits.
 *
 * # Safety
 *
 * `out_engine` must be null or a pointer slot the caller owns, which this
 * call may write.
 */
enum RTuiError rtui_dialog_engine_create(RTuiDialogEngine **out_engine);

/**
 * Cancel all remaining sessions before releasing this controller.
 *
 * # Safety
 *
 * `engine` must be null or a live `RTuiDialogEngine` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns.
 */
enum RTuiError rtui_dialog_engine_destroy(RTuiDialogEngine *engine);

/**
 * Return the owned normal App host Element for this engine.
 *
 * # Safety
 *
 * `engine` must be null or a live `RTuiDialogEngine` handle this library
 * returned and has not destroyed, and no call may destroy it until this one
 * returns. `out_element` must be null or a pointer slot the caller owns,
 * which this call may write.
 */
enum RTuiError rtui_dialog_engine_element(const RTuiDialogEngine *engine,
                                          RTuiElement **out_element);

/**
 * Open one of the six native families. JSON fields are validated per family.
 *
 * # Safety
 *
 * `engine` must be null or a live `RTuiDialogEngine` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns. `options` must be null or a NUL-terminated string that stays
 * readable during the call. `out_id` must be null or a `u32` the caller
 * owns, which this call may write.
 */
enum RTuiError rtui_dialog_engine_open(RTuiDialogEngine *engine,
                                       const char *options,
                                       uint32_t *out_id);

/**
 * Apply exactly one update: progress, input, wizardData, or zIndex.
 *
 * # Safety
 *
 * `engine` must be null or a live `RTuiDialogEngine` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns. `update` must be null or a NUL-terminated string that stays
 * readable during the call.
 */
enum RTuiError rtui_dialog_engine_update(RTuiDialogEngine *engine, uint32_t id, const char *update);

/**
 * Close an active dialog once, retaining its supplied result and data.
 *
 * # Safety
 *
 * `engine` must be null or a live `RTuiDialogEngine` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns. `result` must be null or a NUL-terminated string that stays
 * readable during the call.
 */
enum RTuiError rtui_dialog_engine_close(RTuiDialogEngine *engine, uint32_t id, const char *result);

/**
 * Consume one native Opened/Closed event. Success with null means empty.
 *
 * # Safety
 *
 * `engine` must be null or a live `RTuiDialogEngine` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns. `out_event` must be null or a pointer slot the caller owns;
 * this call stores a string there that the caller releases with
 * `rtui_string_free`, keeping every byte and the terminating NUL where this
 * library put them until then.
 */
enum RTuiError rtui_dialog_engine_take_event(RTuiDialogEngine *engine, char **out_event);

/**
 * Create an empty editor with the native default viewport and line numbers.
 *
 * # Safety
 *
 * `out_editor` must be null or a pointer slot the caller owns, which this
 * call may write.
 */
enum RTuiError rtui_text_editor_create(RTuiTextEditor **out_editor);

/**
 * Release an editor on its creating thread; null is accepted.
 *
 * # Safety
 *
 * `editor` must be null or a live `RTuiTextEditor` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns.
 */
enum RTuiError rtui_text_editor_destroy(RTuiTextEditor *editor);

/**
 * Replace all text and reset cursor, selection and scroll.
 *
 * # Safety
 *
 * `editor` must be null or a live `RTuiTextEditor` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns. `content` must be null or a NUL-terminated string that stays
 * readable during the call.
 */
enum RTuiError rtui_text_editor_set_content(RTuiTextEditor *editor, const char *content);

/**
 * Return an owned UTF-8 copy; release it with rtui_string_free.
 *
 * # Safety
 *
 * `editor` must be null or a live `RTuiTextEditor` handle this library
 * returned and has not destroyed, and no call may destroy it until this one
 * returns. `out_content` must be null or a pointer slot the caller owns;
 * this call stores a string there that the caller releases with
 * `rtui_string_free`, keeping every byte and the terminating NUL where this
 * library put them until then.
 */
enum RTuiError rtui_text_editor_get_content_owned(const RTuiTextEditor *editor, char **out_content);

/**
 * Insert UTF-8 text, replacing the complete selected graphemes.
 *
 * # Safety
 *
 * `editor` must be null or a live `RTuiTextEditor` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns. `text` must be null or a NUL-terminated string that stays
 * readable during the call.
 */
enum RTuiError rtui_text_editor_insert_text(RTuiTextEditor *editor, const char *text);

/**
 * Delete the selection, or one preceding/following complete grapheme.
 *
 * # Safety
 *
 * `editor` must be null or a live `RTuiTextEditor` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns.
 */
enum RTuiError rtui_text_editor_delete(RTuiTextEditor *editor, bool backward);

/**
 * Movement codes: left/right/up/down=0..3, line start/end=4..5,
 * document start/end=6..7, word backward/forward=8..9. Others are invalid.
 *
 * # Safety
 *
 * `editor` must be null or a live `RTuiTextEditor` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns.
 */
enum RTuiError rtui_text_editor_move(RTuiTextEditor *editor, uint32_t movement, bool select);

/**
 * Set a bounded viewport measured in terminal cells; invalid dimensions do not mutate it.
 *
 * # Safety
 *
 * `editor` must be null or a live `RTuiTextEditor` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns.
 */
enum RTuiError rtui_text_editor_set_size(RTuiTextEditor *editor, uint32_t width, uint32_t height);

/**
 * Show or hide the native line-number gutter.
 *
 * # Safety
 *
 * `editor` must be null or a live `RTuiTextEditor` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns.
 */
enum RTuiError rtui_text_editor_set_show_line_numbers(RTuiTextEditor *editor, bool show);

/**
 * Copy the visible styled lines into an owned Element snapshot.
 *
 * # Safety
 *
 * `editor` must be null or a live `RTuiTextEditor` handle this library
 * returned and has not destroyed, and no call may destroy it until this one
 * returns. `out_element` must be null or a pointer slot the caller owns,
 * which this call may write.
 */
enum RTuiError rtui_text_editor_element(const RTuiTextEditor *editor, RTuiElement **out_element);

/**
 * Create a state controller and retain callback/userdata addresses until successful destruction.
 *
 * # Safety
 *
 * `props` must be null or a NUL-terminated string that stays readable during
 * the call. `state` must be null or a NUL-terminated string that stays
 * readable during the call. `userdata` is kept and handed back to the
 * callbacks unchanged, never dereferenced here; the caller keeps what it
 * points to alive for as long as they can run. `out_component` must be null
 * or a pointer slot the caller owns, which this call may write.
 */
enum RTuiError rtui_foreign_component_create(const char *props,
                                             const char *state,
                                             RTuiForeignRenderCallback render,
                                             RTuiForeignEventCallback event,
                                             RTuiForeignDisposeCallback dispose,
                                             void *userdata,
                                             RTuiForeignComponent **out_component);

/**
 * Disable callbacks and dispose userdata once; recursive or wrong-thread destruction fails.
 *
 * # Safety
 *
 * `component` must be null or a live `RTuiForeignComponent` handle this
 * library returned and has not destroyed, and no other call may use it until
 * this one returns.
 */
enum RTuiError rtui_foreign_component_destroy(RTuiForeignComponent *component);

/**
 * Return an owned typed Element referencing this controller, without invoking callbacks.
 *
 * # Safety
 *
 * `component` must be null or a live `RTuiForeignComponent` handle this
 * library returned and has not destroyed, and no call may destroy it until
 * this one returns. `out_element` must be null or a pointer slot the caller
 * owns, which this call may write.
 */
enum RTuiError rtui_foreign_component_element(const RTuiForeignComponent *component,
                                              RTuiElement **out_element);

/**
 * Invoke render synchronously and return an owned snapshot; recursive entry is rejected.
 *
 * # Safety
 *
 * `component` must be null or a live `RTuiForeignComponent` handle this
 * library returned and has not destroyed, and no call may destroy it until
 * this one returns. `out_element` must be null or a pointer slot the caller
 * owns, which this call may write.
 */
enum RTuiError rtui_foreign_component_render(const RTuiForeignComponent *component,
                                             RTuiElement **out_element);

/**
 * Send a JSON object to the event callback; recursive entry is rejected.
 *
 * # Safety
 *
 * `component` must be null or a live `RTuiForeignComponent` handle this
 * library returned and has not destroyed, and no call may destroy it until
 * this one returns. `event` must be null or a NUL-terminated string that
 * stays readable during the call. `out_handled` must be null or a `bool` the
 * caller owns, which this call may write.
 */
enum RTuiError rtui_foreign_component_dispatch(const RTuiForeignComponent *component,
                                               const char *event,
                                               bool *out_handled);

/**
 * Return an independently owned JSON copy of the current props.
 *
 * # Safety
 *
 * `component` must be null or a live `RTuiForeignComponent` handle this
 * library returned and has not destroyed, and no call may destroy it until
 * this one returns. `out_value` must be null or a pointer slot the caller
 * owns; this call stores a string there that the caller releases with
 * `rtui_string_free`, keeping every byte and the terminating NUL where this
 * library put them until then.
 */
enum RTuiError rtui_foreign_component_get_props(const RTuiForeignComponent *component,
                                                char **out_value);

/**
 * Return an independently owned JSON copy of the current state.
 *
 * # Safety
 *
 * `component` must be null or a live `RTuiForeignComponent` handle this
 * library returned and has not destroyed, and no call may destroy it until
 * this one returns. `out_value` must be null or a pointer slot the caller
 * owns; this call stores a string there that the caller releases with
 * `rtui_string_free`, keeping every byte and the terminating NUL where this
 * library put them until then.
 */
enum RTuiError rtui_foreign_component_get_state(const RTuiForeignComponent *component,
                                                char **out_value);

/**
 * Replace valid JSON props and notify subscribing Apps; callable during a callback.
 *
 * # Safety
 *
 * `component` must be null or a live `RTuiForeignComponent` handle this
 * library returned and has not destroyed, and no call may destroy it until
 * this one returns. `value` must be null or a NUL-terminated string that
 * stays readable during the call.
 */
enum RTuiError rtui_foreign_component_set_props(const RTuiForeignComponent *component,
                                                const char *value);

/**
 * Replace valid JSON state and notify subscribing Apps; callable during a callback.
 *
 * # Safety
 *
 * `component` must be null or a live `RTuiForeignComponent` handle this
 * library returned and has not destroyed, and no call may destroy it until
 * this one returns. `value` must be null or a NUL-terminated string that
 * stays readable during the call.
 */
enum RTuiError rtui_foreign_component_set_state(const RTuiForeignComponent *component,
                                                const char *value);

/**
 * Configure native keyboard focus for a foreign-rendered target.
 *
 * # Safety
 *
 * `element` must be null or a live `RTuiElement` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
enum RTuiError rtui_element_set_focus(RTuiElement *element,
                                      bool focusable,
                                      bool auto_focus);

/**
 * Read the last callback failure, or zero. A successful explicit render clears
 * it. App.run retains its existing generic error code; this preserves the cause.
 *
 * # Safety
 *
 * `component` must be null or a live `RTuiForeignComponent` handle this
 * library returned and has not destroyed, and no call may destroy it until
 * this one returns. `out_code` must be null or a `i32` the caller owns,
 * which this call may write.
 */
enum RTuiError rtui_foreign_component_last_error(const RTuiForeignComponent *component,
                                                 int32_t *out_code);

/**
 * Parse native inline CSS; invalid declarations leave out_style null.
 *
 * # Safety
 *
 * `css` must be null or a NUL-terminated string that stays readable during
 * the call. `out_style` must be null or a pointer slot the caller owns,
 * which this call may write.
 */
enum RTuiError rtui_native_style_create(const char *css, RTuiNativeStyle **out_style);

/**
 * Copy a style into a borrowed Element. Neither handle is consumed.
 *
 * # Safety
 *
 * `style` must be null or a live `RTuiNativeStyle` handle this library
 * returned and has not destroyed, and no call may destroy it until this one
 * returns. `element` must be null or a live `RTuiElement` handle this
 * library returned and has not destroyed, and no other call may use it until
 * this one returns.
 */
enum RTuiError rtui_native_style_apply(const RTuiNativeStyle *style, RTuiElement *element);

/**
 * Release a style handle on its creating thread; null is accepted.
 *
 * # Safety
 *
 * `style` must be null or a live `RTuiNativeStyle` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns.
 */
enum RTuiError rtui_native_style_destroy(RTuiNativeStyle *style);

/**
 * Create a string signal
 *
 * # Safety
 *
 * `initial_value` must be null or a NUL-terminated string that stays
 * readable during the call. `out_signal` must be null or a pointer slot the
 * caller owns, which this call may write.
 */
enum RTuiError rtui_signal_string_create(const char *initial_value, RTuiSignal **out_signal);

/**
 * Create an integer signal
 *
 * # Safety
 *
 * `out_signal` must be null or a pointer slot the caller owns, which this
 * call may write.
 */
enum RTuiError rtui_signal_int_create(int64_t initial_value, RTuiSignal **out_signal);

/**
 * Create a float signal
 *
 * # Safety
 *
 * `out_signal` must be null or a pointer slot the caller owns, which this
 * call may write.
 */
enum RTuiError rtui_signal_float_create(double initial_value, RTuiSignal **out_signal);

/**
 * Create a boolean signal
 *
 * # Safety
 *
 * `out_signal` must be null or a pointer slot the caller owns, which this
 * call may write.
 */
enum RTuiError rtui_signal_bool_create(bool initial_value, RTuiSignal **out_signal);

/**
 * Destroy a signal
 *
 * # Safety
 *
 * `signal` must be null or a live `RTuiSignal` handle that this library returned and has not destroyed, no other call may use it until this one returns, and it is not used again after this call.
 */
void rtui_signal_destroy(RTuiSignal *signal);

/**
 * Get string signal value
 *
 * # Safety
 *
 * `signal` must be null or a live `RTuiSignal` handle that this library returned and has not destroyed, and no call may destroy it until this one returns.
 *
 * `buffer` must be null or point to at least `buffer_size` writable bytes.
 */
enum RTuiError rtui_signal_string_get(const RTuiSignal *signal,
                                      char *buffer,
                                      size_t buffer_size);

/**
 * Set string signal value
 *
 * # Safety
 *
 * `signal` must be null or a live `RTuiSignal` handle this library returned
 * and has not destroyed, and no other call may use it until this one
 * returns. `value` must be null or a NUL-terminated string that stays
 * readable during the call.
 */
enum RTuiError rtui_signal_string_set(RTuiSignal *signal, const char *value);

/**
 * Get integer signal value
 *
 * # Safety
 *
 * `signal` must be null or a live `RTuiSignal` handle this library returned
 * and has not destroyed, and no call may destroy it until this one returns.
 * `out_value` must be null or a `i64` the caller owns, which this call may
 * write.
 */
enum RTuiError rtui_signal_int_get(const RTuiSignal *signal, int64_t *out_value);

/**
 * Set integer signal value
 *
 * # Safety
 *
 * `signal` must be null or a live `RTuiSignal` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
enum RTuiError rtui_signal_int_set(RTuiSignal *signal,
                                   int64_t value);

/**
 * Get float signal value
 *
 * # Safety
 *
 * `signal` must be null or a live `RTuiSignal` handle this library returned
 * and has not destroyed, and no call may destroy it until this one returns.
 * `out_value` must be null or a `f64` the caller owns, which this call may
 * write.
 */
enum RTuiError rtui_signal_float_get(const RTuiSignal *signal, double *out_value);

/**
 * Set float signal value
 *
 * # Safety
 *
 * `signal` must be null or a live `RTuiSignal` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
enum RTuiError rtui_signal_float_set(RTuiSignal *signal,
                                     double value);

/**
 * Get boolean signal value
 *
 * # Safety
 *
 * `signal` must be null or a live `RTuiSignal` handle this library returned
 * and has not destroyed, and no call may destroy it until this one returns.
 * `out_value` must be null or a `bool` the caller owns, which this call may
 * write.
 */
enum RTuiError rtui_signal_bool_get(const RTuiSignal *signal, bool *out_value);

/**
 * Set boolean signal value
 *
 * # Safety
 *
 * `signal` must be null or a live `RTuiSignal` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
enum RTuiError rtui_signal_bool_set(RTuiSignal *signal,
                                    bool value);

/**
 * Create a thread-safe string signal
 *
 * # Safety
 *
 * `initial_value` must be null or a NUL-terminated string that stays
 * readable during the call. `out_signal` must be null or a pointer slot the
 * caller owns, which this call may write.
 */
enum RTuiError rtui_thread_safe_signal_string_create(const char *initial_value,
                                                     RTuiThreadSafeSignal **out_signal);

/**
 * Destroy a thread-safe signal
 *
 * # Safety
 *
 * `signal` must be null or a live `RTuiThreadSafeSignal` handle that this library returned and has not destroyed, no other call may use it until this one returns, and it is not used again after this call.
 */
void rtui_thread_safe_signal_destroy(RTuiThreadSafeSignal *signal);

/**
 * Get thread-safe string signal value
 *
 * # Safety
 *
 * `signal` must be null or a live `RTuiThreadSafeSignal` handle that this library returned and has not destroyed, and no call may destroy it until this one returns.
 *
 * `buffer` must be null or point to at least `buffer_size` writable bytes.
 */
enum RTuiError rtui_thread_safe_signal_string_get(const RTuiThreadSafeSignal *signal,
                                                  char *buffer,
                                                  size_t buffer_size);

/**
 * Set thread-safe string signal value
 *
 * # Safety
 *
 * `signal` must be null or a live `RTuiThreadSafeSignal` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns. `value` must be null or a NUL-terminated string that stays
 * readable during the call.
 */
enum RTuiError rtui_thread_safe_signal_string_set(RTuiThreadSafeSignal *signal, const char *value);

/**
 * Create an effect and run its callback now, then after a signal it read through
 * a C getter changes. Cleanup runs before each later run and at destroy.
 * A change the callback itself makes to a signal it read runs it again when
 * the callback returns, at most 100 times in a row, so a callback that
 * normalizes a value sees its result. Signals and effects must stay on their
 * creating C thread; a setter runs effects synchronously on that thread.
 * `rtui_effect_run` also runs it by hand.
 *
 * # Safety
 *
 * `user_data` is kept and handed back to the callbacks unchanged, never
 * dereferenced here; the caller keeps what it points to alive for as long as
 * they can run. Call only on the thread owning any signals the callback reads.
 * `out_effect` must be null or a pointer slot the caller owns,
 * which this call may write.
 */
enum RTuiError rtui_effect_create(RTuiEffectCallback callback,
                                  RTuiEffectCleanupCallback cleanup,
                                  void *user_data,
                                  RTuiEffect **out_effect);

/**
 * Unregister an effect and run the cleanup returned by its last run once.
 * Call this on its creating C thread.
 *
 * # Safety
 *
 * Call only on the creating thread. `effect` must be null or a live `RTuiEffect` handle that this library returned and has not destroyed, no other call may use it until this one returns, and it is not used again after this call.
 */
void rtui_effect_destroy(RTuiEffect *effect);

/**
 * Run an effect by hand on its creating C thread, after its previous cleanup.
 * This run replaces the signal dependencies with those read through C getters.
 *
 * # Safety
 *
 * Call only on the creating thread. `effect` must be null or a live `RTuiEffect` handle that this library returned and has not destroyed, and no call may destroy it until this one returns.
 */
enum RTuiError rtui_effect_run(const RTuiEffect *effect);

/**
 * Create a new integer signal (improved API)
 */
RTuiSignal *rtui_signal_new_int(int initial_value);

/**
 * Create a new string signal (improved API)
 *
 * # Safety
 *
 * `initial_value` must be null or a NUL-terminated string that stays
 * readable during the call.
 */
RTuiSignal *rtui_signal_new_string(const char *initial_value);

/**
 * Create a new boolean signal (improved API)
 */
RTuiSignal *rtui_signal_new_bool(bool initial_value);

/**
 * Create a new float signal (improved API)
 */
RTuiSignal *rtui_signal_new_float(double initial_value);

/**
 * Get the current value of an integer signal (improved API)
 *
 * # Safety
 *
 * `signal` must be null or a live `RTuiSignal` handle that this library returned and has not destroyed, and no call may destroy it until this one returns.
 */
int rtui_signal_get_int(const RTuiSignal *signal);

/**
 * Set the value of an integer signal (improved API)
 *
 * # Safety
 *
 * `signal` must be null or a live `RTuiSignal` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
enum RTuiError rtui_signal_set_int(RTuiSignal *signal,
                                   int value);

/**
 * Get the current value of a string signal (improved API - returns owned string)
 *
 * The string is released with `rtui_string_free`; until then the caller may
 * read it but must not shorten it or write a NUL into it, since the free
 * relies on the terminator where this library put it.
 *
 * # Safety
 *
 * `signal` must be null or a live `RTuiSignal` handle that this library returned and has not destroyed, and no call may destroy it until this one returns.
 */
char *rtui_signal_get_string_owned(const RTuiSignal *signal);

/**
 * Set the value of a string signal (improved API)
 *
 * # Safety
 *
 * `signal` must be null or a live `RTuiSignal` handle this library returned
 * and has not destroyed, and no other call may use it until this one
 * returns. `value` must be null or a NUL-terminated string that stays
 * readable during the call.
 */
enum RTuiError rtui_signal_set_string_new(RTuiSignal *signal, const char *value);

/**
 * Get the current value of a boolean signal (improved API)
 *
 * # Safety
 *
 * `signal` must be null or a live `RTuiSignal` handle that this library returned and has not destroyed, and no call may destroy it until this one returns.
 */
bool rtui_signal_get_bool_new(const RTuiSignal *signal);

/**
 * Set the value of a boolean signal (improved API)
 *
 * # Safety
 *
 * `signal` must be null or a live `RTuiSignal` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
enum RTuiError rtui_signal_set_bool_new(RTuiSignal *signal,
                                        bool value);

/**
 * Get the current value of a float signal (improved API)
 *
 * # Safety
 *
 * `signal` must be null or a live `RTuiSignal` handle that this library returned and has not destroyed, and no call may destroy it until this one returns.
 */
double rtui_signal_get_float_new(const RTuiSignal *signal);

/**
 * Set the value of a float signal (improved API)
 *
 * # Safety
 *
 * `signal` must be null or a live `RTuiSignal` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
enum RTuiError rtui_signal_set_float_new(RTuiSignal *signal,
                                         double value);

/**
 * Destroy a signal (improved API with proper type safety)
 *
 * # Safety
 *
 * `signal` must be null or a live `RTuiSignal` handle that this library returned and has not destroyed, no other call may use it until this one returns, and it is not used again after this call.
 */
void rtui_signal_destroy_new(RTuiSignal *signal);

/**
 * Free a string returned by rtui_signal_get_string_owned
 *
 * # Safety
 *
 * `string` must be null or a string this library returned and has not freed,
 * with every byte and the terminating NUL where the library put them: the
 * caller may read it but must not shorten it or write a NUL into it.
 */
void rtui_string_free(char *string);

/**
 * Create keyed signal storage on the calling C thread. Repeated calls with a
 * key share its signal. Effects run when created and after signals they read
 * through C getters change, with cleanup before each later run and at destroy.
 * Keep these handles on this thread, where C setters synchronously run effects.
 */
RTuiHooks *rtui_hooks_new(void);

/**
 * Destroy a hooks context
 *
 * # Safety
 *
 * `hooks` must be null or a live `RTuiHooks` handle that this library returned and has not destroyed, no other call may use it until this one returns, and it is not used again after this call.
 */
void rtui_hooks_destroy(RTuiHooks *hooks);

/**
 * Get the shared integer signal for this key, creating it only on the first call.
 * Effects run when created. C getters track their reads; C setters run them again
 * on this creating C thread when a read signal changes.
 * Cleanup runs before each later effect run and when that effect is destroyed.
 *
 * # Safety
 *
 * `hooks` must be null or a live `RTuiHooks` handle this library returned
 * and has not destroyed, and no other call may use it until this one
 * returns. `key` must be null or a NUL-terminated string that stays readable
 * during the call.
 */
RTuiSignal *rtui_use_signal_int(RTuiHooks *hooks, const char *key, int initial);

/**
 * Get the shared string signal for this key, creating it only on the first call.
 * Effects run when created. C getters track their reads; C setters run them again
 * on this creating C thread when a read signal changes.
 * Cleanup runs before each later effect run and when that effect is destroyed.
 *
 * # Safety
 *
 * `hooks` must be null or a live `RTuiHooks` handle this library returned
 * and has not destroyed, and no other call may use it until this one
 * returns. `key` must be null or a NUL-terminated string that stays readable
 * during the call. `initial` must be null or a NUL-terminated string that
 * stays readable during the call.
 */
RTuiSignal *rtui_use_signal_string(RTuiHooks *hooks, const char *key, const char *initial);

/**
 * Get the shared boolean signal for this key, creating it only on the first call.
 * Effects run when created. C getters track their reads; C setters run them again
 * on this creating C thread when a read signal changes.
 * Cleanup runs before each later effect run and when that effect is destroyed.
 *
 * # Safety
 *
 * `hooks` must be null or a live `RTuiHooks` handle this library returned
 * and has not destroyed, and no other call may use it until this one
 * returns. `key` must be null or a NUL-terminated string that stays readable
 * during the call.
 */
RTuiSignal *rtui_use_signal_bool(RTuiHooks *hooks, const char *key, bool initial);

/**
 * Create a new renderer
 *
 * # Safety
 *
 * `out_renderer` must be null or a pointer slot the caller owns, which this
 * call may write.
 */
enum RTuiError rtui_renderer_create(uint16_t width, uint16_t height, RTuiRenderer **out_renderer);

/**
 * Destroy a renderer
 *
 * # Safety
 *
 * `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, no other call may use it until this one returns, and it is not used again after this call.
 */
void rtui_renderer_destroy(RTuiRenderer *renderer);

/**
 * Resize the renderer
 *
 * # Safety
 *
 * `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
enum RTuiError rtui_renderer_resize(RTuiRenderer *renderer,
                                    uint16_t width,
                                    uint16_t height);

/**
 * Clear the renderer with a color
 *
 * # Safety
 *
 * `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
enum RTuiError rtui_renderer_clear(RTuiRenderer *renderer,
                                   uint8_t r,
                                   uint8_t g,
                                   uint8_t b);

/**
 * Control frame rendering
 * @param begin: true to begin frame, false to end frame
 *
 * # Safety
 *
 * `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
enum RTuiError rtui_renderer_frame(RTuiRenderer *renderer,
                                   bool begin);

/**
 * Borrow the drawing surface until renderer shutdown or destruction.
 * The caller must serialize access with renderer operations and must not free it.
 *
 * # Safety
 *
 * `renderer` must be null or a live `RTuiRenderer` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns. `out_surface` must be null or a pointer slot the caller owns,
 * which this call may write.
 */
enum RTuiError rtui_renderer_get_surface(RTuiRenderer *renderer, RTuiSurface **out_surface);

/**
 * Restore the terminal without freeing the handle; call destroy afterward.
 *
 * # Safety
 *
 * `renderer` must be null or a live `RTuiRenderer` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
enum RTuiError rtui_renderer_shutdown(RTuiRenderer *renderer);

/**
 * Create a new surface
 *
 * # Safety
 *
 * `out_surface` must be null or a pointer slot the caller owns, which this
 * call may write.
 */
enum RTuiError rtui_surface_create(uint16_t width, uint16_t height, RTuiSurface **out_surface);

/**
 * Destroy a surface
 *
 * # Safety
 *
 * `surface` must be null or a live `RTuiSurface` handle that this library returned and has not destroyed, no other call may use it until this one returns, and it is not used again after this call.
 */
void rtui_surface_destroy(RTuiSurface *surface);

/**
 * Get surface dimensions
 *
 * # Safety
 *
 * `surface` must be null or a live `RTuiSurface` handle this library
 * returned and has not destroyed, and no call may destroy it until this one
 * returns. `out_dimensions` must be null or a `RTuiDimensions` the caller
 * owns, which this call may write.
 */
enum RTuiError rtui_surface_get_dimensions(const RTuiSurface *surface,
                                           struct RTuiDimensions *out_dimensions);

/**
 * Clear the surface
 *
 * # Safety
 *
 * `surface` must be null or a live `RTuiSurface` handle that this library returned and has not destroyed, and no other call may use it until this one returns.
 */
enum RTuiError rtui_surface_clear(RTuiSurface *surface,
                                  uint8_t r,
                                  uint8_t g,
                                  uint8_t b);

/**
 * Set a cell on the surface
 *
 * # Safety
 *
 * `surface` must be null or a live `RTuiSurface` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns. `cell` must be null or a `RTuiCell` that stays readable
 * during the call.
 */
enum RTuiError rtui_surface_set_cell(RTuiSurface *surface,
                                     uint16_t x,
                                     uint16_t y,
                                     const struct RTuiCell *cell);

/**
 * Get a cell from the surface
 *
 * # Safety
 *
 * `surface` must be null or a live `RTuiSurface` handle this library
 * returned and has not destroyed, and no call may destroy it until this one
 * returns. `out_cell` must be null or a `RTuiCell` the caller owns, which
 * this call may write.
 */
enum RTuiError rtui_surface_get_cell(const RTuiSurface *surface,
                                     uint16_t x,
                                     uint16_t y,
                                     struct RTuiCell *out_cell);

/**
 * Draw text on the surface
 *
 * # Safety
 *
 * `surface` must be null or a live `RTuiSurface` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns. `text` must be null or a NUL-terminated string that stays
 * readable during the call. `fg` must be null or a `RTuiColor` that stays
 * readable during the call. `bg` must be null or a `RTuiColor` that stays
 * readable during the call.
 */
enum RTuiError rtui_surface_draw_text(RTuiSurface *surface,
                                      uint16_t x,
                                      uint16_t y,
                                      const char *text,
                                      const struct RTuiColor *fg,
                                      const struct RTuiColor *bg);

/**
 * Fill a rectangle on the surface
 *
 * # Safety
 *
 * `surface` must be null or a live `RTuiSurface` handle this library
 * returned and has not destroyed, and no other call may use it until this
 * one returns. `rect` must be null or a `RTuiRect` that stays readable
 * during the call. `fg` must be null or a `RTuiColor` that stays readable
 * during the call. `bg` must be null or a `RTuiColor` that stays readable
 * during the call.
 */
enum RTuiError rtui_surface_fill_rect(RTuiSurface *surface,
                                      const struct RTuiRect *rect,
                                      uint32_t ch,
                                      const struct RTuiColor *fg,
                                      const struct RTuiColor *bg);

/**
 * Return a native TextInput with its placeholder and initial value.
 * This constructor takes no callbacks.
 *
 * # Safety
 *
 * `placeholder` must be null or a NUL-terminated string that stays readable
 * during the call. `initial_value` must be null or a NUL-terminated string
 * that stays readable during the call. `out_element` must be null or a
 * pointer slot the caller owns, which this call may write.
 */
enum RTuiError rtui_text_input_create(const char *placeholder,
                                      const char *initial_value,
                                      RTuiElement **out_element);

/**
 * Return a native Checkbox with its label and initial checked state.
 * This constructor takes no callbacks.
 *
 * # Safety
 *
 * `label` must be null or a NUL-terminated string that stays readable during
 * the call. `out_element` must be null or a pointer slot the caller owns,
 * which this call may write.
 */
enum RTuiError rtui_checkbox_create(const char *label,
                                    bool initial_checked,
                                    RTuiElement **out_element);

/**
 * Return a native ProgressBar with its range, current value and label.
 * This constructor takes no callbacks.
 *
 * # Safety
 *
 * `label` must be null or a NUL-terminated string that stays readable during
 * the call. `out_element` must be null or a pointer slot the caller owns,
 * which this call may write.
 */
enum RTuiError rtui_progress_bar_create(double min_value,
                                        double max_value,
                                        double current_value,
                                        const char *label,
                                        RTuiElement **out_element);

/**
 * Return a styled text element for a button.
 * This constructor takes no callbacks.
 *
 * # Safety
 *
 * `text` must be null or a NUL-terminated string that stays readable during
 * the call. `out_element` must be null or a pointer slot the caller owns,
 * which this call may write.
 */
enum RTuiError rtui_button_create(const char *text, RTuiElement **out_element);

/**
 * Create a simple text element
 *
 * # Safety
 *
 * `text` must be null or a NUL-terminated string that stays readable during
 * the call. `out_element` must be null or a pointer slot the caller owns,
 * which this call may write.
 */
enum RTuiError rtui_text_element_create(const char *text, RTuiElement **out_element);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#endif  /* REACTIVE_TUI_NATIVE_H */
