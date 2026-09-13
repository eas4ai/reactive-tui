#include <reactive_tui.h>
void documentation_example(void) {
RTuiTerminal *terminal = NULL;
RTuiError status = rtui_terminal_create(&terminal);
if (status == R_TUI_ERROR_SUCCESS) {
    rtui_terminal_destroy(terminal);
}

}
