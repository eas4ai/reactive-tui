use reactive_tui::hooks::clipboard::use_clipboard;
use reactive_tui::reactive::hooks::Hooks;

#[test]
#[ignore = "requires an explicitly dedicated desktop clipboard session"]
fn clipboard_platform_roundtrip() {
    assert_eq!(
        std::env::var("RTUI_CLIPBOARD_DEDICATED_SESSION").as_deref(),
        Ok("1")
    );
    let backend =
        std::env::var("RTUI_CLIPBOARD_EXPECT_BACKEND").expect("expected platform backend");
    let hooks = Hooks::new();
    let (state, copy, paste) = use_clipboard(&hooks);
    assert_eq!(state.get().backend_name(), backend);
    let large = "界e\u{301}🙂\n".repeat(4096);
    for text in [
        "ASCII control",
        "quote ' 界e\u{301} 🙂\nsecond line\n",
        "",
        "two endings\n\n",
        &large,
    ] {
        copy(text);
        assert!(
            state.get().error.is_none(),
            "{backend} copy of {} bytes: {:?}",
            text.len(),
            state.get().error
        );
        let actual = paste();
        assert!(
            actual.as_deref() == Some(text),
            "{backend} round trip differed for {} bytes; error: {:?}",
            text.len(),
            state.get().error
        );
    }
    hooks.cleanup();
    println!("RTUI_CLIPBOARD_PLATFORM_OK backend={backend} cases=5");
}
