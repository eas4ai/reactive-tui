use super::wake::AppWaker;
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Task to be executed by the scheduler.
pub type SchedulerTask = Box<dyn FnOnce() + Send>;
/// Timer callback.
pub type TimerCallback = Box<dyn FnMut() + Send>;
/// Handle for a scheduled timer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TimerId(usize);
impl TimerId {
    fn new() -> Self {
        static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        Self(COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}
struct TimerEntry {
    id: TimerId,
    callback: TimerCallback,
    interval: Duration,
    next_run: Instant,
    repeat: bool,
}
#[derive(Default)]
struct Timers {
    entries: Vec<TimerEntry>,
    // Timers executing outside the lock; true means cancel after the current call.
    running: HashMap<TimerId, bool>,
}

/// Shared work queue and timers. Callbacks execute outside storage locks.
pub struct Scheduler {
    update_queue: Mutex<VecDeque<SchedulerTask>>,
    timers: Mutex<Timers>,
    wake: Mutex<Option<AppWaker>>,
}
impl Scheduler {
    /// Create an unattached scheduler.
    pub fn new() -> Self {
        Self {
            update_queue: Mutex::new(VecDeque::new()),
            timers: Mutex::new(Timers::default()),
            wake: Mutex::new(None),
        }
    }
    /// Attach notifications to one App. Replacing a live App attachment is rejected.
    pub(crate) fn attach(&self, wake: AppWaker) -> crate::error::Result<()> {
        let mut target = self.wake.lock().unwrap();
        if target.as_ref().is_some_and(|old| !old.is_closed()) {
            return Err(crate::error::ReactiveError::invalid_state(
                "scheduler already belongs to a live App",
            ));
        }
        *target = Some(wake);
        Ok(())
    }
    fn notify(&self) {
        let wake = self.wake.lock().unwrap().clone();
        if let Some(wake) = wake {
            wake.wake();
        }
    }
    /// Queue one update and wake an attached App.
    pub fn schedule_update(&self, task: SchedulerTask) {
        self.update_queue
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .push_back(task);
        self.notify();
    }
    /// Whether work is queued.
    pub fn has_pending_updates(&self) -> bool {
        !self
            .update_queue
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .is_empty()
    }
    /// Run the current batch. Work queued by callbacks remains for the next batch.
    pub fn process_updates(&self) {
        let updates =
            std::mem::take(&mut *self.update_queue.lock().unwrap_or_else(|p| p.into_inner()));
        for task in updates {
            task();
        }
    }
    /// Schedule a repeating timer. Zero intervals are eligible once per processing turn.
    pub fn schedule_interval<F>(&self, interval: Duration, callback: F) -> TimerId
    where
        F: FnMut() + Send + 'static,
    {
        self.add_timer(interval, true, Box::new(callback))
    }
    /// Schedule one callback after a delay.
    pub fn schedule_timeout<F>(&self, delay: Duration, callback: F) -> TimerId
    where
        F: FnOnce() + Send + 'static,
    {
        let mut callback = Some(callback);
        self.add_timer(
            delay,
            false,
            Box::new(move || {
                if let Some(callback) = callback.take() {
                    callback();
                }
            }),
        )
    }
    fn add_timer(&self, interval: Duration, repeat: bool, callback: TimerCallback) -> TimerId {
        let id = TimerId::new();
        let next_run = Instant::now()
            .checked_add(interval)
            .expect("timer delay exceeds the clock range");
        self.timers
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .entries
            .push(TimerEntry {
                id,
                callback,
                interval,
                next_run,
                repeat,
            });
        self.notify();
        id
    }
    /// Cancel a timer. A callback already running may finish but will not repeat.
    pub fn cancel_timer(&self, id: TimerId) {
        let removed = {
            let mut timers = self.timers.lock().unwrap_or_else(|p| p.into_inner());
            if let Some(cancelled) = timers.running.get_mut(&id) {
                *cancelled = true;
            }
            timers
                .entries
                .iter()
                .position(|timer| timer.id == id)
                .map(|index| timers.entries.remove(index))
        };
        drop(removed);
        self.notify();
    }
    /// Earliest scheduled callback, used to bound App's idle wait.
    pub fn next_deadline(&self) -> Option<Instant> {
        self.timers
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .entries
            .iter()
            .map(|t| t.next_run)
            .min()
    }
    /// Run ready timers without holding their storage lock.
    pub fn process_timers(&self) {
        self.run_ready_timers();
    }
    pub(crate) fn run_ready_timers(&self) -> bool {
        let now = Instant::now();
        let ready = {
            let mut timers = self.timers.lock().unwrap_or_else(|p| p.into_inner());
            let (ready, later): (Vec<_>, Vec<_>) = std::mem::take(&mut timers.entries)
                .into_iter()
                .partition(|t| t.next_run <= now);
            timers.entries = later;
            for timer in &ready {
                timers.running.insert(timer.id, false);
            }
            ready
        };
        let mut ran = false;
        for mut timer in ready {
            let cancelled = self
                .timers
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .running
                .get(&timer.id)
                .copied()
                .unwrap_or(true);
            if !cancelled {
                ran = true;
                (timer.callback)();
            }
            let mut timers = self.timers.lock().unwrap_or_else(|p| p.into_inner());
            if timers.running.remove(&timer.id) == Some(false) && timer.repeat {
                timer.next_run = Instant::now()
                    .checked_add(timer.interval)
                    .expect("timer interval exceeds the clock range");
                timers.entries.push(timer);
            }
        }
        ran
    }
    /// Discard queued work and cancel timers, including repeats currently executing.
    pub fn clear(&self) {
        let updates =
            std::mem::take(&mut *self.update_queue.lock().unwrap_or_else(|p| p.into_inner()));
        let entries = {
            let mut timers = self.timers.lock().unwrap_or_else(|p| p.into_inner());
            // Missing running entries are treated as cancelled by the runner.
            timers.running.clear();
            std::mem::take(&mut timers.entries)
        };
        drop((updates, entries));
        self.notify();
    }
}
impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_schedule_update() {
        let scheduler = Scheduler::new();
        let counter = Arc::new(AtomicUsize::new(0));

        let counter_clone = counter.clone();
        scheduler.schedule_update(Box::new(move || {
            counter_clone.fetch_add(1, Ordering::Relaxed);
        }));

        assert!(scheduler.has_pending_updates());
        scheduler.process_updates();
        assert!(!scheduler.has_pending_updates());
        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_schedule_timeout() {
        let scheduler = Scheduler::new();
        let counter = Arc::new(AtomicUsize::new(0));

        let counter_clone = counter.clone();
        scheduler.schedule_timeout(Duration::from_millis(0), move || {
            counter_clone.fetch_add(1, Ordering::Relaxed);
        });

        // Timer should fire immediately
        std::thread::sleep(Duration::from_millis(1));
        scheduler.process_timers();

        assert_eq!(counter.load(Ordering::Relaxed), 1);

        // Should not fire again
        scheduler.process_timers();
        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_schedule_interval() {
        let scheduler = Scheduler::new();
        let counter = Arc::new(AtomicUsize::new(0));

        let counter_clone = counter.clone();
        let id = scheduler.schedule_interval(Duration::from_millis(0), move || {
            counter_clone.fetch_add(1, Ordering::Relaxed);
        });

        // Should fire multiple times
        std::thread::sleep(Duration::from_millis(1));
        scheduler.process_timers();
        assert_eq!(counter.load(Ordering::Relaxed), 1);

        scheduler.process_timers();
        assert_eq!(counter.load(Ordering::Relaxed), 2);

        // Cancel the timer
        scheduler.cancel_timer(id);
        scheduler.process_timers();
        assert_eq!(counter.load(Ordering::Relaxed), 2);
    }
    #[test]
    fn timer_can_cancel_itself_and_schedule_more_work() {
        let scheduler = Arc::new(Scheduler::new());
        let timer_id = Arc::new(Mutex::new(None));
        let calls = Arc::new(AtomicUsize::new(0));
        let callback_scheduler = scheduler.clone();
        let callback_id = timer_id.clone();
        let callback_calls = calls.clone();
        let id = scheduler.schedule_interval(Duration::ZERO, move || {
            callback_calls.fetch_add(1, Ordering::Relaxed);
            callback_scheduler.cancel_timer(callback_id.lock().unwrap().unwrap());
            let calls = callback_calls.clone();
            callback_scheduler.schedule_timeout(Duration::ZERO, move || {
                calls.fetch_add(10, Ordering::Relaxed);
            });
        });
        *timer_id.lock().unwrap() = Some(id);
        scheduler.process_timers();
        scheduler.process_timers();
        scheduler.process_timers();
        assert_eq!(calls.load(Ordering::Relaxed), 11);
        assert!(scheduler.next_deadline().is_none());
    }
}
