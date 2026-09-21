use super::ChartProps;
use crate::reactive::{
    component_scope,
    scheduler::{Scheduler, TimerId},
};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Default)]
struct State {
    started: Option<Instant>,
    timer: Option<(TimerId, Instant)>,
}
pub(super) struct Reveal {
    scheduler: Option<Arc<Scheduler>>,
    state: Mutex<State>,
}
impl Reveal {
    pub(super) fn new() -> Self {
        Self {
            scheduler: component_scope::current().map(|s| s.scheduler()),
            state: Mutex::default(),
        }
    }
    pub(super) fn restart(&self) {
        self.cancel();
        self.state.lock().unwrap().started = None;
    }
    pub(super) fn fraction(&self, props: &ChartProps, valid: bool) -> f64 {
        self.at(props, valid, Instant::now())
    }
    fn at(&self, props: &ChartProps, valid: bool, now: Instant) -> f64 {
        let mut state = self.state.lock().unwrap();
        let reduced = props
            .class
            .as_deref()
            .is_some_and(|s| s.split_whitespace().any(|s| s == "reduced-motion"));
        let progress = if !valid
            || !props.animated
            || reduced
            || props.animation_duration == 0
            || self.scheduler.is_none()
        {
            1.0
        } else {
            let started = *state.started.get_or_insert(now);
            (now.saturating_duration_since(started).as_secs_f64()
                / Duration::from_millis(props.animation_duration).as_secs_f64())
            .min(1.0)
        };
        if let Some(scheduler) = &self.scheduler {
            if progress == 1.0 || state.timer.is_some_and(|(_, deadline)| deadline <= now) {
                if let Some((id, _)) = state.timer.take() {
                    scheduler.cancel_timer(id);
                }
            }
            if progress < 1.0 && state.timer.is_none() {
                let interval = Duration::from_millis(16);
                state.timer = Some((scheduler.schedule_timeout(interval, || {}), now + interval));
            }
        }
        progress
    }
    pub(super) fn cancel(&self) {
        if let (Some(scheduler), Some((id, _))) =
            (&self.scheduler, self.state.lock().unwrap().timer.take())
        {
            scheduler.cancel_timer(id);
        }
    }
}
impl Drop for Reveal {
    fn drop(&mut self) {
        self.cancel();
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
}
