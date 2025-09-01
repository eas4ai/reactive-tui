/**
 * @file reactive_tui.h
 * @brief Reactive-TUI C API Header
 * 
 * This header provides a complete C API for the Reactive-TUI library,
 * enabling integration with other programming languages and C/C++ applications.
 * 
 * @version 0.1.0
 * @date 2025-09-01
 */

#ifndef REACTIVE_TUI_H
#define REACTIVE_TUI_H

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
    RTUI_PANIC = -99,
    RTUI_UNKNOWN = -100
} RTuiError;

/**
 * @brief Get the library version
 * @return Version information
 */
RTuiVersion rtui_version(void);

// =============================================================================
// CORE TYPES
// =============================================================================

/**
 * @brief 2D position
 */
typedef struct {
    int32_t x;
    int32_t y;
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
    int32_t x;
    int32_t y;
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
 * @brief Terminal cell
 */
typedef struct {
    char character[8];  // UTF-8 character (up to 4 bytes + null terminator)
    RTuiColor fg;
    RTuiColor bg;
    uint8_t attributes;  // Bold, italic, underline flags
} RTuiCell;

// =============================================================================
// OPAQUE HANDLES
// =============================================================================

typedef struct RTuiTerminal RTuiTerminal;
typedef struct RTuiRenderer RTuiRenderer;
typedef struct RTuiSurface RTuiSurface;
typedef struct RTuiElement RTuiElement;
typedef struct RTuiProps RTuiProps;
typedef struct RTuiVNode RTuiVNode;
typedef struct RTuiPatchList RTuiPatchList;
typedef struct RTuiDialogEngine RTuiDialogEngine;
typedef struct RTuiDialog RTuiDialog;
typedef struct RTuiAnimation RTuiAnimation;
typedef struct RTuiAnimationManager RTuiAnimationManager;
typedef struct RTuiKeyframeAnimation RTuiKeyframeAnimation;
typedef struct RTuiEventRouter RTuiEventRouter;
typedef struct RTuiEvent RTuiEvent;
typedef struct RTuiStyleBuilder RTuiStyleBuilder;
typedef struct RTuiStyle RTuiStyle;
typedef struct RTuiGridLayout RTuiGridLayout;
typedef struct RTuiNodeSpec RTuiNodeSpec;

// =============================================================================
// TERMINAL API
// =============================================================================

/**
 * @brief Create a new terminal
 * @param width Terminal width in characters
 * @param height Terminal height in characters
 * @param out_terminal Output pointer for the created terminal
 * @return Error code
 */
RTuiError rtui_terminal_create(uint16_t width, uint16_t height, RTuiTerminal** out_terminal);

/**
 * @brief Destroy a terminal
 * @param terminal Terminal to destroy
 */
void rtui_terminal_destroy(RTuiTerminal* terminal);

/**
 * @brief Initialize the terminal
 * @param terminal Terminal to initialize
 * @return Error code
 */
RTuiError rtui_terminal_init(RTuiTerminal* terminal);

/**
 * @brief Shutdown the terminal
 * @param terminal Terminal to shutdown
 * @return Error code
 */
RTuiError rtui_terminal_shutdown(RTuiTerminal* terminal);

/**
 * @brief Get terminal size
 * @param terminal Terminal instance
 * @param out_dimensions Output dimensions
 * @return Error code
 */
RTuiError rtui_terminal_get_size(const RTuiTerminal* terminal, RTuiDimensions* out_dimensions);

// =============================================================================
// RENDERER API
// =============================================================================

/**
 * @brief Create a new renderer
 * @param width Renderer width
 * @param height Renderer height
 * @param out_renderer Output pointer for the created renderer
 * @return Error code
 */
RTuiError rtui_renderer_create(uint16_t width, uint16_t height, RTuiRenderer** out_renderer);

/**
 * @brief Destroy a renderer
 * @param renderer Renderer to destroy
 */
void rtui_renderer_destroy(RTuiRenderer* renderer);

/**
 * @brief Resize the renderer
 * @param renderer Renderer to resize
 * @param width New width
 * @param height New height
 * @return Error code
 */
RTuiError rtui_renderer_resize(RTuiRenderer* renderer, uint16_t width, uint16_t height);

/**
 * @brief Clear the renderer with a color
 * @param renderer Renderer to clear
 * @param r Red component (0-255)
 * @param g Green component (0-255)
 * @param b Blue component (0-255)
 * @return Error code
 */
RTuiError rtui_renderer_clear(RTuiRenderer* renderer, uint8_t r, uint8_t g, uint8_t b);

/**
 * @brief Begin or end a frame
 * @param renderer Renderer instance
 * @param begin true to begin frame, false to end frame
 * @return Error code
 */
RTuiError rtui_renderer_frame(RTuiRenderer* renderer, bool begin);

// =============================================================================
// SURFACE API
// =============================================================================

/**
 * @brief Create a new surface
 * @param width Surface width
 * @param height Surface height
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
 * @brief Clear the surface
 * @param surface Surface to clear
 * @param r Red component (0-255)
 * @param g Green component (0-255)
 * @param b Blue component (0-255)
 * @return Error code
 */
RTuiError rtui_surface_clear(RTuiSurface* surface, uint8_t r, uint8_t g, uint8_t b);

/**
 * @brief Set a cell on the surface
 * @param surface Surface to modify
 * @param x X coordinate
 * @param y Y coordinate
 * @param cell Cell data to set
 * @return Error code
 */
RTuiError rtui_surface_set_cell(RTuiSurface* surface, uint16_t x, uint16_t y, const RTuiCell* cell);

/**
 * @brief Get a cell from the surface
 * @param surface Surface to read from
 * @param x X coordinate
 * @param y Y coordinate
 * @param out_cell Output cell data
 * @return Error code
 */
RTuiError rtui_surface_get_cell(const RTuiSurface* surface, uint16_t x, uint16_t y, RTuiCell* out_cell);

// =============================================================================
// DIALOG SYSTEM API
// =============================================================================

/**
 * @brief Dialog types
 */
typedef enum {
    RTUI_DIALOG_CONFIRMATION = 0,
    RTUI_DIALOG_INPUT = 1,
    RTUI_DIALOG_TOAST = 2,
    RTUI_DIALOG_PROGRESS = 3,
    RTUI_DIALOG_WIZARD = 4,
    RTUI_DIALOG_AUTOCOMPLETE = 5
} RTuiDialogType;

/**
 * @brief Dialog positions
 */
typedef enum {
    RTUI_DIALOG_CENTER = 0,
    RTUI_DIALOG_TOP_LEFT = 1,
    RTUI_DIALOG_TOP_RIGHT = 2,
    RTUI_DIALOG_BOTTOM_LEFT = 3,
    RTUI_DIALOG_BOTTOM_RIGHT = 4,
    RTUI_DIALOG_CUSTOM = 5
} RTuiDialogPosition;

/**
 * @brief Dialog results
 */
typedef enum {
    RTUI_DIALOG_CONFIRMED = 0,
    RTUI_DIALOG_CANCELLED = 1,
    RTUI_DIALOG_SELECTED = 2,
    RTUI_DIALOG_DISMISSED = 3
} RTuiDialogResult;

/**
 * @brief Confirmation button types
 */
typedef enum {
    RTUI_CONFIRMATION_OK = 0,
    RTUI_CONFIRMATION_OK_CANCEL = 1,
    RTUI_CONFIRMATION_YES_NO = 2,
    RTUI_CONFIRMATION_YES_NO_CANCEL = 3,
    RTUI_CONFIRMATION_RETRY_CANCEL = 4,
    RTUI_CONFIRMATION_CUSTOM = 5
} RTuiConfirmationButtons;

/**
 * @brief Confirmation icons
 */
typedef enum {
    RTUI_CONFIRMATION_ICON_NONE = 0,
    RTUI_CONFIRMATION_ICON_QUESTION = 1,
    RTUI_CONFIRMATION_ICON_WARNING = 2,
    RTUI_CONFIRMATION_ICON_ERROR = 3,
    RTUI_CONFIRMATION_ICON_INFO = 4,
    RTUI_CONFIRMATION_ICON_SUCCESS = 5
} RTuiConfirmationIcon;

/**
 * @brief Input types
 */
typedef enum {
    RTUI_INPUT_TEXT = 0,
    RTUI_INPUT_PASSWORD = 1,
    RTUI_INPUT_EMAIL = 2,
    RTUI_INPUT_NUMBER = 3,
    RTUI_INPUT_URL = 4,
    RTUI_INPUT_SEARCH = 5,
    RTUI_INPUT_TEL = 6
} RTuiInputType;

/**
 * @brief Toast types
 */
typedef enum {
    RTUI_TOAST_INFO = 0,
    RTUI_TOAST_SUCCESS = 1,
    RTUI_TOAST_WARNING = 2,
    RTUI_TOAST_ERROR = 3
} RTuiToastType;

/**
 * @brief Dialog callback function type
 * @param dialog_id ID of the dialog
 * @param result Dialog result
 * @param data Result data (may be NULL)
 * @param user_data User-provided data
 */
typedef void (*RTuiDialogCallback)(uint64_t dialog_id, RTuiDialogResult result, const char* data, void* user_data);

/**
 * @brief Create a dialog engine
 * @param out_engine Output pointer for the created engine
 * @return Error code
 */
RTuiError rtui_dialog_engine_create(RTuiDialogEngine** out_engine);

/**
 * @brief Destroy a dialog engine
 * @param engine Engine to destroy
 */
void rtui_dialog_engine_destroy(RTuiDialogEngine* engine);

/**
 * @brief Show a confirmation dialog
 * @param engine Dialog engine
 * @param title Dialog title
 * @param message Dialog message
 * @param description Dialog description (may be NULL)
 * @param buttons Button configuration
 * @param icon Icon type
 * @param position Dialog position
 * @param callback Completion callback
 * @param user_data User data for callback
 * @param out_dialog_id Output dialog ID
 * @return Error code
 */
RTuiError rtui_dialog_show_confirmation(
    RTuiDialogEngine* engine,
    const char* title,
    const char* message,
    const char* description,
    RTuiConfirmationButtons buttons,
    RTuiConfirmationIcon icon,
    RTuiDialogPosition position,
    RTuiDialogCallback callback,
    void* user_data,
    uint64_t* out_dialog_id
);

/**
 * @brief Show an input dialog
 * @param engine Dialog engine
 * @param title Dialog title
 * @param prompt Input prompt
 * @param placeholder Placeholder text (may be NULL)
 * @param default_value Default value (may be NULL)
 * @param input_type Type of input
 * @param position Dialog position
 * @param callback Completion callback
 * @param user_data User data for callback
 * @param out_dialog_id Output dialog ID
 * @return Error code
 */
RTuiError rtui_dialog_show_input(
    RTuiDialogEngine* engine,
    const char* title,
    const char* prompt,
    const char* placeholder,
    const char* default_value,
    RTuiInputType input_type,
    RTuiDialogPosition position,
    RTuiDialogCallback callback,
    void* user_data,
    uint64_t* out_dialog_id
);

/**
 * @brief Show a toast notification
 * @param engine Dialog engine
 * @param message Toast message
 * @param toast_type Type of toast
 * @param duration_ms Duration in milliseconds
 * @param position Toast position
 * @param out_dialog_id Output dialog ID
 * @return Error code
 */
RTuiError rtui_dialog_show_toast(
    RTuiDialogEngine* engine,
    const char* message,
    RTuiToastType toast_type,
    uint32_t duration_ms,
    RTuiDialogPosition position,
    uint64_t* out_dialog_id
);

/**
 * @brief Close a dialog
 * @param engine Dialog engine
 * @param dialog_id Dialog ID to close
 * @param result Dialog result
 * @return Error code
 */
RTuiError rtui_dialog_close(RTuiDialogEngine* engine, uint64_t dialog_id, RTuiDialogResult result);

/**
 * @brief Update dialog engine
 * @param engine Dialog engine
 * @param delta_time_ms Time elapsed since last update in milliseconds
 * @return Error code
 */
RTuiError rtui_dialog_engine_update(RTuiDialogEngine* engine, uint32_t delta_time_ms);

// =============================================================================
// COMPONENT SYSTEM API
// =============================================================================

/**
 * @brief Element types
 */
typedef enum {
    RTUI_ELEMENT_COMPONENT = 0,
    RTUI_ELEMENT_TEXT = 1,
    RTUI_ELEMENT_LAYOUT = 2,
    RTUI_ELEMENT_FRAGMENT = 3,
    RTUI_ELEMENT_EMPTY = 4
} RTuiElementType;

/**
 * @brief Layout types
 */
typedef enum {
    RTUI_LAYOUT_FLEX = 0,
    RTUI_LAYOUT_GRID = 1,
    RTUI_LAYOUT_STACK = 2,
    RTUI_LAYOUT_ABSOLUTE = 3
} RTuiLayoutType;

/**
 * @brief Create a component element
 * @param name Component name
 * @param out_element Output pointer for the created element
 * @return Error code
 */
RTuiError rtui_element_create_component(const char* name, RTuiElement** out_element);

/**
 * @brief Create a text element
 * @param text Text content
 * @param out_element Output pointer for the created element
 * @return Error code
 */
RTuiError rtui_element_create_text(const char* text, RTuiElement** out_element);

/**
 * @brief Create a layout element
 * @param layout_type Type of layout
 * @param out_element Output pointer for the created element
 * @return Error code
 */
RTuiError rtui_element_create_layout(RTuiLayoutType layout_type, RTuiElement** out_element);

/**
 * @brief Destroy an element
 * @param element Element to destroy
 */
void rtui_element_destroy(RTuiElement* element);

/**
 * @brief Set element key
 * @param element Element to modify
 * @param key Unique key for the element
 * @return Error code
 */
RTuiError rtui_element_set_key(RTuiElement* element, const char* key);

/**
 * @brief Add child element
 * @param parent Parent element
 * @param child Child element to add
 * @return Error code
 */
RTuiError rtui_element_add_child(RTuiElement* parent, RTuiElement* child);

/**
 * @brief Get element type
 * @param element Element to query
 * @param out_type Output element type
 * @return Error code
 */
RTuiError rtui_element_get_type(const RTuiElement* element, RTuiElementType* out_type);





/**
 * @brief Button variants
 */
typedef enum {
    RTUI_BUTTON_PRIMARY = 0,
    RTUI_BUTTON_SECONDARY = 1,
    RTUI_BUTTON_SUCCESS = 2,
    RTUI_BUTTON_WARNING = 3,
    RTUI_BUTTON_DANGER = 4,
    RTUI_BUTTON_INFO = 5,
    RTUI_BUTTON_LIGHT = 6,
    RTUI_BUTTON_DARK = 7,
    RTUI_BUTTON_CUSTOM = 8
} RTuiButtonVariant;

/**
 * @brief Input types
 */
typedef enum {
    RTUI_INPUT_TEXT = 0,
    RTUI_INPUT_PASSWORD = 1,
    RTUI_INPUT_EMAIL = 2,
    RTUI_INPUT_NUMBER = 3,
    RTUI_INPUT_URL = 4,
    RTUI_INPUT_SEARCH = 5,
    RTUI_INPUT_TEL = 6
} RTuiInputType;

/**
 * @brief Toast types
 */
typedef enum {
    RTUI_TOAST_INFO = 0,
    RTUI_TOAST_SUCCESS = 1,
    RTUI_TOAST_WARNING = 2,
    RTUI_TOAST_ERROR = 3
} RTuiToastType;

/**
 * @brief Dialog callback function type
 * @param dialog_id ID of the dialog
 * @param result Dialog result
 * @param data Result data (may be NULL)
 * @param user_data User-provided data
 */
typedef void (*RTuiDialogCallback)(uint64_t dialog_id, RTuiDialogResult result, const char* data, void* user_data);

/**
 * @brief Create a dialog engine
 * @param out_engine Output pointer for the created engine
 * @return Error code
 */
RTuiError rtui_dialog_engine_create(RTuiDialogEngine** out_engine);

/**
 * @brief Destroy a dialog engine
 * @param engine Engine to destroy
 */
void rtui_dialog_engine_destroy(RTuiDialogEngine* engine);

/**
 * @brief Show a confirmation dialog
 * @param engine Dialog engine
 * @param title Dialog title
 * @param message Dialog message
 * @param ok_text OK button text (NULL for default)
 * @param cancel_text Cancel button text (NULL for default)
 * @param variant Button variant
 * @param position Dialog position
 * @param callback Completion callback
 * @param user_data User data for callback
 * @param out_dialog_id Output dialog ID
 * @return Error code
 */
RTuiError rtui_dialog_show_confirmation(
    RTuiDialogEngine* engine,
    const char* title,
    const char* message,
    const char* ok_text,
    const char* cancel_text,
    RTuiButtonVariant variant,
    RTuiDialogPosition position,
    RTuiDialogCallback callback,
    void* user_data,
    uint64_t* out_dialog_id
);

/**
 * @brief Show an input dialog
 * @param engine Dialog engine
 * @param title Dialog title
 * @param prompt Input prompt
 * @param placeholder Placeholder text (may be NULL)
 * @param default_value Default value (may be NULL)
 * @param input_type Type of input
 * @param position Dialog position
 * @param callback Completion callback
 * @param user_data User data for callback
 * @param out_dialog_id Output dialog ID
 * @return Error code
 */
RTuiError rtui_dialog_show_input(
    RTuiDialogEngine* engine,
    const char* title,
    const char* prompt,
    const char* placeholder,
    const char* default_value,
    RTuiInputType input_type,
    RTuiDialogPosition position,
    RTuiDialogCallback callback,
    void* user_data,
    uint64_t* out_dialog_id
);

/**
 * @brief Show a toast notification
 * @param engine Dialog engine
 * @param message Toast message
 * @param toast_type Type of toast
 * @param duration_ms Duration in milliseconds
 * @param position Toast position
 * @param out_dialog_id Output dialog ID
 * @return Error code
 */
RTuiError rtui_dialog_show_toast(
    RTuiDialogEngine* engine,
    const char* message,
    RTuiToastType toast_type,
    uint32_t duration_ms,
    RTuiDialogPosition position,
    uint64_t* out_dialog_id
);

/**
 * @brief Close a dialog
 * @param engine Dialog engine
 * @param dialog_id Dialog ID to close
 * @param result Dialog result
 * @return Error code
 */
RTuiError rtui_dialog_close(RTuiDialogEngine* engine, uint64_t dialog_id, RTuiDialogResult result);

/**
 * @brief Update dialog engine
 * @param engine Dialog engine
 * @param delta_time_ms Time elapsed since last update in milliseconds
 * @return Error code
 */
RTuiError rtui_dialog_engine_update(RTuiDialogEngine* engine, uint32_t delta_time_ms);

// =============================================================================
// COMPONENT SYSTEM API
// =============================================================================

/**
 * @brief Element types
 */
typedef enum {
    RTUI_ELEMENT_TEXT = 0,
    RTUI_ELEMENT_LAYOUT = 1,
    RTUI_ELEMENT_COMPONENT = 2
} RTuiElementType;

/**
 * @brief Layout types
 */
typedef enum {
    RTUI_LAYOUT_FLEX = 0,
    RTUI_LAYOUT_GRID = 1,
    RTUI_LAYOUT_STACK = 2,
    RTUI_LAYOUT_ABSOLUTE = 3
} RTuiLayoutType;

/**
 * @brief Create a new element
 * @param tag Element tag name
 * @param out_element Output pointer for the created element
 * @return Error code
 */
RTuiError rtui_element_create(const char* tag, RTuiElement** out_element);

/**
 * @brief Create a text element
 * @param text Text content
 * @param out_element Output pointer for the created element
 * @return Error code
 */
RTuiError rtui_element_create_text(const char* text, RTuiElement** out_element);

/**
 * @brief Create a layout element
 * @param layout_type Type of layout
 * @param out_element Output pointer for the created element
 * @return Error code
 */
RTuiError rtui_element_create_layout(RTuiLayoutType layout_type, RTuiElement** out_element);

/**
 * @brief Destroy an element
 * @param element Element to destroy
 */
void rtui_element_destroy(RTuiElement* element);

/**
 * @brief Set element class
 * @param element Element to modify
 * @param class CSS class string
 * @return Error code
 */
RTuiError rtui_element_set_class(RTuiElement* element, const char* class);

/**
 * @brief Set element key
 * @param element Element to modify
 * @param key Unique key for the element
 * @return Error code
 */
RTuiError rtui_element_set_key(RTuiElement* element, const char* key);

/**
 * @brief Add child element
 * @param parent Parent element
 * @param child Child element to add
 * @return Error code
 */
RTuiError rtui_element_add_child(RTuiElement* parent, RTuiElement* child);

/**
 * @brief Set element text content
 * @param element Element to modify
 * @param text Text content
 * @return Error code
 */
RTuiError rtui_element_set_text(RTuiElement* element, const char* text);

/**
 * @brief Set element attribute
 * @param element Element to modify
 * @param name Attribute name
 * @param value Attribute value
 * @return Error code
 */
RTuiError rtui_element_set_attribute(RTuiElement* element, const char* name, const char* value);

/**
 * @brief Get element type
 * @param element Element to query
 * @param out_type Output element type
 * @return Error code
 */
RTuiError rtui_element_get_type(const RTuiElement* element, RTuiElementType* out_type);

/**
 * @brief Get element class
 * @param element Element to query
 * @param out_class Output class string (must be freed with rtui_free_string)
 * @return Error code
 */
RTuiError rtui_element_get_class(const RTuiElement* element, char** out_class);

/**
 * @brief Get element key
 * @param element Element to query
 * @param out_key Output key string (must be freed with rtui_free_string)
 * @return Error code
 */
RTuiError rtui_element_get_key(const RTuiElement* element, char** out_key);

/**
 * @brief Get number of children
 * @param element Element to query
 * @param out_count Output child count
 * @return Error code
 */
RTuiError rtui_element_get_child_count(const RTuiElement* element, size_t* out_count);

/**
 * @brief Get child element at index
 * @param element Parent element
 * @param index Child index
 * @param out_child Output child element
 * @return Error code
 */
RTuiError rtui_element_get_child(const RTuiElement* element, size_t index, RTuiElement** out_child);

// =============================================================================
// PROPS API
// =============================================================================

/**
 * @brief Create props object
 * @param out_props Output pointer for the created props
 * @return Error code
 */
RTuiError rtui_props_create(RTuiProps** out_props);

/**
 * @brief Destroy props object
 * @param props Props to destroy
 */
void rtui_props_destroy(RTuiProps* props);

/**
 * @brief Set a prop value
 * @param props Props object
 * @param key Property key
 * @param value Property value
 * @return Error code
 */
RTuiError rtui_props_set(RTuiProps* props, const char* key, const char* value);

/**
 * @brief Get a prop value
 * @param props Props object
 * @param key Property key
 * @param out_value Output value (must be freed with rtui_free_string)
 * @return Error code
 */
RTuiError rtui_props_get(const RTuiProps* props, const char* key, char** out_value);

// =============================================================================
// ANIMATION SYSTEM API
// =============================================================================

/**
 * @brief Easing function types
 */
typedef enum {
    RTUI_EASING_LINEAR = 0,
    RTUI_EASING_EASE_IN = 1,
    RTUI_EASING_EASE_OUT = 2,
    RTUI_EASING_EASE_IN_OUT = 3,
    RTUI_EASING_EASE_IN_QUAD = 4,
    RTUI_EASING_EASE_OUT_QUAD = 5,
    RTUI_EASING_EASE_IN_OUT_QUAD = 6,
    RTUI_EASING_EASE_IN_CUBIC = 7,
    RTUI_EASING_EASE_OUT_CUBIC = 8,
    RTUI_EASING_EASE_IN_OUT_CUBIC = 9,
    RTUI_EASING_EASE_IN_QUART = 10,
    RTUI_EASING_EASE_OUT_QUART = 11,
    RTUI_EASING_EASE_IN_OUT_QUART = 12,
    RTUI_EASING_EASE_IN_QUINT = 13,
    RTUI_EASING_EASE_OUT_QUINT = 14,
    RTUI_EASING_EASE_IN_OUT_QUINT = 15,
    RTUI_EASING_EASE_IN_SINE = 16,
    RTUI_EASING_EASE_OUT_SINE = 17,
    RTUI_EASING_EASE_IN_OUT_SINE = 18,
    RTUI_EASING_EASE_IN_EXPO = 19,
    RTUI_EASING_EASE_OUT_EXPO = 20,
    RTUI_EASING_EASE_IN_OUT_EXPO = 21,
    RTUI_EASING_EASE_IN_CIRC = 22,
    RTUI_EASING_EASE_OUT_CIRC = 23,
    RTUI_EASING_EASE_IN_OUT_CIRC = 24,
    RTUI_EASING_EASE_IN_BACK = 25,
    RTUI_EASING_EASE_OUT_BACK = 26,
    RTUI_EASING_EASE_IN_OUT_BACK = 27,
    RTUI_EASING_EASE_IN_ELASTIC = 28,
    RTUI_EASING_EASE_OUT_ELASTIC = 29,
    RTUI_EASING_EASE_IN_OUT_ELASTIC = 30,
    RTUI_EASING_EASE_IN_BOUNCE = 31,
    RTUI_EASING_EASE_OUT_BOUNCE = 32,
    RTUI_EASING_EASE_IN_OUT_BOUNCE = 33
} RTuiEasingType;

/**
 * @brief Loop behavior types
 */
typedef enum {
    RTUI_LOOP_NONE = 0,
    RTUI_LOOP_RESTART = 1,
    RTUI_LOOP_REVERSE = 2,
    RTUI_LOOP_PING_PONG = 3
} RTuiLoopBehavior;

/**
 * @brief Animation callback function type
 * @param animation_id Animation ID
 * @param progress Animation progress (0.0 to 1.0)
 * @param user_data User-provided data
 */
typedef void (*RTuiAnimationCallback)(uint64_t animation_id, float progress, void* user_data);

/**
 * @brief Animation completion callback function type
 * @param animation_id Animation ID
 * @param user_data User-provided data
 */
typedef void (*RTuiAnimationCompleteCallback)(uint64_t animation_id, void* user_data);

/**
 * @brief Create an animation manager
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
 * @param delta_time_ms Time elapsed since last update in milliseconds
 * @return Error code
 */
RTuiError rtui_animation_manager_update(RTuiAnimationManager* manager, uint32_t delta_time_ms);

/**
 * @brief Create a new animation
 * @param duration_ms Animation duration in milliseconds
 * @param easing Easing function type
 * @param loop_count Number of loops (0 for infinite)
 * @param loop_behavior Loop behavior
 * @param out_animation Output pointer for the created animation
 * @return Error code
 */
RTuiError rtui_animation_create(
    uint32_t duration_ms,
    RTuiEasingType easing,
    uint32_t loop_count,
    RTuiLoopBehavior loop_behavior,
    RTuiAnimation** out_animation
);

/**
 * @brief Destroy an animation
 * @param animation Animation to destroy
 */
void rtui_animation_destroy(RTuiAnimation* animation);

/**
 * @brief Start an animation
 * @param manager Animation manager
 * @param animation Animation to start
 * @param callback Progress callback
 * @param complete_callback Completion callback
 * @param user_data User data for callbacks
 * @param out_animation_id Output animation ID
 * @return Error code
 */
RTuiError rtui_animation_start(
    RTuiAnimationManager* manager,
    RTuiAnimation* animation,
    RTuiAnimationCallback callback,
    RTuiAnimationCompleteCallback complete_callback,
    void* user_data,
    uint64_t* out_animation_id
);

/**
 * @brief Stop an animation
 * @param manager Animation manager
 * @param animation_id Animation ID to stop
 * @return Error code
 */
RTuiError rtui_animation_stop(RTuiAnimationManager* manager, uint64_t animation_id);

/**
 * @brief Pause an animation
 * @param manager Animation manager
 * @param animation_id Animation ID to pause
 * @return Error code
 */
RTuiError rtui_animation_pause(RTuiAnimationManager* manager, uint64_t animation_id);

/**
 * @brief Resume an animation
 * @param manager Animation manager
 * @param animation_id Animation ID to resume
 * @return Error code
 */
RTuiError rtui_animation_resume(RTuiAnimationManager* manager, uint64_t animation_id);

/**
 * @brief Create a spring animation
 * @param stiffness Spring stiffness
 * @param damping Spring damping
 * @param mass Spring mass
 * @param out_animation Output pointer for the created animation
 * @return Error code
 */
RTuiError rtui_animation_create_spring(float stiffness, float damping, float mass, RTuiAnimation** out_animation);

// =============================================================================
// MEMORY MANAGEMENT
// =============================================================================

/**
 * @brief Free a string returned by the library
 * @param string String to free
 */
void rtui_free_string(char* string);

#ifdef __cplusplus
}
#endif

#endif // REACTIVE_TUI_H
