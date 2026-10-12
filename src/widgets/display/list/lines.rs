//! The lines a list shows (DAT-007, DAT-008): its items whose label holds the
//! filter's text, letters compared without case, each section's title
//! above the items of that section that follow one another, and the loading
//! row last. The cursor moves over the items alone, passing over the titles,
//! the disabled items and the loading row.

use super::ListItem;
use std::ops::Range;

/// One line of a list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Line {
    /// A section's title, above the item `item` and the items of its section
    /// that follow it.
    Title { item: usize },
    /// The item `item` of the props, the `place`th of the items shown
    /// (0-based), under the title on line `title` when it has a section.
    Item {
        item: usize,
        place: usize,
        title: Option<usize>,
    },
    /// The loading row, after the last item.
    Loading,
}

impl Line {
    /// The props' item the line is or stands above, for ordering: the
    /// lines are in the items' order.
    fn order(&self) -> usize {
        match *self {
            Line::Title { item } | Line::Item { item, .. } => item,
            Line::Loading => usize::MAX,
        }
    }
}

/// The lines a list shows for its items, its filter and its loading flag.
#[derive(Clone, Debug, Default)]
pub(super) struct Lines {
    pub(super) lines: Vec<Line>,
    /// How many items show.
    pub(super) shown: usize,
}

