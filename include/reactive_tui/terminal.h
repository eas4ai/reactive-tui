/**
 * @file reactive_tui/terminal.h
 * @brief Terminal control and management for Reactive-TUI
 * 
 * This header contains terminal initialization, configuration, and
 * low-level terminal operations.
 * 
 * @version 0.1.0
 * @date 2025-09-01
 */

#ifndef REACTIVE_TUI_TERMINAL_H
#define REACTIVE_TUI_TERMINAL_H

#ifdef __cplusplus
extern "C" {
#endif

#include "core.h"
#include "events.h"

// =============================================================================
// OPAQUE TYPES
// =============================================================================

typedef struct RTuiTerminal RTuiTerminal;

// =============================================================================
// TERMINAL MANAGEMENT
// =============================================================================

/**
 * @brief Create a new terminal
 * @param out_terminal Output pointer for the created terminal
 * @return Error code
 */
RTuiError rtui_terminal_create(RTuiTerminal** out_terminal);

/**
 * @brief Destroy a terminal
 * @param terminal Terminal to destroy
 */
void rtui_terminal_destroy(RTuiTerminal* terminal);

/**
 * @brief Get terminal dimensions
 * @param terminal Terminal instance
 * @param out_dimensions Output dimensions
 * @return Error code
 */
RTuiError rtui_terminal_get_dimensions(const RTuiTerminal* terminal, RTuiDimensions* out_dimensions);

/**
 * @brief Enter raw mode (modern terminal mode)
 * @param terminal Terminal to configure
 * @return Error code
 */
RTuiError rtui_terminal_enter_raw_mode(RTuiTerminal* terminal);

/**
 * @brief Exit raw mode
 * @param terminal Terminal to restore
 * @return Error code
 */
RTuiError rtui_terminal_exit_raw_mode(RTuiTerminal* terminal);

/**
 * @brief Control synchronized updates
 * @param terminal Terminal instance
 * @param begin true to begin sync, false to end sync
 * @return Error code
 */
RTuiError rtui_terminal_sync(RTuiTerminal* terminal, bool begin);

/**
 * @brief Clear the terminal screen
 * @param terminal Terminal instance
 * @return Error code
 */
RTuiError rtui_terminal_clear(RTuiTerminal* terminal);

/**
 * @brief Set cursor position
 * @param terminal Terminal instance
 * @param x X coordinate
 * @param y Y coordinate
 * @return Error code
 */
RTuiError rtui_terminal_set_cursor(RTuiTerminal* terminal, uint16_t x, uint16_t y);

/**
 * @brief Show or hide cursor
 * @param terminal Terminal instance
 * @param visible true to show cursor, false to hide
 * @return Error code
 */
RTuiError rtui_terminal_set_cursor_visible(RTuiTerminal* terminal, bool visible);

/**
 * @brief Write text to terminal at current cursor position
 * @param terminal Terminal instance
 * @param text Text to write (UTF-8)
 * @return Error code
 */
RTuiError rtui_terminal_write(RTuiTerminal* terminal, const char* text);

/**
 * @brief Flush terminal output
 * @param terminal Terminal instance
 * @return Error code
 */
RTuiError rtui_terminal_flush(RTuiTerminal* terminal);

// =============================================================================
// MODERN TERMINAL API DECLARATIONS
// =============================================================================

/**
 * @brief Cursor style enumeration
 */
typedef enum {
    RTUI_CURSOR_BLOCK = 0,      ///< Block cursor
    RTUI_CURSOR_UNDERLINE = 1,  ///< Underline cursor
    RTUI_CURSOR_BAR = 2         ///< Bar cursor
} RTuiCursorStyle;

/**
 * @brief Terminal capabilities structure
 */
typedef struct {
    bool colors_256;        ///< Supports 256 colors
    bool colors_truecolor;  ///< Supports true color (24-bit)
    bool mouse_support;     ///< Supports mouse events
    bool kitty_keyboard;    ///< Supports Kitty keyboard protocol
    bool sixel_support;     ///< Supports Sixel graphics
    bool unicode_support;   ///< Supports Unicode
    uint16_t width;         ///< Terminal width in characters
    uint16_t height;        ///< Terminal height in characters
} RTuiCapabilities;

/**
 * @brief Create a new terminal instance
 * @return Pointer to terminal or NULL on failure
 */
RTuiTerminal* createTerminal(void);

/**
 * @brief Destroy a terminal and free its resources
 * @param terminal Terminal to destroy
 */
void destroyTerminal(RTuiTerminal* terminal);

/**
 * @brief Setup terminal for TUI mode
 * @param terminal Target terminal
 * @param use_alternate_screen Whether to use alternate screen buffer
 */
void setupTerminal(RTuiTerminal* terminal, bool use_alternate_screen);

/**
 * @brief Clear the terminal screen
 * @param terminal Target terminal
 */
void clearTerminal(RTuiTerminal* terminal);

/**
 * @brief Get terminal capabilities
 * @param terminal Target terminal
 * @param capabilities Output capabilities structure
 * @return true if successful
 */
bool getTerminalCapabilities(RTuiTerminal* terminal, RTuiCapabilities* capabilities);

/**
 * @brief Process capability response from terminal
 * @param terminal Target terminal
 * @param response Response string from terminal
 * @param response_len Length of response
 */
void processCapabilityResponse(RTuiTerminal* terminal, const char* response, size_t response_len);

/**
 * @brief Set cursor position
 * @param terminal Target terminal
 * @param x X coordinate (0-based)
 * @param y Y coordinate (0-based)
 */
void setCursorPosition(RTuiTerminal* terminal, uint16_t x, uint16_t y);

/**
 * @brief Set cursor style
 * @param terminal Target terminal
 * @param style Cursor style
 */
void setCursorStyle(RTuiTerminal* terminal, RTuiCursorStyle style);

/**
 * @brief Set cursor color
 * @param terminal Target terminal
 * @param r Red component (0-255)
 * @param g Green component (0-255)
 * @param b Blue component (0-255)
 */
void setCursorColor(RTuiTerminal* terminal, uint8_t r, uint8_t g, uint8_t b);

/**
 * @brief Set terminal title
 * @param terminal Target terminal
 * @param title Title string
 * @param title_len Length of title
 */
void setTerminalTitle(RTuiTerminal* terminal, const char* title, size_t title_len);

/**
 * @brief Enable mouse support
 * @param terminal Target terminal
 */
void enableMouse(RTuiTerminal* terminal);

/**
 * @brief Disable mouse support
 * @param terminal Target terminal
 */
void disableMouse(RTuiTerminal* terminal);

/**
 * @brief Enable Kitty keyboard protocol
 * @param terminal Target terminal
 */
void enableKittyKeyboard(RTuiTerminal* terminal);

/**
 * @brief Disable Kitty keyboard protocol
 * @param terminal Target terminal
 */
void disableKittyKeyboard(RTuiTerminal* terminal);

#ifdef __cplusplus
}
#endif

#endif // REACTIVE_TUI_TERMINAL_H
