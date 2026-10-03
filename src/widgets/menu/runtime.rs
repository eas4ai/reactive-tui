//! Menu overlays can extend beyond their component root's layout rectangle.
use crate::{
    component::{Component, Element, LayoutInfo, LifecycleEvent},
    event::{router::EventResult, types::Event},
    reactive::component_scope,
};
use std::sync::{Arc, Mutex};

pub(super) trait MenuRuntime: Component<State = ()> {
    fn event(&mut self, event: &Event, props: &Self::Props) -> EventResult;
}

pub(super) struct WorldEvents<C: MenuRuntime>(Arc<Mutex<C>>);
impl<C: MenuRuntime + 'static> Component for WorldEvents<C>
where
    C::Props: Clone,
{
    type Props = C::Props;
    type State = ();
    fn new(props: Self::Props) -> Self {
        Self(Arc::new(Mutex::new(C::new(props))))
    }
    fn update(&mut self, props: &Self::Props, state: &mut ()) -> bool {
        self.0.lock().unwrap().update(props, state)
    }
    fn layout(&mut self, layout: LayoutInfo, props: &mut Self::Props, state: &mut ()) -> bool {
        self.0.lock().unwrap().layout(layout, props, state)
    }
    fn render(&self, props: &Self::Props, state: &()) -> Element {
        let mut root = self.0.lock().unwrap().render(props, state);
        let owner = self.0.clone();
        let config = props.clone();
        let scope = component_scope::current();
        root.metadata.events.push(Arc::new(move |event| {
            if !matches!(event, Event::Mouse(_)) {
                return EventResult::Ignored;
            }
            let _binding = scope.as_ref().map(|scope| scope.enter(false));
            owner.lock().unwrap().event(event, &config)
        }));
        root
    }
    fn handle_event(&mut self, event: &Event, props: &mut Self::Props, _: &mut ()) -> EventResult {
        if matches!(event, Event::Mouse(_)) {
            EventResult::Ignored
        } else {
            self.0.lock().unwrap().event(event, props)
        }
    }
    fn on_lifecycle(&mut self, event: LifecycleEvent, state: &mut ()) {
        self.0.lock().unwrap().on_lifecycle(event, state);
    }
}