/// A letter as the filter compares it: without case.
fn fold(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

/// The character ranges of `label` that match `needle` (folded), each match
/// after the one before it; none for an empty needle.
pub(super) fn matches(label: &str, needle: &[char]) -> Vec<Range<usize>> {
    let mut found = Vec::new();
    if needle.is_empty() {
        return found;
    }
    let chars: Vec<char> = label.chars().map(fold).collect();
    let mut at = 0;
    while at + needle.len() <= chars.len() {
        if chars[at..at + needle.len()] == *needle {
            found.push(at..at + needle.len());
            at += needle.len();
        } else {
            at += 1;
        }
    }
    found
}

/// The filter's text as the labels are compared with it.
pub(super) fn needle(filter: &str) -> Vec<char> {
    filter.chars().map(fold).collect()
}

/// Whether `label` holds `needle`.
fn holds(label: &str, needle: &[char]) -> bool {
    if needle.is_empty() {
        return true;
    }
    let chars: Vec<char> = label.chars().map(fold).collect();
    chars.windows(needle.len()).any(|window| window == needle)
}

impl Lines {
    /// The lines of `items` whose label holds `filter`, under the titles of
    /// the sections that keep an item, with the loading row last while
    /// `loading`.
    pub(super) fn new(items: &[ListItem], filter: &str, loading: bool) -> Self {
        let needle = needle(filter);
        let mut lines = Vec::new();
        let mut shown = 0;
        let mut section: Option<(&str, usize)> = None;
        for (index, item) in items.iter().enumerate() {
            if !holds(&item.label, &needle) {
                continue;
            }
            let title = match item.section.as_deref() {
                None => {
                    section = None;
                    None
                }
                Some(name) => match section {
                    Some((current, line)) if current == name => Some(line),
                    _ => {
                        let line = lines.len();
                        lines.push(Line::Title { item: index });
                        section = Some((name, line));
                        Some(line)
                    }
                },
            };
            lines.push(Line::Item {
                item: index,
                place: shown,
                title,
            });
            shown += 1;
        }
        if loading {
            lines.push(Line::Loading);
        }
        Self { lines, shown }
    }

    /// How many lines there are.
    pub(super) fn len(&self) -> usize {
        self.lines.len()
    }

    /// The line `line`, if there is one.
    pub(super) fn get(&self, line: usize) -> Option<Line> {
        self.lines.get(line).copied()
    }

    /// The line that shows the props' item `item`, if it shows.
    pub(super) fn line_of(&self, item: usize) -> Option<usize> {
        let at = self.lines.partition_point(|line| line.order() < item);
        let at = match self.lines.get(at) {
            Some(Line::Title { .. }) => at + 1,
            _ => at,
        };
        matches!(self.lines.get(at), Some(Line::Item { item: shown, .. }) if *shown == item)
            .then_some(at)
    }

    /// The item of line `line` when the cursor can stand on it: an item
    /// that is not disabled.
    pub(super) fn reachable(&self, items: &[ListItem], line: usize) -> Option<usize> {
        match self.lines.get(line)? {
            Line::Item { item, .. } if !items[*item].disabled => Some(*item),
            _ => None,
        }
    }

    /// The first line at or after `from` the cursor can stand on, else the
    /// last one before it.
    pub(super) fn reachable_from(&self, items: &[ListItem], from: usize) -> Option<usize> {
        let from = from.min(self.len().saturating_sub(1));
        (from..self.len())
            .find(|line| self.reachable(items, *line).is_some())
            .or_else(|| {
                (0..from)
                    .rev()
                    .find(|line| self.reachable(items, *line).is_some())
            })
    }

    /// The last line at or before `from` the cursor can stand on, else the
    /// first one after it.
    pub(super) fn reachable_until(&self, items: &[ListItem], from: usize) -> Option<usize> {
        let from = from.min(self.len().saturating_sub(1));
        (0..=from)
            .rev()
            .find(|line| self.reachable(items, *line).is_some())
            .or_else(|| (from..self.len()).find(|line| self.reachable(items, *line).is_some()))
    }

    /// The next line after `line` the cursor can stand on, if any.
    pub(super) fn next(&self, items: &[ListItem], line: usize) -> Option<usize> {
        (line + 1..self.len()).find(|line| self.reachable(items, *line).is_some())
    }

    /// The last line before `line` the cursor can stand on, if any.
    pub(super) fn previous(&self, items: &[ListItem], line: usize) -> Option<usize> {
        (0..line.min(self.len()))
            .rev()
            .find(|line| self.reachable(items, *line).is_some())
    }

    /// The place among the items shown of the item on line `line`, or of
    /// the first item after it, for the lines in view.
    pub(super) fn place_at(&self, line: usize) -> usize {
        self.lines[line.min(self.len())..]
            .iter()
            .find_map(|line| match line {
                Line::Item { place, .. } => Some(*place),
                _ => None,
            })
            .unwrap_or(self.shown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// DAT-008's list: Apple, Banana, a section Citrus of Lemon and Lime,
    /// and a disabled Mango.
    fn fruits() -> Vec<ListItem> {
        vec![
            ListItem::new("apple", "Apple"),
            ListItem::new("banana", "Banana"),
            ListItem::new("lemon", "Lemon").section("Citrus"),
            ListItem::new("lime", "Lime").section("Citrus"),
            ListItem::new("mango", "Mango").disabled(true),
        ]
    }

    #[test]
    fn dat_008_a_section_has_one_title_above_its_items_and_the_cursor_passes_over_it() {
        let items = fruits();
        let lines = Lines::new(&items, "", true);
        assert_eq!(
            lines.lines,
            vec![
                Line::Item {
                    item: 0,
                    place: 0,
                    title: None
                },
                Line::Item {
                    item: 1,
                    place: 1,
                    title: None
                },
                Line::Title { item: 2 },
                Line::Item {
                    item: 2,
                    place: 2,
                    title: Some(2)
                },
                Line::Item {
                    item: 3,
                    place: 3,
                    title: Some(2)
                },
                Line::Item {
                    item: 4,
                    place: 4,
                    title: None
                },
                Line::Loading,
            ]
        );
        assert_eq!(lines.shown, 5);
        assert_eq!(lines.next(&items, 1), Some(3), "Down passes over the title");
        assert_eq!(lines.previous(&items, 3), Some(1), "Up passes back over it");
        assert_eq!(
            lines.next(&items, 4),
            None,
            "the disabled Mango is passed over"
        );
        assert_eq!(lines.reachable_until(&items, 6), Some(4), "End is Lime");
        assert_eq!(
            lines.line_of(2),
            Some(3),
            "Lemon is on the line after its title"
        );
        assert_eq!(lines.line_of(4), Some(5));
        assert_eq!(lines.place_at(2), 2, "a title stands above the third item");
    }

    #[test]
    fn dat_008_a_filter_keeps_the_items_that_hold_it_without_case() {
        let items = fruits();
        for filter in ["le", "LE", "lE"] {
            let lines = Lines::new(&items, filter, false);
            assert_eq!(
                lines.lines,
                vec![
                    Line::Item {
                        item: 0,
                        place: 0,
                        title: None
                    },
                    Line::Title { item: 2 },
                    Line::Item {
                        item: 2,
                        place: 1,
                        title: Some(1)
                    },
                ],
                "{filter} keeps Apple and, under Citrus, Lemon"
            );
            assert_eq!(lines.line_of(3), None, "Lime does not show");
        }
        assert_eq!(matches("Apple", &needle("LE")), vec![3..5]);
        assert_eq!(matches("Lemon level", &needle("le")), vec![0..2, 6..8]);
        assert!(matches("Apple", &needle("")).is_empty());
        assert_eq!(Lines::new(&items, "zz", false).shown, 0);
    }
}
