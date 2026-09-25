use super::{MenuItem, MenuItemType, MenuSeparator, MenuStyle};
use crate::{
    accessibility::{Node, Role},
    builder::ElementBuilder,
    component::{Element, ElementType, LayoutInfo, LayoutType},
    layout::style::{Direction, StyleBuilder},
};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

pub(super) struct RowOptions<'a> {
    pub horizontal: bool,
    pub style: &'a MenuStyle,
    pub shortcuts: bool,
    pub enabled: bool,
    pub focused: bool,
    pub selection: &'a [usize],
}

#[derive(Default)]
pub(super) struct MenuView {
    pub scroll_offsets: Mutex<HashMap<usize, usize>>,
    pub targets: Arc<Mutex<HashMap<Vec<usize>, LayoutInfo>>>,
    visible_targets: Mutex<HashSet<Vec<usize>>>,
    pub panels: Arc<Mutex<HashMap<usize, LayoutInfo>>>,
    pub row_heights: Arc<Mutex<HashMap<Vec<usize>, f32>>>,
    pub chrome: Arc<Mutex<HashMap<(usize, bool), LayoutInfo>>>,
}

pub(super) fn node(style: StyleBuilder, children: Vec<Element>) -> Element {
    ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
        .styles(style)
        .children(children)
        .build()
}

