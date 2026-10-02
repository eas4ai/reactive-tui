/// Lifecycle events that components can respond to
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleEvent {
    /// Component is being mounted to the tree
    Mount,

    /// Component is being unmounted from the tree
    Unmount,

    /// Component received new props
    PropsChanged,

    /// Component's parent changed
    ParentChanged,

    /// Component gained focus
    Focus,

    /// Component lost focus
    Blur,

    /// Component became visible
    Show,

    /// Component became hidden
    Hide,
}

/// Lifecycle phase of a component
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecyclePhase {
    /// Component is not yet mounted
    Unmounted,

    /// Component is mounting
    Mounting,

    /// Component is mounted and active
    Mounted,

    /// Component is updating
    Updating,

    /// Component is unmounting
    Unmounting,
}

/// Tracks the lifecycle state of a component
pub struct Lifecycle {
    phase: LifecyclePhase,
    mount_count: usize,
    update_count: usize,
}

impl Default for Lifecycle {
    fn default() -> Self {
        Self {
            phase: LifecyclePhase::Unmounted,
            mount_count: 0,
            update_count: 0,
        }
    }
}

impl Lifecycle {
    /// Create a new lifecycle tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the current lifecycle phase
    pub fn phase(&self) -> LifecyclePhase {
        self.phase
    }

    /// Check if the component is currently mounted
    pub fn is_mounted(&self) -> bool {
        matches!(
            self.phase,
            LifecyclePhase::Mounted | LifecyclePhase::Updating
        )
    }

    /// Begin the mounting phase
    pub fn mount(&mut self) {
        self.phase = LifecyclePhase::Mounting;
        self.mount_count += 1;
    }

    /// Complete the mounting phase
    pub fn complete_mount(&mut self) {
        self.phase = LifecyclePhase::Mounted;
    }

    /// Begin an update cycle
    pub fn begin_update(&mut self) {
        if self.is_mounted() {
            self.phase = LifecyclePhase::Updating;
            self.update_count += 1;
        }
    }

    /// Complete an update cycle
    pub fn complete_update(&mut self) {
        if self.phase == LifecyclePhase::Updating {
            self.phase = LifecyclePhase::Mounted;
        }
    }

    /// Begin the unmounting phase
    pub fn unmount(&mut self) {
        self.phase = LifecyclePhase::Unmounting;
    }

    /// Complete the unmounting phase
    pub fn complete_unmount(&mut self) {
        self.phase = LifecyclePhase::Unmounted;
    }

    /// Get the number of times this component has been mounted
    pub fn mount_count(&self) -> usize {
        self.mount_count
    }

