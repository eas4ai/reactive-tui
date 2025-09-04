/**
 * @file reactive_tui/core.h
 * @brief Core types and error handling for Reactive-TUI
 * 
 * This header contains fundamental types, error codes, and basic structures
 * used throughout the Reactive-TUI library.
 * 
 * @version 0.1.0
 * @date 2025-09-01
 */

#ifndef REACTIVE_TUI_CORE_H
#define REACTIVE_TUI_CORE_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

// =============================================================================
// VERSION AND ERROR HANDLING
// =============================================================================

/**
 * @brief Library version information
 */
typedef struct {
    uint32_t major;
    uint32_t minor;
    uint32_t patch;
    uint32_t abi_version;
} RTuiVersion;

/**
 * @brief Error codes returned by FFI functions
 */
typedef enum {
    RTUI_SUCCESS = 0,
    RTUI_INVALID_PARAMETER = -1,
    RTUI_NULL_POINTER = -2,
    RTUI_BUFFER_TOO_SMALL = -3,
    RTUI_OUT_OF_MEMORY = -4,
    RTUI_INVALID_UTF8 = -5,
    RTUI_TERMINAL_NOT_AVAILABLE = -6,
    RTUI_NOT_SUPPORTED = -7,
    RTUI_ALREADY_EXISTS = -8,
    RTUI_NOT_FOUND = -9,
    RTUI_INVALID_STATE = -10,
    RTUI_INVALID_POINTER = -11,
    RTUI_INTERNAL_ERROR = -12,
    RTUI_PANIC = -99,
    RTUI_UNKNOWN = -100
} RTuiError;

/**
 * @brief Get the library version
 * @return Version information
 */
RTuiVersion rtui_version(void);

/**
 * @brief Initialize the library (must be called before any other functions)
 * @return Error code
 */
RTuiError rtui_init(void);

/**
 * @brief Cleanup the library (call when done using the library)
 */
void rtui_cleanup(void);

// =============================================================================
// CORE TYPES
// =============================================================================

/**
 * @brief 2D position
 */
typedef struct {
    uint16_t x;
    uint16_t y;
} RTuiPosition;

/**
 * @brief 2D dimensions
 */
typedef struct {
    uint16_t width;
    uint16_t height;
} RTuiDimensions;

/**
 * @brief Rectangle
 */
typedef struct {
    uint16_t x;
    uint16_t y;
    uint16_t width;
    uint16_t height;
} RTuiRect;

/**
 * @brief RGB color
 */
typedef struct {
    uint8_t r;
    uint8_t g;
    uint8_t b;
} RTuiColor;

/**
 * @brief RGBA color
 */
typedef struct {
    uint8_t r;
    uint8_t g;
    uint8_t b;
    uint8_t a;
} RTuiColorRGBA;

/**
 * @brief Text attributes
 */
typedef struct {
    bool bold;
    bool italic;
    bool underline;
    bool strikethrough;
    bool reverse;
    bool blink;
    bool hidden;
} RTuiTextAttributes;

/**
 * @brief Terminal cell
 */
typedef struct {
    uint32_t ch;  // Unicode codepoint
    RTuiColor fg;
    RTuiColor bg;
    RTuiTextAttributes attrs;
} RTuiCell;

// =============================================================================
// MEMORY MANAGEMENT
// =============================================================================

/**
 * @brief Free a string returned by the library
 * @param string String to free
 */
void rtui_free_string(char* string);

// =============================================================================
// MODERN FFI API DECLARATIONS
// =============================================================================

// Forward declarations for opaque types
typedef struct RTuiRenderer RTuiRenderer;
typedef struct RTuiBuffer RTuiBuffer;
typedef struct RTuiTerminal RTuiTerminal;
typedef struct RTuiTextBuffer RTuiTextBuffer;

// =============================================================================
// RENDERER FUNCTIONS
// =============================================================================

/**
 * @brief Create a new renderer
 * @param width Width in characters
 * @param height Height in characters
 * @param use_alternate_screen Whether to use alternate screen buffer
 * @param split_height Split height for dual-buffer mode (0 for single buffer)
 * @return Pointer to renderer or NULL on failure
 */
RTuiRenderer* createRenderer(uint32_t width, uint32_t height, bool use_alternate_screen, uint32_t split_height);

/**
 * @brief Destroy a renderer and free its resources
 * @param renderer Renderer to destroy
 */
void destroyRenderer(RTuiRenderer* renderer);

/**
 * @brief Set the background color for the renderer
 * @param renderer Target renderer
 * @param r Red component (0.0-1.0)
 * @param g Green component (0.0-1.0)
 * @param b Blue component (0.0-1.0)
 * @param a Alpha component (0.0-1.0)
 */
void setBackgroundColor(RTuiRenderer* renderer, float r, float g, float b, float a);

/**
 * @brief Render the current frame to the terminal
 * @param renderer Target renderer
 */
void render(RTuiRenderer* renderer);

/**
 * @brief Resize the renderer
 * @param renderer Target renderer
 * @param width New width in characters
 * @param height New height in characters
 */
void resizeRenderer(RTuiRenderer* renderer, uint32_t width, uint32_t height);

/**
 * @brief Get the renderer's surface for direct drawing
 * @param renderer Target renderer
 * @return Pointer to the renderer's surface buffer
 */
RTuiBuffer* getRendererSurface(RTuiRenderer* renderer);

#ifdef __cplusplus
}
#endif

#endif // REACTIVE_TUI_CORE_H
