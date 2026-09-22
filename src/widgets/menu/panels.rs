use super::{
    model::{list_at, MenuModel},
    view::{node, MenuView, RowOptions},
};
use crate::{
    accessibility::{Node, Role},
    component::{Element, LayoutInfo},
    layout::style::{Direction, StyleBuilder},
    widgets::display::overlay::{global_rect, local_rect},
};
use std::sync::Arc;
use taffy::geometry::Rect;

pub(super) struct PanelOptions<'a> {
    pub rows: RowOptions<'a>,
    pub maximum: usize,
    pub width: Option<u16>,
    pub focus: Option<crate::component::FocusProps>,
    pub height: Option<u16>,
    pub leading: Vec<Element>,
    pub trailing: Vec<Element>,
    pub border: bool,
    pub shadow: bool,
}

pub(super) fn bounds(root: LayoutInfo) -> Rect<f32> {
    local_rect(
        root,
        Rect {
            left: root.clip.x,
            top: root.clip.y,
            right: root.clip.x + root.clip.width,
            bottom: root.clip.y + root.clip.height,
        },
    )
}

impl MenuView {
    pub fn anchor(&self, root: LayoutInfo, path: &[usize]) -> Option<Rect<f32>> {
        let layout = *self.targets.lock().unwrap().get(path)?;
        Some(local_rect(
            root,
            global_rect(
                layout,
                Rect {
                    left: 0.0,
                    top: 0.0,
                    right: layout.size.0,
                    bottom: layout.size.1,
                },
            ),
        ))
    }

