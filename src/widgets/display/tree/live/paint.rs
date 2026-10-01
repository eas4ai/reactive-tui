use super::*;
use crate::{
    accessibility::{Node, Role, Toggled},
    builder::ElementBuilder,
    component::{FocusProps, LayoutType},
    layout::style::{Direction, StyleBuilder},
    widgets::display::{look, table::border},
};
use unicode_width::UnicodeWidthStr;

impl LiveTree {
    fn prefix(&self, row: &Row, props: &TreeProps) -> Element {
        let indent = row
            .node
            .level
            .saturating_mul(props.indent_size as usize)
            .min(u16::MAX as usize);
        let width = indent + usize::from(props.show_lines && row.node.level > 0) * 3;
        let class = props.line_style.as_deref().unwrap_or(look::MUTED);
        let mut children = Vec::new();
        if props.show_lines && row.node.level > 0 {
            let mut mark = |text: &str, x: usize, width: usize| {
                children.push(
                    ElementBuilder::new(ElementType::Text(text.to_string()))
                        .styles(
                            StyleBuilder::new()
                                .position_absolute()
                                .inset_left(x as f32)
                                .inset_top(0.0)
                                .width_px(width as f32)
                                .height_px(1.0),
                        )
                        .class(class)
                        .build(),
                );
            };
            for (level, continues) in row.branches.iter().enumerate() {
                if *continues && props.indent_size > 0 {
                    mark(
                        "│",
                        level
                            .saturating_mul(props.indent_size as usize)
                            .min(u16::MAX as usize),
                        1,
                    );
                }
            }
            mark(if row.last { "└─" } else { "├─" }, indent, 3);
        }
        ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(
                StyleBuilder::new()
                    .width_px(width as f32)
                    .height_px(1.0)
                    .flex_shrink(0.0),
            )
            .children(children)
            .build()
    }
    fn icon(row: &Row, props: &TreeProps) -> String {
        if !props.show_icons {
            return String::new();
        }
        format!(
            "{} ",
            row.node
                .icon
                .as_deref()
                .unwrap_or(if row.node.has_children {
                    "📁"
                } else {
                    "📄"
                })
        )
    }
    pub(super) fn row_width(row: &Row, props: &TreeProps) -> usize {
        row.node
            .level
            .saturating_mul(props.indent_size as usize)
            .min(u16::MAX as usize)
            .saturating_add(usize::from(props.show_lines && row.node.level > 0) * 3)
            .saturating_add(2 + usize::from(row.node.checked.is_some()) * 2)
            .saturating_add(UnicodeWidthStr::width(Self::icon(row, props).as_str()))
            .saturating_add(UnicodeWidthStr::width(row.node.label.as_str()))
            .saturating_add(usize::from(row.loading) * 2)
    }
    fn text(
        &self,
        text: String,
        width: usize,
        class: &str,
        target: Option<(&str, Part)>,
    ) -> Element {
        let mut element = ElementBuilder::new(ElementType::Text(text))
            .styles(
                StyleBuilder::new()
                    .width_px(width as f32)
                    .height_px(1.0)
                    .flex_shrink(0.0)
                    .overflow_hidden(),
            )
            .class(&format!("whitespace-pre truncate {class}"))
            .build();
        if let Some((id, part)) = target {
            let id = id.to_string();
            let targets = self.targets.clone();
            element.metadata.layout.push(Arc::new(move |layout| {
                targets.lock().unwrap().push((id.clone(), part, layout));
                false
            }));
        }
        element
    }
    fn paint_row(&self, row: &Row, props: &TreeProps, state: &TreeState, y: usize) -> Element {
        let node = &row.node;
        let enabled = row.selectable || row.checkable || row.expandable;
        let selected = state.selected_nodes.contains(&node.id);
        // The cursor's row shows `selection` while the tree holds the focus,
        // a selected node `accent` unless the application styled it, the
        // node under the pointer `hover` (DAT-001).
        let mut class = look::row(
            state.focused && self.cursor.as_deref() == Some(node.id.as_str()),
            selected && props.selected_style.is_none(),
            state.hover_node.as_deref() == Some(node.id.as_str()),
            !enabled,
        )
        .to_string();
        if let Some(style) = &props.node_style {
            class.push_str(&format!(" {style}"));
        }
        if node.expanded {
            if let Some(style) = &props.expanded_style {
                class.push_str(&format!(" {style}"));
            }
        }
        if !node.has_children {
            if let Some(style) = &props.leaf_style {
                class.push_str(&format!(" {style}"));
            }
        }
        if let Some(style) = &node.style {
            class.push_str(&format!(" {style}"));
        }
        if node.matched {
            class.push_str(" underline");
        }
        if selected {
            if let Some(style) = &props.selected_style {
                class.push_str(&format!(" {style}"));
            }
        }
        if state.drop_target.as_ref() == Some(&node.id) {
            class.push_str(" underline font-bold");
        }
        let prefix_width = node
            .level
            .saturating_mul(props.indent_size as usize)
            .min(u16::MAX as usize)
            + usize::from(props.show_lines && node.level > 0) * 3;
        let mut cells = vec![self.prefix(row, props)];
        cells.push(
            self.text(
                if row.expandable {
                    if node.expanded {
                        "▼ "
                    } else {
                        "▶ "
                    }
                } else {
                    "  "
                }
                .to_string(),
                2,
                look::MUTED,
                Some((&node.id, Part::Expand)),
            ),
        );
        if let Some(checked) = node.checked {
            let mut semantic = Node::new(Role::CheckBox);
            semantic.set_label(format!("{} checkbox", node.label));
            semantic.set_toggled(if checked {
                Toggled::True
            } else {
                Toggled::False
            });
            if row.checkable {
                semantic.set_clickable();
            } else {
                semantic.set_disabled();
            }
            let mut checkbox = self
                .text(
                    if checked { "☑ " } else { "☐ " }.to_string(),
                    2,
                    if checked { look::MARK } else { look::MUTED },
                    Some((&node.id, Part::Check)),
                )
                .with_accessibility(semantic)
                .with_key("checkbox");
            if row.checkable {
                checkbox
                    .metadata
                    .accessibility_options
                    .get_or_insert_default()
                    .click_event = Some(crate::event::CustomEvent::new(
                    "reactive_tui.tree.check",
                    node.id.as_bytes().to_vec(),
                ));
            }
            cells.push(checkbox);
        }
        let icon = Self::icon(row, props);
        let icon_width = UnicodeWidthStr::width(icon.as_str());
        let label = format!(
            "{icon}{}{}",
            node.label,
            if row.loading { " ⟳" } else { "" }
        );
        let width = Self::row_width(row, props).max(state.scroll_state.viewport_width as usize);
        let label_width = width
            .saturating_sub(prefix_width + 2 + usize::from(node.checked.is_some()) * 2)
            .max(icon_width);
        cells.push(self.text(label, label_width, "", Some((&node.id, Part::Label))));
        let mut accessible = Node::new(Role::TreeItem);
        accessible.set_label(&node.label);
        accessible.set_selected(selected);
        accessible.inner.set_level(node.level.saturating_add(1));
        accessible.inner.set_position_in_set(row.position);
        accessible.inner.set_size_of_set(row.count);
        if row.expandable {
            accessible.set_expanded(node.expanded);
        }
        if !enabled {
            accessible.set_disabled();
        } else {
            accessible.set_clickable();
        }
        let mut element = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(
                StyleBuilder::new()
                    .display_flex()
                    .direction(Direction::Row)
                    .position_absolute()
                    .inset_top(y as f32 - self.scroll as f32)
                    .inset_left(-(state.scroll_state.offset_x as f32))
                    .width_px(width as f32)
                    .height_px(1.0),
            )
            .class(&class)
            .children(cells)
            .build()
            .with_key(format!("node:{}", node.id))
            .with_accessibility(accessible);
        if enabled {
            let options = element
                .metadata
                .accessibility_options
                .get_or_insert_default();
            options.focus = state.focused && self.cursor.as_deref() == Some(&node.id);
            options.focus_event = Some(crate::event::CustomEvent::new(
                "reactive_tui.tree.focus",
                node.id.as_bytes().to_vec(),
            ));
            options.click_event = Some(crate::event::CustomEvent::new(
                "reactive_tui.tree.activate",
                node.id.as_bytes().to_vec(),
            ));
        }
        element
    }
    pub(super) fn paint(&self, props: &TreeProps, state: &TreeState) -> Element {
        self.targets.lock().unwrap().clear();
        if let Some(error) = &self.error {
            return Element::text(error).with_class(format!(
                "w-full whitespace-normal break-words {}",
                look::ERROR
            ));
        }
        let (width, height) = self
            .viewport
            .map_or((0.0, 0.0), |layout| layout.content_size());
        let inset = self.viewport.map_or([0.0; 4], |layout| layout.insets);
        let mut rows = Vec::new();
        for (index, row) in self.rows.iter().enumerate() {
            if props.virtual_scrolling
                && (index < self.scroll
                    || index >= self.scroll.saturating_add(height.max(0.0) as usize))
            {
                continue;
            }
            rows.push(self.paint_row(row, props, state, index));
        }
        if self.rows.is_empty() {
            rows.push(
                Element::text(
                    if props
                        .search_term
                        .as_ref()
                        .is_some_and(|term| !term.is_empty())
                    {
                        "No matching nodes found"
                    } else {
                        "No data"
                    },
                )
                .with_class("whitespace-normal break-words w-full"),
            );
        }
        let content = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(
                StyleBuilder::new()
                    .position_absolute()
                    .inset_left(inset[0])
                    .inset_top(inset[1])
                    .width_px(width)
                    .height_px(height)
                    .overflow_hidden(),
            )
            .children(rows)
            .build()
            .with_key("viewport");
        // Give an auto-sized parent the tree's natural size: the rows are
        // absolute, so this child carries their width and their count.
        let bordered = border::enabled(&props.border);
        let natural = self
            .rows
            .len()
            .max(1)
            .saturating_add(usize::from(bordered) * 2)
            .min(u16::MAX as usize);
        let intrinsic_width = self
            .rows
            .iter()
            .map(|row| Self::row_width(row, props))
            .max()
            .unwrap_or(7);
        let intrinsic = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(
                StyleBuilder::new()
                    .width_px(intrinsic_width as f32)
                    .height_px(natural as f32),
            )
            .build()
            .with_key("intrinsic-size");
        let mut children = vec![content, intrinsic];
        if let Some(layout) = self.viewport {
            children.extend(border::elements(
                &props.border,
                layout.size.0.max(0.0) as usize,
                layout.size.1.max(0.0) as usize,
            ));
        }
        // The box is its parent's background with no fill of its own
        // (DAT-001). It fills the width and the height its parent allots,
        // and takes its natural size in a parent that allots none, unless
        // the props set a size (DAT-002).
        let mut style = StyleBuilder::new()
            .display_flex()
            .direction(Direction::Column)
            .padding_all_px(if bordered { 1.0 } else { 0.0 })
            .overflow_hidden();
        style = match (props.height, props.max_height) {
            (Some(height), _) => style.height_px(height as f32),
            (None, Some(max)) => style
                .height_px(natural.min(max as usize) as f32)
                .max_height_percent(100.0),
            (None, None) => style
                .height_percent(100.0)
                .min_height_px(0.0)
                .max_height_percent(100.0),
        };
        style = match props.width {
            Some(width) => style.width_px(width as f32),
            None => style.width_percent(100.0),
        };
        let mut node = Node::new(Role::Tree);
        if let Some(label) = &props.aria_label {
            node.set_label(label.clone());
        }
        ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(style)
            .class("min-w-0")
            .children(children)
            .build()
            .with_focus(FocusProps::input())
            .with_accessibility(node)
    }
}
