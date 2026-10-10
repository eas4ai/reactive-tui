//! The shimmer display piece (docs/spec/display-pieces.md), carrying DIS-001, DIS-003.
//!
//! A shimmer marks busy text: its cells are `text-muted`, and a band of three
//! `foreground` cells sweeps across them once about every two seconds. Under
//! reduced motion the text stays plain. It has no node of its own; its parent
//! carries the busy state while it shows.

use super::spinner::{reduced_motion, Clock};
use crate::builder::core::span;
use crate::component::{Component, Element, LifecycleEvent, Props};
use std::{any::Any, time::Duration};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// One sweep of the band across the text (DIS-003).
const CYCLE: Duration = Duration::from_secs(2);
/// How often the band is moved to its next cell.
const TICK: Duration = Duration::from_millis(100);
/// How many cells the bright band covers.
const BAND: usize = 3;
/// The color of the text outside the band.
const TEXT: &str = "text-muted";
/// The color of the cells inside the band.
const BRIGHT: &str = "text-foreground";

/// The props of a shimmer: its text, the classes of its element and whether
/// the band moves. Under `reduced-motion` in `class`, or when `animated` is
/// false, the text stays plain.
#[derive(Clone, PartialEq)]
pub struct ShimmerProps {
    /// The text the band sweeps across.
    pub text: String,
    /// The classes of the shimmer's element, from the builder.
    pub class: String,
    /// Whether the band moves.
    pub animated: bool,
}

impl Props for ShimmerProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// The grapheme clusters of `text`, each with its width in cells. A cluster
/// is one user-perceived character: a letter with its combining accent, or a
/// joined emoji, so it is never split between two runs (DIS-002, DIS-003).
fn clusters(text: &str) -> impl Iterator<Item = (&str, usize)> {
    text.graphemes(true)
        .map(|cluster| (cluster, UnicodeWidthStr::width(cluster)))
}

/// The width of `text` in cells.
fn text_cells(text: &str) -> usize {
    clusters(text).map(|(_, width)| width).sum()
}

/// The first cell of the band at `elapsed`, counted from the cell before
/// the text so the band enters from the left and leaves at the right. The
/// band is `BAND` cells wide and `length` is the text's width in cells.
fn band_start(elapsed: Duration, length: usize) -> isize {
    let phase = elapsed.as_millis() % CYCLE.as_millis();
    let travel = (length + BAND) as u128;
    let step = phase * travel / CYCLE.as_millis();
    step as isize - BAND as isize
}

/// The runs of `text` for a band starting at cell `band`, or no band. A
/// cluster is bright when the band reaches its first cell, so a cluster is
/// never split between two runs.
fn runs(text: &str, band: Option<isize>) -> Vec<(bool, String)> {
    let mut runs: Vec<(bool, String)> = Vec::new();
    let mut cell: isize = 0;
    for (cluster, width) in clusters(text) {
        let bright = band.is_some_and(|start| cell >= start && cell < start + BAND as isize);
        cell += width as isize;
        match runs.last_mut() {
            Some((lit, run)) if *lit == bright => run.push_str(cluster),
            _ => runs.push((bright, cluster.to_string())),
        }
    }
    runs
}

/// The shimmer component: its text in runs of muted and bright cells.
pub struct Shimmer {
    clock: Clock,
}

impl Component for Shimmer {
    type Props = ShimmerProps;
    type State = ();

    fn new(_props: Self::Props) -> Self {
        Self {
            clock: Clock::new(),
        }
    }

    fn render(&self, props: &Self::Props, _state: &Self::State) -> Element {
        let animate = props.animated && !reduced_motion(&props.class);
        let elapsed = self.clock.elapsed(animate, TICK);
        let band = animate.then(|| band_start(elapsed, text_cells(&props.text)));
        // One element per run of one color.
        let children = runs(&props.text, band)
            .into_iter()
            .map(|(bright, text)| {
                span()
                    .class(if bright { BRIGHT } else { TEXT })
                    .text(&text)
                    .build()
            })
            .collect::<Vec<_>>();
        span()
            .class(&props.class)
            .children(children)
            .build()
            .with_busy_parent()
    }

    fn on_lifecycle(&mut self, event: LifecycleEvent, _state: &mut Self::State) {
        if event == LifecycleEvent::Unmount {
            self.clock.cancel();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{band_start, runs, text_cells, CYCLE};
    use crate::builder::shimmer;
    use std::time::Duration;

    #[test]
    fn dis_004_shimmer_has_no_node_of_its_own() {
        let element = shimmer("Loading the report").build();
        assert!(element.metadata.accessibility.is_none());
    }

    #[test]
    fn dis_003_a_band_never_splits_a_grapheme_cluster() {
        // "e" with its combining accent is one cluster of one cell.
        let text = "e\u{301}X";
        let cells = text_cells(text);
        assert_eq!(cells, 2);
        for millis in (0..CYCLE.as_millis()).step_by(10) {
            let start = band_start(Duration::from_millis(millis as u64), cells);
            let frame = runs(text, Some(start));
            assert!(
                frame.iter().all(|(_, run)| !run.starts_with('\u{301}')),
                "a run starts with the accent at band {start}: {frame:?}"
            );
            assert!(
                frame.iter().any(|(_, run)| run.starts_with("e\u{301}")),
                "the accent is not in the run of its base at band {start}: {frame:?}"
            );
            let joined: String = frame.iter().map(|(_, run)| run.as_str()).collect();
            assert_eq!(joined, text, "the runs lose text at band {start}");
        }
    }

    #[test]
    fn dis_003_a_two_cell_emoji_is_bright_only_when_the_band_reaches_its_first_cell() {
        // The emoji takes cells 0 and 1 and "!" takes cell 2. A band starting
        // at cell 1 reaches the emoji's second cell only, so the emoji stays
        // plain and the "!" is bright.
        let text = "\u{1F44D}\u{1F3FD}!";
        assert_eq!(text_cells(text), 3);
        assert_eq!(
            runs(text, Some(1)),
            vec![
                (false, "\u{1F44D}\u{1F3FD}".to_string()),
                (true, "!".to_string()),
            ]
        );
        // A band starting at cell 0 reaches both cells of the emoji.
        assert_eq!(
            runs(text, Some(0)),
            vec![(true, "\u{1F44D}\u{1F3FD}!".to_string())]
        );
    }
}
