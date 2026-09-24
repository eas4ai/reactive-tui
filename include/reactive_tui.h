/**
 * @file reactive_tui.h
 * @brief Main header for Reactive-TUI C API
 *
 * This is the main header file that includes all Reactive-TUI modules.
 * Include this file to access the complete Reactive-TUI C API.
 *
 * @version 0.1.0
 * @date 2025-09-01
 *
 * @example
 * ```c
 * #include <reactive_tui.h>
 *
 * int main(void) {
 *     // Create terminal and renderer using modern API
 *     RTuiTerminal* terminal = createTerminal();
 *     if (!terminal) return 1;
 *
 *     setupTerminal(terminal, true); // Use alternate screen
 *
 *     RTuiRenderer* renderer = createRenderer(80, 24);
 *     if (!renderer) {
 *         destroyTerminal(terminal);
 *         return 1;
 *     }
 *
 *     // Create and use a text buffer
 *     RTuiTextBuffer* text = createTextBuffer(1000, 0);
 *     const uint8_t message[] = "Hello, Modern FFI!";
 *     const float green[] = {0.0f, 1.0f, 0.0f, 1.0f};
 *     const float black[] = {0.0f, 0.0f, 0.0f, 1.0f};
 *     const uint8_t attr = 0;
 *     textBufferWriteChunk(text, message, sizeof(message) - 1, green, black, &attr);
 *
 *     // Render using integrated pipeline
 *     renderTextToTerminal(text, terminal, 10, 5, 80, 24);
 *
 *     // Cleanup
 *     destroyTextBuffer(text);
 *     destroyRenderer(renderer, false, 0);
 *     destroyTerminal(terminal);
 *
 *     return 0;
 * }
 * ```
 */

#ifndef REACTIVE_TUI_H
#define REACTIVE_TUI_H

#ifdef __cplusplus
extern "C" {
#endif

// =============================================================================
// CORE MODULES (Currently Implemented)
// =============================================================================

/**
 * @brief Core types, error handling, and library initialization
 */
#include "reactive_tui/core.h"

/**
 * @brief Event system for keyboard, mouse, and other input events
 */
#include "reactive_tui/events.h"

/**
 * @brief Terminal control and management
 */
#include "reactive_tui/terminal.h"

/**
 * @brief Surface and buffer operations for efficient rendering
 */
#include "reactive_tui/surface.h"

/**
 * @brief Text buffer operations and formatted text handling
 */
#include "reactive_tui/text.h"

/**
 * @brief System integration functions for end-to-end workflows
 */
#include "reactive_tui/integration.h"

/**
 * @brief Performance monitoring and debugging functions
 */
#include "reactive_tui/stats.h"

/**
 * @brief Rendering system and frame management
 */
#include "reactive_tui/render.h"

/**
 * @brief Dialog system for modal windows and user interactions
 */
#include "reactive_tui/dialogs.h"

/**
 * @brief Animation system with easing and property animation
 */
#include "reactive_tui/animation.h"

/**
 * @brief Application framework and lifecycle management
 */
#include "reactive_tui/app.h"

/**
 * @brief Element builder API with CSS styling
 */
#include "reactive_tui/builder.h"

/**
 * @brief CSS-like layout with flexbox/grid
 */
#include "reactive_tui/layout.h"

// =============================================================================
// FUTURE MODULES (To Be Implemented)
// =============================================================================

/*
 * The following modules will be added in future releases:
 *
 * #include "reactive_tui/widgets.h"    // Widget library (input, display, layout)
 * #include "reactive_tui/reactive.h"   // Reactive system (hooks, signals)
 * #include "reactive_tui/theme.h"      // Theming and styling system
 * #include "reactive_tui/editor.h"     // Text editor components
 * #include "reactive_tui/syntax.h"     // Syntax highlighting
 * #include "reactive_tui/markdown.h"   // Markdown rendering
 * #include "reactive_tui/platform.h"   // Platform-specific features
 */

// =============================================================================
// CONVENIENCE MACROS
// =============================================================================

/**
 * @brief Check if an operation succeeded
 */
#define RTUI_SUCCESS_CHECK(expr) ((expr) == RTUI_SUCCESS)

/**
 * @brief Check if an operation failed
 */
#define RTUI_FAILED(expr) ((expr) != RTUI_SUCCESS)

/**
 * @brief Return early if operation fails
 */
#define RTUI_RETURN_IF_FAILED(expr) \
    do { \
        RTuiError _err = (expr); \
        if (_err != RTUI_SUCCESS) return _err; \
    } while(0)

/**
 * @brief Create an RGB color
 */
#define RTUI_RGB(r, g, b) ((RTuiColor){(r), (g), (b)})

/**
 * @brief Create an RGBA color
 */
#define RTUI_RGBA(r, g, b, a) ((RTuiColorRGBA){(r), (g), (b), (a)})

/**
 * @brief Create a position
 */
#define RTUI_POS(x, y) ((RTuiPosition){(x), (y)})

/**
 * @brief Create dimensions
 */
#define RTUI_SIZE(w, h) ((RTuiDimensions){(w), (h)})

/**
 * @brief Create a rectangle
 */
#define RTUI_RECT(x, y, w, h) ((RTuiRect){(x), (y), (w), (h)})

/**
 * @brief Create default text attributes (no formatting)
 */
#define RTUI_TEXT_ATTRS_DEFAULT ((RTuiTextAttributes){false, false, false, false, false, false, false})

/**
 * @brief Create bold text attributes
 */
#define RTUI_TEXT_ATTRS_BOLD ((RTuiTextAttributes){true, false, false, false, false, false, false})

/**
 * @brief Create italic text attributes
 */
#define RTUI_TEXT_ATTRS_ITALIC ((RTuiTextAttributes){false, true, false, false, false, false, false})

/**
 * @brief Create underlined text attributes
 */
#define RTUI_TEXT_ATTRS_UNDERLINE ((RTuiTextAttributes){false, false, true, false, false, false, false})

// =============================================================================
// COMMON COLORS
// =============================================================================

/**
 * @brief Common color constants
 */
#define RTUI_COLOR_BLACK   RTUI_RGB(0, 0, 0)
#define RTUI_COLOR_WHITE   RTUI_RGB(255, 255, 255)
#define RTUI_COLOR_RED     RTUI_RGB(255, 0, 0)
#define RTUI_COLOR_GREEN   RTUI_RGB(0, 255, 0)
#define RTUI_COLOR_BLUE    RTUI_RGB(0, 0, 255)
#define RTUI_COLOR_YELLOW  RTUI_RGB(255, 255, 0)
#define RTUI_COLOR_CYAN    RTUI_RGB(0, 255, 255)
#define RTUI_COLOR_MAGENTA RTUI_RGB(255, 0, 255)
#define RTUI_COLOR_GRAY    RTUI_RGB(128, 128, 128)

#ifdef __cplusplus
}
#endif

#endif // REACTIVE_TUI_H
