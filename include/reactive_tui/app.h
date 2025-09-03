/**
 * @file reactive_tui/app.h
 * @brief Application framework for Reactive-TUI
 * 
 * This header contains the main application framework, including app builders,
 * lifecycle management, and performance monitoring.
 * 
 * @version 0.1.0
 * @date 2025-09-01
 */

#ifndef REACTIVE_TUI_APP_H
#define REACTIVE_TUI_APP_H

#ifdef __cplusplus
extern "C" {
#endif

#include "core.h"
#include "builder.h"

// =============================================================================
// OPAQUE TYPES
// =============================================================================

typedef struct RTuiAppBuilder RTuiAppBuilder;
typedef struct RTuiApp RTuiApp;
typedef struct RTuiRootComponent RTuiRootComponent;

// =============================================================================
// ENUMS
// =============================================================================

/**
 * @brief Performance mode enumeration
 */
typedef enum {
    RTUI_PERFORMANCE_POWER_SAVER = 0,
    RTUI_PERFORMANCE_BALANCED = 1,
    RTUI_PERFORMANCE_HIGH_PERFORMANCE = 2,
    RTUI_PERFORMANCE_ADAPTIVE = 3
} RTuiPerformanceMode;

// =============================================================================
// STRUCTURES
// =============================================================================

/**
 * @brief Performance metrics structure
 */
typedef struct {
    float average_frame_time_ms;
    float min_frame_time_ms;
    float max_frame_time_ms;
    uint32_t dropped_frames;
    uint32_t total_frames;
    uint32_t current_fps;
    uint32_t target_fps;
} RTuiPerformanceMetrics;

// =============================================================================
// CALLBACKS
// =============================================================================

/**
 * @brief Root component callback function type
 * @param user_data User-provided data
 * @return Root element for the application
 */
typedef RTuiElement* (*RTuiRootComponentCallback)(void* user_data);

// =============================================================================
// APP BUILDER FUNCTIONS
// =============================================================================

/**
 * @brief Create a new app builder
 * @param out_builder Output pointer for the created builder
 * @return Error code
 */
RTuiError rtui_app_builder_create(RTuiAppBuilder** out_builder);

/**
 * @brief Destroy an app builder
 * @param builder Builder to destroy
 */
void rtui_app_builder_destroy(RTuiAppBuilder* builder);

/**
 * @brief Set debug mode for app builder (in-place modification)
 * @param builder Builder to modify
 * @param debug Enable debug mode
 * @return Error code
 */
RTuiError rtui_app_builder_debug(
    RTuiAppBuilder* builder,
    bool debug
);

/**
 * @brief Set performance mode for app builder (in-place modification)
 * @param builder Builder to modify
 * @param mode Performance mode
 * @return Error code
 */
RTuiError rtui_app_builder_performance_mode(
    RTuiAppBuilder* builder,
    RTuiPerformanceMode mode
);

/**
 * @brief Set debug backend for app builder
 * @param builder Builder to modify
 * @param width Backend width
 * @param height Backend height
 * @param out_builder Output pointer for the modified builder
 * @return Error code
 */
RTuiError rtui_app_builder_backend_debug(
    RTuiAppBuilder* builder,
    uint16_t width,
    uint16_t height,
    RTuiAppBuilder** out_builder
);

/**
 * @brief Set root component for app builder
 * @param builder Builder to modify
 * @param callback Root component callback
 * @param user_data User data for callback
 * @param out_builder Output pointer for the modified builder
 * @return Error code
 */
RTuiError rtui_app_builder_root_component(
    RTuiAppBuilder* builder,
    RTuiRootComponentCallback callback,
    void* user_data,
    RTuiAppBuilder** out_builder
);

/**
 * @brief Build the app from the builder
 * @param builder Builder to build from
 * @param out_app Output pointer for the created app
 * @return Error code
 */
RTuiError rtui_app_builder_build(
    RTuiAppBuilder* builder,
    RTuiApp** out_app
);

// =============================================================================
// APP FUNCTIONS
// =============================================================================

/**
 * @brief Destroy an app
 * @param app App to destroy
 */
void rtui_app_destroy(RTuiApp* app);

/**
 * @brief Run the app (blocking call)
 * @param app App to run
 * @return Error code
 */
RTuiError rtui_app_run(RTuiApp* app);

/**
 * @brief Stop the app
 * @param app App to stop
 * @return Error code
 */
RTuiError rtui_app_quit(RTuiApp* app);

/**
 * @brief Get app terminal size
 * @param app App instance
 * @param out_dimensions Output dimensions
 * @return Error code
 */
RTuiError rtui_app_get_size(const RTuiApp* app, RTuiDimensions* out_dimensions);

/**
 * @brief Set app performance mode
 * @param app App instance
 * @param mode Performance mode
 * @return Error code
 */
RTuiError rtui_app_set_performance_mode(RTuiApp* app, RTuiPerformanceMode mode);

/**
 * @brief Get current FPS
 * @param app App instance
 * @param out_fps Output FPS value
 * @return Error code
 */
RTuiError rtui_app_get_current_fps(const RTuiApp* app, uint32_t* out_fps);

/**
 * @brief Get performance metrics
 * @param app App instance
 * @param out_metrics Output metrics structure
 * @return Error code
 */
RTuiError rtui_app_get_performance_metrics(
    const RTuiApp* app,
    RTuiPerformanceMetrics* out_metrics
);

#ifdef __cplusplus
}
#endif

#endif // REACTIVE_TUI_APP_H
