/**
 * @file reactive_tui/stats.h
 * @brief Performance monitoring and debugging for Reactive-TUI
 * 
 * This header contains performance statistics, debugging functions,
 * and profiling capabilities for monitoring TUI application performance.
 * 
 * @version 0.1.0
 * @date 2025-09-04
 */

#ifndef REACTIVE_TUI_STATS_H
#define REACTIVE_TUI_STATS_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

// Forward declarations
typedef struct RTuiRenderer RTuiRenderer;

// =============================================================================
// ENUMERATIONS
// =============================================================================

/**
 * @brief Debug overlay corner positions
 */
typedef enum {
    RTUI_OVERLAY_TOP_LEFT = 0,     ///< Top-left corner
    RTUI_OVERLAY_TOP_RIGHT = 1,    ///< Top-right corner
    RTUI_OVERLAY_BOTTOM_LEFT = 2,  ///< Bottom-left corner
    RTUI_OVERLAY_BOTTOM_RIGHT = 3  ///< Bottom-right corner
} RTuiDebugOverlayCorner;

/**
 * @brief Log levels for debugging
 */
typedef enum {
    RTUI_LOG_ERROR = 0,  ///< Error messages
    RTUI_LOG_WARN = 1,   ///< Warning messages
    RTUI_LOG_INFO = 2,   ///< Informational messages
    RTUI_LOG_DEBUG = 3,  ///< Debug messages
    RTUI_LOG_TRACE = 4   ///< Trace messages
} RTuiLogLevel;

// =============================================================================
// CALLBACK TYPES
// =============================================================================

/**
 * @brief Log callback function type
 * @param level Log level
 * @param message Log message (null-terminated)
 */
typedef void (*RTuiLogCallback)(RTuiLogLevel level, const char* message);

// =============================================================================
// PERFORMANCE MONITORING
// =============================================================================

/**
 * @brief Update performance statistics
 * @param renderer Target renderer
 * @param time Current time
 * @param fps Current FPS
 * @param frame_callback_time Frame callback execution time
 */
void updateStats(RTuiRenderer* renderer, double time, uint32_t fps, double frame_callback_time);

/**
 * @brief Update memory usage statistics
 * @param renderer Target renderer
 * @param heap_used Heap memory used in bytes
 * @param heap_total Total heap memory in bytes
 * @param stack_used Stack memory used in bytes
 */
void updateMemoryStats(RTuiRenderer* renderer, uint64_t heap_used, uint64_t heap_total, uint64_t stack_used);

/**
 * @brief Get frame statistics
 * @param renderer Target renderer
 * @param fps Output FPS value
 * @param frame_time Output frame time in milliseconds
 * @param memory_usage Output memory usage in bytes
 * @return true if statistics are available
 */
bool getFrameStats(RTuiRenderer* renderer, float* fps, float* frame_time, uint64_t* memory_usage);

/**
 * @brief Reset performance counters
 * @param renderer Target renderer
 */
void resetPerformanceCounters(RTuiRenderer* renderer);

// =============================================================================
// DEBUG OVERLAY
// =============================================================================

/**
 * @brief Set render offset for debugging
 * @param renderer Target renderer
 * @param x X offset
 * @param y Y offset
 */
void setRenderOffset(RTuiRenderer* renderer, int32_t x, int32_t y);

/**
 * @brief Set debug overlay visibility and position
 * @param renderer Target renderer
 * @param enabled Whether overlay is enabled
 * @param corner Corner position for overlay
 */
void setDebugOverlay(RTuiRenderer* renderer, bool enabled, RTuiDebugOverlayCorner corner);

// =============================================================================
// HIT TESTING AND DEBUGGING
// =============================================================================

/**
 * @brief Add point to hit testing grid
 * @param renderer Target renderer
 * @param x X coordinate
 * @param y Y coordinate
 * @param hit_type Type of hit (user-defined)
 */
void addToHitGrid(RTuiRenderer* renderer, uint32_t x, uint32_t y, uint32_t hit_type);

/**
 * @brief Check if point was hit
 * @param renderer Target renderer
 * @param x X coordinate
 * @param y Y coordinate
 * @return Hit type or 0 if no hit
 */
uint32_t checkHit(RTuiRenderer* renderer, uint32_t x, uint32_t y);

/**
 * @brief Dump hit grid to debug output
 * @param renderer Target renderer
 */
void dumpHitGrid(RTuiRenderer* renderer);

// =============================================================================
// BUFFER DEBUGGING
// =============================================================================

/**
 * @brief Dump all buffers to debug output
 * @param renderer Target renderer
 */
void dumpBuffers(RTuiRenderer* renderer);

/**
 * @brief Dump stdout buffer contents
 * @param renderer Target renderer
 */
void dumpStdoutBuffer(RTuiRenderer* renderer);

// =============================================================================
// LOGGING SYSTEM
// =============================================================================

/**
 * @brief Set log callback function
 * @param callback Callback function for log messages
 */
void setLogCallback(RTuiLogCallback callback);

/**
 * @brief Log a message (internal use)
 * @param level Log level
 * @param message Message to log
 */
void logMessage(RTuiLogLevel level, const char* message);

// =============================================================================
// PROFILING
// =============================================================================

/**
 * @brief Start profiling session
 * @param renderer Target renderer
 * @param detailed Whether to collect detailed statistics
 */
void startProfiling(RTuiRenderer* renderer, bool detailed);

/**
 * @brief Stop profiling session
 * @param renderer Target renderer
 */
void stopProfiling(RTuiRenderer* renderer);

// =============================================================================
// CONVENIENCE MACROS
// =============================================================================

/**
 * @brief Log an error message
 */
#define RTUI_LOG_ERROR_MSG(msg) logMessage(RTUI_LOG_ERROR, msg)

/**
 * @brief Log a warning message
 */
#define RTUI_LOG_WARN_MSG(msg) logMessage(RTUI_LOG_WARN, msg)

/**
 * @brief Log an info message
 */
#define RTUI_LOG_INFO_MSG(msg) logMessage(RTUI_LOG_INFO, msg)

/**
 * @brief Log a debug message
 */
#define RTUI_LOG_DEBUG_MSG(msg) logMessage(RTUI_LOG_DEBUG, msg)

/**
 * @brief Log a trace message
 */
#define RTUI_LOG_TRACE_MSG(msg) logMessage(RTUI_LOG_TRACE, msg)

#ifdef __cplusplus
}
#endif

#endif // REACTIVE_TUI_STATS_H
