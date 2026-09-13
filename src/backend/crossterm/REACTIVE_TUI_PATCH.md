# Crossterm input readiness repair

Upstream: https://github.com/crossterm-rs/crossterm, crates.io release 0.29.0.
Archive SHA-256: d8b9f2e4c67f833b660cdb0a3523065869fb35570177239812ed4c905aeff87b

Preserve pending Unix Mio readiness across returned events and check input
readiness before reading a retained token. Public APIs and Windows code remain
upstream. Regression coverage includes controlled queued input bursts, mixed
readiness, zero-timeout exhaustion and application resize workflows.

The original MIT license is in LICENSE.

Reactive-TUI uses a direct path dependency so downstream checkouts also use
this implementation; a root-only Cargo patch would not propagate to consumers.
