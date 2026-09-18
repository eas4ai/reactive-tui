# Pre-release dependency and code quality

Status: Agreed 2026-09-14
Prefix: DQC

This specification remediates Fable audit findings H6, M8, M11, M12, and L17.
It covers the production dependency graph, terminal-safe diagnostics, dead
code, meaningful tests, and the documented public API.

[DQC-001]
The production dependency graph MUST have no known vulnerable or unsound
package without a reviewed exception that names the exposure and mitigation.
Unused vulnerable dependencies MUST be removed. A checked-in cargo-deny policy
MUST enforce advisories, licenses, sources, and allowed duplicates.
Falsifier: cargo audit or cargo deny reports an unreviewed vulnerability or
unsound package, `atty` remains reachable, or the policy omits a required
section.
Mechanism: Locked cargo audit and cargo deny checks run against the production
feature graph and compare every exception with its decision record.

[DQC-002]
Maintained replacements MUST be used for unmaintained production dependencies
when available. Duplicate taffy and vte versions MUST be aligned or justified.
The maintained crossterm fork MUST keep a distinct package identity. The
default Markdown feature set MUST NOT require oniguruma.
Falsifier: The locked graph contains an unexplained unmaintained package, an
unapproved taffy or vte duplicate, an ambiguous crossterm package name, or
onig_sys in the default dependency tree.
Mechanism: Dependency-tree assertions inspect default, minimal, FFI, and
Markdown feature graphs on the supported toolchain.

[DQC-003]
Library diagnostics MUST use the logging API or a caller-owned output channel.
Normal library operation MUST NOT print directly to the process stdout or
stderr streams.
Falsifier: A non-test library path reaches `print!`, `println!`, `eprint!`, or
`eprintln!`, or a captured terminal session contains an internal diagnostic.
Mechanism: A source audit and captured-output integration test exercise parser,
terminal, reconciliation, focus, and window diagnostic paths.

[DQC-004]
Dead FFI modules, disconnected integration tests, and unsafe test hooks MUST be
removed or connected to a maintained public behavior. Tests MUST make an
observable assertion instead of treating absence of a panic as success.
Falsifier: Shipped source contains an unreferenced export module or unsafe
global test hook, a named integration test is not compiled, or a readiness test
can pass without checking a result.
Mechanism: Target enumeration, export inventory, dead-code review, and mutation
of sampled assertions prove that the intended tests compile and can fail.

[DQC-005]
Every supported public item MUST have useful API documentation. Test-only
helpers MUST be gated from production builds. Unreferenced legacy public
helpers MUST be removed or explicitly retained.
Falsifier: rustdoc reports a missing supported item, `create_test_image` ships
as production API without a contract, or an unreferenced legacy control helper
remains public without a retention decision.
Mechanism: Strict rustdoc and public-API inventory checks run with all supported
feature sets and compare intentional exceptions with recorded decisions.
