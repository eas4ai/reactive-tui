//! Lock-free animation state management to prevent deadlocks
//!
//! This module provides a lock-free alternative to the RwLock-based
//! animation state management to eliminate deadlock potential.

use super::{AnimationState, EasingFunction, LoopMode};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Lock-free animation state using atomic operations
#[derive(Debug)]
pub struct LockFreeAnimationState {
    /// Current animation progress (0.0 to 1.0) as fixed-point integer
    /// Stored as u64 where value = progress * 1_000_000_000
    progress: AtomicU64,

    /// Current time in microseconds
    current_time_micros: AtomicU64,

    /// Animation state packed into atomic u64
    /// Bits 0-7: AnimationState enum
    /// Bits 8-15: loops_completed (max 255)
    /// Bit 16: is_reversed
    /// Bit 17: is_paused
    /// Bits 18-63: reserved
    state_packed: AtomicU64,

    /// Last update timestamp in microseconds since epoch
    last_update_micros: AtomicU64,

    /// Animation completed flag for fast checking
    is_completed: AtomicBool,
}

impl LockFreeAnimationState {
    const PROGRESS_SCALE: u64 = 1_000_000_000;

    /// Create a new lock-free animation state
    pub fn new() -> Self {
        Self {
            progress: AtomicU64::new(0),
            current_time_micros: AtomicU64::new(0),
            state_packed: AtomicU64::new(AnimationState::Stopped as u64),
            last_update_micros: AtomicU64::new(0),
            is_completed: AtomicBool::new(false),
        }
    }

    /// Get current progress (0.0 to 1.0)
    pub fn get_progress(&self) -> f32 {
        let raw = self.progress.load(Ordering::Acquire);
        (raw as f64 / Self::PROGRESS_SCALE as f64) as f32
    }

    /// Set progress (0.0 to 1.0)
    pub fn set_progress(&self, progress: f32) {
        let scaled = (progress.clamp(0.0, 1.0) as f64 * Self::PROGRESS_SCALE as f64) as u64;
        self.progress.store(scaled, Ordering::Release);
    }

    /// Get current time as Duration
    pub fn get_current_time(&self) -> Duration {
        let micros = self.current_time_micros.load(Ordering::Acquire);
        Duration::from_micros(micros)
    }

    /// Set current time
    pub fn set_current_time(&self, time: Duration) {
        let micros = time.as_micros().min(u64::MAX as u128) as u64;
        self.current_time_micros.store(micros, Ordering::Release);
    }

    /// Get animation state
    pub fn get_state(&self) -> AnimationState {
        let packed = self.state_packed.load(Ordering::Acquire);
        let state_value = (packed & 0xFF) as u8;

        match state_value {
            0 => AnimationState::Stopped,
            1 => AnimationState::Playing,
            2 => AnimationState::Paused,
            3 => AnimationState::Completed,
            _ => AnimationState::Stopped, // Default for invalid values
        }
    }

    /// Set animation state
    pub fn set_state(&self, state: AnimationState) {
        let mut packed = self.state_packed.load(Ordering::Acquire);
        packed = (packed & !0xFF) | (state as u64);
        self.state_packed.store(packed, Ordering::Release);

        // Update completed flag for fast checking
        if matches!(state, AnimationState::Completed) {
            self.is_completed.store(true, Ordering::Release);
        } else {
            self.is_completed.store(false, Ordering::Release);
        }
    }

    /// Get loops completed count
    pub fn get_loops_completed(&self) -> u32 {
        let packed = self.state_packed.load(Ordering::Acquire);
        ((packed >> 8) & 0xFF) as u32
    }

    /// Increment loops completed
    pub fn increment_loops(&self) {
        loop {
            let current = self.state_packed.load(Ordering::Acquire);
            let loops = ((current >> 8) & 0xFF) as u32;
            if loops >= 255 {
                break; // Max loops reached
            }
            let new_loops = loops + 1;
            let new_packed = (current & !(0xFF << 8)) | ((new_loops as u64) << 8);

            if self
                .state_packed
                .compare_exchange_weak(current, new_packed, Ordering::Release, Ordering::Acquire)
                .is_ok()
            {
                break;
            }
        }
    }

    /// Check if animation is reversed
    pub fn is_reversed(&self) -> bool {
        let packed = self.state_packed.load(Ordering::Acquire);
        (packed & (1 << 16)) != 0
    }

    /// Toggle reversed state
    pub fn toggle_reversed(&self) {
        loop {
            let current = self.state_packed.load(Ordering::Acquire);
            let new_packed = current ^ (1 << 16); // Toggle bit 16

            if self
                .state_packed
                .compare_exchange_weak(current, new_packed, Ordering::Release, Ordering::Acquire)
                .is_ok()
            {
                break;
            }
        }
    }

