# Reuse native build dependencies while preserving independent failure demonstrations

Surfaced from: API-011
Captured: 2026-09-11T16:34:42.630Z

The native workflow has no build cache. check-conpty-platform.py builds the deliberately broken resize case in a fresh temporary directory with an empty target directory, compiling dependencies again within the same job. Investigate toolchain/platform/lockfile-aware Cargo caching and dependency artifact reuse for the isolated negative build. Keep candidate source untouched, require the corrected case to pass and the disabled-resize case to fail for the expected reason, and never reuse evidence receipts as a substitute for execution. Measure cold and warm job durations and verify cache invalidation with a real source change before claiming improvement.
