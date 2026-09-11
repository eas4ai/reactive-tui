# Scope performance hook context to its owning App

Surfaced from: API-013
Captured: 2026-09-11T13:11:33.940Z

The new App animation-target render probes exposed existing global performance state leakage: App::publish_performance_context leaves Auto mode and frame duration in GLOBAL_CTX, changing unrelated FPS-hook defaults after App cleanup. API-013 tests now run those App probes in child processes for isolation. Review the global context ownership and cleanup under API-019; target-handle work does not repair this pre-existing behavior.
