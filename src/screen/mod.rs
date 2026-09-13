use crate::component::Element;
use crate::reactive::runtime::RuntimeContext;
use crate::render::reconcile::{PatchOp, Reconciler};
use crate::render::tree::{element_to_render_node, RenderTree};
use std::time::{Duration, Instant};

mod composition;
/// Screen management functionality
pub mod manager;
mod runtime;
/// Screen transition effects and animations
pub mod transitions;

pub use manager::*;
pub use transitions::*;

/// Unique identifier for a screen
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScreenId(String);

impl ScreenId {
    /// Create a new screen ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the ID as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ScreenId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for ScreenId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Type alias for patch update callback
type PatchUpdateCallback = Box<dyn Fn(&[PatchOp]) + Send + Sync>;
/// Type alias for lifecycle callback
type LifecycleCallback = Box<dyn Fn() + Send + Sync>;

/// Lifecycle hooks for screen events
#[derive(Default)]
pub struct ScreenHooks {
    /// Called when screen becomes active
    pub on_activate: Option<LifecycleCallback>,
    /// Called when screen becomes inactive
    pub on_deactivate: Option<LifecycleCallback>,
    /// Called when screen updates with patch operations
    pub on_update: Option<PatchUpdateCallback>,
    /// Called when screen is created
    pub on_create: Option<LifecycleCallback>,
    /// Called when screen is destroyed
    pub on_destroy: Option<LifecycleCallback>,
}

/// A single screen with its own render tree and reactive context
pub struct Screen {
    /// Unique identifier for this screen
    pub id: ScreenId,
    /// Human-readable name for this screen
    pub name: String,
    /// Render tree for this screen
    pub render_tree: RenderTree,
    /// Reactive context for state management
    pub reactive_context: RuntimeContext,
    /// Whether this screen is currently active
    pub is_active: bool,
    /// When this screen was last rendered
    pub last_rendered: Option<Instant>,
    /// Screen lifecycle hooks
    pub hooks: ScreenHooks,
    /// VDOM reconciler for efficient updates
    reconciler: Reconciler,
    /// Root element of the screen
    root_element: Option<Element>,
    runtime: runtime::ScreenRuntime,
}

impl Screen {
    /// Create a new screen with the given ID and name
    pub fn new(id: ScreenId, name: String) -> Self {
        Self {
            id,
            name,
            render_tree: RenderTree::new(),
            reactive_context: RuntimeContext::new(),
            is_active: false,
            last_rendered: None,
            hooks: ScreenHooks::default(),
            reconciler: Reconciler::new(),
            root_element: None,
            runtime: runtime::ScreenRuntime::new(),
        }
    }

    /// Set lifecycle hooks for this screen
    pub fn with_hooks(mut self, hooks: ScreenHooks) -> Self {
        self.hooks = hooks;
        self
    }

    /// Set the content of this screen
    pub fn set_content(&mut self, element: Element) {
        self.root_element = Some(element.clone());

        // Convert element to render tree
        let root_node = element_to_render_node(element);
        let mut new_tree = RenderTree::new();
        new_tree.set_root(root_node);

        // Generate patches
        let diff_result = self.reconciler.diff(&self.render_tree, &new_tree);

        // Call update hook
        if let Some(ref on_update) = self.hooks.on_update {
            on_update(&diff_result.patches);
        }

        // Update tree
        self.render_tree = new_tree;
        self.last_rendered = Some(Instant::now());
    }

    /// Activate this screen
    pub fn activate(&mut self) {
        if !self.is_active {
            self.is_active = true;
            if let Some(ref on_activate) = self.hooks.on_activate {
                on_activate();
            }
        }
    }

    /// Deactivate this screen
    pub fn deactivate(&mut self) {
        if self.is_active {
            self.is_active = false;
            if let Some(ref on_deactivate) = self.hooks.on_deactivate {
                on_deactivate();
            }
        }
    }

    /// Update the content of this screen
    pub fn update_content(&mut self, element: Element) {
        self.set_content(element);
    }

