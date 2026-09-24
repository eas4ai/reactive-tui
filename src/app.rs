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
use crate::render::{Reconciler, RenderTree};
use std::sync::Arc;
use std::time::Instant;

pub(crate) fn resume_caught_panic<T>(context: &str, result: std::thread::Result<T>) -> T {
    match result {
        Ok(value) => value,
        Err(payload) => {
            let message = panic_message(payload.as_ref());
            log::error!("{context} panicked: {message}");
            std::panic::resume_unwind(payload);
        }
    }
}

pub(crate) fn panic_message(payload: &(dyn std::any::Any + Send)) -> &str {
    payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("non-string panic payload")
}

pub(crate) fn cleanup_after_panic(context: &str, cleanup: impl FnOnce() -> Result<()>) {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(cleanup)) {
        Ok(Ok(())) => {}
        Ok(Err(error)) => log::warn!("{context} panic cleanup failed: {error}"),
        Err(_) => log::warn!("{context} cleanup panicked while handling the original failure"),
    }
}

pub(crate) mod event_tree;
pub(crate) mod focus_manager;
mod motion;
mod performance;
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
    reconciler: Reconciler,
    fps_manager: AdaptiveFpsManager,
    performance: performance::Owner,
    updaters: crate::ui::UpdateRegistry,
    last_render_duration: std::time::Duration,
    last_presented: Option<Instant>,
    /// The animation targets of the frame before the last present: the last
    /// frame whose flush a present acknowledged, republished when a present
    /// reports a failure (PIP-002).
    acknowledged_targets: crate::animation::PresentedTargets,
    animation_manager: AnimationManager,
    animation_targets: crate::animation::TargetRegistry,
    motion: motion::MotionTree,
    focus_manager: FocusManager,
    #[cfg(target_os = "linux")]
    accessibility: Option<crate::accessibility::Connection>,
    #[cfg(target_os = "linux")]
    accessibility_required: bool,
    #[cfg(target_os = "linux")]
    accessibility_snapshot: crate::accessibility::Snapshot,
    #[cfg(target_os = "linux")]
    accessibility_name: String,
    running: bool,
    debug: bool,
    last_frame_time: Instant,
    resize_count: usize,
    /// Whether the last presented frame came from the root's cell frame
    /// rather than its element tree; such a root has no layout to prepare.
    presents_cells: bool,
    quit_key: Option<(
        crate::event::types::KeyCode,
        crate::event::types::KeyModifiers,
    )>,
}

impl App {
    /// Make `theme` the active theme for every component, utility class and
    /// chart drawn by this application.
    pub fn set_theme(&mut self, theme: crate::theme::Theme) {
        crate::theme::Theme::set_active(theme);
    }

    /// Register a refresh callback owned by this App. Keep the returned token alive.
    /// Requests coalesce and run in registration order before a subsequent render.
    pub fn register_updater(
        &mut self,
        updater: impl crate::ui::Updater + 'static,
    ) -> crate::ui::UpdateRegistration {
        self.updaters.register(updater, self.wake.clone())
    }

