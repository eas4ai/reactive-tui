# Repair legacy signal handle ownership and keep signal families distinct

Surfaced from: ABI-001
Captured: 2026-09-08T21:50:21.710Z

ABI source review found that rtui_signal_string_create/int_create/float_create/bool_create allocate different Signal<T> values, while rtui_signal_destroy always drops Signal<String>. The newer rtui_signal_new_* API instead allocates FFISignal and requires rtui_signal_destroy_new. Both families use the same opaque RTuiSignal name, so mixing them is unsafe. No mismatched call was executed. Repair or retire the legacy family in a separate native reactive commitment; binding migration guidance directs consumers to the matched newer family.
