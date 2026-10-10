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

/// The first cell of the band at `elapsed`, counted from the cell before
/// the text so the band enters from the left and leaves at the right. The
/// band is `BAND` cells wide.
fn band_start(elapsed: Duration, length: usize) -> isize {
    let phase = elapsed.as_millis() % CYCLE.as_millis();
    let travel = (length + BAND) as u128;
    let step = phase * travel / CYCLE.as_millis();
    step as isize - BAND as isize
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
        let cells: Vec<char> = props.text.chars().collect();
        let start = if animate {
            band_start(elapsed, cells.len())
        } else {
            0
        };
        // Group the cells into runs of one color, one element a run.
        let mut runs: Vec<(bool, String)> = Vec::new();
        for (index, cell) in cells.iter().enumerate() {
            let index = index as isize;
            let bright = animate && index >= start && index < start + BAND as isize;
            match runs.last_mut() {
                Some((lit, text)) if *lit == bright => text.push(*cell),
                _ => runs.push((bright, cell.to_string())),
            }
        }
        let children = runs
            .into_iter()
            .map(|(bright, text)| {
                span()
                    .class(if bright { BRIGHT } else { TEXT })
                    .text(&text)
                    .build()
            })
            .collect::<Vec<_>>();
        span().class(&props.class).children(children).build()
    }

    fn on_lifecycle(&mut self, event: LifecycleEvent, _state: &mut Self::State) {
        if event == LifecycleEvent::Unmount {
            self.clock.cancel();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::builder::shimmer;

    #[test]
    fn dis_004_shimmer_has_no_node_of_its_own() {
        let element = shimmer("Loading the report").build();
        assert!(element.metadata.accessibility.is_none());
    }
}
