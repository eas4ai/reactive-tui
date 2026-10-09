#include "reactive_tui/core.h"
#include "reactive_tui/builder.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <sys/ioctl.h>
#include <unistd.h>

struct State { unsigned calls; };

static RTuiElement *render_root(void *user_data) {
    struct State *state = user_data;
    struct winsize size;
    assert(ioctl(STDIN_FILENO, TIOCGWINSZ, &size) == 0);
    state->calls++;
    char text[80];
    assert(snprintf(text, sizeof(text), "NATIVE size %u %u e\xcc\x81\xe7\x95\x8c",
                    size.ws_col, size.ws_row) > 0);
    RTuiElementBuilder *builder = NULL;
    RTuiElement *element = NULL;
    assert(rtui_div(&builder) == RTUI_SUCCESS);
    assert(rtui_element_builder_class(builder, "w-full h-1") == RTUI_SUCCESS);
    assert(rtui_element_builder_key(builder, "native-root") == RTUI_SUCCESS);
    assert(rtui_element_builder_text(builder, text) == RTUI_SUCCESS);
    assert(rtui_element_builder_build(builder, &element) == RTUI_SUCCESS);
    return element; /* Ownership transfers to App; user_data lives through run. */
}

int main(int argc, char **argv) {
    assert(argc == 2);
    RTuiAppBuilder *builder = NULL;
    RTuiApp *app = NULL;
    assert(rtui_app_builder_backend_suprtui(NULL) == RTUI_NULL_POINTER);
    assert(rtui_app_builder_create(&builder) == RTUI_SUCCESS);
    assert(rtui_app_builder_backend_suprtui(builder) == RTUI_SUCCESS);
    if (strcmp(argv[1], "reselect") == 0) {
        assert(rtui_app_builder_backend_suprtui(builder) == RTUI_SUCCESS);
        assert(rtui_app_builder_backend_crossterm(builder) == RTUI_SUCCESS);
        assert(rtui_app_builder_backend_suprtui(builder) == RTUI_SUCCESS);
        assert(rtui_app_builder_backend_debug(builder, 32, 8) == RTUI_SUCCESS);
        assert(rtui_app_builder_backend_crossterm(builder) == RTUI_SUCCESS);
        assert(rtui_app_builder_backend_suprtui(builder) == RTUI_SUCCESS);
    }
    if (strcmp(argv[1], "missing-root") == 0) {
        assert(rtui_app_builder_build(builder, &app) != RTUI_SUCCESS);
        assert(app == NULL);
        puts("ENTRY_POINT_BUILD_ERROR");
        return 0;
    }
    struct State state = {0};
    assert(rtui_app_builder_root_component(builder, render_root, &state) == RTUI_SUCCESS);
    assert(rtui_app_builder_build(builder, &app) == RTUI_SUCCESS);
    assert(rtui_app_run(app) == RTUI_SUCCESS); /* Consumes app. */
    assert(state.calls >= 2);
    puts("ENTRY_POINT_CLEAN_EXIT");
    return 0;
}
