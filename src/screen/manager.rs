use super::{Screen, ScreenHooks, ScreenId, TransitionConfig, TransitionState, TransitionType};
use crate::backend::Backend;
use crate::component::Element;
use crate::event::types as rt_event;
use std::collections::HashMap;

/// Manages multiple screens with transitions
pub struct ScreenManager {
    screens: HashMap<ScreenId, Screen>,
    active_screen: Option<ScreenId>,
    backend: Box<dyn Backend>,
    presented: crate::render::RenderTree,
    transition_state: TransitionState,
    default_transition: TransitionConfig,
    global_hotkeys: HashMap<rt_event::KeyCode, ScreenId>,
}

impl ScreenManager {
    /// Create a new screen manager with the given backend
    pub fn new(backend: Box<dyn Backend>) -> Self {
        Self {
            screens: HashMap::new(),
            active_screen: None,
            backend,
            presented: crate::render::RenderTree::new(),
            transition_state: TransitionState::default(),
            default_transition: TransitionConfig::default(),
            global_hotkeys: HashMap::new(),
        }
    }

    /// Set the default transition configuration
    pub fn set_default_transition(&mut self, config: TransitionConfig) {
        self.default_transition = config;
    }

    /// Create a new screen
    pub fn create_screen(
        &mut self,
        id: impl Into<ScreenId>,
        name: String,
        element: Element,
    ) -> Result<(), String> {
        let screen_id = id.into();

        if self.screens.contains_key(&screen_id) {
            return Err(format!(
                "Screen with id '{}' already exists",
                screen_id.as_str()
            ));
        }

        let mut screen = Screen::new(screen_id.clone(), name);
        screen.set_content(element);

        // Call create hook
        if let Some(ref on_create) = screen.hooks.on_create {
            on_create();
        }

        self.screens.insert(screen_id.clone(), screen);

        // If this is the first screen, make it active
        if self.active_screen.is_none() {
            self.switch_to_immediate(screen_id)?;
        }

        Ok(())
    }

    /// Create a screen with lifecycle hooks
    pub fn create_screen_with_hooks(
        &mut self,
        id: impl Into<ScreenId>,
        name: String,
        element: Element,
        hooks: ScreenHooks,
    ) -> Result<(), String> {
        let screen_id = id.into();

        if self.screens.contains_key(&screen_id) {
            return Err(format!(
                "Screen with id '{}' already exists",
                screen_id.as_str()
            ));
        }

        let mut screen = Screen::new(screen_id.clone(), name).with_hooks(hooks);
        screen.set_content(element);

        // Call create hook
        if let Some(ref on_create) = screen.hooks.on_create {
            on_create();
        }

        self.screens.insert(screen_id.clone(), screen);

        // If this is the first screen, make it active
        if self.active_screen.is_none() {
            self.switch_to_immediate(screen_id)?;
        }

        Ok(())
    }

    /// Remove a screen
    pub fn remove_screen(&mut self, id: &ScreenId) -> Result<(), String> {
        if !self.screens.contains_key(id) {
            return Err(format!("Screen with id '{}' does not exist", id.as_str()));
        }

        // If removing the active screen, switch to another one
        if self.active_screen.as_ref() == Some(id) {
            let other_screen = self.screens.keys().find(|&k| k != id).cloned();

            if let Some(other_id) = other_screen {
                self.switch_to_immediate(other_id)?;
            } else {
                self.screens
                    .get_mut(id)
                    .expect("existing screen")
                    .runtime
                    .render(
                        Element::text(""),
                        self.backend.as_mut(),
                        &mut self.presented,
                    )
                    .map_err(|error| error.to_string())?;
                self.active_screen = None;
                self.transition_state = TransitionState::default();
            }
        }

        if self.transition_state.to_screen.as_ref() == Some(id) {
            self.transition_state = TransitionState::default();
            self.render_active_screen()?;
        }
        self.global_hotkeys.retain(|_, target| target != id);
        self.screens.remove(id);
        Ok(())
    }

