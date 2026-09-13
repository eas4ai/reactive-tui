#include <reactive_tui.h>

int main(void) {
    RTuiRenderer *renderer = NULL;
    RTuiSurface *surface = NULL;
    RTuiError status = rtui_init();
    if (status != R_TUI_ERROR_SUCCESS) return 1;

    status = rtui_renderer_create(40, 10, &renderer);
    if (status != R_TUI_ERROR_SUCCESS) goto cleanup;
    status = rtui_renderer_get_surface(renderer, &surface);
    if (status != R_TUI_ERROR_SUCCESS) goto cleanup;
    status = rtui_renderer_frame(renderer, true);
    if (status != R_TUI_ERROR_SUCCESS) goto cleanup;
    status = rtui_surface_clear(surface, 15, 20, 30);
    if (status != R_TUI_ERROR_SUCCESS) goto cleanup;
    const RTuiColor white = {255, 255, 255};
    const RTuiColor background = {15, 20, 30};
    status = rtui_surface_draw_text(surface, 2, 2, "Hello from C", &white, &background);
    if (status != R_TUI_ERROR_SUCCESS) goto cleanup;
    status = rtui_renderer_frame(renderer, false);

cleanup:
    if (renderer) {
        RTuiError shutdown = rtui_renderer_shutdown(renderer);
        if (status == R_TUI_ERROR_SUCCESS) status = shutdown;
        rtui_renderer_destroy(renderer);
    }
    rtui_cleanup();
    return status == R_TUI_ERROR_SUCCESS ? 0 : 1;
}
