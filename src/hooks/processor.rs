use super::mouse::{
    coordinate_delta, ClickState, DragAndDropOptions, DragAndDropState, DragState, GestureState,
    GestureType, HoverState, LongPressState, MousePositionState, SwipeDirection, WheelDeltaMode,
    WheelState,
};
use crate::event::types::{MouseButton, MouseEvent, MouseEventKind, Position};
use crate::reactive::hooks::ThreadSafeSignal;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// A 2D velocity vector
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Velocity {
    /// Velocity in the X direction (horizontal)
    pub dx: f64,
    /// Velocity in the Y direction (vertical)
    pub dy: f64,
}

impl Velocity {
    /// Create a new velocity with given x and y components
    pub fn new(dx: f64, dy: f64) -> Self {
        Self { dx, dy }
    }

    /// Calculate the magnitude (speed) of the velocity vector
    pub fn magnitude(&self) -> f64 {
        (self.dx * self.dx + self.dy * self.dy).sqrt()
    }
}

/// Processes raw mouse events and updates hook states
#[derive(Clone)]
pub struct MouseEventProcessor {
    /// Track which component the mouse is over
    hovered_component: Arc<Mutex<Option<String>>>,

    /// Track drag state
    drag_start: Arc<Mutex<Option<ActiveDrag>>>,

    /// Track click timing for double/triple click detection
    last_click: Arc<Mutex<Option<ClickHistory>>>,

    /// Track long press state
    press_start: Arc<Mutex<Option<ActivePress>>>,

    /// Track gesture detection
    gesture_points: Arc<Mutex<Vec<(Position, Instant)>>>,

    /// Component that owns the current gesture point history.
    gesture_owner: Arc<Mutex<Option<String>>>,

    /// Component hover states
    hover_states: Arc<Mutex<HashMap<String, ThreadSafeSignal<HoverState>>>>,

    /// Component drag states
    drag_states: Arc<Mutex<HashMap<String, ThreadSafeSignal<DragState>>>>,

    /// Component click states
    click_states: Arc<Mutex<HashMap<String, ThreadSafeSignal<ClickState>>>>,

    /// Component mouse position states
    position_states: Arc<Mutex<HashMap<String, ThreadSafeSignal<MousePositionState>>>>,

    /// Component long press states
    press_states: Arc<Mutex<HashMap<String, ThreadSafeSignal<LongPressState>>>>,

    /// Per-component long-press thresholds.
    press_thresholds: Arc<Mutex<HashMap<String, Duration>>>,

    /// Per-component drag-and-drop state and options.
    drag_and_drop_states: Arc<Mutex<HashMap<String, DragAndDropRegistration>>>,

    /// Component wheel states
    wheel_states: Arc<Mutex<HashMap<String, ThreadSafeSignal<WheelState>>>>,

    /// Component gesture states
    gesture_states: Arc<Mutex<HashMap<String, ThreadSafeSignal<GestureState>>>>,
}

#[derive(Clone)]
struct ActiveDrag {
    owner: String,
    route_id: String,
    position: Position,
    started: Instant,
    button: MouseButton,
}

#[derive(Clone)]
struct ActivePress {
    owner: String,
    position: Position,
    started: Instant,
}

struct ClickHistory {
    owner: String,
    position: Position,
    occurred: Instant,
    count: usize,
}

#[derive(Clone)]
struct DragAndDropRegistration {
    signal: ThreadSafeSignal<DragAndDropState>,
    options: DragAndDropOptions,
}

/// App component ownership supplied privately while a component renders.
#[derive(Clone)]
pub(crate) struct MouseHookContext {
    pub(crate) owner: String,
    pub(crate) processor: MouseEventProcessor,
}

