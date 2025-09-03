/**
 * @file reactive_tui/animation.h
 * @brief Animation system for Reactive-TUI
 * 
 * This header contains animation management, easing functions, and
 * property animation for smooth UI transitions.
 * 
 * @version 0.1.0
 * @date 2025-09-01
 */

#ifndef REACTIVE_TUI_ANIMATION_H
#define REACTIVE_TUI_ANIMATION_H

#ifdef __cplusplus
extern "C" {
#endif

#include "core.h"

// =============================================================================
// OPAQUE TYPES
// =============================================================================

typedef struct RTuiAnimationManager RTuiAnimationManager;
typedef struct RTuiAnimation RTuiAnimation;

// =============================================================================
// ANIMATION ENUMS
// =============================================================================

/**
 * @brief Easing function types
 */
typedef enum {
    RTUI_EASING_LINEAR = 0,
    RTUI_EASING_EASE_IN = 1,
    RTUI_EASING_EASE_OUT = 2,
    RTUI_EASING_EASE_IN_OUT = 3,
    RTUI_EASING_BOUNCE = 4,
    RTUI_EASING_ELASTIC = 5,
    RTUI_EASING_BACK = 6,
    RTUI_EASING_EXPO = 7,
    RTUI_EASING_CIRC = 8,
    RTUI_EASING_SINE = 9,
    RTUI_EASING_QUAD = 10,
    RTUI_EASING_CUBIC = 11,
    RTUI_EASING_QUART = 12,
    RTUI_EASING_QUINT = 13,
    RTUI_EASING_SPRING = 14
} RTuiEasingType;

/**
 * @brief Loop behavior types
 */
typedef enum {
    RTUI_LOOP_NONE = 0,
    RTUI_LOOP_INFINITE = 1,
    RTUI_LOOP_COUNT = 2,
    RTUI_LOOP_PING_PONG = 3
} RTuiLoopMode;

/**
 * @brief Animation property types
 */
typedef enum {
    RTUI_PROPERTY_OPACITY = 0,
    RTUI_PROPERTY_TRANSLATE_X = 1,
    RTUI_PROPERTY_TRANSLATE_Y = 2,
    RTUI_PROPERTY_SCALE_X = 3,
    RTUI_PROPERTY_SCALE_Y = 4,
    RTUI_PROPERTY_ROTATION = 5,
    RTUI_PROPERTY_WIDTH = 6,
    RTUI_PROPERTY_HEIGHT = 7
} RTuiAnimatedProperty;

// =============================================================================
// ANIMATION MANAGER
// =============================================================================

/**
 * @brief Create a new animation manager
 * @param out_manager Output pointer for the created manager
 * @return Error code
 */
RTuiError rtui_animation_manager_create(RTuiAnimationManager** out_manager);

/**
 * @brief Destroy an animation manager
 * @param manager Manager to destroy
 */
void rtui_animation_manager_destroy(RTuiAnimationManager* manager);

/**
 * @brief Update the animation manager
 * @param manager Animation manager
 * @return Error code
 */
RTuiError rtui_animation_manager_update(RTuiAnimationManager* manager);

/**
 * @brief Add animation to manager
 * @param manager Animation manager
 * @param animation Animation to add
 * @param out_id Output animation ID string
 * @return Error code
 */
RTuiError rtui_animation_manager_add(
    RTuiAnimationManager* manager,
    RTuiAnimation* animation,
    char** out_id
);

/**
 * @brief Remove animation from manager
 * @param manager Animation manager
 * @param animation_id Animation ID to remove
 * @return Error code
 */
RTuiError rtui_animation_manager_remove(
    RTuiAnimationManager* manager,
    const char* animation_id
);

// =============================================================================
// ANIMATION CREATION
// =============================================================================

/**
 * @brief Create a new animation
 * @param id Animation identifier
 * @param duration_ms Animation duration in milliseconds
 * @param easing Easing function type
 * @param loop_mode Loop mode
 * @param loop_count Number of loops (used with RTUI_LOOP_COUNT)
 * @param out_animation Output pointer for the created animation
 * @return Error code
 */
RTuiError rtui_animation_create(
    const char* id,
    uint32_t duration_ms,
    RTuiEasingType easing,
    RTuiLoopMode loop_mode,
    uint32_t loop_count,
    RTuiAnimation** out_animation
);

/**
 * @brief Create a spring animation
 * @param id Animation identifier
 * @param stiffness Spring stiffness
 * @param damping Spring damping
 * @param mass Spring mass
 * @param out_animation Output pointer for the created animation
 * @return Error code
 */
RTuiError rtui_animation_create_spring(const char* id, float stiffness, float damping, float mass, RTuiAnimation** out_animation);

/**
 * @brief Destroy an animation
 * @param animation Animation to destroy
 */
void rtui_animation_destroy(RTuiAnimation* animation);

// =============================================================================
// ANIMATION CONTROL
// =============================================================================

/**
 * @brief Play an animation
 * @param animation Animation to play
 * @return Error code
 */
RTuiError rtui_animation_play(RTuiAnimation* animation);

/**
 * @brief Stop an animation
 * @param animation Animation to stop
 * @return Error code
 */
RTuiError rtui_animation_stop(RTuiAnimation* animation);

/**
 * @brief Pause an animation
 * @param animation Animation to pause
 * @return Error code
 */
RTuiError rtui_animation_pause(RTuiAnimation* animation);

/**
 * @brief Check if animation is playing
 * @param animation Animation to check
 * @param out_playing Output playing status
 * @return Error code
 */
RTuiError rtui_animation_is_playing(const RTuiAnimation* animation, bool* out_playing);

/**
 * @brief Get animation progress
 * @param animation Animation to check
 * @param out_progress Output progress (0.0 to 1.0)
 * @return Error code
 */
RTuiError rtui_animation_get_progress(const RTuiAnimation* animation, float* out_progress);

// =============================================================================
// ANIMATION PROPERTIES
// =============================================================================

/**
 * @brief Set animation property
 * @param animation Animation to modify
 * @param property Property to animate
 * @param from_value Starting value
 * @param to_value Ending value
 * @return Error code
 */
RTuiError rtui_animation_set_property(
    RTuiAnimation* animation,
    RTuiAnimatedProperty property,
    float from_value,
    float to_value
);

#ifdef __cplusplus
}
#endif

#endif // REACTIVE_TUI_ANIMATION_H
