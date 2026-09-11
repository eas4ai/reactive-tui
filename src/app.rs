use crate::backend::Backend;
use crate::component::Element;
use crate::display::adaptive::{AdaptiveConfig, AdaptiveFpsManager};
use crate::display::monitor::PerformanceMode;
use crate::event::router::{EventResult, EventRouter};

use crate::animation::AnimationManager;
use crate::error::Result;
use crate::reactive::scheduler::Scheduler;
pub use crate::reactive::wake::AppWaker;
use crate::reactive::wake::Scope;
use crate::render::reconcile::PatchOp;
use crate::render::{Reconciler, RenderTree};
use std::sync::Arc;
use std::time::Instant;

pub(crate) mod event_tree;
pub(crate) mod focus_manager;
mod motion;
use focus_manager::FocusManager;

/// Trait for root components that can render to an Element
pub trait RootComponent: Send + Sync {
    /// Render the component to an Element tree
    fn render(&self) -> Element;
    /// Optional complete cell screen, used by direct cell producers.
    fn cell_frame(&self) -> Result<Option<std::sync::Arc<crate::backend::CellFrame>>> {
        Ok(None)
    }
    /// Attach this App's wake handle to background producers.
    fn attach_waker(&mut self, _wake: AppWaker) {}
    /// Whether App may read another input event. A paused root must wake App
    /// when it can accept input again; timers and stop requests remain active.
    fn accepts_input(&self) -> bool {
        true
    }
    /// Opt into idle waiting. The default preserves periodic update() polling.
    fn wake_driven(&self) -> bool {
        false
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
    /// Handle input left unhandled by the router and widget CustomEvent notifications.
    /// Notifications arrive after the originating widget handler returns; handled events redraw.
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
    components: crate::component::runtime::ComponentRuntime,
    hook_scope: Arc<crate::reactive::component_scope::ComponentScope>,
    scheduler: Arc<Scheduler>,
    wake: AppWaker,
    router: EventRouter,
    event_tree: event_tree::EventTree,
    tree: RenderTree,
    previous_tree: RenderTree,
    reconciler: Reconciler,
    fps_manager: AdaptiveFpsManager,
    animation_manager: AnimationManager,
    motion: motion::MotionTree,
    focus_manager: FocusManager,
    #[cfg(target_os = "linux")]
    accessibility: Option<crate::accessibility::Connection>,
    #[cfg(target_os = "linux")]
    accessibility_snapshot: crate::accessibility::Snapshot,
    #[cfg(target_os = "linux")]
    accessibility_name: String,
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
        #[cfg(target_os = "linux")]
        let result = match self
            .accessibility
            .take()
            .map(|mut connection| connection.close())
        {
            Some(Err(cleanup)) => match result {
                Err(error) => Err(crate::error::ReactiveError::invalid_state(format!(
                    "{error}; screen-reader cleanup: {cleanup}"
                ))),
                Ok(()) => Err(cleanup),
            },
            _ => result,
        };
        self.wake.close();
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
        self.root.attach_waker(self.wake.clone());
        let (width, height) = self.backend.size();
        self.root.resize(width, height)?;
        if self.debug {
            self.backend.set_debug_overlay(true);
        }
        self.render()?;
        self.fps_manager.benchmark_if_needed(&self.tree);
        let mut next_frame = Instant::now() + self.fps_manager.get_frame_duration();
        let mut dirty = false;
        while self.running {
            let requests = self.wake.take();
            if requests.stop {
                break;
            }
            dirty |= requests.redraw;
            #[cfg(target_os = "linux")]
            {
                dirty |= self.process_accessibility_actions()?;
            }
            if self.scheduler.has_pending_updates() {
                self.scheduler.process_updates();
                dirty = true;
            }
            dirty |= self.scheduler.run_ready_timers();
            let now = Instant::now();
            if (self.animation_manager.active_count() > 0
                || crate::hooks::animation::has_hook_animations()
                || self.motion.active())
                && now >= next_frame
            {
                self.animation_manager.update();
                dirty = true;
            }
            if dirty && now >= next_frame {
                let start = Instant::now();
                self.render()?;
                let elapsed = start.elapsed();
                self.fps_manager.record_frame_performance(
                    elapsed,
                    elapsed,
                    elapsed > self.fps_manager.get_frame_duration(),
                );
                self.last_frame_time = Instant::now();
                next_frame = self.last_frame_time + self.fps_manager.get_frame_duration();
                dirty = false;
            }
            match self.root.update()? {
                RootUpdate::Unchanged => {}
                RootUpdate::Redraw => dirty = true,
                RootUpdate::Exit => break,
            }
            let mut deadline = self.scheduler.next_deadline();
            if dirty
                || (self.animation_manager.active_count() > 0
                    || crate::hooks::animation::has_hook_animations()
                    || self.motion.active())
            {
                deadline = Some(deadline.map_or(next_frame, |end| end.min(next_frame)));
            }
            if !self.root.wake_driven() {
                let poll = Instant::now() + self.fps_manager.get_frame_duration();
                deadline = Some(deadline.map_or(poll, |end| end.min(poll)));
            }
            let timeout = deadline.map(|end| end.saturating_duration_since(Instant::now()));
            if self.root.accepts_input() {
                if let Some(event) = self.backend.poll_event_with_wake(timeout, &self.wake)? {
                    dirty |= self.process_input(&event)?;
                }
            } else {
                self.wake.wait(timeout);
            }
        }
        self.running = false;
        Ok(())
    }

