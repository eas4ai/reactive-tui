//! Screen-owned component resources and acknowledged event geometry.

use crate::{
    app::{
        event_tree::EventTree,
        focus_manager::{FocusManager, FocusPlan},
    },
    backend::Backend,
    component::{runtime::ComponentRuntime, Element},
    error::Result,
    event::{
        router::{EventResult, EventRouter},
        types::Event,
    },
    reactive::{
        component_scope::ComponentScope,
        wake::{AppWaker, Scope},
        Scheduler,
    },
    render::{tree::resolved_element_to_render_node, Reconciler, RenderTree},
};
use std::sync::Arc;

pub(super) struct ScreenRuntime {
    components: ComponentRuntime,
    scheduler: Arc<Scheduler>,
    scope: Arc<ComponentScope>,
    wake: AppWaker,
    pub(super) animation_targets: crate::animation::TargetRegistry,
    events: EventTree,
    focus: FocusManager,
    router: EventRouter,
    reconciler: Reconciler,
}

impl ScreenRuntime {
    pub(super) fn new() -> Self {
        let scheduler = Arc::new(Scheduler::new());
        let wake = AppWaker::new();
        Self {
            components: ComponentRuntime::default(),
            scope: ComponentScope::new(scheduler.clone()),
            scheduler,
            animation_targets: crate::animation::TargetRegistry::new(wake.clone()),
            wake,
            events: EventTree::default(),
            focus: FocusManager::new(),
            router: EventRouter::new(),
            reconciler: Reconciler::new(),
        }
    }

    pub(super) fn prepare(&mut self, element: Element, width: u16) -> Result<Element> {
        let _wake = Scope::enter(&self.wake);
        let _scope = self.scope.enter(true);
        self.scheduler.process_updates();
        self.scheduler.run_ready_timers();
        crate::hooks::animation::update_hook_animations();
        let mut element = self.components.resolve(element)?;
        crate::component::bridge::resolve_viewport_styles(&mut element, width)?;
        crate::accessibility::style::prepare(&mut element)?;
        let mut element = self.events.styled(&element, &self.router, width);
        crate::accessibility::style::prepare(&mut element)?;
        self.animation_targets.apply(&mut element)?;
        Ok(element)
    }

    pub(super) fn render(
        &mut self,
        element: Element,
        backend: &mut dyn Backend,
        presented: &mut RenderTree,
    ) -> Result<()> {
        let element = self.prepare(element, backend.size().0)?;
        self.present(&element, backend, presented)?;
        self.animation_targets
            .publish(&element, backend.component_layouts(), 0)?;
        self.acknowledge(&element, backend);
        Ok(())
    }

    pub(super) fn present(
        &mut self,
        element: &Element,
        backend: &mut dyn Backend,
        presented: &mut RenderTree,
    ) -> Result<()> {
        let mut tree = RenderTree::new();
        tree.set_root(resolved_element_to_render_node(element.clone()));
        if !backend.render_frame(element)? {
            let patches = self.reconciler.diff(presented, &tree).patches;
            backend.apply_patches(&patches, &tree)?;
        }
        backend.present()?;
        *presented = tree;
        Ok(())
    }

    pub(super) fn acknowledge(&mut self, element: &Element, backend: &dyn Backend) {
        let _wake = Scope::enter(&self.wake);
        if let Some(nodes) = backend.painted_nodes() {
            self.components.anchors.publish(element, nodes);
            let cell_hits = backend.hit_cells().map(|hits| (hits, backend.size().0));
            let (focus, _) = self.events.sync(
                element,
                nodes,
                backend.component_layouts(),
                cell_hits,
                &mut self.router,
            );
            self.focus.apply(&mut self.router, focus);
        } else {
            self.components.anchors.clear();
            self.events.clear(&mut self.router);
            self.focus.apply(&mut self.router, FocusPlan::default());
        }
    }

    /// The source subtree starts after the composition root and its layer wrapper.
    /// Keep event identities local while using the actual transformed geometry.
    pub(super) fn acknowledge_layer(&mut self, element: &Element, backend: &dyn Backend) {
        let range = 2..2 + super::composition::node_count(element);
        let nodes = backend
            .painted_nodes()
            .unwrap_or_default()
            .iter()
            .filter(|node| range.contains(&node.element_index))
            .map(|node| {
                let mut node = node.clone();
                node.element_index -= 2;
                node
            })
            .collect::<Vec<_>>();
        let layouts = backend.component_layouts().map(|layouts| {
            layouts
                .iter()
                .filter(|node| range.contains(&node.element_index))
                .map(|node| {
                    let mut node = *node;
                    node.element_index -= 2;
                    node
                })
                .collect::<Vec<_>>()
        });
        let _wake = Scope::enter(&self.wake);
        self.components.anchors.publish(element, &nodes);
        let (focus, _) =
            self.events
                .sync(element, &nodes, layouts.as_deref(), None, &mut self.router);
        self.focus.apply(&mut self.router, focus);
    }

    pub(super) fn process_event(&mut self, event: &Event) -> bool {
        let _wake = Scope::enter(&self.wake);
        let _scope = self.scope.enter(false);
        let notifications = crate::event::notifications::Dispatch::enter();
        let before = (self.router.get_focus(), self.router.hovered_node());
        let mut handled = self.router.process_event(event) != EventResult::Ignored;
        handled |= before != (self.router.get_focus(), self.router.hovered_node());
        for notification in notifications.take() {
            handled |=
                self.router.process_event(&Event::Custom(notification)) != EventResult::Ignored;
        }
        handled
    }
}

impl Drop for ScreenRuntime {
    fn drop(&mut self) {
        self.components.clear();
        self.scope.close();
        self.scheduler.clear();
        self.wake.close();
    }
}
