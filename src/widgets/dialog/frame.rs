pub(super) use super::engine::Activity;
use super::{DialogAnchor, DialogPosition};
use crate::{
    component::LayoutInfo,
    core::geometry::{Rect, Size},
    widgets::display::modal::{ModalPosition, ModalProps, ModalSize},
};

pub(super) fn activity() -> Activity {
    crate::reactive::component_scope::lookup::<super::engine::Presentation>()
        .map(|presentation| presentation.activity)
        .unwrap_or_default()
}

fn cell(value: usize) -> u16 {
    value.min(u16::MAX as usize) as u16
}

pub(super) fn apply_bounds(modal: &mut ModalProps, bounds: Rect) {
    if let Some(presentation) =
        crate::reactive::component_scope::lookup::<super::engine::Presentation>()
    {
        modal.z_index = presentation.z_index;
        modal.focus_trap &= presentation.focus_trap;
    }
    if !bounds.size.is_empty() {
        modal.width = ModalSize::Fixed(cell(bounds.size.width));
        modal.height = ModalSize::Fixed(cell(bounds.size.height));
        modal.position = ModalPosition::Custom {
            x: cell(bounds.origin.x),
            y: cell(bounds.origin.y),
        };
    }
}

pub(super) fn escape_allowed(allowed: bool) -> bool {
    allowed
        && crate::reactive::component_scope::lookup::<super::engine::Presentation>()
            .is_none_or(|presentation| presentation.escape_to_close)
}

pub(super) fn modal(
    props: ModalProps,
    escape_closable: bool,
    role: crate::accessibility::Role,
) -> crate::component::Element {
    modal_with_presented_callback(props, escape_closable, role, None)
}

pub(super) fn modal_with_presented_callback(
    mut props: ModalProps,
    escape_closable: bool,
    role: crate::accessibility::Role,
    on_presented: Option<std::sync::Arc<dyn Fn() + Send + Sync>>,
) -> crate::component::Element {
    use crate::widgets::display::modal::{Modal, ModalAnimation};
    let motion = crate::reactive::component_scope::lookup::<super::engine::Presentation>().map(
        |presentation| {
            props.visible &= presentation.activity.active();
            if !presentation.backdrop_blur {
                props.backdrop_style = None;
            }
            props.animation = if presentation.animated {
                ModalAnimation::Fade
            } else {
                ModalAnimation::None
            };
            presentation.motion
        },
    );
    Modal::with_presentation(
        props,
        role,
        escape_allowed(escape_closable),
        motion,
        on_presented,
    )
}

pub(super) fn position(
    position: &DialogPosition,
    layout: Option<LayoutInfo>,
) -> Result<ModalPosition, String> {
    Ok(match position {
        DialogPosition::Center => ModalPosition::Center,
        DialogPosition::TopCenter => ModalPosition::Top,
        DialogPosition::BottomCenter => ModalPosition::Bottom,
        DialogPosition::LeftCenter => ModalPosition::Left,
        DialogPosition::RightCenter => ModalPosition::Right,
        DialogPosition::Fixed(point) => ModalPosition::Custom {
            x: cell(point.x),
            y: cell(point.y),
        },
        DialogPosition::Custom(position) => {
            let viewport = layout.map_or(Size::zero(), |layout| {
                Size::new(layout.clip.width as usize, layout.clip.height as usize)
            });
            let point = position(viewport);
            ModalPosition::Custom {
                x: cell(point.x),
                y: cell(point.y),
            }
        }
        DialogPosition::RelativeTo {
            element_id,
            offset,
            anchor,
        } => {
            let anchors = crate::reactive::component_scope::current()
                .and_then(|scope| {
                    scope.lookup::<std::sync::Arc<crate::component::anchors::Anchors>>()
                })
                .ok_or_else(|| format!("Dialog anchor requires an App: {element_id}"))?;
            let bounds = anchors.lookup(element_id)?;
            let (x, y) = match anchor {
                DialogAnchor::TopLeft => (0.0, 0.0),
                DialogAnchor::TopCenter => (0.5, 0.0),
                DialogAnchor::TopRight => (1.0, 0.0),
                DialogAnchor::CenterLeft => (0.0, 0.5),
                DialogAnchor::Center => (0.5, 0.5),
                DialogAnchor::CenterRight => (1.0, 0.5),
                DialogAnchor::BottomLeft => (0.0, 1.0),
                DialogAnchor::BottomCenter => (0.5, 1.0),
                DialogAnchor::BottomRight => (1.0, 1.0),
            };
            ModalPosition::Custom {
                x: cell(
                    ((bounds.x + x * bounds.width).round().max(0.0) as usize)
                        .saturating_add(offset.x),
                ),
                y: cell(
                    ((bounds.y + y * bounds.height).round().max(0.0) as usize)
                        .saturating_add(offset.y),
                ),
            }
        }
    })
}