    /// Get patches generated since the last render
    pub fn get_patches_since_last_render(&mut self) -> Vec<PatchOp> {
        if let Some(ref element) = self.root_element.clone() {
            let root_node = element_to_render_node(element.clone());
            let mut new_tree = RenderTree::new();
            new_tree.set_root(root_node);

            let diff_result = self.reconciler.diff(&self.render_tree, &new_tree);
            self.render_tree = new_tree;

            diff_result.patches
        } else {
            Vec::new()
        }
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        if let Some(ref on_destroy) = self.hooks.on_destroy {
            on_destroy();
        }
    }
}

/// Screen switching transition types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionType {
    /// No transition effect
    None,
    /// Fade in/out transition
    Fade,
    /// Slide from right to left
    SlideLeft,
    /// Slide from left to right
    SlideRight,
    /// Slide from bottom to top
    SlideUp,
    /// Slide from top to bottom
    SlideDown,
    /// Scale transition effect
    Scale,
    /// Terminal flip: compress the source horizontally, then expand the target.
    Flip,
    /// Terminal cube approximation: adjacent horizontally compressed faces.
    Cube,
    /// Push transition effect
    Push,
}

/// Configuration for screen transitions
#[derive(Debug, Clone)]
pub struct TransitionConfig {
    /// Type of transition animation to use
    pub transition_type: TransitionType,
    /// Duration of the transition animation
    pub duration: Duration,
    /// Easing function for the transition
    pub easing: EasingFunction,
    /// Compatibility metadata retained for callers; terminal painting ignores it.
    pub animation_id: Option<String>,
    /// Compatibility preference; the terminal painter does not use a GPU.
    pub use_hardware_acceleration: bool,
    /// Compatibility metadata retained for callers; terminal painting ignores it.
    pub custom_properties: std::collections::HashMap<String, f32>,
}

impl Default for TransitionConfig {
    fn default() -> Self {
        Self {
            transition_type: TransitionType::Fade,
            duration: Duration::from_millis(300),
            easing: EasingFunction::EaseOutCubic,
            animation_id: None,
            use_hardware_acceleration: false,
            custom_properties: std::collections::HashMap::new(),
        }
    }
}

impl TransitionConfig {
    /// Attach an identifier as compatibility metadata.
    pub fn with_animation_id(mut self, id: impl Into<String>) -> Self {
        self.animation_id = Some(id.into());
        self
    }

    /// Record a hardware preference as compatibility metadata.
    pub fn with_hardware_acceleration(mut self) -> Self {
        self.use_hardware_acceleration = true;
        self
    }

    /// Attach a custom property as compatibility metadata.
    pub fn with_custom_property(mut self, key: impl Into<String>, value: f32) -> Self {
        self.custom_properties.insert(key.into(), value);
        self
    }

    /// Create preset configurations optimized for different scenarios
    pub fn preset_smooth_fade() -> Self {
        Self {
            transition_type: TransitionType::Fade,
            duration: Duration::from_millis(300),
            easing: EasingFunction::SpringGentle,
            ..Default::default()
        }
    }

    /// Create a quick slide transition preset
    ///
    /// # Returns
    /// A `TransitionConfig` with fast slide-left animation
    pub fn preset_quick_slide() -> Self {
        Self {
            transition_type: TransitionType::SlideLeft,
            duration: Duration::from_millis(200),
            easing: EasingFunction::SlideSmooth,
            ..Default::default()
        }
    }

    /// Create a bouncy scale transition preset
    ///
    /// # Returns
    /// A `TransitionConfig` with bouncy scale animation
    pub fn preset_bouncy_scale() -> Self {
        Self {
            transition_type: TransitionType::Scale,
            duration: Duration::from_millis(400),
            easing: EasingFunction::SnapBounce,
            ..Default::default()
        }
    }

    /// Create a dramatic flip transition preset
    ///
    /// # Returns
    /// A `TransitionConfig` with dramatic flip animation
    pub fn preset_dramatic_flip() -> Self {
        Self {
            transition_type: TransitionType::Flip,
            duration: Duration::from_millis(600),
            easing: EasingFunction::SpringDramatic,
            ..Default::default()
        }
    }

    /// Get the animation description for debugging
    pub fn description(&self) -> String {
        format!(
            "{:?} transition with {} easing over {}ms",
            self.transition_type,
            self.easing.description(),
            self.duration.as_millis()
        )
    }
}