impl Default for MouseEventProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl MouseEventProcessor {
    /// Create a new mouse event processor
    pub fn new() -> Self {
        Self {
            hovered_component: Arc::new(Mutex::new(None)),
            drag_start: Arc::new(Mutex::new(None)),
            last_click: Arc::new(Mutex::new(None)),
            press_start: Arc::new(Mutex::new(None)),
            gesture_points: Arc::new(Mutex::new(Vec::new())),
            gesture_owner: Arc::new(Mutex::new(None)),
            hover_states: Arc::new(Mutex::new(HashMap::new())),
            drag_states: Arc::new(Mutex::new(HashMap::new())),
            click_states: Arc::new(Mutex::new(HashMap::new())),
            position_states: Arc::new(Mutex::new(HashMap::new())),
            press_states: Arc::new(Mutex::new(HashMap::new())),
            press_thresholds: Arc::new(Mutex::new(HashMap::new())),
            drag_and_drop_states: Arc::new(Mutex::new(HashMap::new())),
            wheel_states: Arc::new(Mutex::new(HashMap::new())),
            gesture_states: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a component's hover state signal
    pub fn register_hover(&self, component_id: String, signal: ThreadSafeSignal<HoverState>) {
        self.hover_states
            .lock()
            .unwrap()
            .insert(component_id, signal);
    }

    /// Register a component's drag state signal
    pub fn register_drag(&self, component_id: String, signal: ThreadSafeSignal<DragState>) {
        self.drag_states
            .lock()
            .unwrap()
            .insert(component_id, signal);
    }

    /// Register a component's click state signal
    pub fn register_clicks(&self, component_id: String, signal: ThreadSafeSignal<ClickState>) {
        self.click_states
            .lock()
            .unwrap()
            .insert(component_id, signal);
    }

    /// Register a component's mouse position state signal
    pub fn register_position(
        &self,
        component_id: String,
        signal: ThreadSafeSignal<MousePositionState>,
    ) {
        self.position_states
            .lock()
            .unwrap()
            .insert(component_id, signal);
    }

    /// Register a component's long press state signal
    pub fn register_press(&self, component_id: String, signal: ThreadSafeSignal<LongPressState>) {
        self.register_press_with_threshold(component_id, signal, Duration::from_millis(800));
    }

    /// Register a component's long press state with its detection threshold.
    pub fn register_press_with_threshold(
        &self,
        component_id: String,
        signal: ThreadSafeSignal<LongPressState>,
        threshold: Duration,
    ) {
        self.press_states
            .lock()
            .unwrap()
            .insert(component_id.clone(), signal);
        self.press_thresholds
            .lock()
            .unwrap()
            .insert(component_id, threshold);
    }

    /// Register a component's combined drag-and-drop state and options.
    pub fn register_drag_and_drop(
        &self,
        component_id: String,
        signal: ThreadSafeSignal<DragAndDropState>,
        options: DragAndDropOptions,
    ) {
        self.drag_and_drop_states
            .lock()
            .unwrap()
            .insert(component_id, DragAndDropRegistration { signal, options });
    }

    /// Register a component's wheel state signal
    pub fn register_wheel(&self, component_id: String, signal: ThreadSafeSignal<WheelState>) {
        self.wheel_states
            .lock()
            .unwrap()
            .insert(component_id, signal);
    }

    /// Register a component's gesture state signal
    pub fn register_gesture(&self, component_id: String, signal: ThreadSafeSignal<GestureState>) {
        self.gesture_states
            .lock()
            .unwrap()
            .insert(component_id, signal);
    }

    /// Remove every hook registration owned by one component.
    pub fn unregister_component(&self, component_id: &str) {
        self.hover_states.lock().unwrap().remove(component_id);
        self.drag_states.lock().unwrap().remove(component_id);
        self.click_states.lock().unwrap().remove(component_id);
        self.position_states.lock().unwrap().remove(component_id);
        self.press_states.lock().unwrap().remove(component_id);
        self.press_thresholds.lock().unwrap().remove(component_id);
        self.drag_and_drop_states
            .lock()
            .unwrap()
            .remove(component_id);
        self.wheel_states.lock().unwrap().remove(component_id);
        self.gesture_states.lock().unwrap().remove(component_id);
        let mut hovered = self.hovered_component.lock().unwrap();
        if hovered.as_deref() == Some(component_id) {
            *hovered = None;
        }
        let mut drag = self.drag_start.lock().unwrap();
        if drag
            .as_ref()
            .is_some_and(|active| active.owner == component_id)
        {
            *drag = None;
        }
        let mut press = self.press_start.lock().unwrap();
        if press
            .as_ref()
            .is_some_and(|active| active.owner == component_id)
        {
            *press = None;
        }
        let mut click = self.last_click.lock().unwrap();
        if click
            .as_ref()
            .is_some_and(|history| history.owner == component_id)
        {
            *click = None;
        }
        let mut gesture_owner = self.gesture_owner.lock().unwrap();
        if gesture_owner.as_deref() == Some(component_id) {
            *gesture_owner = None;
            self.gesture_points.lock().unwrap().clear();
        }
    }

    #[cfg(test)]
    pub(crate) fn registration_count(&self) -> usize {
        self.hover_states.lock().unwrap().len()
            + self.drag_states.lock().unwrap().len()
            + self.click_states.lock().unwrap().len()
            + self.position_states.lock().unwrap().len()
            + self.press_states.lock().unwrap().len()
            + self.drag_and_drop_states.lock().unwrap().len()
            + self.wheel_states.lock().unwrap().len()
            + self.gesture_states.lock().unwrap().len()
    }

    /// Process a mouse event and update all registered hook states
    pub fn process_event(&self, event: &MouseEvent, component_id: Option<&str>) {
        self.process_routed_event(event, component_id, component_id, None);
    }

    pub(crate) fn process_routed_event(
        &self,
        event: &MouseEvent,
        owner: Option<&str>,
        route_id: Option<&str>,
        local_position: Option<Position>,
    ) {
        let now = Instant::now();
        self.restart_changed_units(event.position, now);
        let mut local_event = event.clone();
        if let Some(position) = local_position {
            local_event.position = position;
        }

        match event.kind {
            MouseEventKind::Move => {
                self.handle_mouse_move(&local_event, owner);
                self.update_drag_if_active(event, route_id);
                self.track_gesture_point(local_event.position, owner, now);
            }

            MouseEventKind::Down => {
                self.handle_mouse_down(event, owner, route_id, now);
            }

            MouseEventKind::Up => {
                self.handle_mouse_up(event, owner, now);
            }

            MouseEventKind::Click => {
                self.handle_click(&local_event, owner, now);
            }

            MouseEventKind::DoubleClick => {
                self.handle_double_click(&local_event, owner);
            }

            MouseEventKind::TripleClick => {
                self.handle_triple_click(&local_event, owner);
            }

            MouseEventKind::Drag => {
                self.handle_drag(event, route_id);
            }

            MouseEventKind::Enter => {
                self.handle_enter(&local_event, owner, now);
            }

            MouseEventKind::Leave => {
                self.handle_leave(owner);
            }

            MouseEventKind::Wheel => {
                self.handle_wheel(event, owner);
            }
        }
    }

    fn restart_changed_units(&self, position: Position, now: Instant) {
        let changed = |previous| coordinate_delta(previous, position).is_none();
        let mut drag = self.drag_start.lock().unwrap();
        if drag.as_ref().is_some_and(|active| changed(active.position)) {
            if let Some(active) = drag.as_mut() {
                active.position = position;
                active.started = now;
            }
            for signal in self.drag_states.lock().unwrap().values() {
                signal.update(|state| {
                    if state.drag_start.is_some() {
                        state.is_dragging = false;
                        state.drag_start = Some(position);
                        state.current_position = Some(position);
                        state.drag_delta = (0, 0);
                    }
                });
            }
        }
        let mut press = self.press_start.lock().unwrap();
        if press
            .as_ref()
            .is_some_and(|active| changed(active.position))
        {
            if let Some(active) = press.as_mut() {
                active.position = position;
                active.started = now;
            }
            for signal in self.press_states.lock().unwrap().values() {
                signal.update(|state| {
                    if state.is_pressing {
                        state.press_start = Some(now);
                        state.duration = Duration::ZERO;
                        state.is_long_press = false;
                    }
                });
            }
        }
        let mut click = self.last_click.lock().unwrap();
        if click
            .as_ref()
            .is_some_and(|history| changed(history.position))
        {
            *click = None;
        }
        let gesture_owner = self.gesture_owner.lock().unwrap();
        let mut points = self.gesture_points.lock().unwrap();
        if points
            .last()
            .is_some_and(|(previous, _)| changed(*previous))
        {
            points.clear();
            if let Some(owner) = gesture_owner.as_deref() {
                if let Some(signal) = self.gesture_states.lock().unwrap().get(owner) {
                    signal.set(GestureState::default());
                }
            }
        }
    }

    fn handle_mouse_move(&self, event: &MouseEvent, component_id: Option<&str>) {
        // Update position states for components
        if let Some(id) = component_id {
            if let Some(signal) = self.position_states.lock().unwrap().get(id) {
                signal.set(MousePositionState {
                    position: Some(event.position),
                    client_position: Some(self.position_to_client(event.position)),
                    is_inside: true,
                });
            }
        }

        // Handle hover detection
        let mut hovered = self.hovered_component.lock().unwrap();
        if let Some(id) = component_id {
            let id_string = id.to_string();
            if *hovered != Some(id_string.clone()) {
                // Leave previous component
                if let Some(prev_id) = hovered.take() {
                    if let Some(signal) = self.hover_states.lock().unwrap().get(&prev_id) {
                        signal.set(HoverState {
                            is_hovered: false,
                            entered_at: None,
                            position: None,
                        });
                    }
                }

                // Enter new component
                if let Some(signal) = self.hover_states.lock().unwrap().get(&id_string) {
                    signal.set(HoverState {
                        is_hovered: true,
                        entered_at: Some(Instant::now()),
                        position: Some(event.position),
                    });
                }
                *hovered = Some(id_string);
            }
        } else if let Some(previous) = hovered.take() {
            if let Some(signal) = self.hover_states.lock().unwrap().get(&previous) {
                signal.set(HoverState::default());
            }
        }
    }

    fn handle_mouse_down(
        &self,
        event: &MouseEvent,
        owner: Option<&str>,
        route_id: Option<&str>,
        now: Instant,
    ) {
        let (Some(owner), Some(route_id)) = (owner, route_id) else {
            return;
        };
        // Start drag tracking
        *self.drag_start.lock().unwrap() = Some(ActiveDrag {
            owner: owner.to_owned(),
            route_id: route_id.to_owned(),
            position: event.position,
            started: now,
            button: event.button,
        });

        // Start press tracking
        *self.press_start.lock().unwrap() = Some(ActivePress {
            owner: owner.to_owned(),
            position: event.position,
            started: now,
        });

        // Update press states
        if let Some(signal) = self.press_states.lock().unwrap().get(owner) {
            signal.set(LongPressState {
                is_pressing: true,
                is_long_press: false,
                press_start: Some(now),
                duration: Duration::ZERO,
            });
        }
    }

    fn handle_mouse_up(&self, event: &MouseEvent, _component_id: Option<&str>, now: Instant) {
        // End drag
        if let Some(active) = self.drag_start.lock().unwrap().take() {
            if let Some(signal) = self.drag_states.lock().unwrap().get(&active.owner) {
                signal.update(|state| {
                    state.is_dragging = false;
                    state.drag_start = None;
                    state.current_position = Some(event.position);
                });
            }
            if let Some(registration) = self.drag_and_drop_states.lock().unwrap().get(&active.owner)
            {
                registration.signal.update(|state| {
                    state.drag.is_dragging = false;
                    state.drag.current_position = Some(event.position);
                    state.can_drop = false;
                });
            }
        }

        // End press
        if let Some(active) = self.press_start.lock().unwrap().take() {
            let duration = now - active.started;
            if let Some(signal) = self.press_states.lock().unwrap().get(&active.owner) {
                let threshold = self
                    .press_thresholds
                    .lock()
                    .unwrap()
                    .get(&active.owner)
                    .copied()
                    .unwrap_or(Duration::from_millis(800));
                signal.update(|state| {
                    state.is_pressing = false;
                    state.duration = duration;
                    state.is_long_press = duration >= threshold;
                });
            }
        }
    }

    fn handle_click(&self, event: &MouseEvent, component_id: Option<&str>, now: Instant) {
        let Some(component_id) = component_id else {
            return;
        };
        let mut last_click = self.last_click.lock().unwrap();

        // Check for double/triple click
        let click_count = if let Some(history) = &*last_click {
            if history.owner == component_id
                && now - history.occurred < Duration::from_millis(500)
                && self.positions_close(event.position, history.position)
            {
                history.count + 1
            } else {
                1
            }
        } else {
            1
        };

        *last_click = Some(ClickHistory {
            owner: component_id.to_owned(),
            position: event.position,
            occurred: now,
            count: click_count,
        });

        // Update click states
        if let Some(signal) = self.click_states.lock().unwrap().get(component_id) {
            signal.set(ClickState {
                click_count,
                last_click: Some(now),
                position: Some(event.position),
                is_double_click: click_count == 2,
                is_triple_click: click_count == 3,
            });
        }
    }

    fn handle_double_click(&self, event: &MouseEvent, component_id: Option<&str>) {
        if let Some(id) = component_id {
            if let Some(signal) = self.click_states.lock().unwrap().get(id) {
                signal.update(|state| {
                    state.click_count = 2;
                    state.is_double_click = true;
                    state.position = Some(event.position);
                });
            }
        }
    }

    fn handle_triple_click(&self, event: &MouseEvent, component_id: Option<&str>) {
        if let Some(id) = component_id {
            if let Some(signal) = self.click_states.lock().unwrap().get(id) {
                signal.update(|state| {
                    state.click_count = 3;
                    state.is_triple_click = true;
                    state.position = Some(event.position);
                });
            }
        }
    }

    fn handle_drag(&self, event: &MouseEvent, route_id: Option<&str>) {
        if let Some(active) = self.drag_start.lock().unwrap().clone() {
            let delta = self.calculate_delta(active.position, event.position);

            if let Some(signal) = self.drag_states.lock().unwrap().get(&active.owner) {
                signal.set(DragState {
                    is_dragging: true,
                    is_over_drop_zone: false,
                    drag_start: Some(active.position),
                    current_position: Some(event.position),
                    drag_delta: delta,
                    button: active.button,
                });
            }
            if let Some(registration) = self
                .drag_and_drop_states
                .lock()
                .unwrap()
                .get(&active.owner)
                .cloned()
            {
                let handle_matches = registration
                    .options
                    .drag_handle_selector
                    .as_deref()
                    .is_none_or(|handle| handle == active.route_id);
                let distance = coordinate_delta(active.position, event.position)
                    .map_or(0.0, |(dx, dy)| dx.hypot(dy));
                let dragging = handle_matches && distance >= registration.options.drag_threshold;
                let valid_target = route_id.filter(|target| {
                    registration
                        .options
                        .drop_zones
                        .iter()
                        .any(|zone| zone == *target)
                });
                let can_drop =
                    dragging && (valid_target.is_some() || registration.options.allow_drag_outside);
                registration.signal.set(DragAndDropState {
                    drag: DragState {
                        is_dragging: dragging,
                        is_over_drop_zone: valid_target.is_some(),
                        drag_start: Some(active.position),
                        current_position: Some(event.position),
                        drag_delta: delta,
                        button: active.button,
                    },
                    is_over_valid_drop: valid_target.is_some(),
                    drop_target: valid_target.map(str::to_owned),
                    can_drop,
                });
            }
        }
    }

    fn handle_enter(&self, event: &MouseEvent, component_id: Option<&str>, now: Instant) {
        if let Some(id) = component_id {
            // Update hover state
            if let Some(signal) = self.hover_states.lock().unwrap().get(id) {
                signal.set(HoverState {
                    is_hovered: true,
                    entered_at: Some(now),
                    position: Some(event.position),
                });
            }

            // Update position state
            if let Some(signal) = self.position_states.lock().unwrap().get(id) {
                signal.update(|state| {
                    state.is_inside = true;
                    state.position = Some(event.position);
                });
            }

            *self.hovered_component.lock().unwrap() = Some(id.to_string());
        }
    }

    fn handle_leave(&self, component_id: Option<&str>) {
        if let Some(id) = component_id {
            // Update hover state
            if let Some(signal) = self.hover_states.lock().unwrap().get(id) {
                signal.set(HoverState {
                    is_hovered: false,
                    entered_at: None,
                    position: None,
                });
            }

            // Update position state
            if let Some(signal) = self.position_states.lock().unwrap().get(id) {
                signal.update(|state| {
                    state.is_inside = false;
                });
            }

            let mut hovered = self.hovered_component.lock().unwrap();
            if *hovered == Some(id.to_string()) {
                *hovered = None;
            }
        }
    }

    fn handle_wheel(&self, event: &MouseEvent, component_id: Option<&str>) {
        if let Some(id) = component_id {
            if let Some(signal) = self.wheel_states.lock().unwrap().get(id) {
                let (delta_x, delta_y, delta_mode, is_scrolling) = match &event.wheel {
                    Some(wheel) => {
                        let (x, y, mode) = match wheel.delta {
                            crate::event::types::WheelDelta::Lines { x, y } => {
                                (f64::from(x), f64::from(y), WheelDeltaMode::Line)
                            }
                            crate::event::types::WheelDelta::Pixels { x, y } => {
                                (f64::from(x), f64::from(y), WheelDeltaMode::Pixel)
                            }
                        };
                        (
                            x,
                            y,
                            mode,
                            wheel.phase != crate::event::types::WheelPhase::Ended,
                        )
                    }
                    None => match event.button {
                        MouseButton::Other(4) => (0.0, -1.0, WheelDeltaMode::Line, true),
                        MouseButton::Other(5) => (0.0, 1.0, WheelDeltaMode::Line, true),
                        MouseButton::Other(6) => (-1.0, 0.0, WheelDeltaMode::Line, true),
                        MouseButton::Other(7) => (1.0, 0.0, WheelDeltaMode::Line, true),
                        _ => (0.0, 0.0, WheelDeltaMode::Line, false),
                    },
                };

                signal.set(WheelState {
                    delta_x,
                    delta_y,
                    delta_mode,
                    is_scrolling,
                });
            }
        }
    }

    fn update_drag_if_active(&self, event: &MouseEvent, route_id: Option<&str>) {
        if self.drag_start.lock().unwrap().is_some() {
            self.handle_drag(event, route_id);
        }
    }

    fn track_gesture_point(&self, position: Position, component_id: Option<&str>, now: Instant) {
        let Some(component_id) = component_id else {
            self.gesture_points.lock().unwrap().clear();
            *self.gesture_owner.lock().unwrap() = None;
            return;
        };
        let mut owner = self.gesture_owner.lock().unwrap();
        if owner.as_deref() != Some(component_id) {
            self.gesture_points.lock().unwrap().clear();
            *owner = Some(component_id.to_owned());
        }
        drop(owner);
        let mut points = self.gesture_points.lock().unwrap();
        points.push((position, now));

        // Keep only recent points (last 500ms)
        points.retain(|(_, time)| now - *time < Duration::from_millis(500));

        // Detect gestures if we have enough points
        if points.len() >= 3 {
            if let Some(gesture) = self.detect_gesture(&points) {
                if let Some(signal) = self.gesture_states.lock().unwrap().get(component_id) {
                    signal.set(GestureState {
                        gesture_type: gesture,
                        is_active: true,
                        start_position: points.first().map(|(p, _)| *p),
                        end_position: points.last().map(|(p, _)| *p),
                        velocity: self.calculate_velocity(&points),
                    });
                }
            }
        }
    }

    fn detect_gesture(&self, points: &[(Position, Instant)]) -> Option<GestureType> {
        if points.len() < 3 {
            return None;
        }

        let first = points.first().unwrap().0;
        let last = points.last().unwrap().0;

        let (dx, dy) = coordinate_delta(first, last)?;

        // Simple swipe detection based on direction and distance
        let distance = dx.hypot(dy);
        if distance < 5.0 {
            return None;
        }

        if dx.abs() > dy.abs() {
            // Horizontal swipe
            if dx > 0.0 {
                Some(GestureType::Swipe(SwipeDirection::Right))
            } else {
                Some(GestureType::Swipe(SwipeDirection::Left))
            }
        } else {
            // Vertical swipe
            if dy > 0.0 {
                Some(GestureType::Swipe(SwipeDirection::Down))
            } else {
                Some(GestureType::Swipe(SwipeDirection::Up))
            }
        }
    }

    fn calculate_velocity(&self, points: &[(Position, Instant)]) -> (f64, f64) {
        let vel = self.calculate_velocity_vec(points);
        (vel.dx, vel.dy)
    }

    fn calculate_velocity_vec(&self, points: &[(Position, Instant)]) -> Velocity {
        if points.len() < 2 {
            return Velocity::default();
        }

        let (p1, t1) = points[points.len() - 2];
        let (p2, t2) = points[points.len() - 1];

        let dt = (t2 - t1).as_secs_f64();
        if dt == 0.0 {
            return Velocity::default();
        }

        let Some((dx, dy)) = coordinate_delta(p1, p2) else {
            return Velocity::default();
        };

        Velocity::new(dx / dt, dy / dt)
    }

    fn position_to_client(&self, pos: Position) -> (f64, f64) {
        (pos.x() as f64, pos.y() as f64)
    }

    fn positions_close(&self, p1: Position, p2: Position) -> bool {
        coordinate_delta(p1, p2).is_some_and(|(dx, dy)| dx.abs() <= 3.0 && dy.abs() <= 3.0)
    }

    fn calculate_delta(&self, start: Position, current: Position) -> (i32, i32) {
        // Float-to-integer casts saturate at the public i32 field's limits.
        coordinate_delta(start, current).map_or((0, 0), |(dx, dy)| (dx as i32, dy as i32))
    }
}
