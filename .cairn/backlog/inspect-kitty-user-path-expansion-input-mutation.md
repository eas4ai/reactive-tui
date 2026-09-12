# Inspect Kitty user-path expansion input mutation

Surfaced from: API-014
Captured: 2026-09-12T15:20:04.029Z

The pinned Kitty 0.45.0 source temporarily writes through the slash returned from strchr on a const char path in kitty/launcher/utils.h expand_tilde (lines 60-62). Callers include configuration environment strings and PyUnicode_AsUTF8 in kitty/data-types.c. GCC reports discarded qualifiers. The image-host build retains this as a visible warning using a specific warning-as-error exception. The image controls use private absolute configuration paths and do not audit user-path expansion. Investigate the input ownership and make any upstream path repair separately; it is outside the approved image-layer ordering change. No runtime failure or exploit is asserted.
