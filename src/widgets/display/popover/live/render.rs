use super::*;
use crate::widgets::display::overlay::{global_rect, local_rect};
use crate::{
    builder::ElementBuilder,
    component::{ElementType, FocusProps},
    layout::style::StyleBuilder,
};

fn node(style: StyleBuilder, children: Vec<Element>) -> Element {
    ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
        .styles(style)
        .children(children)
        .build()
}

impl Popover {
    pub(in super::super) fn render_live(&self, props: &PopoverProps) -> Element {
        if let Some(error) = validation_error(props) {
            self.live.reject(&self.state, props);
            self.state.lock().unwrap().arrow_position = None;
            return Element::text(error).class("text-red-500");
        }
        let (visible, progress) = self.live.sample(&self.state, props);
        self.state.lock().unwrap().arrow_position = None;
        let mut trigger_content = props.trigger_element.clone();
        let mut trigger_class = trigger_content.class.take().unwrap_or_default();
        trigger_class.push_str(if visible {
            " aria-expanded-true"
        } else {
            " aria-expanded-false"
        });
        trigger_content.class = Some(trigger_class);
        if matches!(trigger_content.element_type, ElementType::Text(_))
            && trigger_content.focus.is_none()
        {
            trigger_content.focus = Some(FocusProps::button());
        }
        if props.trigger == PopoverTrigger::Click {
            // Add the wrapper's action without replacing a typed child's role,
            // label, text runs or selection during component expansion.
            trigger_content
                .metadata
                .accessibility_options
                .get_or_insert_default()
                .clickable = true;
        }
        let trigger_disabled = trigger_content.metadata.disabled;
        let mut trigger = node(
            StyleBuilder::new().z_index(i32::from(props.z_index) + 2),
            vec![trigger_content],
        )
        .class("self-start")
        .with_key("popover-trigger");
        trigger.metadata.disabled = trigger_disabled;
        let owner = self.clone();
        let config = props.clone();
        trigger
            .metadata
            .capture_events
            .push(Arc::new(move |event| owner.observe(event, &config, true)));
        self.measure(&mut trigger, Part::Trigger);
        let mut children = vec![trigger];
        let (root_layout, body_layout, trigger_layout, explicit) = {
            let data = self.live.data.lock().unwrap();
            (data.root, data.body, data.trigger, data.explicit_trigger)
        };
        if let Some(root) = root_layout {
            let trigger_rect = if explicit {
                self.state.lock().unwrap().trigger_rect
            } else {
                trigger_layout
                    .map(screen_rect)
                    .unwrap_or_else(|| screen_rect(root))
            };
            self.state.lock().unwrap().trigger_rect = trigger_rect;
            if visible || progress > 0.0 {
                let bounds = local_rect(
                    root,
                    Rect {
                        left: root.clip.x,
                        top: root.clip.y,
                        right: root.clip.x + root.clip.width,
                        bottom: root.clip.y + root.clip.height,
                    },
                );
                let trigger_local = local_rect(root, trigger_rect);
                let measured = body_layout.map_or((0.0, 0.0), |layout| layout.size);
                let constrained = self.apply_size_constraints(
                    Self::make_rect(0.0, 0.0, measured.0, measured.1),
                    props,
                );
                let content_size = (
                    constrained.right.ceil() as u16,
                    constrained.bottom.ceil() as u16,
                );
                let placed = self.calculate_rect_for_position(
                    props.position,
                    trigger_local,
                    content_size,
                    props.offset,
                );
                let (placed, position) = self.adjust_for_boundaries(
                    placed,
                    props.position,
                    trigger_local,
                    content_size,
                    bounds,
                    props,
                );
                let ready = body_layout.is_some();
                let hidden = !ready || placed.right <= placed.left || placed.bottom <= placed.top;
                let changed_position = {
                    let mut state = self.state.lock().unwrap();
                    state.calculated_rect = global_rect(root, placed);
                    let changed = ready && state.position != position;
                    if ready {
                        state.position = position;
                        state.boundary_adjusted = placed
                            != self.calculate_rect_for_position(
                                props.position,
                                trigger_local,
                                content_size,
                                props.offset,
                            );
                    }
                    changed
                };
                if changed_position {
                    if let Some(callback) = &props.on_position_change {
                        callback(position);
                    }
                }
                if visible
                    && ready
                    && !hidden
                    && (props.close_on_outside_click || props.backdrop_filter)
                {
                    let mut shield = node(
                        StyleBuilder::new()
                            .position_absolute()
                            .inset_left(bounds.left)
                            .inset_top(bounds.top)
                            .width_px((bounds.right - bounds.left).max(0.0))
                            .height_px((bounds.bottom - bounds.top).max(0.0))
                            .z_index(i32::from(props.z_index)),
                        vec![],
                    )
                    .with_key("popover-shield");
                    if props.backdrop_filter {
                        shield.class = Some("bg-black/30".into());
                    }
                    let owner = self.clone();
                    let close = props.close_on_outside_click;
                    shield.metadata.events.push(Arc::new(move |event| {
                        if close && matches!(event, Event::Mouse(_)) && events::activation(event) {
                            owner.hide();
                            return EventResult::Consumed;
                        }
                        EventResult::Ignored
                    }));
                    children.push(shield);
                }
                let mut style = StyleBuilder::new()
                    .position_absolute()
                    .inset_left(placed.left)
                    .inset_top(placed.top)
                    .overflow_hidden()
                    .z_index(i32::from(props.z_index) + 1);
                if props.boundary_behavior != BoundaryBehavior::Ignore {
                    style = style
                        .max_width_px((bounds.right - bounds.left).max(0.0))
                        .max_height_px((bounds.bottom - bounds.top).max(0.0));
                }
                if let Some(value) = props.min_width {
                    style = style.min_width_px(f32::from(value));
                }
                if let Some(value) = props.max_width {
                    style = style.max_width_px(f32::from(value));
                }
                if let Some(value) = props.min_height {
                    style = style.min_height_px(f32::from(value));
                }
                if let Some(value) = props.max_height {
                    style = style.max_height_px(f32::from(value));
                }
                if hidden {
                    style = style.opacity(0.0);
                } else {
                    match props.animation {
                        PopoverAnimation::Fade => style = style.opacity(progress),
                        PopoverAnimation::Scale | PopoverAnimation::Bounce => {
                            let scale = if props.animation == PopoverAnimation::Bounce {
                                self.ease_out_bounce(progress)
                            } else {
                                0.8 + 0.2 * self.ease_out_back(progress)
                            };
                            style.motion.transform.scale_x = scale;
                            style.motion.transform.scale_y = scale;
                        }
                        PopoverAnimation::Slide => {
                            let offset = 3.0 * (1.0 - self.ease_out_cubic(progress));
                            match position {
                                PopoverPosition::Top
                                | PopoverPosition::TopStart
                                | PopoverPosition::TopEnd => style.motion.transform.y = offset,
                                PopoverPosition::Bottom
                                | PopoverPosition::BottomStart
                                | PopoverPosition::BottomEnd => style.motion.transform.y = -offset,
                                PopoverPosition::Left
                                | PopoverPosition::LeftStart
                                | PopoverPosition::LeftEnd => style.motion.transform.x = offset,
                                _ => style.motion.transform.x = -offset,
                            }
                        }
                        PopoverAnimation::None => {}
                    }
                }
                let arrow_style = style
                    .clone()
                    .width_px(placed.right - placed.left)
                    .height_px(placed.bottom - placed.top)
                    .z_index(i32::from(props.z_index))
                    .overflow_visible();
                let mut body = node(style, vec![props.content.clone()]).with_key("popover-body");
                body.metadata.inert = !visible || hidden;
                body.metadata.accessibility =
                    Some(crate::accessibility::Node::new(if props.focus_trap {
                        crate::accessibility::Role::Dialog
                    } else {
                        crate::accessibility::Role::Group
                    }));
                if visible && !hidden && props.focus_trap {
                    body.focus = Some(FocusProps::modal());
                } else if visible && !hidden && props.auto_focus {
                    body.metadata.focus_scope = true;
                    body.focus = Some(FocusProps {
                        tab_index: -1,
                        ..Default::default()
                    });
                }
                let owner = self.clone();
                let config = props.clone();
                body.metadata
                    .capture_events
                    .push(Arc::new(move |event| owner.observe(event, &config, false)));
                body.metadata.events.push(Arc::new(|event| {
                    if matches!(event, Event::Mouse(_)) && events::activation(event) {
                        EventResult::Consumed
                    } else {
                        EventResult::Ignored
                    }
                }));
                self.measure(&mut body, Part::Body);
                children.push(body);
                if ready && !hidden && props.arrow.enabled && props.arrow.size > 0 {
                    children.push(self.arrow(
                        props,
                        placed,
                        trigger_local,
                        position,
                        arrow_style,
                        visible,
                        root,
                    ));
                }
            }
        }
        let mut root = node(StyleBuilder::new(), children)
            .class("relative items-start")
            .with_key("popover-container");
        self.measure(&mut root, Part::Root);
        let owner = self.clone();
        let config = props.clone();
        root.metadata
            .events
            .push(Arc::new(move |event| owner.escape(event, &config)));
        root
    }

