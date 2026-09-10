use super::*;
use crate::{
    builder::ElementBuilder,
    component::{ElementType, FocusProps, LayoutInfo, LayoutType, LifecycleEvent},
    event::types::{FocusEventKind, MouseButton, MouseEventKind},
    layout::style::{Direction, StyleBuilder},
};
use std::sync::{Arc, Mutex};

mod motion;

#[derive(Clone, Debug)]
pub(super) struct LiveProps {
    pub config: AccordionProps,
    pub seed: AccordionState,
}

impl PartialEq for LiveProps {
    fn eq(&self, other: &Self) -> bool {
        self.config == other.config && same_seed(&self.seed, &other.seed)
    }
}

fn same_seed(a: &AccordionState, b: &AccordionState) -> bool {
    a.expanded_sections == b.expanded_sections
        && a.focused_section == b.focused_section
        && a.last_interaction == b.last_interaction
        && a.initialized == b.initialized
}

impl Props for LiveProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub(super) struct LiveAccordion {
    viewport: Option<LayoutInfo>,
    headers: Arc<Mutex<HashMap<String, LayoutInfo>>>,
    heights: Arc<Mutex<HashMap<String, f32>>>,
    seed: AccordionState,
    focused: bool,
    motion: motion::AccordionMotion,
}

impl Component for LiveAccordion {
    type Props = LiveProps;
    type State = AccordionState;

    fn new(props: Self::Props) -> Self {
        Self {
            viewport: None,
            headers: Arc::default(),
            heights: Arc::default(),
            seed: props.seed,
            focused: false,
            motion: motion::AccordionMotion::new(),
        }
    }

