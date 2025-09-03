/**
 * @file reactive_tui/render.h
 * @brief Rendering system for Reactive-TUI
 * 
 * This header contains the rendering engine, frame management, and
 * display operations for efficient terminal output.
 * 
 * @version 0.1.0
 * @date 2025-09-01
 */

#ifndef REACTIVE_TUI_RENDER_H
#define REACTIVE_TUI_RENDER_H

#ifdef __cplusplus
extern "C" {
#endif

#include "core.h"
#include "terminal.h"
#include "surface.h"

// =============================================================================
// OPAQUE TYPES
// =============================================================================

typedef struct RTuiRenderer RTuiRenderer;

// =============================================================================
// RENDERER MANAGEMENT
// =============================================================================

/**
 * @brief Create a new renderer
 * @param terminal Terminal to render to
 * @param out_renderer Output pointer for the created renderer
 * @return Error code
 */
RTuiError rtui_renderer_create(RTuiTerminal* terminal, RTuiRenderer** out_renderer);

/**
 * @brief Destroy a renderer
 * @param renderer Renderer to destroy
 */
void rtui_renderer_destroy(RTuiRenderer* renderer);

/**
 * @brief Get renderer dimensions
 * @param renderer Renderer instance
 * @param out_dimensions Output dimensions
 * @return Error code
 */
RTuiError rtui_renderer_get_dimensions(const RTuiRenderer* renderer, RTuiDimensions* out_dimensions);

/**
 * @brief Resize the renderer
 * @param renderer Renderer to resize
 * @param width New width
 * @param height New height
 * @return Error code
 */
RTuiError rtui_renderer_resize(RTuiRenderer* renderer, uint16_t width, uint16_t height);

// =============================================================================
// FRAME OPERATIONS
// =============================================================================

/**
 * @brief Begin a new frame
 * @param renderer Renderer instance
 * @return Error code
 */
RTuiError rtui_renderer_begin_frame(RTuiRenderer* renderer);

/**
 * @brief End the current frame and present to terminal
 * @param renderer Renderer instance
 * @return Error code
 */
RTuiError rtui_renderer_end_frame(RTuiRenderer* renderer);

/**
 * @brief Clear the current frame
 * @param renderer Renderer instance
 * @return Error code
 */
RTuiError rtui_renderer_clear(RTuiRenderer* renderer);

/**
 * @brief Clear the current frame with specific color
 * @param renderer Renderer instance
 * @param color Background color
 * @return Error code
 */
RTuiError rtui_renderer_clear_with_color(RTuiRenderer* renderer, RTuiColor color);

// =============================================================================
// DRAWING OPERATIONS
// =============================================================================

/**
 * @brief Draw a surface to the renderer
 * @param renderer Renderer instance
 * @param surface Surface to draw
 * @param x X offset
 * @param y Y offset
 * @return Error code
 */
RTuiError rtui_renderer_draw_surface(RTuiRenderer* renderer, const RTuiSurface* surface, uint16_t x, uint16_t y);

/**
 * @brief Draw a portion of a surface to the renderer
 * @param renderer Renderer instance
 * @param surface Surface to draw from
 * @param src_rect Source rectangle
 * @param dest_x Destination X coordinate
 * @param dest_y Destination Y coordinate
 * @return Error code
 */
RTuiError rtui_renderer_draw_surface_rect(
    RTuiRenderer* renderer,
    const RTuiSurface* surface,
    RTuiRect src_rect,
    uint16_t dest_x,
    uint16_t dest_y
);

/**
 * @brief Draw text directly to the renderer
 * @param renderer Renderer instance
 * @param x X coordinate
 * @param y Y coordinate
 * @param text Text to draw (UTF-8)
 * @param fg Foreground color
 * @param bg Background color
 * @param attrs Text attributes
 * @return Error code
 */
RTuiError rtui_renderer_draw_text(
    RTuiRenderer* renderer,
    uint16_t x,
    uint16_t y,
    const char* text,
    RTuiColor fg,
    RTuiColor bg,
    RTuiTextAttributes attrs
);

/**
 * @brief Draw a filled rectangle
 * @param renderer Renderer instance
 * @param rect Rectangle to fill
 * @param cell Cell to fill with
 * @return Error code
 */
RTuiError rtui_renderer_fill_rect(RTuiRenderer* renderer, RTuiRect rect, const RTuiCell* cell);

/**
 * @brief Draw a rectangle border
 * @param renderer Renderer instance
 * @param rect Rectangle bounds
 * @param cell Cell for border
 * @return Error code
 */
RTuiError rtui_renderer_draw_rect(RTuiRenderer* renderer, RTuiRect rect, const RTuiCell* cell);

#ifdef __cplusplus
}
#endif

#endif // REACTIVE_TUI_RENDER_H
