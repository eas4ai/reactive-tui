//! Chart motion: the reveal from zero to full when data first appears
//! (CHT-004) and the transition from the previous values to new ones
//! (CHT-022). Both drive frames through the scheduler's 16 ms timer and
//! both are skipped under `reduced-motion`.

use super::ChartProps;
use crate::reactive::{
    component_scope,
    scheduler::{Scheduler, TimerId},
};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

fn reduced_motion(props: &ChartProps) -> bool {
    props
        .class
        .as_deref()
        .is_some_and(|s| s.split_whitespace().any(|s| s == "reduced-motion"))
}

/// A 16 ms frame timer shared by the reveal and the transition.
struct Ticker {
    scheduler: Option<Arc<Scheduler>>,
    timer: Mutex<Option<(TimerId, Instant)>>,
}

impl Ticker {
    fn new() -> Self {
        Self {
            scheduler: component_scope::current().map(|s| s.scheduler()),
            timer: Mutex::new(None),
        }
    }

    fn available(&self) -> bool {
        self.scheduler.is_some()
    }

    /// Keep a timer pending while `running`, cancel it otherwise.
    fn drive(&self, running: bool, now: Instant) {
        let Some(scheduler) = &self.scheduler else {
            return;
        };
        let mut timer = self.timer.lock().unwrap_or_else(|e| e.into_inner());
        if !running || timer.is_some_and(|(_, deadline)| deadline <= now) {
            if let Some((id, _)) = timer.take() {
                scheduler.cancel_timer(id);
            }
        }
        if running && timer.is_none() {
            let interval = Duration::from_millis(16);
            *timer = Some((scheduler.schedule_timeout(interval, || {}), now + interval));
        }
    }

    fn cancel(&self) {
        if let (Some(scheduler), Some((id, _))) = (
            &self.scheduler,
            self.timer.lock().unwrap_or_else(|e| e.into_inner()).take(),
        ) {
            scheduler.cancel_timer(id);
        }
    }
}

impl Drop for Ticker {
    fn drop(&mut self) {
        self.cancel();
    }
}

/// The reveal: 0 to 1 over `animation_duration` when the chart first shows
/// valid data.
pub(super) struct Reveal {
    ticker: Ticker,
    /// When the reveal started, and whether it has finished. Once the data
    /// is fully shown, enabling animation later never replays the reveal.
    state: Mutex<(Option<Instant>, bool)>,
}

impl Reveal {
    pub(super) fn new() -> Self {
        Self {
            ticker: Ticker::new(),
            state: Mutex::new((None, false)),
        }
    }

    /// Start the reveal over from zero.
    pub(super) fn restart(&self) {
        self.ticker.cancel();
        *self.state.lock().unwrap_or_else(|e| e.into_inner()) = (None, false);
    }

    /// The reveal fraction now.
    pub(super) fn fraction(&self, props: &ChartProps, valid: bool) -> f64 {
        self.at(props, valid, Instant::now())
    }

    fn at(&self, props: &ChartProps, valid: bool, now: Instant) -> f64 {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let progress = if !valid || state.1 {
            1.0
        } else if !props.animated
            || reduced_motion(props)
            || props.animation_duration == 0
            || !self.ticker.available()
        {
            state.1 = true;
            1.0
        } else {
            let start = *state.0.get_or_insert(now);
            let progress = (now.saturating_duration_since(start).as_secs_f64()
                / Duration::from_millis(props.animation_duration).as_secs_f64())
            .min(1.0);
            if progress >= 1.0 {
                state.1 = true;
            }
            progress
        };
        drop(state);
        self.ticker.drive(progress < 1.0, now);
        progress
    }

    pub(super) fn cancel(&self) {
        self.ticker.cancel();
    }
}

#[derive(Default)]
struct TransitionState {
    from: Vec<Vec<f64>>,
    to: Vec<Vec<f64>>,
    /// What the chart last showed, the start of the next transition.
    current: Vec<Vec<f64>>,
    started: Option<Instant>,
}

/// The value transition: from the values last shown to the new target over
/// `transition_duration`, ending exactly at the target and never moving
/// away from the target on the way.
pub(super) struct Transition {
    ticker: Ticker,
    state: Mutex<TransitionState>,
}

impl Transition {
    pub(super) fn new() -> Self {
        Self {
            ticker: Ticker::new(),
            state: Mutex::new(TransitionState::default()),
        }
    }

    /// The values to show now for `target`, and whether a transition is
    /// still running. A target with a different shape (series count or
    /// lengths) is shown at once.
    pub(super) fn values(&self, props: &ChartProps, target: &[Vec<f64>]) -> (Vec<Vec<f64>>, bool) {
        self.at(props, target, Instant::now())
    }

