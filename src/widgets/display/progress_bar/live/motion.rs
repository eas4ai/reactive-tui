use super::*;
use crate::reactive::{
    component_scope,
    scheduler::{Scheduler, TimerId},
};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
const DURATION: Duration = Duration::from_millis(200);
struct State {
    origin: Instant,
    changed: Instant,
    from: f64,
    target: Option<f64>,
    timer: Option<(TimerId, Instant)>,
}
pub(super) struct Sample {
    pub fraction: f64,
    pub phase: f64,
    pub alpha: f32,
}
pub(super) struct Motion {
    scheduler: Option<Arc<Scheduler>>,
    state: Mutex<State>,
    offset: f64,
}
impl Motion {
    pub(super) fn new(seed: &ProgressBarState) -> Self {
        let now = Instant::now();
        Self {
            scheduler: component_scope::current().map(|s| s.scheduler()),
            state: Mutex::new(State {
                origin: now,
                changed: now,
                from: 0.0,
                target: None,
                timer: None,
            }),
            offset: if seed.indeterminate_position.is_finite() {
                (seed.indeterminate_position / 100.0).rem_euclid(1.0)
            } else {
                0.0
            },
        }
    }
    pub(super) fn sample(&self, props: &ProgressBarProps, valid: bool) -> Sample {
        self.at(props, valid, Instant::now())
    }
    fn at(&self, props: &ProgressBarProps, valid: bool, now: Instant) -> Sample {
        let mut state = self.state.lock().unwrap();
        let target = ProgressBar.calculate_percentage(props) / 100.0;
        let elapsed =
            now.saturating_duration_since(state.changed).as_secs_f64() / DURATION.as_secs_f64();
        let current = state.from + (state.target.unwrap_or(target) - state.from) * elapsed.min(1.0);
        if state.target.is_none() {
            state.from = target;
            state.target = Some(target);
            state.changed = now;
        } else if state.target != Some(target) {
            state.from = current;
            state.target = Some(target);
            state.changed = now;
        }
        let reduced = props
            .style
            .as_deref()
            .is_some_and(|s| s.split_whitespace().any(|s| s == "reduced-motion"));
        let animate = valid && !reduced && self.scheduler.is_some();
        let fraction = if animate && props.animated {
            let elapsed =
                now.saturating_duration_since(state.changed).as_secs_f64() / DURATION.as_secs_f64();
            state.from + (target - state.from) * elapsed.min(1.0)
        } else {
            state.from = target;
            target
        };
        let active = animate && (props.indeterminate || props.pulse || fraction != target);
        if let Some(scheduler) = &self.scheduler {
            if !active || state.timer.is_some_and(|(_, deadline)| deadline <= now) {
                if let Some((id, _)) = state.timer.take() {
                    scheduler.cancel_timer(id);
                }
            }
            if active && state.timer.is_none() {
                let interval = Duration::from_millis(16);
                state.timer = Some((scheduler.schedule_timeout(interval, || {}), now + interval));
            }
        }
        let phase = if animate {
            (now.saturating_duration_since(state.origin).as_secs_f64() / 1.2 + self.offset).fract()
        } else {
            0.5
        };
        let alpha = if animate && props.pulse {
            0.55 + 0.45 * ((phase * std::f64::consts::TAU).sin() as f32 + 1.0) / 2.0
        } else {
            1.0
        };
        Sample {
            fraction,
            phase,
            alpha,
        }
    }
    pub(super) fn cancel(&self) {
        if let (Some(scheduler), Some((id, _))) =
            (&self.scheduler, self.state.lock().unwrap().timer.take())
        {
            scheduler.cancel_timer(id);
        }
    }
}
impl Drop for Motion {
    fn drop(&mut self) {
        self.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn progress_clocks_interpolate_reverse_stop_and_release_owned_deadlines() {
        let scheduler = Arc::new(Scheduler::new());
        let scope = component_scope::ComponentScope::new(scheduler.clone());
        let _scope = scope.enter(true);
        let motion = Motion::new(&ProgressBarState::default());
        let start = Instant::now();
        let mut props = ProgressBarProps {
            value: 25.0,
            animated: true,
            ..Default::default()
        };
        assert_eq!(motion.at(&props, true, start).fraction, 0.25);
        assert!(scheduler.next_deadline().is_none());
        props.value = 100.0;
        assert_eq!(motion.at(&props, true, start).fraction, 0.25);
        assert!(scheduler.next_deadline().is_some());
        assert_eq!(
            motion
                .at(&props, true, start + Duration::from_millis(100))
                .fraction,
            0.625
        );
        props.value = 0.0;
        assert_eq!(
            motion
                .at(&props, true, start + Duration::from_millis(100))
                .fraction,
            0.625
        );
        assert_eq!(
            motion
                .at(&props, true, start + Duration::from_millis(200))
                .fraction,
            0.3125
        );
        assert_eq!(
            motion
                .at(&props, true, start + Duration::from_millis(300))
                .fraction,
            0.0
        );
        assert!(scheduler.next_deadline().is_none());
        props.indeterminate = true;
        motion.at(&props, true, start + Duration::from_millis(400));
        assert!(scheduler.next_deadline().is_some());
        props.style = Some("reduced-motion".into());
        let sample = motion.at(&props, true, start + Duration::from_millis(500));
        assert_eq!(sample.phase, 0.5);
        assert_eq!(sample.alpha, 1.0);
        assert!(scheduler.next_deadline().is_none());
        props.style = None;
        props.pulse = true;
        motion.at(&props, true, start + Duration::from_millis(600));
        assert!(scheduler.next_deadline().is_some());
        drop(motion);
        assert!(scheduler.next_deadline().is_none());
    }
}