    fn measure(&self, element: &mut Element, part: Part) {
        let runtime = self.live.clone();
        element
            .metadata
            .layout
            .push(Arc::new(move |layout| runtime.record(part, layout)));
    }

    #[allow(clippy::too_many_arguments)]
    fn arrow(
        &self,
        props: &PopoverProps,
        body: Rect<f32>,
        trigger: Rect<f32>,
        position: PopoverPosition,
        style: StyleBuilder,
        visible: bool,
        root: LayoutInfo,
    ) -> Element {
        use PopoverPosition::*;
        let vertical = matches!(
            position,
            Top | TopStart | TopEnd | Bottom | BottomStart | BottomEnd
        );
        let before = matches!(
            position,
            Bottom | BottomStart | BottomEnd | Right | RightStart | RightEnd
        );
        let (lo, hi, center) = if vertical {
            (body.left, body.right, (trigger.left + trigger.right) / 2.0)
        } else {
            (body.top, body.bottom, (trigger.top + trigger.bottom) / 2.0)
        };
        let size = usize::from(props.arrow.size)
            .min(((hi - lo + 1.0) / 2.0).max(1.0) as usize)
            .min(
                if vertical {
                    root.clip.height
                } else {
                    root.clip.width
                }
                .max(1.0) as usize,
            );
        let center = (center + f32::from(props.arrow.offset))
            .clamp(lo, (hi - 1.0).max(lo))
            .floor();
        let edge = if vertical {
            if before {
                body.top
            } else {
                body.bottom
            }
        } else if before {
            body.left
        } else {
            body.right
        };
        let tip_axis = if before {
            edge - size as f32
        } else {
            edge + size as f32 - 1.0
        };
        let tip = match (props.arrow.style, vertical, before) {
            (ArrowStyle::Solid, true, true) => "▲",
            (ArrowStyle::Solid, true, false) => "▼",
            (ArrowStyle::Solid, false, true) => "◀",
            (ArrowStyle::Solid, false, false) => "▶",
            (ArrowStyle::Outline, true, true) => "△",
            (ArrowStyle::Outline, true, false) => "▽",
            (ArrowStyle::Outline, false, true) => "◁",
            (ArrowStyle::Outline, false, false) => "▷",
            (ArrowStyle::Double, true, true) => "⇈",
            (ArrowStyle::Double, true, false) => "⇊",
            (ArrowStyle::Double, false, true) => "⇇",
            (ArrowStyle::Double, false, false) => "⇉",
        };
        let mut cells = Vec::new();
        for depth in 0..size {
            for cross in -(depth as i32)..=depth as i32 {
                let boundary = cross.unsigned_abs() as usize == depth;
                if props.arrow.style != ArrowStyle::Solid && !boundary {
                    continue;
                }
                let axis = tip_axis
                    + if before {
                        depth as f32
                    } else {
                        -(depth as f32)
                    };
                let (x, y) = if vertical {
                    (center + cross as f32, axis)
                } else {
                    (axis, center + cross as f32)
                };
                let mark = if depth == 0 {
                    tip
                } else if props.arrow.style == ArrowStyle::Double {
                    "═"
                } else {
                    "█"
                };
                cells.push(
                    ElementBuilder::new(ElementType::Text(mark.into()))
                        .styles(
                            StyleBuilder::new()
                                .position_absolute()
                                .inset_left(x - body.left)
                                .inset_top(y - body.top)
                                .width_px(1.0)
                                .height_px(1.0),
                        )
                        .build(),
                );
            }
        }
        let (x, y) = if vertical {
            (center, tip_axis)
        } else {
            (tip_axis, center)
        };
        let point = global_rect(root, Self::make_rect(x, y, 0.0, 0.0));
        self.state.lock().unwrap().arrow_position = (point.left >= 0.0 && point.top >= 0.0)
            .then_some(Position::cell(point.left as u16, point.top as u16));
        let mut arrow = node(style, cells).with_key("popover-arrow");
        arrow.metadata.inert = !visible;
        let owner = self.clone();
        let config = props.clone();
        arrow
            .metadata
            .capture_events
            .push(Arc::new(move |event| owner.observe(event, &config, false)));
        let mut label = crate::accessibility::Node::new(crate::accessibility::Role::Label);
        label.set_hidden();
        arrow.metadata.accessibility = Some(label);
        arrow
    }
}

fn screen_rect(layout: LayoutInfo) -> Rect<f32> {
    global_rect(
        layout,
        Rect {
            left: 0.0,
            top: 0.0,
            right: layout.size.0,
            bottom: layout.size.1,
        },
    )
}