    /// Switch to a screen immediately (no transition)
    pub fn switch_to_immediate(&mut self, id: ScreenId) -> Result<(), String> {
        if !self.screens.contains_key(&id) {
            return Err(format!("Screen with id '{}' does not exist", id.as_str()));
        }

        self.transition_state = TransitionState::default();

        // Deactivate current screen
        if let Some(ref current_id) = self.active_screen {
            if let Some(screen) = self.screens.get_mut(current_id) {
                screen.deactivate();
            }
        }

        // Activate new screen
        if let Some(screen) = self.screens.get_mut(&id) {
            screen.activate();
        }

        self.active_screen = Some(id);
        self.render_active_screen()?;

        Ok(())
    }

    /// Switch to a screen with transition
    pub fn switch_to(&mut self, id: ScreenId) -> Result<(), String> {
        self.switch_to_with_transition(id, None)
    }

    /// Switch to a screen with custom transition
    pub fn switch_to_with_transition(
        &mut self,
        id: ScreenId,
        transition: Option<TransitionConfig>,
    ) -> Result<(), String> {
        if !self.screens.contains_key(&id) {
            return Err(format!("Screen with id '{}' does not exist", id.as_str()));
        }

        if self.active_screen.as_ref() == Some(&id) {
            return Ok(()); // Already on this screen
        }

        let config = transition.unwrap_or_else(|| self.default_transition.clone());

        // Start transition
        self.transition_state
            .start_transition(self.active_screen.clone(), id.clone(), config);

        // If no transition, switch immediately
        if self.transition_state.config.transition_type == TransitionType::None {
            self.complete_transition()?;
        }

        Ok(())
    }

    /// Update the content of a specific screen
    pub fn update_screen(&mut self, id: &ScreenId, element: Element) -> Result<(), String> {
        let screen = self
            .screens
            .get_mut(id)
            .ok_or_else(|| format!("Screen with id '{}' does not exist", id.as_str()))?;

        screen.update_content(element);

        // If this is the active screen and not transitioning, re-render
        if self.active_screen.as_ref() == Some(id) && !self.transition_state.is_transitioning {
            self.render_active_screen()?;
        }

        Ok(())
    }

    /// Get the currently active screen ID
    pub fn get_active_screen(&self) -> Option<&ScreenId> {
        self.active_screen.as_ref()
    }

    /// Get a list of all screen IDs
    pub fn get_screen_ids(&self) -> Vec<&ScreenId> {
        self.screens.keys().collect()
    }

    /// Check if a screen exists
    pub fn has_screen(&self, id: &ScreenId) -> bool {
        self.screens.contains_key(id)
    }

    /// Set a global hotkey for screen switching
    pub fn set_hotkey(&mut self, key: rt_event::KeyCode, screen_id: ScreenId) {
        self.global_hotkeys.insert(key, screen_id);
    }

    /// Process an event (handles global hotkeys and routes to active screen)
    pub fn process_event(&mut self, event: &rt_event::Event) -> Result<bool, String> {
        // Handle global hotkeys
        if let rt_event::Event::Key(key_event) = event {
            if key_event.kind == rt_event::KeyEventKind::Press {
                if let Some(screen_id) = self.global_hotkeys.get(&key_event.code).cloned() {
                    self.switch_to(screen_id)?;
                    return Ok(true); // Event consumed
                }
            }
        }
        if let rt_event::Event::Resize(size) = event {
            if size.width > 0 && size.height > 0 {
                self.backend
                    .resize(usize::from(size.width), usize::from(size.height));
                self.update()?;
            }
            return Ok(true);
        }
        let handled = self
            .active_screen
            .as_ref()
            .and_then(|id| self.screens.get_mut(id))
            .is_some_and(|screen| screen.runtime.process_event(event));
        if handled {
            self.update()?;
        }
        Ok(handled)
    }

    /// Update transitions and render
    pub fn update(&mut self) -> Result<(), String> {
        // Update transition state
        let transition_complete = self.transition_state.update();

        if transition_complete {
            self.complete_transition()?;
        } else if self.transition_state.is_transitioning {
            self.render_transition()?;
        } else {
            self.render_active_screen()?;
        }

        Ok(())
    }

    /// Wait until every frame this manager presented has been written. An
    /// operation returns once its frame is submitted and the backend may
    /// write it afterwards (PIP-001); a caller that reads the output, such
    /// as a test, calls this first.
    pub fn sync(&mut self) -> Result<(), String> {
        self.backend.sync().map_err(|error| error.to_string())
    }

