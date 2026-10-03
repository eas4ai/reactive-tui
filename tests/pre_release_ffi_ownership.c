#include "reactive_tui/native.h"

#include <assert.h>
#include <stdio.h>

int main(void) {
    assert(rtui_init() == R_TUI_ERROR_SUCCESS);

    RTuiAppBuilder *builder = NULL;
    RTuiApp *app = NULL;
    assert(rtui_app_builder_create(&builder) == R_TUI_ERROR_SUCCESS);
    assert(rtui_app_builder_build(builder, &app) == R_TUI_ERROR_INTERNAL_ERROR);
    assert(app == NULL);
    rtui_app_builder_destroy(builder);

    RTuiAnimationManager *manager = NULL;
    RTuiAnimation *animation = NULL;
    assert(rtui_animation_manager_create(&manager) == R_TUI_ERROR_SUCCESS);
    assert(rtui_animation_create(
               "owned", 100, R_TUI_EASING_TYPE_LINEAR, R_TUI_LOOP_MODE_NONE, 0, &animation) ==
           R_TUI_ERROR_SUCCESS);
    char *animation_id = NULL;
    assert(rtui_animation_manager_add(manager, animation, &animation_id) == R_TUI_ERROR_SUCCESS);
    assert(animation_id != NULL);
    assert(rtui_animation_play(animation) == R_TUI_ERROR_INVALID_POINTER);
    rtui_animation_destroy(animation);
    assert(rtui_animation_manager_update(manager) == R_TUI_ERROR_SUCCESS);
    assert(rtui_animation_manager_remove(manager, animation_id) == R_TUI_ERROR_SUCCESS);
    rtui_free_string(animation_id);
    rtui_animation_manager_destroy(manager);

    rtui_cleanup();
    puts("FFS-003 ownership and callback synchronization passed");
    return 0;
}
