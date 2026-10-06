//! The startup exchange's replies, read the same way on every platform: the
//! background color reply to `OSC 11 ; ?`, the Sixel attribute of a primary
//! device attributes reply, and the collector that assembles a reply from
//! input that arrives one character at a time (Windows console records).

use std::io;

use crate::event::InternalEvent;

fn could_not_parse_event_error() -> io::Error {
    io::Error::other("Could not parse an event.")
}

/// The reply to `OSC 11 ; ?`: `ESC ] 11 ; rgb:R/G/B`, ended by BEL or ST,
/// each channel one to four hex digits, scaled to 16 bits. `Ok(None)` while
/// the reply is not complete; an error for anything else after `ESC ]`.
pub(crate) fn background_color(buffer: &[u8]) -> io::Result<Option<InternalEvent>> {
    assert!(buffer.starts_with(b"\x1B]")); // ESC ]
    let body = &buffer[2..];
    let text = if let Some(text) = body.strip_suffix(b"\x07") {
        text
    } else if let Some(text) = body.strip_suffix(b"\x1B\\") {
        text
    } else if body.len() > 64 {
        return Err(could_not_parse_event_error());
    } else {
        return Ok(None);
    };
    let channels: Vec<u16> = std::str::from_utf8(text)
        .ok()
        .and_then(|text| text.strip_prefix("11;rgb:"))
        .map(|rgb| {
            rgb.split('/')
                .filter_map(|hex| {
                    let digits = u32::try_from(hex.len())
                        .ok()
                        .filter(|n| (1..=4).contains(n))?;
                    let value = u32::from_str_radix(hex, 16).ok()?;
                    Some((value * 0xFFFF / ((1 << (4 * digits)) - 1)) as u16)
                })
                .collect()
        })
        .unwrap_or_default();
    match channels[..] {
        [red, green, blue] => Ok(Some(InternalEvent::BackgroundColor(red, green, blue))),
        _ => Err(could_not_parse_event_error()),
    }
}

/// Whether the attributes of a primary device attributes reply, the bytes
/// between `ESC [ ?` and the final `c`, list Sixel graphics (attribute 4,
/// <https://vt100.net/docs/vt510-rm/DA1.html>).
pub(crate) fn lists_sixel(attributes: &[u8]) -> bool {
    attributes
        .split(|byte| *byte == b';')
        .any(|attribute| attribute == b"4")
}

/// What feeding one record to a [`ReplyCollector`] gave.
#[cfg_attr(not(windows), allow(dead_code))]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Fed<R> {
    /// The record may be part of a reply and is held.
    Held,
    /// The held records formed this reply.
    Reply(InternalEvent),
    /// These records are not a reply, in their order: keys to deliver as
    /// they are. Empty when the held records were a reply to a question
    /// that was not asked, which no one typed and no one reads.
    Keys(Vec<R>),
}

/// Assembles a startup reply from input that arrives one character per
/// record, as a Windows console delivers a terminal's reply (INP-013): each
/// character comes as a key record with no key code, so a record with a key
/// code is a typed key and never part of a reply. A typed key passes
/// through at once, whatever is held: it says nothing about the records
/// around it, which the terminal sent.
///
/// A reply starts with an Escape. While the collector holds records it
/// decides on each new one: an OSC string that ends is the background color
/// reply; a CSI sequence that ends is the device attributes reply; an Escape
/// followed by anything else gives the held records back as keys. Any other
/// complete sequence is from the terminal too: while the startup exchange is
/// pending it is dropped, and afterwards it is given back as keys, as it
/// read before the collector existed. The collector keeps collecting after
/// the exchange, so a reply that arrives late is still a reply, which the
/// event source then consumes.
///
/// Only the Windows event source feeds it; the Unix parsers read a reply
/// from a byte buffer. It is built and tested on every platform.
#[cfg_attr(not(windows), allow(dead_code))]
#[derive(Debug)]
pub(crate) struct ReplyCollector<R> {
    held: Vec<R>,
    bytes: Vec<u8>,
}

impl<R> Default for ReplyCollector<R> {
    fn default() -> Self {
        Self {
            held: Vec::new(),
            bytes: Vec::new(),
        }
    }
}

/// A reply longer than this is not one the startup exchange asked for.
#[cfg_attr(not(windows), allow(dead_code))]
const LONGEST_REPLY: usize = 128;

#[cfg_attr(not(windows), allow(dead_code))]
impl<R> ReplyCollector<R> {
    /// Whether records are held.
    pub(crate) fn is_empty(&self) -> bool {
        self.held.is_empty()
    }

    /// Takes every held record.
    fn take(&mut self) -> Vec<R> {
        self.bytes.clear();
        std::mem::take(&mut self.held)
    }

    /// What a complete sequence that is no reply becomes: nothing while the
    /// exchange is pending, keys afterwards.
    fn not_a_reply(&mut self, pending: bool) -> Fed<R> {
        let held = self.take();
        Fed::Keys(if pending { Vec::new() } else { held })
    }

