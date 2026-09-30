use super::*;
use crate::widgets::layout::look;
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
    /// The section whose header is under the pointer.
    hovered: Option<String>,
    motion: motion::AccordionMotion,
}

impl LiveAccordion {
    /// The section whose header the pointer is on.
    fn header_at(&self, mouse: &crate::event::MouseEvent) -> Option<String> {
        let root = self.viewport?;
        let [a, b, c, d, tx, ty] = root.transform;
        let (x, y) = (mouse.position.x() as f32, mouse.position.y() as f32);
        let (x, y) = (a * x + c * y + tx, b * x + d * y + ty);
        self.headers
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
            })
    }
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
            hovered: None,
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
        let count = config.sections.len();
        for (index, section) in config.sections.iter().enumerate() {
            let mut decoration = crate::accessibility::Node::new(crate::accessibility::Role::Label);
            decoration.set_hidden();
            let expanded = state.expanded_sections.get(&section.id) == Some(&true);
            let focused = self.focused && state.focused_section.as_ref() == Some(&section.id);
            // The header's row: `selection` while it holds the focus, `hover`
            // under the pointer; its title `foreground` (`text-muted` when
            // disabled) and its glyph `text-muted` (NAV-001).
            let fill = if focused {
                look::FOCUSED
            } else if self.hovered.as_ref() == Some(&section.id) {
                look::HOVER
            } else {
                ""
            };
            let (title_classes, glyph_classes) = if focused {
                ("", "")
            } else if section.disabled {
                (look::DISABLED, look::DISABLED)
            } else {
                (look::TEXT, look::MUTED)
            };
            let header = if let Some(custom) = &section.custom_header {
                custom.clone()
            } else {
                let mut label = String::new();
                if let Some(icon) = &section.icon {
                    label.push_str(icon);
                    label.push(' ');
                }
                label.push_str(&section.title);
                Element::text(label)
                    .with_class(format!("whitespace-pre shrink-0 {title_classes}"))
                    .with_accessibility(decoration.clone())
            };
            let mut header_children = vec![header];
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
                    .with_class(format!("whitespace-pre shrink-0 {glyph_classes}"))
                    .with_accessibility(decoration.clone()),
                );
            }
            let mut header = Element::layout(LayoutType::Flex)
                .with_key("header")
                .with_class(format!("flex flex-row h-1 shrink-0 min-w-0 w-full {fill}"))
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
            accessible.inner.set_position_in_set(index + 1);
            accessible.inner.set_size_of_set(count);
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
        let mut group = crate::accessibility::Node::new(crate::accessibility::Role::Group);
        if let Some(label) = &config.aria_label {
            group.set_label(label.clone());
        }
        Element::layout(LayoutType::Flex)
            .with_class(format!(
                "flex flex-col w-full min-w-0 {}",
                config.class.as_deref().unwrap_or("")
            ))
            .with_focus(focus)
            .with_children(sections)
            .with_accessibility(group)
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
                if matches!(mouse.kind, MouseEventKind::Move | MouseEventKind::Enter) =>
            {
                self.hovered = self.header_at(mouse);
                EventResult::Ignored
            }
            Event::Mouse(mouse) if mouse.kind == MouseEventKind::Leave => {
                self.hovered = None;
                EventResult::Ignored
            }
            Event::Mouse(mouse)
                if mouse.kind == MouseEventKind::Down && mouse.button == MouseButton::Left =>
            {
                let Some(id) = self.header_at(mouse).filter(|id| {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// NAV-004: an accordion header's node tells its label, whether it is
    /// expanded, its position and the count of sections; the accordion's
    /// own node is named only by `aria_label`.
    #[test]
    fn nav_004_an_accordion_header_tells_its_position_and_the_count_of_sections() {
        let config = AccordionBuilder::new()
            .section(AccordionSection::new("one", "Focused demo").expanded(true))
            .section(AccordionSection::new("two", "Variants"))
            .props;
        let props = LiveProps {
            config,
            seed: AccordionState::default(),
        };
        let mut live = LiveAccordion::new(props.clone());
        let state = live.initial_state(&props);
        let element = live.render(&props, &state);
        assert_eq!(
            element
                .metadata
                .accessibility
                .as_ref()
                .and_then(|node| node.inner.label()),
            None,
            "the accordion has no name when the props set none"
        );
        let headers: Vec<_> = element
            .children
            .iter()
            .map(|section| {
                section.children[0]
                    .metadata
                    .accessibility
                    .as_ref()
                    .expect("a header's node")
            })
            .collect();
        assert_eq!(headers[0].inner.label(), Some("Focused demo"));
        assert_eq!(headers[0].inner.is_expanded(), Some(true));
        assert_eq!(headers[1].inner.is_expanded(), Some(false));
        assert_eq!(headers[0].inner.position_in_set(), Some(1));
        assert_eq!(headers[1].inner.position_in_set(), Some(2));
        assert_eq!(headers[1].inner.size_of_set(), Some(2));
    }
}
