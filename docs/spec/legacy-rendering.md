# Legacy rendering observations

Status: Observed
Prefix: OLD

[OLD-001]
The legacy App render path MUST select its initial-repaint branch while previous_tree is empty.
Falsifier: the existing first branch populates previous_tree before returning.
Evidence: `src/app.rs:284`; recorded before the recovery implementation.

[OLD-002]
The legacy DirectTtyBackend MUST return success without painting a small patch set containing only Insert, Remove, or Update.
Falsifier: a path in apply_patches paints one of those patches when there are at most ten.
Evidence: `src/backend/direct_tty.rs:332`; recorded before the recovery implementation.

These requirements record defects as observed behavior. They are not desired
behavior and are excluded from the commitment. See RND-001 for the agreed path.
