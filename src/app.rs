use crate::backend::Backend;
use crate::component::Element;
use crate::display::adaptive::{AdaptiveConfig, AdaptiveFpsManager};
use crate::display::monitor::PerformanceMode;
use crate::event::router::{EventResult, EventRouter};

use crate::animation::AnimationManager;
use crate::error::Result;
use crate::reactive::scheduler::Scheduler;
use crate::render::reconcile::PatchOp;
use crate::render::{Reconciler, RenderTree};
use std::time::{Duration, Instant};

mod focus_manager;
use focus_manager::FocusManager;

/// Estimated time for a single render operation in milliseconds
const RENDER_TIME_ESTIMATE_MS: u64 = 1;

/// Trait for root components that can render to an Element
pub trait RootComponent: Send + Sync {
    /// Render the component to an Element tree
    fn render(&self) -> Element;
    /// Optional complete cell screen, used by direct cell producers.
    fn cell_frame(&self) -> Result<Option<std::sync::Arc<crate::backend::CellFrame>>> {
        Ok(None)
    }
    /// Poll background state once per event-loop iteration.
    fn update(&mut self) -> Result<RootUpdate> {
        Ok(RootUpdate::Unchanged)
    }
    /// Called initially and when the application viewport changes.
    fn resize(&mut self, _width: u16, _height: u16) -> Result<()> {
        Ok(())
    }
    /// Fallible input path; existing roots retain their handle_event behavior.
    fn try_handle_event(&mut self, event: &crate::event::types::Event) -> Result<EventResult> {
        Ok(self.handle_event(event))
    }
    /// Handle input left unhandled by the event router; handled input redraws.
    fn handle_event(&self, _event: &crate::event::types::Event) -> EventResult {
        EventResult::Ignored
    }
}

/// A root component's background state transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RootUpdate {
    /// No visible state change.
    Unchanged,
    /// Publish the root's new state.
    Redraw,
    /// Leave the event loop and restore the host terminal.
    Exit,
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
    focus_manager: FocusManager,
    running: bool,
    debug: bool,
    last_frame_time: Instant,
    resize_count: usize,
    quit_key: Option<(
        crate::event::types::KeyCode,
        crate::event::types::KeyModifiers,
    )>,
}

impl App {
    /// Create a new application builder
    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    /// Run the application main loop
    pub fn run(mut self) -> Result<()> {
        let result = self.run_loop();
        let cleanup = self.backend.shutdown();
        match (result, cleanup) {
            (Err(error), Err(cleanup)) => Err(crate::error::ReactiveError::terminal(format!(
                "{error}; terminal cleanup also failed: {cleanup}"
            ))),
            (Err(error), _) | (_, Err(error)) => Err(error),
            _ => Ok(()),
        }
    }

