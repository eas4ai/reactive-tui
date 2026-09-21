# enforce dependency policy across supported production targets

Level: Judged
Decided by: Codex
Rests on: DQC-001
Would be wrong if: The allowed license set conflicts with project distribution, the target set omits a supported platform, or an exception passes without naming its exact advisory and mitigation.

## Decision

Check the locked dependency graph for the supported Linux, macOS, and Windows targets with all production features. Deny every vulnerability and unsound advisory, deny direct unmaintained dependencies while DQC-002 resolves the named transitive maintenance findings, deny unknown registries and Git sources, allow only reviewed permissive licenses, and deny duplicate versions except exact package-version skips with reasons. Ban atty explicitly. An advisory ignore is valid only when its exact advisory, package version, exposure, and mitigation are recorded in an active decision.

## Realized by

- 7ca3e155bcfb82fcb528ce406b0041421a7f785f fix: enforce DQC-001 dependency policy
