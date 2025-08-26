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
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn phase(&self) -> LifecyclePhase {
        self.phase
    }
    
    pub fn is_mounted(&self) -> bool {
        matches!(self.phase, LifecyclePhase::Mounted | LifecyclePhase::Updating)
    }
    
    pub fn mount(&mut self) {
        self.phase = LifecyclePhase::Mounting;
        self.mount_count += 1;
    }
    
    pub fn complete_mount(&mut self) {
        self.phase = LifecyclePhase::Mounted;
    }
    
    pub fn begin_update(&mut self) {
        if self.is_mounted() {
            self.phase = LifecyclePhase::Updating;
            self.update_count += 1;
        }
    }
    
    pub fn complete_update(&mut self) {
        if self.phase == LifecyclePhase::Updating {
            self.phase = LifecyclePhase::Mounted;
        }
    }
    
    pub fn unmount(&mut self) {
        self.phase = LifecyclePhase::Unmounting;
    }
    
    pub fn complete_unmount(&mut self) {
        self.phase = LifecyclePhase::Unmounted;
    }
    
    pub fn mount_count(&self) -> usize {
        self.mount_count
    }
    
    pub fn update_count(&self) -> usize {
        self.update_count
    }
}