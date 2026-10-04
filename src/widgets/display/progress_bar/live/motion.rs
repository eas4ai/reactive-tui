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
    /// The frames an indeterminate bar's picture has glided for: its
    /// segment moves one frame's step a render, at the App's frame rate,
    /// so the picture of a given frame is always the same (PIX-005).
    frames: u64,
}
pub(super) struct Sample {
    pub fraction: f64,
    pub phase: f64,
    pub alpha: f32,
    /// How far along the track an indeterminate bar's pixel segment lies,
    /// 0 to 1: gliding from end to end and back over two seconds, or under
    /// `reduced-motion` stepping a quarter of the track once a second
    /// (PIX-005).
    #[cfg_attr(not(feature = "wgpu-graphics"), allow(dead_code))]
    pub glide: f64,
}
pub(super) struct Motion {
    scheduler: Option<Arc<Scheduler>>,
    state: Mutex<State>,
    offset: f64,
    /// The App's target frames a second: an indeterminate bar's picture
    /// glides from end to end and back over two seconds of them.
    fps: u64,
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
                frames: 0,
            }),
            fps: component_scope::lookup::<crate::hooks::perf_context::PerformanceContext>()
                .map_or(60, |context| u64::from(context.fps_state.get().target_fps))
                .max(1),
            offset: if seed.indeterminate_position.is_finite() {
                (seed.indeterminate_position / 100.0).rem_euclid(1.0)
            } else {
                0.0
            },
        }
    }
    /// The bar as it is now; `pixels` says whether its picture glides, so
    /// an indeterminate bar under `reduced-motion` still steps once a
    /// second (PIX-005).
    pub(super) fn sample(&self, props: &ProgressBarProps, valid: bool, pixels: bool) -> Sample {
        self.at(props, valid, pixels, Instant::now())
    }
    fn at(&self, props: &ProgressBarProps, valid: bool, pixels: bool, now: Instant) -> Sample {
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
        // An indeterminate bar's picture under `reduced-motion` steps once
        // a second, so the bar renders again then (PIX-005).
        let stepping =
            valid && reduced && pixels && props.indeterminate && self.scheduler.is_some();
        let interval = if active {
            Some(Duration::from_millis(16))
        } else if stepping {
            Some(Duration::from_secs(1))
        } else {
            None
        };
        if let Some(scheduler) = &self.scheduler {
            if interval.is_none() || state.timer.is_some_and(|(_, deadline)| deadline <= now) {
                if let Some((id, _)) = state.timer.take() {
                    scheduler.cancel_timer(id);
                }
            }
            if let (Some(interval), None) = (interval, state.timer) {
                state.timer = Some((scheduler.schedule_timeout(interval, || {}), now + interval));
            }
        }
        let since_origin = now.saturating_duration_since(state.origin).as_secs_f64();
        let phase = if animate {
            (since_origin / 1.2 + self.offset).fract()
        } else {
            0.5
        };
        let glide = if animate {
            let cycle = 2 * self.fps;
            let frame = state.frames % cycle;
            state.frames += 1;
            let t = (frame as f64 / cycle as f64 + self.offset).fract();
            if t < 0.5 {
                2.0 * t
            } else {
                2.0 - 2.0 * t
            }
        } else if stepping {
            (since_origin.floor() as u64 % 4) as f64 / 3.0
        } else {
            0.0
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
            glide,
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
        assert_eq!(motion.at(&props, true, false, start).fraction, 0.25);
        assert!(scheduler.next_deadline().is_none());
        props.value = 100.0;
        assert_eq!(motion.at(&props, true, false, start).fraction, 0.25);
        assert!(scheduler.next_deadline().is_some());
        assert_eq!(
            motion
                .at(&props, true, false, start + Duration::from_millis(100))
                .fraction,
            0.625
        );
        props.value = 0.0;
        assert_eq!(
            motion
                .at(&props, true, false, start + Duration::from_millis(100))
                .fraction,
            0.625
        );
        assert_eq!(
            motion
                .at(&props, true, false, start + Duration::from_millis(200))
                .fraction,
            0.3125
        );
        assert_eq!(
            motion
                .at(&props, true, false, start + Duration::from_millis(300))
                .fraction,
            0.0
        );
        assert!(scheduler.next_deadline().is_none());
        props.indeterminate = true;
        motion.at(&props, true, false, start + Duration::from_millis(400));
        assert!(scheduler.next_deadline().is_some());
        props.style = Some("reduced-motion".into());
        let sample = motion.at(&props, true, false, start + Duration::from_millis(500));
        assert_eq!(sample.phase, 0.5);
        assert_eq!(sample.alpha, 1.0);
        assert!(scheduler.next_deadline().is_none());
        props.style = None;
        props.pulse = true;
        motion.at(&props, true, false, start + Duration::from_millis(600));
        assert!(scheduler.next_deadline().is_some());
        drop(motion);
        assert!(scheduler.next_deadline().is_none());
    }
}