/// Easing functions for smooth animations
/// Enhanced with screen transition-specific easing functions
/// Easing functions for screen transitions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EasingFunction {
    // Basic easing
    /// Linear interpolation (constant speed)
    Linear,
    /// Quadratic ease-in (slow start)
    EaseInQuad,
    /// Quadratic ease-out (slow end)
    EaseOutQuad,
    /// Quadratic ease-in-out (slow start and end)
    EaseInOutQuad,
    /// Cubic ease-in (slower start)
    EaseInCubic,
    /// Cubic ease-out (slower end)
    EaseOutCubic,
    /// Cubic ease-in-out (slower start and end)
    EaseInOutCubic,

    // Advanced easing
    /// Back ease-in (overshoot at start)
    EaseInBack,
    /// Back ease-out (overshoot at end)
    EaseOutBack,
    /// Back ease-in-out (overshoot at both ends)
    EaseInOutBack,
    /// Bounce ease-out (bouncing effect at end)
    EaseOutBounce,
    /// Elastic ease-in (elastic effect at start)
    EaseInElastic,
    /// Elastic ease-out (elastic effect at end)
    EaseOutElastic,
    /// Elastic ease-in-out (elastic effect at both ends)
    EaseInOutElastic,

    // Screen transition optimized easing
    /// Smooth slide with slight overshoot - perfect for screen slides
    SlideSmooth,
    /// Quick snap with bounce - good for scale transitions
    SnapBounce,
    /// Gentle spring - perfect for fade transitions
    SpringGentle,
    /// Dramatic spring - good for attention-grabbing transitions
    SpringDramatic,
}

