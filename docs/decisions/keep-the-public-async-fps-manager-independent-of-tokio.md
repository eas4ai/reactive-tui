# Keep the public async FPS manager independent of Tokio

Level: Judged
Decided by: Codex
Rests on: API-015
Would be wrong if: The public async methods require a Tokio runtime, the lock crosses another await while held, or an existing public method or feature is removed.
History: The feature matrix preserves optional Tokio and the existing public async manager. Previous ownership decisions favor narrow private resource changes without public API removal.

## Decision

Use the async Mutex already provided by mandatory futures-util for the private AsyncAdaptiveFpsManager state. Each async method acquires it and performs one synchronous manager operation; no guard survives another await. Preserve all public signatures and leave the Tokio event-loop adapter behind its existing feature. Gate imports used only by that adapter. Enable the documented portable_simd compiler feature only when the existing nightly-only simd feature is selected, then verify scalar-equivalent color behavior and all matrix configurations.

## Realized by

- fcd24aa35407859fc44a14eb6bb904b5b1083b97 Restore independent no-default and nightly SIMD feature builds