    fn run_loop(&mut self) -> Result<()> {
        self.running = true;
        let (width, height) = self.backend.size();
        self.root.resize(width, height)?;

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
            // Use full frame_duration from adaptive FPS manager without artificial caps
            let poll_timeout = frame_duration.as_millis() as u64;
            if let Some(event) = self.backend.poll_event(Some(poll_timeout))? {
                // Handle application-level events first
                let mut should_render = false;

                match &event {
                    crate::event::types::Event::Key(key_event) => {
                        let quit = self.quit_key.as_ref().map_or_else(
                            || {
                                (key_event.code == crate::event::types::KeyCode::Char('c')
                                    && key_event.modifiers.ctrl)
                                    || key_event.code == crate::event::types::KeyCode::Escape
                            },
                            |(code, modifiers)| key_event.matches(code.clone(), *modifiers),
                        );
                        if quit && key_event.kind != crate::event::types::KeyEventKind::Release {
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
                let mut result = self.router.process_event(&event);
                if result == EventResult::Ignored && self.running {
                    result = self.root.try_handle_event(&event)?;
                }

                // Trigger re-render if event was handled or if we need to render for other reasons
                if result != EventResult::Ignored || should_render {
                    self.render()?;
                }
            }

            if !self.running {
                break;
            }
            match self.root.update()? {
                RootUpdate::Unchanged => {}
                RootUpdate::Redraw => self.render()?,
                RootUpdate::Exit => {
                    self.running = false;
                    break;
                }
            }

            // 2) Process scheduled updates from reactive system
            if self.scheduler.has_pending_updates() {
                self.scheduler.process_updates();
                self.render()?;
            }

            // 3) Update animations and check if render is needed
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
            let render_time =
                frame_elapsed.saturating_sub(Duration::from_millis(RENDER_TIME_ESTIMATE_MS));

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

    /// Move focus to next focusable element (declarative)
    pub fn focus_next(&mut self) {
        self.focus_manager.focus_next()
    }

    /// Move focus to previous focusable element (declarative)  
    pub fn focus_previous(&mut self) {
        self.focus_manager.focus_previous()
    }

    /// Move focus in a direction using router (imperative legacy)
    /// Returns the newly focused node if successful
    pub fn focus_move(
        &mut self,
        dir: crate::event::FocusDirection,
    ) -> Option<crate::event::router::NodeId> {
        self.router.focus_move(dir)
    }

    /// Get the currently focused element (declarative)
    pub fn current_focus(&self) -> Option<&crate::event::router::NodeId> {
        self.focus_manager.current_focus()
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

        // Update hook-based animations as part of component lifecycle
        // This ensures hooks are synchronized with component rendering
        crate::hooks::animation::update_hook_animations();

        if let Some(frame) = self.root.cell_frame()? {
            self.backend.render_cells(frame)?;
            return self.backend.present();
        }

        // Build element tree from root component
        let element = self.root.render();

        // Process declarative focus properties from the element tree
        let root_id = crate::event::router::NodeId::new();
        self.focus_manager.process_element_tree(&element, root_id);

        if self.backend.render_frame(&element)? {
            self.tree.set_root(element_to_render_node(element));
            self.backend.present()?;
            if let Some(req) = crate::hooks::perf_context::take_requested_performance_mode() {
                self.fps_manager.set_performance_mode(req);
            }
            return Ok(());
        }

        // Convert to RenderTree
        let root_node = element_to_render_node(element.clone());

        // Check if this is the first render
        if self.previous_tree.root().is_none() {
            // First render - set up both trees
            let mut new_tree = RenderTree::new();
            new_tree.set_root(root_node);
            self.tree = new_tree;

            // Trigger full repaint for first render
            if self.tree.root().is_some() {
                // The backend's apply_patches with >10 patches triggers full repaint
                // We can use this to our advantage for the first render
                let dummy_patches: Vec<PatchOp> = (0..11)
                    .map(|i| PatchOp::Insert {
                        parent_key: None,
                        index: 0,
                        node_key: crate::render::tree::NodeKey::index(i),
                    })
                    .collect();
                self.backend.apply_patches(&dummy_patches, &self.tree)?;
            }
        } else {
            // Create temporary tree for diffing without full allocation
            let mut temp_tree = RenderTree::new();
            temp_tree.set_root(root_node);

            // Diff against previous tree
            let diff_result = self.reconciler.diff(&self.previous_tree, &temp_tree);

            // Apply patches if there are changes
            if !diff_result.patches.is_empty() {
                // Apply patches to update the current tree
                crate::render::reconcile::apply_patches(&diff_result.patches, &mut self.tree)?;

                // Use incremental patch-based rendering for performance
                self.backend
                    .apply_patches(&diff_result.patches, &self.tree)?;
            }

            // Efficiently swap tree roots without allocating full tree structures
            // Move current tree to previous_tree, and temp_tree to current tree
            // This avoids the full memory swap of std::mem::replace
            if let Some(new_root) = temp_tree.take_root() {
                if let Some(current_root) = self.tree.replace_root(new_root) {
                    self.previous_tree.set_root(current_root);
                }
            }
        }

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

        if width == 0 || height == 0 {
            return Ok(());
        }
        self.root.resize(width, height)?;

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
        // Log errors but don't panic in Drop (following Rust best practices)
        if let Err(e) = self.cleanup() {
            // Use log crate if available, otherwise stderr
            // This ensures the error is visible in both debug and release builds
            eprintln!(
                "Warning: Failed to cleanup components during App drop: {}",
                e
            );
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
    quit_key: Option<(
        crate::event::types::KeyCode,
        crate::event::types::KeyModifiers,
    )>,
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self {
            backend: None,
            root: None,
            debug: false,
            performance_mode: PerformanceMode::Balanced,
            adaptive_config: None,
            quit_key: None,
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

    /// Reserve one explicit quit chord instead of the default Ctrl+C and Escape.
    pub fn quit_key(
        mut self,
        code: crate::event::types::KeyCode,
        modifiers: crate::event::types::KeyModifiers,
    ) -> Self {
        self.quit_key = Some((code, modifiers));
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
            focus_manager: FocusManager::new(),
            running: false,
            debug: self.debug,
            last_frame_time: Instant::now(),
            resize_count: 0,
            quit_key: self.quit_key,
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
