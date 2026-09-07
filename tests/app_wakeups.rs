use reactive_tui::app::AppWaker;

#[test]
fn pending_wake_is_retained_and_coalesced() {
    let wake = AppWaker::new();
    for _ in 0..1000 { wake.request_redraw(); }
    assert!(wake.is_pending());
}
