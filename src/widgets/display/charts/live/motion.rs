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
    /// The progress the current values were computed at, 0 to 1.
    t: f64,
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

    /// The running transition's progress and the values it started from, as
    /// of the last `values` call, so the renderer's automatic range can move
    /// with the values from the old range to the new one (CHT-022); `None`
    /// when no transition runs.
    pub(super) fn in_progress(&self) -> Option<(f64, Vec<Vec<f64>>)> {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.started.map(|_| (state.t, state.from.clone()))
    }

    fn at(&self, props: &ChartProps, target: &[Vec<f64>], now: Instant) -> (Vec<Vec<f64>>, bool) {
        // A target with a NaN or infinite value is shown as the error
        // message, not animated, and leaves the last valid rendering as the
        // start of the next transition (CHT-022, CHT-026).
        if target.iter().flatten().any(|v| !v.is_finite()) {
            self.ticker.drive(false, now);
            return (target.to_vec(), false);
        }
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
        state.t = t;
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

/// How long the hover marks of a plot picture take to ease in when the hover
/// begins, and the band to glide to another bar (CHT-038).
const HOVER_MOTION: Duration = Duration::from_millis(150);

/// What the hover marks of a plot picture show now (CHT-038).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct HoverFrame {
    pub series: usize,
    pub index: usize,
    /// 0 to 1: how far the dots, halo and fade have eased in.
    pub focus: f32,
    /// The band's center along the category axis, in cells, where the
    /// glide has it.
    pub band: f64,
}

#[derive(Default)]
struct HoverState {
    /// The hovered (series, index) the marks are for.
    target: Option<(usize, usize)>,
    /// The band center the glide started from, and the one it goes to.
    from: f64,
    to: f64,
    /// When the current motion began; none while nothing moves.
    started: Option<Instant>,
    /// Whether the motion eases the marks in (the hover began) rather than
    /// gliding the band (the hover moved to another datum).
    entering: bool,
    /// The band's center and the focus as of the last frame.
    band: f64,
    focus: f32,
}

/// The hover's motion: the marks ease in over [`HOVER_MOTION`] when the
/// hover begins, with the band on the datum at once; when it moves to
/// another datum the band glides there over the same time. Both snap under
/// `reduced-motion` (CHT-038).
pub(super) struct Hover {
    ticker: Ticker,
    state: Mutex<HoverState>,
}

impl Hover {
    pub(super) fn new() -> Self {
        Self {
            ticker: Ticker::new(),
            state: Mutex::new(HoverState::default()),
        }
    }

    /// The marks to draw now for `target`: the hovered series and index and
    /// its band's center in cells, or none when nothing is hovered.
    pub(super) fn frame(
        &self,
        props: &ChartProps,
        target: Option<(usize, usize, f64)>,
    ) -> Option<HoverFrame> {
        self.at(props, target, Instant::now())
    }

    fn at(
        &self,
        props: &ChartProps,
        target: Option<(usize, usize, f64)>,
        now: Instant,
    ) -> Option<HoverFrame> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let Some((series, index, band)) = target else {
            *state = HoverState::default();
            drop(state);
            self.ticker.drive(false, now);
            return None;
        };
        let instant = reduced_motion(props) || !self.ticker.available();
        let key = (series, index);
        if state.target != Some(key) {
            if state.target.is_none() {
                // The hover begins: the band is on the datum, the marks
                // ease in.
                state.from = band;
                state.to = band;
                state.band = band;
                state.focus = 0.0;
                state.entering = true;
            } else {
                // Another datum: the band glides from where it is.
                state.from = state.band;
                state.to = band;
                state.focus = 1.0;
                state.entering = false;
            }
            state.target = Some(key);
            state.started = (!instant).then_some(now);
        } else if state.to != band && state.started.is_none() {
            // The same datum at another place (a resize): there at once.
            state.from = band;
            state.to = band;
            state.band = band;
        }
        let progress = match state.started {
            Some(start) => (now.saturating_duration_since(start).as_secs_f64()
                / HOVER_MOTION.as_secs_f64())
            .min(1.0),
            None => 1.0,
        };
        // Eased out: quick at first, settling at the end.
        let eased = 1.0 - (1.0 - progress).powi(3);
        state.band = state.from + (state.to - state.from) * eased;
        state.focus = if state.entering { eased as f32 } else { 1.0 };
        if progress >= 1.0 {
            state.started = None;
            state.band = state.to;
            state.focus = 1.0;
        }
        let running = state.started.is_some();
        let frame = HoverFrame {
            series,
            index,
            focus: state.focus,
            band: state.band,
        };
        drop(state);
        self.ticker.drive(running, now);
        Some(frame)
    }

    pub(super) fn cancel(&self) {
        self.ticker.cancel();
    }
}

#[cfg(test)]
mod hover_tests {
    use super::*;

    fn props(class: Option<&str>) -> ChartProps {
        ChartProps {
            class: class.map(str::to_owned),
            ..Default::default()
        }
    }

    /// CHT-038: the band glides from the bar hovered before to the new one
    /// over 150 ms and the marks ease in when the hover begins; under
    /// reduced-motion both snap.
    #[test]
    fn cht_038_the_band_glides_and_the_marks_ease_in_unless_motion_is_reduced() {
        let hover = Hover::new();
        let start = Instant::now();
        // No scheduler in a unit test: the ticker is unavailable, so the
        // motion is instant. The reduced-motion rule is the same path.
        let first = hover.at(&props(None), Some((0, 1, 10.0)), start).unwrap();
        assert_eq!((first.focus, first.band), (1.0, 10.0));
        let moved = hover
            .at(&props(Some("reduced-motion")), Some((0, 4, 40.0)), start)
            .unwrap();
        assert_eq!((moved.focus, moved.band), (1.0, 40.0));
        assert!(hover.at(&props(None), None, start).is_none());
    }

    /// The glide's arithmetic: half way through the motion the band is most
    /// of the way there (eased out), and at the end exactly there.
    #[test]
    fn cht_038_the_glide_eases_out_and_ends_on_the_target() {
        let hover = Hover::new();
        let start = Instant::now();
        {
            let mut state = hover.state.lock().unwrap();
            state.target = Some((0, 1));
            state.from = 10.0;
            state.to = 40.0;
            state.band = 10.0;
            state.focus = 1.0;
            state.started = Some(start);
        }
        let halfway = hover
            .at(&props(None), Some((0, 1, 40.0)), start + HOVER_MOTION / 2)
            .unwrap();
        assert!(
            halfway.band > 25.0 && halfway.band < 40.0,
            "half way: {}",
            halfway.band
        );
        let done = hover
            .at(&props(None), Some((0, 1, 40.0)), start + HOVER_MOTION)
            .unwrap();
        assert_eq!(done.band, 40.0);
    }
}
