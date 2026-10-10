//! The skeleton display piece (docs/spec/display-pieces.md), carrying DIS-002, DIS-003, DIS-004.
//!
//! A skeleton holds the place of content still on its way: rows of `border`
//! that fill their parent's width and pulse between `border` and `surface`
//! over about two seconds. It has no node of its own; its parent carries the
//! busy state while it shows.

use super::spinner::{reduced_motion, Clock};
use crate::builder::core::div;
use crate::component::{Component, Element, LifecycleEvent, Props};
use std::{any::Any, time::Duration};

/// One full pulse: `border`, then `surface`, each for half of it (DIS-003).
const CYCLE: Duration = Duration::from_secs(2);
/// How often the pulse is checked for its next color.
const TICK: Duration = Duration::from_millis(100);
/// The fill of a skeleton row while it is bright.
const BRIGHT: &str = "bg-border";
/// The fill of a skeleton row while it is dim.
const DIM: &str = "bg-surface";

/// The props of a skeleton: its row count, the classes of its element and
/// whether it pulses. Under `reduced-motion` in `class`, or when `animated`
/// is false, it stands still in `border`.
#[derive(Clone, PartialEq)]
pub struct SkeletonProps {
    /// How many rows the skeleton shows.
    pub rows: u16,
    /// The classes of the skeleton's element, from the builder.
    pub class: String,
    /// Whether the skeleton pulses.
    pub animated: bool,
}

impl Props for SkeletonProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// The skeleton component: a column of full-width rows, one row apart.
pub struct Skeleton {
    clock: Clock,
}

impl Component for Skeleton {
    type Props = SkeletonProps;
    type State = ();

    fn new(_props: Self::Props) -> Self {
        Self {
            clock: Clock::new(),
        }
    }

    fn render(&self, props: &Self::Props, _state: &Self::State) -> Element {
        let animate = props.animated && !reduced_motion(&props.class);
        let elapsed = self.clock.elapsed(animate, TICK);
        let bright = !animate || elapsed.as_millis() % CYCLE.as_millis() < CYCLE.as_millis() / 2;
        let fill = if bright { BRIGHT } else { DIM };
        let mut children = Vec::new();
        for row in 0..props.rows {
            if row > 0 {
                children.push(div().class("h-1 w-full").build());
            }
            children.push(div().class(&format!("h-1 w-full {fill}")).build());
        }
        // The skeleton fills its parent's width unless the classes set one.
        let mut classes = vec!["flex-col".to_string()];
        if !props.class.split_whitespace().any(|c| c.starts_with("w-")) {
            classes.push("w-full".to_string());
        }
        classes.push(props.class.clone());
        div().class(&classes.join(" ")).children(children).build()
    }

    fn on_lifecycle(&mut self, event: LifecycleEvent, _state: &mut Self::State) {
        if event == LifecycleEvent::Unmount {
            self.clock.cancel();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::builder::skeleton;

    #[test]
    fn dis_004_skeleton_has_no_node_of_its_own() {
        let element = skeleton().build();
        assert!(element.metadata.accessibility.is_none());
    }
}
