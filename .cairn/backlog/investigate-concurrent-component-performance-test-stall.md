# Investigate concurrent component performance test stall

Surfaced from: RND-001
Captured: 2026-09-07T22:08:47.435Z

During final renderer regression testing, both tests in tests/simple_performance_test.rs stalled together for over 60 seconds with zero CPU. They call shared global component registry/cleanup APIs and do not use the new renderer. Stopped only that test process; both tests passed immediately with --test-threads=1. Earlier parallel full-suite runs passed this target. Investigate the intermittent shared-registry/test concurrency issue; no legacy repair was made in the renderer commitment.
