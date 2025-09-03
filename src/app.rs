use crate::backend::Backend;
use crate::component::Element;
use crate::display::adaptive::{AdaptiveConfig, AdaptiveFpsManager};
use crate::display::monitor::PerformanceMode;
use crate::event::router::{EventResult, EventRouter};

use crate::animation::AnimationManager;
use crate::error::Result;
use crate::reactive::scheduler::Scheduler;
use std::time::{Duration, Instant};
use crate::render::{Reconciler, RenderTree};

/// Trait for root components that can render to an Element
pub trait RootComponent: Send + Sync {
    /// Render the component to an Element tree
    fn render(&self) -> Element;
}

/// Main application context managing the reactive component tree
pub struct App {
    backend: Box<dyn Backend>,
    root: Box<dyn RootComponent>,
    scheduler: Scheduler,
    router: EventRouter,
    tree: RenderTree,
    previous_tree: RenderTree,
    reconciler: Reconciler,
    fps_manager: AdaptiveFpsManager,
    animation_manager: AnimationManager,
    running: bool,
    debug: bool,
    last_frame_time: Instant,
    resize_count: usize,
}

impl App {
    /// Create a new application builder
    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    /// Run the application main loop
    pub fn run(mut self) -> Result<()> {
        self.running = true;

        // Initial render and benchmark
        self.render()?;
        self.fps_manager.benchmark_if_needed(&self.tree);

        // If debug mode is on, enable backend's debug overlay (no-op if unsupported)
        if self.debug {
            self.backend.set_debug_overlay(true);
        }

        // Show display capabilities
        if self.debug {
            eprintln!("{}", self.fps_manager.get_recommendation_summary());
        }

        while self.running {
            let frame_start = Instant::now();
            let frame_duration = self.fps_manager.get_frame_duration();

            // Update global performance context at start of frame
            {
                use crate::hooks::perf_context::{
                    get_global_performance_context, set_global_performance_context,
                    PerformanceContext,
                };
                use crate::reactive::hooks::ThreadSafeSignal;
                use std::sync::Arc;

                let last_ms = self.last_frame_time.elapsed().as_secs_f32() * 1000.0;
                let metrics = self.fps_manager.get_performance_metrics();
                let fps_state = crate::hooks::fps::FpsState {
                    target_fps: self.fps_manager.get_target_fps(),
                    current_fps: metrics.current_fps,
                    avg_render_time_ms: metrics.avg_render_time_ms,
                    drop_rate_percent: metrics.drop_rate_percent,
                    is_stable: metrics.is_stable,
                    mode: crate::display::monitor::PerformanceMode::Auto,
                };
                let frame_timing = crate::hooks::fps::FrameTiming {
                    last_frame_ms: last_ms,
                    target_frame_ms: frame_duration.as_secs_f32() * 1000.0,
                    budget_remaining_ms: 0.0,
                };

                if let Some(ctx) = get_global_performance_context() {
                    ctx.fps_state.set(fps_state);
                    ctx.metrics.set(metrics.clone());
                    ctx.frame_timing.set(frame_timing);
                } else {
                    let ctx = PerformanceContext {
                        fps_state: ThreadSafeSignal::new(fps_state),
                        metrics: ThreadSafeSignal::new(metrics.clone()),
                        frame_timing: ThreadSafeSignal::new(frame_timing),
                        set_mode: Arc::new(|mode| {
                            crate::hooks::perf_context::request_performance_mode(mode)
                        }),
                    };
                    set_global_performance_context(Arc::new(ctx));
                }
            }

            // 1) Poll input with timeout based on target FPS
            let poll_timeout = frame_duration.as_millis() as u64;
            if let Some(event) = self.backend.poll_event(Some(poll_timeout.min(16)))? {
                // Handle application-level events first
                let mut should_render = false;

                match &event {
                    crate::event::types::Event::Key(key_event) => {
                        // Check for quit key (Ctrl+C or Esc)
                        if (key_event.code == crate::event::types::KeyCode::Char('c')
                            && key_event.modifiers.ctrl)
                            || key_event.code == crate::event::types::KeyCode::Escape
                        {
                            self.running = false;
                        }
                    }
                    crate::event::types::Event::Resize(resize_event) => {
                        // Handle terminal resize - update backend and force re-render
                        self.handle_resize(resize_event.width, resize_event.height)?;
                        should_render = true; // Always re-render on resize
                    }
                    _ => {}
                }

                // Process event through the event system
                let result = self.router.process_event(&event);

                // Trigger re-render if event was handled or if we need to render for other reasons
                if result != EventResult::Ignored || should_render {
                    self.render()?;
                }
            }

            // 2) Process scheduled updates from reactive system
            if self.scheduler.has_pending_updates() {
                self.scheduler.process_updates();
                self.render()?;
            }

            // 3) Update animations and check if render is needed
            // Update hook-based animations
            crate::hooks::animation::update_hook_animations();

            // Update declarative animations
            self.animation_manager.update();

            // Render if we have active animations
            if self.animation_manager.active_count() > 0 {
                self.render()?;
            }

            // 4) Process any timer callbacks
            self.scheduler.process_timers();

            // 5) Frame timing and adaptive FPS
            let frame_elapsed = frame_start.elapsed();
            let render_time = frame_elapsed.saturating_sub(Duration::from_millis(1)); // Estimate

            // Check if we need to drop this frame
            let dropped = frame_elapsed > frame_duration;

            // Record performance and potentially adjust FPS
            self.fps_manager
                .record_frame_performance(frame_elapsed, render_time, dropped);

            // Sleep if we finished early
            if let Some(sleep_duration) = frame_duration.checked_sub(frame_elapsed) {
                std::thread::sleep(sleep_duration);
            }

            self.last_frame_time = Instant::now();
        }

        Ok(())
    }

