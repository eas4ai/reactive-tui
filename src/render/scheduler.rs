use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Priority levels for render operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    /// Immediate priority - user input, animations
    Immediate = 3,
    /// High priority - visible UI updates
    High = 2,
    /// Normal priority - standard updates
    Normal = 1,
    /// Low priority - off-screen or deferred updates
    Low = 0,
}

/// Handle to a scheduled render operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScheduleHandle(usize);

impl ScheduleHandle {
    fn new() -> Self {
        static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        Self(COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}

/// A scheduled render task
#[derive(Clone)]
struct RenderTask {
    handle: ScheduleHandle,
    priority: Priority,
    callback: Arc<dyn Fn() + Send + Sync>,
    scheduled_at: Instant,
    deadline: Option<Instant>,
}

impl PartialEq for RenderTask {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

impl Eq for RenderTask {}

impl PartialOrd for RenderTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RenderTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => {
                // Earlier scheduled first
                other.scheduled_at.cmp(&self.scheduled_at)
            }
            other => other,
        }
    }
}

/// Frame budget management
struct FrameBudget {
    _target_fps: u32,
    frame_duration: Duration,
    last_frame_start: Option<Instant>,
    current_frame_start: Option<Instant>,
}

impl FrameBudget {
    fn new(target_fps: u32) -> Self {
        Self {
            _target_fps: target_fps,
            frame_duration: Duration::from_millis(1000 / target_fps as u64),
            last_frame_start: None,
            current_frame_start: None,
        }
    }

    /// Start a new frame
    fn start_frame(&mut self) -> Duration {
        let now = Instant::now();
        self.last_frame_start = self.current_frame_start;
        self.current_frame_start = Some(now);
        self.frame_duration
    }

    /// Check remaining time in current frame
    #[allow(dead_code)]
    fn remaining_budget(&self) -> Duration {
        if let Some(start) = self.current_frame_start {
            let elapsed = start.elapsed();
            if elapsed < self.frame_duration {
                self.frame_duration - elapsed
            } else {
                Duration::ZERO
            }
        } else {
            self.frame_duration
        }
    }

    /// Check if we have time for another task
    #[allow(dead_code)]
    fn has_budget(&self, estimated_duration: Duration) -> bool {
        self.remaining_budget() >= estimated_duration
    }
}

/// Render scheduler for managing update priorities and frame budgets
pub struct RenderScheduler {
    tasks: Arc<Mutex<BinaryHeap<RenderTask>>>,
    pending_handles: Arc<Mutex<HashMap<ScheduleHandle, Priority>>>,
    frame_budget: Arc<Mutex<FrameBudget>>,
    is_running: Arc<Mutex<bool>>,
    stats: Arc<Mutex<SchedulerStats>>,
}

#[derive(Debug, Default)]
struct SchedulerStats {
    total_tasks: usize,
    completed_tasks: usize,
    dropped_frames: usize,
    average_frame_time: Duration,
}

impl RenderScheduler {
    /// Create a new render scheduler with target FPS
    pub fn new(target_fps: u32) -> Self {
        Self {
            tasks: Arc::new(Mutex::new(BinaryHeap::new())),
            pending_handles: Arc::new(Mutex::new(HashMap::new())),
            frame_budget: Arc::new(Mutex::new(FrameBudget::new(target_fps))),
            is_running: Arc::new(Mutex::new(false)),
            stats: Arc::new(Mutex::new(SchedulerStats::default())),
        }
    }

