//! The startup exchange on Windows (INP-013): while `terminal::query_startup`
//! waits for its replies, the event source assembles them from the console's
//! key records.

use std::sync::atomic::AtomicBool;

use crossterm_winapi::KeyEventRecord;

/// Set while `terminal::query_startup` waits for its replies. The event
/// source then feeds the key records that carry no key code to a reply
/// collector instead of parsing them as keys.
pub(crate) static STARTUP_REPLIES_PENDING: AtomicBool = AtomicBool::new(false);

/// The character of a key record that can be part of a terminal's reply: a
/// key-down record with no virtual-key code, which is how a pseudo console
/// delivers each character of a reply. `None` for a typed key, which carries
/// its key code, for a key release, and for a record without a character.
pub(crate) fn reply_character(record: &KeyEventRecord) -> Option<char> {
    if !record.key_down || record.virtual_key_code != 0 {
        return None;
    }
    char::from_u32(u32::from(record.u_char)).filter(|character| *character != '\0')
}
