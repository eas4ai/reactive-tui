use super::runtime::{MenuRuntime, WorldEvents};
pub(super) type LiveMenuBar = WorldEvents<MenuBarRuntime>;
use super::model::{item_mut, list_at, shortcut_path, MenuModel};
use super::panels::{bounds, PanelOptions};
use super::view::{node, MenuView, RowOptions};
use super::{MenuBarProps, MenuBarState, MenuItem, TextCallback};
use crate::{
    accessibility::{Node, Role},
    component::{Component, Element, FocusProps, LayoutInfo, Props},
    event::{
        router::EventResult,
        types::{Event, FocusEventKind, KeyCode, KeyEventKind, MouseButton, MouseEventKind},
    },
    layout::style::{Direction, StyleBuilder},
};
use std::{any::Any, sync::Arc};

#[derive(Clone)]
pub(super) struct LiveProps {
    pub config: MenuBarProps,
    pub seed: MenuBarState,
    pub selected: Option<TextCallback>,
    pub opened: Option<Arc<dyn Fn(usize) + Send + Sync>>,
    pub closed: Option<Arc<dyn Fn() + Send + Sync>>,
}
pub(super) fn same<T: ?Sized>(a: &Option<Arc<T>>, b: &Option<Arc<T>>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}
impl PartialEq for LiveProps {
    fn eq(&self, other: &Self) -> bool {
        self.config == other.config
            && self.seed == other.seed
            && same(&self.selected, &other.selected)
            && same(&self.opened, &other.opened)
            && same(&self.closed, &other.closed)
    }
}
impl Props for LiveProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Default)]
pub(super) struct MenuBarRuntime {
    menu: MenuModel,
    seed: MenuBarState,
    focused: bool,
    root: Option<LayoutInfo>,
    view: MenuView,
}

fn seed_path(seed: &MenuBarState) -> Vec<usize> {
    let mut path: Vec<_> = seed.selected_index.into_iter().collect();
    if seed.dropdown_open && !path.is_empty() {
        path.push(seed.submenu_selected_index.unwrap_or(0));
    }
    path
}

impl MenuBarRuntime {
    fn close(&mut self, props: &LiveProps) {
        let open = self.menu.path.len() > 1;
        self.menu.path.truncate(1);
        if open {
            if let Some(callback) = &props.closed {
                callback();
            }
        }
    }
    fn activate(&mut self, props: &LiveProps) {
        let Some(item) = item_mut(&mut self.menu.items, &self.menu.path) else {
            return;
        };
        if !item.is_selectable() {
            return;
        }
        if item.has_submenu() {
            if let Some(index) = item.submenu.iter().position(MenuItem::is_selectable) {
                self.menu.path.push(index);
                if self.menu.path.len() == 2 {
                    if let Some(callback) = &props.opened {
                        callback(self.menu.path[0]);
                    }
                }
            }
            return;
        }
        let Some(invoked) = self.menu.take_action() else {
            return;
        };
        self.close(props);
        invoked.execute();
        if let Some(callback) = &props.selected {
            callback(&invoked.id);
        }
    }
}

