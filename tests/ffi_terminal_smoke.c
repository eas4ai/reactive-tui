/* Compile against the shipped declarations and call the real shared library. */
#include "reactive_tui/terminal.h"
#include <assert.h>
#include <stdio.h>

int main(void) {
    RTuiTerminal *terminal = NULL;
    RTuiDimensions dimensions = {0};
    RTuiEvent event = {0};
    assert(rtui_terminal_create(NULL) == RTUI_NULL_POINTER);
    assert(rtui_terminal_create(&terminal) == RTUI_SUCCESS);
    assert(terminal != NULL);
    assert(rtui_terminal_get_dimensions(terminal, &dimensions) == RTUI_SUCCESS);
    assert(dimensions.width == 80 && dimensions.height == 24);
    assert(rtui_terminal_get_dimensions(terminal, NULL) == RTUI_NULL_POINTER);
    assert(rtui_terminal_get_dimensions(NULL, &dimensions) == RTUI_NULL_POINTER);
    assert(rtui_terminal_sync(terminal, true) == RTUI_SUCCESS);
    assert(rtui_terminal_sync(terminal, false) == RTUI_SUCCESS);
    assert(rtui_terminal_sync(NULL, true) == RTUI_NULL_POINTER);
    assert(rtui_terminal_poll_event(0, NULL) == RTUI_NULL_POINTER);
    assert(rtui_terminal_poll_event(0, &event) == RTUI_NOT_FOUND);
    rtui_terminal_destroy(terminal);
    rtui_terminal_destroy(NULL);
    puts("C terminal ABI smoke passed");
    return 0;
}
