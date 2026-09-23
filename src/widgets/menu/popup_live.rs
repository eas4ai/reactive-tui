use super::runtime::{MenuRuntime, WorldEvents};
pub(super) type LivePopup = WorldEvents<PopupRuntime>;
use super::{
    model::{item_mut, list_at, shortcut_path, MenuModel},
    panels::{bounds, PanelOptions},
    view::{node, MenuView, RowOptions},
    PopupMenuProps, PopupMenuState, PopupPlacement, TextCallback,
};
use crate::{
    accessibility::{Node, Role},
    component::{Component, Element, FocusProps, LayoutInfo, Props},
    event::{
        router::EventResult,
        types::{Event, FocusEventKind, KeyCode, KeyEventKind, MouseButton, MouseEventKind},
    },
    layout::style::StyleBuilder,
};
use std::{any::Any, sync::Arc};

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum RelativePlacement {
    Above,
    Below,
    Left,
    Right,
    Auto,
}

#[derive(Clone)]
pub(super) struct LiveProps {
    pub config: PopupMenuProps,
    pub relative_placement: Option<RelativePlacement>,
    pub seed: PopupMenuState,
    pub selected: Option<TextCallback>,
    pub shown: Option<Arc<dyn Fn() + Send + Sync>>,
    pub hidden: Option<Arc<dyn Fn() + Send + Sync>>,
}
impl PartialEq for LiveProps {
    fn eq(&self, other: &Self) -> bool {
        self.config == other.config
            && self.seed == other.seed
            && self.relative_placement == other.relative_placement
            && super::menubar_live::same(&self.selected, &other.selected)
            && super::menubar_live::same(&self.shown, &other.shown)
            && super::menubar_live::same(&self.hidden, &other.hidden)
    }
}
impl Props for LiveProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub(super) struct PopupRuntime {
    menu: MenuModel,
    seed: PopupMenuState,
    view: MenuView,
    root: Option<LayoutInfo>,
    visible: bool,
    authored_visible: bool,
    focused: bool,
    cursor: Option<(u16, u16)>,
    hidden: Option<Arc<dyn Fn() + Send + Sync>>,
}
impl PopupRuntime {
    fn close(&mut self, props: &LiveProps) {
        if self.visible {
            self.visible = false;
            self.focused = false;
            if let Some(callback) = &props.hidden {
                callback();
            }
        }
    }
    fn activate(&mut self, props: &LiveProps) {
        if let Some(item) = item_mut(&mut self.menu.items, &self.menu.path) {
            if item.has_submenu() {
                if let Some(index) = item.submenu.iter().position(super::MenuItem::is_selectable) {
                    self.menu.path.push(index);
                }
                return;
            }
        }
        let Some(item) = self.menu.take_action() else {
            return;
        };
        item.execute();
        if let Some(callback) = &props.selected {
            callback(&item.id);
        }
        if props.config.auto_close {
            self.close(props);
        }
    }
    fn origin(&self, root: LayoutInfo, props: &LiveProps) -> (f32, f32) {
        let panel = self.view.panels.lock().unwrap().get(&0).copied();
        let (width, height) = panel.map_or((0.0, 0.0), |layout| layout.size);
        if let Some(side) = props.relative_placement {
            return match side {
                RelativePlacement::Above => (0.0, -height),
                RelativePlacement::Left => (-width, 0.0),
                RelativePlacement::Right => (root.size.0, 0.0),
                RelativePlacement::Below | RelativePlacement::Auto => (0.0, root.size.1),
            };
        }
        let placement = &props.config.placement;
        if matches!(placement, PopupPlacement::Cursor) && self.cursor.is_none() {
            return (0.0, 0.0);
        }
        let (x, y) = match *placement {
            PopupPlacement::Cursor => self.cursor.unwrap_or((0, 0)),
            PopupPlacement::Position { x, y }
            | PopupPlacement::Widget { x, y, .. }
            | PopupPlacement::Below { x, y, .. }
            | PopupPlacement::Above { x, y, .. }
            | PopupPlacement::Left { x, y, .. }
            | PopupPlacement::Right { x, y, .. } => (x, y),
        };
        let point = crate::widgets::display::overlay::local_rect(
            root,
            taffy::geometry::Rect {
                left: f32::from(x),
                right: f32::from(x),
                top: f32::from(y),
                bottom: f32::from(y),
            },
        );
        match *placement {
            PopupPlacement::Widget { height, .. } => (point.left, point.top + f32::from(height)),
            PopupPlacement::Above { .. } => (point.left, point.top - height),
            PopupPlacement::Left { .. } => (point.left - width, point.top),
            _ => (point.left, point.top),
        }
    }
}
impl Component for PopupRuntime {
    type Props = LiveProps;
    type State = ();
    fn new(props: Self::Props) -> Self {
        let view = MenuView::default();
        view.scroll_offsets
            .lock()
            .unwrap()
            .insert(0, props.seed.scroll_offset);
        if props.config.visible {
            if let Some(callback) = &props.shown {
                callback();
            }
        }
        Self {
            menu: MenuModel::new(
                props.config.items,
                props.seed.selected_index.into_iter().collect(),
            ),
            view,
            root: None,
            visible: props.config.visible,
            authored_visible: props.config.visible,
            focused: false,
            cursor: props.seed.mouse_position,
            seed: props.seed,
            hidden: props.hidden,
        }
    }
    fn update(&mut self, props: &Self::Props, _: &mut ()) -> bool {
        self.menu.update(&props.config.items);
        if self.seed != props.seed {
            self.view
                .scroll_offsets
                .lock()
                .unwrap()
                .insert(0, props.seed.scroll_offset);
            self.menu.path = props.seed.selected_index.into_iter().collect();
            self.menu.repair_selection();
            self.cursor = props.seed.mouse_position;
            self.seed = props.seed.clone();
        }
        self.hidden = props.hidden.clone();
        if self.authored_visible != props.config.visible {
            self.authored_visible = props.config.visible;
            if props.config.visible {
                self.visible = true;
                if let Some(callback) = &props.shown {
                    callback();
                }
            } else {
                self.close(props);
            }
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
        if !self.visible {
            return Element::empty();
        }
        let mut children = Vec::new();
        if let Some(root) = self.root {
            let bounds = bounds(root);
            if props.config.close_on_outside_click {
                children.push(
                    node(
                        StyleBuilder::new()
                            .position_absolute()
                            .inset_left(bounds.left)
                            .inset_top(bounds.top)
                            .width_px(bounds.right - bounds.left)
                            .height_px(bounds.bottom - bounds.top)
                            .z_index(998),
                        vec![],
                    )
                    .with_key("menu-shield"),
                );
            }
            for depth in 0..self.menu.path.len().max(1) {
                let parent = &self.menu.path[..depth];
                let origin = if depth == 0 {
                    self.origin(root, props)
                } else if let Some(anchor) = self.view.anchor(root, parent) {
                    (anchor.right, anchor.top)
                } else {
                    continue;
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
                            shortcuts: true,
                            enabled: props.config.enabled,
                            focused: self.focused,
                            selection: &self.menu.path,
                        },
                        maximum: props.config.max_visible_items,
                        width: props.config.width,
                        focus: None,
                        height: None,
                        leading: Vec::new(),
                        trailing: Vec::new(),
                        border: props.config.show_border,
                        shadow: props.config.show_shadow,
                    },
                ));
            }
        }
        let mut root =
            node(StyleBuilder::new(), children).with_accessibility(Node::new(Role::Menu));
        if props.config.enabled {
            root = root.with_focus(FocusProps {
                auto_focus: true,
                ..FocusProps::menu()
            });
        }
        root.metadata.disabled = !props.config.enabled;
        root
    }

    fn on_lifecycle(&mut self, event: crate::component::LifecycleEvent, _: &mut ()) {
        if event == crate::component::LifecycleEvent::Unmount && self.visible {
            self.visible = false;
            if let Some(callback) = &self.hidden {
                callback();
            }
        }
    }
}

