# Minimal doctest linker failure

Formal API-018 receipt 20260913T132658798Z recorded one rust-lld failure in
syn archive relocation processing. Rust compilation diagnostics did not reject
the example; 73 other minimal doctests passed. The same committed source and
artifacts passed all 74 minimal doctests (10 ignored) with RUSTDOCFLAGS adding
-C link-arg=-Wl,--threads=1. No source or dependency artifact was changed.
This establishes a passing bounded-linker run, not the cause of the original
linker failure. The original receipt remains unchanged. Future checks in this
session inherit the one-thread linker flag to respect the user's concurrency
limit across up to 12 concurrent doctest processes. Full formal acceptance is
still required. No product repair is claimed for this toolchain diagnostic.
