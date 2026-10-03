use super::{MenuItem, MenuItemType};
use crate::event::types::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Default)]
pub(super) struct MenuModel {
    pub items: Vec<MenuItem>,
    previous: Vec<MenuItem>,
    pub path: Vec<usize>,
}

pub(super) fn list_at<'a>(items: &'a [MenuItem], path: &[usize]) -> &'a [MenuItem] {
    match path.split_first() {
        Some((index, rest)) => items
            .get(*index)
            .map_or(&[], |item| list_at(&item.submenu, rest)),
        None => items,
    }
}
pub(super) fn item_mut<'a>(items: &'a mut [MenuItem], path: &[usize]) -> Option<&'a mut MenuItem> {
    let (index, rest) = path.split_first()?;
    let item = items.get_mut(*index)?;
    if rest.is_empty() {
        Some(item)
    } else {
        item_mut(&mut item.submenu, rest)
    }
}
fn reconcile(next: &mut [MenuItem], previous: &[MenuItem], current: &[MenuItem]) {
    for item in next {
        let old = previous.iter().find(|old| old.id == item.id);
        let live = current.iter().find(|live| live.id == item.id);
        if let (Some(old), Some(live)) = (old, live) {
            if item.item_type == old.item_type {
                item.item_type = live.item_type.clone();
            }
            reconcile(&mut item.submenu, &old.submenu, &live.submenu);
        }
    }
}
fn shortcut_matches(keys: &[String], event: &KeyEvent) -> bool {
    let mut modifiers = KeyModifiers::empty();
    let mut key = None;
    for token in keys {
        let token = token.to_ascii_lowercase();
        match token.as_str() {
            "ctrl" | "control" => modifiers.ctrl = true,
            "alt" | "option" => modifiers.alt = true,
            "shift" => modifiers.shift = true,
            "meta" | "super" | "cmd" | "command" => modifiers.meta = true,
            _ if key.is_none() => key = Some(token),
            _ => return false,
        }
    }
    let Some(key) = key else {
        return false;
    };
    if modifiers != event.modifiers {
        return false;
    }
    match &event.code {
        KeyCode::Char(character) => key == character.to_ascii_lowercase().to_string(),
        KeyCode::F(number) => key == format!("f{number}"),
        KeyCode::Escape => matches!(key.as_str(), "esc" | "escape"),
        KeyCode::Enter => matches!(key.as_str(), "enter" | "return"),
        KeyCode::Unknown | KeyCode::Null => false,
        code => key == format!("{code:?}").to_ascii_lowercase(),
    }
}

pub(super) fn shortcut_path(items: &[MenuItem], event: &KeyEvent) -> Option<Vec<usize>> {
    for (index, item) in items
        .iter()
        .enumerate()
        .filter(|(_, item)| item.is_selectable())
    {
        if item
            .shortcut
            .as_ref()
            .is_some_and(|shortcut| shortcut_matches(&shortcut.keys, event))
        {
            return Some(vec![index]);
        }
        if let Some(mut path) = shortcut_path(&item.submenu, event) {
            path.insert(0, index);
            return Some(path);
        }
    }
    None
}

