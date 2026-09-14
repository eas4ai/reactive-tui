/* Legacy type names and constants. Native call signatures live in native.h.
 * Types without a native counterpart remain C-only conveniences; see
 * include/README.md and bindings/typescript/MIGRATION.md before passing data to a native function.
 */
#ifndef REACTIVE_TUI_COMPAT_H
#define REACTIVE_TUI_COMPAT_H
#include "native.h"

typedef struct {
    uint16_t x;
    uint16_t y;
} RTuiPosition;

typedef struct {
    uint8_t r;
    uint8_t g;
    uint8_t b;
    uint8_t a;
} RTuiColorRGBA;

typedef bool (*RTuiEventHandler)(const RTuiEvent* event, void* user_data);

typedef enum {
    RTUI_CURSOR_BLOCK = 0,      ///< Block cursor
    RTUI_CURSOR_UNDERLINE = 1,  ///< Underline cursor
    RTUI_CURSOR_BAR = 2         ///< Bar cursor
} RTuiCursorStyle;

typedef void (*RTuiClickHandler)(void* user_data);

typedef struct RTuiRootComponent RTuiRootComponent;

typedef struct RTuiDialogEngine RTuiDialogEngine;

typedef struct RTuiDialog RTuiDialog;

typedef struct RTuiStyleBuilder RTuiStyleBuilder;

typedef struct RTuiComputedStyle RTuiComputedStyle;

typedef enum {
    RTUI_DISPLAY_NONE = 0,
    RTUI_DISPLAY_FLEX = 1,
    RTUI_DISPLAY_GRID = 2,
    RTUI_DISPLAY_BLOCK = 3,
    RTUI_DISPLAY_INLINE = 4,
    RTUI_DISPLAY_INLINE_BLOCK = 5
} RTuiDisplayType;

typedef enum {
    RTUI_FLEX_DIRECTION_ROW = 0,
    RTUI_FLEX_DIRECTION_ROW_REVERSE = 1,
    RTUI_FLEX_DIRECTION_COLUMN = 2,
    RTUI_FLEX_DIRECTION_COLUMN_REVERSE = 3
} RTuiFlexDirection;

typedef enum {
    RTUI_JUSTIFY_CONTENT_FLEX_START = 0,
    RTUI_JUSTIFY_CONTENT_FLEX_END = 1,
    RTUI_JUSTIFY_CONTENT_CENTER = 2,
    RTUI_JUSTIFY_CONTENT_SPACE_BETWEEN = 3,
    RTUI_JUSTIFY_CONTENT_SPACE_AROUND = 4,
    RTUI_JUSTIFY_CONTENT_SPACE_EVENLY = 5
} RTuiJustifyContent;

typedef enum {
    RTUI_ALIGN_ITEMS_FLEX_START = 0,
    RTUI_ALIGN_ITEMS_FLEX_END = 1,
    RTUI_ALIGN_ITEMS_CENTER = 2,
    RTUI_ALIGN_ITEMS_BASELINE = 3,
    RTUI_ALIGN_ITEMS_STRETCH = 4
} RTuiAlignItems;

typedef enum {
    RTUI_DIMENSION_UNIT_PIXELS = 0,
    RTUI_DIMENSION_UNIT_PERCENT = 1,
    RTUI_DIMENSION_UNIT_AUTO = 2,
    RTUI_DIMENSION_UNIT_FLEX = 3
} RTuiDimensionUnit;

typedef struct {
    float value;
    RTuiDimensionUnit unit;
} RTuiDimension;

typedef struct {
    float r;
    float g;
    float b;
    float a;
} RTuiStyleColor;

typedef struct {
    uint32_t start_index;    ///< Starting character index
    uint32_t length;         ///< Length of line in characters
    uint32_t visual_width;   ///< Visual width accounting for Unicode
} RTuiLineInfo;

typedef enum {
    RTUI_OVERLAY_TOP_LEFT = 0,     ///< Top-left corner
    RTUI_OVERLAY_TOP_RIGHT = 1,    ///< Top-right corner
    RTUI_OVERLAY_BOTTOM_LEFT = 2,  ///< Bottom-left corner
    RTUI_OVERLAY_BOTTOM_RIGHT = 3  ///< Bottom-right corner
} RTuiDebugOverlayCorner;

typedef enum {
    RTUI_LOG_ERROR = 0,  ///< Error messages
    RTUI_LOG_WARN = 1,   ///< Warning messages
    RTUI_LOG_INFO = 2,   ///< Informational messages
    RTUI_LOG_DEBUG = 3,  ///< Debug messages
    RTUI_LOG_TRACE = 4   ///< Trace messages
} RTuiLogLevel;

typedef LogCallback RTuiLogCallback;

