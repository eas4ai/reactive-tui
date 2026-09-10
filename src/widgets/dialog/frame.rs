use super::{DialogAnchor, DialogPosition};
use crate::{
    component::LayoutInfo,
    core::geometry::{Rect, Size},
    widgets::display::modal::{ModalPosition, ModalProps, ModalSize},
};

fn cell(value: usize) -> u16 {
    value.min(u16::MAX as usize) as u16
}

pub(super) fn apply_bounds(modal: &mut ModalProps, bounds: Rect) {
    if !bounds.size.is_empty() {
        modal.width = ModalSize::Fixed(cell(bounds.size.width));
        modal.height = ModalSize::Fixed(cell(bounds.size.height));
        modal.position = ModalPosition::Custom {
            x: cell(bounds.origin.x),
            y: cell(bounds.origin.y),
        };
    }
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