    fn at(&self, props: &ChartProps, target: &[Vec<f64>], now: Instant) -> (Vec<Vec<f64>>, bool) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let same_shape = state.current.len() == target.len()
            && state
                .current
                .iter()
                .zip(target)
                .all(|(a, b)| a.len() == b.len());
        let duration = Duration::from_millis(props.transition_duration);
        let skip = !same_shape
            || state.current.is_empty()
            || reduced_motion(props)
            || props.transition_duration == 0
            || !self.ticker.available();
        if state.to != target {
            if skip {
                state.from = target.to_vec();
                state.started = None;
            } else {
                state.from = state.current.clone();
                state.started = Some(now);
            }
            state.to = target.to_vec();
        }
        let t = match state.started {
            Some(start) if !skip => (now.saturating_duration_since(start).as_secs_f64()
                / duration.as_secs_f64())
            .min(1.0),
            _ => 1.0,
        };
        let values: Vec<Vec<f64>> = if t >= 1.0 {
            state.started = None;
            target.to_vec()
        } else {
            state
                .from
                .iter()
                .zip(target)
                .map(|(a, b)| a.iter().zip(b).map(|(x, y)| x + (y - x) * t).collect())
                .collect()
        };
        state.current = values.clone();
        let running = t < 1.0;
        drop(state);
        self.ticker.drive(running, now);
        (values, running)
    }

    pub(super) fn cancel(&self) {
        self.ticker.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reveal_interpolates_stops_restarts_and_releases_deadlines() {
        let scheduler = Arc::new(Scheduler::new());
        let scope = component_scope::ComponentScope::new(scheduler.clone());
        let _scope = scope.enter(true);
        let reveal = Reveal::new();
        let mut props = ChartProps {
            animated: true,
            animation_duration: 400,
            ..Default::default()
        };
        let start = Instant::now();
        assert_eq!(reveal.at(&props, true, start), 0.0);
        assert!(scheduler.next_deadline().is_some());
        assert_eq!(
            reveal.at(&props, true, start + Duration::from_millis(100)),
            0.25
        );
        assert_eq!(
            reveal.at(&props, true, start + Duration::from_millis(400)),
            1.0
        );
        assert!(scheduler.next_deadline().is_none());
        reveal.restart();
        assert_eq!(reveal.at(&props, true, start), 0.0);
        props.class = Some("reduced-motion".into());
        assert_eq!(reveal.at(&props, true, start), 1.0);
        assert!(scheduler.next_deadline().is_none());
        props.class = None;
        reveal.restart();
        reveal.at(&props, true, start);
        drop(reveal);
        assert!(scheduler.next_deadline().is_none());
    }

    #[test]
    fn transition_moves_from_the_shown_values_to_the_target_and_ends_exactly() {
        let scheduler = Arc::new(Scheduler::new());
        let scope = component_scope::ComponentScope::new(scheduler.clone());
        let _scope = scope.enter(true);
        let transition = Transition::new();
        let props = ChartProps {
            transition_duration: 200,
            ..Default::default()
        };
        let start = Instant::now();
        // First data shows at once.
        let (first, running) = transition.at(&props, &[vec![5.0, 5.0]], start);
        assert_eq!(first, vec![vec![5.0, 5.0]]);
        assert!(!running);
        // A change animates from 5 to 9, starting when the target changes.
        let (begin, running) = transition.at(&props, &[vec![9.0, 9.0]], start);
        assert_eq!(begin, vec![vec![5.0, 5.0]]);
        assert!(running);
        let (mid, running) = transition.at(
            &props,
            &[vec![9.0, 9.0]],
            start + Duration::from_millis(100),
        );
        assert!(running);
        assert_eq!(mid, vec![vec![7.0, 7.0]]);
        assert!(scheduler.next_deadline().is_some());
        let (end, running) = transition.at(
            &props,
            &[vec![9.0, 9.0]],
            start + Duration::from_millis(400),
        );
        assert_eq!(end, vec![vec![9.0, 9.0]]);
        assert!(!running);
        assert!(scheduler.next_deadline().is_none());
        // A retarget mid-flight starts from what was shown, never from zero.
        transition.at(
            &props,
            &[vec![1.0, 1.0]],
            start + Duration::from_millis(400),
        );
        let (again, _) = transition.at(
            &props,
            &[vec![1.0, 1.0]],
            start + Duration::from_millis(500),
        );
        assert_eq!(again, vec![vec![5.0, 5.0]]);
        // A shape change snaps.
        let (snap, running) =
            transition.at(&props, &[vec![2.0]], start + Duration::from_millis(500));
        assert_eq!(snap, vec![vec![2.0]]);
        assert!(!running);
        // Reduced motion snaps too.
        let reduced = ChartProps {
            class: Some("reduced-motion".into()),
            ..props
        };
        let (r, running) =
            transition.at(&reduced, &[vec![8.0]], start + Duration::from_millis(500));
        assert_eq!(r, vec![vec![8.0]]);
        assert!(!running);
    }
}
