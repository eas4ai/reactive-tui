//! Named key actions and the keymap that binds keys to them
//! (docs/spec/keymap.md).
//!
//! A widget asks the active keymap which [`Action`] a key means instead of
//! matching key codes itself, so an application rebinds a key once and every
//! widget follows (KEY-001). A [`KeyBinding`] has one text form, the hint a
//! menu shows beside an item and the text the screen reader is told
//! (KEY-002). Typing is not an action: a widget that takes text keeps taking a
//! plain character as text, so a binding to a plain character reaches only
//! widgets that take no text.

use std::fmt;
use std::sync::{Arc, OnceLock, RwLock};

use crate::event::types::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// A named key action, bound to keys by a [`Keymap`].
///
/// The defaults, in the order of `Action::ALL`: Confirm (Enter), Activate
/// (Space), Cancel (Escape), Up, Down, Left, Right, Home, End, PageUp and
/// PageDown (the keys of those names), Next (Tab), Previous (Shift+Tab),
/// Delete (Delete), Sort (`s`), Expand (`+`), Collapse (`-`), ContextMenu
/// (Shift+F10), Copy (Ctrl+C), Cut (Ctrl+X), Paste (Ctrl+V), Undo (Ctrl+Z),
/// Redo (Ctrl+Y) and Search (Ctrl+F).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    /// Take the focused choice: choose the option, open the panel, follow the link.
    Confirm,
    /// Toggle or press the focused control: a box, a radio, a button.
    Activate,
    /// Close what is open or leave what was started.
    Cancel,
    /// Move up by one.
    Up,
    /// Move down by one.
    Down,
    /// Move left by one.
    Left,
    /// Move right by one.
    Right,
    /// Move to the first.
    Home,
    /// Move to the last.
    End,
    /// Move up by one page.
    PageUp,
    /// Move down by one page.
    PageDown,
    /// Move the focus to the next stop.
    Next,
    /// Move the focus to the previous stop.
    Previous,
    /// Remove the focused item: close a tab, delete the selection.
    Delete,
    /// Sort by the focused column.
    Sort,
    /// Expand the focused node or section.
    Expand,
    /// Collapse the focused node or section.
    Collapse,
    /// Open the context menu, or a widget's overflow menu, for the focused element.
    ContextMenu,
    /// Copy the selection.
    Copy,
    /// Cut the selection.
    Cut,
    /// Paste.
    Paste,
    /// Undo the last edit.
    Undo,
    /// Redo the last undone edit.
    Redo,
    /// Open the search.
    Search,
}

impl Action {
    /// Every action, in the order the defaults are listed.
    pub const ALL: [Action; 24] = [
        Action::Confirm,
        Action::Activate,
        Action::Cancel,
        Action::Up,
        Action::Down,
        Action::Left,
        Action::Right,
        Action::Home,
        Action::End,
        Action::PageUp,
        Action::PageDown,
        Action::Next,
        Action::Previous,
        Action::Delete,
        Action::Sort,
        Action::Expand,
        Action::Collapse,
        Action::ContextMenu,
        Action::Copy,
        Action::Cut,
        Action::Paste,
        Action::Undo,
        Action::Redo,
        Action::Search,
    ];

