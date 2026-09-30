#include "reactive_tui/native.h"
#include <assert.h>
#include <limits.h>
#include <stdint.h>
#include <string.h>

int main(void) {
    for (int i = 0; i < 64; ++i) {
        RTuiSignal *signal = NULL;
        int64_t value = 0;
        assert(rtui_signal_int_create(INT64_MIN, &signal) == 0);
        assert(rtui_signal_int_get(signal, &value) == 0 && value == INT64_MIN);
        assert(rtui_signal_int_set(signal, INT64_MAX) == 0);
        assert(rtui_signal_int_get(signal, &value) == 0 && value == INT64_MAX);
        assert(rtui_signal_bool_set(signal, true) != 0);
        rtui_signal_destroy(signal);
        assert(rtui_signal_float_create(1.25, &signal) == 0);
        double floating = 0;
        assert(rtui_signal_float_set(signal, 5.5) == 0);
        assert(rtui_signal_float_get(signal, &floating) == 0 && floating == 5.5);
        rtui_signal_destroy(signal);
        assert(rtui_signal_bool_create(false, &signal) == 0);
        bool boolean = false;
        assert(rtui_signal_bool_set(signal, true) == 0);
        assert(rtui_signal_bool_get(signal, &boolean) == 0 && boolean);
        rtui_signal_destroy(signal);
        assert(rtui_signal_string_create("hello", &signal) == 0);
        assert(rtui_signal_string_set(signal, "world") == 0);
        char buffer[16];
        assert(rtui_signal_string_get(signal, buffer, sizeof buffer) == 0);
        assert(strcmp(buffer, "world") == 0);
        rtui_signal_destroy(signal);
        signal = rtui_signal_new_int(INT_MAX);
        assert(signal && rtui_signal_get_int(signal) == INT_MAX);
        rtui_signal_destroy_new(signal);
    }
    return 0;
}
