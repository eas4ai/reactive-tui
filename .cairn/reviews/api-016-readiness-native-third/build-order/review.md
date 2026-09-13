# Native C library build order

Native run 34742335822 verified all five Darwin harness guards, three dependency
readiness tests, five legacy cases, eight Rust App routes, three manual routes,
and all 2048 input bytes (1022 initially queued). Its four C runs failed in dyld:
the library at target/debug/deps/libreactive_tui.dylib lacked the FFI symbol
rtui_app_builder_backend_crossterm. Native output hashes and committed/stable
metadata were verified before preserving the raw artifact.

The mechanism built with FFI and then built a default-feature test dependency.
The latter replaced the deps shared library without FFI; Darwin loaded that
path. Linux inspection independently reproduced the mismatched artifacts:
the top-level library exported the symbol and the deps library did not.
Build the test dependency first and the FFI library last. After correction,
both Linux artifacts export the required symbol and the entire local
entry-point suite passes. This preserves every consumer and assertion.
Native macOS C verification still requires a fresh run.

Focused edit-check passed. Ripwire quality-delta exited 2 on preexisting and
reference findings; test-gate exited 4. These are not passing checks.
Self-audit: the single build-order correction addresses the demonstrated
artifact mismatch without changing production APIs or narrowing acceptance.