    /// The action's name, as written in the specification.
    pub fn name(self) -> &'static str {
        match self {
            Action::Confirm => "Confirm",
            Action::Activate => "Activate",
            Action::Cancel => "Cancel",
            Action::Up => "Up",
            Action::Down => "Down",
            Action::Left => "Left",
            Action::Right => "Right",
            Action::Home => "Home",
            Action::End => "End",
            Action::PageUp => "PageUp",
            Action::PageDown => "PageDown",
            Action::Next => "Next",
            Action::Previous => "Previous",
            Action::Delete => "Delete",
            Action::Sort => "Sort",
            Action::Expand => "Expand",
            Action::Collapse => "Collapse",
            Action::ContextMenu => "ContextMenu",
            Action::Copy => "Copy",
            Action::Cut => "Cut",
            Action::Paste => "Paste",
            Action::Undo => "Undo",
            Action::Redo => "Redo",
            Action::Search => "Search",
        }
    }

    /// The action's default bindings.
    fn defaults(self) -> Vec<KeyBinding> {
        let key = KeyBinding::new;
        match self {
            Action::Confirm => vec![key(KeyCode::Enter)],
            Action::Activate => vec![key(KeyCode::Space)],
            Action::Cancel => vec![key(KeyCode::Escape)],
            Action::Up => vec![key(KeyCode::Up)],
            Action::Down => vec![key(KeyCode::Down)],
            Action::Left => vec![key(KeyCode::Left)],
            Action::Right => vec![key(KeyCode::Right)],
            Action::Home => vec![key(KeyCode::Home)],
            Action::End => vec![key(KeyCode::End)],
            Action::PageUp => vec![key(KeyCode::PageUp)],
            Action::PageDown => vec![key(KeyCode::PageDown)],
            Action::Next => vec![key(KeyCode::Tab)],
            Action::Previous => vec![key(KeyCode::Tab).shift()],
            Action::Delete => vec![key(KeyCode::Delete)],
            Action::Sort => vec![key(KeyCode::Char('s'))],
            Action::Expand => vec![key(KeyCode::Char('+'))],
            Action::Collapse => vec![key(KeyCode::Char('-'))],
            Action::ContextMenu => vec![key(KeyCode::F(10)).shift()],
            Action::Copy => vec![key(KeyCode::Char('c')).ctrl()],
            Action::Cut => vec![key(KeyCode::Char('x')).ctrl()],
            Action::Paste => vec![key(KeyCode::Char('v')).ctrl()],
            Action::Undo => vec![key(KeyCode::Char('z')).ctrl()],
            Action::Redo => vec![key(KeyCode::Char('y')).ctrl()],
            Action::Search => vec![key(KeyCode::Char('f')).ctrl()],
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// A key with its modifiers, as a keymap binds it and as a key event
/// presses it.
///
/// Two spellings of one key are one binding: `KeyCode::Char(' ')` is
/// `KeyCode::Space`, and `KeyCode::BackTab` is Shift+Tab. A plain character's
/// shift state is the terminal's business (`+` arrives with or without it), so
/// a binding to a plain character ignores it; every other key compares its
/// modifiers exactly.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    code: KeyCode,
    ctrl: bool,
    alt: bool,
    shift: bool,
    meta: bool,
}

impl KeyBinding {
    /// A binding to `code` with no modifier.
    pub fn new(code: KeyCode) -> Self {
        let (code, shift) = match code {
            KeyCode::Char(' ') => (KeyCode::Space, false),
            KeyCode::BackTab => (KeyCode::Tab, true),
            code => (code, false),
        };
        Self {
            code,
            ctrl: false,
            alt: false,
            shift,
            meta: false,
        }
    }

    /// The binding a key event presses: its key and modifiers, normalized
    /// as [`KeyBinding::new`] normalizes them.
    pub fn from_event(event: &KeyEvent) -> Self {
        let mut binding = Self::new(event.code.clone());
        binding.ctrl = event.modifiers.ctrl;
        binding.alt = event.modifiers.alt;
        binding.shift |= event.modifiers.shift;
        binding.meta = event.modifiers.meta;
        binding
    }

    /// With Ctrl held.
    pub fn ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    /// With Alt held.
    pub fn alt(mut self) -> Self {
        self.alt = true;
        self
    }

    /// With Shift held.
    pub fn shift(mut self) -> Self {
        self.shift = true;
        self
    }

    /// With Meta (Super, Command) held.
    pub fn meta(mut self) -> Self {
        self.meta = true;
        self
    }

    /// The key, without its modifiers.
    pub fn code(&self) -> &KeyCode {
        &self.code
    }

    /// The modifiers held.
    pub fn modifiers(&self) -> KeyModifiers {
        KeyModifiers {
            shift: self.shift,
            ctrl: self.ctrl,
            alt: self.alt,
            meta: self.meta,
        }
    }

    /// Whether `event` presses this binding. A release presses nothing.
    pub fn matches(&self, event: &KeyEvent) -> bool {
        if event.kind == KeyEventKind::Release {
            return false;
        }
        let pressed = Self::from_event(event);
        if self.ctrl != pressed.ctrl || self.alt != pressed.alt || self.meta != pressed.meta {
            return false;
        }
        match (&self.code, &pressed.code) {
            // A plain character: the character itself carries the shift.
            (KeyCode::Char(bound), KeyCode::Char(typed))
                if !self.ctrl && !self.alt && !self.meta =>
            {
                bound == typed
            }
            // A character with Ctrl, Alt or Meta: the terminal may report it
            // in either case, so the letter compares case-insensitively and
            // the shift flag decides.
            (KeyCode::Char(bound), KeyCode::Char(typed)) => {
                self.shift == pressed.shift && bound.to_lowercase().eq(typed.to_lowercase())
            }
            (bound, typed) => bound == typed && self.shift == pressed.shift,
        }
    }

    /// The binding's one text form: the modifiers as `Ctrl+`, `Alt+`,
    /// `Shift+` and `Meta+` in that order, then the key's name (`Enter`,
    /// `Escape`, `Space`, `Tab`, `F2`, `Up`, `Page Down`, a letter in upper
    /// case).
    pub fn display(&self) -> String {
        let mut text = String::new();
        if self.ctrl {
            text.push_str("Ctrl+");
        }
        if self.alt {
            text.push_str("Alt+");
        }
        if self.shift {
            text.push_str("Shift+");
        }
        if self.meta {
            text.push_str("Meta+");
        }
        text.push_str(&Self::key_name(&self.code));
        text
    }

    fn key_name(code: &KeyCode) -> String {
        match code {
            KeyCode::Char(c) => c.to_uppercase().collect(),
            KeyCode::F(n) => format!("F{n}"),
            KeyCode::Enter => "Enter".into(),
            KeyCode::Escape => "Escape".into(),
            KeyCode::Space => "Space".into(),
            KeyCode::Tab => "Tab".into(),
            KeyCode::BackTab => "Shift+Tab".into(),
            KeyCode::Backspace => "Backspace".into(),
            KeyCode::Delete => "Delete".into(),
            KeyCode::Insert => "Insert".into(),
            KeyCode::Home => "Home".into(),
            KeyCode::End => "End".into(),
            KeyCode::PageUp => "Page Up".into(),
            KeyCode::PageDown => "Page Down".into(),
            KeyCode::Up => "Up".into(),
            KeyCode::Down => "Down".into(),
            KeyCode::Left => "Left".into(),
            KeyCode::Right => "Right".into(),
            KeyCode::CapsLock => "Caps Lock".into(),
            KeyCode::NumLock => "Num Lock".into(),
            KeyCode::ScrollLock => "Scroll Lock".into(),
            KeyCode::MediaPlay => "Play".into(),
            KeyCode::MediaPause => "Pause".into(),
            KeyCode::MediaPlayPause => "Play/Pause".into(),
            KeyCode::MediaStop => "Stop".into(),
            KeyCode::MediaNext => "Next Track".into(),
            KeyCode::MediaPrevious => "Previous Track".into(),
            KeyCode::Null | KeyCode::Unknown => "?".into(),
        }
    }
}

impl fmt::Display for KeyBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.display())
    }
}