impl MenuRuntime for PopupRuntime {
    fn event(&mut self, event: &Event, props: &Self::Props) -> EventResult {
        if !self.visible || !props.config.enabled {
            return EventResult::Ignored;
        }
        match event {
            Event::Custom(event) if self.menu.focus_request(event) => {}
            Event::Focus(event) => match event.kind {
                FocusEventKind::Gained => self.focused = true,
                FocusEventKind::Lost => self.focused = false,
                _ => return EventResult::Ignored,
            },
            Event::Key(key) if self.focused && key.kind != KeyEventKind::Release => {
                if let Some(path) = shortcut_path(&self.menu.items, key) {
                    self.menu.path = path;
                    self.activate(props);
                    return EventResult::Consumed;
                }
                match key.code {
                    KeyCode::Up => self.menu.move_selection(-1),
                    KeyCode::Down => self.menu.move_selection(1),
                    KeyCode::PageUp => self
                        .menu
                        .move_selection(-self.view.page_size(&self.menu.path)),
                    KeyCode::PageDown => self
                        .menu
                        .move_selection(self.view.page_size(&self.menu.path)),
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
                            .rposition(super::MenuItem::is_selectable)
                        {
                            self.menu.path.truncate(depth);
                            self.menu.path.push(index);
                        }
                    }
                    KeyCode::Escape | KeyCode::Left if self.menu.path.len() > 1 => {
                        self.menu.path.pop();
                    }
                    KeyCode::Escape => self.close(props),
                    KeyCode::Right
                        if item_mut(&mut self.menu.items, &self.menu.path)
                            .is_some_and(|item| item.has_submenu()) =>
                    {
                        self.activate(props)
                    }
                    KeyCode::Enter | KeyCode::Space | KeyCode::Char(' ') => self.activate(props),
                    _ => return EventResult::Ignored,
                }
            }
            Event::Mouse(mouse) => {
                let (x, y) = (mouse.position.x() as f32, mouse.position.y() as f32);
                let target = self.view.hit(x, y);
                match mouse.kind {
                    MouseEventKind::Down | MouseEventKind::Click
                        if mouse.button == MouseButton::Left =>
                    {
                        if let Some(path) = target {
                            self.menu.path = path;
                            self.activate(props);
                        } else if props.config.close_on_outside_click {
                            let inside = self.view.contains_panel(
                                x,
                                y,
                                self.menu.path.len().saturating_sub(1),
                            );
                            if !inside {
                                self.close(props);
                            }
                        }
                    }
                    MouseEventKind::Move => {
                        let Some(path) = target else {
                            return EventResult::Ignored;
                        };
                        if !self.menu.hover(path) {
                            return EventResult::Ignored;
                        }
                    }
                    MouseEventKind::Wheel if target.is_some() => {
                        let Some(wheel) = &mouse.wheel else {
                            return EventResult::Ignored;
                        };
                        if !self.menu.wheel(&target.unwrap(), wheel) {
                            return EventResult::Ignored;
                        }
                    }
                    _ => return EventResult::Ignored,
                }
            }
            _ => return EventResult::Ignored,
        }
        EventResult::Consumed
    }
}
