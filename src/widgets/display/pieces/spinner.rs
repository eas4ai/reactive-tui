//! The spinner display piece (docs/spec/display-pieces.md), carrying DIS-002, DIS-003, DIS-004.
//!
//! A spinner shows work in progress as one cycling glyph from the icon
//! catalog, followed by its label. The screen reader hears the label once,
//! when the spinner appears; the frames are not announced.

use super::icon::{SPINNER_ASCII_FRAMES, SPINNER_FRAMES};
use crate::accessibility::{Live, Node, Role};
use crate::builder::core::span;
use crate::component::{Component, Element, LifecycleEvent, Props};
use crate::reactive::{
    component_scope,
    scheduler::{Scheduler, TimerId},
};
use crate::widgets::display::charts::glyph_support;
use std::{
    any::Any,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// How long one spinner frame shows: about ten frames a second (DIS-003).
pub(super) const FRAME: Duration = Duration::from_millis(100);

/// The props of a spinner: its label, the classes of its element and whether
/// it animates. Under `reduced-motion` in `class`, or when `animated` is
/// false, it stands still on its first frame and requests no frame.
#[derive(Clone, PartialEq)]
pub struct SpinnerProps {
    /// The status label a screen reader and the text beside the glyph read.
    pub label: String,
    /// The classes of the spinner's element, from the builder.
    pub class: String,
    /// Whether the spinner cycles its frames.
    pub animated: bool,
}

impl Props for SpinnerProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Whether the class string asks for reduced motion.
pub(super) fn reduced_motion(class: &str) -> bool {
    class.split_whitespace().any(|c| c == "reduced-motion")
}

/// The clock a motion piece reads its frames from (DIS-003): the time since
/// it was made, and one timer that asks the App for the next frame. While
/// the piece does not animate no timer is held, so the App idles.
pub(super) struct Clock {
    origin: Instant,
    scheduler: Option<Arc<Scheduler>>,
    /// The pending timer and when it fires.
    timer: Mutex<Option<(TimerId, Instant)>>,
}

impl Clock {
    /// A clock that starts now, in the component scope that is current.
    pub(super) fn new() -> Self {
        Self {
            origin: Instant::now(),
            scheduler: component_scope::current().map(|scope| scope.scheduler()),
            timer: Mutex::new(None),
        }
    }

    /// The time since the clock started. When `animate` is set, it also keeps
    /// one timer pending that fires after `tick`, so the next frame comes.
    pub(super) fn elapsed(&self, animate: bool, tick: Duration) -> Duration {
        let now = Instant::now();
        if let Some(scheduler) = &self.scheduler {
            let mut timer = self.timer.lock().unwrap_or_else(|e| e.into_inner());
            let due = timer.is_none_or(|(_, deadline)| deadline <= now);
            if !animate || due {
                if let Some((id, _)) = timer.take() {
                    scheduler.cancel_timer(id);
                }
            }
            if animate && due {
                let id = scheduler.schedule_timeout(tick, || {});
                *timer = Some((id, now + tick));
            }
        }
        now.saturating_duration_since(self.origin)
    }

    /// Release the pending timer, if any.
    pub(super) fn cancel(&self) {
        if let Some(scheduler) = &self.scheduler {
            if let Some((id, _)) = self.timer.lock().unwrap_or_else(|e| e.into_inner()).take() {
                scheduler.cancel_timer(id);
            }
        }
    }
}

impl Drop for Clock {
    fn drop(&mut self) {
        self.cancel();
    }
}

/// The spinner component: a status element that cycles the catalog's frames.
pub struct Spinner {
    clock: Clock,
}

impl Component for Spinner {
    type Props = SpinnerProps;
    type State = ();

    fn new(_props: Self::Props) -> Self {
        Self {
            clock: Clock::new(),
        }
    }

    fn render(&self, props: &Self::Props, _state: &Self::State) -> Element {
        let animate = props.animated && !reduced_motion(&props.class);
        let elapsed = self.clock.elapsed(animate, FRAME);
        let frames = if glyph_support() {
            SPINNER_FRAMES
        } else {
            SPINNER_ASCII_FRAMES
        };
        let index = if animate {
            (elapsed.as_millis() / FRAME.as_millis()) as usize % frames.len()
        } else {
            0
        };
        // The node is set once: its label and its live region. The frames
        // change only the painted text, so a screen reader hears it once.
        let mut node = Node::new(Role::Status);
        node.set_label(props.label.clone());
        node.set_live(Live::Polite);
        let mut classes = vec!["text-muted".to_string()];
        classes.push(props.class.clone());
        span()
            .class(&classes.join(" "))
            .text(&format!("{} {}", frames[index], props.label))
            .build()
            .with_accessibility(node)
    }

    fn on_lifecycle(&mut self, event: LifecycleEvent, _state: &mut Self::State) {
        if event == LifecycleEvent::Unmount {
            self.clock.cancel();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reactive::component_scope::ComponentScope;

    fn props(animated: bool, class: &str) -> SpinnerProps {
        SpinnerProps {
            label: "Saving".to_string(),
            class: class.to_string(),
            animated,
        }
    }

    #[test]
    fn dis_004_spinner_is_a_status_labelled_by_its_label_and_announced_once() {
        let props = props(true, "");
        let element = Spinner::new(props.clone()).render(&props, &());
        let node = element
            .metadata
            .accessibility
            .as_ref()
            .expect("an accessible node");
        assert_eq!(node.role(), Role::Status);
        assert_eq!(node.inner.label(), Some("Saving"));
        assert_eq!(node.inner.live(), Some(Live::Polite));
    }

    #[test]
    fn dis_003_spinner_requests_a_timer_only_while_it_animates() {
        let scheduler = Arc::new(Scheduler::new());
        let scope = ComponentScope::new(scheduler.clone());
        let _scope = scope.enter(true);

        // One mounted spinner, re-rendered with each set of props.
        let spinner = Spinner::new(props(true, ""));
        spinner.render(&props(true, ""), &());
        assert!(scheduler.next_deadline().is_some());

        spinner.render(&props(false, ""), &());
        assert!(scheduler.next_deadline().is_none());

        spinner.render(&props(true, "reduced-motion"), &());
        assert!(scheduler.next_deadline().is_none());
    }
}
