#include <rtui.h>
#include <stdio.h>

int main() {
    ReactiveError err;
    ReactiveTerminal* terminal = NULL;
    RTuiRenderer* renderer = NULL;
    RTuiSurface* surface = NULL;
    
    // Initialize library
    err = rtui_init();
    if (err != RTUI_SUCCESS) return 1;
    
    // Create terminal
    err = rtui_terminal_create(&terminal);
    if (err != RTUI_SUCCESS) goto cleanup;
    
    // Enter raw mode
    err = rtui_terminal_enter_raw_mode(terminal);
    if (err != RTUI_SUCCESS) goto cleanup;
    
    // Create renderer
    err = rtui_renderer_create(terminal, &renderer);
    if (err != RTUI_SUCCESS) goto cleanup;
    
    // Main loop
    while (1) {
        // Begin frame
        rtui_renderer_begin_frame(renderer);
        
        // Get surface for drawing
        rtui_renderer_get_surface(renderer, &surface);
        
        // Draw something
        RTuiColor white = {255, 255, 255};
        RTuiColor black = {0, 0, 0};
        rtui_surface_draw_text(surface, 10, 10, "Hello from C!", &white, &black);
        
        // End frame
        rtui_renderer_end_frame(renderer, terminal);
        
        // Check for exit
        RTuiEvent event;
        if (rtui_terminal_poll_event(terminal, 100, &event) == RTUI_SUCCESS) {
            if (event.event_type == RTUI_EVENT_KEY && event.data.key.key_code == 27) {
                break; // ESC pressed
            }
        }
    }
    
cleanup:
    if (renderer) {
        rtui_renderer_shutdown(renderer, terminal);
        rtui_renderer_destroy(renderer);
    }
    if (terminal) {
        rtui_terminal_exit_raw_mode(terminal);
        rtui_terminal_destroy(terminal);
    }
    rtui_cleanup();
    
    return err == RTUI_SUCCESS ? 0 : 1;
}
