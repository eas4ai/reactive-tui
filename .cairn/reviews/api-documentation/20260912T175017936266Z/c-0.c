#include <reactive_tui.h>
void documentation_example(void) {
ReactiveTerminal* terminal = NULL;
ReactiveError err = rtui_terminal_create(&terminal);
if (err == RTUI_SUCCESS) {
    // Use terminal
    rtui_terminal_destroy(terminal);
}

}