    pub fn panel(
        &self,
        menu: &MenuModel,
        parent: &[usize],
        root: LayoutInfo,
        origin: (f32, f32),
        options: PanelOptions<'_>,
    ) -> Vec<Element> {
        let border = options.border
            && !options
                .rows
                .style
                .border_classes
                .split_whitespace()
                .any(|token| token == "border-0");
        let depth = parent.len();
        let bounds = bounds(root);
        let measured = self.panels.lock().unwrap().get(&depth).copied();
        let (width, height) = measured.map_or((0.0, 0.0), |layout| layout.size);
        let x = origin.0.min(bounds.right - width).max(bounds.left);
        let y = origin.1.min(bounds.bottom - height).max(bounds.top);
        let insets = measured.map_or(0.0, |layout| layout.insets[1] + layout.insets[3]);
        let extra_height: f32 = self
            .chrome
            .lock()
            .unwrap()
            .iter()
            .filter(|((panel_depth, after), _)| {
                *panel_depth == depth
                    && if *after {
                        !options.trailing.is_empty()
                    } else {
                        !options.leading.is_empty()
                    }
            })
            .map(|(_, layout)| layout.size.1)
            .sum();
        let available_height = options
            .height
            .map_or(bounds.bottom - bounds.top, f32::from)
            .min(bounds.bottom - bounds.top);
        let maximum = options
            .maximum
            .max(1)
            .min((available_height - insets - extra_height).max(1.0) as usize);
        let eligible: Vec<_> = list_at(&menu.items, parent)
            .iter()
            .enumerate()
            .filter(|(_, item)| item.visible)
            .collect();
        let selected = eligible
            .iter()
            .position(|(index, _)| Some(index) == menu.path.get(depth))
            .unwrap_or(0);
        let row_heights = self.row_heights.lock().unwrap();
        let heights: Vec<f32> = eligible
            .iter()
            .map(|(index, item)| {
                let mut path = parent.to_vec();
                path.push(*index);
                row_heights
                    .get(&path)
                    .copied()
                    .unwrap_or_else(|| {
                        let separator = item.separator != super::MenuSeparator::None;
                        if item.item_type == super::MenuItemType::Separator {
                            1.0
                        } else {
                            item.text.lines().count().max(1) as f32 + u8::from(separator) as f32
                        }
                    })
                    .max(1.0)
            })
            .collect();
        drop(row_heights);
        let row_budget = (available_height - insets - extra_height).max(1.0);
        let mut offsets = self.scroll_offsets.lock().unwrap();
        let mut start = offsets.get(&depth).copied().unwrap_or(0).min(selected);
        let mut used: f32 = heights.iter().take(selected + 1).skip(start).sum();
        while start < selected && (selected - start + 1 > maximum || used > row_budget) {
            used -= heights[start];
            start += 1;
        }
        offsets.insert(depth, start);
        offsets.retain(|panel_depth, _| *panel_depth <= menu.path.len());
        drop(offsets);
        let mut end = (selected + 1).min(eligible.len());
        while end < eligible.len() && end - start < maximum && used + heights[end] <= row_budget {
            used += heights[end];
            end += 1;
        }
        let rows: Vec<_> = eligible
            .into_iter()
            .skip(start)
            .take(end - start)
            .map(|(index, item)| {
                let mut path = parent.to_vec();
                path.push(index);
                let mut row = self.row(&options.rows, item, path.clone());
                let heights = self.row_heights.clone();
                row.metadata.layout.push(Arc::new(move |layout| {
                    heights.lock().unwrap().insert(path.clone(), layout.size.1)
                        != Some(layout.size.1)
                }));
                row
            })
            .collect();
        let max_width = options
            .rows
            .style
            .max_width
            .map_or(bounds.right - bounds.left, f32::from)
            .min(bounds.right - bounds.left)
            .max(0.0);
        let mut style = StyleBuilder::new()
            .display_flex()
            .direction(Direction::Column)
            .position_absolute()
            .inset_left(x)
            .inset_top(y)
            .z_index(1000 + depth as i32 * 2)
            .min_width_px(f32::from(options.rows.style.min_width).min(max_width))
            .max_width_px(max_width)
            .max_height_px((bounds.bottom - bounds.top).max(0.0))
            .cell_border(f32::from(u8::from(border)))
            .padding_all_px(f32::from(
                options.rows.style.padding.saturating_sub(u16::from(border)),
            ))
            .opacity(if measured.is_some() { 1.0 } else { 0.0 })
            .overflow_hidden();
        if let Some(width) = options.width {
            style = style.width_px(f32::from(width).min(max_width));
        }
        if let Some(height) = options.height {
            style = style.height_px(f32::from(height).min(available_height));
        }
        let mut contents = Vec::new();
        for (after, parts) in [(false, options.leading), (true, options.trailing)] {
            if !after {
                contents.extend(rows.iter().cloned());
            }
            if parts.is_empty() {
                continue;
            }
            let mut chrome = node(
                StyleBuilder::new()
                    .display_flex()
                    .direction(Direction::Column)
                    .flex_shrink(0.0),
                parts,
            )
            .with_key(if after { "menu-footer" } else { "menu-header" });
            let measures = self.chrome.clone();
            chrome.metadata.layout.push(Arc::new(move |layout| {
                measures.lock().unwrap().insert((depth, after), layout) != Some(layout)
            }));
            if after {
                contents.push(chrome);
            } else {
                contents.insert(0, chrome);
            }
        }
        if border {
            let classes = options
                .rows
                .style
                .border_classes
                .split_whitespace()
                .filter(|token| !matches!(*token, "border" | "border-0" | "border-2" | "border-4"))
                .map(|token| {
                    token
                        .strip_prefix("border-")
                        .map_or_else(|| token.to_owned(), |color| format!("text-{color}"))
                })
                .collect::<Vec<_>>()
                .join(" ");
            let outline = crate::widgets::display::table::border::elements(
                &crate::widgets::display::Border::default(),
                width as usize,
                height as usize,
            )
            .into_iter()
            .map(|element| element.with_class(&classes))
            .collect();
            contents.push(node(
                StyleBuilder::new()
                    .position_absolute()
                    .inset_left(-1.0)
                    .inset_top(-1.0)
                    .width_px(width)
                    .height_px(height),
                outline,
            ));
        }
        let mut panel = node(style, contents)
            .with_key(format!("menu-panel:{depth}"))
            .with_class(&options.rows.style.base_classes)
            .with_accessibility(Node::new(Role::Menu));
        if let Some(focus) = options.focus {
            panel = panel.with_focus(focus);
        }
        let panels = self.panels.clone();
        panel.metadata.layout.push(Arc::new(move |layout| {
            panels.lock().unwrap().insert(depth, layout) != Some(layout)
        }));
        let mut layers = Vec::new();
        if options.shadow && options.rows.style.show_shadow && measured.is_some() {
            layers.push(
                node(
                    StyleBuilder::new()
                        .position_absolute()
                        .inset_left(x + 1.0)
                        .inset_top(y + 1.0)
                        .width_px(width)
                        .height_px(height)
                        .bg_rgba(0.0, 0.0, 0.0, 0.4)
                        .z_index(999 + depth as i32 * 2),
                    vec![],
                )
                .with_key(format!("menu-shadow:{depth}")),
            );
        }
        layers.push(panel);
        layers
    }
}