    /// Register a focusable node with optional tab index
    /// Returns true if successful
    pub fn register_focusable(
        &mut self,
        node_id: crate::event::router::NodeId,
        tab_index: Option<i32>,
    ) -> bool {
        self.router.add_focusable(node_id, tab_index)
    }

    /// Unregister a focusable node
    pub fn unregister_focusable(&mut self, node_id: crate::event::router::NodeId) {
        self.router.remove_focusable(node_id);
    }

    /// Set spatial info used for arrow-key navigation
    /// Returns true if successful
    pub fn set_focus_spatial(
        &mut self,
        node_id: crate::event::router::NodeId,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) -> bool {
        self.router.update_spatial(node_id, x, y, w, h)
    }

    /// Set initial focus and update router
    /// Returns true if successful
    pub fn set_initial_focus(&mut self, node_id: crate::event::router::NodeId) -> bool {
        self.router.set_focus(Some(node_id));
        // Check if focus was actually set
        self.router.get_focus() == Some(node_id)
    }

    /// Move focus to next and update router
    /// Returns the newly focused node if successful
    pub fn focus_next(&mut self) -> Option<crate::event::router::NodeId> {
        self.router.focus_next()
    }

    /// Move focus to previous and update router
    /// Returns the newly focused node if successful
    pub fn focus_previous(&mut self) -> Option<crate::event::router::NodeId> {
        self.router.focus_prev()
    }

    /// Move focus in a direction and update router
    /// Returns the newly focused node if successful
    pub fn focus_move(&mut self, dir: crate::event::FocusDirection) -> Option<crate::event::router::NodeId> {
        self.router.focus_move(dir)
    }

    /// Create a focus trap for a container (e.g., modal dialog)
    /// This restricts focus navigation to only nodes within the container
    pub fn create_focus_trap(&mut self, container: crate::event::router::NodeId, trapped_nodes: Vec<crate::event::router::NodeId>) -> bool {
        self.router.create_focus_trap(container, trapped_nodes)
    }

    /// Remove a focus trap and restore previous focus behavior
    pub fn remove_focus_trap(&mut self, container: crate::event::router::NodeId) -> bool {
        self.router.remove_focus_trap(container)
    }

    /// Check if focus is currently trapped
    pub fn is_focus_trapped(&self) -> bool {
        self.router.is_focus_trapped()
    }

