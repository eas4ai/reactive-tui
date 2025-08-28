use crate::backend::Backend;
use crate::component::Element;
use crate::display::adaptive::{AdaptiveConfig, AdaptiveFpsManager};
use crate::display::monitor::PerformanceMode;
use crate::event::router::{EventResult, EventRouter};
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

        // Show display capabilities
        if self.debug {
            eprintln!("{}", self.fps_manager.get_recommendation_summary());
        }

        while self.running {
            let frame_start = Instant::now();
            let frame_duration = self.fps_manager.get_frame_duration();

            // 1) Poll input with timeout based on target FPS
            let poll_timeout = frame_duration.as_millis() as u64;
            if let Some(event) = self.backend.poll_event(Some(poll_timeout.min(16)))? {
                // Route event through component tree
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
}
