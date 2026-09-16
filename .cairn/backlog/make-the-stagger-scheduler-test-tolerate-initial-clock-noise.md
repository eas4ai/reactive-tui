# Make the stagger scheduler test tolerate initial clock noise

Surfaced from: CAT-002
Outside because: The catalog uses a separate bounded cube clock and does not exercise or modify this hook test's initial-sample equality.
Captured: 2026-09-16T14:06:50.575Z

The catalog regression run failed rac_001_stagger_uses_component_scheduler_and_cancels_all_work because its initial value was 1.9880182e-9 instead of exactly 0.0. Record rerun results in the catalog review. The framework source is unchanged.
