#include "reactive_tui/native.h"

#include <assert.h>
#include <stdint.h>
#include <stdio.h>

static uint32_t write_text(RTuiTextBuffer *buffer, const char *text, uint32_t length) {
    return textBufferWriteChunk(buffer, (const uint8_t *)text, length, NULL, NULL, NULL);
}

int main(void) {
    RTuiTextBuffer *text = createTextBuffer(4, 0);
    assert(text != NULL);
    assert(write_text(text, "abcd", 4) == 4);
    assert(textBufferGetLength(text) == 4);
    assert(textBufferGetCapacity(text) == 4);

    uint32_t *characters = textBufferGetCharPtr(text);
    assert(characters != NULL && characters[0] == 'a' && characters[3] == 'd');

    textBufferResize(text, 2);
    assert(textBufferGetLength(text) == 2);
    assert(textBufferGetCapacity(text) == 2);
    characters = textBufferGetCharPtr(text);
    assert(characters != NULL && characters[0] == 'a' && characters[1] == 'b');

    RTuiBuffer *surface = createOptimizedBuffer(8, 1, false, 0, NULL, 0);
    assert(surface != NULL);
    assert(renderTextBufferToSurface(text, surface, 0, 0, 8) == 2);

    textBufferResize(text, 5);
    assert(textBufferGetLength(text) == 2);
    assert(textBufferGetCapacity(text) == 5);
    assert(write_text(text, "XYZQ", 4) == 3);
    assert(textBufferGetLength(text) == 5);
    characters = textBufferGetCharPtr(text);
    assert(characters != NULL);
    assert(characters[0] == 'a' && characters[1] == 'b');
    assert(characters[2] == 'X' && characters[3] == 'Y' && characters[4] == 'Z');
    assert(renderTextBufferToSurface(text, surface, 1, 0, 7) == 5);

    textBufferResize(text, 3);
    assert(textBufferGetLength(text) == 3);
    assert(textBufferGetCapacity(text) == 3);
    characters = textBufferGetCharPtr(text);
    assert(characters != NULL && characters[2] == 'X');

    destroyOptimizedBuffer(surface);
    destroyTextBuffer(text);
    puts("FFS-002 text-buffer resize invariants passed");
    return 0;
}