impl MenuView {
    pub fn clear(&self) {
        self.visible_targets.lock().unwrap().clear();
    }
    pub fn page_size(&self, selection: &[usize]) -> isize {
        let depth = selection.len().saturating_sub(1);
        let parent = &selection[..depth];
        self.visible_targets
            .lock()
            .unwrap()
            .iter()
            .filter(|path| path.len() == depth + 1 && path.starts_with(parent))
            .count()
            .max(1)
            .min(isize::MAX as usize) as isize
    }
    pub fn contains_panel(&self, x: f32, y: f32, max_depth: usize) -> bool {
        self.panels.lock().unwrap().iter().any(|(depth, layout)| {
            *depth <= max_depth
                && x >= layout.clip.x
                && y >= layout.clip.y
                && x < layout.clip.x + layout.clip.width
                && y < layout.clip.y + layout.clip.height
                && layout.local_cell(x, y).is_some()
        })
    }
    pub fn hit(&self, x: f32, y: f32) -> Option<Vec<usize>> {
        let visible = self.visible_targets.lock().unwrap();
        self.targets
            .lock()
            .unwrap()
            .iter()
            .filter(|(path, layout)| {
                visible.contains(*path)
                    && x >= layout.clip.x
                    && y >= layout.clip.y
                    && x < layout.clip.x + layout.clip.width
                    && y < layout.clip.y + layout.clip.height
                    && layout.local_cell(x, y).is_some()
            })
            .max_by_key(|(path, _)| path.len())
            .map(|(path, _)| path.clone())
    }
    fn separator(&self, options: &RowOptions<'_>, kind: &MenuSeparator, path: &[usize]) -> Element {
        let glyph = match (kind, options.horizontal) {
            (MenuSeparator::None | MenuSeparator::Space, _) => " ",
            (MenuSeparator::Line, false) => "─",
            (MenuSeparator::Line, true) => "│",
            (MenuSeparator::ThickLine, false) => "━",
            (MenuSeparator::ThickLine, true) => "┃",
            (MenuSeparator::DoubleLine, false) => "═",
            (MenuSeparator::DoubleLine, true) => "║",
            (MenuSeparator::Dashed, false) => "╌",
            (MenuSeparator::Dashed, true) => "╎",
            (MenuSeparator::Dotted, _) => "·",
        };
        let depth = path.len().saturating_sub(1);
        let width = if options.horizontal {
            1
        } else {
            self.panels
                .lock()
                .unwrap()
                .get(&depth)
                .map_or(usize::from(options.style.min_width), |panel| {
                    panel.content_size().0.max(0.0) as usize
                })
        };
        node(
            StyleBuilder::new()
                .height_px(1.0)
                .flex_shrink(0.0)
                .overflow_hidden(),
            vec![Element::text(glyph.repeat(width)).with_class("whitespace-pre")],
        )
        .with_class(&options.style.separator_classes)
        .with_key(format!("separator:{path:?}"))
        .with_accessibility(Node::new(Role::Splitter))
    }
    pub fn row(&self, options: &RowOptions<'_>, item: &MenuItem, path: Vec<usize>) -> Element {
        if item.item_type == MenuItemType::Separator {
            return self.separator(options, &item.separator, &path);
        }
        let separator = (item.separator != MenuSeparator::None)
            .then(|| self.separator(options, &item.separator, &path));
        self.visible_targets.lock().unwrap().insert(path.clone());
        let selected = options.selection.starts_with(&path);
        let style = options.style;
        let prefix = match item.item_type {
            MenuItemType::Checkbox { checked } => {
                if checked {
                    "[x] "
                } else {
                    "[ ] "
                }
            }
            MenuItemType::Radio { selected, .. } => {
                if selected {
                    "(●) "
                } else {
                    "( ) "
                }
            }
            _ => "",
        };
        let mut children = vec![];
        if style.show_icons {
            if let Some(icon) = &item.icon {
                children.push(Element::text(format!("{icon} ")).with_class(&style.icon_classes));
            }
        }
        children.push(
            Element::text(format!("{prefix}{}", item.text)).with_class("whitespace-pre shrink-0"),
        );
        if style.show_shortcuts && options.shortcuts {
            if let Some(shortcut) = &item.shortcut {
                children.push(
                    Element::text(format!("  {}", shortcut.display))
                        .with_class(&style.shortcut_classes),
                );
            }
        }
        if item.has_submenu() && !options.horizontal {
            children.push(Element::text(" ▶"));
        }
        let mut semantic = Node::new(match item.item_type {
            MenuItemType::Checkbox { .. } => Role::MenuItemCheckBox,
            MenuItemType::Radio { .. } => Role::MenuItemRadio,
            MenuItemType::Separator => Role::Splitter,
            _ => Role::MenuItem,
        });
        semantic.set_clickable();
        semantic.set_label(item.text.clone());
        if !item.enabled || !options.enabled {
            semantic.set_disabled();
        }
        if let Some(description) = &item.description {
            semantic.set_description(description.clone());
        }
        if let MenuItemType::Checkbox { checked } = item.item_type {
            semantic.inner.set_toggled(if checked {
                accesskit::Toggled::True
            } else {
                accesskit::Toggled::False
            });
        }
        if let MenuItemType::Radio { selected, .. } = item.item_type {
            semantic.inner.set_toggled(if selected {
                accesskit::Toggled::True
            } else {
                accesskit::Toggled::False
            });
        }
        if item.has_submenu() {
            semantic
                .inner
                .set_expanded(options.selection.len() > path.len() && selected);
        }
        let mut result = node(
            StyleBuilder::new()
                .display_flex()
                .direction(Direction::Row)
                .padding_x_px(1.0)
                .flex_shrink(0.0),
            children,
        )
        .with_class(format!(
            "whitespace-pre {}",
            if !item.enabled || !options.enabled {
                &style.disabled_classes
            } else if selected && options.focused {
                &style.focused_classes
            } else if selected {
                &style.selected_classes
            } else {
                ""
            }
        ))
        .with_key(format!("item:{}", item.id))
        .with_accessibility(semantic);
        let accessible = result
            .metadata
            .accessibility_options
            .get_or_insert_default();
        accessible.focus = options.selection == path;
        accessible.focus_event = Some(crate::event::CustomEvent::new(
            "reactive_tui.menu.focus",
            serde_json::to_vec(&path).expect("menu indices serialize"),
        ));
        let targets = self.targets.clone();
        result.metadata.layout.push(Arc::new(move |layout| {
            targets.lock().unwrap().insert(path.clone(), layout) != Some(layout)
        }));
        if let Some(separator) = separator {
            node(
                StyleBuilder::new()
                    .display_flex()
                    .direction(if options.horizontal {
                        Direction::Row
                    } else {
                        Direction::Column
                    })
                    .flex_shrink(0.0),
                vec![result, separator],
            )
            .with_key(format!("item:{}", item.id))
        } else {
            result
        }
    }
}