#define RTUI_SUCCESS R_TUI_ERROR_SUCCESS
#define RTUI_INVALID_PARAMETER R_TUI_ERROR_INVALID_PARAMETER
#define RTUI_NULL_POINTER R_TUI_ERROR_NULL_POINTER
#define RTUI_BUFFER_TOO_SMALL R_TUI_ERROR_BUFFER_TOO_SMALL
#define RTUI_OUT_OF_MEMORY R_TUI_ERROR_OUT_OF_MEMORY
#define RTUI_INVALID_UTF8 R_TUI_ERROR_INVALID_UTF8
#define RTUI_TERMINAL_NOT_AVAILABLE R_TUI_ERROR_TERMINAL_NOT_AVAILABLE
#define RTUI_NOT_SUPPORTED R_TUI_ERROR_NOT_SUPPORTED
#define RTUI_ALREADY_EXISTS R_TUI_ERROR_ALREADY_EXISTS
#define RTUI_NOT_FOUND R_TUI_ERROR_NOT_FOUND
#define RTUI_INVALID_STATE R_TUI_ERROR_INVALID_STATE
#define RTUI_INVALID_POINTER R_TUI_ERROR_INVALID_POINTER
#define RTUI_INTERNAL_ERROR R_TUI_ERROR_INTERNAL_ERROR
#define RTUI_PANIC R_TUI_ERROR_PANIC
#define RTUI_UNKNOWN R_TUI_ERROR_UNKNOWN
#define RTUI_EVENT_KEY R_TUI_EVENT_TYPE_KEY
#define RTUI_EVENT_MOUSE R_TUI_EVENT_TYPE_MOUSE
#define RTUI_EVENT_RESIZE R_TUI_EVENT_TYPE_RESIZE
#define RTUI_EVENT_FOCUS R_TUI_EVENT_TYPE_FOCUS
#define RTUI_EVENT_PASTE R_TUI_EVENT_TYPE_PASTE
#define RTUI_PERFORMANCE_POWER_SAVER R_TUI_PERFORMANCE_MODE_POWER_SAVE
#define RTUI_PERFORMANCE_BALANCED R_TUI_PERFORMANCE_MODE_BALANCED
#define RTUI_PERFORMANCE_HIGH_PERFORMANCE R_TUI_PERFORMANCE_MODE_PERFORMANCE
#define RTUI_EASING_LINEAR R_TUI_EASING_TYPE_LINEAR
#define RTUI_EASING_EASE_IN R_TUI_EASING_TYPE_EASE_IN
#define RTUI_EASING_EASE_OUT R_TUI_EASING_TYPE_EASE_OUT
#define RTUI_EASING_EASE_IN_OUT R_TUI_EASING_TYPE_EASE_IN_OUT
#define RTUI_EASING_BOUNCE R_TUI_EASING_TYPE_BOUNCE
#define RTUI_EASING_ELASTIC R_TUI_EASING_TYPE_ELASTIC
#define RTUI_EASING_BACK R_TUI_EASING_TYPE_BACK
#define RTUI_EASING_EXPO R_TUI_EASING_TYPE_EXPO
#define RTUI_EASING_CIRC R_TUI_EASING_TYPE_CIRC
#define RTUI_EASING_SINE R_TUI_EASING_TYPE_SINE
#define RTUI_EASING_QUAD R_TUI_EASING_TYPE_QUAD
#define RTUI_EASING_CUBIC R_TUI_EASING_TYPE_CUBIC
#define RTUI_EASING_QUART R_TUI_EASING_TYPE_QUART
#define RTUI_EASING_QUINT R_TUI_EASING_TYPE_QUINT
#define RTUI_EASING_SPRING R_TUI_EASING_TYPE_SPRING
#define RTUI_LOOP_NONE R_TUI_LOOP_MODE_NONE
#define RTUI_LOOP_INFINITE R_TUI_LOOP_MODE_INFINITE
#define RTUI_LOOP_COUNT R_TUI_LOOP_MODE_COUNT
#define RTUI_LOOP_PING_PONG R_TUI_LOOP_MODE_PING_PONG
#define RTUI_PROPERTY_OPACITY R_TUI_ANIMATED_PROPERTY_OPACITY
#define RTUI_PROPERTY_TRANSLATE_X R_TUI_ANIMATED_PROPERTY_TRANSLATE_X
#define RTUI_PROPERTY_TRANSLATE_Y R_TUI_ANIMATED_PROPERTY_TRANSLATE_Y
#define RTUI_PROPERTY_SCALE_X R_TUI_ANIMATED_PROPERTY_SCALE_X
#define RTUI_PROPERTY_SCALE_Y R_TUI_ANIMATED_PROPERTY_SCALE_Y
#define RTUI_PROPERTY_ROTATION R_TUI_ANIMATED_PROPERTY_ROTATION
#define RTUI_PROPERTY_WIDTH R_TUI_ANIMATED_PROPERTY_WIDTH
#define RTUI_PROPERTY_HEIGHT R_TUI_ANIMATED_PROPERTY_HEIGHT

#endif
