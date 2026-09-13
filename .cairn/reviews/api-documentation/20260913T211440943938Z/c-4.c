#include <reactive_tui/core.h>
#include <reactive_tui/surface.h>

int main(void) {
    RTuiSurface *surface = NULL;
    RTuiError status = rtui_init();
    if (status != R_TUI_ERROR_SUCCESS) return 1;
    status = rtui_surface_create(40, 10, &surface);
    if (status == R_TUI_ERROR_SUCCESS) {
        status = rtui_surface_clear(surface, 15, 20, 30);
        rtui_surface_destroy(surface);
    }
    rtui_cleanup();
    return status == R_TUI_ERROR_SUCCESS ? 0 : 1;
}
