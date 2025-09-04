/**
 * @file reactive_tui/integration.h
 * @brief System integration functions for Reactive-TUI
 * 
 * This header contains high-level integration functions that connect
 * different systems (Surface, TextBuffer, Terminal, Renderer) together
 * for seamless end-to-end workflows.
 * 
 * @version 0.1.0
 * @date 2025-09-04
 */

#ifndef REACTIVE_TUI_INTEGRATION_H
#define REACTIVE_TUI_INTEGRATION_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

// Forward declarations
typedef struct RTuiBuffer RTuiBuffer;
typedef struct RTuiTerminal RTuiTerminal;
typedef struct RTuiRenderer RTuiRenderer;
typedef struct RTuiTextBuffer RTuiTextBuffer;

// =============================================================================
// INTEGRATED RENDERING PIPELINES
// =============================================================================

/**
 * @brief Render surface to terminal through renderer
 * 
 * This integrates Surface → Renderer → Terminal in a single operation.
 * Creates a temporary renderer, copies surface data, and renders to terminal.
 * 
 * @param surface Source surface buffer
 * @param terminal Target terminal
 * @return true if successful, false on error
 * 
 * @note This function handles all intermediate resource management automatically
 */
bool renderSurfaceToTerminal(const RTuiBuffer* surface, RTuiTerminal* terminal);

/**
 * @brief Complete rendering pipeline: TextBuffer → Surface → Renderer → Terminal
 *
 * This integrates all systems in a single high-level operation.
 * Creates temporary surface, renders text to it, then renders to terminal.
 * 
 * @param text_buffer Source text buffer
 * @param terminal Target terminal
 * @param x X position for text rendering
 * @param y Y position for text rendering
 * @param width Surface width to create
 * @param height Surface height to create
 * @return true if successful, false on error
 * 
 * @note Automatically manages all intermediate resources (surface creation/cleanup)
 */
bool renderTextToTerminal(const RTuiTextBuffer* text_buffer, RTuiTerminal* terminal,
                          uint32_t x, uint32_t y, uint32_t width, uint32_t height);

/**
 * @brief Integrated renderer with stats collection
 *
 * This combines Renderer → Terminal with automatic stats collection.
 * Renders using the provided renderer and optionally collects performance metrics.
 * 
 * @param renderer Source renderer (must be set up with content)
 * @param terminal Target terminal
 * @param collect_stats Whether to enable detailed performance statistics
 * @return true if successful, false on error
 * 
 * @note When collect_stats is true, performance metrics are logged automatically
 */
bool renderWithStats(RTuiRenderer* renderer, RTuiTerminal* terminal, bool collect_stats);

// =============================================================================
// CONVENIENCE FUNCTIONS
// =============================================================================

/**
 * @brief Get buffer dimensions
 * @param buffer Target buffer
 * @param width Output width
 * @param height Output height
 */
void getBufferDims(const RTuiBuffer* buffer, uint32_t* width, uint32_t* height);

/**
 * @brief Create a complete rendering context
 * @param width Width in characters
 * @param height Height in characters
 * @param terminal Output terminal pointer
 * @param renderer Output renderer pointer
 * @param surface Output surface pointer
 * @return true if all components created successfully
 * 
 * @note Caller must destroy all components when done
 */
bool createRenderingContext(uint32_t width, uint32_t height,
                            RTuiTerminal** terminal, RTuiRenderer** renderer, RTuiBuffer** surface);

/**
 * @brief Destroy a complete rendering context
 * @param terminal Terminal to destroy (can be NULL)
 * @param renderer Renderer to destroy (can be NULL)
 * @param surface Surface to destroy (can be NULL)
 * 
 * @note Safe to call with NULL pointers
 */
void destroyRenderingContext(RTuiTerminal* terminal, RTuiRenderer* renderer, RTuiBuffer* surface);

// =============================================================================
// WORKFLOW EXAMPLES
// =============================================================================

/**
 * @example Basic Text Rendering
 * ```c
 * // Create text buffer and add content
 * RTuiTextBuffer* text = createTextBuffer(1000, 0);
 * float green[] = {0.0f, 1.0f, 0.0f, 1.0f};
 * float black[] = {0.0f, 0.0f, 0.0f, 1.0f};
 * uint32_t attr = 0;
 * textBufferAppendText(text, "Hello World!", 12, green, black, &attr);
 * 
 * // Create terminal
 * RTuiTerminal* terminal = createTerminal();
 * setupTerminal(terminal, true);
 * 
 * // Render directly to terminal (complete pipeline)
 * renderTextToTerminal(text, terminal, 0, 0, 80, 24);
 * 
 * // Cleanup
 * destroyTextBuffer(text);
 * destroyTerminal(terminal);
 * ```
 */

/**
 * @example Surface-based Rendering
 * ```c
 * // Create surface and draw content
 * RTuiBuffer* surface = createOptimizedBuffer(80, 24, false, 0, NULL, 0);
 * float white[] = {1.0f, 1.0f, 1.0f, 1.0f};
 * float blue[] = {0.0f, 0.0f, 1.0f, 1.0f};
 * bufferDrawText(surface, 10, 10, "Surface Text", 12, white, blue, NULL);
 * 
 * // Create terminal and render
 * RTuiTerminal* terminal = createTerminal();
 * setupTerminal(terminal, true);
 * renderSurfaceToTerminal(surface, terminal);
 * 
 * // Cleanup
 * destroyOptimizedBuffer(surface);
 * destroyTerminal(terminal);
 * ```
 */

#ifdef __cplusplus
}
#endif

#endif // REACTIVE_TUI_INTEGRATION_H
