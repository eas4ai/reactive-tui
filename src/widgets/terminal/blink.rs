use crate::reactive::{
    component_scope,
    scheduler::{Scheduler, TimerId},
};
use crate::terminal::cursor::CursorShape;
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

const INTERVAL: Duration = Duration::from_millis(500);

pub(super) struct Blink {
    scheduler: Option<Arc<Scheduler>>,
    state: Mutex<State>,
}
struct State {
    activity: Option<Instant>,
    origin: Instant,
    timer: Option<TimerId>,
    deadline: Option<Instant>,
}
impl Blink {
    pub(super) fn new() -> Self {
        Self {
            scheduler: component_scope::current().map(|scope| scope.scheduler()),
            state: Mutex::new(State {
                activity: None,
                origin: Instant::now(),
                timer: None,
                deadline: None,
            }),
        }
    }
    pub(super) fn visible(&self, active: bool, shape: CursorShape, activity: Instant) -> bool {
        self.at(active, shape, activity, Instant::now())
    }
    fn at(&self, active: bool, shape: CursorShape, activity: Instant, now: Instant) -> bool {
        let blinking = matches!(
            shape,
            CursorShape::BlinkingBlock | CursorShape::BlinkingUnderline | CursorShape::BlinkingBar
        );
        if !active || !blinking || self.scheduler.is_none() {
            self.cancel();
            return active;
        }
        let scheduler = self.scheduler.as_ref().unwrap();
        let mut state = self.state.lock().unwrap();
        if state.activity != Some(activity) {
            state.activity = Some(activity);
            state.origin = now;
            if let Some(timer) = state.timer.take() {
                scheduler.cancel_timer(timer);
            }
            state.deadline = None;
        }
        let elapsed = now.saturating_duration_since(state.origin);
        let phase = elapsed.as_millis() / INTERVAL.as_millis();
        if state.deadline.is_none_or(|deadline| deadline <= now) {
            if let Some(timer) = state.timer.take() {
                scheduler.cancel_timer(timer);
            }
            let remaining =
                INTERVAL - Duration::from_nanos((elapsed.as_nanos() % INTERVAL.as_nanos()) as u64);
            state.timer = Some(scheduler.schedule_timeout(remaining, || {}));
            state.deadline = Some(now + remaining);
        }
        phase.is_multiple_of(2)
    }
    pub(super) fn cancel(&self) {
        let mut state = self.state.lock().unwrap();
        if let (Some(scheduler), Some(timer)) = (&self.scheduler, state.timer.take()) {
            scheduler.cancel_timer(timer);
        }
        state.activity = None;
        state.deadline = None;
    }
}
impl Drop for Blink {
    fn drop(&mut self) {
        self.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cursor_blink_follows_activity_and_releases_deadlines() {
        let scheduler = Arc::new(Scheduler::new());
        let scope = component_scope::ComponentScope::new(scheduler.clone());
        let _guard = scope.enter(true);
        let blink = Blink::new();
        let start = Instant::now();
        let shape = CursorShape::BlinkingBlock;
        assert!(blink.at(true, shape, start, start));
        assert!(scheduler.next_deadline().is_some());
        assert!(!blink.at(true, shape, start, start + INTERVAL));
        assert!(blink.at(true, shape, start, start + INTERVAL * 2));
        assert!(blink.at(true, shape, start + INTERVAL, start + INTERVAL * 3));
        assert!(!blink.at(false, shape, start, start + INTERVAL * 4));
        assert!(scheduler.next_deadline().is_none());
        assert!(blink.at(true, shape, start, start + INTERVAL * 5));
        assert!(blink.at(true, CursorShape::Block, start, start + INTERVAL * 6));
        assert!(scheduler.next_deadline().is_none());
        blink.at(true, shape, start, start + INTERVAL * 7);
        drop(blink);
        assert!(scheduler.next_deadline().is_none());
    }
}
