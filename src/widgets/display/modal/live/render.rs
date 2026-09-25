use super::*;
use crate::{
    accessibility::{Node, Role},
    builder::{self, ElementBuilder},
    component::{ElementType, FocusProps, LayoutType},
    layout::style::{Direction, StyleBuilder},
    widgets::{
        display::{overlay::local_rect, table::border},
        layout::ScrollViewBuilder,
    },
};

fn node(style: StyleBuilder, children: Vec<Element>) -> Element {
    ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
        .styles(style)
        .children(children)
        .build()
}
fn size(layout: Option<LayoutInfo>) -> (f32, f32) {
    layout.map_or((0.0, 0.0), |layout| layout.size)
}

impl Runtime {
    fn measure(self: &Arc<Self>, element: &mut Element, part: Part) {
        let owner = self.clone();
        element
            .metadata
            .layout
            .push(Arc::new(move |layout| owner.record(part, layout)));
    }
    pub(super) fn render(
        self: &Arc<Self>,
        props: &ModalProps,
        role: Role,
        escape_closable: bool,
        motion: Option<&Motion>,
        on_presented: Option<&Arc<dyn Fn() + Send + Sync>>,
    ) -> Element {
        if let Some(error) = validation_error(props) {
            self.cancel();
            return Element::text(error).class("text-red-500");
        }
        let (visible, progress, measured) = self.sample_with_duration(
            props,
            Instant::now(),
            motion.map_or(Duration::from_millis(200), |motion| motion.duration),
        );
        let mut children = Vec::new();
        if let Some(root) = measured.root.filter(|_| visible || progress > 0.0) {
            let bounds = local_rect(
                root,
                Rect {
                    left: root.clip.x,
                    top: root.clip.y,
                    right: root.clip.x + root.clip.width,
                    bottom: root.clip.y + root.clip.height,
                },
            );
            let viewport = (
                (bounds.right - bounds.left).max(0.0) as u16,
                (bounds.bottom - bounds.top).max(0.0) as u16,
            );
            let inset = if border::enabled(&props.border) {
                1.0
            } else {
                0.0
            };
            let insets = measured.body.map_or([inset; 4], |layout| layout.insets);
            let header_height = if props.title.is_some() || props.closable {
                size(measured.header).1.max(1.0)
            } else {
                0.0
            };
            let footer_height = if props.footer.is_some() || !props.buttons.is_empty() {
                size(measured.footer).1 + size(measured.buttons).1
            } else {
                0.0
            };
            let natural_width = size(measured.content)
                .0
                .max(size(measured.title).0 + if props.closable { 2.0 } else { 0.0 })
                .max(size(measured.footer).0)
                .max(size(measured.buttons).0)
                + insets[0]
                + insets[2];
            let natural_height =
                size(measured.content).1 + header_height + footer_height + insets[1] + insets[3];
            let mut config = props.clone();
            if config.width == ModalSize::Auto {
                config.width =
                    ModalSize::Fixed(if measured.content.is_some() || props.content.is_none() {
                        natural_width.ceil().max(1.0) as u16
                    } else {
                        viewport.0
                    });
            }
            if config.height == ModalSize::Auto {
                let width = Modal
                    .calculate_dimensions(&config, viewport.0, viewport.1)
                    .0;
                let content_width = (f32::from(width) - insets[0] - insets[2]).max(0.0);
                let scrollbar_height =
                    if props.scrollable && size(measured.content).0 > content_width {
                        1.0
                    } else {
                        0.0
                    };
                config.height =
                    ModalSize::Fixed(if measured.content.is_some() || props.content.is_none() {
                        (natural_height + scrollbar_height).ceil().max(1.0) as u16
                    } else {
                        viewport.1
                    });
            }
            let (dragged_position, resized_size) = {
                let data = self.data.lock().unwrap();
                (data.position, data.size)
            };
            let dimensions = resized_size
                .unwrap_or_else(|| Modal.calculate_dimensions(&config, viewport.0, viewport.1));
            let dimensions = (dimensions.0.min(viewport.0), dimensions.1.min(viewport.1));
            let position = dragged_position
                .unwrap_or_else(|| Modal.calculate_position(&config, dimensions, viewport));
            let position = (
                position.0.min(viewport.0.saturating_sub(dimensions.0)),
                position.1.min(viewport.1.saturating_sub(dimensions.1)),
            );
            let (left, top) = (
                bounds.left + f32::from(position.0),
                bounds.top + f32::from(position.1),
            );
            {
                let mut data = self.data.lock().unwrap();
                data.rect = Rect {
                    left,
                    top,
                    right: left + f32::from(dimensions.0),
                    bottom: top + f32::from(dimensions.1),
                };
                data.bounds = bounds;
                data.observed.position = position;
                data.observed.size = dimensions;
            }
            let ready = measured.body.is_some() && dimensions.0 > 0 && dimensions.1 > 0;
            let owner = self.clone();
            let config = props.clone();
            let mut backdrop = node(
                StyleBuilder::new()
                    .position_absolute()
                    .inset_left(bounds.left)
                    .inset_top(bounds.top)
                    .size_px(Some(f32::from(viewport.0)), Some(f32::from(viewport.1)))
                    .z_index(i32::from(props.z_index))
                    .opacity(progress),
                vec![],
            )
            .class(props.backdrop_style.clone().unwrap_or_default())
            .with_key("modal-backdrop");
            backdrop.metadata.inert = !visible;
            backdrop.metadata.events.push(Arc::new(move |event| {
                if events::activate(event) && config.closable && config.backdrop_clickable {
                    owner.close(&config, ModalCloseReason::BackdropClick);
                }
                if matches!(event, Event::Mouse(_)) {
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            }));
            if props.backdrop_style.is_some() || props.focus_trap {
                children.push(backdrop);
            }
            let available = (
                (f32::from(dimensions.0) - insets[0] - insets[2]).max(0.0),
                (f32::from(dimensions.1) - insets[1] - insets[3] - header_height - footer_height)
                    .max(0.0),
            );
            let mut body_children = Vec::new();
            if let Some(header) = self.header(props) {
                body_children.push(header);
            }
            let mut content = props.content.clone().unwrap_or_else(Element::empty);
            let mut class = content.class.take().unwrap_or_default();
            class.push_str(" self-start");
            content.class = Some(class);
            self.measure(&mut content, Part::Content);
            let content = if props.scrollable {
                ScrollViewBuilder::new(content)
                    .viewport_size(available.0 as usize, available.1 as usize)
                    .scroll_x(true)
                    .scroll_y(true)
                    .show_scrollbars(
                        size(measured.content).0 > available.0
                            || size(measured.content).1 > available.1,
                    )
                    .render()
            } else {
                content
            };
            let content = node(
                StyleBuilder::new()
                    .width_px(available.0)
                    .height_px(available.1)
                    .flex_shrink(0.0)
                    .overflow_hidden(),
                vec![content],
            )
            .class(props.content_style.clone().unwrap_or_default())
            .with_key("modal-content");
            body_children.push(content);
            if let Some(footer) = &props.footer {
                let mut footer = node(StyleBuilder::new().flex_shrink(0.0), vec![footer.clone()])
                    .class(format!(
                        "self-start {}",
                        props.footer_style.as_deref().unwrap_or("")
                    ))
                    .with_key("modal-footer");
                self.measure(&mut footer, Part::Footer);
                body_children.push(footer);
            }
            if !props.buttons.is_empty() {
                body_children.push(self.buttons(props));
            }
            body_children.extend(border::elements(
                &props.border,
                dimensions.0 as usize,
                dimensions.1 as usize,
            ));
            if props.resizable && visible && ready {
                body_children.extend(self.handles(dimensions));
            }
            let mut style = StyleBuilder::new()
                .position_absolute()
                .direction(Direction::Column)
                .inset_left(left)
                .inset_top(top)
                .size_px(Some(f32::from(dimensions.0)), Some(f32::from(dimensions.1)))
                .overflow_hidden()
                .z_index(i32::from(props.z_index) + 1);
            if border::enabled(&props.border) {
                style = style.padding_all_px(1.0);
            }
            if !ready {
                style = style.opacity(0.0);
            } else if let Some(motion) = motion {
                if let Err(error) = (motion.apply)(progress, &mut style) {
                    self.cancel();
                    return Element::text(error).class("text-red-500");
                }
            } else {
                match props.animation {
                    ModalAnimation::Fade => style = style.opacity(progress),
                    ModalAnimation::Slide => {
                        style.motion.transform.y = -3.0 * (1.0 - progress).powi(3)
                    }
                    ModalAnimation::Scale => {
                        style.motion.transform.scale_x = 0.8 + progress * 0.2;
                        style.motion.transform.scale_y = 0.8 + progress * 0.2;
                    }
                    ModalAnimation::Bounce => {
                        let eased = if progress < 0.5 {
                            4.0 * progress * progress
                        } else {
                            1.0 - (1.0 - progress).powi(2)
                                * (progress * std::f32::consts::TAU).sin().abs()
                        };
                        style.motion.transform.scale_x = eased;
                        style.motion.transform.scale_y = eased;
                    }
                    ModalAnimation::None => {}
                }
            }
            let mut semantic = Node::new(role);
            if let Some(title) = &props.title {
                semantic.set_label(title);
            }
            let mut body = node(style, body_children)
                .class(props.modal_style.clone().unwrap_or_default())
                .with_key("modal-dialog")
                .with_accessibility(semantic);
            body.metadata.inert = !visible || !ready;
            if visible && ready {
                if props.focus_trap {
                    body.focus = Some(FocusProps {
                        auto_focus: props.auto_focus,
                        ..FocusProps::modal()
                    });
                } else if props.auto_focus {
                    body.metadata.focus_scope = true;
                    body.focus = Some(FocusProps {
                        tab_index: -1,
                        ..Default::default()
                    });
                }
            }
            body.metadata.events.push(Arc::new(|event| {
                if matches!(event, Event::Mouse(_)) {
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            }));
            self.measure(&mut body, Part::Body);
            // App invokes layout callbacks only after successful presentation.
            // Measurement frames and entrance animation do not consume a
            // notification's display time.
            if visible && ready && progress == 1.0 {
                if let Some(callback) = on_presented {
                    let callback = callback.clone();
                    body.metadata.layout.push(Arc::new(move |_| {
                        callback();
                        false
                    }));
                }
            }
            children.push(body);
        }
        let mut root = node(
            StyleBuilder::new()
                .position_absolute()
                .width_px(0.0)
                .height_px(0.0),
            children,
        )
        .with_key("modal-owner");
        self.measure(&mut root, Part::Root);
        let owner = self.clone();
        root.metadata
            .capture_events
            .push(Arc::new(move |event| owner.drag(event)));
        let owner = self.clone();
        let config = props.clone();
        root.metadata.events.push(Arc::new(move |event| {
            owner.escape(event, &config, escape_closable)
        }));
        root
    }
    fn header(self: &Arc<Self>, props: &ModalProps) -> Option<Element> {
        if props.title.is_none() && !props.closable {
            return None;
        }
        let mut children = Vec::new();
        if let Some(title) = &props.title {
            let mut title = Element::text(title)
                .class("self-start")
                .with_key("modal-title");
            self.measure(&mut title, Part::Title);
            children.push(title);
        }
        if props.closable {
            let owner = self.clone();
            let config = props.clone();
            let mut close = builder::button()
                .text("✕")
                .class(&format!(
                    "absolute right-0 top-0 w-1 h-1 p-0 {}",
                    props.close_button_style.as_deref().unwrap_or("")
                ))
                .on_click(move || owner.close(&config, ModalCloseReason::CloseButton))
                .build()
                .with_key("modal-close");
            close.focus = Some(FocusProps {
                focusable: props.keyboard_navigation,
                ..FocusProps::button()
            });
            let mut label = Node::new(Role::Button);
            label.set_label("Close");
            close.metadata.accessibility = Some(label);
            children.push(close);
        }
        let mut header = node(
            StyleBuilder::new()
                .direction(Direction::Row)
                .min_height_px(1.0)
                .flex_shrink(0.0),
            children,
        )
        .class(format!(
            "relative w-full {}",
            props.header_style.as_deref().unwrap_or("")
        ))
        .with_key("modal-header");
        if props.draggable {
            let owner = self.clone();
            header
                .metadata
                .events
                .push(Arc::new(move |event| owner.begin_drag(event, None)));
        }
        self.measure(&mut header, Part::Header);
        Some(header)
    }
    fn buttons(self: &Arc<Self>, props: &ModalProps) -> Element {
        let children = props
            .buttons
            .iter()
            .map(|button| {
                let owner = self.clone();
                let config = props.clone();
                let action = button.clone();
                let mut node = builder::button()
                    .text(&button.label)
                    .class(&format!(
                        "h-1 p-0 {}",
                        button.style.as_deref().unwrap_or("")
                    ))
                    .on_click(move || owner.button(&config, &action))
                    .build()
                    .with_key(&button.id);
                node.metadata.disabled = button.disabled;
                node.focus = Some(FocusProps {
                    auto_focus: button.autofocus,
                    focusable: props.keyboard_navigation,
                    ..FocusProps::button()
                });
                node
            })
            .collect();
        let mut buttons = node(
            StyleBuilder::new()
                .direction(Direction::Row)
                .gap_px(1.0, 0.0)
                .flex_wrap(true)
                .max_width_percent(100.0)
                .flex_shrink(0.0),
            children,
        )
        .class(format!(
            "self-end {}",
            props.footer_style.as_deref().unwrap_or("")
        ))
        .with_key("modal-buttons");
        self.measure(&mut buttons, Part::Buttons);
        buttons
    }
    fn handles(self: &Arc<Self>, size: (u16, u16)) -> Vec<Element> {
        use ResizeHandle::*;
        let (right, bottom) = (size.0.saturating_sub(1), size.1.saturating_sub(1));
        [
            (TopLeft, 0, 0),
            (Top, right / 2, 0),
            (TopRight, right, 0),
            (Left, 0, bottom / 2),
            (Right, right, bottom / 2),
            (BottomLeft, 0, bottom),
            (Bottom, right / 2, bottom),
            (BottomRight, right, bottom),
        ]
        .into_iter()
        .map(|(handle, x, y)| {
            let mut node = node(
                StyleBuilder::new()
                    .position_absolute()
                    .inset_left(f32::from(x))
                    .inset_top(f32::from(y))
                    .width_px(1.0)
                    .height_px(1.0)
                    .z_index(2),
                vec![],
            )
            .with_key(format!("resize-{handle:?}"));
            let owner = self.clone();
            node.metadata.events.push(Arc::new(move |event| {
                owner.begin_drag(event, Some(handle.clone()))
            }));
            node
        })
        .collect()
    }
}
