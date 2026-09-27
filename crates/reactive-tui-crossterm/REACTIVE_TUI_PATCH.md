# Crossterm input readiness repair

Upstream: https://github.com/crossterm-rs/crossterm, crates.io release 0.29.0.
Archive SHA-256: d8b9f2e4c67f833b660cdb0a3523065869fb35570177239812ed4c905aeff87b

Preserve pending Unix Mio readiness across returned events and check input
readiness before reading a retained token. Public APIs and Windows code remain
upstream. Regression coverage includes controlled queued input bursts, mixed
readiness, zero-timeout exhaustion and application resize workflows.

The upstream event-stream-async-std example and its async-std
dev-dependency are removed: async-std is discontinued (RUSTSEC-2025-0052).
The event-stream-tokio example shows the same event stream on tokio.

The original MIT license is in LICENSE.

Reactive TUI publishes this implementation as `reactive-tui-crossterm` and
aliases it to the Rust crate name `crossterm`. Downstream builds therefore use
the repaired implementation without a root-only Cargo patch.
