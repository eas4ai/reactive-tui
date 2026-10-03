use super::{AccordionProps, AccordionState};
use crate::reactive::{
    component_scope,
    scheduler::{Scheduler, TimerId},
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

struct Transition {
    from: f32,
    target: f32,
    changed: Instant,
    delay: Duration,
}

#[derive(Default)]
struct MotionState {
    sections: HashMap<String, Transition>,
    timer: Option<(TimerId, Instant)>,
}

pub(super) struct AccordionMotion {
    scheduler: Option<Arc<Scheduler>>,
    state: Mutex<MotionState>,
}

impl AccordionMotion {
    pub(super) fn new() -> Self {
        Self {
            scheduler: component_scope::current().map(|s| s.scheduler()),
            state: Mutex::default(),
        }
    }

    pub(super) fn fractions(
        &self,
        props: &AccordionProps,
        expanded: &AccordionState,
    ) -> HashMap<String, f32> {
        self.at(props, expanded, Instant::now())
    }

    fn at(
        &self,
        props: &AccordionProps,
        expanded: &AccordionState,
        now: Instant,
    ) -> HashMap<String, f32> {
        let mut state = self.state.lock().unwrap();
        state
            .sections
            .retain(|id, _| props.sections.iter().any(|s| &s.id == id));
        let mut changed = 0u32;
        let mut active = false;
        let mut result = HashMap::new();
        for section in &props.sections {
            let target = if expanded.expanded_sections.get(&section.id) == Some(&true) {
                1.0
            } else {
                0.0
            };
            let transition = state
                .sections
                .entry(section.id.clone())
                .or_insert(Transition {
                    from: target,
                    target,
                    changed: now,
                    delay: Duration::ZERO,
                });
            if transition.target != target {
                transition.from = fraction(transition, props.animation.duration, now);
                transition.target = target;
                transition.changed = now;
                transition.delay = if props.animation.stagger {
                    props.animation.stagger_delay.saturating_mul(changed)
                } else {
                    Duration::ZERO
                };
                changed = changed.saturating_add(1);
            }
            if props.reduced_motion
                || props.animation.duration.is_zero()
                || self.scheduler.is_none()
            {
                transition.from = target;
            }
            let value = fraction(transition, props.animation.duration, now);
            active |= value != target;
            result.insert(section.id.clone(), value);
        }
        if let Some(scheduler) = &self.scheduler {
            if state.timer.is_some_and(|(_, deadline)| deadline <= now) || !active {
                if let Some((id, _)) = state.timer.take() {
                    scheduler.cancel_timer(id);
                }
            }
            if active && state.timer.is_none() {
                let interval = Duration::from_millis(16);
                state.timer = Some((scheduler.schedule_timeout(interval, || {}), now + interval));
            }
        }
        result
    }

    pub(super) fn cancel(&self) {
        if let (Some(scheduler), Some((id, _))) =
            (&self.scheduler, self.state.lock().unwrap().timer.take())
        {
            scheduler.cancel_timer(id);
        }
    }
}

impl Drop for AccordionMotion {
    fn drop(&mut self) {
        self.cancel();
    }
}

fn fraction(transition: &Transition, duration: Duration, now: Instant) -> f32 {
    let elapsed = now.saturating_duration_since(transition.changed);
    if transition.from == transition.target {
        return transition.target;
    }
    if elapsed < transition.delay {
        return transition.from;
    }
    let elapsed = elapsed.saturating_sub(transition.delay);
    if elapsed >= duration {
        return transition.target;
    }
    let t = (elapsed.as_secs_f64() / duration.as_secs_f64()) as f32;
    // Critically damped spring response, normalized to reach its endpoint at duration.
    let spring = |t: f32| 1.0 - (1.0 + 6.0 * t) * (-6.0 * t).exp();
    transition.from + (transition.target - transition.from) * (spring(t) / spring(1.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::layout::accordion::{AccordionAnimation, AccordionSection};

    #[test]
    fn spring_progress_stagger_reversal_and_cleanup_are_bounded() {
        let scheduler = Arc::new(Scheduler::new());
        let scope = component_scope::ComponentScope::new(scheduler.clone());
        let _scope = scope.enter(true);
        let motion = AccordionMotion::new();
        let props = AccordionProps {
            sections: vec![
                AccordionSection::new("a", "A"),
                AccordionSection::new("b", "B"),
            ],
            animation: AccordionAnimation {
                duration: Duration::from_millis(300),
                stagger: true,
                stagger_delay: Duration::from_millis(100),
            },
            ..Default::default()
        };
        let mut state = AccordionState::default();
        let now = Instant::now();
        motion.at(&props, &state, now);
        state
            .expanded_sections
            .extend([("a".into(), true), ("b".into(), true)]);
        assert_eq!(motion.at(&props, &state, now)["a"], 0.0);
        assert!(scheduler.next_deadline().is_some());
        let values = motion.at(&props, &state, now + Duration::from_millis(50));
        assert!(values["a"] > 0.0 && values["a"] < 1.0);
        assert_eq!(values["b"], 0.0);
        state.expanded_sections.insert("a".into(), false);
        let reversed = motion.at(&props, &state, now + Duration::from_millis(50));
        assert_eq!(reversed["a"], values["a"]);
        let values = motion.at(&props, &state, now + Duration::from_millis(450));
        assert_eq!((values["a"], values["b"]), (0.0, 1.0));
        assert!(scheduler.next_deadline().is_none());
        state.expanded_sections.insert("b".into(), false);
        motion.at(&props, &state, now + Duration::from_millis(460));
        assert!(scheduler.next_deadline().is_some());
        drop(motion);
        assert!(scheduler.next_deadline().is_none());
    }
}