    fn initial_state(&mut self, props: &Self::Props) -> Self::State {
        let mut state = props.seed.clone();
        Accordion.update(&props.config, &mut state);
        state
    }

    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        if !same_seed(&self.seed, &props.seed) {
            *state = props.seed.clone();
            self.seed = props.seed.clone();
        }
        self.heights
            .lock()
            .unwrap()
            .retain(|id, _| props.config.sections.iter().any(|s| &s.id == id));
        Accordion.update(&props.config, state)
    }

    fn layout(
        &mut self,
        layout: LayoutInfo,
        _props: &mut Self::Props,
        _state: &mut Self::State,
    ) -> bool {
        self.viewport = Some(layout);
        false
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        let config = &props.config;
        self.headers.lock().unwrap().clear();
        let fractions = self.motion.fractions(config, state);
        let mut sections = Vec::new();
        for section in &config.sections {
            let mut decoration = crate::accessibility::Node::new(crate::accessibility::Role::Label);
            decoration.set_hidden();
            let expanded = state.expanded_sections.get(&section.id) == Some(&true);
            let focused = self.focused && state.focused_section.as_ref() == Some(&section.id);
            let mut header = if let Some(custom) = &section.custom_header {
                custom.clone()
            } else {
                let mut label = String::new();
                if let Some(icon) = &section.icon {
                    label.push_str(icon);
                    label.push(' ');
                }
                label.push_str(&section.title);
                Element::text(label)
                    .with_class("whitespace-pre shrink-0")
                    .with_accessibility(decoration.clone())
            };
            if section.disabled {
                header = header.with_class("text-gray-500");
            }
            let mut header_children = vec![
                Element::text(if focused { "▶ " } else { "  " })
                    .with_class("whitespace-pre shrink-0")
                    .with_accessibility(decoration.clone()),
                header,
            ];
            if config.show_icons {
                header_children.push(
                    Element::text(format!(
                        " {}",
                        if expanded {
                            &config.collapse_icon
                        } else {
                            &config.expand_icon
                        }
                    ))
                    .with_class("whitespace-pre shrink-0")
                    .with_accessibility(decoration.clone()),
                );
            }
            let mut header = Element::layout(LayoutType::Flex)
                .with_key("header")
                .with_class("flex flex-row shrink-0 min-w-0")
                .with_children(header_children);
            let mut accessible =
                crate::accessibility::Node::new(crate::accessibility::Role::Button);
            if let Some(label) = section
                .aria_label
                .as_ref()
                .or_else(|| section.custom_header.is_none().then_some(&section.title))
            {
                accessible.set_label(label.clone());
            }
            accessible.set_expanded(expanded);
            accessible.set_clickable();
            if section.disabled {
                accessible.set_disabled();
            }
            header.metadata.accessibility = Some(accessible);
            let options = header
                .metadata
                .accessibility_options
                .get_or_insert_default();
            options.focus = state.focused_section.as_ref() == Some(&section.id);
            options.focus_event = Some(crate::event::CustomEvent::new(
                "reactive_tui.accordion.focus",
                section.id.as_bytes().to_vec(),
            ));
            let id = section.id.clone();
            let headers = self.headers.clone();
            header.metadata.layout.push(Arc::new(move |layout| {
                headers.lock().unwrap().insert(id.clone(), layout);
                false
            }));
            let mut content = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
                .styles(
                    StyleBuilder::new()
                        .display_flex()
                        .direction(Direction::Column)
                        .position_absolute()
                        .inset_top(0.0)
                        .inset_left(0.0)
                        .width_percent(100.0),
                )
                .class("shrink-0 min-w-0")
                .child(section.content.clone())
                .build();
            let heights = self.heights.clone();
            let id = section.id.clone();
            content.metadata.layout.push(Arc::new(move |layout| {
                let height = layout.size.1;
                heights.lock().unwrap().insert(id.clone(), height) != Some(height)
            }));
            let height = self
                .heights
                .lock()
                .unwrap()
                .get(&section.id)
                .copied()
                .unwrap_or(0.0);
            let fraction = fractions.get(&section.id).copied().unwrap_or(0.0);
            let mut body = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
                .styles(
                    StyleBuilder::new()
                        .display_flex()
                        .direction(Direction::Column)
                        .height_px((height * fraction).round())
                        .overflow_hidden(),
                )
                .class("relative shrink-0 min-w-0")
                .child(content)
                .build()
                .with_key("body");
            if !expanded {
                body.metadata.inert = true;
                let mut accessible =
                    crate::accessibility::Node::new(crate::accessibility::Role::Group);
                accessible.set_hidden();
                body.metadata.accessibility = Some(accessible);
            }
            sections.push(
                Element::layout(LayoutType::Flex)
                    .with_key(&section.id)
                    .with_class(format!(
                        "flex flex-col shrink-0 min-w-0 {}",
                        section.class.as_deref().unwrap_or("")
                    ))
                    .with_children(vec![header, body]),
            );
        }
        let mut focus = FocusProps::input();
        focus.focusable = config.sections.iter().any(|s| !s.disabled);
        Element::layout(LayoutType::Flex)
            .with_class(format!(
                "flex flex-col min-w-0 {}",
                config.class.as_deref().unwrap_or("")
            ))
            .with_focus(focus)
            .with_children(sections)
    }

    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        match event {
            Event::Custom(event) if event.name == "reactive_tui.accordion.focus" => {
                let Ok(id) = std::str::from_utf8(&event.data) else {
                    return EventResult::Ignored;
                };
                if !props
                    .config
                    .sections
                    .iter()
                    .any(|section| section.id == id && !section.disabled)
                {
                    return EventResult::Ignored;
                }
                state.focused_section = Some(id.to_owned());
                EventResult::Consumed
            }
            Event::Focus(focus) => {
                match focus.kind {
                    FocusEventKind::Gained => self.focused = true,
                    FocusEventKind::Lost => self.focused = false,
                    _ => return EventResult::Ignored,
                }
                EventResult::Consumed
            }
            Event::Key(key) if self.focused && props.config.keyboard_navigation => {
                Accordion.handle_keyboard_event(key, &props.config, state)
            }
            Event::Mouse(mouse)
                if matches!(mouse.kind, MouseEventKind::Down | MouseEventKind::Click)
                    && mouse.button == MouseButton::Left =>
            {
                let Some(root) = self.viewport else {
                    return EventResult::Ignored;
                };
                let [a, b, c, d, tx, ty] = root.transform;
                let (x, y) = (mouse.position.x() as f32, mouse.position.y() as f32);
                let (x, y) = (a * x + c * y + tx, b * x + d * y + ty);
                let id = self
                    .headers
                    .lock()
                    .unwrap()
                    .iter()
                    .find_map(|(id, layout)| {
                        let clip = layout.clip;
                        (x >= clip.x
                            && y >= clip.y
                            && x < clip.x + clip.width
                            && y < clip.y + clip.height
                            && layout.local_cell(x, y).is_some())
                        .then(|| id.clone())
                    });
                let Some(id) = id.filter(|id| {
                    props
                        .config
                        .sections
                        .iter()
                        .any(|s| &s.id == id && !s.disabled)
                }) else {
                    return EventResult::Ignored;
                };
                state.focused_section = Some(id.clone());
                Accordion.toggle_section(&id, &props.config, state);
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }

    fn on_lifecycle(&mut self, event: LifecycleEvent, _state: &mut Self::State) {
        if event == LifecycleEvent::Unmount {
            self.motion.cancel();
        }
    }
}
