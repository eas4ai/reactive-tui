# Repair deep render-tree stack overflow with the embedded feature

Captured: 2026-09-16
Surfaced while verifying: EMB-002 control-text conversion repair

The full library suite with embedded-terminal aborts in
render::tree::tests::api019_public_render_tree_keeps_fragment_custom_deep_and_wide_nodes.
Its 2048-level tree runs in a 128 KiB thread and overflows that stack. An isolated
run also aborts. Removing the control-text conversion fix and rerunning the same
test still aborts, so the snapshot repair does not cause this failure. No render
tree code changed in this repair. The remaining suite passes with 1053 passed,
eight ignored, and this one test explicitly filtered out. Investigate the deep
tree separately; do not report the unrestricted feature-enabled suite as passing.