    /// Get the number of times this component has been updated
    pub fn update_count(&self) -> usize {
        self.update_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_creation() {
        let lifecycle = Lifecycle::new();
        assert_eq!(lifecycle.phase(), LifecyclePhase::Unmounted);
        assert!(!lifecycle.is_mounted());
        assert_eq!(lifecycle.mount_count(), 0);
        assert_eq!(lifecycle.update_count(), 0);
    }

    #[test]
    fn test_lifecycle_default() {
        let lifecycle = Lifecycle::default();
        assert_eq!(lifecycle.phase(), LifecyclePhase::Unmounted);
        assert!(!lifecycle.is_mounted());
        assert_eq!(lifecycle.mount_count(), 0);
        assert_eq!(lifecycle.update_count(), 0);
    }

    #[test]
    fn test_mount_lifecycle() {
        let mut lifecycle = Lifecycle::new();

        // Start mounting
        lifecycle.mount();
        assert_eq!(lifecycle.phase(), LifecyclePhase::Mounting);
        assert!(!lifecycle.is_mounted()); // Not yet fully mounted
        assert_eq!(lifecycle.mount_count(), 1);

        // Complete mounting
        lifecycle.complete_mount();
        assert_eq!(lifecycle.phase(), LifecyclePhase::Mounted);
        assert!(lifecycle.is_mounted());
    }

    #[test]
    fn test_update_lifecycle() {
        let mut lifecycle = Lifecycle::new();

        // Can't update when not mounted
        lifecycle.begin_update();
        assert_eq!(lifecycle.phase(), LifecyclePhase::Unmounted);
        assert_eq!(lifecycle.update_count(), 0);

        // Mount first
        lifecycle.mount();
        lifecycle.complete_mount();
        assert!(lifecycle.is_mounted());

        // Now can update
        lifecycle.begin_update();
        assert_eq!(lifecycle.phase(), LifecyclePhase::Updating);
        assert!(lifecycle.is_mounted()); // Still considered mounted during update
        assert_eq!(lifecycle.update_count(), 1);

        // Complete update
        lifecycle.complete_update();
        assert_eq!(lifecycle.phase(), LifecyclePhase::Mounted);
        assert!(lifecycle.is_mounted());
    }

    #[test]
    fn test_unmount_lifecycle() {
        let mut lifecycle = Lifecycle::new();

        // Mount first
        lifecycle.mount();
        lifecycle.complete_mount();
        assert!(lifecycle.is_mounted());

        // Start unmounting
        lifecycle.unmount();
        assert_eq!(lifecycle.phase(), LifecyclePhase::Unmounting);
        assert!(!lifecycle.is_mounted());

        // Complete unmounting
        lifecycle.complete_unmount();
        assert_eq!(lifecycle.phase(), LifecyclePhase::Unmounted);
        assert!(!lifecycle.is_mounted());
    }

    #[test]
    fn test_multiple_mount_cycles() {
        let mut lifecycle = Lifecycle::new();

        // First mount cycle
        lifecycle.mount();
        lifecycle.complete_mount();
        assert_eq!(lifecycle.mount_count(), 1);

        // Unmount
        lifecycle.unmount();
        lifecycle.complete_unmount();

        // Second mount cycle
        lifecycle.mount();
        lifecycle.complete_mount();
        assert_eq!(lifecycle.mount_count(), 2);
    }

    #[test]
    fn test_multiple_updates() {
        let mut lifecycle = Lifecycle::new();

        // Mount first
        lifecycle.mount();
        lifecycle.complete_mount();

        // Multiple update cycles
        for i in 1..=5 {
            lifecycle.begin_update();
            lifecycle.complete_update();
            assert_eq!(lifecycle.update_count(), i);
            assert_eq!(lifecycle.phase(), LifecyclePhase::Mounted);
        }
    }

    #[test]
    fn test_update_during_mounting() {
        let mut lifecycle = Lifecycle::new();

        // Start mounting but don't complete
        lifecycle.mount();
        assert_eq!(lifecycle.phase(), LifecyclePhase::Mounting);

        // Try to update during mounting - should not work
        lifecycle.begin_update();
        assert_eq!(lifecycle.phase(), LifecyclePhase::Mounting); // Should remain mounting
        assert_eq!(lifecycle.update_count(), 0);
    }

    #[test]
    fn test_lifecycle_events() {
        // Test that all lifecycle events are defined
        let events = [
            LifecycleEvent::Mount,
            LifecycleEvent::Unmount,
            LifecycleEvent::PropsChanged,
            LifecycleEvent::ParentChanged,
            LifecycleEvent::Focus,
            LifecycleEvent::Blur,
            LifecycleEvent::Show,
            LifecycleEvent::Hide,
        ];

        // Should be able to compare events
        assert_eq!(events[0], LifecycleEvent::Mount);
        assert_ne!(events[0], LifecycleEvent::Unmount);
    }

    #[test]
    fn test_lifecycle_phases() {
        // Test that all lifecycle phases are defined
        let phases = [
            LifecyclePhase::Unmounted,
            LifecyclePhase::Mounting,
            LifecyclePhase::Mounted,
            LifecyclePhase::Updating,
            LifecyclePhase::Unmounting,
        ];

        // Should be able to compare phases
        assert_eq!(phases[0], LifecyclePhase::Unmounted);
        assert_ne!(phases[0], LifecyclePhase::Mounted);
    }

    #[test]
    fn test_is_mounted_states() {
        let mut lifecycle = Lifecycle::new();

        // Unmounted - not mounted
        assert!(!lifecycle.is_mounted());

        // Mounting - not yet mounted
        lifecycle.mount();
        assert!(!lifecycle.is_mounted());

        // Mounted - is mounted
        lifecycle.complete_mount();
        assert!(lifecycle.is_mounted());

        // Updating - still mounted
        lifecycle.begin_update();
        assert!(lifecycle.is_mounted());

        // Back to mounted - still mounted
        lifecycle.complete_update();
        assert!(lifecycle.is_mounted());

        // Unmounting - no longer mounted
        lifecycle.unmount();
        assert!(!lifecycle.is_mounted());

        // Unmounted - not mounted
        lifecycle.complete_unmount();
        assert!(!lifecycle.is_mounted());
    }
}
