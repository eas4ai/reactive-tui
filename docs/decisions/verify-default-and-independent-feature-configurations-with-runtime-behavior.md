# Verify default and independent feature configurations with runtime behavior

Level: Judged
Decided by: Codex
Rests on: API-015
Would be wrong if: An advertised feature disappears from the matrix, a zero-test run passes, or no-default success depends on another feature enabling Tokio.
History: Existing compatibility decisions preserve working APIs and the explicit nightly SIMD requirement. No feature is removed or relabeled by this matrix.

## Decision

Check the default build, no-default build, each named non-SIMD feature independently, their combined stable configuration, and SIMD alone and all features on nightly. Run focused async FPS and color behavior in each configuration plus existing App component workflows without defaults and with defaults. Verify matrix coverage against Cargo metadata so a new feature cannot be silently omitted. Preserve optional Tokio and all public async FPS methods; the latter must run under a non-Tokio executor.

## Realized by

- 30c0b1cd47b7243b911c078958ca98aae00ff3f6 Declare independent feature builds and runtime acceptance
