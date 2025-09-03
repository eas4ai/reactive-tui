/**
 * @file reactive_tui/events.h
 * @brief Event system for Reactive-TUI
 * 
 * This header contains event types, structures, and handling functions
 * for keyboard, mouse, and other input events.
 * 
 * @version 0.1.0
 * @date 2025-09-01
 */

#ifndef REACTIVE_TUI_EVENTS_H
#define REACTIVE_TUI_EVENTS_H

#ifdef __cplusplus
extern "C" {
#endif

#include "core.h"

// =============================================================================
// EVENT TYPES
// =============================================================================

/**
 * @brief Event type enumeration
 */
typedef enum {
    RTUI_EVENT_KEY = 0,
    RTUI_EVENT_MOUSE = 1,
    RTUI_EVENT_RESIZE = 2,
    RTUI_EVENT_FOCUS = 3,
    RTUI_EVENT_PASTE = 4
} RTuiEventType;

/**
 * @brief Key event structure
 */
typedef struct {
    uint32_t key_code;
    uint8_t modifiers;  // Bit flags: 1=Shift, 2=Ctrl, 4=Alt, 8=Meta
} RTuiKeyEvent;

/**
 * @brief Mouse event structure
 */
typedef struct {
    uint16_t x;
    uint16_t y;
    uint8_t button;
    uint8_t modifiers;
    uint8_t event_type;  // 0=Press, 1=Release, 2=Move, 3=ScrollUp, 4=ScrollDown
} RTuiMouseEvent;

/**
 * @brief Resize event structure
 */
typedef struct {
    uint16_t width;
    uint16_t height;
} RTuiResizeEvent;

/**
 * @brief Event data union
 */
typedef union {
    RTuiKeyEvent key;
    RTuiMouseEvent mouse;
    RTuiResizeEvent resize;
} RTuiEventData;

/**
 * @brief Event structure
 */
typedef struct {
    RTuiEventType event_type;
    RTuiEventData data;
} RTuiEvent;

// =============================================================================
// EVENT HANDLING
// =============================================================================

/**
 * @brief Event handler callback function
 * @param event The event that occurred
 * @param user_data User-provided data
 * @return true if event was handled, false to continue propagation
 */
typedef bool (*RTuiEventHandler)(const RTuiEvent* event, void* user_data);

/**
 * @brief Poll for terminal events
 * @param timeout_ms Timeout in milliseconds (0 for no timeout)
 * @param out_event Output event structure
 * @return Error code
 */
RTuiError rtui_terminal_poll_event(uint32_t timeout_ms, RTuiEvent* out_event);

#ifdef __cplusplus
}
#endif

#endif // REACTIVE_TUI_EVENTS_H