impl Component for MenuBarRuntime {
    type Props = LiveProps;
    type State = ();
    fn new(props: Self::Props) -> Self {
        let view = MenuView::default();
        view.scroll_offsets
            .lock()
            .unwrap()
            .insert(1, props.seed.dropdown_scroll_offset);
        Self {
            menu: MenuModel::new(props.config.items, seed_path(&props.seed)),
            seed: props.seed,
            view,
            ..Default::default()
        }
    }
    fn update(&mut self, props: &Self::Props, _: &mut ()) -> bool {
        let was_open = self.menu.path.len() > 1;
        self.menu.update(&props.config.items);
        if self.seed != props.seed {
            self.view
                .scroll_offsets
                .lock()
                .unwrap()
                .insert(1, props.seed.dropdown_scroll_offset);
            self.menu.path = seed_path(&props.seed);
            self.menu.repair_selection();
            self.seed = props.seed.clone();
        }
        if was_open && self.menu.path.len() <= 1 {
            if let Some(callback) = &props.closed {
                callback();
            }
        }
        if !props.config.visible || !props.config.enabled {
            self.close(props);
        }
        true
    }
    fn layout(&mut self, layout: LayoutInfo, _: &mut Self::Props, _: &mut ()) -> bool {
        let changed = self.root != Some(layout);
        self.root = Some(layout);
        changed
    }
    fn render(&self, props: &Self::Props, _: &()) -> Element {
        self.view.clear();
        if !props.config.visible {
            return Element::empty();
        }
        let mut bar = vec![];
        if let Some(title) = &props.config.title {
            bar.push(Element::text(title).with_class("px-1 font-bold"));
        }
        bar.extend(
            self.menu
                .items
                .iter()
                .enumerate()
                .filter(|(_, item)| item.visible)
                .map(|(index, item)| {
                    self.view.row(
                        &RowOptions {
                            horizontal: true,
                            style: &props.config.style,
                            shortcuts: props.config.show_shortcuts,
                            enabled: props.config.enabled,
                            focused: self.focused,
                            selection: &self.menu.path,
                        },
                        item,
                        vec![index],
                    )
                }),
        );
        let mut children = vec![node(
            StyleBuilder::new().display_flex().direction(Direction::Row),
            bar,
        )];
        if let Some(root) = self.root {
            if self.menu.path.len() > 1 {
                let area = bounds(root);
                children.push(
                    node(
                        StyleBuilder::new()
                            .position_absolute()
                            .inset_left(area.left)
                            .inset_top(area.top)
                            .width_px(area.right - area.left)
                            .height_px(area.bottom - area.top)
                            .z_index(998),
                        vec![],
                    )
                    .with_key("menu-shield"),
                );
            }
            for depth in 1..self.menu.path.len() {
                let parent = &self.menu.path[..depth];
                let Some(anchor) = self.view.anchor(root, parent) else {
                    continue;
                };
                let origin = if depth == 1 {
                    (anchor.left, anchor.bottom)
                } else {
                    (anchor.right, anchor.top)
                };
                children.extend(self.view.panel(
                    &self.menu,
                    parent,
                    root,
                    origin,
                    PanelOptions {
                        rows: RowOptions {
                            horizontal: false,
                            style: &props.config.style,
                            shortcuts: props.config.show_shortcuts,
                            enabled: props.config.enabled,
                            focused: self.focused,
                            selection: &self.menu.path,
                        },
                        maximum: props.config.max_dropdown_height,
                        width: None,
                        focus: None,
                        height: None,
                        leading: Vec::new(),
                        trailing: Vec::new(),
                        border: true,
                        shadow: true,
                    },
                ));
            }
        }
        let mut root = node(
            StyleBuilder::new()
                .display_flex()
                .padding_all_px(f32::from(props.config.style.padding))
                .direction(Direction::Column),
            children,
        )
        .with_class(&props.config.style.base_classes)
        .with_accessibility(Node::new(Role::MenuBar));
        if props.config.enabled {
            root = root.with_focus(FocusProps::input());
        }
        root.metadata.disabled = !props.config.enabled;
        root
    }
}

