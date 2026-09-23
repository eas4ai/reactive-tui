use bitflags::bitflags;

bitflags! {
    /// State flags for component state management
    ///
    /// These flags provide efficient tracking of component state
    /// using bit manipulation instead of separate boolean fields.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct StateFlags: u32 {
        /// Component is visible
        const VISIBLE = 0b00000001;
        /// Component is enabled and can receive input
        const ENABLED = 0b00000010;
        /// Component has keyboard focus
        const FOCUSED = 0b00000100;
        /// Mouse is hovering over component
        const HOVERED = 0b00001000;
        /// Component state has changed and needs re-render
        const DIRTY = 0b00010000;
        /// Component is currently animating
        const ANIMATING = 0b00100000;
        /// Component is selected (for list items, etc)
        const SELECTED = 0b01000000;
        /// Component is active (being interacted with)
        const ACTIVE = 0b10000000;

        // Additional common states
        /// Component is in a pressed state
        const PRESSED = 0b0000000100000000;
        /// Component is checked (for checkboxes, radio buttons)
        const CHECKED = 0b0000001000000000;
        /// Component is expanded (for trees, accordions)
        const EXPANDED = 0b0000010000000000;
        /// Component is dragging
        const DRAGGING = 0b0000100000000000;
        /// Component is a drop target
        const DROP_TARGET = 0b0001000000000000;
        /// Component has an error
        const ERROR = 0b0010000000000000;
        /// Component has a warning
        const WARNING = 0b0100000000000000;
        /// Component is loading
        const LOADING = 0b1000000000000000;

        /// Default state for new components (visible and enabled)
        const DEFAULT = Self::VISIBLE.bits() | Self::ENABLED.bits();
    }
}

impl Default for StateFlags {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl StateFlags {
    /// Check if component needs re-rendering
    pub fn needs_render(&self) -> bool {
        self.contains(Self::DIRTY)
    }

    /// Mark component as needing re-render
    pub fn mark_dirty(&mut self) {
        self.insert(Self::DIRTY);
    }

    /// Mark component as clean (rendered)
    pub fn mark_clean(&mut self) {
        self.remove(Self::DIRTY);
    }

    /// Check if component can receive input
    pub fn is_interactive(&self) -> bool {
        self.contains(Self::VISIBLE | Self::ENABLED)
    }

    /// Check if component is in any active state
    pub fn is_active(&self) -> bool {
        self.intersects(Self::FOCUSED | Self::HOVERED | Self::ACTIVE | Self::PRESSED)
    }

    /// Set multiple flags at once
    pub fn set_multiple(&mut self, flags: Self, value: bool) {
        if value {
            self.insert(flags);
        } else {
            self.remove(flags);
        }
    }

    /// Toggle specific flags
    pub fn toggle_flags(&mut self, flags: Self) {
        *self = self.symmetric_difference(flags);
    }
}

/// Trait for components that use state flags
pub trait StateFlagged {
    /// Get current state flags
    fn state_flags(&self) -> StateFlags;

    /// Set state flags
    fn set_state_flags(&mut self, flags: StateFlags);

    /// Update state flags with a function
    fn update_state_flags(&mut self, f: impl FnOnce(&mut StateFlags)) {
        let mut flags = self.state_flags();
        f(&mut flags);
        self.set_state_flags(flags);
    }

    /// Check if component has specific flags
    fn has_flags(&self, flags: StateFlags) -> bool {
        self.state_flags().contains(flags)
    }

    /// Check if component is visible
    fn is_visible(&self) -> bool {
        self.has_flags(StateFlags::VISIBLE)
    }

    /// Check if component is enabled
    fn is_enabled(&self) -> bool {
        self.has_flags(StateFlags::ENABLED)
    }

    /// Check if component has focus
    fn is_focused(&self) -> bool {
        self.has_flags(StateFlags::FOCUSED)
    }

    /// Set visibility
    fn set_visible(&mut self, visible: bool) {
        self.update_state_flags(|flags| {
            flags.set_multiple(StateFlags::VISIBLE, visible);
            flags.mark_dirty(); // Always mark dirty when visibility changes
        });
    }

    /// Set enabled state
    fn set_enabled(&mut self, enabled: bool) {
        self.update_state_flags(|flags| {
            flags.set_multiple(StateFlags::ENABLED, enabled);
            flags.mark_dirty();
        });
    }

    /// Set focus state
    fn set_focused(&mut self, focused: bool) {
        self.update_state_flags(|flags| {
            flags.set_multiple(StateFlags::FOCUSED, focused);
            flags.mark_dirty();
        });
    }
}

/// Example implementation for a component with state flags
#[derive(Clone, Debug, Default)]
pub struct ComponentState {
    flags: StateFlags,
    // Other component state fields...
}

impl StateFlagged for ComponentState {
    fn state_flags(&self) -> StateFlags {
        self.flags
    }

    fn set_state_flags(&mut self, flags: StateFlags) {
        self.flags = flags;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_flags() {
        let flags = StateFlags::default();
        assert!(flags.contains(StateFlags::VISIBLE));
        assert!(flags.contains(StateFlags::ENABLED));
        assert!(!flags.contains(StateFlags::FOCUSED));
    }

    #[test]
    fn test_flag_operations() {
        let mut flags = StateFlags::empty();

        flags.insert(StateFlags::VISIBLE | StateFlags::FOCUSED);
        assert!(flags.contains(StateFlags::VISIBLE));
        assert!(flags.contains(StateFlags::FOCUSED));

        flags.remove(StateFlags::FOCUSED);
        assert!(!flags.contains(StateFlags::FOCUSED));

        flags.toggle_flags(StateFlags::HOVERED);
        assert!(flags.contains(StateFlags::HOVERED));

        flags.toggle_flags(StateFlags::HOVERED);
        assert!(!flags.contains(StateFlags::HOVERED));
    }

    #[test]
    fn test_state_methods() {
        let mut flags = StateFlags::default();

        assert!(flags.is_interactive());

        flags.mark_dirty();
        assert!(flags.needs_render());

        flags.mark_clean();
        assert!(!flags.needs_render());

        flags.insert(StateFlags::FOCUSED);
        assert!(flags.is_active());
    }

    #[test]
    fn test_state_flagged_trait() {
        let mut state = ComponentState::default();

        assert!(state.is_visible());
        assert!(state.is_enabled());
        assert!(!state.is_focused());

        state.set_visible(false);
        assert!(!state.is_visible());
        assert!(state.state_flags().contains(StateFlags::DIRTY));

        state.set_focused(true);
        assert!(state.is_focused());

        state.update_state_flags(|flags| {
            flags.insert(StateFlags::SELECTED | StateFlags::EXPANDED);
        });
        assert!(state.has_flags(StateFlags::SELECTED));
        assert!(state.has_flags(StateFlags::EXPANDED));
    }
}
