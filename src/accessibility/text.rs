//! Shared conversion from UTF-8 text and byte endpoints to reader text runs.
use super::{Node, Role};
use crate::component::Element;
use unicode_segmentation::UnicodeSegmentation;

pub(crate) fn append_text_runs(
    element: &mut Element,
    node: &mut Node,
    source: &str,
    mask: bool,
    selection: Option<[usize; 2]>,
) {
    let mut positions = [(0, 0); 2];
    let [anchor, focus] = selection.unwrap_or([0, 0]);
    let mut offset = 0;
    let mut value = String::new();
    // Keep a final empty run after a newline, including an empty field.
    let lines: Vec<_> = source
        .split_inclusive('\n')
        .chain((source.is_empty() || source.ends_with('\n')).then_some(""))
        .collect();
    for (index, line) in lines.iter().enumerate() {
        let mut text = String::new();
        let mut lengths = Vec::new();
        let mut boundaries = vec![offset];
        for (byte, grapheme) in line.grapheme_indices(true) {
            if mask {
                text.push('•');
                lengths.push(3);
                boundaries.push(offset + byte + grapheme.len());
            } else {
                text.push_str(grapheme);
                if let Ok(length) = u8::try_from(grapheme.len()) {
                    lengths.push(length);
                    boundaries.push(offset + byte + grapheme.len());
                } else {
                    // AccessKit stores lengths in u8. Preserve all text for
                    // unusually long clusters; editor caret endpoints still
                    // map to the outer cluster boundaries.
                    for (scalar_byte, scalar) in grapheme.char_indices() {
                        lengths.push(scalar.len_utf8() as u8);
                        boundaries.push(offset + byte + scalar_byte + scalar.len_utf8());
                    }
                }
            }
        }
        for (position, byte) in positions.iter_mut().zip([anchor, focus]) {
            let byte = byte.min(source.len());
            if byte >= offset && (byte < offset + line.len() || index + 1 == lines.len()) {
                *position = (
                    index,
                    boundaries
                        .partition_point(|boundary| *boundary <= byte)
                        .saturating_sub(1),
                );
            }
        }
        offset += line.len();
        value.push_str(&text);
        let mut run = Node::new(Role::TextRun);
        run.set_value(text);
        run.inner.set_character_lengths(lengths);
        run.inner
            .set_text_direction(accesskit::TextDirection::LeftToRight);
        element.children.push(
            Element::text("")
                .key(format!("reactive_tui.accessibility.text.{index}"))
                .class("sr-only")
                .with_accessibility(run),
        );
    }
    node.set_value(value);
    node.text_selection = selection.map(|_| positions);
}
