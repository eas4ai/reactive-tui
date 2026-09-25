//! Semantic accessibility for terminal widgets.
//!
//! Screen-reader integration targets Orca with GNOME Terminal on Linux.
//! Other terminal and screen-reader pairs are unverified. Node labels may differ
//! from painted text; state and focus must be published alongside the label.

pub use accesskit::{Live, Role, Toggled};

pub(crate) mod style;
pub(crate) mod text;

/// Role, label and state attached to one Element.
/// Identity, relationships and supported actions are owned by App.
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub(crate) inner: Box<accesskit::Node>,
    // Positions within the direct TextRun children, resolved to App IDs on export.
    pub(crate) text_selection: Option<[(usize, usize); 2]>,
}

impl Node {
    /// Describe an element with the given semantic role.
    pub fn new(role: Role) -> Self {
        Self {
            inner: Box::new(accesskit::Node::new(role)),
            text_selection: None,
        }
    }
    /// Label announced independently of the painted text.
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.inner.set_label(label.into());
    }
    /// Additional accessible description.
    pub fn set_description(&mut self, description: impl Into<String>) {
        self.inner.set_description(description.into());
    }
    /// Expanded or collapsed state of a disclosure control.
    pub fn set_expanded(&mut self, expanded: bool) {
        self.inner.set_expanded(expanded);
    }
    /// Selection state of an option, tab or row.
    pub fn set_selected(&mut self, selected: bool) {
        self.inner.set_selected(selected);
    }
    /// Checked, unchecked or mixed state.
    pub fn set_toggled(&mut self, toggled: Toggled) {
        self.inner.set_toggled(toggled);
    }
    /// Mark the node disabled for assistive actions as well as announcements.
    pub fn set_disabled(&mut self) {
        self.inner.set_disabled();
    }
    /// Hide this element and its descendants from assistive technology.
    pub fn set_hidden(&mut self) {
        self.inner.set_hidden();
    }
    /// Mark a value as read-only.
    pub fn set_read_only(&mut self) {
        self.inner.set_read_only();
    }
    /// Accessible textual value. Do not put unmasked passwords here.
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.inner.set_value(value.into());
    }
    /// Announce subsequent content changes according to the requested priority.
    pub fn set_live(&mut self, live: Live) {
        self.inner.set_live(live);
    }
    /// Expose activation through this element's existing App event handlers.
    pub fn set_clickable(&mut self) {
        self.inner.add_action(accesskit::Action::Click);
    }
    /// Mark the content as still being prepared, as ARIA `aria-busy` does:
    /// assistive technology waits for it before reading it.
    pub fn set_busy(&mut self) {
        self.inner.set_busy();
    }
    /// Whether [`Node::set_busy`] marked the content as still being prepared.
    pub fn is_busy(&self) -> bool {
        self.inner.is_busy()
    }
}

#[cfg(target_os = "linux")]
// Keep the upstream adapter facade intact, including methods not used by this
// private integration, so the maintained delta remains small and reviewable.
#[allow(dead_code)]
mod platform;

#[cfg(target_os = "linux")]
mod connection;
#[cfg(target_os = "linux")]
pub(crate) use connection::Connection;

#[cfg(target_os = "linux")]
use crate::event::{hit::Bounds, router::NodeId, CustomEvent};
#[cfg(target_os = "linux")]
use std::collections::HashMap;

#[cfg(target_os = "linux")]
#[derive(Clone)]
pub(crate) struct Target {
    pub node: NodeId,
    pub owner: NodeId,
    pub focus_event: Option<CustomEvent>,
    pub click_event: Option<CustomEvent>,
    pub bounds: Bounds,
    pub clickable: bool,
}

#[cfg(target_os = "linux")]
pub(crate) struct Snapshot {
    pub update: accesskit::TreeUpdate,
    pub targets: HashMap<accesskit::NodeId, Target>,
}

#[cfg(target_os = "linux")]
impl Snapshot {
    pub(crate) fn empty(name: &str) -> Self {
        let mut root = accesskit::Node::new(Role::Window);
        root.set_label(name);
        Self {
            update: accesskit::TreeUpdate {
                nodes: vec![(accesskit::NodeId(0), root)],
                tree: Some(accesskit::TreeInfo::new(accesskit::NodeId(0))),
                tree_id: accesskit::TreeId::ROOT,
                focus: accesskit::NodeId(0),
            },
            targets: HashMap::new(),
        }
    }
}