    /// Complete the current transition
    fn complete_transition(&mut self) -> Result<(), String> {
        if let Some(to_screen) = self.transition_state.to_screen.clone() {
            // Deactivate old screen
            if let Some(ref from_screen) = self.transition_state.from_screen {
                if let Some(screen) = self.screens.get_mut(from_screen) {
                    screen.deactivate();
                }
            }

            // Activate new screen
            if let Some(screen) = self.screens.get_mut(&to_screen) {
                screen.activate();
            }

            self.active_screen = Some(to_screen);
        }

        // Reset transition state
        self.transition_state = TransitionState::default();

        // Render the new active screen
        self.render_active_screen()?;

        Ok(())
    }

    /// Render the active screen
    fn render_active_screen(&mut self) -> Result<(), String> {
        if let Some(screen) = self
            .active_screen
            .as_ref()
            .and_then(|id| self.screens.get_mut(id))
        {
            let element = screen
                .root_element
                .clone()
                .unwrap_or_else(|| Element::text(""));
            screen
                .runtime
                .render(element, self.backend.as_mut(), &mut self.presented)
                .map_err(|error| format!("Failed to render active screen: {error}"))?;
        }
        Ok(())
    }

    /// Render transition between screens
    fn render_transition(&mut self) -> Result<(), String> {
        let Some(to_id) = self.transition_state.to_screen.clone() else {
            return Ok(());
        };
        let from_id = self.transition_state.from_screen.clone();
        let width = self.backend.size().0;
        let mut prepare = |id: &ScreenId| -> Result<Element, String> {
            let screen = self
                .screens
                .get_mut(id)
                .ok_or_else(|| format!("Missing transition screen {}", id.as_str()))?;
            screen
                .runtime
                .prepare(
                    screen
                        .root_element
                        .clone()
                        .unwrap_or_else(|| Element::text("")),
                    width,
                )
                .map_err(|error| error.to_string())
        };
        let from = from_id
            .as_ref()
            .map(&mut prepare)
            .transpose()?
            .unwrap_or_else(|| Element::text(""));
        let to = prepare(&to_id)?;
        let frame = super::composition::compose(
            from.clone(),
            to.clone(),
            self.transition_state.config.transition_type,
            self.transition_state.progress,
        );
        let owner = from_id.as_ref().unwrap_or(&to_id);
        let runtime = &mut self
            .screens
            .get_mut(owner)
            .expect("prepared screen")
            .runtime;
        runtime
            .present(&frame, self.backend.as_mut(), &mut self.presented)
            .map_err(|error| error.to_string())?;
        runtime
            .animation_targets
            .publish(&from, self.backend.component_layouts(), 2)
            .map_err(|error| error.to_string())?;
        if from_id.is_some() {
            runtime.acknowledge_layer(&from, self.backend.as_ref());
        }
        self.screens
            .get_mut(&to_id)
            .expect("prepared destination")
            .runtime
            .animation_targets
            .publish(
                &to,
                self.backend.component_layouts(),
                3 + super::composition::node_count(&from),
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    /// Look up targets in one screen, isolated from other screens with the same IDs.
    /// The screen must have presented a frame before its targets can be resolved.
    pub fn animation_targets(
        &self,
        screen: &ScreenId,
    ) -> Result<crate::animation::AnimationTargetContext, crate::animation::AnimationTargetError>
    {
        self.screens
            .get(screen)
            .map(|screen| screen.runtime.animation_targets.context())
            .ok_or_else(|| {
                crate::animation::AnimationTargetError::MissingTarget(screen.as_str().into())
            })
    }

    /// Find one uniquely keyed target in a screen's last presented tree.
    pub fn animation_target(
        &self,
        screen: &ScreenId,
        id: &str,
    ) -> Result<crate::animation::AnimationTarget, crate::animation::AnimationTargetError> {
        self.animation_targets(screen)?.target(id)
    }

    /// Get backend size
    pub fn size(&self) -> (u16, u16) {
        self.backend.size()
    }

    /// Check if currently transitioning
    pub fn is_transitioning(&self) -> bool {
        self.transition_state.is_transitioning
    }
}

#[cfg(test)]
mod tests;
