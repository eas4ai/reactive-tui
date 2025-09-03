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

#ifdef __cplusplus
}
#endif

#endif // REACTIVE_TUI_TERMINAL_H