    /// Feeds one record; `character` is its character when it has no key
    /// code (a candidate reply character), and `None` for a typed key;
    /// `pending` says whether the startup exchange still waits for replies.
    pub(crate) fn feed(&mut self, record: R, character: Option<char>, pending: bool) -> Fed<R> {
        let Some(character) = character else {
            // A typed key: delivered at once, around whatever the terminal
            // is sending.
            return Fed::Keys(vec![record]);
        };
        if self.held.is_empty() {
            if character != '\x1B' {
                return Fed::Keys(vec![record]);
            }
            self.held.push(record);
            self.bytes.push(0x1B);
            return Fed::Held;
        }
        self.held.push(record);
        let mut utf8 = [0u8; 4];
        self.bytes
            .extend_from_slice(character.encode_utf8(&mut utf8).as_bytes());
        match self.bytes.get(1) {
            Some(b']') => match background_color(&self.bytes) {
                Ok(None) if self.bytes.len() <= LONGEST_REPLY => Fed::Held,
                Ok(Some(reply)) => {
                    self.take();
                    Fed::Reply(reply)
                }
                // Another OSC string, or one too long to be the reply.
                _ => self.not_a_reply(pending),
            },
            Some(b'[') => {
                let last = *self.bytes.last().unwrap();
                if self.bytes.len() > 2 && (0x40..=0x7E).contains(&last) {
                    if last == b'c' && self.bytes.get(2) == Some(&b'?') {
                        let sixel = lists_sixel(&self.bytes[3..self.bytes.len() - 1]);
                        self.take();
                        Fed::Reply(InternalEvent::PrimaryDeviceAttributes { sixel })
                    } else {
                        self.not_a_reply(pending)
                    }
                } else if self.bytes.len() > LONGEST_REPLY {
                    self.not_a_reply(pending)
                } else {
                    Fed::Held
                }
            }
            // ESC followed by anything else is no reply: keys as they came.
            _ => Fed::Keys(self.take()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PENDING: bool = true;
    const AFTER: bool = false;

    /// Feeds the characters of `text` as reply candidates, each record its
    /// index, and returns what the last one gave.
    fn feed_all(collector: &mut ReplyCollector<usize>, text: &str, pending: bool) -> Fed<usize> {
        let mut last = Fed::Held;
        for (index, character) in text.chars().enumerate() {
            last = collector.feed(index, Some(character), pending);
        }
        last
    }

    #[test]
    fn test_the_background_reply_is_collected_from_characters() {
        let mut collector = ReplyCollector::default();
        assert_eq!(
            feed_all(&mut collector, "\x1b]11;rgb:ffff/ffff/ffff\x1b\\", PENDING),
            Fed::Reply(InternalEvent::BackgroundColor(0xFFFF, 0xFFFF, 0xFFFF))
        );
        assert!(collector.is_empty());
        assert_eq!(
            feed_all(&mut collector, "\x1b]11;rgb:80/00/ff\x07", PENDING),
            Fed::Reply(InternalEvent::BackgroundColor(0x8080, 0, 0xFFFF))
        );
    }

    #[test]
    fn test_the_device_attributes_reply_ends_with_its_sixel_attribute() {
        let mut collector = ReplyCollector::default();
        assert_eq!(
            feed_all(&mut collector, "\x1b[?62;22c", PENDING),
            Fed::Reply(InternalEvent::PrimaryDeviceAttributes { sixel: false })
        );
        assert_eq!(
            feed_all(&mut collector, "\x1b[?62;4;22c", PENDING),
            Fed::Reply(InternalEvent::PrimaryDeviceAttributes { sixel: true })
        );
    }

    #[test]
    fn test_characters_before_the_end_are_held() {
        let mut collector = ReplyCollector::default();
        assert_eq!(
            feed_all(&mut collector, "\x1b]11;rgb:ff", PENDING),
            Fed::Held
        );
        assert!(!collector.is_empty());
        assert_eq!(
            feed_all(&mut collector, "ff/ffff/ffff\x1b\\", PENDING),
            Fed::Reply(InternalEvent::BackgroundColor(0xFFFF, 0xFFFF, 0xFFFF))
        );
    }

    /// The adversary's finding 2: a typed key says nothing about the records
    /// the terminal sent around it.
    #[test]
    fn test_a_typed_key_passes_through_and_the_held_reply_completes() {
        let mut collector: ReplyCollector<&str> = ReplyCollector::default();
        assert_eq!(collector.feed("x", None, PENDING), Fed::Keys(vec!["x"]));
        assert_eq!(
            collector.feed("a", Some('a'), PENDING),
            Fed::Keys(vec!["a"])
        );
        for (record, character) in [
            ("esc", '\x1b'),
            ("]", ']'),
            ("1", '1'),
            ("1", '1'),
            (";", ';'),
            ("r", 'r'),
            ("g", 'g'),
            ("b", 'b'),
            (":", ':'),
            ("f", 'f'),
        ] {
            assert_eq!(collector.feed(record, Some(character), PENDING), Fed::Held);
        }
        assert_eq!(collector.feed("y", None, PENDING), Fed::Keys(vec!["y"]));
        assert!(!collector.is_empty(), "the typed key released the reply");
        let mut last = Fed::Held;
        for (record, character) in [
            ("f", 'f'),
            ("f", 'f'),
            ("f", 'f'),
            ("/", '/'),
            ("f", 'f'),
            ("f", 'f'),
            ("f", 'f'),
            ("f", 'f'),
            ("/", '/'),
            ("f", 'f'),
            ("f", 'f'),
            ("f", 'f'),
            ("f", 'f'),
            ("esc", '\x1b'),
            ("\\", '\\'),
        ] {
            last = collector.feed(record, Some(character), PENDING);
        }
        assert_eq!(
            last,
            Fed::Reply(InternalEvent::BackgroundColor(0xFFFF, 0xFFFF, 0xFFFF))
        );
        assert!(collector.is_empty());
    }

    #[test]
    fn test_an_escape_followed_by_another_character_is_given_back() {
        for pending in [PENDING, AFTER] {
            let mut collector: ReplyCollector<&str> = ReplyCollector::default();
            assert_eq!(collector.feed("esc", Some('\x1b'), pending), Fed::Held);
            assert_eq!(
                collector.feed("x", Some('x'), pending),
                Fed::Keys(vec!["esc", "x"])
            );
        }
    }

    #[test]
    fn test_a_reply_to_a_question_not_asked_is_dropped_while_pending() {
        let mut collector = ReplyCollector::default();
        assert_eq!(
            feed_all(&mut collector, "\x1b[?0u", PENDING),
            Fed::Keys(Vec::new())
        );
        assert_eq!(
            feed_all(&mut collector, "\x1b]10;rgb:0/0/0\x07", PENDING),
            Fed::Keys(Vec::new())
        );
        assert_eq!(
            feed_all(&mut collector, "\x1b[1;1R", PENDING),
            Fed::Keys(Vec::new())
        );
        assert!(collector.is_empty());
    }

    /// The adversary's finding 1: after the exchange a reply is still a reply
    /// (the source consumes it), and any other sequence reads as it did
    /// before the collector existed.
    #[test]
    fn test_after_the_exchange_a_late_reply_is_a_reply_and_other_sequences_are_keys() {
        let mut collector = ReplyCollector::default();
        assert_eq!(
            feed_all(&mut collector, "\x1b]11;rgb:ffff/ffff/ffff\x1b\\", AFTER),
            Fed::Reply(InternalEvent::BackgroundColor(0xFFFF, 0xFFFF, 0xFFFF))
        );
        assert_eq!(
            feed_all(&mut collector, "\x1b[?62;22c", AFTER),
            Fed::Reply(InternalEvent::PrimaryDeviceAttributes { sixel: false })
        );
        assert_eq!(
            feed_all(&mut collector, "\x1b[200~", AFTER),
            Fed::Keys(vec![0, 1, 2, 3, 4, 5])
        );
        assert_eq!(
            feed_all(&mut collector, "\x1b]10;rgb:0/0/0\x07", AFTER),
            Fed::Keys((0..15).collect())
        );
        assert!(collector.is_empty());
    }

    #[test]
    fn test_a_reply_that_spans_the_end_of_the_exchange_is_still_assembled() {
        let mut collector = ReplyCollector::default();
        assert_eq!(
            feed_all(&mut collector, "\x1b]11;rgb:ff", PENDING),
            Fed::Held
        );
        assert_eq!(
            feed_all(&mut collector, "ff/ffff/ffff\x1b\\", AFTER),
            Fed::Reply(InternalEvent::BackgroundColor(0xFFFF, 0xFFFF, 0xFFFF))
        );
    }

    #[test]
    fn test_a_sequence_too_long_to_be_a_reply_is_dropped_or_given_back() {
        // An OSC string is given up after 64 bytes of body, as the Unix
        // reader does; a CSI sequence after LONGEST_REPLY bytes in all.
        let mut collector = ReplyCollector::default();
        let long = format!("\x1b]{}", "1".repeat(64));
        assert_eq!(feed_all(&mut collector, &long, PENDING), Fed::Held);
        assert_eq!(
            collector.feed(usize::MAX, Some('1'), PENDING),
            Fed::Keys(Vec::new())
        );
        assert!(collector.is_empty());
        let long = format!("\x1b[{}", "1".repeat(LONGEST_REPLY - 2));
        assert_eq!(feed_all(&mut collector, &long, AFTER), Fed::Held);
        let given_back = collector.feed(usize::MAX, Some('1'), AFTER);
        assert!(
            matches!(&given_back, Fed::Keys(keys) if keys.len() == LONGEST_REPLY + 1),
            "after the exchange a too-long sequence is given back: {given_back:?}"
        );
        assert!(collector.is_empty());
    }

    #[test]
    fn test_the_sixel_attribute_is_read_from_the_attributes() {
        assert!(lists_sixel(b"62;4;22"));
        assert!(lists_sixel(b"4"));
        assert!(!lists_sixel(b"62;22"));
        assert!(!lists_sixel(b"62;42"));
    }
}
