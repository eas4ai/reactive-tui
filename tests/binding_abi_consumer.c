#include "reactive_tui/core.h"
#include "reactive_tui/terminal.h"
#include "reactive_tui/builder.h"
#include "reactive_tui/surface.h"
#include "reactive_tui/render.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <termios.h>
#include <unistd.h>

static void terminal_contract(void) {
    RTuiTerminal *terminal = NULL;
    assert(rtui_terminal_create(NULL) == RTUI_NULL_POINTER);
    assert(rtui_terminal_create(&terminal) == RTUI_SUCCESS && terminal);
    RTuiDimensions dimensions = {0, 0};
    assert(rtui_terminal_get_dimensions(terminal, &dimensions) == RTUI_SUCCESS);
    assert(dimensions.width == 80 && dimensions.height == 24);
    struct {
        uint64_t before;
        RTuiCapabilities capabilities;
        uint64_t after;
    } guarded;
    memset(&guarded, 0xA5, sizeof(guarded));
    getTerminalCapabilities(terminal, &guarded.capabilities);
    assert(guarded.before == UINT64_C(0xA5A5A5A5A5A5A5A5));
    assert(guarded.after == UINT64_C(0xA5A5A5A5A5A5A5A5));
    assert(guarded.capabilities.unicode_level >= 1 && guarded.capabilities.unicode_level <= 2);
    assert(guarded.capabilities.mouse);
    setupTerminal(terminal, false);
    struct termios mode;
    assert(tcgetattr(STDIN_FILENO, &mode) == 0);
    assert((mode.c_lflag & (ICANON | ECHO)) == 0);
    setCursorPosition(terminal, 2, 3, true);
    rtui_terminal_destroy(terminal);
    rtui_terminal_destroy(NULL);
}

static void surface_contract(void) {
    RTuiSurface *surface = NULL;
    assert(rtui_surface_create(0, 2, &surface) == RTUI_INVALID_PARAMETER);
    assert(rtui_surface_create(4, 3, &surface) == RTUI_SUCCESS && surface);
    RTuiCell cell = {0};
    cell.ch = 0x754C;
    cell.fg.r = 0x12; cell.fg.g = 0x34; cell.fg.b = 0x56;
    cell.bg.r = 0xAB; cell.bg.g = 0xCD; cell.bg.b = 0xEF;
    cell.attrs.bold = true; cell.attrs.italic = true;
    cell.attrs.underline = true; cell.attrs.reverse = true; cell.attrs.strikethrough = true;
    assert(rtui_surface_set_cell(surface, 3, 2, &cell) == RTUI_SUCCESS);
    RTuiCell result = {0};
    assert(rtui_surface_get_cell(surface, 3, 2, &result) == RTUI_SUCCESS);
    assert(result.ch == cell.ch);
    assert(memcmp(&result.fg, &cell.fg, sizeof(cell.fg)) == 0);
    assert(memcmp(&result.bg, &cell.bg, sizeof(cell.bg)) == 0);
    assert(result.attrs.bold && result.attrs.italic && result.attrs.underline);
    assert(result.attrs.reverse && result.attrs.strikethrough);
    /* The existing native surface API returns a default cell outside its bounds. */
    assert(rtui_surface_get_cell(surface, 9, 2, &result) == RTUI_SUCCESS);
    assert(result.ch == 0); /* Rust's derived char default is NUL. */
    assert(rtui_surface_get_cell(NULL, 0, 0, &result) == RTUI_NULL_POINTER);
    rtui_surface_destroy(surface);
    rtui_surface_destroy(NULL);

    RTuiRenderer *renderer = NULL;
    assert(rtui_renderer_create(4, 3, &renderer) == RTUI_SUCCESS);
    assert(rtui_renderer_get_surface(renderer, &surface) == RTUI_SUCCESS);
    assert(rtui_renderer_clear(renderer, 0x12, 0x34, 0x56) == RTUI_SUCCESS);
    assert(rtui_surface_get_cell(surface, 0, 0, &result) == RTUI_SUCCESS);
    assert(result.bg.r == 0x12 && result.bg.g == 0x34 && result.bg.b == 0x56);
    assert(rtui_renderer_resize(renderer, 7, 2) == RTUI_SUCCESS);
    RTuiDimensions size = {0, 0};
    assert(rtui_surface_get_dimensions(surface, &size) == RTUI_SUCCESS);
    assert(size.width == 7 && size.height == 2);
    assert(rtui_renderer_frame(renderer, true) == RTUI_SUCCESS);
    assert(rtui_surface_set_cell(surface, 1, 1, &cell) == RTUI_SUCCESS);
    assert(rtui_renderer_frame(renderer, false) == RTUI_SUCCESS);
    assert(rtui_renderer_shutdown(renderer) == RTUI_SUCCESS);
    rtui_renderer_destroy(renderer); /* surface is borrowed, never separately freed */
}

