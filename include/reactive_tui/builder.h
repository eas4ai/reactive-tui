/**
 * @file reactive_tui/builder.h
 * @brief Element builder system for Reactive-TUI
 * 
 * This header contains the declarative UI building API with element builders,
 * CSS styling, and component composition.
 * 
 * @version 0.1.0
 * @date 2025-09-01
 */

#ifndef REACTIVE_TUI_BUILDER_H
#define REACTIVE_TUI_BUILDER_H

#ifdef __cplusplus
extern "C" {
#endif

#include "core.h"

// =============================================================================
// OPAQUE TYPES
// =============================================================================

typedef struct RTuiElementBuilder RTuiElementBuilder;
typedef struct RTuiElement RTuiElement;

// =============================================================================
// CALLBACKS
// =============================================================================

/**
 * @brief Click handler callback function type
 * @param user_data User-provided data
 */
typedef void (*RTuiClickHandler)(void* user_data);

// =============================================================================
// ELEMENT BUILDERS
// =============================================================================

/**
 * @brief Create a div element builder
 * @param out_builder Output pointer for the created builder
 * @return Error code
 */
RTuiError rtui_div(RTuiElementBuilder** out_builder);

/**
 * @brief Create a span element builder
 * @param out_builder Output pointer for the created builder
 * @return Error code
 */
RTuiError rtui_span(RTuiElementBuilder** out_builder);

/**
 * @brief Create a button element builder
 * @param out_builder Output pointer for the created builder
 * @return Error code
 */
RTuiError rtui_button(RTuiElementBuilder** out_builder);

/**
 * @brief Create a paragraph element builder
 * @param out_builder Output pointer for the created builder
 * @return Error code
 */
RTuiError rtui_p(RTuiElementBuilder** out_builder);

/**
 * @brief Create an h1 heading element builder
 * @param out_builder Output pointer for the created builder
 * @return Error code
 */
RTuiError rtui_h1(RTuiElementBuilder** out_builder);

/**
 * @brief Create an h2 heading element builder
 * @param out_builder Output pointer for the created builder
 * @return Error code
 */
RTuiError rtui_h2(RTuiElementBuilder** out_builder);

/**
 * @brief Create an h3 heading element builder
 * @param out_builder Output pointer for the created builder
 * @return Error code
 */
RTuiError rtui_h3(RTuiElementBuilder** out_builder);

// =============================================================================
// BUILDER METHODS
// =============================================================================

/**
 * @brief Destroy an element builder
 * @param builder Builder to destroy
 */
void rtui_element_builder_destroy(RTuiElementBuilder* builder);

/**
 * @brief Add CSS classes to element builder (in-place modification)
 * @param builder Builder to modify
 * @param classes CSS classes string
 * @return Error code
 */
RTuiError rtui_element_builder_class(
    RTuiElementBuilder* builder,
    const char* classes
);

/**
 * @brief Set text content for element builder (in-place modification)
 * @param builder Builder to modify
 * @param text Text content
 * @return Error code
 */
RTuiError rtui_element_builder_text(
    RTuiElementBuilder* builder,
    const char* text
);

/**
 * @brief Set key for element builder (in-place modification)
 * @param builder Builder to modify
 * @param key Element key
 * @return Error code
 */
RTuiError rtui_element_builder_key(
    RTuiElementBuilder* builder,
    const char* key
);

/**
 * @brief Add a child element to builder (in-place modification)
 * @param builder Builder to modify
 * @param child Child element
 * @return Error code
 */
RTuiError rtui_element_builder_child(
    RTuiElementBuilder* builder,
    RTuiElement* child
);

/**
 * @brief Add multiple children to builder (in-place modification)
 * @param builder Builder to modify
 * @param children Array of child elements
 * @param children_count Number of children
 * @return Error code
 */
RTuiError rtui_element_builder_children(
    RTuiElementBuilder* builder,
    RTuiElement* const* children,
    size_t children_count
);

/**
 * @brief Build the final element from builder
 * @param builder Builder to build from
 * @param out_element Output pointer for the created element
 * @return Error code
 */
RTuiError rtui_element_builder_build(
    RTuiElementBuilder* builder,
    RTuiElement** out_element
);

// =============================================================================
// ELEMENT FUNCTIONS
// =============================================================================

/**
 * @brief Destroy an element
 * @param element Element to destroy
 */
void rtui_element_destroy(RTuiElement* element);

/**
 * @brief Create a text element directly
 * @param text Text content
 * @param out_element Output pointer for the created element
 * @return Error code
 */
RTuiError rtui_element_text(const char* text, RTuiElement** out_element);

/**
 * @brief Create an empty element
 * @param out_element Output pointer for the created element
 * @return Error code
 */
RTuiError rtui_element_empty(RTuiElement** out_element);

// =============================================================================
// LAYOUT HELPERS
// =============================================================================

/**
 * @brief Create a primary button with click handler
 * @param text Button text
 * @param handler Click handler callback
 * @param user_data User data for callback
 * @param out_element Output pointer for the created element
 * @return Error code
 */
RTuiError rtui_primary_button(
    const char* text,
    RTuiClickHandler handler,
    void* user_data,
    RTuiElement** out_element
);

/**
 * @brief Create a card layout with children
 * @param children Array of child elements
 * @param children_count Number of children
 * @param out_element Output pointer for the created element
 * @return Error code
 */
RTuiError rtui_card(
    RTuiElement* const* children,
    size_t children_count,
    RTuiElement** out_element
);

#ifdef __cplusplus
}
#endif

#endif // REACTIVE_TUI_BUILDER_H
