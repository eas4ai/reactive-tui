# Isolate clipboard unit tests from the live desktop

Level: Judged
Decided by: Shawn and Codex
Rests on: DFT-004, MNT-004
Would be wrong if: The fixture skips clipboard hook behavior, removes assertions, changes global environment for concurrent tests, or is presented as proof that live desktop clipboard operations cannot stall.

## Decision

Two whole-suite runs timed out in wl-copy children. Run the clipboard operation tests in bounded child test processes with private command fixtures and environment. Retain backend detection, real command spawning and stdin/stdout transfer, hook state checks, and add a copy/paste round trip. Keep the production clipboard implementation unchanged. Capture its unbounded live-desktop wait separately; this maintenance repair makes unit evidence repeatable and does not certify a desktop integration.

## Realized by

- eae70fd70190cde9d267178dd3fbd122c1726d28 Isolate clipboard unit tests and verify command round trips