    /// Create a new application builder
    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    /// Run the application main loop
    pub fn run(mut self) -> Result<()> {
        crate::reactive::local_hooks::with_current_scope(move || {
            let result =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.run_scoped()));
            if let Err(payload) = &result {
                cleanup_after_panic("Terminal", || {
                    self.backend
                        .shutdown_after_panic(panic_message(payload.as_ref()))
                });
            }
            drop(self);
            resume_caught_panic("Application", result)
        })
    }

    fn run_scoped(&mut self) -> Result<()> {
        #[cfg(unix)]
        let _termination_signals =
            crate::platform::unix::register_termination_waker(self.wake.clone())?;
        let result = self.run_loop();
        self.updaters.close();
        #[cfg(target_os = "linux")]
        let result = match self
            .accessibility
            .take()
            .map(|mut connection| connection.close())
        {
            Some(Err(cleanup)) if self.accessibility_required => match result {
                Err(error) => Err(crate::error::ReactiveError::invalid_state(format!(
                    "{error}; screen-reader cleanup: {cleanup}"
                ))),
                Ok(()) => Err(cleanup),
            },
            Some(Err(cleanup)) => {
                log::warn!("automatically selected screen-reader integration disabled: {cleanup}");
                result
            }
            _ => result,
        };
        self.wake.close();
        self.performance.close();
        self.publish_performance_context();
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
        self.updaters.dispatch()?;
        self.render()?;
        self.fps_manager.benchmark_if_needed(&self.tree);
        self.publish_performance_context();
        let mut next_frame = Instant::now() + self.fps_manager.get_frame_duration();
        let mut dirty = false;
        while self.running {
            let requests = self.wake.take();
            if requests.stop {
                break;
            }
            dirty |= requests.redraw;
            dirty |= self.updaters.dispatch()?;
            if let Some(mode) = self.performance.take_request() {
                if mode != self.fps_manager.performance_mode() {
                    self.fps_manager.set_performance_mode(mode);
                    dirty = true;
                }
            }
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
                self.render()?;
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
        if self.accessibility.is_some() {
            use crate::event::types::FocusEventKind;
            let focused = match event {
                Event::Focus(focus) if focus.kind == FocusEventKind::Lost => Some(false),
                Event::Focus(focus) if focus.kind == FocusEventKind::Gained => Some(true),
                Event::Key(_) | Event::Mouse(_) => Some(true),
                _ => None,
            };
            if let Some(focused) = focused {
                let result = self.accessibility.as_mut().unwrap().focus(focused);
                self.resolve_accessibility(result)?;
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
                    self.lay_out_after_resize()?;
                    self.render()?;
                }
                dirty = true;
            }
            _ => {}
        }
        let state = (self.router.get_focus(), self.router.hovered_node());
        if let Event::Mouse(mouse) = event {
            let target = self.router.determine_target(event);
            let component = self.event_tree.innermost_component(target);
            self.components.process_mouse_event(component, mouse);
        }
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
    fn resolve_accessibility<T>(&mut self, result: Result<T>) -> Result<Option<T>> {
        match result {
            Ok(value) => Ok(Some(value)),
            Err(error) if self.accessibility_required => Err(error),
            Err(error) => {
                log::warn!("automatically selected screen-reader integration disabled: {error}");
                self.accessibility.take();
                Ok(None)
            }
        }
    }

    #[cfg(target_os = "linux")]
    fn publish_accessibility(&mut self) -> Result<()> {
        let Some(result) = self
            .accessibility
            .as_mut()
            .map(|connection| connection.publish(&self.accessibility_snapshot))
        else {
            return Ok(());
        };
        self.resolve_accessibility(result)?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn process_accessibility_actions(&mut self) -> Result<bool> {
        use crate::event::{
            types::{MouseButton, MouseEvent, MouseEventKind, Position},
            Event,
        };
        let actions = match self
            .accessibility
            .as_ref()
            .map(|connection| connection.actions())
        {
            Some(result) => match self.resolve_accessibility(result)? {
                Some(actions) => actions,
                None => return Ok(false),
            },
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
        self.performance
            .publish(&self.fps_manager, self.last_render_duration);
    }

    /// Obtain this App's performance snapshots and thread-safe mode setter.
    /// Retained handles keep their last snapshots after exit; setters become inert.
    /// Global performance functions serve standalone callers and do not control Apps.
    pub fn performance_context(&self) -> Arc<crate::hooks::perf_context::PerformanceContext> {
        self.performance.context()
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
        self.publish_performance_context();
        self.wake.request_redraw();
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
    /// The backend reported the previous frame's flush failure and fell back
    /// to the last acknowledged geometry; the animation targets follow it
    /// (PIP-002).
    fn fall_back_to_acknowledged_frame(&mut self) -> Result<()> {
        self.animation_targets
            .publish_targets(self.acknowledged_targets.clone())
            .map(drop)
    }

    fn render(&mut self) -> Result<()> {
        let start = Instant::now();
        self.render_frame()?;
        let presented = Instant::now();
        self.last_render_duration = presented.duration_since(start);
        let interval = self
            .last_presented
            .replace(presented)
            .map_or(self.fps_manager.get_frame_duration(), |previous| {
                presented.duration_since(previous)
            });
        self.fps_manager.record_frame_performance(
            interval,
            self.last_render_duration,
            self.last_render_duration > self.fps_manager.get_frame_duration(),
        );
        self.publish_performance_context();
        Ok(())
    }

    /// Lay the frame out at the new size before the first frame at that
    /// size is presented, and give each component its new layout and each
    /// anchor its new bounds, so nothing paints the geometry it had before
    /// the resize (BAR-003). A backend that cannot lay out ahead of a
    /// present skips this.
    fn lay_out_after_resize(&mut self) -> Result<()> {
        if self.presents_cells {
            return Ok(());
        }
        let _scope = Scope::enter(&self.wake);
        let _hooks = self.hook_scope.enter(true);
        self.begin_frame();
        let (state_styled, styled) = self.styled_frame()?;
        if let Some(frame) = self.backend.layout_frame(std::sync::Arc::new(styled))? {
            event_tree::EventTree::publish_layouts(&state_styled, &frame.layouts);
            self.components.anchors.publish(&state_styled, &frame.nodes);
        }
        Ok(())
    }

    /// Apply a pending performance mode, publish the performance context
    /// and advance hook animations before the root renders.
    fn begin_frame(&mut self) {
        if let Some(mode) = self.performance.take_request() {
            if mode != self.fps_manager.performance_mode() {
                self.fps_manager.set_performance_mode(mode);
            }
        }
        // Publish before hooks read: this render invalidated our old subscriptions.
        self.publish_performance_context();
        assert!(
            crate::reactive::component_scope::provide((*self.performance.context()).clone())
                .is_ok()
        );

        // Update hook-based animations as part of component lifecycle
        // This ensures hooks are synchronized with component rendering
        crate::hooks::animation::update_hook_animations();
    }

    /// The frame's element tree twice: with interaction-state styles, as
    /// events and accessibility read it, and with motion and animation
    /// targets applied, as the backend paints it.
    fn styled_frame(&mut self) -> Result<(Element, Element)> {
        let mut element = self.components.resolve(self.root.render())?;
        crate::component::bridge::resolve_viewport_styles(&mut element, self.backend.size().0)?;
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
        self.animation_targets.apply(&mut styled)?;
        Ok((state_styled, styled))
    }

    fn render_frame(&mut self) -> Result<()> {
        let _scope = Scope::enter(&self.wake);
        let _hooks = self.hook_scope.enter(true);
        self.begin_frame();
        use crate::render::tree::resolved_element_to_render_node;

        if let Some(frame) = self.root.cell_frame()? {
            self.presents_cells = true;
            self.motion.clear();
            self.components.clear();
            self.event_tree.clear(&mut self.router);
            self.focus_manager
                .apply(&mut self.router, focus_manager::FocusPlan::default());
            self.backend.render_cells(frame)?;
            self.backend.present()?;
            self.animation_targets.clear();
            #[cfg(target_os = "linux")]
            {
                self.accessibility_snapshot =
                    crate::accessibility::Snapshot::empty(&self.accessibility_name);
                self.publish_accessibility()?;
            }
            return Ok(());
        }

        self.presents_cells = false;
        let (state_styled, styled) = self.styled_frame()?;
        if self.backend.render_frame(&styled)? {
            if let Err(error) = self.backend.present() {
                self.fall_back_to_acknowledged_frame()?;
                return Err(error);
            }
            let targets = crate::animation::TargetRegistry::collect(
                &styled,
                self.backend.component_layouts(),
                0,
            )?;
            self.acknowledged_targets = self.animation_targets.publish_targets(targets)?;
            let state = (self.router.get_focus(), self.router.hovered_node());
            if let Some(geometry) = self.backend.painted_nodes() {
                let anchors_changed = self.components.anchors.publish(&state_styled, geometry);
                let cell_hits = self
                    .backend
                    .hit_cells()
                    .map(|hits| (hits, self.backend.size().0));
                let (focus, layout_changed) = self.event_tree.sync(
                    &state_styled,
                    geometry,
                    self.backend.component_layouts(),
                    cell_hits,
                    &mut self.router,
                );
                self.focus_manager.apply(&mut self.router, focus);
                #[cfg(target_os = "linux")]
                if self.accessibility.is_some() {
                    self.accessibility_snapshot = self.event_tree.accessibility_snapshot(
                        &state_styled,
                        geometry,
                        &self.router,
                        &self.accessibility_name,
                    );
                    self.publish_accessibility()?;
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
            let mut presented = RenderTree::new();
            presented.set_root(resolved_element_to_render_node(styled));
            self.tree = presented;
            return Ok(());
        }

        #[cfg(target_os = "linux")]
        if self.accessibility.is_some() {
            return Err(crate::error::ReactiveError::invalid_state(
                "screen-reader integration requires a complete Element frame backend",
            ));
        }
        // Patch payloads name nodes in the candidate, including the first root.
        // Keep the acknowledged tree intact until output succeeds.
        let mut candidate = RenderTree::new();
        candidate.set_root(resolved_element_to_render_node(styled.clone()));
        let changes = self.reconciler.diff(&self.tree, &candidate);
        if !changes.patches.is_empty() {
            self.backend.apply_patches(&changes.patches, &candidate)?;
        }
        self.backend.present()?;
        self.tree = candidate;

        self.animation_targets
            .publish(&styled, self.backend.component_layouts(), 0)?;

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
            log::debug!(
                "🔄 Terminal resize #{}: {}x{}",
                self.resize_count,
                width,
                height
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

    /// Obtain a lookup context for targets in this App's presented frames.
    pub fn animation_targets(&self) -> crate::animation::AnimationTargetContext {
        self.animation_targets.context()
    }

    /// Look up a target from the last successfully presented Element frame.
    pub fn animation_target(
        &self,
        id: &str,
    ) -> std::result::Result<
        crate::animation::AnimationTarget,
        crate::animation::AnimationTargetError,
    > {
        self.animation_targets.context().target(id)
    }

    /// Access animations driven by this App's event loop.
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
        self.animation_targets.clear();
        self.hook_scope.close();
        crate::component::registry::global_cleanup_all().map(|legacy| owned + legacy)
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.updaters.close();
        #[cfg(target_os = "linux")]
        self.accessibility.take();
        self.wake.close();
        self.performance.close();
        self.scheduler.clear();
        // Ensure all components are cleaned up when app is dropped
        // Log errors but don't panic in Drop (following Rust best practices)
        if let Err(e) = self.cleanup() {
            log::warn!(
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

#[cfg(target_os = "linux")]
fn build_accessibility(
    selection: Option<bool>,
    backend: &dyn Backend,
    name: &str,
    wake: AppWaker,
) -> Result<(Option<crate::accessibility::Connection>, bool)> {
    let required = selection == Some(true);
    let selected = selection.unwrap_or_else(|| {
        backend.is_interactive_terminal()
            && std::env::var_os("DBUS_SESSION_BUS_ADDRESS")
                .is_some_and(|address| !address.is_empty())
    });
    if !selected {
        return Ok((None, required));
    }
    match crate::accessibility::Connection::new(name, wake) {
        Ok(connection) => Ok((Some(connection), required)),
        Err(error) if required => Err(error),
        Err(error) => {
            log::warn!("automatically selected screen-reader integration disabled: {error}");
            Ok((None, required))
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
        let (accessibility, accessibility_required) = build_accessibility(
            self.accessibility,
            backend.as_ref(),
            &self.accessibility_name,
            wake.clone(),
        )?;
        Ok(App {
            backend,
            root,
            components: crate::component::runtime::ComponentRuntime::default(),
            hook_scope: crate::reactive::component_scope::ComponentScope::new(scheduler.clone()),
            scheduler,
            wake: wake.clone(),
            router: EventRouter::new_with_size(width, height),
            event_tree: event_tree::EventTree::default(),
            tree: RenderTree::new(),
            reconciler: Reconciler::new(),
            performance: performance::Owner::new(&fps_manager, wake.clone()),
            updaters: crate::ui::UpdateRegistry::default(),
            last_render_duration: std::time::Duration::ZERO,
            last_presented: None,
            acknowledged_targets: Default::default(),
            presents_cells: false,
            fps_manager,
            animation_manager: AnimationManager::new(),
            animation_targets: crate::animation::TargetRegistry::new(wake.clone()),
            motion: motion::MotionTree::default(),
            focus_manager: FocusManager::new(),
            #[cfg(target_os = "linux")]
            accessibility,
            #[cfg(target_os = "linux")]
            accessibility_required,
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
    use crate::backend::{DebugBackend, SuprTuiBackend};

    struct TestComponent {
        counter: i32,
    }

    impl RootComponent for TestComponent {
        fn render(&self) -> Element {
            Element::text(format!("Counter: {}", self.counter))
        }
    }

    struct PanicComponent;

    impl RootComponent for PanicComponent {
        fn render(&self) -> Element {
            panic!("TRL001_MAIN_PANIC")
        }
    }

    struct SignalComponent;

    impl RootComponent for SignalComponent {
        fn render(&self) -> Element {
            eprintln!("TRL002_SIGNAL_READY:{}", std::process::id());
            Element::text("waiting for termination signal")
        }
    }

    #[test]
    #[ignore = "invoked by the TRL-001 PTY mechanism"]
    fn main_thread_panic_restores_the_owned_terminal() {
        if std::env::var("REACTIVE_TUI_TRL_001_PROBE").as_deref() != Ok("main") {
            eprintln!(
                "SKIP: run by the TRL-001 PTY mechanism with REACTIVE_TUI_TRL_001_PROBE=main"
            );
            return;
        }
        let backend = SuprTuiBackend::new().expect("PTY backend must start");
        let app = App::builder()
            .backend(backend)
            .root(PanicComponent)
            .build()
            .expect("panic probe app must build");
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| app.run()));
        assert!(
            panic.is_err(),
            "main-thread panic must resume after cleanup"
        );
    }

    #[cfg(unix)]
    #[test]
    #[ignore = "invoked by the TRL-002 PTY mechanism"]
    fn termination_signal_uses_the_app_wake_path() {
        if !std::env::var("REACTIVE_TUI_TRL_002_PROBE").is_ok_and(|probe| probe.starts_with("SIG"))
        {
            eprintln!(
                "SKIP: run by the TRL-002 PTY mechanism with REACTIVE_TUI_TRL_002_PROBE=SIG..."
            );
            return;
        }
        let backend = SuprTuiBackend::new().expect("PTY backend must start");
        let app = App::builder()
            .backend(backend)
            .root(SignalComponent)
            .build()
            .expect("signal probe app must build");
        let outcome = app.run();
        assert!(
            outcome.is_ok(),
            "signal must request graceful shutdown: {outcome:?}"
        );
        eprintln!("TRL002_SIGNAL_SHUTDOWN");
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

#[cfg(test)]
mod animation_target_tests;