    /// Check if animation is paused
    pub fn is_paused(&self) -> bool {
        let packed = self.state_packed.load(Ordering::Acquire);
        (packed & (1 << 17)) != 0
    }

    /// Set paused state
    pub fn set_paused(&self, paused: bool) {
        loop {
            let current = self.state_packed.load(Ordering::Acquire);
            let new_packed = if paused {
                current | (1 << 17)
            } else {
                current & !(1 << 17)
            };

            if self
                .state_packed
                .compare_exchange_weak(current, new_packed, Ordering::Release, Ordering::Acquire)
                .is_ok()
            {
                break;
            }
        }
    }

    /// Check if animation is completed (fast path)
    pub fn is_completed(&self) -> bool {
        self.is_completed.load(Ordering::Acquire)
    }

    /// Reset animation state
    pub fn reset(&self) {
        self.progress.store(0, Ordering::Release);
        self.current_time_micros.store(0, Ordering::Release);
        self.state_packed
            .store(AnimationState::Stopped as u64, Ordering::Release);
        self.last_update_micros.store(0, Ordering::Release);
        self.is_completed.store(false, Ordering::Release);
    }
}

impl Default for LockFreeAnimationState {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe animation update without locks
pub struct LockFreeAnimationUpdater {
    state: Arc<LockFreeAnimationState>,
    duration: Duration,
    easing: EasingFunction,
    loop_mode: LoopMode,
}

impl LockFreeAnimationUpdater {
    /// Create a new lock-free animation updater
    pub fn new(
        state: Arc<LockFreeAnimationState>,
        duration: Duration,
        easing: EasingFunction,
        loop_mode: LoopMode,
    ) -> Self {
        Self {
            state,
            duration,
            easing,
            loop_mode,
        }
    }

    /// Update animation state without locks
    /// Returns true if animation should continue, false if completed
    pub fn update(&self, delta_time: Duration) -> bool {
        // Fast path: check if already completed
        if self.state.is_completed() {
            return false;
        }

        // Update current time
        let current_time = self.state.get_current_time() + delta_time;
        self.state.set_current_time(current_time);

        // Calculate progress
        let raw_progress = if self.duration.as_millis() > 0 {
            current_time.as_millis() as f32 / self.duration.as_millis() as f32
        } else {
            1.0
        };

        // Handle completion and looping
        if raw_progress >= 1.0 {
            match self.loop_mode {
                LoopMode::None => {
                    self.state.set_progress(1.0);
                    self.state.set_state(AnimationState::Completed);
                    return false;
                }
                LoopMode::Infinite => {
                    self.state.set_current_time(Duration::ZERO);
                    self.state.set_progress(0.0);
                    self.state.increment_loops();
                }
                LoopMode::Count(count) => {
                    let loops = self.state.get_loops_completed();
                    if loops < count - 1 {
                        self.state.set_current_time(Duration::ZERO);
                        self.state.set_progress(0.0);
                        self.state.increment_loops();
                    } else {
                        self.state.set_progress(1.0);
                        self.state.set_state(AnimationState::Completed);
                        return false;
                    }
                }
                LoopMode::PingPong => {
                    self.state.toggle_reversed();
                    self.state.set_current_time(Duration::ZERO);
                    self.state.increment_loops();
                }
            }
        } else {
            // Apply easing and update progress
            let eased_progress = self.easing.apply(raw_progress.clamp(0.0, 1.0));
            let final_progress = if self.state.is_reversed() {
                1.0 - eased_progress
            } else {
                eased_progress
            };

            self.state.set_progress(final_progress);
            self.state.set_state(AnimationState::Playing);
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_free_animation_state() {
        let state = LockFreeAnimationState::new();

        // Test initial state
        assert_eq!(state.get_progress(), 0.0);
        assert_eq!(state.get_state(), AnimationState::Stopped);
        assert!(!state.is_completed());

        // Test progress update
        state.set_progress(0.5);
        assert!((state.get_progress() - 0.5).abs() < 0.001);

        // Test state change
        state.set_state(AnimationState::Playing);
        assert_eq!(state.get_state(), AnimationState::Playing);

        // Test completion
        state.set_state(AnimationState::Completed);
        assert!(state.is_completed());
    }

    #[test]
    fn test_concurrent_updates() {
        use std::thread;

        let state = Arc::new(LockFreeAnimationState::new());
        let mut handles = vec![];

        // Spawn multiple threads updating the state
        for i in 0..10 {
            let state_clone = state.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..1000 {
                    state_clone.set_progress((i as f32) / 10.0);
                    state_clone.increment_loops();
                }
            }));
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // State should be consistent (no crashes or corruption)
        assert!(state.get_progress() >= 0.0 && state.get_progress() <= 1.0);
        assert!(state.get_loops_completed() > 0);
    }
}
