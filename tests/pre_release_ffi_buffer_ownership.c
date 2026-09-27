#include "reactive_tui/native.h"

#include <assert.h>
#include <stdio.h>

int main(void) {
    RTuiBuffer *buffer = createOptimizedBuffer(2, 2, false, 0, NULL, 0);
    assert(buffer != NULL);

    uint32_t *characters = bufferGetCharPtr(buffer);
    assert(characters != NULL);
    bufferReleaseCharPtr(characters + 1, 3);
    bufferReleaseCharPtr((uint32_t *)((unsigned char *)characters + 1), 3);
    bufferReleaseCharPtr(characters, 9999);
    bufferReleaseCharPtr(characters, 1);

    float *foreground = bufferGetFgPtr(buffer);
    assert(foreground != NULL);
    bufferReleaseFgPtr(foreground, 0);
    bufferReleaseFgPtr(foreground, 16);

    float *background = bufferGetBgPtr(buffer);
    assert(background != NULL);
    bufferReleaseBgPtr(background, 1);
    bufferReleaseBgPtr(background, 16);

    uint8_t *attributes = bufferGetAttributesPtr(buffer);
    assert(attributes != NULL);
    bufferReleaseAttrPtr(attributes, 4000);
    bufferReleaseAttrPtr(attributes, 4);

    characters = bufferGetCharPtr(buffer);
    assert(characters != NULL);
    destroyOptimizedBuffer(buffer);
    destroyOptimizedBuffer(buffer);
    bufferReleaseCharPtr(characters, 0);
    bufferReleaseCharPtr(characters, 4);

    RTuiTextBuffer *text = createTextBuffer(8, 0);
    assert(text != NULL);
    destroyTextBuffer(text);
    destroyTextBuffer(text);

    RTuiElement *element = NULL;
    assert(rtui_element_empty(&element) == R_TUI_ERROR_SUCCESS && element != NULL);
    assert(rtui_element_add_child(element, element) == R_TUI_ERROR_INVALID_PARAMETER);
    rtui_element_destroy(element);

    puts("FFS-004 buffer ownership and self-parent rejection passed");
    return 0;
}
