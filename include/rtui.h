/*
 * Reactive TUI - C API Header
 * 
 * This header provides a stable C ABI for using reactive-tui from C and other languages.
 * 
 * Version: 0.1.0
 * ABI Version: 1
 * 
 * Threading: Most functions are NOT thread-safe unless explicitly documented.
 * Error Handling: All functions returning RTuiError should check for RTUI_SUCCESS.
 * Memory: Destroy functions must be called to free resources.
 */

#ifndef RTUI_H
#define RTUI_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ========== Version Information ========== */

typedef struct {
    uint32_t major;
    uint32_t minor;
    uint32_t patch;
    uint32_t abi_version;
} RTuiVersion;

/* ========== Error Codes ========== */

typedef enum {
    RTUI_SUCCESS = 0,
    RTUI_ERROR_INVALID_PARAMETER = -1,
    RTUI_ERROR_NULL_POINTER = -2,
    RTUI_ERROR_BUFFER_TOO_SMALL = -3,
    RTUI_ERROR_OUT_OF_MEMORY = -4,
    RTUI_ERROR_INVALID_UTF8 = -5,
    RTUI_ERROR_TERMINAL_NOT_AVAILABLE = -6,
    RTUI_ERROR_NOT_SUPPORTED = -7,
    RTUI_ERROR_ALREADY_EXISTS = -8,
    RTUI_ERROR_NOT_FOUND = -9,
    RTUI_ERROR_INVALID_STATE = -10,
    RTUI_ERROR_PANIC = -99,
    RTUI_ERROR_UNKNOWN = -100
} RTuiError;

/* ========== Opaque Handles ========== */

typedef struct RTuiTerminal RTuiTerminal;
typedef struct RTuiSurface RTuiSurface;
typedef struct RTuiRenderer RTuiRenderer;

/* ========== Basic Types ========== */

typedef struct {
    uint8_t r;
    uint8_t g;
    uint8_t b;
} RTuiColor;

typedef struct {
    uint16_t width;
    uint16_t height;
} RTuiDimensions;

typedef struct {
    uint16_t x;
    uint16_t y;
} RTuiPosition;

typedef struct {
    uint16_t x;
    uint16_t y;
    uint16_t width;
    uint16_t height;
} RTuiRect;

typedef struct {
    bool bold;
    bool italic;
    bool underline;
    bool strikethrough;
    bool reverse;
    bool blink;
    bool hidden;
} RTuiTextAttributes;

typedef struct {
    uint32_t ch;  /* Unicode codepoint */
    RTuiColor fg;
    RTuiColor bg;
    RTuiTextAttributes attrs;
} RTuiCell;

/* ========== Event Types ========== */

typedef enum {
    RTUI_EVENT_KEY = 0,
    RTUI_EVENT_MOUSE = 1,
    RTUI_EVENT_RESIZE = 2,
    RTUI_EVENT_FOCUS = 3,
    RTUI_EVENT_PASTE = 4
} RTuiEventType;

/* Key modifiers (bit flags) */
#define RTUI_MOD_SHIFT 0x01
#define RTUI_MOD_CTRL  0x02
#define RTUI_MOD_ALT   0x04
#define RTUI_MOD_META  0x08

typedef struct {
    uint32_t key_code;
    uint8_t modifiers;
} RTuiKeyEvent;

/* Mouse event types */
#define RTUI_MOUSE_PRESS      0
#define RTUI_MOUSE_RELEASE    1
#define RTUI_MOUSE_MOVE       2
#define RTUI_MOUSE_SCROLL_UP  3
#define RTUI_MOUSE_SCROLL_DOWN 4

typedef struct {
    uint16_t x;
    uint16_t y;
    uint8_t button;
    uint8_t modifiers;
    uint8_t event_type;
} RTuiMouseEvent;

typedef struct {
    uint16_t width;
    uint16_t height;
} RTuiResizeEvent;

typedef union {
    RTuiKeyEvent key;
    RTuiMouseEvent mouse;
    RTuiResizeEvent resize;
} RTuiEventData;

typedef struct {
    RTuiEventType event_type;
    RTuiEventData data;
} RTuiEvent;

/* ========== Callbacks ========== */

typedef void (*RTuiEventCallback)(const RTuiEvent* event, void* user_data);
typedef void (*RTuiRenderCallback)(RTuiSurface* surface, void* user_data);

/* ========== Library Functions ========== */

/* Version and initialization */
RTuiVersion rtui_version(void);
RTuiError rtui_init(void);
void rtui_cleanup(void);

/* ========== Terminal Functions ========== */

RTuiError rtui_terminal_create(RTuiTerminal** out_terminal);
void rtui_terminal_destroy(RTuiTerminal* terminal);
RTuiError rtui_terminal_get_dimensions(const RTuiTerminal* terminal, RTuiDimensions* out_dimensions);
RTuiError rtui_terminal_enter_raw_mode(RTuiTerminal* terminal);
RTuiError rtui_terminal_exit_raw_mode(RTuiTerminal* terminal);
RTuiError rtui_terminal_sync(RTuiTerminal* terminal, bool begin);  /* true=begin, false=end */
RTuiError rtui_terminal_poll_event(uint32_t timeout_ms, RTuiEvent* out_event);

/* ========== Surface Functions ========== */

RTuiError rtui_surface_create(uint16_t width, uint16_t height, RTuiSurface** out_surface);
void rtui_surface_destroy(RTuiSurface* surface);
RTuiError rtui_surface_get_dimensions(const RTuiSurface* surface, RTuiDimensions* out_dimensions);
RTuiError rtui_surface_clear(RTuiSurface* surface, uint8_t r, uint8_t g, uint8_t b);
RTuiError rtui_surface_set_cell(RTuiSurface* surface, uint16_t x, uint16_t y, const RTuiCell* cell);
RTuiError rtui_surface_get_cell(const RTuiSurface* surface, uint16_t x, uint16_t y, RTuiCell* out_cell);
RTuiError rtui_surface_draw_text(RTuiSurface* surface, uint16_t x, uint16_t y, const char* text, 
                                  const RTuiColor* fg, const RTuiColor* bg);
RTuiError rtui_surface_fill_rect(RTuiSurface* surface, const RTuiRect* rect, uint32_t ch,
                                  const RTuiColor* fg, const RTuiColor* bg);

/* ========== Renderer Functions ========== */

RTuiError rtui_renderer_create(uint16_t width, uint16_t height, RTuiRenderer** out_renderer);
void rtui_renderer_destroy(RTuiRenderer* renderer);
RTuiError rtui_renderer_frame(RTuiRenderer* renderer, bool begin);  /* true=begin, false=end */
RTuiError rtui_renderer_get_surface(RTuiRenderer* renderer, RTuiSurface** out_surface);
RTuiError rtui_renderer_resize(RTuiRenderer* renderer, uint16_t width, uint16_t height);
RTuiError rtui_renderer_clear(RTuiRenderer* renderer, uint8_t r, uint8_t g, uint8_t b);
RTuiError rtui_renderer_shutdown(RTuiRenderer* renderer);

#ifdef __cplusplus
}
#endif

#endif /* RTUI_H */