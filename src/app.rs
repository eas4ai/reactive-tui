use crate::backend::Backend;
use crate::component::Element;
use crate::display::adaptive::{AdaptiveConfig, AdaptiveFpsManager};
use crate::display::monitor::PerformanceMode;
use crate::event::router::{EventResult, EventRouter};
use crate::event::{FocusDirection, FocusManager};
use crate::reactive::scheduler::Scheduler;
use crate::render::reconcile::Reconciler;
use crate::render::tree::{RenderTree, element_to_render_node};
use std::io::Result;
use std::time::{Duration, Instant};

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
    focus: FocusManager,
    tree: RenderTree,
    reconciler: Reconciler,
    fps_manager: AdaptiveFpsManager,
    running: bool,
    debug: bool,
    last_frame_time: Instant,
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
        if self.debug { self.backend.set_debug_overlay(true); }

        // Show display capabilities
        if self.debug {
            eprintln!("{}", self.fps_manager.get_recommendation_summary());
        }

        while self.running {
            let frame_start = Instant::now();
            let frame_duration = self.fps_manager.get_frame_duration();

            // Update global performance context at start of frame
            {
                use crate::hooks::perf_context::{get_global_performance_context, set_global_performance_context, PerformanceContext};
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
                // First handle focus traversal keys (do not route these to components)
                if let crate::event::types::Event::Key(ref key_event) = event {
                    use crate::event::types::KeyCode;
                    let mut focus_changed = false;
                    match key_event.code {
                        KeyCode::Tab => {
                            if let Some(id) = self.focus.focus_next() {
                                self.router.set_focus(Some(id));
                                focus_changed = true;
                            }
                        }
                        KeyCode::BackTab => {
                            if let Some(id) = self.focus.focus_previous() {
                                self.router.set_focus(Some(id));
                                focus_changed = true;
                            }
                        }
                        KeyCode::Up => {
                            if let Some(id) = self.focus.move_focus(FocusDirection::Up) {
                                self.router.set_focus(Some(id));
                                focus_changed = true;
                            }
                        }
                        KeyCode::Down => {
                            if let Some(id) = self.focus.move_focus(FocusDirection::Down) {
                                self.router.set_focus(Some(id));
                                focus_changed = true;
                            }
                        }
                        KeyCode::Left => {
                            if let Some(id) = self.focus.move_focus(FocusDirection::Left) {
                                self.router.set_focus(Some(id));
                                focus_changed = true;
                            }
                        }
                        KeyCode::Right => {
                            if let Some(id) = self.focus.move_focus(FocusDirection::Right) {
                                self.router.set_focus(Some(id));
                                focus_changed = true;
                            }
                        }
                        _ => {}
                    }
                    if focus_changed {
                        // Re-render due to focus change and continue
                        self.render()?;
                        continue;
                    }
                }

                // Route remaining events through component tree
                let result = self.router.dispatch_to_focus(&event);

                // Handle special events
                match event {
                    crate::event::types::Event::Key(key_event) => {
                        // Check for quit key (Ctrl+C or Esc)
                        if (key_event.code == crate::event::types::KeyCode::Char('c')
                            && key_event.modifiers.ctrl)
                            || key_event.code == crate::event::types::KeyCode::Escape
                        {
                            self.running = false;
                        }
                    }
                    crate::event::types::Event::Resize(_) => {
                        // Force full re-render on resize
                        self.render()?;
                        continue;
                    }
                    _ => {}
                }

                // If event was handled, trigger re-render
                if result != EventResult::Ignored {
                    self.render()?;
                }
            }

            // 2) Process scheduled updates from reactive system
            if self.scheduler.has_pending_updates() {
                self.scheduler.process_updates();
                self.render()?;
            }

            // 3) Process any timer callbacks
            self.scheduler.process_timers();

            // 4) Frame timing and adaptive FPS
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
    pub fn register_focusable(
        &mut self,
        node_id: crate::event::router::NodeId,
        tab_index: Option<i32>,
    ) {
        self.focus.register_focusable(node_id, tab_index, true);
    }

    /// Unregister a focusable node
    pub fn unregister_focusable(&mut self, node_id: crate::event::router::NodeId) {
        self.focus.unregister_focusable(node_id);
    }

    /// Set spatial info used for arrow-key navigation
    pub fn set_focus_spatial(
        &mut self,
        node_id: crate::event::router::NodeId,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) {
        self.focus.update_spatial(node_id, x, y, w, h);
    }

    /// Set initial focus and update router
    pub fn set_initial_focus(&mut self, node_id: crate::event::router::NodeId) {
        if self.focus.set_focus(Some(node_id)) {
            self.router.set_focus(Some(node_id));
        }
    }

    /// Move focus to next and update router
    pub fn focus_next(&mut self) {
        if let Some(id) = self.focus.focus_next() {
            self.router.set_focus(Some(id));
        }
    }

    /// Move focus to previous and update router
    pub fn focus_previous(&mut self) {
        if let Some(id) = self.focus.focus_previous() {
            self.router.set_focus(Some(id));
        }
    }

    /// Move focus in a direction and update router
    pub fn focus_move(&mut self, dir: crate::event::FocusDirection) {
        if let Some(id) = self.focus.move_focus(dir) {
            self.router.set_focus(Some(id));
        }
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
        // Build element tree from root component
        let element = self.root.render();

        // Convert to render tree
        let mut new_tree = RenderTree::new();
        let root_node = element_to_render_node(element);
        new_tree.set_root(root_node);

        // Reconcile to get patches
        let diff_result = self.reconciler.diff(&self.tree, &new_tree);
        let patches = diff_result.patches;

        if self.debug {
            eprintln!("App: Applying {} patches", patches.len());
        }

        // Apply patches to backend
        self.backend.apply_patches(&patches, &new_tree)?;

        // Update stored tree
        self.tree = new_tree;

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
            focus: FocusManager::default(),
            tree: RenderTree::new(),
            reconciler: Reconciler::new(),
            fps_manager,
            running: false,
            debug: self.debug,
            last_frame_time: Instant::now(),
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
    fn test_focus_tab_traversal_updates_router() {
        let backend = DebugBackend::new(80, 24);
        let component = TestComponent { counter: 0 };

        let mut app = App::builder()
            .backend(backend)
            .root(component)
            .build()
            .unwrap();

        use crate::event::router::NodeId;
        let id1 = NodeId::new();
        let id2 = NodeId::new();
        let id3 = NodeId::new();

        // Register focusable nodes in order and set initial focus
        app.register_focusable(id1, Some(0));
        app.register_focusable(id2, Some(1));
        app.register_focusable(id3, Some(2));
        app.set_initial_focus(id1);

        assert_eq!(app.router.get_focus(), Some(id1));
        app.focus_next();
        assert_eq!(app.router.get_focus(), Some(id2));
        app.focus_previous();
        assert_eq!(app.router.get_focus(), Some(id1));
    }

    #[test]
    fn test_focus_spatial_movement_updates_router() {
        let backend = DebugBackend::new(80, 24);
        let component = TestComponent { counter: 0 };

        let mut app = App::builder()
            .backend(backend)
            .root(component)
            .build()
            .unwrap();

        use crate::event::FocusDirection;
        use crate::event::router::NodeId;
        let top = NodeId::new();
        let middle = NodeId::new();
        let bottom = NodeId::new();

        app.register_focusable(top, Some(0));
        app.register_focusable(middle, Some(1));
        app.register_focusable(bottom, Some(2));
        app.set_focus_spatial(top, 10.0, 10.0, 10.0, 3.0);
        app.set_focus_spatial(middle, 10.0, 20.0, 10.0, 3.0);
        app.set_focus_spatial(bottom, 10.0, 30.0, 10.0, 3.0);

        app.set_initial_focus(middle);
        assert_eq!(app.router.get_focus(), Some(middle));

        app.focus_move(FocusDirection::Up);
        assert_eq!(app.router.get_focus(), Some(top));

        app.set_initial_focus(middle);
        app.focus_move(FocusDirection::Down);
        assert_eq!(app.router.get_focus(), Some(bottom));
    }
}
