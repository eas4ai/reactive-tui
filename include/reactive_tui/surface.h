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

// =============================================================================
// MODERN BUFFER API DECLARATIONS
// =============================================================================

/**
 * @brief Create an optimized buffer for drawing operations
 * @param width Width in characters
 * @param height Height in characters
 * @param respect_alpha Whether to respect alpha blending
 * @param width_method Width calculation method (0 = default)
 * @param id_ptr Optional identifier pointer
 * @param id_len Length of identifier
 * @return Pointer to buffer or NULL on failure
 */
RTuiBuffer* createOptimizedBuffer(uint32_t width, uint32_t height, bool respect_alpha,
                                  uint8_t width_method, const uint8_t* id_ptr, size_t id_len);

/**
 * @brief Destroy a buffer and free its resources
 * @param buffer Buffer to destroy
 */
void destroyOptimizedBuffer(RTuiBuffer* buffer);

/**
 * @brief Get buffer width
 * @param buffer Target buffer
 * @return Width in characters
 */
uint32_t getBufferWidth(const RTuiBuffer* buffer);

/**
 * @brief Get buffer height
 * @param buffer Target buffer
 * @return Height in characters
 */
uint32_t getBufferHeight(const RTuiBuffer* buffer);

/**
 * @brief Clear the buffer with a solid color
 * @param buffer Target buffer
 * @param r Red component (0.0-1.0)
 * @param g Green component (0.0-1.0)
 * @param b Blue component (0.0-1.0)
 * @param a Alpha component (0.0-1.0)
 */
void bufferClear(RTuiBuffer* buffer, float r, float g, float b, float a);

/**
 * @brief Draw text to the buffer
 * @param buffer Target buffer
 * @param x X coordinate
 * @param y Y coordinate
 * @param text Text to draw (UTF-8)
 * @param text_len Length of text in bytes
 * @param fg Foreground color (RGBA array)
 * @param bg Background color (RGBA array)
 * @param attr Text attributes
 */
void bufferDrawText(RTuiBuffer* buffer, uint32_t x, uint32_t y, const char* text,
                    size_t text_len, const float* fg, const float* bg, const uint32_t* attr);

/**
 * @brief Write to buffer with direct memory access
 * @param buffer Target buffer
 * @param x X coordinate
 * @param y Y coordinate
 * @param text Text to write
 * @param text_len Length of text
 * @param fg Foreground color array
 * @param bg Background color array
 * @param attr Attributes
 */
void writeToBuffer(RTuiBuffer* buffer, uint32_t x, uint32_t y, const char* text,
                   size_t text_len, const float* fg, const float* bg, const uint32_t* attr);

/**
 * @brief Set a single cell with alpha blending
 * @param buffer Target buffer
 * @param x X coordinate
 * @param y Y coordinate
 * @param ch Character
 * @param fg Foreground color
 * @param bg Background color
 * @param attr Attributes
 */
void bufferSetCellWithAlphaBlending(RTuiBuffer* buffer, uint32_t x, uint32_t y, uint32_t ch,
                                    const float* fg, const float* bg, uint32_t attr);

/**
 * @brief Fill a rectangular region
 * @param buffer Target buffer
 * @param x X coordinate
 * @param y Y coordinate
 * @param width Width of rectangle
 * @param height Height of rectangle
 * @param ch Fill character
 * @param fg Foreground color
 * @param bg Background color
 * @param attr Attributes
 */
void bufferFillRect(RTuiBuffer* buffer, uint32_t x, uint32_t y, uint32_t width, uint32_t height,
                    uint32_t ch, const float* fg, const float* bg, uint32_t attr);

/**
 * @brief Resize buffer
 * @param buffer Target buffer
 * @param width New width
 * @param height New height
 */
void bufferResize(RTuiBuffer* buffer, uint32_t width, uint32_t height);

/**
 * @brief Get/set alpha blending mode
 */
bool bufferGetRespectAlpha(const RTuiBuffer* buffer);
void bufferSetRespectAlpha(RTuiBuffer* buffer, bool respect_alpha);

// =============================================================================
// DIRECT MEMORY ACCESS FUNCTIONS
// =============================================================================

/**
 * @brief Get direct pointer to character buffer
 * @param buffer Target buffer
 * @return Pointer to character array (must be freed with bufferReleaseCharPtr)
 * @warning Caller must call bufferReleaseCharPtr to avoid memory leaks
 */
uint32_t* bufferGetCharPtr(RTuiBuffer* buffer);

/**
 * @brief Get direct pointer to foreground color buffer
 * @param buffer Target buffer
 * @return Pointer to RGBA color array (must be freed with bufferReleaseFgPtr)
 * @warning Caller must call bufferReleaseFgPtr to avoid memory leaks
 */
float* bufferGetFgPtr(RTuiBuffer* buffer);

/**
 * @brief Get direct pointer to background color buffer
 * @param buffer Target buffer
 * @return Pointer to RGBA color array (must be freed with bufferReleaseBgPtr)
 * @warning Caller must call bufferReleaseBgPtr to avoid memory leaks
 */
float* bufferGetBgPtr(RTuiBuffer* buffer);

/**
 * @brief Get direct pointer to attributes buffer
 * @param buffer Target buffer
 * @return Pointer to attributes array (must be freed with bufferReleaseAttrPtr)
 * @warning Caller must call bufferReleaseAttrPtr to avoid memory leaks
 */
uint8_t* bufferGetAttributesPtr(RTuiBuffer* buffer);

// =============================================================================
// MEMORY CLEANUP FUNCTIONS (CRITICAL - MUST BE CALLED)
// =============================================================================

/**
 * @brief Release character buffer pointer
 * @param ptr Pointer returned by bufferGetCharPtr
 * @param length Number of elements (width * height)
 * @warning REQUIRED: Must be called for every bufferGetCharPtr call
 */
void bufferReleaseCharPtr(uint32_t* ptr, size_t length);

/**
 * @brief Release foreground color buffer pointer
 * @param ptr Pointer returned by bufferGetFgPtr
 * @param length Number of elements (width * height * 4)
 * @warning REQUIRED: Must be called for every bufferGetFgPtr call
 */
void bufferReleaseFgPtr(float* ptr, size_t length);

/**
 * @brief Release background color buffer pointer
 * @param ptr Pointer returned by bufferGetBgPtr
 * @param length Number of elements (width * height * 4)
 * @warning REQUIRED: Must be called for every bufferGetBgPtr call
 */
void bufferReleaseBgPtr(float* ptr, size_t length);

/**
 * @brief Release attributes buffer pointer
 * @param ptr Pointer returned by bufferGetAttributesPtr
 * @param length Number of elements (width * height)
 * @warning REQUIRED: Must be called for every bufferGetAttributesPtr call
 */
void bufferReleaseAttrPtr(uint8_t* ptr, size_t length);

#ifdef __cplusplus
}
#endif

#endif // REACTIVE_TUI_SURFACE_H
