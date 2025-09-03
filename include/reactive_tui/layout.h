/**
 * @file reactive_tui/layout.h
 * @brief Layout system for Reactive-TUI
 * 
 * This header contains CSS-like layout with flexbox/grid support,
 * style builders, and utility class application.
 * 
 * @version 0.1.0
 * @date 2025-09-01
 */

#ifndef REACTIVE_TUI_LAYOUT_H
#define REACTIVE_TUI_LAYOUT_H

#ifdef __cplusplus
extern "C" {
#endif

#include "core.h"

// =============================================================================
// OPAQUE TYPES
// =============================================================================

typedef struct RTuiStyleBuilder RTuiStyleBuilder;
typedef struct RTuiComputedStyle RTuiComputedStyle;

// =============================================================================
// ENUMS
// =============================================================================

/**
 * @brief Display type enumeration
 */
typedef enum {
    RTUI_DISPLAY_NONE = 0,
    RTUI_DISPLAY_FLEX = 1,
    RTUI_DISPLAY_GRID = 2,
    RTUI_DISPLAY_BLOCK = 3,
    RTUI_DISPLAY_INLINE = 4,
    RTUI_DISPLAY_INLINE_BLOCK = 5
} RTuiDisplayType;

/**
 * @brief Flex direction enumeration
 */
typedef enum {
    RTUI_FLEX_DIRECTION_ROW = 0,
    RTUI_FLEX_DIRECTION_ROW_REVERSE = 1,
    RTUI_FLEX_DIRECTION_COLUMN = 2,
    RTUI_FLEX_DIRECTION_COLUMN_REVERSE = 3
} RTuiFlexDirection;

/**
 * @brief Justify content enumeration
 */
typedef enum {
    RTUI_JUSTIFY_CONTENT_FLEX_START = 0,
    RTUI_JUSTIFY_CONTENT_FLEX_END = 1,
    RTUI_JUSTIFY_CONTENT_CENTER = 2,
    RTUI_JUSTIFY_CONTENT_SPACE_BETWEEN = 3,
    RTUI_JUSTIFY_CONTENT_SPACE_AROUND = 4,
    RTUI_JUSTIFY_CONTENT_SPACE_EVENLY = 5
} RTuiJustifyContent;

/**
 * @brief Align items enumeration
 */
typedef enum {
    RTUI_ALIGN_ITEMS_FLEX_START = 0,
    RTUI_ALIGN_ITEMS_FLEX_END = 1,
    RTUI_ALIGN_ITEMS_CENTER = 2,
    RTUI_ALIGN_ITEMS_BASELINE = 3,
    RTUI_ALIGN_ITEMS_STRETCH = 4
} RTuiAlignItems;

/**
 * @brief Dimension unit enumeration
 */
typedef enum {
    RTUI_DIMENSION_UNIT_PIXELS = 0,
    RTUI_DIMENSION_UNIT_PERCENT = 1,
    RTUI_DIMENSION_UNIT_AUTO = 2,
    RTUI_DIMENSION_UNIT_FLEX = 3
} RTuiDimensionUnit;

// =============================================================================
// STRUCTURES
// =============================================================================

/**
 * @brief Dimension value (can be pixels, percentage, or auto)
 */
typedef struct {
    float value;
    RTuiDimensionUnit unit;
} RTuiDimension;

/**
 * @brief RGBA color for styling
 */
typedef struct {
    float r;
    float g;
    float b;
    float a;
} RTuiStyleColor;

// =============================================================================
// STYLE BUILDER FUNCTIONS
// =============================================================================

/**
 * @brief Create a new style builder
 * @param out_builder Output pointer for the created builder
 * @return Error code
 */
RTuiError rtui_style_builder_create(RTuiStyleBuilder** out_builder);

/**
 * @brief Destroy a style builder
 * @param builder Builder to destroy
 */
void rtui_style_builder_destroy(RTuiStyleBuilder* builder);

/**
 * @brief Apply utility CSS classes to style builder
 * @param builder Builder to modify
 * @param classes CSS classes string
 * @param out_builder Output pointer for the modified builder
 * @return Error code
 */
RTuiError rtui_apply_utility_classes(
    RTuiStyleBuilder* builder,
    const char* classes,
    RTuiStyleBuilder** out_builder
);

/**
 * @brief Set display property
 * @param builder Builder to modify
 * @param display Display type
 * @param out_builder Output pointer for the modified builder
 * @return Error code
 */
RTuiError rtui_style_builder_display(
    RTuiStyleBuilder* builder,
    RTuiDisplayType display,
    RTuiStyleBuilder** out_builder
);

/**
 * @brief Set flex direction property
 * @param builder Builder to modify
 * @param direction Flex direction
 * @param out_builder Output pointer for the modified builder
 * @return Error code
 */
RTuiError rtui_style_builder_flex_direction(
    RTuiStyleBuilder* builder,
    RTuiFlexDirection direction,
    RTuiStyleBuilder** out_builder
);

/**
 * @brief Set justify content property
 * @param builder Builder to modify
 * @param justify Justify content
 * @param out_builder Output pointer for the modified builder
 * @return Error code
 */
RTuiError rtui_style_builder_justify_content(
    RTuiStyleBuilder* builder,
    RTuiJustifyContent justify,
    RTuiStyleBuilder** out_builder
);

/**
 * @brief Set align items property
 * @param builder Builder to modify
 * @param align Align items
 * @param out_builder Output pointer for the modified builder
 * @return Error code
 */
RTuiError rtui_style_builder_align_items(
    RTuiStyleBuilder* builder,
    RTuiAlignItems align,
    RTuiStyleBuilder** out_builder
);

/**
 * @brief Set width property
 * @param builder Builder to modify
 * @param width Width dimension
 * @param out_builder Output pointer for the modified builder
 * @return Error code
 */
RTuiError rtui_style_builder_width(
    RTuiStyleBuilder* builder,
    RTuiDimension width,
    RTuiStyleBuilder** out_builder
);

/**
 * @brief Set height property
 * @param builder Builder to modify
 * @param height Height dimension
 * @param out_builder Output pointer for the modified builder
 * @return Error code
 */
RTuiError rtui_style_builder_height(
    RTuiStyleBuilder* builder,
    RTuiDimension height,
    RTuiStyleBuilder** out_builder
);

/**
 * @brief Set padding (all sides)
 * @param builder Builder to modify
 * @param padding Padding value
 * @param out_builder Output pointer for the modified builder
 * @return Error code
 */
RTuiError rtui_style_builder_padding(
    RTuiStyleBuilder* builder,
    float padding,
    RTuiStyleBuilder** out_builder
);

/**
 * @brief Set margin (all sides)
 * @param builder Builder to modify
 * @param margin Margin value
 * @param out_builder Output pointer for the modified builder
 * @return Error code
 */
RTuiError rtui_style_builder_margin(
    RTuiStyleBuilder* builder,
    float margin,
    RTuiStyleBuilder** out_builder
);

/**
 * @brief Set background color
 * @param builder Builder to modify
 * @param color Background color
 * @param out_builder Output pointer for the modified builder
 * @return Error code
 */
RTuiError rtui_style_builder_background_color(
    RTuiStyleBuilder* builder,
    RTuiStyleColor color,
    RTuiStyleBuilder** out_builder
);

/**
 * @brief Set text color
 * @param builder Builder to modify
 * @param color Text color
 * @param out_builder Output pointer for the modified builder
 * @return Error code
 */
RTuiError rtui_style_builder_color(
    RTuiStyleBuilder* builder,
    RTuiStyleColor color,
    RTuiStyleBuilder** out_builder
);

/**
 * @brief Build computed styles from style builder
 * @param builder Builder to build from
 * @param out_style Output pointer for the computed styles
 * @return Error code
 */
RTuiError rtui_style_builder_build(
    RTuiStyleBuilder* builder,
    RTuiComputedStyle** out_style
);

/**
 * @brief Destroy computed styles
 * @param style Computed styles to destroy
 */
void rtui_computed_style_destroy(RTuiComputedStyle* style);

#ifdef __cplusplus
}
#endif

#endif // REACTIVE_TUI_LAYOUT_H
