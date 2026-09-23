use crate::reactive::{
    component_scope,
    scheduler::{Scheduler, TimerId},
};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

const DURATION: Duration = Duration::from_millis(120);

struct Motion {
    from: (usize, usize),
    target: (usize, usize),
    started: Instant,
    timer: Option<(TimerId, Instant)>,
}

pub(super) struct ScrollMotion {
    scheduler: Option<Arc<Scheduler>>,
    state: Mutex<Motion>,
}

impl ScrollMotion {
    pub(super) fn new() -> Self {
        Self {
            scheduler: component_scope::current().map(|scope| scope.scheduler()),
            state: Mutex::new(Motion {
                from: (0, 0),
                target: (0, 0),
                started: Instant::now(),
                timer: None,
            }),
        }
    }

    pub(super) fn position(&self, target: (usize, usize), smooth: bool) -> (usize, usize) {
        let mut state = self.state.lock().unwrap();
        let now = Instant::now();
        let current = interpolate(&state, now);
        if state.target != target {
            state.from = current;
            state.target = target;
            state.started = now;
        }
        if !smooth || self.scheduler.is_none() {
            state.from = target;
        }
        let position = interpolate(&state, now);
        if let Some(scheduler) = &self.scheduler {
            if state.timer.is_some_and(|(_, deadline)| now >= deadline) {
                if let Some((timer, _)) = state.timer.take() {
                    scheduler.cancel_timer(timer);
                }
            }
            if position != target && state.timer.is_none() {
                // Running scheduled work makes App redraw. The callback retains
                // no component or resource; unmount cancels its deadline.
                let interval = Duration::from_millis(16);
                state.timer = Some((scheduler.schedule_timeout(interval, || {}), now + interval));
            } else if position == target {
                if let Some((timer, _)) = state.timer.take() {
                    scheduler.cancel_timer(timer);
                }
            }
        }
        position
    }

    pub(super) fn cancel(&self) {
        let timer = self.state.lock().unwrap().timer.take();
        if let (Some(scheduler), Some((timer, _))) = (&self.scheduler, timer) {
            scheduler.cancel_timer(timer);
        }
    }
}

impl Drop for ScrollMotion {
    fn drop(&mut self) {
        self.cancel();
    }
}

fn interpolate(state: &Motion, now: Instant) -> (usize, usize) {
    let fraction =
        (now.duration_since(state.started).as_secs_f64() / DURATION.as_secs_f64()).min(1.0);
    let axis = |from: usize, to: usize| {
        (from as f64 + (to as f64 - from as f64) * fraction).round() as usize
    };
    (
        axis(state.from.0, state.target.0),
        axis(state.from.1, state.target.1),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animation_is_bounded_and_cancellation_releases_deadlines() {
        let scheduler = Arc::new(Scheduler::new());
        let scope = component_scope::ComponentScope::new(scheduler.clone());
        let _scope = scope.enter(true);
        let motion = ScrollMotion::new();
        assert_eq!(motion.position((20, 10), true), (0, 0));
        let deadline = scheduler
            .next_deadline()
            .expect("animation schedules a frame");
        motion.position((20, 10), true);
        assert_eq!(
            scheduler.next_deadline(),
            Some(deadline),
            "rendering must not postpone the frame"
        );
        {
            let mut state = motion.state.lock().unwrap();
            state.started = Instant::now() - DURATION;
        }
        assert_eq!(motion.position((20, 10), true), (20, 10));
        assert!(scheduler.next_deadline().is_none());
        motion.position((0, 0), true);
        assert!(scheduler.next_deadline().is_some());
        motion.cancel();
        assert!(scheduler.next_deadline().is_none());
        motion.position((0, 0), true);
        drop(motion);
        assert!(scheduler.next_deadline().is_none());
    }
}