impl MenuModel {
    pub fn focus_request(&mut self, event: &crate::event::CustomEvent) -> bool {
        if event.name != "reactive_tui.menu.focus" {
            return false;
        }
        let Ok(path) = serde_json::from_slice::<Vec<usize>>(&event.data) else {
            return false;
        };
        if path.is_empty() {
            return false;
        }
        let mut items = self.items.as_slice();
        for index in &path {
            let Some(item) = items.get(*index).filter(|item| item.is_selectable()) else {
                return false;
            };
            items = &item.submenu;
        }
        self.path = path;
        true
    }
    pub fn new(items: Vec<MenuItem>, path: Vec<usize>) -> Self {
        let mut model = Self {
            previous: items.clone(),
            items,
            path,
        };
        model.repair_selection();
        model
    }
    pub fn update(&mut self, incoming: &[MenuItem]) {
        let mut items = incoming.to_vec();
        reconcile(&mut items, &self.previous, &self.items);
        let mut path = Vec::new();
        for depth in 0..self.path.len() {
            let Some(selected) = list_at(&self.items, &self.path[..depth]).get(self.path[depth])
            else {
                break;
            };
            let siblings = list_at(&items, &path);
            let Some(index) = siblings
                .iter()
                .position(|item| item.id == selected.id && item.is_selectable())
            else {
                break;
            };
            path.push(index);
        }
        self.path = path;
        self.previous = incoming.to_vec();
        self.items = items;
        self.repair_selection();
    }
    pub fn take_action(&mut self) -> Option<MenuItem> {
        let item = item_mut(&mut self.items, &self.path)?;
        if !item.is_selectable() || item.has_submenu() {
            return None;
        }
        let invoked = item.clone();
        match &mut item.item_type {
            MenuItemType::Checkbox { checked } => *checked = !*checked,
            MenuItemType::Radio { selected, .. } => *selected = true,
            _ => {}
        }
        if let MenuItemType::Radio { group, .. } = &invoked.item_type {
            let path = self.path.clone();
            let parent = &path[..path.len() - 1];
            let siblings = if parent.is_empty() {
                Some(self.items.as_mut_slice())
            } else {
                item_mut(&mut self.items, parent).map(|item| item.submenu.as_mut_slice())
            };
            if let Some(siblings) = siblings {
                for (index, sibling) in siblings.iter_mut().enumerate() {
                    if let MenuItemType::Radio {
                        group: other,
                        selected,
                    } = &mut sibling.item_type
                    {
                        if group == other {
                            *selected = Some(&index) == path.last();
                        }
                    }
                }
            }
        }
        Some(invoked)
    }
    pub fn repair_selection(&mut self) {
        let mut valid = Vec::new();
        for index in self.path.iter().copied().chain(std::iter::once(usize::MAX)) {
            let items = list_at(&self.items, &valid);
            let index = items
                .get(index)
                .filter(|item| item.is_selectable())
                .map(|_| index)
                .or_else(|| items.iter().position(MenuItem::is_selectable));
            let Some(index) = index else {
                break;
            };
            valid.push(index);
            if valid.len() >= self.path.len().max(1) {
                break;
            }
        }
        self.path = valid;
    }
    pub fn hover(&mut self, path: Vec<usize>) -> bool {
        let Some(item) = item_mut(&mut self.items, &path).filter(|item| item.is_selectable())
        else {
            return false;
        };
        let submenu = item.has_submenu();
        if self.path.starts_with(&path) && (self.path.len() > path.len() || !submenu) {
            return false;
        }
        let child = submenu
            .then(|| item.submenu.iter().position(MenuItem::is_selectable))
            .flatten();
        self.path = path;
        if let Some(child) = child {
            self.path.push(child);
        }
        true
    }
    pub fn wheel(&mut self, target: &[usize], wheel: &crate::event::types::WheelEvent) -> bool {
        use crate::event::types::WheelDelta;
        let delta = match wheel.delta {
            WheelDelta::Lines { y, .. } => y,
            WheelDelta::Pixels { y, .. } => y / 16.0,
        };
        if !delta.is_finite() || delta == 0.0 || target.is_empty() {
            return false;
        }
        let parent = &target[..target.len() - 1];
        if self.path.len() < target.len() || !self.path.starts_with(parent) {
            self.path = target.to_vec();
        } else {
            self.path.truncate(target.len());
        }
        let steps = delta.abs().round().max(1.0) as isize;
        self.move_selection(if delta > 0.0 { steps } else { -steps });
        true
    }
    pub fn move_selection(&mut self, delta: isize) {
        let depth = self.path.len().saturating_sub(1);
        let items = list_at(&self.items, &self.path[..depth]);
        let eligible: Vec<_> = items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.is_selectable())
            .map(|(i, _)| i)
            .collect();
        if eligible.is_empty() {
            return;
        }
        let current = self
            .path
            .last()
            .and_then(|index| eligible.iter().position(|i| i == index))
            .unwrap_or(0);
        let next = (current as i128 + delta as i128).rem_euclid(eligible.len() as i128) as usize;
        self.path.truncate(depth);
        self.path.push(eligible[next]);
    }
}
