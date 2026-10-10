use super::{MenuItem, MenuItemType, MenuSeparator, MenuStyle};
use crate::{
    accessibility::{Node, Role},
    builder::ElementBuilder,
    component::{Element, ElementType, LayoutInfo, LayoutType},
    layout::style::{Direction, StyleBuilder},
    widgets::display::pieces::{separator_glyph, SeparatorStyle},
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
    /// The path of the rows each panel depth was last measured for, so a
    /// panel for another row at the same depth is not placed by the old
    /// panel's size.
    pub opened: Mutex<HashMap<usize, Vec<usize>>>,
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
        // The line's glyph comes from the separator piece's table (DIS-006);
        // a menu bar's separator runs down, a panel's across.
        let style = match kind {
            MenuSeparator::None | MenuSeparator::Space => None,
            MenuSeparator::Line => Some(SeparatorStyle::Line),
            MenuSeparator::ThickLine => Some(SeparatorStyle::Thick),
            MenuSeparator::DoubleLine => Some(SeparatorStyle::Double),
            MenuSeparator::Dashed => Some(SeparatorStyle::Dashed),
            MenuSeparator::Dotted => Some(SeparatorStyle::Dotted),
        };
        let glyph = style.map_or(' ', |style| separator_glyph(style, options.horizontal));
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
            vec![Element::text(glyph.to_string().repeat(width)).with_class("whitespace-pre")],
        )
        .with_class(&options.style.separator_classes)
        .with_key(format!("separator:{path:?}"))
        .with_accessibility(Node::new(Role::Splitter))
    }
    pub fn row<'a>(&self, options: &RowOptions<'a>, item: &MenuItem, path: Vec<usize>) -> Element {
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
        // The current row and a disabled row are one color from end to
        // end: an icon or a shortcut in a color of its own could not be
        // read on every selection color.
        let plain = selected || !item.enabled || !options.enabled;
        let part = |classes: &'a str| if plain { "" } else { classes };
        let mut children = vec![];
        if style.show_icons {
            if let Some(icon) = &item.icon {
                children
                    .push(Element::text(format!("{icon} ")).with_class(part(&style.icon_classes)));
            }
        }
        children.push(
            Element::text(format!("{prefix}{}", item.text)).with_class("whitespace-pre shrink-0"),
        );
        // The hint is the explicit shortcut's text or, for an item bound to
        // an action, the active keymap's binding of it (KEY-002).
        let hint = item.hint();
        if style.show_shortcuts && options.shortcuts {
            if let Some(hint) = &hint {
                children.push(
                    Element::text(format!("  {hint}")).with_class(part(&style.shortcut_classes)),
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
        if let Some(hint) = &hint {
            // The screen reader hears the same text the hint shows (KEY-002).
            semantic.set_keyboard_shortcut(hint.clone());
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use accesskit::Toggled;

    /// What the screen reader is told of the row that `item` makes at
    /// `path`, in a menu whose current path is `selection`.
    fn spoken(item: &MenuItem, path: &[usize], selection: &[usize]) -> accesskit::Node {
        let style = MenuStyle::default();
        let row = MenuView::default().row(
            &RowOptions {
                horizontal: false,
                style: &style,
                shortcuts: true,
                enabled: true,
                focused: true,
                selection,
            },
            item,
            path.to_vec(),
        );
        *row.metadata
            .accessibility
            .expect("a row has an accessibility node")
            .inner
    }

    /// KEY-002: a row bound to an action tells the screen reader the same
    /// text its hint shows, the active keymap's binding, before and after a
    /// rebind; an explicit shortcut keeps its own text.
    #[test]
    fn key_002_a_row_bound_to_an_action_tells_the_screen_reader_the_bindings_text() {
        use crate::event::types::KeyCode;
        use crate::keymap::{Action, KeyBinding, Keymap};
        use crate::widgets::menu::MenuShortcut;
        let copy = MenuItem::new("copy", "Copy").bound_to(Action::Copy);
        assert_eq!(
            spoken(&copy, &[0], &[0]).keyboard_shortcut(),
            Some("Ctrl+C"),
            "the default binding"
        );
        let mut rebound = Keymap::default();
        rebound.rebind(Action::Copy, [KeyBinding::new(KeyCode::F(5))]);
        let _scope = Keymap::scoped(rebound);
        assert_eq!(spoken(&copy, &[0], &[0]).keyboard_shortcut(), Some("F5"));
        let explicit = MenuItem::new("save", "Save")
            .shortcut(MenuShortcut::new("Ctrl+S", vec!["ctrl", "s"]))
            .bound_to(Action::Copy);
        assert_eq!(
            spoken(&explicit, &[0], &[0]).keyboard_shortcut(),
            Some("Ctrl+S"),
            "an explicit shortcut keeps its text"
        );
        assert_eq!(
            spoken(&MenuItem::new("plain", "Plain"), &[0], &[0]).keyboard_shortcut(),
            None
        );
    }

    #[test]
    fn bar_003_a_menu_row_tells_the_screen_reader_its_kind_and_its_state() {
        let action = spoken(&MenuItem::new("new", "New file"), &[0], &[0]);
        assert_eq!(
            (action.role(), action.label(), action.is_disabled()),
            (Role::MenuItem, Some("New file"), false)
        );
        assert!(
            action.supports_action(accesskit::Action::Click),
            "BAR-003: a row can be activated by the screen reader"
        );

        let disabled = spoken(&MenuItem::new("off", "Off").enabled(false), &[1], &[0]);
        assert!(disabled.is_disabled(), "BAR-003: a disabled row says so");

        let described = spoken(
            &MenuItem::new("save", "Save").description("Writes the file"),
            &[0],
            &[0],
        );
        assert_eq!(described.description(), Some("Writes the file"));

        for (checked, expected) in [(true, Toggled::True), (false, Toggled::False)] {
            let checkbox = spoken(
                &MenuItem::checkbox("wrap", "Wrap", checked, |_| {}),
                &[0],
                &[0],
            );
            assert_eq!(
                (checkbox.role(), checkbox.toggled()),
                (Role::MenuItemCheckBox, Some(expected)),
                "BAR-003: a checkbox row checked {checked}"
            );
        }

        let radio = spoken(
            &MenuItem::radio("size", "Large", true, "sizes", || {}),
            &[0],
            &[0],
        );
        assert_eq!(
            (radio.role(), radio.toggled()),
            (Role::MenuItemRadio, Some(Toggled::True))
        );

        let submenu = MenuItem::submenu("more", "More", vec![MenuItem::new("one", "One")]);
        assert_eq!(
            (
                spoken(&submenu, &[0], &[0]).is_expanded(),
                spoken(&submenu, &[0], &[0, 0]).is_expanded()
            ),
            (Some(false), Some(true)),
            "BAR-003: a row with a submenu says whether it is open"
        );
    }

    #[test]
    fn bar_003_a_separator_is_a_splitter_and_takes_no_action() {
        let separator = spoken(&MenuItem::separator(), &[0], &[1]);
        assert_eq!(separator.role(), Role::Splitter);
        assert!(!separator.supports_action(accesskit::Action::Click));
    }
}
