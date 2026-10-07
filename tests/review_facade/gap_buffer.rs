//! Part of tests/review_facade.rs: TXT-004, the gap buffer's documentation
//! states its cost and its edits move text in place.

use reactive_tui::editor::GapBuffer;

/// The module documentation of src/editor/gap_buffer.rs: its `//!` lines.
fn module_docs() -> String {
    include_str!("../../src/editor/gap_buffer.rs")
        .lines()
        .take_while(|line| line.starts_with("//!") || line.trim().is_empty())
        .filter(|line| line.starts_with("//!"))
        .map(|line| line.trim_start_matches("//!").trim())
        .collect::<Vec<_>>()
        .join(" ")
}

/// TXT-004: the documentation does not call the buffer zero-copy or its
/// edits O(1) without qualification.
#[test]
fn txt_004_the_documentation_states_the_cost() {
    let docs = module_docs();
    let lower = docs.to_lowercase();
    assert!(
        !lower.contains("zero-copy") && !lower.contains("zero copy") && !docs.contains("O(1)"),
        "TXT-004: the gap buffer's documentation still claims zero-copy or O(1) edits: {docs:?}"
    );
}

/// TXT-004: an insert of one character that only moves the gap allocates
/// nothing: the text between the old and the new gap moves in place.
#[test]
fn txt_004_an_insert_that_moves_the_gap_allocates_nothing() {
    let mut buffer = GapBuffer::from_string(&"a".repeat(4096)).unwrap();
    // The gap sits at the end after this insert; the next one at the start
    // moves it across the whole text.
    buffer.insert_char(4096, 'z').unwrap();
    let allocations = crate::allocations_during(|| buffer.insert_char(0, 'x').unwrap());
    assert_eq!(
        allocations, 0,
        "TXT-004: a one-character insert that moved the gap allocated {allocations} times"
    );
    assert_eq!(buffer.get_char(0), Some('x'));
    assert_eq!(buffer.len(), 4098);
}

/// TXT-004: constructors and inserts enforce bytes and refuse edits atomically.
#[test]
fn txt_004_the_limit_is_in_bytes_and_enforced() {
    use reactive_tui::editor::GapBufferError;
    let text = "界".repeat(300);
    let expected = GapBufferError::LimitExceeded {
        requested_bytes: 1200,
        limit_bytes: 1024,
    };
    let mut buffer = GapBuffer::with_limit_bytes(1024).unwrap();
    assert_eq!(buffer.insert_str(0, &text), Err(expected.clone()));
    assert!(buffer.is_empty());
    assert_eq!(
        GapBuffer::from_string_with_limit(&text, 1024).unwrap_err(),
        expected
    );
    buffer.insert_str(0, &"a".repeat(256)).unwrap();
    let before = buffer.to_string();
    assert!(matches!(
        buffer.insert_char(10, 'x'),
        Err(GapBufferError::LimitExceeded {
            requested_bytes: 1028,
            limit_bytes: 1024
        })
    ));
    assert_eq!(buffer.to_string(), before);
    assert!(GapBuffer::with_capacity(GapBuffer::DEFAULT_LIMIT_BYTES / 4 + 1).is_err());
    assert!(GapBuffer::with_capacity(usize::MAX).is_err());
    assert!(GapBuffer::with_limit_bytes(0)
        .unwrap()
        .insert_char(0, 'a')
        .is_err());
}
