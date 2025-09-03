/**
 * @file reactive_tui/dialogs.h
 * @brief Dialog system for Reactive-TUI
 * 
 * This header contains dialog management, modal windows, and
 * user interaction components like confirmations and input dialogs.
 * 
 * @version 0.1.0
 * @date 2025-09-01
 */

#ifndef REACTIVE_TUI_DIALOGS_H
#define REACTIVE_TUI_DIALOGS_H

#ifdef __cplusplus
extern "C" {
#endif

#include "core.h"

// =============================================================================
// OPAQUE TYPES
// =============================================================================

typedef struct RTuiDialogEngine RTuiDialogEngine;
typedef struct RTuiDialog RTuiDialog;

// =============================================================================
// DIALOG ENGINE
// =============================================================================

/**
 * @brief Create a new dialog engine
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
 * @brief Update the dialog engine (call each frame)
 * @param engine Engine to update
 * @return Error code
 */
RTuiError rtui_dialog_engine_update(RTuiDialogEngine* engine);

/**
 * @brief Check if any dialogs are active
 * @param engine Engine to check
 * @param out_active Output active status
 * @return Error code
 */
RTuiError rtui_dialog_engine_has_active_dialogs(const RTuiDialogEngine* engine, bool* out_active);

// =============================================================================
// DIALOG CREATION
// =============================================================================

/**
 * @brief Create a confirmation dialog
 * @param engine Dialog engine
 * @param title Dialog title
 * @param message Dialog message
 * @param confirm_text Confirm button text (or NULL for default)
 * @param cancel_text Cancel button text (or NULL for default)
 * @param out_dialog Output pointer for the created dialog
 * @return Error code
 */
RTuiError rtui_dialog_create_confirmation(
    RTuiDialogEngine* engine,
    const char* title,
    const char* message,
    const char* confirm_text,
    const char* cancel_text,
    RTuiDialog** out_dialog
);

/**
 * @brief Create an input dialog
 * @param engine Dialog engine
 * @param title Dialog title
 * @param prompt Input prompt text
 * @param default_value Default input value (or NULL)
 * @param placeholder Placeholder text (or NULL)
 * @param out_dialog Output pointer for the created dialog
 * @return Error code
 */
RTuiError rtui_dialog_create_input(
    RTuiDialogEngine* engine,
    const char* title,
    const char* prompt,
    const char* default_value,
    const char* placeholder,
    RTuiDialog** out_dialog
);

/**
 * @brief Create a toast notification
 * @param engine Dialog engine
 * @param message Toast message
 * @param duration_ms Duration in milliseconds (0 for persistent)
 * @param out_dialog Output pointer for the created dialog
 * @return Error code
 */
RTuiError rtui_dialog_create_toast(
    RTuiDialogEngine* engine,
    const char* message,
    uint32_t duration_ms,
    RTuiDialog** out_dialog
);

/**
 * @brief Create a progress dialog
 * @param engine Dialog engine
 * @param title Dialog title
 * @param message Progress message
 * @param out_dialog Output pointer for the created dialog
 * @return Error code
 */
RTuiError rtui_dialog_create_progress(
    RTuiDialogEngine* engine,
    const char* title,
    const char* message,
    RTuiDialog** out_dialog
);

// =============================================================================
// DIALOG MANAGEMENT
// =============================================================================

/**
 * @brief Show a dialog
 * @param dialog Dialog to show
 * @return Error code
 */
RTuiError rtui_dialog_show(RTuiDialog* dialog);

/**
 * @brief Hide a dialog
 * @param dialog Dialog to hide
 * @return Error code
 */
RTuiError rtui_dialog_hide(RTuiDialog* dialog);

/**
 * @brief Close a dialog
 * @param dialog Dialog to close
 * @return Error code
 */
RTuiError rtui_dialog_close(RTuiDialog* dialog);

/**
 * @brief Check if dialog is visible
 * @param dialog Dialog to check
 * @param out_visible Output visibility status
 * @return Error code
 */
RTuiError rtui_dialog_is_visible(const RTuiDialog* dialog, bool* out_visible);

// =============================================================================
// DIALOG INTERACTION
// =============================================================================

/**
 * @brief Get dialog result (for confirmation dialogs)
 * @param dialog Dialog to check
 * @param out_confirmed Output confirmation status
 * @return Error code
 */
RTuiError rtui_dialog_get_confirmation_result(const RTuiDialog* dialog, bool* out_confirmed);

/**
 * @brief Get input dialog text
 * @param dialog Dialog to check
 * @param buffer Output buffer for text
 * @param buffer_size Size of output buffer
 * @return Error code
 */
RTuiError rtui_dialog_get_input_text(const RTuiDialog* dialog, char* buffer, size_t buffer_size);

/**
 * @brief Set progress dialog value
 * @param dialog Progress dialog
 * @param progress Progress value (0.0 to 1.0)
 * @return Error code
 */
RTuiError rtui_dialog_set_progress(RTuiDialog* dialog, float progress);

/**
 * @brief Update progress dialog message
 * @param dialog Progress dialog
 * @param message New message
 * @return Error code
 */
RTuiError rtui_dialog_set_progress_message(RTuiDialog* dialog, const char* message);

#ifdef __cplusplus
}
#endif

#endif // REACTIVE_TUI_DIALOGS_H
