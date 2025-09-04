/**
 * @file reactive_tui/text.h
 * @brief Text buffer operations for Reactive-TUI
 * 
 * This header contains text buffer management functions for handling
 * formatted text with colors and attributes.
 * 
 * @version 0.1.0
 * @date 2025-09-04
 */

#ifndef REACTIVE_TUI_TEXT_H
#define REACTIVE_TUI_TEXT_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

// Forward declarations
typedef struct RTuiTextBuffer RTuiTextBuffer;
typedef struct RTuiBuffer RTuiBuffer;
typedef struct RTuiTerminal RTuiTerminal;

// =============================================================================
// TEXT BUFFER STRUCTURES
// =============================================================================

/**
 * @brief Line information structure
 */
typedef struct {
    uint32_t start_index;    ///< Starting character index
    uint32_t length;         ///< Length of line in characters
    uint32_t visual_width;   ///< Visual width accounting for Unicode
} RTuiLineInfo;

// =============================================================================
// TEXT BUFFER MANAGEMENT
// =============================================================================

/**
 * @brief Create a new text buffer
 * @param length Initial capacity in characters
 * @param width_method Width calculation method (0 = default)
 * @return Pointer to text buffer or NULL on failure
 */
RTuiTextBuffer* createTextBuffer(uint32_t length, uint8_t width_method);

/**
 * @brief Destroy a text buffer and free its resources
 * @param buffer Text buffer to destroy
 */
void destroyTextBuffer(RTuiTextBuffer* buffer);

/**
 * @brief Get direct pointer to character data
 * @param buffer Target text buffer
 * @return Pointer to character array
 */
const uint32_t* textBufferGetCharPtr(const RTuiTextBuffer* buffer);

/**
 * @brief Get current length of text buffer
 * @param buffer Target text buffer
 * @return Number of characters currently stored
 */
uint32_t textBufferGetLength(const RTuiTextBuffer* buffer);

/**
 * @brief Get capacity of text buffer
 * @param buffer Target text buffer
 * @return Maximum number of characters that can be stored
 */
uint32_t textBufferGetCapacity(const RTuiTextBuffer* buffer);

/**
 * @brief Resize text buffer capacity
 * @param buffer Target text buffer
 * @param new_capacity New capacity in characters
 * @return true if successful
 */
bool textBufferResize(RTuiTextBuffer* buffer, uint32_t new_capacity);

/**
 * @brief Reset text buffer (clear all content)
 * @param buffer Target text buffer
 */
void textBufferReset(RTuiTextBuffer* buffer);

// =============================================================================
// TEXT WRITING OPERATIONS
// =============================================================================

/**
 * @brief Write a chunk of text with formatting
 * @param buffer Target text buffer
 * @param text_bytes Text data (UTF-8)
 * @param text_len Length of text in bytes
 * @param fg Foreground color (RGBA array) or NULL for default
 * @param bg Background color (RGBA array) or NULL for default
 * @param attr Attributes or NULL for default
 * @return Number of characters written
 */
uint32_t textBufferWriteChunk(RTuiTextBuffer* buffer, const uint8_t* text_bytes, uint32_t text_len,
                              const float* fg, const float* bg, const uint8_t* attr);

/**
 * @brief Append text to buffer with formatting
 * @param buffer Target text buffer
 * @param text Text to append
 * @param text_len Length of text
 * @param fg Foreground color
 * @param bg Background color
 * @param attr Attributes
 * @return Number of characters appended
 */
uint32_t textBufferAppendText(RTuiTextBuffer* buffer, const char* text, size_t text_len,
                              const float* fg, const float* bg, const uint32_t* attr);

// =============================================================================
// TEXT SELECTION
// =============================================================================

/**
 * @brief Set text selection range
 * @param buffer Target text buffer
 * @param start Start index
 * @param end End index
 * @param fg Selection foreground color (RGBA)
 * @param bg Selection background color (RGBA)
 */
void textBufferSetSelection(RTuiTextBuffer* buffer, uint32_t start, uint32_t end,
                            const float* fg, const float* bg);

/**
 * @brief Reset text selection
 * @param buffer Target text buffer
 */
void textBufferResetSelection(RTuiTextBuffer* buffer);

/**
 * @brief Get selection information
 * @param buffer Target text buffer
 * @param start Output start index
 * @param end Output end index
 * @return true if selection is active
 */
bool textBufferGetSelectionInfo(const RTuiTextBuffer* buffer, uint32_t* start, uint32_t* end);

// =============================================================================
// DEFAULT FORMATTING
// =============================================================================

/**
 * @brief Set default foreground color
 * @param buffer Target text buffer
 * @param fg RGBA color array
 */
void textBufferSetDefaultFg(RTuiTextBuffer* buffer, const float* fg);

/**
 * @brief Set default background color
 * @param buffer Target text buffer
 * @param bg RGBA color array
 */
void textBufferSetDefaultBg(RTuiTextBuffer* buffer, const float* bg);

/**
 * @brief Set default text attributes
 * @param buffer Target text buffer
 * @param attr Attributes value
 */
void textBufferSetDefaultAttributes(RTuiTextBuffer* buffer, uint8_t attr);

/**
 * @brief Reset all default formatting to library defaults
 * @param buffer Target text buffer
 */
void textBufferResetDefaults(RTuiTextBuffer* buffer);

// =============================================================================
// TEXT RENDERING FUNCTIONS
// =============================================================================

/**
 * @brief Render text buffer to a surface
 * @param buffer Source text buffer
 * @param surface Target surface buffer
 * @param x X offset in surface
 * @param y Y offset in surface
 * @param max_width Maximum width to render
 * @return Number of characters rendered
 */
uint32_t renderTextBufferToSurface(const RTuiTextBuffer* buffer, RTuiBuffer* surface,
                                   uint32_t x, uint32_t y, uint32_t max_width);

/**
 * @brief Render text buffer to a renderer
 * @param buffer Source text buffer
 * @param renderer Target renderer
 * @param x X offset in renderer
 * @param y Y offset in renderer
 * @param max_width Maximum width to render
 * @return Number of characters rendered
 */
uint32_t renderTextBufferToRenderer(const RTuiTextBuffer* buffer, RTuiRenderer* renderer,
                                    uint32_t x, uint32_t y, uint32_t max_width);

/**
 * @brief Direct text buffer to terminal rendering (high-level convenience)
 * @param buffer Source text buffer
 * @param terminal Target terminal
 * @param x X position
 * @param y Y position
 * @param width Surface width
 * @param height Surface height
 * @return true if successful
 */
bool renderTextBufferDirect(const RTuiTextBuffer* buffer, RTuiTerminal* terminal,
                            uint32_t x, uint32_t y, uint32_t width, uint32_t height);

#ifdef __cplusplus
}
#endif

#endif // REACTIVE_TUI_TEXT_H
