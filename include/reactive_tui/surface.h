/**
 * @file reactive_tui/surface.h
 * @brief Surface and buffer operations for Reactive-TUI
 * 
 * This header contains surface management, cell manipulation, and
 * buffer operations for efficient terminal rendering.
 * 
 * @version 0.1.0
 * @date 2025-09-01
 */

#ifndef REACTIVE_TUI_SURFACE_H
#define REACTIVE_TUI_SURFACE_H

#ifdef __cplusplus
extern "C" {
#endif

#include "core.h"

// =============================================================================
// OPAQUE TYPES
// =============================================================================

typedef struct RTuiSurface RTuiSurface;

// =============================================================================
// SURFACE MANAGEMENT
// =============================================================================

/**
 * @brief Create a new surface
 * @param width Surface width in characters
 * @param height Surface height in characters
 * @param out_surface Output pointer for the created surface
 * @return Error code
 */
RTuiError rtui_surface_create(uint16_t width, uint16_t height, RTuiSurface** out_surface);

/**
 * @brief Destroy a surface
 * @param surface Surface to destroy
 */
void rtui_surface_destroy(RTuiSurface* surface);

/**
 * @brief Get surface dimensions
 * @param surface Surface instance
 * @param out_dimensions Output dimensions
 * @return Error code
 */
RTuiError rtui_surface_get_dimensions(const RTuiSurface* surface, RTuiDimensions* out_dimensions);

/**
 * @brief Resize a surface
 * @param surface Surface to resize
 * @param width New width
 * @param height New height
 * @return Error code
 */
RTuiError rtui_surface_resize(RTuiSurface* surface, uint16_t width, uint16_t height);

/**
 * @brief Clear the surface with a specific cell
 * @param surface Surface to clear
 * @param cell Cell to fill with
 * @return Error code
 */
RTuiError rtui_surface_clear(RTuiSurface* surface, const RTuiCell* cell);

/**
 * @brief Clear the surface with default empty cell
 * @param surface Surface to clear
 * @return Error code
 */
RTuiError rtui_surface_clear_default(RTuiSurface* surface);

// =============================================================================
// CELL OPERATIONS
// =============================================================================

/**
 * @brief Set a cell at specific coordinates
 * @param surface Surface to modify
 * @param x X coordinate
 * @param y Y coordinate
 * @param cell Cell data to set
 * @return Error code
 */
RTuiError rtui_surface_set_cell(RTuiSurface* surface, uint16_t x, uint16_t y, const RTuiCell* cell);

/**
 * @brief Get a cell at specific coordinates
 * @param surface Surface to read from
 * @param x X coordinate
 * @param y Y coordinate
 * @param out_cell Output cell data
 * @return Error code
 */
RTuiError rtui_surface_get_cell(const RTuiSurface* surface, uint16_t x, uint16_t y, RTuiCell* out_cell);

/**
 * @brief Set text at specific coordinates with attributes
 * @param surface Surface to modify
 * @param x X coordinate
 * @param y Y coordinate
 * @param text Text to set (UTF-8)
 * @param fg Foreground color
 * @param bg Background color
 * @param attrs Text attributes
 * @return Error code
 */
RTuiError rtui_surface_set_text(
    RTuiSurface* surface,
    uint16_t x,
    uint16_t y,
    const char* text,
    RTuiColor fg,
    RTuiColor bg,
    RTuiTextAttributes attrs
);

/**
 * @brief Fill a rectangular area with a specific cell
 * @param surface Surface to modify
 * @param rect Rectangle to fill
 * @param cell Cell to fill with
 * @return Error code
 */
RTuiError rtui_surface_fill_rect(RTuiSurface* surface, RTuiRect rect, const RTuiCell* cell);

/**
 * @brief Copy a rectangular area from one surface to another
 * @param dest_surface Destination surface
 * @param dest_x Destination X coordinate
 * @param dest_y Destination Y coordinate
 * @param src_surface Source surface
 * @param src_rect Source rectangle
 * @return Error code
 */
RTuiError rtui_surface_copy_rect(
    RTuiSurface* dest_surface,
    uint16_t dest_x,
    uint16_t dest_y,
    const RTuiSurface* src_surface,
    RTuiRect src_rect
);

#ifdef __cplusplus
}
#endif

#endif // REACTIVE_TUI_SURFACE_H
