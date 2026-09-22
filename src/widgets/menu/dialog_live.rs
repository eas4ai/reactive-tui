use super::runtime::{MenuRuntime, WorldEvents};
pub(super) type LiveDialog = WorldEvents<DialogRuntime>;
use super::{
    model::{item_mut, list_at, shortcut_path, MenuModel},
    panels::{bounds, PanelOptions},
    view::{node, MenuView, RowOptions},
    DialogMenuProps, DialogMenuState, DialogMenuType, MenuItem, MenuItemType, TextCallback,
};
use crate::{
    accessibility::{Node, Role},
    component::{Component, Element, FocusProps, LayoutInfo, Props},
    event::{
        router::EventResult,
        types::{Event, FocusEventKind, KeyCode, KeyEventKind, MouseButton, MouseEventKind},
    },
    layout::style::StyleBuilder,
    reactive::ThreadSafeSignal,
};
use std::{
    any::Any,
    sync::{Arc, Mutex},
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

type VoidCallback = Arc<dyn Fn() + Send + Sync>;
#[derive(Clone)]
pub(super) struct LiveProps {
    pub config: DialogMenuProps,
    pub seed: DialogMenuState,
    pub selected: Option<TextCallback>,
    pub confirmed: Option<Arc<dyn Fn(Vec<String>) + Send + Sync>>,
    pub cancelled: Option<VoidCallback>,
    pub submitted: Option<TextCallback>,
    pub shown: Option<VoidCallback>,
    pub hidden: Option<VoidCallback>,
}
impl PartialEq for LiveProps {
    fn eq(&self, other: &Self) -> bool {
        self.config == other.config
            && self.seed == other.seed
            && super::menubar_live::same(&self.selected, &other.selected)
            && super::menubar_live::same(&self.confirmed, &other.confirmed)
            && super::menubar_live::same(&self.cancelled, &other.cancelled)
            && super::menubar_live::same(&self.submitted, &other.submitted)
            && super::menubar_live::same(&self.shown, &other.shown)
            && super::menubar_live::same(&self.hidden, &other.hidden)
    }
}
impl Props for LiveProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn contains_selectable(items: &[MenuItem], id: &str) -> bool {
    items.iter().any(|item| {
        item.is_selectable()
            && ((!item.has_submenu() && item.id == id) || contains_selectable(&item.submenu, id))
    })
}

fn paint_selections(items: &mut [MenuItem], selected: &[String]) {
    for item in items {
        if item.has_submenu() {
            paint_selections(&mut item.submenu, selected);
        } else if item.is_selectable() {
            item.item_type = MenuItemType::Checkbox {
                checked: selected.contains(&item.id),
            };
        }
    }
}

fn close(visible: &ThreadSafeSignal<bool>, hidden: &Option<VoidCallback>) {
    if visible.get() {
        visible.set(false);
        if let Some(callback) = hidden {
            callback();
        }
    }
}
fn cancel(visible: &ThreadSafeSignal<bool>, props: &LiveProps) {
    if !visible.get() {
        return;
    }
    if let Some(item) = props
        .config
        .cancel_button
        .and_then(|index| props.config.items.get(index))
    {
        item.execute();
    }
    if let Some(callback) = &props.cancelled {
        callback();
    }
    close(visible, &props.hidden);
}

pub(super) struct DialogRuntime {
    menu: MenuModel,
    view: MenuView,
    input: Arc<Mutex<DialogMenuState>>,
    selected: Arc<Mutex<Vec<String>>>,
    seed: DialogMenuState,
    visible: ThreadSafeSignal<bool>,
    authored_visible: bool,
    focused: ThreadSafeSignal<bool>,
    root: Option<LayoutInfo>,
    input_layout: Arc<Mutex<Option<LayoutInfo>>>,
    hidden: Option<VoidCallback>,
}
impl DialogRuntime {
    fn input_focus(&self) -> FocusProps {
        let gained = self.focused.clone();
        let lost = self.focused.clone();
        FocusProps {
            auto_focus: true,
            on_focus: Some(Arc::new(move || gained.set(true))),
            on_blur: Some(Arc::new(move || lost.set(false))),
            ..FocusProps::input()
        }
    }

    fn activate(&mut self, props: &LiveProps) {
        if let Some(item) = item_mut(&mut self.menu.items, &self.menu.path) {
            if !item.is_selectable() {
                return;
            }
            if item.has_submenu() {
                if let Some(index) = item.submenu.iter().position(MenuItem::is_selectable) {
                    self.menu.path.push(index);
                }
                return;
            }
            if props.config.dialog_type == DialogMenuType::MultiSelection {
                let mut selected = self.selected.lock().unwrap();
                if let Some(index) = selected.iter().position(|id| *id == item.id) {
                    selected.remove(index);
                } else {
                    selected.push(item.id.clone());
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
        close(&self.visible, &props.hidden);
    }
    fn confirm(&self, props: &LiveProps) {
        if let Some(callback) = &props.confirmed {
            let values = self.selected.lock().unwrap().clone();
            callback(values);
        }
        close(&self.visible, &props.hidden);
    }
    fn submit(&self, props: &LiveProps) {
        let value = self.input.lock().unwrap().input_text.clone();
        if let Some(callback) = &props.submitted {
            callback(&value);
        }
        close(&self.visible, &props.hidden);
    }
    fn input_window(&self, state: &DialogMenuState) -> std::ops::Range<usize> {
        let width = self
            .input_layout
            .lock()
            .unwrap()
            .map_or(1.0, |layout| layout.content_size().0)
            .max(1.0) as usize;
        let cursor = state.input_boundary();
        let caret_width = state.input_text[cursor..]
            .graphemes(true)
            .next()
            .map_or(1, UnicodeWidthStr::width)
            .max(1);
        let mut cells = caret_width;
        let mut start = cursor;
        for (index, grapheme) in state.input_text[..cursor].grapheme_indices(true).rev() {
            if cells + grapheme.width() > width {
                break;
            }
            cells += grapheme.width();
            start = index;
        }
        let mut end = cursor;
        let mut cells = state.input_text[start..cursor].width();
        for (index, grapheme) in state.input_text[cursor..].grapheme_indices(true) {
            if cells + grapheme.width() > width {
                break;
            }
            cells += grapheme.width();
            end = cursor + index + grapheme.len();
        }
        start..end
    }
    fn chrome(&self, props: &LiveProps) -> (Vec<Element>, Vec<Element>) {
        let mut leading = Vec::new();
        if let Some(title) = &props.config.title {
            leading.push(Element::text(title).with_class("font-bold"));
        }
        if let Some(message) = &props.config.message {
            leading.push(Element::text(message));
        }
        if props.config.dialog_type == DialogMenuType::Input {
            let state = self.input.lock().unwrap();
            let cursor = state.input_boundary();
            let window = self.input_window(&state);
            let before = &state.input_text[window.start..cursor];
            let after = &state.input_text[cursor..window.end];
            let caret = after.graphemes(true).next().unwrap_or(" ");
            let mut semantics = Node::new(Role::TextInput);
            semantics.set_label(props.config.title.as_deref().unwrap_or("Input"));
            semantics.inner.set_value(state.input_text.clone());
            let mut input = node(
                StyleBuilder::new()
                    .display_flex()
                    .direction(crate::layout::style::Direction::Row)
                    .width_percent(100.0)
                    .min_width_px(0.0)
                    .height_px(1.0)
                    .overflow_hidden(),
                vec![
                    Element::text(before).with_class("shrink-0"),
                    Element::text(caret).with_class(if self.focused.get() {
                        "reverse shrink-0"
                    } else {
                        "shrink-0"
                    }),
                    Element::text(after.get(caret.len()..).unwrap_or("")).with_class("shrink-0"),
                ],
            )
            .with_class("whitespace-pre")
            .with_accessibility(semantics)
            .with_key("dialog-input");
            let layout = self.input_layout.clone();
            input.metadata.layout.push(Arc::new(move |next| {
                let mut current = layout.lock().unwrap();
                let changed = *current != Some(next);
                *current = Some(next);
                changed
            }));
            leading.push(input);
        }
        let mut trailing = Vec::new();
        if matches!(
            props.config.dialog_type,
            DialogMenuType::Input | DialogMenuType::MultiSelection
        ) {
            let visible = self.visible.clone();
            let hidden = props.hidden.clone();
            let input = self.input.clone();
            let submitted = props.submitted.clone();
            let selected = self.selected.clone();
            let confirmed = props.confirmed.clone();
            let is_input = props.config.dialog_type == DialogMenuType::Input;
            trailing.push(
                crate::builder::button()
                    .text(if is_input { "Submit" } else { "Confirm" })
                    .disabled(!props.config.enabled)
                    .class("p-0")
                    .on_click(move || {
                        if !visible.get() {
                            return;
                        }
                        if is_input {
                            let text = input.lock().unwrap().input_text.clone();
                            if let Some(callback) = &submitted {
                                callback(&text);
                            }
                        } else {
                            let values = selected.lock().unwrap().clone();
                            if let Some(callback) = &confirmed {
                                callback(values);
                            }
                        }
                        close(&visible, &hidden);
                    })
                    .build(),
            );
        }
        if props.config.show_close_button {
            let visible = self.visible.clone();
            let config = props.clone();
            trailing.push(
                crate::builder::button()
                    .text("Close")
                    .disabled(!props.config.enabled)
                    .class("p-0")
                    .on_click(move || cancel(&visible, &config))
                    .build(),
            );
        }
        (leading, trailing)
    }
}
impl Component for DialogRuntime {
    type Props = LiveProps;
    type State = ();
    fn new(props: Self::Props) -> Self {
        let view = MenuView::default();
        view.scroll_offsets
            .lock()
            .unwrap()
            .insert(0, props.seed.scroll_offset);
        let selected = props
            .seed
            .selected_items
            .iter()
            .filter_map(|index| props.config.items.get(*index))
            .filter(|item| item.is_selectable() && !item.has_submenu())
            .map(|item| item.id.clone())
            .collect();
        let initial = props.seed.selected_index.or(props.config.default_button);
        if props.config.visible {
            if let Some(callback) = &props.shown {
                callback();
            }
        }
        Self {
            menu: MenuModel::new(props.config.items, initial.into_iter().collect()),
            view,
            input: Arc::new(Mutex::new(props.seed.clone())),
            selected: Arc::new(Mutex::new(selected)),
            seed: props.seed,
            visible: ThreadSafeSignal::new(props.config.visible),
            authored_visible: props.config.visible,
            focused: ThreadSafeSignal::new(false),
            root: None,
            input_layout: Arc::new(Mutex::new(None)),
            hidden: props.hidden,
        }
    }
    fn update(&mut self, props: &Self::Props, _: &mut ()) -> bool {
        self.menu.update(&props.config.items);
        self.hidden = props.hidden.clone();
        self.selected
            .lock()
            .unwrap()
            .retain(|id| contains_selectable(&props.config.items, id));
        if self.seed != props.seed {
            self.view
                .scroll_offsets
                .lock()
                .unwrap()
                .insert(0, props.seed.scroll_offset);
            self.seed = props.seed.clone();
            *self.input.lock().unwrap() = props.seed.clone();
            self.menu.path = props
                .seed
                .selected_index
                .or(props.config.default_button)
                .into_iter()
                .collect();
            self.menu.repair_selection();
            *self.selected.lock().unwrap() = props
                .seed
                .selected_items
                .iter()
                .filter_map(|index| props.config.items.get(*index))
                .filter(|item| item.is_selectable() && !item.has_submenu())
                .map(|item| item.id.clone())
                .collect();
        }
        if self.authored_visible != props.config.visible {
            self.authored_visible = props.config.visible;
            if props.config.visible {
                self.visible.set(true);
                if let Some(callback) = &props.shown {
                    callback();
                }
            } else {
                close(&self.visible, &props.hidden);
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
        if !self.visible.get() {
            return Element::empty();
        }
        let mut children = Vec::new();
        if let Some(root) = self.root {
            let bounds = bounds(root);
            if props.config.modal || props.config.close_on_outside_click {
                let mut style = StyleBuilder::new()
                    .position_absolute()
                    .inset_left(bounds.left)
                    .inset_top(bounds.top)
                    .width_px(bounds.right - bounds.left)
                    .height_px(bounds.bottom - bounds.top)
                    .z_index(998);
                if props.config.modal {
                    style = style.bg_rgba(0.0, 0.0, 0.0, 0.3);
                }
                children.push(node(style, vec![]).with_key("dialog-shield"));
            }
            let selected = self.selected.lock().unwrap().clone();
            let mut display = self.menu.items.clone();
            if props.config.dialog_type == DialogMenuType::MultiSelection {
                paint_selections(&mut display, &selected);
            }
            let display = MenuModel::new(display, self.menu.path.clone());
            for depth in 0..self.menu.path.len().max(1) {
                let parent = &self.menu.path[..depth];
                let origin = if depth == 0 {
                    if props.config.centered {
                        let size = self
                            .view
                            .panels
                            .lock()
                            .unwrap()
                            .get(&0)
                            .map_or((0.0, 0.0), |layout| layout.size);
                        (
                            bounds.left + (bounds.right - bounds.left - size.0) / 2.0,
                            bounds.top + (bounds.bottom - bounds.top - size.1) / 2.0,
                        )
                    } else {
                        let (x, y) = props.config.position.unwrap_or((0, 0));
                        (f32::from(x), f32::from(y))
                    }
                } else if let Some(anchor) = self.view.anchor(root, parent) {
                    (anchor.right, anchor.top)
                } else {
                    continue;
                };
                let (leading, trailing) = if depth == 0 {
                    self.chrome(props)
                } else {
                    (Vec::new(), Vec::new())
                };
                children.extend(self.view.panel(
                    &display,
                    parent,
                    root,
                    origin,
                    PanelOptions {
                        rows: RowOptions {
                            horizontal: false,
                            style: &props.config.style,
                            shortcuts: true,
                            enabled: props.config.enabled,
                            focused: self.focused.get(),
                            selection: &self.menu.path,
                        },
                        maximum: usize::MAX,
                        width: props.config.width,
                        focus: None,
                        height: props.config.height,
                        leading,
                        trailing,
                        border: props.config.show_border,
                        shadow: props.config.show_shadow,
                    },
                ));
            }
        }
        let mut semantics = Node::new(Role::Dialog);
        if let Some(title) = &props.config.title {
            semantics.set_label(title.clone());
        }
        let mut content = node(StyleBuilder::new(), children).with_key("dialog-menu-content");
        if props.config.enabled {
            content = content.with_focus(self.input_focus());
        }
        let mut root = node(StyleBuilder::new(), vec![content]).with_accessibility(semantics);
        if props.config.enabled {
            root = root.with_focus(FocusProps {
                auto_focus: true,
                restore_focus: true,
                trap_focus: props.config.modal,
                ..FocusProps::input()
            });
        }
        root.metadata.disabled = !props.config.enabled;
        root
    }

    fn on_lifecycle(&mut self, event: crate::component::LifecycleEvent, _: &mut ()) {
        if event == crate::component::LifecycleEvent::Unmount {
            close(&self.visible, &self.hidden);
        }
    }
}

impl MenuRuntime for DialogRuntime {
    fn event(&mut self, event: &Event, props: &Self::Props) -> EventResult {
        if !self.visible.get() || !props.config.enabled {
            return EventResult::Ignored;
        }
        match event {
            Event::Custom(event) if self.menu.focus_request(event) => {}
            Event::Focus(event) => match event.kind {
                FocusEventKind::Gained => self.focused.set(true),
                FocusEventKind::Lost => self.focused.set(false),
                _ => return EventResult::Ignored,
            },
            Event::Paste(paste) if props.config.dialog_type == DialogMenuType::Input => {
                let text: String = paste
                    .content
                    .chars()
                    .filter(|ch| !ch.is_control())
                    .collect();
                self.input.lock().unwrap().insert_text(&text);
            }
            Event::Key(key) if key.kind != KeyEventKind::Release => {
                if key.code == KeyCode::Escape {
                    if self.menu.path.len() > 1 {
                        self.menu.path.pop();
                    } else if props.config.close_on_escape {
                        cancel(&self.visible, props);
                    }
                    return EventResult::Consumed;
                }
                if props.config.dialog_type == DialogMenuType::Input {
                    let mut state = self.input.lock().unwrap();
                    match key.code {
                        KeyCode::Char(ch)
                            if !key.modifiers.ctrl && !key.modifiers.alt && !key.modifiers.meta =>
                        {
                            state.insert_char(ch)
                        }
                        KeyCode::Space => state.insert_char(' '),
                        KeyCode::Backspace => state.delete_char(),
                        KeyCode::Delete => {
                            let cursor = state.input_boundary();
                            state.move_cursor_right();
                            if state.input_cursor != cursor {
                                state.delete_char();
                            }
                        }
                        KeyCode::Left => state.move_cursor_left(),
                        KeyCode::Right => state.move_cursor_right(),
                        KeyCode::Home => state.input_cursor = 0,
                        KeyCode::End => state.input_cursor = state.input_text.len(),
                        KeyCode::Enter => {
                            drop(state);
                            self.submit(props);
                        }
                        _ => return EventResult::Ignored,
                    }
                    return EventResult::Consumed;
                }
                if let Some(path) = shortcut_path(&self.menu.items, key) {
                    self.menu.path = path;
                    self.activate(props);
                    return EventResult::Consumed;
                }
                match key.code {
                    KeyCode::PageUp => self
                        .menu
                        .move_selection(-self.view.page_size(&self.menu.path)),
                    KeyCode::PageDown => self
                        .menu
                        .move_selection(self.view.page_size(&self.menu.path)),
                    KeyCode::Up => self.menu.move_selection(-1),
                    KeyCode::Down => self.menu.move_selection(1),
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
                    KeyCode::Left if self.menu.path.len() > 1 => {
                        self.menu.path.pop();
                    }
                    KeyCode::Right
                        if item_mut(&mut self.menu.items, &self.menu.path)
                            .is_some_and(|item| item.has_submenu()) =>
                    {
                        self.activate(props)
                    }
                    KeyCode::Enter | KeyCode::Space | KeyCode::Char(' ') => self.activate(props),
                    KeyCode::Tab if props.config.dialog_type == DialogMenuType::MultiSelection => {
                        self.confirm(props)
                    }
                    _ => return EventResult::Ignored,
                }
            }
            Event::Mouse(mouse) if mouse.kind == MouseEventKind::Move => {
                let Some(path) = self
                    .view
                    .hit(mouse.position.x() as f32, mouse.position.y() as f32)
                else {
                    return EventResult::Ignored;
                };
                if !self.menu.hover(path) {
                    return EventResult::Ignored;
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
                if matches!(mouse.kind, MouseEventKind::Down | MouseEventKind::Click)
                    && mouse.button == MouseButton::Left =>
            {
                let (x, y) = (mouse.position.x() as f32, mouse.position.y() as f32);
                if let Some(path) = self.view.hit(x, y) {
                    self.menu.path = path;
                    self.activate(props);
                } else if props.config.dialog_type == DialogMenuType::Input
                    && self
                        .input_layout
                        .lock()
                        .unwrap()
                        .is_some_and(|layout| layout.local_cell(x, y).is_some())
                {
                    let layout = self.input_layout.lock().unwrap().unwrap();
                    let column = layout.local_cell(x, y).unwrap().0 as usize;
                    let mut state = self.input.lock().unwrap();
                    let start = self.input_window(&state).start;
                    let mut cells = 0;
                    let mut byte = state.input_text.len();
                    for (index, text) in state.input_text[start..].grapheme_indices(true) {
                        if cells + text.width() > column {
                            byte = start + index;
                            break;
                        }
                        cells += text.width();
                    }
                    state.input_cursor = byte;
                } else if !props.config.modal
                    && props.config.close_on_outside_click
                    && !self
                        .view
                        .contains_panel(x, y, self.menu.path.len().saturating_sub(1))
                {
                    cancel(&self.visible, props);
                }
            }
            _ => return EventResult::Ignored,
        }
        EventResult::Consumed
    }
}
