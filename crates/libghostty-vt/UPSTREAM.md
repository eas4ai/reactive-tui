# Upstream source

This package contains the `libghostty-vt` source from
[`uzaaft/libghostty-rs`](https://github.com/uzaaft/libghostty-rs) commit
`5988a0b78b4aa804d1c12e66bbfe662bd97d81c0`.

Reactive TUI publishes it under a project-owned package name because the
published upstream 0.2.1 package does not expose the API used by the framework.
The Rust library name remains `libghostty_vt` for source compatibility.

Local change: the optional `allocator-api2` dependency (feature
`allocator_api`) is 0.2 instead of upstream's 0.4, the line hashbrown uses
elsewhere in the workspace, so the dependency checks find one version.
The crate uses only its `Allocator` trait and `Layout`, which 0.2 has.

The copied source is distributed under the included MIT license.
