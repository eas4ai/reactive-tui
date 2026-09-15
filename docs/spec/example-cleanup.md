# Supported example cleanup

Status: Agreed 2026-09-15
Prefix: EXC

The developer tested the repository examples in Kitty. `gradient_blocks`
worked, while `animated_patterns` rendered a static frame and did not honor
its displayed Ctrl+Q shortcut, `embedded_shell` segfaulted, and the remaining
examples had visible defects. Broken examples are worse than absent examples
because they present unsupported behavior as a working introduction.

[EXC-001]
The existing Rust example set MUST retain `gradient_blocks.rs` and remove
`animated_patterns.rs`, `dialog_engine.rs`, `embedded_shell.rs`,
`suprtui_counter.rs`, `visual_effects.rs`, and `wake_counter.rs`.
Tracked documentation and manifests MUST NOT advertise commands or behavior
from the removed examples. The surviving gradient example MUST compile with
the locked supported toolchain.
Falsifier: A removed example or tracked reference to one remains, the surviving
example is absent, or `cargo +1.91.0 check --locked --example gradient_blocks`
fails.
Mechanism: An exact example inventory and tracked-reference scan run before a
locked compile of `gradient_blocks`.

This cleanup does not claim the removed behavior is repaired. The planned
widget catalog will demonstrate animation with a responsive spinning cube and
will use explicit, verified quit handling.
