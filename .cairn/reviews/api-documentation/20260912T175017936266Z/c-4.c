#include <reactive_tui/core.h>
#include <reactive_tui/terminal.h>
#include <reactive_tui/render.h>

int main() {
    rtui_init();
    
    RTuiTerminal* terminal;
    RTuiRenderer* renderer;
    
    rtui_terminal_create(&terminal);
    rtui_renderer_create(terminal, &renderer);
    
    // Basic rendering only...
    
    rtui_renderer_destroy(renderer);
    rtui_terminal_destroy(terminal);
    rtui_cleanup();
}