    fn process_input(&mut self, event: &crate::event::types::Event) -> Result<bool> {
        use crate::event::types::{Event, KeyCode, KeyEventKind};
        let notifications = crate::event::notifications::Dispatch::enter();
        #[cfg(target_os = "linux")]
        if let Some(accessibility) = &mut self.accessibility {
            use crate::event::types::FocusEventKind;
            match event {
                Event::Focus(focus) if focus.kind == FocusEventKind::Lost => {
                    accessibility.focus(false)?
                }
                Event::Focus(focus) if focus.kind == FocusEventKind::Gained => {
                    accessibility.focus(true)?
                }
                Event::Key(_) | Event::Mouse(_) => accessibility.focus(true)?,
                _ => {}
            }
        }
        let mut dirty = false;
        match event {
            Event::Key(key) => {
                let quit = self
                    .quit_key
                    .as_ref()
                    .is_some_and(|(code, modifiers)| key.matches(code.clone(), *modifiers));
                if quit && key.kind != KeyEventKind::Release {
                    self.running = false;
                }
            }
            Event::Resize(size) => {
                self.handle_resize(size.width, size.height)?;
                if size.width > 0 && size.height > 0 {
                    self.render()?;
                }
                dirty = true;
            }
            _ => {}
        }
        let state = (self.router.get_focus(), self.router.hovered_node());
        let mut result = self.router.process_event(event);
        dirty |= state != (self.router.get_focus(), self.router.hovered_node());
        if result == EventResult::Ignored && self.running {
            result = self.root.try_handle_event(event)?;
        }
        if self.quit_key.is_none() && result == EventResult::Ignored {
            if let Event::Key(key) = event {
                if key.kind != KeyEventKind::Release
                    && ((key.code == KeyCode::Char('c') && key.modifiers.ctrl)
                        || key.code == KeyCode::Escape)
                {
                    self.running = false;
                }
            }
        }
        let pending = notifications.take();
        for notification in pending {
            dirty |=
                self.root.try_handle_event(&Event::Custom(notification))? != EventResult::Ignored;
        }
        drop(notifications);
        Ok(dirty || result != EventResult::Ignored)
    }

