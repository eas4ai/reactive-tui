# Cargo feature configurations

API-015 requires the default and no-default builds, each non-SIMD feature alone
without defaults, their combined configuration, and nightly SIMD alone and with
all features. `scripts/check-api-features.py` checks the catalog against Cargo
metadata, compiles all targets and runs feature behavior in each configuration.
Existing App component workflows also run with and without default features.
The matrix is an acceptance obligation; consult `.cairn/evidence/API-015` for
current results rather than treating this declaration as a passing build.

| Feature | Contract |
| --- | --- |
| default | Enables `tokio`. |
| tokio | Exposes the Tokio event-loop adapter. |
| tokio-util | Retains the existing optional dependency feature. |
| async-capabilities | Retains the existing alias that enables `tokio`. |
| debug, debug_patches | Enable their existing diagnostic paths. |
| ffi | Retains the existing binding feature. |
| embedded-terminal | Enables the pinned libghostty embedded-terminal path on Unix. |
| simd | Enables portable SIMD color operations and requires nightly Rust. |

No-default builds must retain core App rendering and the public async FPS manager.
That manager must preserve reads and updates when driven by a non-Tokio executor;
users must not need to create a Tokio runtime for it. This does not expose the
Tokio-specific event-loop type when the `tokio` feature is disabled. No feature
name or working API is removed by this requirement.

Run `cairn check API-015` with the repository's target directory selected and
both the active stable toolchain and `nightly` installed. The receipt records
the actual compiler output; selecting `simd` on stable is outside its documented
nightly-only contract. Native host behavior remains covered by the separate
platform receipts; a feature build does not substitute for those workflows.
