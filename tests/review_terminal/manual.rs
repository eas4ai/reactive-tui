//! Part of tests/review_terminal.rs: what the manual promises.

/// PLT-016: the graphics manual describes the forwarding of stderr and
/// names the one driver line that is filtered, instead of a blackout.
#[test]
fn plt_016_the_manual_says_what_graphics_startup_filters() {
    let manual = include_str!("../../manual/wgpu-graphics.md");
    for stale in ["stderr goes nowhere", "is lost"] {
        assert!(
            !manual.contains(stale),
            "PLT-016: manual/wgpu-graphics.md still says {stale:?} about stderr while graphics start"
        );
    }
    assert!(
        manual.contains("not a conformant Vulkan implementation"),
        "PLT-016: manual/wgpu-graphics.md does not quote the driver line that graphics startup filters"
    );
}