    #[cfg(target_os = "linux")]
    fn process_accessibility_actions(&mut self) -> Result<bool> {
        use crate::event::{
            types::{MouseButton, MouseEvent, MouseEventKind, Position},
            Event,
        };
        let actions = match &self.accessibility {
            Some(accessibility) => accessibility.actions()?,
            None => return Ok(false),
        };
        let mut dirty = false;
        for action in actions {
            if !matches!(
                action.action,
                accesskit::Action::Focus | accesskit::Action::Click
            ) {
                continue;
            }
            let Some(target) = self
                .accessibility_snapshot
                .targets
                .get(&action.target_node)
                .cloned()
            else {
                continue;
            };
            if action.action == accesskit::Action::Click && !target.clickable {
                continue;
            }
            let notifications = crate::event::notifications::Dispatch::enter();
            self.router.set_focus(Some(target.owner));
            // Focus traps and disabled/removed nodes remain authoritative.
            if self.router.get_focus() != Some(target.owner) {
                continue;
            }
            if let Some(event) = target.focus_event {
                self.router.route_event(&Event::Custom(event), target.node);
            }
            if action.action == accesskit::Action::Click {
                if let Some(event) = target.click_event {
                    self.router.route_event(&Event::Custom(event), target.node);
                } else {
                    let bounds = target.bounds;
                    let event = MouseEvent::new(
                        MouseEventKind::Down,
                        Position::cell(
                            (bounds.x + bounds.width / 2.0).floor() as u16,
                            (bounds.y + bounds.height / 2.0).floor() as u16,
                        ),
                    )
                    .with_button(MouseButton::Left);
                    self.router.route_event(&Event::Mouse(event), target.node);
                }
            }
            for notification in notifications.take() {
                self.root.try_handle_event(&Event::Custom(notification))?;
            }
            dirty = true;
        }
        Ok(dirty)
    }

    fn publish_performance_context(&self) {
        {
            use crate::hooks::perf_context::{
                get_global_performance_context, set_global_performance_context, PerformanceContext,
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
                target_frame_ms: self.fps_manager.get_frame_duration().as_secs_f32() * 1000.0,
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
    }

    /// Clone the handle used to request redraws or graceful stop from other threads.
    pub fn waker(&self) -> AppWaker {
        self.wake.clone()
    }

    /// Access this App's shared work and timer scheduler.
    pub fn scheduler(&self) -> Arc<Scheduler> {
        self.scheduler.clone()
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
        self.router.focus_next();
    }

    /// Move focus to previous focusable element (declarative)  
    pub fn focus_previous(&mut self) {
        self.router.focus_prev();
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
        self.router.current_focus_ref()
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
        let _scope = Scope::enter(&self.wake);
        let _hooks = self.hook_scope.enter(true);
        self.publish_performance_context();
        use crate::render::tree::resolved_element_to_render_node;

        // Update hook-based animations as part of component lifecycle
        // This ensures hooks are synchronized with component rendering
        crate::hooks::animation::update_hook_animations();

        if let Some(frame) = self.root.cell_frame()? {
            self.motion.clear();
            self.components.clear();
            self.event_tree.clear(&mut self.router);
            self.focus_manager
                .apply(&mut self.router, focus_manager::FocusPlan::default());
            self.backend.render_cells(frame)?;
            self.backend.present()?;
            #[cfg(target_os = "linux")]
            {
                self.accessibility_snapshot =
                    crate::accessibility::Snapshot::empty(&self.accessibility_name);
                if let Some(accessibility) = &mut self.accessibility {
                    accessibility.publish(&self.accessibility_snapshot)?;
                }
            }
            return Ok(());
        }

        // Build element tree from root component
        let mut element = self.components.resolve(self.root.render())?;
        // Base semantics establish disabled state before state variants are
        // selected. Resolve again afterward for conditional semantic styles.
        crate::accessibility::style::prepare(&mut element)?;

        let mut state_styled =
            self.event_tree
                .styled(&element, &self.router, self.backend.size().0);
        crate::accessibility::style::prepare(&mut state_styled)?;
        let mut styled = state_styled.clone();
        self.motion
            .apply(&mut styled, Instant::now(), self.backend.size())?;
        if self.backend.render_frame(&styled)? {
            self.backend.present()?;
            let state = (self.router.get_focus(), self.router.hovered_node());
            if let Some(geometry) = self.backend.painted_nodes() {
                let anchors_changed = self.components.anchors.publish(&state_styled, geometry);
                let (focus, layout_changed) = self.event_tree.sync(
                    &state_styled,
                    geometry,
                    self.backend.component_layouts(),
                    &mut self.router,
                );
                self.focus_manager.apply(&mut self.router, focus);
                #[cfg(target_os = "linux")]
                if let Some(accessibility) = &mut self.accessibility {
                    self.accessibility_snapshot = self.event_tree.accessibility_snapshot(
                        &state_styled,
                        geometry,
                        &self.router,
                        &self.accessibility_name,
                    );
                    accessibility.publish(&self.accessibility_snapshot)?;
                }
                if layout_changed || anchors_changed {
                    self.wake.request_redraw();
                }
            } else {
                if self.components.anchors.clear() {
                    self.wake.request_redraw();
                }
                #[cfg(target_os = "linux")]
                if self.accessibility.is_some() {
                    return Err(crate::error::ReactiveError::invalid_state(
                        "screen-reader integration requires presented Element geometry",
                    ));
                }
                self.event_tree.clear(&mut self.router);
                self.focus_manager
                    .apply(&mut self.router, focus_manager::FocusPlan::default());
            }
            if state != (self.router.get_focus(), self.router.hovered_node()) {
                self.wake.request_redraw();
            }
            self.tree.set_root(resolved_element_to_render_node(styled));
            if let Some(req) = crate::hooks::perf_context::take_requested_performance_mode() {
                self.fps_manager.set_performance_mode(req);
            }
            return Ok(());
        }

        #[cfg(target_os = "linux")]
        if self.accessibility.is_some() {
            return Err(crate::error::ReactiveError::invalid_state(
                "screen-reader integration requires a complete Element frame backend",
            ));
        }
        // Convert to RenderTree
        let root_node = resolved_element_to_render_node(element.clone());

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
        let owned = self.components.clear();
        self.hook_scope.close();
        crate::component::registry::global_cleanup_all().map(|legacy| owned + legacy)
    }
}

impl Drop for App {
    fn drop(&mut self) {
        #[cfg(target_os = "linux")]
        self.accessibility.take();
        self.wake.close();
        self.scheduler.clear();
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
    scheduler: Option<Arc<Scheduler>>,
    accessibility: Option<bool>,
    accessibility_name: String,
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
            scheduler: None,
            accessibility: None,
            accessibility_name: "Reactive TUI".into(),
            quit_key: None,
        }
    }
}

impl AppBuilder {
    /// Enable or disable the Linux screen-reader connection.
    /// Enabled automatically for an interactive Linux terminal with a desktop
    /// session-bus address. Explicit enabling reports connection failures.
    /// Enabling on other platforms returns an error during build.
    pub fn screen_reader(mut self, enabled: bool) -> Self {
        self.accessibility = Some(enabled);
        self
    }

