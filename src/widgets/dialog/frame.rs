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
        modal.offset = presentation.offset;
        modal.on_placed = presentation.placed.clone();
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

/// The look the dialog theme gives a button variant, or, when the theme
/// has no entry for it, the fill of the role the variant names (`warning`,
/// `success`, `info`), the primary look for `primary`, the danger look for
/// `danger` and the secondary look for the rest (OVL-001).
pub(super) fn button_look(theme: &super::DialogTheme, variant: &str) -> String {
    use crate::widgets::display::modal::{DANGER_BUTTON, PRIMARY_BUTTON, SECONDARY_BUTTON};
    if let Some(look) = theme.button_styles.get(variant) {
        return look.clone();
    }
    match variant {
        "primary" => PRIMARY_BUTTON.to_string(),
        "danger" => DANGER_BUTTON.to_string(),
        "warning" | "success" | "info" => format!(
            "px-1 bg-{variant} text-{variant}-foreground cursor-pointer focus:bg-selection focus:text-selection-foreground"
        ),
        _ => SECONDARY_BUTTON.to_string(),
    }
}

/// `button` in the look `button_look` gives `variant`.
pub(super) fn styled(
    mut button: crate::widgets::display::modal::ModalButton,
    theme: &super::DialogTheme,
    variant: &str,
) -> crate::widgets::display::modal::ModalButton {
    button.style = Some(button_look(theme, variant));
    button
}

/// The width of a dialog with a field when the options do not size it: 40
/// cells, and at most half the viewport the dialog is laid out in (OVL-002).
pub(super) fn field_dialog_width(layout: Option<LayoutInfo>) -> ModalSize {
    let viewport = layout.map_or(80.0, |layout| layout.clip.width);
    ModalSize::Fixed(((viewport / 2.0).floor() as u16).clamp(1, 40))
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