static void builder_contract(void) {
    typedef RTuiError (*Factory)(RTuiElementBuilder **);
    Factory factories[] = {rtui_div, rtui_span, rtui_button, rtui_p, rtui_h1, rtui_h2, rtui_h3};
    const char *defaults[] = {"", "inline", "bg-blue-500", "block", "text-4xl", "text-3xl", "text-2xl"};
    for (size_t index = 0; index < sizeof(factories) / sizeof(factories[0]); ++index) {
        RTuiElementBuilder *builder = NULL;
        assert(factories[index](NULL) == RTUI_NULL_POINTER);
        assert(factories[index](&builder) == RTUI_SUCCESS);
        assert(rtui_element_builder_class(builder, "audit") == RTUI_SUCCESS);
        assert(rtui_element_builder_key(builder, "stable-key") == RTUI_SUCCESS);
        assert(rtui_element_builder_text(builder, "hello") == RTUI_SUCCESS);
        RTuiElement *element = NULL;
        assert(rtui_element_builder_build(builder, &element) == RTUI_SUCCESS);
        char *value = NULL;
        assert(rtui_element_get_key(element, &value) == RTUI_SUCCESS);
        assert(strcmp(value, "stable-key") == 0); rtui_free_string(value);
        assert(rtui_element_get_class(element, &value) == RTUI_SUCCESS);
        assert(strstr(value, "audit") && strstr(value, defaults[index])); rtui_free_string(value);
        assert(rtui_element_get_text_content(element, &value) == RTUI_SUCCESS);
        assert(strcmp(value, "hello") == 0); rtui_free_string(value);
        rtui_element_destroy(element);
    }
    RTuiElementBuilder *builder = NULL;
    RTuiElement *child = NULL, *element = NULL;
    assert(rtui_div(&builder) == RTUI_SUCCESS);
    assert(rtui_element_text("child", &child) == RTUI_SUCCESS);
    RTuiElement *duplicate[] = {child, child};
    assert(rtui_element_builder_children(builder, duplicate, 2) == RTUI_INVALID_PARAMETER);
    assert(rtui_element_builder_children(builder, NULL, 1) == RTUI_NULL_POINTER);
    assert(rtui_element_builder_children(builder, NULL, 0) == RTUI_SUCCESS);
    assert(rtui_element_builder_child(builder, child) == RTUI_SUCCESS);
    assert(rtui_element_builder_build(builder, &element) == RTUI_SUCCESS);
    size_t count = 0;
    assert(rtui_element_get_child_count(element, &count) == RTUI_SUCCESS && count == 1);
    RTuiElement *clone = NULL;
    assert(rtui_element_get_child(element, 0, &clone) == RTUI_SUCCESS);
    rtui_element_destroy(element);
    char *value = NULL;
    assert(rtui_element_get_text_content(clone, &value) == RTUI_SUCCESS);
    assert(strcmp(value, "child") == 0); rtui_free_string(value);
    rtui_element_destroy(clone);
    assert(rtui_element_empty(&child) == RTUI_SUCCESS);
    RTuiElement *children[] = {child};
    assert(rtui_card(children, 1, &element) == RTUI_SUCCESS);
    assert(rtui_element_get_child_count(element, &count) == RTUI_SUCCESS && count == 1);
    rtui_element_destroy(element);
    assert(rtui_span(&builder) == RTUI_SUCCESS);
    rtui_element_builder_destroy(builder);
}

int main(void) {
    assert(rtui_init() == RTUI_SUCCESS);
    RTuiVersion version = rtui_version();
    assert(version.abi_version == 1);
    terminal_contract();
    surface_contract();
    builder_contract();
    rtui_cleanup();
    puts("C/C++ ABI consumer lifecycle and value checks passed");
    return 0;
}