impl EasingFunction {
    /// Apply the easing function to a normalized time value (0.0 to 1.0)
    pub fn apply(self, t: f32) -> f32 {
        match self {
            EasingFunction::Linear => t,
            EasingFunction::EaseInQuad => t * t,
            EasingFunction::EaseOutQuad => 1.0 - (1.0 - t) * (1.0 - t),
            EasingFunction::EaseInOutQuad => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - 2.0 * (1.0 - t) * (1.0 - t)
                }
            }
            EasingFunction::EaseInCubic => t * t * t,
            EasingFunction::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
            EasingFunction::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            EasingFunction::EaseInBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                c3 * t * t * t - c1 * t * t
            }
            EasingFunction::EaseOutBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
            }
            EasingFunction::EaseInOutBack => {
                let c1 = 1.70158;
                let c2 = c1 * 1.525;
                if t < 0.5 {
                    (2.0 * t).powi(2) * ((c2 + 1.0) * 2.0 * t - c2) / 2.0
                } else {
                    ((2.0 * t - 2.0).powi(2) * ((c2 + 1.0) * (t * 2.0 - 2.0) + c2) + 2.0) / 2.0
                }
            }
            EasingFunction::EaseOutBounce => {
                let n1 = 7.5625;
                let d1 = 2.75;

                if t < 1.0 / d1 {
                    n1 * t * t
                } else if t < 2.0 / d1 {
                    let t = t - 1.5 / d1;
                    n1 * t * t + 0.75
                } else if t < 2.5 / d1 {
                    let t = t - 2.25 / d1;
                    n1 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / d1;
                    n1 * t * t + 0.984375
                }
            }

            // Elastic easing functions
            EasingFunction::EaseInElastic => {
                let c4 = (2.0 * std::f32::consts::PI) / 3.0;
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    -(2.0_f32.powf(10.0 * (t - 1.0))) * ((t - 1.1) * c4).sin()
                }
            }

            EasingFunction::EaseOutElastic => {
                let c4 = (2.0 * std::f32::consts::PI) / 3.0;
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    2.0_f32.powf(-10.0 * t) * ((t - 0.1) * c4).sin() + 1.0
                }
            }

            EasingFunction::EaseInOutElastic => {
                let c5 = (2.0 * std::f32::consts::PI) / 4.5;
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else if t < 0.5 {
                    -(2.0_f32.powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * c5).sin()) / 2.0
                } else {
                    (2.0_f32.powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * c5).sin()) / 2.0 + 1.0
                }
            }

            // Screen transition optimized easing
            EasingFunction::SlideSmooth => {
                // Cubic with slight overshoot - perfect for slides
                let t_adj = if t < 0.8 {
                    t / 0.8
                } else {
                    1.0 + (t - 0.8) * 0.5
                };
                let cubic = t_adj * t_adj * t_adj;
                cubic.min(1.0)
            }

            EasingFunction::SnapBounce => {
                // Quick acceleration with bounce - good for scale
                if t < 0.7 {
                    let t_norm = t / 0.7;
                    t_norm * t_norm
                } else {
                    let bounce_t = (t - 0.7) / 0.3;
                    1.0 + 0.3 * (bounce_t * std::f32::consts::PI * 2.0).sin() * (1.0 - bounce_t)
                }
            }

            EasingFunction::SpringGentle => {
                // Gentle spring motion - perfect for fades
                let spring_factor = 0.8;
                let damping = 0.7;
                let freq = 1.5;

                if t >= 1.0 {
                    1.0
                } else {
                    1.0 - (spring_factor * (-damping * t).exp() * (freq * t).cos())
                }
            }

            EasingFunction::SpringDramatic => {
                // More dramatic spring - attention-grabbing
                let spring_factor = 1.2;
                let damping = 0.5;
                let freq = 2.0;

                if t >= 1.0 {
                    1.0
                } else {
                    1.0 - (spring_factor * (-damping * t).exp() * (freq * t).cos())
                }
            }
        }
    }

    /// Get a description of the easing function for debugging
    pub fn description(self) -> &'static str {
        match self {
            EasingFunction::Linear => "Linear",
            EasingFunction::EaseInQuad => "Ease In Quadratic",
            EasingFunction::EaseOutQuad => "Ease Out Quadratic",
            EasingFunction::EaseInOutQuad => "Ease In-Out Quadratic",
            EasingFunction::EaseInCubic => "Ease In Cubic",
            EasingFunction::EaseOutCubic => "Ease Out Cubic",
            EasingFunction::EaseInOutCubic => "Ease In-Out Cubic",
            EasingFunction::EaseInBack => "Ease In Back",
            EasingFunction::EaseOutBack => "Ease Out Back",
            EasingFunction::EaseInOutBack => "Ease In-Out Back",
            EasingFunction::EaseOutBounce => "Ease Out Bounce",
            EasingFunction::EaseInElastic => "Ease In Elastic",
            EasingFunction::EaseOutElastic => "Ease Out Elastic",
            EasingFunction::EaseInOutElastic => "Ease In-Out Elastic",
            EasingFunction::SlideSmooth => "Slide Smooth (Screen Optimized)",
            EasingFunction::SnapBounce => "Snap Bounce (Screen Optimized)",
            EasingFunction::SpringGentle => "Spring Gentle (Screen Optimized)",
            EasingFunction::SpringDramatic => "Spring Dramatic (Screen Optimized)",
        }
    }

    /// Get recommended easing functions for different transition types
    pub fn recommended_for_transition(transition_type: TransitionType) -> Vec<EasingFunction> {
        match transition_type {
            TransitionType::None => vec![EasingFunction::Linear],
            TransitionType::Fade => vec![
                EasingFunction::SpringGentle,
                EasingFunction::EaseOutCubic,
                EasingFunction::EaseInOutQuad,
            ],
            TransitionType::SlideLeft
            | TransitionType::SlideRight
            | TransitionType::SlideUp
            | TransitionType::SlideDown => vec![
                EasingFunction::SlideSmooth,
                EasingFunction::EaseOutBack,
                EasingFunction::EaseInOutCubic,
            ],
            TransitionType::Scale => vec![
                EasingFunction::SnapBounce,
                EasingFunction::EaseOutBack,
                EasingFunction::SpringDramatic,
            ],
            TransitionType::Flip => vec![
                EasingFunction::EaseInOutCubic,
                EasingFunction::EaseOutElastic,
                EasingFunction::SpringGentle,
            ],
            TransitionType::Cube => vec![
                EasingFunction::EaseInOutBack,
                EasingFunction::SlideSmooth,
                EasingFunction::EaseOutCubic,
            ],
            TransitionType::Push => vec![
                EasingFunction::SlideSmooth,
                EasingFunction::EaseOutQuad,
                EasingFunction::EaseInOutCubic,
            ],
        }
    }

    /// Create preset configurations for common screen transition scenarios
    pub fn preset_smooth() -> EasingFunction {
        EasingFunction::EaseOutCubic
    }

    /// Create a bouncy easing preset
    ///
    /// # Returns
    /// An ease-out bounce easing function
    pub fn preset_bouncy() -> EasingFunction {
        EasingFunction::EaseOutBounce
    }

    /// Create an elastic easing preset
    ///
    /// # Returns
    /// An ease-out elastic easing function
    pub fn preset_elastic() -> EasingFunction {
        EasingFunction::EaseOutElastic
    }

    /// Create a spring easing preset
    ///
    /// # Returns
    /// A gentle spring easing function
    pub fn preset_spring() -> EasingFunction {
        EasingFunction::SpringGentle
    }

    /// Create a dramatic easing preset
    ///
    /// # Returns
    /// A dramatic spring easing function
    pub fn preset_dramatic() -> EasingFunction {
        EasingFunction::SpringDramatic
    }
}

