//! The list panel a select and a text input's suggestions open
//! (docs/spec/input-widgets.md, CTL-003): under the control's row, or
//! above it when only the space above holds it, painted whole over what
//! was on the screen, also inside a modal, a popover or a box that clips,
//! without moving the content under the control.
//!
//! The control's root box grows to hold the panel, and a negative margin
//! of the same size keeps its place in the flow, so the content around
//! it stays where it was and the pointer reaches every row of the panel
//! through the control; the panel itself is `unclipped`, so a box that
//! clips the control does not cut it, and it stacks at a popover's level.

use std::sync::{Arc, Mutex};

use crate::{
    builder::ElementBuilder,
    component::{Element, ElementType, LayoutInfo, LayoutType},
    event::hit::Bounds,
    layout::style::StyleBuilder,
    widgets::display::table::border,
    widgets::display::Border,
};

/// The stacking level of a panel: a popover's.
pub(crate) const PANEL_LAYER: i32 = 2000;

/// Where a panel opens and how many rows it shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Placement {
    /// The panel stands above the control's row.
    pub above: bool,
    /// The rows the panel shows, as far as the screen holds them.
    pub rows: usize,
}

impl Placement {
    /// The rows the panel takes on the screen: its rows and its border.
    pub fn height(&self) -> usize {
        self.rows + 2
    }
}

/// The panel's place for `wanted` rows under a control whose root is
/// `root` on a screen of `screen`. Under the row when the space below
/// holds every row; above it when only the space above does; else at
/// the side with more room, scrolling. Unmeasured, the panel opens below
/// with every row.
pub(crate) fn place(root: Option<LayoutInfo>, screen: Option<Bounds>, wanted: usize) -> Placement {
    let wanted = wanted.max(1);
    let (Some(root), Some(screen)) = (root, screen) else {
        return Placement {
            above: false,
            rows: wanted,
        };
    };
    let row_top = root.transform[5];
    let below = (screen.y + screen.height - row_top - 1.0).max(0.0) as usize;
    let above = (row_top - screen.y).max(0.0) as usize;
    let fits = |room: usize| room >= wanted + 2;
    if fits(below) {
        Placement {
            above: false,
            rows: wanted,
        }
    } else if fits(above) {
        Placement {
            above: true,
            rows: wanted,
        }
    } else if above > below {
        Placement {
            above: true,
            rows: above.saturating_sub(2).max(1),
        }
    } else {
        Placement {
            above: false,
            rows: below.saturating_sub(2).max(1),
        }
    }
}

/// The last screen a panel was laid out on: its clip, which is the screen
/// because the panel is unclipped.
#[derive(Default)]
pub(crate) struct Measured {
    screen: Mutex<Option<Bounds>>,
}

impl Measured {
    pub fn screen(&self) -> Option<Bounds> {
        *self.screen.lock().unwrap()
    }

    /// A layout callback that records the panel's clip.
    fn record(self: &Arc<Self>) -> crate::component::element::LayoutCallback {
        let measured = self.clone();
        Arc::new(move |layout: LayoutInfo| {
            let mut screen = measured.screen.lock().unwrap();
            let changed = *screen != Some(layout.clip);
            *screen = Some(layout.clip);
            changed
        })
    }
}

/// The style of a control's root while its panel is open: the row and the
/// panel in a column, a negative margin giving the panel's rows back to
/// the flow, and the root as wide as the panel with the same margin at the
/// right, so nothing around the control moves. `root` is the control's
/// last layout, whose insets (its padding and border) the box keeps.
pub(crate) fn host_style(
    placement: Placement,
    root: Option<LayoutInfo>,
    field_width: usize,
    panel_width: usize,
) -> StyleBuilder {
    let insets = root.map_or([0.0; 4], |root| root.insets);
    let panel_height = placement.height() as f32;
    let width = field_width.max(panel_width) as f32;
    let mut style = StyleBuilder::new()
        .width_px(width + insets[0] + insets[2])
        .height_px(1.0 + panel_height + insets[1] + insets[3])
        .margin_r_px(field_width as f32 - width)
        .overflow_visible()
        .z_index(PANEL_LAYER);
    style = if placement.above {
        style.margin_t_px(-panel_height)
    } else {
        style.margin_b_px(-panel_height)
    };
    style
}

/// The panel: a bordered box in `surface` of `rows` rows, `width` cells
/// wide, unclipped and at a popover's level. `measured` records the
/// screen it was laid out on.
pub(crate) fn element(
    placement: Placement,
    width: usize,
    rows: Vec<Element>,
    measured: &Arc<Measured>,
) -> Element {
    let height = placement.height();
    let mut children = border::elements(
        &Border {
            color: Some("border".into()),
            ..Border::default()
        },
        width,
        height,
    );
    children.push(
        Element::layout(LayoutType::Flex)
            .class("flex flex-col shrink-0")
            .children(rows),
    );
    let mut panel = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
        .styles(
            StyleBuilder::new()
                .width_px(width as f32)
                .height_px(height as f32)
                .flex_shrink(0.0)
                .padding_all_px(1.0)
                .overflow_hidden()
                .z_index(PANEL_LAYER)
                .unclipped(),
        )
        .class("bg-surface text-foreground")
        .children(children)
        .build();
    panel.metadata.layout.push(measured.record());
    panel
}
