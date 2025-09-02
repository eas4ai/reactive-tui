use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Task to be executed by the scheduler
pub type SchedulerTask = Box<dyn FnOnce() + Send>;

/// Timer callback
pub type TimerCallback = Box<dyn FnMut() + Send>;

/// Handle for a scheduled timer
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TimerId(usize);

impl TimerId {
    /// Create a new unique timer ID
    fn new() -> Self {
        static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        Self(COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}

/// Timer entry in the scheduler
struct TimerEntry {
    /// Unique identifier for this timer
    id: TimerId,
    /// Callback function to execute
    callback: TimerCallback,
    /// Interval between executions
    interval: Duration,
    /// When this timer should next run
    next_run: Instant,
    /// Whether this timer repeats
    repeat: bool,
}

/// Scheduler for managing reactive updates and timers
pub struct Scheduler {
    /// Queue of pending updates
    update_queue: Arc<Mutex<VecDeque<SchedulerTask>>>,
    /// Active timers
    timers: Arc<Mutex<Vec<TimerEntry>>>,
    /// Flag to track if updates are pending
    has_updates: Arc<Mutex<bool>>,
}

impl Scheduler {
    /// Create a new scheduler
    pub fn new() -> Self {
        Self {
            update_queue: Arc::new(Mutex::new(VecDeque::new())),
            timers: Arc::new(Mutex::new(Vec::new())),
            has_updates: Arc::new(Mutex::new(false)),
        }
    }

    /// Schedule an update task
    pub fn schedule_update(&self, task: SchedulerTask) {
        let mut queue = self.update_queue.lock().unwrap();
        queue.push_back(task);
        *self.has_updates.lock().unwrap() = true;
    }

    /// Check if there are pending updates
    pub fn has_pending_updates(&self) -> bool {
        *self.has_updates.lock().unwrap()
    }

    /// Process all pending updates
    pub fn process_updates(&self) {
        let mut queue = self.update_queue.lock().unwrap();
        let mut updates = Vec::new();

        // Drain the queue
        while let Some(task) = queue.pop_front() {
            updates.push(task);
        }

        // Clear the flag
        *self.has_updates.lock().unwrap() = false;

        // Drop the lock before executing tasks
        drop(queue);

        // Execute all tasks
        for task in updates {
            task();
        }
    }

    /// Schedule a timer that runs at intervals
    pub fn schedule_interval<F>(&self, interval: Duration, callback: F) -> TimerId
    where
        F: FnMut() + Send + 'static,
    {
        let id = TimerId::new();
        let entry = TimerEntry {
            id,
            callback: Box::new(callback),
            interval,
            next_run: Instant::now() + interval,
            repeat: true,
        };

        self.timers.lock().unwrap().push(entry);
        id
    }

    /// Schedule a one-time timer
    pub fn schedule_timeout<F>(&self, delay: Duration, callback: F) -> TimerId
    where
        F: FnOnce() + Send + 'static,
    {
        let id = TimerId::new();

        // Wrap FnOnce in FnMut for storage
        let mut callback_opt = Some(callback);
        let entry = TimerEntry {
            id,
            callback: Box::new(move || {
                if let Some(cb) = callback_opt.take() {
                    cb();
                }
            }),
            interval: delay,
            next_run: Instant::now() + delay,
            repeat: false,
        };

        self.timers.lock().unwrap().push(entry);
        id
    }

    /// Cancel a timer
    pub fn cancel_timer(&self, id: TimerId) {
        let mut timers = self.timers.lock().unwrap();
        timers.retain(|timer| timer.id != id);
    }

    /// Process any timers that are ready to run
    pub fn process_timers(&self) {
        let now = Instant::now();
        let mut timers = self.timers.lock().unwrap();
        let mut indices_to_run = Vec::new();
        let mut completed_ids = Vec::new();

        // Find timers that are ready to run
        for (index, timer) in timers.iter().enumerate() {
            if timer.next_run <= now {
                indices_to_run.push(index);
                if !timer.repeat {
                    completed_ids.push(timer.id);
                }
            }
        }

        // Execute callbacks and update repeat timers
        for index in indices_to_run {
            let timer = &mut timers[index];
            (timer.callback)();
            if timer.repeat {
                timer.next_run = now + timer.interval;
            }
        }

        // Remove one-time timers that have completed
        timers.retain(|timer| !completed_ids.contains(&timer.id));
    }

    /// Clear all scheduled updates and timers
    pub fn clear(&self) {
        self.update_queue.lock().unwrap().clear();
        self.timers.lock().unwrap().clear();
        *self.has_updates.lock().unwrap() = false;
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
}
