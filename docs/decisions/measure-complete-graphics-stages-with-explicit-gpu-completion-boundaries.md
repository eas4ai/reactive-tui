# Measure complete graphics stages with explicit GPU completion boundaries

Level: Judged
Decided by: agent
Rests on: GPU-005
Would be wrong if: Render timing ends before GPU completion, readback includes unfinished rendering, or reported presentation is claimed to measure display scanout.

## Decision

Split draw and texture-copy submissions so a bounded cancellable completion wait ends rendering before readback begins. Measure the same path used by the catalog, including this extra synchronization cost. A bounded native-backend benchmark measures conversion and terminal-write completion separately, excluding App reconciliation and display scanout. Report initialization separately, actual hardware identity, and measured CPU/GPU pixel differences; verify Kitty in an owned Xvfb display, not the developer desktop.

## Realized by

- a1ebe32ad27e54b1164778d0d55592d38de47257 feat: measure graphics stages and verify isolated Kitty presentation
