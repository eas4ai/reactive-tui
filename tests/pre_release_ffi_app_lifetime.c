#include "reactive_tui/native.h"

#include <assert.h>
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>

struct RunState {
    RTuiApp *app;
    enum RTuiError result;
};

static void *run_app(void *data) {
    struct RunState *state = data;
    state->result = rtui_app_run(state->app);
    return NULL;
}

static void effect_callback(void *data) {
    uint32_t *calls = data;
    *calls += 1;
}

int main(void) {
    assert(rtui_init() == R_TUI_ERROR_SUCCESS);

    RTuiAppBuilder *builder = NULL;
    assert(rtui_app_builder_create(&builder) == R_TUI_ERROR_SUCCESS && builder != NULL);
    assert(rtui_app_builder_backend_debug(builder, 80, 24) == R_TUI_ERROR_SUCCESS);
    assert(rtui_app_builder_root_component(builder, NULL, NULL) == R_TUI_ERROR_SUCCESS);

    RTuiApp *app = NULL;
    assert(rtui_app_builder_build(builder, &app) == R_TUI_ERROR_SUCCESS && app != NULL);
    struct RTuiDimensions size = {0, 0};
    assert(rtui_app_get_size(app, &size) == R_TUI_ERROR_SUCCESS);
    assert(size.width == 80 && size.height == 24);

    struct RunState state = {.app = app, .result = R_TUI_ERROR_UNKNOWN};
    pthread_t thread;
    assert(pthread_create(&thread, NULL, run_app, &state) == 0);
    assert(rtui_app_quit(app) == R_TUI_ERROR_SUCCESS);
    assert(pthread_join(thread, NULL) == 0);
    assert(state.result == R_TUI_ERROR_SUCCESS);

    assert(rtui_app_get_size(app, &size) == R_TUI_ERROR_INVALID_STATE);
    assert(rtui_app_run(app) == R_TUI_ERROR_INVALID_STATE);
    assert(rtui_app_quit(app) == R_TUI_ERROR_SUCCESS);
    rtui_app_destroy(app);

    uint32_t effect_calls = 0;
    RTuiEffect *effect = NULL;
    assert(rtui_effect_create(effect_callback, NULL, &effect_calls, &effect) == R_TUI_ERROR_SUCCESS);
    assert(rtui_effect_run(effect) == R_TUI_ERROR_SUCCESS);
    rtui_effect_destroy(effect);
    assert(effect_calls == 1);

    rtui_cleanup();
    puts("FFS-001 app lifetime and nullable callbacks passed");
    return 0;
}
