# crates.io release preparation

Status: Agreed 2026-09-14
Prefix: CRT

The developer chose crates.io preparation after the framework manual and before
the public README. The first non-yanked release candidate is version 0.1.0.
Publishing, tagging, and the GitHub release follow the README commitment so the
crate page does not ship the stale public introduction.

[CRT-001]
The release candidate MUST have a registry-resolvable package graph. Every
normal path dependency MUST also declare an exact version. Every package name
MUST be owned by the project or available for first publication. The root crate
MUST use project-owned packages for the exact pinned `libghostty-vt` and
`libghostty-vt-sys` source. The root crate MUST NOT contain a git dependency. Their Rust
library names and the terminal API used by the framework MUST remain unchanged.
Falsifier: A normal dependency has only a path, a git source remains, a package
name belongs to another publisher, or a dependency alias changes the Rust crate
name used by current source.
Mechanism: `python3 scripts/check-crates-release.py` reads every release
manifest and checks names, versions, dependency sources, aliases, and publish
order.

[CRT-002]
Every crate archive MUST contain the source and legal files needed to build and
understand that crate. The root archive MUST also contain the manual and release
changelog. Archives MUST exclude Cairn state, internal specifications, scripts,
verification output, repository automation, and unrelated bindings.
Falsifier: `cargo package --list` shows an internal path, omits a required
runtime or legal file, or includes repository material outside the declared
release boundary.
Mechanism: `python3 scripts/check-crates-release.py` asks Cargo for each archive
file list and validates it against the declared boundary.

[CRT-003]
The 0.1.0 candidate MUST declare the project repository, license, README, and a
conservative Rust 1.91 minimum. Each companion crate MUST package successfully.
The root source tree MUST compile with no default features, default features,
FFI, embedded terminal support using Zig 0.16.0, and nightly SIMD. Commands
MUST run one at a time with no more than eight Cargo jobs.
Falsifier: Metadata is missing or stale, a package build fails, a named root
configuration fails to compile, commands overlap, or a command or its child
tools can use more than eight logical CPUs on Linux.
Mechanism: `bash scripts/check-crates-release-build.sh` packages and checks each
candidate sequentially with `CARGO_BUILD_JOBS=8`.