/// Current state of a screen transition
#[derive(Debug, Clone)]
pub struct TransitionState {
    /// Whether a transition is currently in progress
    pub is_transitioning: bool,
    /// ID of the screen being transitioned from
    pub from_screen: Option<ScreenId>,
    /// ID of the screen being transitioned to
    pub to_screen: Option<ScreenId>,
    /// Current progress of the transition (0.0 to 1.0)
    pub progress: f32,
    /// When the transition started
    pub start_time: Option<Instant>,
    /// Configuration for the current transition
    pub config: TransitionConfig,
}

impl Default for TransitionState {
    fn default() -> Self {
        Self {
            is_transitioning: false,
            from_screen: None,
            to_screen: None,
            progress: 0.0,
            start_time: None,
            config: TransitionConfig::default(),
        }
    }
}

impl TransitionState {
    /// Start a new transition
    ///
    /// # Arguments
    /// * `from` - Screen transitioning from (None for initial)
    /// * `to` - Screen transitioning to
    /// * `config` - Transition configuration
    pub fn start_transition(
        &mut self,
        from: Option<ScreenId>,
        to: ScreenId,
        config: TransitionConfig,
    ) {
        self.is_transitioning = true;
        self.from_screen = from;
        self.to_screen = Some(to);
        self.progress = 0.0;
        self.start_time = Some(Instant::now());
        self.config = config;
    }

    /// Update transition progress
    ///
    /// # Returns
    /// true when this call completes the transition
    pub fn update(&mut self) -> bool {
        self.update_at(Instant::now())
    }

    pub(crate) fn update_at(&mut self, now: Instant) -> bool {
        if !self.is_transitioning {
            return false;
        }

        let Some(start_time) = self.start_time else {
            return false;
        };

        let elapsed = now.saturating_duration_since(start_time);
        if elapsed >= self.config.duration {
            self.is_transitioning = false;
            self.progress = 1.0;
            self.start_time = None;
            true // Transition completed
        } else {
            let t = elapsed.as_secs_f32() / self.config.duration.as_secs_f32();
            self.progress = self.config.easing.apply(t);
            false // Still transitioning
        }
    }

    /// Check if transition is complete
    ///
    /// # Returns
    /// true if transition has finished
    pub fn is_complete(&self) -> bool {
        !self.is_transitioning && self.progress >= 1.0
    }
}

#[cfg(test)]
mod residual_metadata_tests {
    use super::*;

    #[test]
    fn api019_compatibility_metadata_does_not_change_terminal_transition_progress() {
        let destination = ScreenId::new("destination");
        let plain = TransitionConfig::default();
        let decorated = TransitionConfig::default()
            .with_animation_id("legacy")
            .with_custom_property("unused", 99.0)
            .with_hardware_acceleration();
        let started = Instant::now();
        let mut left = TransitionState::default();
        let mut right = TransitionState::default();
        left.start_transition(None, destination.clone(), plain);
        right.start_transition(None, destination, decorated);
        left.start_time = Some(started);
        right.start_time = Some(started);
        let sample = started + Duration::from_millis(150);
        assert_eq!(left.update_at(sample), right.update_at(sample));
        assert_eq!(left.progress, right.progress);
    }
}