    /// Schedule a render task
    pub fn schedule<F>(&self, priority: Priority, callback: F) -> ScheduleHandle
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.schedule_with_deadline(priority, callback, None)
    }

    /// Schedule a render task with a deadline
    pub fn schedule_with_deadline<F>(
        &self,
        priority: Priority,
        callback: F,
        deadline: Option<Duration>,
    ) -> ScheduleHandle
    where
        F: Fn() + Send + Sync + 'static,
    {
        let handle = ScheduleHandle::new();
        let task = RenderTask {
            handle,
            priority,
            callback: Arc::new(callback),
            scheduled_at: Instant::now(),
            deadline: deadline.map(|d| Instant::now() + d),
        };

        let mut tasks = self.tasks.lock().unwrap_or_else(|poisoned| {
            // Recover from poisoned lock by clearing the data
            poisoned.into_inner()
        });
        let mut pending = self
            .pending_handles
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        tasks.push(task);
        pending.insert(handle, priority);

        let mut stats = self
            .stats
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        stats.total_tasks += 1;

        handle
    }

    /// Cancel a scheduled task
    pub fn cancel(&self, handle: ScheduleHandle) -> bool {
        let mut pending = self
            .pending_handles
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        pending.remove(&handle).is_some()
    }

    /// Execute tasks for one frame
    pub fn execute_frame(&self) -> usize {
        let mut completed = 0;
        let frame_budget = {
            let mut budget = self
                .frame_budget
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            budget.start_frame()
        };

        let frame_start = Instant::now();
        let frame_deadline = frame_start + frame_budget;

        // Set running flag
        {
            let mut running = self
                .is_running
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if *running {
                return 0; // Already running
            }
            *running = true;
        }

        // Process tasks until frame budget exhausted
        loop {
            let task = {
                let mut tasks = self
                    .tasks
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let mut pending = self
                    .pending_handles
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());

                // Find next valid task
                let mut found_task = None;
                while let Some(task) = tasks.pop() {
                    if pending.remove(&task.handle).is_some() {
                        found_task = Some(task);
                        break;
                    }
                    // Task was cancelled, skip it
                }
                found_task
            };

            let Some(task) = task else {
                break; // No more tasks
            };

            // Check deadline
            if let Some(deadline) = task.deadline {
                if Instant::now() > deadline {
                    // Task missed deadline, skip it
                    continue;
                }
            }

            // Check frame budget
            if Instant::now() >= frame_deadline {
                // Out of time, reschedule task
                let mut tasks = self
                    .tasks
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let mut pending = self
                    .pending_handles
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                tasks.push(task.clone());
                pending.insert(task.handle, task.priority);

                let mut stats = self
                    .stats
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                stats.dropped_frames += 1;
                break;
            }

            // Execute task
            (task.callback)();
            completed += 1;

            let mut stats = self
                .stats
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            stats.completed_tasks += 1;
        }

        // Update stats
        {
            let mut stats = self
                .stats
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let frame_time = frame_start.elapsed();

            // Simple moving average
            if stats.average_frame_time == Duration::ZERO {
                stats.average_frame_time = frame_time;
            } else {
                stats.average_frame_time = (stats.average_frame_time + frame_time) / 2;
            }
        }

        // Clear running flag
        *self
            .is_running
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = false;

        completed
    }

    /// Check if there are pending tasks
    pub fn has_pending_tasks(&self) -> bool {
        !self
            .tasks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .is_empty()
    }

    /// Get the number of pending tasks
    pub fn pending_count(&self) -> usize {
        self.tasks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }

    /// Clear all pending tasks
    pub fn clear(&self) {
        self.tasks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
        self.pending_handles
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
    }

    /// Get scheduler statistics
    pub fn stats(&self) -> String {
        let stats = self
            .stats
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        format!(
            "Scheduler Stats: {} total, {} completed, {} dropped frames, {:?} avg frame time",
            stats.total_tasks,
            stats.completed_tasks,
            stats.dropped_frames,
            stats.average_frame_time
        )
    }
}

impl Default for RenderScheduler {
    fn default() -> Self {
        Self::new(60) // Default to 60 FPS
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

    #[test]
    fn test_priority_ordering() {
        let scheduler = RenderScheduler::new(60);
        let execution_order = Arc::new(Mutex::new(Vec::new()));

        // Schedule tasks in reverse priority order
        let order1 = execution_order.clone();
        scheduler.schedule(Priority::Low, move || {
            order1
                .lock()
                .expect("Test mutex should not be poisoned")
                .push("low");
        });

        let order2 = execution_order.clone();
        scheduler.schedule(Priority::Normal, move || {
            order2
                .lock()
                .expect("Test mutex should not be poisoned")
                .push("normal");
        });

        let order3 = execution_order.clone();
        scheduler.schedule(Priority::Immediate, move || {
            order3
                .lock()
                .expect("Test mutex should not be poisoned")
                .push("immediate");
        });

        // Execute all tasks in frame
        scheduler.execute_frame();

        // Check execution order: immediate should run first, then normal, then low
        let order = execution_order
            .lock()
            .expect("Test mutex should not be poisoned");
        assert_eq!(order[0], "immediate");
        assert_eq!(order[1], "normal");
        assert_eq!(order[2], "low");
    }

    #[test]
    fn test_cancel_task() {
        let scheduler = RenderScheduler::new(60);
        let executed = Arc::new(AtomicUsize::new(0));

        let executed_clone = executed.clone();
        let handle = scheduler.schedule(Priority::Normal, move || {
            executed_clone.fetch_add(1, AtomicOrdering::SeqCst);
        });

        // Cancel the task
        assert!(scheduler.cancel(handle));

        // Execute frame
        scheduler.execute_frame();

        // Task should not have executed
        assert_eq!(executed.load(AtomicOrdering::SeqCst), 0);
    }

    #[test]
    fn test_frame_budget() {
        let mut budget = FrameBudget::new(60);
        let frame_duration = budget.start_frame();

        assert_eq!(frame_duration, Duration::from_millis(16)); // ~60 FPS
        assert!(budget.has_budget(Duration::from_millis(10)));
        assert!(!budget.has_budget(Duration::from_millis(20)));
    }
}