impl MenuRuntime for MenuBarRuntime {
    fn event(&mut self, event: &Event, props: &Self::Props) -> EventResult {
        if !props.config.visible || !props.config.enabled {
            return EventResult::Ignored;
        }
        match event {
            Event::Custom(event) if self.menu.focus_request(event) => {}
            Event::Focus(focus) => match focus.kind {
                FocusEventKind::Gained => self.focused = true,
                FocusEventKind::Lost => {
                    self.focused = false;
                    self.close(props);
                }
                _ => return EventResult::Ignored,
            },
            Event::Key(key) if self.focused && key.kind != KeyEventKind::Release => {
                if let Some(path) = shortcut_path(&self.menu.items, key) {
                    // Close only an actually open dropdown, before selecting the shortcut target.
                    self.close(props);
                    self.menu.path = path;
                    if self.menu.path.len() > 1 {
                        let Some(item) = item_mut(&mut self.menu.items, &self.menu.path) else {
                            return EventResult::Ignored;
                        };
                        if !item.has_submenu() {
                            // Activation closes its selected path; it must not emit a second close.
                            let mut invocation = props.clone();
                            invocation.closed = None;
                            self.activate(&invocation);
                            return EventResult::Consumed;
                        }
                    }
                    self.activate(props);
                    return EventResult::Consumed;
                }
                match key.code {
                    KeyCode::Right if self.menu.path.len() == 1 => self.menu.move_selection(1),
                    KeyCode::Left if self.menu.path.len() == 1 => self.menu.move_selection(-1),
                    KeyCode::Left | KeyCode::Escape if self.menu.path.len() > 1 => {
                        if self.menu.path.len() == 2 {
                            self.close(props);
                        } else {
                            self.menu.path.pop();
                        }
                    }
                    KeyCode::Down
                        if self.menu.path.len() == 1
                            && item_mut(&mut self.menu.items, &self.menu.path)
                                .is_some_and(|item| item.has_submenu()) =>
                    {
                        self.activate(props)
                    }
                    KeyCode::Down if self.menu.path.len() == 1 => return EventResult::Ignored,
                    KeyCode::Down => self.menu.move_selection(1),
                    KeyCode::Up => self.menu.move_selection(-1),
                    KeyCode::PageUp => self
                        .menu
                        .move_selection(-self.view.page_size(&self.menu.path)),
                    KeyCode::PageDown => self
                        .menu
                        .move_selection(self.view.page_size(&self.menu.path)),
                    KeyCode::Right
                        if item_mut(&mut self.menu.items, &self.menu.path)
                            .is_some_and(|item| item.has_submenu()) =>
                    {
                        self.activate(props)
                    }
                    KeyCode::Enter | KeyCode::Space | KeyCode::Char(' ') => self.activate(props),
                    KeyCode::Home => {
                        if let Some(last) = self.menu.path.last_mut() {
                            *last = usize::MAX;
                        }
                        self.menu.repair_selection();
                    }
                    KeyCode::End => {
                        let depth = self.menu.path.len().saturating_sub(1);
                        if let Some(index) = list_at(&self.menu.items, &self.menu.path[..depth])
                            .iter()
                            .rposition(MenuItem::is_selectable)
                        {
                            self.menu.path.truncate(depth);
                            self.menu.path.push(index);
                        }
                    }
                    _ => return EventResult::Ignored,
                }
            }
            Event::Mouse(mouse) if mouse.kind == MouseEventKind::Wheel => {
                let Some(target) = self
                    .view
                    .hit(mouse.position.x() as f32, mouse.position.y() as f32)
                else {
                    return EventResult::Ignored;
                };
                if !mouse
                    .wheel
                    .as_ref()
                    .is_some_and(|wheel| self.menu.wheel(&target, wheel))
                {
                    return EventResult::Ignored;
                }
            }
            Event::Mouse(mouse)
                if matches!(
                    mouse.kind,
                    MouseEventKind::Down | MouseEventKind::Click | MouseEventKind::Move
                ) =>
            {
                let (x, y) = (mouse.position.x() as f32, mouse.position.y() as f32);
                let target = self.view.hit(x, y);
                let Some(path) = target else {
                    if self.menu.path.len() > 1
                        && mouse.button == MouseButton::Left
                        && matches!(mouse.kind, MouseEventKind::Down | MouseEventKind::Click)
                        && !self.view.contains_panel(x, y, self.menu.path.len() - 1)
                    {
                        self.close(props);
                        return EventResult::Consumed;
                    }
                    return EventResult::Ignored;
                };
                if !list_at(&self.menu.items, &path[..path.len() - 1])
                    .get(*path.last().unwrap())
                    .is_some_and(MenuItem::is_selectable)
                {
                    return EventResult::Ignored;
                }
                let was_open = self.menu.path.len() > 1;
                let submenu =
                    item_mut(&mut self.menu.items, &path).is_some_and(|item| item.has_submenu());
                if mouse.kind == MouseEventKind::Move {
                    if self.menu.path.starts_with(&path)
                        && (self.menu.path.len() > path.len() || !submenu)
                    {
                        return EventResult::Ignored;
                    }
                    if was_open && path.len() == 1 && self.menu.path.first() != path.first() {
                        self.close(props);
                    }
                    self.menu.path = path;
                    if was_open && submenu {
                        self.activate(props);
                    }
                } else if mouse.button == MouseButton::Left {
                    if was_open && path.len() == 1 {
                        let same = self.menu.path.first() == path.first();
                        self.close(props);
                        if same {
                            return EventResult::Consumed;
                        }
                    }
                    self.menu.path = path;
                    self.activate(props);
                } else {
                    return EventResult::Ignored;
                }
            }
            _ => return EventResult::Ignored,
        }
        EventResult::Consumed
    }
}