    /// Get the active focus trap container, if any
    pub fn get_active_focus_trap(&self) -> Option<crate::event::router::NodeId> {
        self.router.get_active_focus_trap()
    }

    /// Stop the application
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Set performance mode
    pub fn set_performance_mode(&mut self, mode: PerformanceMode) {
        self.fps_manager.set_performance_mode(mode);
    }

    /// Get current FPS
    pub fn get_current_fps(&self) -> u32 {
        self.fps_manager.get_target_fps()
    }

    /// Get performance metrics
    pub fn get_performance_metrics(&self) -> crate::display::monitor::PerformanceMetrics {
        self.fps_manager.get_performance_metrics()
    }

    /// Render the current state
    fn render(&mut self) -> Result<()> {
        use crate::render::tree::element_to_render_node;
        
        // Build element tree from root component
        let element = self.root.render();
        
        // Convert to RenderTree
        let root_node = element_to_render_node(element.clone());
        let mut new_tree = RenderTree::new();
        new_tree.set_root(root_node);
        
        // Diff against previous tree
        let diff_result = self.reconciler.diff(&self.previous_tree, &new_tree);
        
        // Apply patches if there are changes
        if !diff_result.patches.is_empty() {
            // Apply patches (automatic component cleanup will be handled)
            let _ = crate::render::reconcile::apply_patches(
                &diff_result.patches,
                &mut self.tree,
            );

            // For now, still do full render, but we have the patches for future optimization
            self.backend.render_full(&element)?;
        } else if self.previous_tree.root().is_none() {
            // First render
            self.backend.render_full(&element)?;
        }
        
        // Store current tree for next frame
        self.previous_tree = std::mem::replace(&mut self.tree, new_tree);

        // Present frame
        self.backend.present()?;

        // After present, update global performance context (if set)
        if let Some(req) = crate::hooks::perf_context::take_requested_performance_mode() {
            self.fps_manager.set_performance_mode(req);
        }

        Ok(())
    }

    /// Get the current terminal size
    pub fn size(&self) -> (u16, u16) {
        self.backend.size()
    }

    /// Get the current terminal size as Size struct
    pub fn get_size(&self) -> crate::core::geometry::Size {
        let (w, h) = self.backend.size();
        crate::core::geometry::Size::new(w as usize, h as usize)
    }

    /// Handle terminal resize events
    fn handle_resize(&mut self, width: u16, height: u16) -> Result<()> {
        // Increment resize counter for debugging/testing
        self.resize_count += 1;

        if self.debug {
            eprintln!(
                "🔄 Terminal resize #{}: {}x{}",
                self.resize_count, width, height
            );
        }

        // Update backend renderer dimensions
        self.backend.resize(width as usize, height as usize);

        // Clear any cached layout since dimensions changed - force full re-layout
        self.tree = crate::render::tree::RenderTree::new();

        Ok(())
    }

    /// Get the number of resize events processed
    pub fn resize_count(&self) -> usize {
        self.resize_count
    }

    /// Get access to the animation manager
    pub fn animation_manager(&mut self) -> &mut AnimationManager {
        &mut self.animation_manager
    }

    /// Get read-only access to the animation manager
    pub fn animation_manager_ref(&self) -> &AnimationManager {
        &self.animation_manager
    }

    /// Stop the application gracefully
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Cleanup all component instances (called automatically on drop)
    pub fn cleanup(&mut self) -> crate::error::Result<usize> {
        crate::component::registry::global_cleanup_all()
    }
}

impl Drop for App {
    fn drop(&mut self) {
        // Ensure all components are cleaned up when app is dropped
        if let Err(e) = self.cleanup() {
            #[cfg(debug_assertions)]
            eprintln!("Warning: Failed to cleanup components during App drop: {}", e);
        }
    }
}

/// Builder for creating an App instance
pub struct AppBuilder {
    backend: Option<Box<dyn Backend>>,
    root: Option<Box<dyn RootComponent>>,
    debug: bool,
    performance_mode: PerformanceMode,
    adaptive_config: Option<AdaptiveConfig>,
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self {
            backend: None,
            root: None,
            debug: false,
            performance_mode: PerformanceMode::Balanced,
            adaptive_config: None,
        }
    }
}

