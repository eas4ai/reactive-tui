# Retain reviewed wgpu 27 transitive policy exceptions

Level: Judged
Decided by: Codex
Rests on: DQC-001, DQC-002, GPU-001, 20260917T113752918Z-971432
Would be wrong if: The exception admits another license or package version, ignores an advisory, hides an entire dependency subtree, or the retained upstream constraints can converge without a dependency API migration.

## Decision

Keep the approved wgpu 27.0.1 graph unchanged. Permit CC0-1.0 only for hexf-parse 0.2.1, the Naga hexadecimal-float parser, not globally. The upstream package manifest declares that license; the Creative Commons legal code includes a copyright waiver and fallback permission, but explicitly does not waive patent or trademark rights and provides no warranty. This is a limited dependency-policy choice, not a legal guarantee. Keep duplicate denial and all advisory, source, and license enforcement. Retain only the incompatible upstream lines: foldhash 0.1.5/0.2.0, hashbrown 0.15.5/0.16.1/0.17.1, rustc-hash 1.1.0/2.1.3, thiserror and thiserror-impl 1.0.69/2.0.20. gpu-descriptor requires hashbrown 0.15; wgpu/Naga require 0.16; current accessibility/indexmap/lru use 0.17. Naga/core use rustc-hash 1 while comrak uses 2. gpu-allocator uses thiserror 1 while the root/wgpu use 2. These semver-incompatible types remain inside their owning dependencies. Add exact-version skips only for the older lines and cite this decision; do not use skip-tree, global warn/allow, dependency overrides, or new forks. Revisit each exception when its upstream constraint changes. Include the actual supported Windows GNU target in the all-feature policy graph.

## Sources

Source references: [CC0 legal code](https://creativecommons.org/publicdomain/zero/1.0/legalcode.en),
[crate-specific license exceptions](https://embarkstudios.github.io/cargo-deny/checks/licenses/cfg.html#the-exceptions-field-optional),
and [duplicate-version skips](https://embarkstudios.github.io/cargo-deny/checks/bans/cfg.html#the-skip-field-optional).

## Realized by

(none yet: recorded, not built)