/// Which keys mean which [`Action`].
///
/// `Keymap::default()` holds the default bindings; [`Keymap::bind`] adds one,
/// [`Keymap::unbind`] removes one and [`Keymap::rebind`] replaces an action's
/// bindings. The keymap every widget reads is the active one,
/// [`Keymap::active`], changed by [`Keymap::set_active`] or
/// `App::set_keymap`.
#[derive(Clone, Debug, PartialEq)]
pub struct Keymap {
    bindings: Vec<(Action, KeyBinding)>,
}

impl Default for Keymap {
    fn default() -> Self {
        let bindings = Action::ALL
            .iter()
            .flat_map(|action| {
                action
                    .defaults()
                    .into_iter()
                    .map(move |binding| (*action, binding))
            })
            .collect();
        Self { bindings }
    }
}

fn active_slot() -> &'static RwLock<Arc<Keymap>> {
    static ACTIVE: OnceLock<RwLock<Arc<Keymap>>> = OnceLock::new();
    ACTIVE.get_or_init(|| RwLock::new(Arc::new(Keymap::default())))
}

impl Keymap {
    /// A keymap with no binding at all.
    pub fn empty() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }

    /// Adds `key` as a binding of `action`, keeping the others.
    pub fn bind(&mut self, action: Action, key: KeyBinding) -> &mut Self {
        if !self.bindings.iter().any(|(a, k)| *a == action && *k == key) {
            self.bindings.push((action, key));
        }
        self
    }

    /// Removes `key` from the bindings of `action`.
    pub fn unbind(&mut self, action: Action, key: &KeyBinding) -> &mut Self {
        self.bindings.retain(|(a, k)| !(*a == action && k == key));
        self
    }

    /// Replaces every binding of `action` with `keys`.
    pub fn rebind(
        &mut self,
        action: Action,
        keys: impl IntoIterator<Item = KeyBinding>,
    ) -> &mut Self {
        self.bindings.retain(|(a, _)| *a != action);
        for key in keys {
            self.bind(action, key);
        }
        self
    }

    /// The bindings of `action`, in the order they were bound.
    pub fn bindings(&self, action: Action) -> Vec<KeyBinding> {
        self.bindings
            .iter()
            .filter(|(a, _)| *a == action)
            .map(|(_, k)| k.clone())
            .collect()
    }

    /// The first binding of `action`, the one a hint shows.
    pub fn binding(&self, action: Action) -> Option<KeyBinding> {
        self.bindings
            .iter()
            .find(|(a, _)| *a == action)
            .map(|(_, k)| k.clone())
    }

    /// The first action `event` presses, or none: a release presses nothing,
    /// and so does a key no binding names.
    pub fn action(&self, event: &KeyEvent) -> Option<Action> {
        self.bindings
            .iter()
            .find(|(_, key)| key.matches(event))
            .map(|(action, _)| *action)
    }

    /// Every action `event` presses, in binding order.
    pub fn actions(&self, event: &KeyEvent) -> Vec<Action> {
        self.bindings
            .iter()
            .filter(|(_, key)| key.matches(event))
            .map(|(action, _)| *action)
            .collect()
    }

    /// Whether `event` presses a binding of `action`.
    pub fn is(&self, event: &KeyEvent, action: Action) -> bool {
        self.bindings
            .iter()
            .any(|(a, key)| *a == action && key.matches(event))
    }

    /// The keymap every widget reads.
    pub fn active() -> Arc<Keymap> {
        Arc::clone(&active_slot().read().unwrap_or_else(|e| e.into_inner()))
    }

    /// Makes `keymap` the one every widget reads from the next event on;
    /// returns it.
    pub fn set_active(keymap: Keymap) -> Arc<Keymap> {
        let keymap = Arc::new(keymap);
        *active_slot().write().unwrap_or_else(|e| e.into_inner()) = Arc::clone(&keymap);
        keymap
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code)
    }

    fn press_with(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code).with_modifiers(modifiers)
    }

    /// KEY-001: every listed action has the default binding the specification
    /// names, read back through its text form.
    #[test]
    fn key_001_every_action_has_its_default_binding() {
        let keymap = Keymap::default();
        let expected = [
            (Action::Confirm, "Enter"),
            (Action::Activate, "Space"),
            (Action::Cancel, "Escape"),
            (Action::Up, "Up"),
            (Action::Down, "Down"),
            (Action::Left, "Left"),
            (Action::Right, "Right"),
            (Action::Home, "Home"),
            (Action::End, "End"),
            (Action::PageUp, "Page Up"),
            (Action::PageDown, "Page Down"),
            (Action::Next, "Tab"),
            (Action::Previous, "Shift+Tab"),
            (Action::Delete, "Delete"),
            (Action::Sort, "S"),
            (Action::Expand, "+"),
            (Action::Collapse, "-"),
            (Action::ContextMenu, "Shift+F10"),
            (Action::Copy, "Ctrl+C"),
            (Action::Cut, "Ctrl+X"),
            (Action::Paste, "Ctrl+V"),
            (Action::Undo, "Ctrl+Z"),
            (Action::Redo, "Ctrl+Y"),
            (Action::Search, "Ctrl+F"),
        ];
        assert_eq!(expected.len(), Action::ALL.len());
        for (action, text) in expected {
            let bindings = keymap.bindings(action);
            assert_eq!(
                bindings.iter().map(KeyBinding::display).collect::<Vec<_>>(),
                vec![text.to_string()],
                "{action} has one default binding, {text}"
            );
        }
    }

    /// KEY-001: a key presses the action bound to it, a release presses
    /// nothing, and the two spellings of Space and of Shift+Tab are one key.
    #[test]
    fn key_001_a_press_names_its_action_and_a_release_none() {
        let keymap = Keymap::default();
        assert_eq!(keymap.action(&press(KeyCode::Enter)), Some(Action::Confirm));
        assert_eq!(
            keymap.action(&press(KeyCode::Space)),
            Some(Action::Activate)
        );
        assert_eq!(
            keymap.action(&press(KeyCode::Char(' '))),
            Some(Action::Activate)
        );
        assert_eq!(
            keymap.action(&press(KeyCode::BackTab)),
            Some(Action::Previous)
        );
        assert_eq!(
            keymap.action(&press_with(KeyCode::Tab, KeyModifiers::shift())),
            Some(Action::Previous)
        );
        assert_eq!(
            keymap.action(&press_with(KeyCode::F(10), KeyModifiers::shift())),
            Some(Action::ContextMenu)
        );
        assert_eq!(keymap.action(&press(KeyCode::F(10))), None);
        assert_eq!(
            keymap.action(&press_with(KeyCode::Char('c'), KeyModifiers::ctrl())),
            Some(Action::Copy)
        );
        assert_eq!(keymap.action(&press(KeyCode::Char('c'))), None);
        assert_eq!(
            keymap.action(&press(KeyCode::Enter).with_kind(KeyEventKind::Release)),
            None
        );
        assert!(keymap.is(&press(KeyCode::Char('s')), Action::Sort));
        assert!(!keymap.is(&press(KeyCode::Char('S')), Action::Sort));
    }

    /// KEY-001: a plain character ignores the terminal's shift flag, while a
    /// character with Ctrl compares the flag and the letter in either case.
    #[test]
    fn key_001_a_plain_character_ignores_shift_and_a_ctrl_letter_does_not() {
        let keymap = Keymap::default();
        assert_eq!(
            keymap.action(&press_with(KeyCode::Char('+'), KeyModifiers::shift())),
            Some(Action::Expand)
        );
        let mut rebound = Keymap::default();
        rebound.rebind(
            Action::Copy,
            [KeyBinding::new(KeyCode::Char('c')).ctrl().shift()],
        );
        let ctrl_shift = KeyModifiers {
            shift: true,
            ..KeyModifiers::ctrl()
        };
        assert_eq!(
            rebound.action(&press_with(KeyCode::Char('c'), ctrl_shift)),
            Some(Action::Copy)
        );
        assert_eq!(
            rebound.action(&press_with(KeyCode::Char('C'), ctrl_shift)),
            Some(Action::Copy)
        );
        assert_eq!(
            rebound.action(&press_with(KeyCode::Char('c'), KeyModifiers::ctrl())),
            None
        );
    }

    /// KEY-001: `bind` adds, `unbind` removes, `rebind` replaces.
    #[test]
    fn key_001_bind_adds_unbind_removes_and_rebind_replaces() {
        let mut keymap = Keymap::default();
        keymap.bind(Action::Confirm, KeyBinding::new(KeyCode::F(2)));
        assert_eq!(keymap.action(&press(KeyCode::F(2))), Some(Action::Confirm));
        assert_eq!(keymap.action(&press(KeyCode::Enter)), Some(Action::Confirm));
        keymap.bind(Action::Confirm, KeyBinding::new(KeyCode::F(2)));
        assert_eq!(
            keymap.bindings(Action::Confirm).len(),
            2,
            "a binding is added once"
        );
        keymap.unbind(Action::Confirm, &KeyBinding::new(KeyCode::Enter));
        assert_eq!(keymap.action(&press(KeyCode::Enter)), None);
        keymap.rebind(Action::Confirm, [KeyBinding::new(KeyCode::F(3))]);
        assert_eq!(keymap.action(&press(KeyCode::F(2))), None);
        assert_eq!(keymap.action(&press(KeyCode::F(3))), Some(Action::Confirm));
        assert_eq!(
            keymap.binding(Action::Confirm).map(|k| k.display()),
            Some("F3".to_string())
        );
        assert!(Keymap::empty().action(&press(KeyCode::Enter)).is_none());
    }

    /// KEY-001: the active keymap is the default until one is set, and the
    /// one set is what `active` returns.
    #[test]
    #[serial_test::serial(keymap)]
    fn key_001_the_active_keymap_is_the_default_until_one_is_set() {
        let before = Keymap::active();
        assert_eq!(*before, Keymap::default());
        let mut rebound = Keymap::default();
        rebound.rebind(Action::Confirm, [KeyBinding::new(KeyCode::F(2))]);
        Keymap::set_active(rebound.clone());
        assert_eq!(*Keymap::active(), rebound);
        Keymap::set_active(Keymap::default());
        assert_eq!(*Keymap::active(), Keymap::default());
    }

    /// KEY-002: the text form writes the modifiers in order, then the key's
    /// name.
    #[test]
    fn key_002_a_binding_has_one_text_form() {
        assert_eq!(
            KeyBinding::new(KeyCode::F(10)).ctrl().shift().display(),
            "Ctrl+Shift+F10"
        );
        assert_eq!(KeyBinding::new(KeyCode::PageDown).display(), "Page Down");
        assert_eq!(KeyBinding::new(KeyCode::Char('s')).display(), "S");
        assert_eq!(KeyBinding::new(KeyCode::Char(' ')).display(), "Space");
        assert_eq!(KeyBinding::new(KeyCode::BackTab).display(), "Shift+Tab");
        assert_eq!(
            KeyBinding::new(KeyCode::Enter).meta().alt().display(),
            "Alt+Meta+Enter"
        );
        assert_eq!(
            KeyBinding::new(KeyCode::Char('c')).ctrl().to_string(),
            "Ctrl+C"
        );
        assert_eq!(Action::PageDown.to_string(), "PageDown");
    }
}