impl AppBuilder {
    /// Set the backend implementation
    pub fn backend(mut self, backend: impl Backend + 'static) -> Self {
        self.backend = Some(Box::new(backend));
        self
    }

    /// Set the root component
    pub fn root(mut self, root: impl RootComponent + 'static) -> Self {
        self.root = Some(Box::new(root));
        self
    }

    /// Enable debug mode
    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Set performance mode
    pub fn performance_mode(mut self, mode: PerformanceMode) -> Self {
        self.performance_mode = mode;
        self
    }

    /// Set custom adaptive FPS configuration
    pub fn adaptive_config(mut self, config: AdaptiveConfig) -> Self {
        self.adaptive_config = Some(config);
        self
    }

    /// Build the App instance
    pub fn build(self) -> Result<App> {
        let backend = self.backend.ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "Backend is required")
        })?;

        let root = self.root.ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Root component is required",
            )
        })?;

        let mut fps_config = self.adaptive_config.unwrap_or_default();
        fps_config.mode = self.performance_mode;
        let mut fps_manager = AdaptiveFpsManager::with_config(fps_config);
        fps_manager.set_performance_mode(self.performance_mode);

        Ok(App {
            backend,
            root,
            scheduler: Scheduler::new(),
            router: EventRouter::new(),
            tree: RenderTree::new(),
            previous_tree: RenderTree::new(),
            reconciler: Reconciler::new(),
            fps_manager,
            animation_manager: AnimationManager::new(),
            running: false,
            debug: self.debug,
            last_frame_time: Instant::now(),
            resize_count: 0,
        })
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::DebugBackend;

    struct TestComponent {
        counter: i32,
    }

    impl RootComponent for TestComponent {
        fn render(&self) -> Element {
            Element::text(format!("Counter: {}", self.counter))
        }
    }

    #[test]
    fn test_app_builder() {
        let backend = DebugBackend::new(80, 24);
        let component = TestComponent { counter: 0 };

        let app = App::builder()
            .backend(backend)
            .root(component)
            .debug(true)
            .build();

        assert!(app.is_ok());
    }

    #[test]
    fn test_app_size() {
        let backend = DebugBackend::new(80, 24);
        let component = TestComponent { counter: 0 };

        let app = App::builder()
            .backend(backend)
            .root(component)
            .build()
            .unwrap();

        assert_eq!(app.size(), (80, 24));
    }

    #[test]
    fn test_event_processing_integration() {
        let backend = DebugBackend::new(80, 24);
        let component = TestComponent { counter: 0 };

        let mut app = App::builder()
            .backend(backend)
            .root(component)
            .build()
            .unwrap();

        use crate::event::router::NodeId;
        let id1 = NodeId::new();

        app.register_focusable(id1, Some(0));
        app.set_initial_focus(id1);

        assert_eq!(app.router.get_focus(), Some(id1));

        // Test that the router now handles event processing
        use crate::event::types::{Event, KeyCode, KeyEvent};
        let key_event = Event::Key(KeyEvent::new(KeyCode::Space));
        let result = app.router.process_event(&key_event);

        // Event processing should work (even if not handled by any component)
        assert!(matches!(result, crate::event::router::EventResult::Ignored));
    }

    #[test]
    fn test_animation_integration() {
        let backend = DebugBackend::new(80, 24);
        let component = TestComponent { counter: 0 };

        let mut app = App::builder()
            .backend(backend)
            .root(component)
            .build()
            .unwrap();

        // Test that animation manager is accessible
        assert_eq!(app.animation_manager_ref().active_count(), 0);

        // Add a test animation
        use crate::animation::fade_in;
        use std::time::Duration;
        let animation = fade_in("test", Duration::from_millis(500));
        let _id = app.animation_manager().add_animation(animation);

        assert_eq!(app.animation_manager_ref().active_count(), 1);
    }
}
