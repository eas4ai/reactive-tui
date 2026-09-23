//! CSS Animation Manager - Bridge between CSS animations and component animations
//!
//! This module provides the integration layer that converts CSS animation classes
//! into actual component animations using the animation system.

use crate::animation::Animation;
use crate::layout::css::animations::{
    apply_css_animation_to_component, get_available_css_animations,
};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

/// Manages the lifecycle of CSS animations applied to components
pub struct CssAnimationManager {
    /// Active animations by component ID
    active_animations: Arc<RwLock<HashMap<String, Vec<Animation>>>>,
    /// Animation registry for cleanup
    animation_registry: Arc<RwLock<HashMap<String, String>>>, // animation_id -> component_id
}

impl std::fmt::Debug for CssAnimationManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CssAnimationManager")
            .field("active_animations", &"<HashMap<String, Vec<Animation>>>")
            .field("animation_registry", &self.animation_registry)
            .finish()
    }
}

impl CssAnimationManager {
    /// Create a new CSS animation manager
    pub fn new() -> Self {
        Self {
            active_animations: Arc::new(RwLock::new(HashMap::new())),
            animation_registry: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Apply a CSS animation to a component
    pub fn apply_animation(&self, component_id: &str, animation_name: &str) -> Result<(), String> {
        // Create the animation from CSS spec
        let animation = apply_css_animation_to_component(component_id, animation_name)?;
        let animation_id = animation.id.clone();

        // Store the animation
        {
            let mut active = self
                .active_animations
                .write()
                .map_err(|_| "Failed to acquire write lock on active animations")?;

            active
                .entry(component_id.to_string())
                .or_insert_with(Vec::new)
                .push(animation);
        }

        // Register for cleanup
        {
            let mut registry = self
                .animation_registry
                .write()
                .map_err(|_| "Failed to acquire write lock on animation registry")?;

            registry.insert(animation_id, component_id.to_string());
        }

        Ok(())
    }

    /// Remove all animations for a component
    pub fn remove_component_animations(&self, component_id: &str) -> Result<usize, String> {
        let mut removed_count = 0;

        // Remove from active animations
        {
            let mut active = self
                .active_animations
                .write()
                .map_err(|_| "Failed to acquire write lock on active animations")?;

            if let Some(animations) = active.remove(component_id) {
                removed_count = animations.len();
            }
        }

        // Clean up registry entries
        {
            let mut registry = self
                .animation_registry
                .write()
                .map_err(|_| "Failed to acquire write lock on animation registry")?;

            registry.retain(|_, comp_id| comp_id != component_id);
        }

        Ok(removed_count)
    }

    /// Get active animations count for a component
    pub fn get_component_animation_count(&self, component_id: &str) -> Result<usize, String> {
        let active = self
            .active_animations
            .read()
            .map_err(|_| "Failed to acquire read lock on active animations")?;

        Ok(active.get(component_id).map(|v| v.len()).unwrap_or(0))
    }

    /// Check if a component has any active animations
    pub fn has_animations(&self, component_id: &str) -> bool {
        if let Ok(active) = self.active_animations.read() {
            active
                .get(component_id)
                .is_some_and(|anims| !anims.is_empty())
        } else {
            false
        }
    }

    /// Get all active component IDs with animations
    pub fn get_animated_components(&self) -> Vec<String> {
        if let Ok(active) = self.active_animations.read() {
            active.keys().cloned().collect()
        } else {
            vec![]
        }
    }

    /// Get statistics about active animations
    pub fn get_stats(&self) -> CssAnimationStats {
        let (component_count, total_animations) = if let Ok(active) = self.active_animations.read()
        {
            let component_count = active.len();
            let total_animations = active.values().map(|v| v.len()).sum();
            (component_count, total_animations)
        } else {
            (0, 0)
        };

        CssAnimationStats {
            active_components: component_count,
            total_animations,
            available_css_animations: get_available_css_animations().len(),
        }
    }

    /// Clear all animations (for cleanup)
    pub fn clear_all(&self) -> Result<usize, String> {
        let total_removed;

        {
            let mut active = self
                .active_animations
                .write()
                .map_err(|_| "Failed to acquire write lock on active animations")?;

            total_removed = active.values().map(|v| v.len()).sum();
            active.clear();
        }

        {
            let mut registry = self
                .animation_registry
                .write()
                .map_err(|_| "Failed to acquire write lock on animation registry")?;

            registry.clear();
        }

        Ok(total_removed)
    }
}

impl Default for CssAnimationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about CSS animations
#[derive(Debug, Clone)]
pub struct CssAnimationStats {
    /// Number of components with active animations
    pub active_components: usize,
    /// Total number of active animations
    pub total_animations: usize,
    /// Number of available CSS animation types
    pub available_css_animations: usize,
}

/// Global CSS animation manager instance
static GLOBAL_CSS_ANIMATION_MANAGER: OnceLock<CssAnimationManager> = OnceLock::new();

/// Get the global CSS animation manager
pub fn get_css_animation_manager() -> &'static CssAnimationManager {
    GLOBAL_CSS_ANIMATION_MANAGER.get_or_init(CssAnimationManager::new)
}

/// Apply a CSS animation to a component globally
pub fn apply_css_animation_global(component_id: &str, animation_name: &str) -> Result<(), String> {
    get_css_animation_manager().apply_animation(component_id, animation_name)
}

/// Remove all CSS animations for a component globally
pub fn remove_css_animations_global(component_id: &str) -> Result<usize, String> {
    get_css_animation_manager().remove_component_animations(component_id)
}

/// Check if a component has CSS animations globally
pub fn has_css_animations_global(component_id: &str) -> bool {
    get_css_animation_manager().has_animations(component_id)
}

/// Get global CSS animation statistics
pub fn get_css_animation_stats_global() -> CssAnimationStats {
    get_css_animation_manager().get_stats()
}

/// Clear all global CSS animations (for testing)
pub fn clear_all_css_animations_global() -> Result<usize, String> {
    get_css_animation_manager().clear_all()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_css_animation_manager_creation() {
        let manager = CssAnimationManager::new();
        let stats = manager.get_stats();

        assert_eq!(stats.active_components, 0);
        assert_eq!(stats.total_animations, 0);
        assert!(stats.available_css_animations > 0); // Should have built-in animations
    }

    #[test]
    fn test_apply_css_animation() {
        let manager = CssAnimationManager::new();

        // Apply a pulse animation
        let result = manager.apply_animation("test-component", "pulse");
        assert!(result.is_ok());

        // Check that the component has animations
        assert!(manager.has_animations("test-component"));

        let animation_count = manager
            .get_component_animation_count("test-component")
            .expect("Should be able to get animation count");
        assert_eq!(animation_count, 1);

        let stats = manager.get_stats();
        assert_eq!(stats.active_components, 1);
        assert_eq!(stats.total_animations, 1);
    }

    #[test]
    fn test_remove_component_animations() {
        let manager = CssAnimationManager::new();

        // Apply multiple animations
        manager
            .apply_animation("test-component", "pulse")
            .expect("Should be able to apply pulse animation");
        manager
            .apply_animation("test-component", "bounce")
            .expect("Should be able to apply bounce animation");

        assert_eq!(manager.get_stats().total_animations, 2);

        // Remove all animations for the component
        let removed = manager
            .remove_component_animations("test-component")
            .expect("Should be able to remove component animations");
        assert_eq!(removed, 2);

        assert!(!manager.has_animations("test-component"));
        assert_eq!(manager.get_stats().total_animations, 0);
    }

    #[test]
    fn test_global_functions() {
        // Test global functions
        let result = apply_css_animation_global("global-test", "spin");
        assert!(result.is_ok());

        assert!(has_css_animations_global("global-test"));

        let stats = get_css_animation_stats_global();
        assert!(stats.total_animations > 0);

        let removed = remove_css_animations_global("global-test")
            .expect("Should be able to remove global animations");
        assert!(removed > 0);

        assert!(!has_css_animations_global("global-test"));
    }
}
