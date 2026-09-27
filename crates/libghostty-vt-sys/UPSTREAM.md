# Upstream source

This package contains the `libghostty-vt-sys` source from
[`uzaaft/libghostty-rs`](https://github.com/uzaaft/libghostty-rs) commit
`5988a0b78b4aa804d1c12e66bbfe662bd97d81c0`.

The build script pins Ghostty commit
`22d13172cde98a0a4dda05d3d6a3fcb0dd8ed018`. The Rust library name remains
`libghostty_vt_sys` so the companion safe wrapper keeps its existing imports.

Local change: `build.rs` names its own path as `build.rs`, relative to the
package as Cargo requires, instead of the upstream workspace path
`crates/libghostty-vt-sys/build.rs`, which named a missing file and made
every build rerun the script. With `GHOSTTY_SOURCE_DIR` set it also asks
Cargo to rerun when that checkout changes.

The copied source is distributed under the included MIT license.
