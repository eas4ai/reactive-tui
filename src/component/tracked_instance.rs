//! Tracked component instances with automatic cleanup
//!
//! This module provides a wrapper around component instances that ensures
//! automatic cleanup when the instance is dropped, preventing memory leaks.

use super::instance::AnyComponentInstance;
use super::LifecycleEvent;
use crate::render::tree::NodeKey;
use std::sync::Arc;

/// A component instance that automatically cleans up when dropped
pub struct TrackedComponentInstance {
    instance: Option<AnyComponentInstance>,
    node_key: NodeKey,
}

impl TrackedComponentInstance {
    /// Create a new tracked component instance. The registry argument is kept
    /// for compatibility; retaining it would create a registry/instance cycle.
    pub fn new(
        instance: AnyComponentInstance,
        node_key: NodeKey,
        _registry: Arc<super::registry::ComponentRegistry>,
    ) -> Self {
        Self {
            instance: Some(instance),
            node_key,
        }
    }

    /// Get a reference to the inner component instance
    pub fn instance(&self) -> Option<&AnyComponentInstance> {
        self.instance.as_ref()
    }

    /// Get a mutable reference to the inner component instance
    pub fn instance_mut(&mut self) -> Option<&mut AnyComponentInstance> {
        self.instance.as_mut()
    }

    /// Take the inner instance, leaving None in its place
    pub fn take_instance(&mut self) -> Option<AnyComponentInstance> {
        self.instance.take()
    }

    /// Get the node key
    pub fn node_key(&self) -> &NodeKey {
        &self.node_key
    }

    /// Manually trigger cleanup (called by Drop automatically)
    fn cleanup(&mut self) {
        if let Some(mut instance) = self.instance.take() {
            // Call unmount lifecycle event
            instance.on_lifecycle(LifecycleEvent::Unmount);

            // Remove CSS animations
            let component_id = format!("{:?}", self.node_key);
            if let Err(_e) =
                crate::layout::css::manager::remove_css_animations_global(&component_id)
            {
                #[cfg(debug_assertions)]
                log::warn!(
                    "Failed to remove CSS animations during cleanup for component '{}': {}",
                    component_id,
                    _e
                );
            }
        }
    }
}

impl Drop for TrackedComponentInstance {
    fn drop(&mut self) {
        // Automatic cleanup when the instance is dropped
        self.cleanup();

        // Log cleanup for debugging
        #[cfg(debug_assertions)]
        log::debug!(
            "TrackedComponentInstance dropped for node {:?}",
            self.node_key
        );
    }
}

/// A reference-counted tracked component instance
pub type SharedTrackedInstance = Arc<std::sync::RwLock<TrackedComponentInstance>>;

/// Helper to create a shared tracked instance
pub fn create_shared_tracked_instance(
    instance: AnyComponentInstance,
    node_key: NodeKey,
    registry: Arc<super::registry::ComponentRegistry>,
) -> SharedTrackedInstance {
    Arc::new(std::sync::RwLock::new(TrackedComponentInstance::new(
        instance, node_key, registry,
    )))
}
