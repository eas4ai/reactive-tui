/// Declarative focus management for components
///
/// This module provides React-like declarative focus properties that can be
/// attached to elements, replacing the imperative focus trap API.
use std::sync::Arc;

/// Focus-related properties that can be attached to any element
#[derive(Clone)]
pub struct FocusProps {
    /// Request focus on mount or when changed from false to true.
    pub auto_focus: bool,

    /// Whether this element can receive focus via keyboard navigation
    pub focusable: bool,

    /// Whether focus should be trapped within this container and its children
    pub trap_focus: bool,

    /// Tab order: positive values first, then zero in rendered order; negative
    /// values allow explicit focus but are skipped by Tab navigation.
    pub tab_index: i32,

    /// Whether to restore focus to previous element when this element unmounts
    pub restore_focus: bool,

    /// Callback when element receives focus
    pub on_focus: Option<Arc<dyn Fn() + Send + Sync>>,

    /// Callback when element loses focus
    pub on_blur: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl std::fmt::Debug for FocusProps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FocusProps")
            .field("auto_focus", &self.auto_focus)
            .field("focusable", &self.focusable)
            .field("trap_focus", &self.trap_focus)
            .field("tab_index", &self.tab_index)
            .field("restore_focus", &self.restore_focus)
            .field("on_focus", &self.on_focus.is_some())
            .field("on_blur", &self.on_blur.is_some())
            .finish()
    }
}

impl PartialEq for FocusProps {
    fn eq(&self, other: &Self) -> bool {
        self.auto_focus == other.auto_focus
            && self.focusable == other.focusable
            && self.trap_focus == other.trap_focus
            && self.tab_index == other.tab_index
            && self.restore_focus == other.restore_focus
        // Note: We intentionally don't compare on_focus and on_blur callbacks
        // as function pointers can't be meaningfully compared
    }
}

impl Default for FocusProps {
    fn default() -> Self {
        Self {
            auto_focus: false,
            focusable: true,
            trap_focus: false,
            tab_index: 0,
            restore_focus: false,
            on_focus: None,
            on_blur: None,
        }
    }
}

impl FocusProps {
    /// Create focus props for an input element
    pub fn input() -> Self {
        Self {
            focusable: true,
            tab_index: 0,
            ..Default::default()
        }
    }

    /// Create focus props for a button
    pub fn button() -> Self {
        Self {
            focusable: true,
            tab_index: 0,
            ..Default::default()
        }
    }

    /// Create focus props for a modal dialog
    pub fn modal() -> Self {
        Self {
            auto_focus: true,
            trap_focus: true,
            restore_focus: true,
            ..Default::default()
        }
    }

    /// Create focus props for a menu
    pub fn menu() -> Self {
        Self {
            trap_focus: true,
            restore_focus: true,
            ..Default::default()
        }
    }
}

/// Builder for FocusProps
pub struct FocusPropsBuilder {
    props: FocusProps,
}

impl Default for FocusPropsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FocusPropsBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            props: FocusProps::default(),
        }
    }

    /// Set auto focus
    pub fn auto_focus(mut self, value: bool) -> Self {
        self.props.auto_focus = value;
        self
    }

    /// Set focusable
    pub fn focusable(mut self, value: bool) -> Self {
        self.props.focusable = value;
        self
    }

    /// Set trap focus
    pub fn trap_focus(mut self, value: bool) -> Self {
        self.props.trap_focus = value;
        self
    }

    /// Set tab index
    pub fn tab_index(mut self, value: i32) -> Self {
        self.props.tab_index = value;
        self
    }

    /// Set restore focus
    pub fn restore_focus(mut self, value: bool) -> Self {
        self.props.restore_focus = value;
        self
    }

    /// Set on focus callback
    pub fn on_focus(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.props.on_focus = Some(Arc::new(callback));
        self
    }

    /// Set on blur callback
    pub fn on_blur(mut self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.props.on_blur = Some(Arc::new(callback));
        self
    }

    /// Build the focus props
    pub fn build(self) -> FocusProps {
        self.props
    }
}