    /// Name of this App in the screen reader's accessible window list.
    pub fn accessibility_name(mut self, name: impl Into<String>) -> Self {
        self.accessibility_name = name.into();
        self
    }

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

    /// Use a shared scheduler. It may belong to only one live App at a time.
    pub fn scheduler(mut self, scheduler: Arc<Scheduler>) -> Self {
        self.scheduler = Some(scheduler);
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

        let wake = AppWaker::new();
        let scheduler = self.scheduler.unwrap_or_else(|| Arc::new(Scheduler::new()));
        scheduler.attach(wake.clone())?;
        let (width, height) = backend.size();
        #[cfg(not(target_os = "linux"))]
        if self.accessibility == Some(true) {
            return Err(crate::error::ReactiveError::invalid_state(
                "screen-reader integration currently supports Linux only",
            ));
        }
        #[cfg(target_os = "linux")]
        let accessibility = self
            .accessibility
            .unwrap_or_else(|| {
                backend.is_interactive_terminal()
                    && std::env::var_os("DBUS_SESSION_BUS_ADDRESS")
                        .is_some_and(|address| !address.is_empty())
            })
            .then(|| crate::accessibility::Connection::new(&self.accessibility_name, wake.clone()))
            .transpose()?;
        Ok(App {
            backend,
            root,
            components: crate::component::runtime::ComponentRuntime::default(),
            hook_scope: crate::reactive::component_scope::ComponentScope::new(scheduler.clone()),
            scheduler,
            wake,
            router: EventRouter::new_with_size(width, height),
            event_tree: event_tree::EventTree::default(),
            tree: RenderTree::new(),
            previous_tree: RenderTree::new(),
            reconciler: Reconciler::new(),
            fps_manager,
            animation_manager: AnimationManager::new(),
            motion: motion::MotionTree::default(),
            focus_manager: FocusManager::new(),
            #[cfg(target_os = "linux")]
            accessibility,
            #[cfg(target_os = "linux")]
            accessibility_snapshot: crate::accessibility::Snapshot::empty(&self.accessibility_name),
            #[cfg(target_os = "linux")]
            accessibility_name: self.accessibility_name,
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
