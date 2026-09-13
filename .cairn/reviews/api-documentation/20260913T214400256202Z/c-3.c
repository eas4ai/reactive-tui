#include <reactive_tui.h>

int main(void) {
    RTuiError status = rtui_init();
    if (status != R_TUI_ERROR_SUCCESS) return 1;
    rtui_cleanup();
    return 0;
}
