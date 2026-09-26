//! Owned dialog sessions presented by the normal App component runtime.

use super::*;
use crate::reactive::scheduler::{Scheduler, TimerId};
use crate::{component::Element, reactive::ThreadSafeSignal};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

mod content;
mod live;
mod mailbox;
mod motion;
use content::Content;
pub(super) use live::Presentation;
use mailbox::Mailbox;
pub use motion::DialogAnimationFrame;
type AnimationCallback = Arc<dyn Fn(f32) -> DialogAnimationFrame + Send + Sync>;

const EVENT_CAPACITY: usize = 1024;
type CloseCallback = Arc<dyn Fn(DialogResult) + Send + Sync>;

/// A dialog operation could not be accepted. No dialog is opened on error.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DialogEngineError {
    /// The maximum number of active dialogs has been reached.
    #[error("the maximum number of active dialogs has been reached")]
    LimitReached,
    /// Drain dialog events before opening another dialog.
    #[error("drain dialog events before opening another dialog")]
    EventQueueFull,
    /// Dialog identifiers are exhausted.
    #[error("dialog identifiers are exhausted")]
    IdExhausted,
    /// The configured dialog z-index range is exhausted.
    #[error("the configured dialog z-index range is exhausted")]
    LayerExhausted,
    /// Dialog is not active.
    #[error("dialog is not active")]
    NotFound,
    /// This update does not apply to this dialog type.
    #[error("this update does not apply to this dialog type")]
    WrongType,
    /// Progress must be finite.
    #[error("progress must be finite")]
    InvalidProgress,
    /// Enable_async must be called before requesting an async receiver.
    #[error("enable_async must be called before requesting an async receiver")]
    AsyncDisabled,
    /// This dialog completion receiver has already been taken.
    #[error("this dialog completion receiver has already been taken")]
    CompletionTaken,
    /// The asynchronous event receiver has already been taken.
    #[error("the asynchronous event receiver has already been taken")]
    EventsTaken,
    /// The dialog host is shutting down.
    #[error("the dialog host is shutting down")]
    ShuttingDown,
    /// A custom animation name has not been registered with this engine.
    #[error("unknown dialog animation: {0}")]
    UnknownAnimation(String),
    /// A custom animation must have a nonempty name.
    #[error("dialog animation names must be nonempty")]
    InvalidAnimationName,
    /// The animation duration exceeds the monotonic clock's range.
    #[error("dialog animation duration exceeds the supported clock range")]
    InvalidDuration,
}

/// Changes to an open dialog. User edits survive unrelated redraws.
#[derive(Debug, Clone)]
pub enum DialogUpdate {
    /// Set finite progress; values outside 0..=1 are clamped.
    Progress(f32),
    /// Replace an input dialog's value.
    InputValue(String),
    /// Replace the data supplied to wizard steps and completion callbacks.
    WizardData(HashMap<String, String>),
    /// Set relative stacking priority. Equal priorities retain opening order.
    ZIndex(u16),
}

/// One result from an accepted dialog. Obtain this before closing the dialog.
/// Dropping the receiver does not cancel the dialog or lose its Closed event.
#[derive(Debug)]
pub struct DialogCompletion(async_channel::Receiver<DialogResult>);
impl DialogCompletion {
    /// Wait without blocking an executor thread. Works without Tokio.
    pub async fn result(self) -> DialogResult {
        self.0.recv().await.unwrap_or(DialogResult::Cancelled)
    }
    /// Consume a result if completion has already occurred.
    pub fn try_result(&self) -> Option<DialogResult> {
        self.0.try_recv().ok()
    }
}

/// Async access to the same event queue consumed by `DialogEngine::take_event`.
/// Only one async receiver can be taken. Synchronous reads share its queue.
#[derive(Debug)]
pub struct DialogEvents(Arc<Mailbox>);
impl DialogEvents {
    /// Returns None after the engine is dropped and buffered events are drained.
    pub async fn next(&mut self) -> Option<DialogEvent> {
        let _waiting = self.0.waiting();
        std::future::poll_fn(|cx| self.0.poll_next(cx)).await
    }
    /// Consume the next queued event, if any.
    pub fn try_next(&self) -> Option<DialogEvent> {
        self.0.take()
    }
}

struct Entry {
    content: Content,
    priority: u16,
    completion: async_channel::Sender<DialogResult>,
    receiver: Option<async_channel::Receiver<DialogResult>>,
    on_close: Option<CloseCallback>,
    activity: Activity,
}

struct Retiring {
    content: Content,
    priority: u16,
    activity: Activity,
    timer: Option<TimerId>,
}

#[derive(Clone)]
pub(super) struct Activity(Arc<AtomicBool>);
impl Default for Activity {
    fn default() -> Self {
        Self(Arc::new(AtomicBool::new(true)))
    }
}
impl PartialEq for Activity {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Activity {
    pub(super) fn active(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

struct State {
    dialogs: HashMap<DialogId, Entry>,
    retiring: HashMap<DialogId, Retiring>,
    order: Vec<DialogId>,
    next_id: Option<u32>,
    config: DialogEngineConfig,
    async_enabled: bool,
    events_taken: bool,
    last_error: Option<DialogEngineError>,
    focus: DialogFocusManager,
    mounted: bool,
    shutting_down: bool,
    animations: HashMap<String, AnimationCallback>,
    scheduler: Option<std::sync::Weak<Scheduler>>,
}

struct Core {
    state: Mutex<State>,
    changed: ThreadSafeSignal<u64>,
    events: Arc<Mailbox>,
}
impl Core {
    fn wake(&self) {
        self.changed.update(|value| *value = value.wrapping_add(1));
    }
    fn close(&self, id: DialogId, result: DialogResult) -> bool {
        self.finish(id, result, None)
    }
    fn finish(
        &self,
        id: DialogId,
        result: DialogResult,
        owner: Option<std::sync::Weak<Self>>,
    ) -> bool {
        let mut transition = None;
        let entry = {
            let mut state = self.state.lock().unwrap();
            let Some(entry) = state.dialogs.remove(&id) else {
                return false;
            };
            entry.activity.0.store(false, Ordering::Release);
            if !state.shutting_down
                && owner.is_some()
                && state.config.default_theme.animation != DialogAnimation::None
                && !state.config.animation_duration.is_zero()
            {
                if let Some(scheduler) = state.scheduler.as_ref().and_then(std::sync::Weak::upgrade)
                {
                    transition = Some((scheduler, state.config.animation_duration));
                    state.retiring.insert(
                        id,
                        Retiring {
                            content: entry.content.clone(),
                            priority: entry.priority,
                            activity: entry.activity.clone(),
                            timer: None,
                        },
                    );
                }
            }
            state.order.retain(|current| *current != id);
            state.focus.dialog_closed(id);
            // Opening reserved one slot for this event, even if no consumer runs.
            self.events.push(DialogEvent::Closed(id, result.clone()));
            entry
        };
        if let (Some((scheduler, duration)), Some(owner)) = (transition, owner) {
            let timer = scheduler.schedule_timeout(duration, move || {
                if let Some(core) = owner.upgrade() {
                    let retired = core.state.lock().unwrap().retiring.remove(&id);
                    drop(retired);
                    core.wake();
                }
            });
            let retained = {
                let mut state = self.state.lock().unwrap();
                if let Some(retiring) = state.retiring.get_mut(&id) {
                    retiring.timer = Some(timer);
                    true
                } else {
                    false
                }
            };
            if !retained {
                scheduler.cancel_timer(timer);
            }
        }
        // No engine lock is held while waking consumers or invoking user code.
        let _ = entry.completion.try_send(result.clone());
        self.events.wake();
        self.wake();
        if let Some(callback) = entry.on_close {
            callback(result);
        }
        true
    }
    fn cancel_all(&self) {
        let (entries, retiring, scheduler) = {
            let mut state = self.state.lock().unwrap();
            if state.shutting_down {
                return;
            }
            state.shutting_down = true;
            let order = std::mem::take(&mut state.order);
            let entries: Vec<_> = order
                .into_iter()
                .rev()
                .filter_map(|id| {
                    let entry = state.dialogs.remove(&id)?;
                    entry.activity.0.store(false, Ordering::Release);
                    self.events
                        .push(DialogEvent::Closed(id, DialogResult::Cancelled));
                    Some(entry)
                })
                .collect();
            state.focus = DialogFocusManager::new();
            (
                entries,
                std::mem::take(&mut state.retiring),
                state.scheduler.as_ref().and_then(std::sync::Weak::upgrade),
            )
        };
        if let Some(scheduler) = scheduler {
            for entry in retiring.values() {
                if let Some(timer) = entry.timer {
                    scheduler.cancel_timer(timer);
                }
            }
        }
        drop(retiring);
        let mut callbacks = Vec::new();
        for entry in entries {
            let _ = entry.completion.try_send(DialogResult::Cancelled);
            if let Some(callback) = entry.on_close {
                callbacks.push(callback);
            }
        }
        self.events.wake();
        self.wake();
        let mut panic = None;
        for callback in callbacks {
            if let Err(payload) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                callback(DialogResult::Cancelled)
            })) {
                if panic.is_none() {
                    panic = Some(payload);
                }
            }
        }
        self.state.lock().unwrap().shutting_down = false;
        if let Some(payload) = panic {
            if !std::thread::panicking() {
                std::panic::resume_unwind(payload);
            }
        }
    }
}
impl Drop for Core {
    fn drop(&mut self) {
        // Resolve handles even when a never-mounted engine is abandoned.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.cancel_all()));
        self.events.close();
        if let Err(payload) = result {
            if !std::thread::panicking() {
                std::panic::resume_unwind(payload);
            }
        }
    }
}

/// Shared dialog controller. Clones address the same sessions. Render one host
/// per engine with `render`, either inside a root or as the App root itself.
#[derive(Clone)]
pub struct DialogEngine {
    core: Arc<Core>,
}

/// A controller reference that does not keep its engine alive. Use this in
/// callbacks stored by that engine to avoid a strong reference cycle.
#[derive(Clone)]
pub struct WeakDialogEngine(std::sync::Weak<Core>);
impl WeakDialogEngine {
    /// Obtain a controller while an owner or mounted App still holds the engine.
    pub fn upgrade(&self) -> Option<DialogEngine> {
        self.0.upgrade().map(|core| DialogEngine { core })
    }
}
impl std::fmt::Debug for DialogEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DialogEngine")
            .field("active_dialogs", &self.active_count())
            .finish()
    }
}
impl Default for DialogEngine {
    fn default() -> Self {
        Self::new()
    }
}
impl DialogEngine {
    /// Create a non-owning reference for callbacks stored by this engine.
    pub fn downgrade(&self) -> WeakDialogEngine {
        WeakDialogEngine(Arc::downgrade(&self.core))
    }
    /// Create an engine with the default limits and theme.
    pub fn new() -> Self {
        Self::with_config(DialogEngineConfig::default())
    }
    /// Create an engine with explicit configuration.
    pub fn with_config(config: DialogEngineConfig) -> Self {
        Self {
            core: Arc::new(Core {
                state: Mutex::new(State {
                    dialogs: HashMap::new(),
                    retiring: HashMap::new(),
                    order: Vec::new(),
                    next_id: Some(1),
                    config,
                    async_enabled: false,
                    events_taken: false,
                    last_error: None,
                    focus: DialogFocusManager::new(),
                    mounted: false,
                    shutting_down: false,
                    animations: HashMap::new(),
                    scheduler: None,
                }),
                changed: ThreadSafeSignal::new(0),
                events: Arc::new(Mailbox::default()),
            }),
        }
    }
    /// Enable nonblocking result and event receivers. Synchronous events are
    /// always retained, including those emitted before this call.
    pub fn enable_async(&mut self) {
        self.core.state.lock().unwrap().async_enabled = true;
    }
    /// Take the single asynchronous event receiver for this engine.
    pub fn events(&self) -> Result<DialogEvents, DialogEngineError> {
        let mut state = self.core.state.lock().unwrap();
        if !state.async_enabled {
            return Err(DialogEngineError::AsyncDisabled);
        }
        if state.events_taken {
            return Err(DialogEngineError::EventsTaken);
        }
        state.events_taken = true;
        Ok(DialogEvents(self.core.events.clone()))
    }
    /// Take the single completion receiver for an active dialog.
    pub fn completion(&mut self, id: DialogId) -> Result<DialogCompletion, DialogEngineError> {
        let mut state = self.core.state.lock().unwrap();
        if !state.async_enabled {
            return Err(DialogEngineError::AsyncDisabled);
        }
        state
            .dialogs
            .get_mut(&id)
            .ok_or(DialogEngineError::NotFound)?
            .receiver
            .take()
            .map(DialogCompletion)
            .ok_or(DialogEngineError::CompletionTaken)
    }
    /// Consume the next Opened or Closed event synchronously.
    pub fn take_event(&mut self) -> Option<DialogEvent> {
        self.core.events.take()
    }
    /// Number of currently open sessions.
    pub fn active_count(&self) -> usize {
        self.core.state.lock().unwrap().dialogs.len()
    }
    /// Whether this ID identifies an active session.
    pub fn is_open(&self, id: DialogId) -> bool {
        self.core.state.lock().unwrap().dialogs.contains_key(&id)
    }
    /// Supply the frame function for `DialogAnimation::Custom(name)`.
    /// Progress is 0..=1. Callbacks run without engine locks and must return
    /// finite values; invalid output closes the session with an Error result.
    pub fn register_animation(
        &mut self,
        name: impl Into<String>,
        callback: impl Fn(f32) -> DialogAnimationFrame + Send + Sync + 'static,
    ) -> Result<(), DialogEngineError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(DialogEngineError::InvalidAnimationName);
        }
        let old = self
            .core
            .state
            .lock()
            .unwrap()
            .animations
            .insert(name, Arc::new(callback));
        drop(old);
        self.core.wake();
        Ok(())
    }
    /// The most recent failed legacy show call; a successful show clears it.
    pub fn last_error(&self) -> Option<DialogEngineError> {
        self.core.state.lock().unwrap().last_error.clone()
    }
    /// Close once. Unknown or already-closed IDs have no effect.
    pub fn close_dialog(&mut self, id: DialogId, result: DialogResult) {
        self.core.close(id, result);
    }
    /// Cancel current dialogs. Close callbacks cannot open replacements while
    /// cancellation is in progress.
    pub fn close_all(&mut self) {
        self.core.cancel_all();
    }
    /// Produce a keyed App host. The normal App router owns input and focus;
    /// there is no parallel engine hit-test tree with guessed bounds.
    pub fn render(&self) -> Element {
        Element::typed::<live::Host>(live::HostProps(self.clone()))
    }
    /// Update an active session and wake its App. This never resets another
    /// dialog or recreates a retained control.
    pub fn update(&mut self, id: DialogId, update: DialogUpdate) -> Result<(), DialogEngineError> {
        {
            let mut state = self.core.state.lock().unwrap();
            let entry = state
                .dialogs
                .get_mut(&id)
                .ok_or(DialogEngineError::NotFound)?;
            match update {
                DialogUpdate::ZIndex(priority) => entry.priority = priority,
                update => entry.content.update(update)?,
            }
        }
        self.core.wake();
        Ok(())
    }
    fn open(&mut self, mut content: Content) -> Result<DialogId, DialogEngineError> {
        let id = {
            let mut state = self.core.state.lock().unwrap();
            if state.shutting_down {
                return Err(DialogEngineError::ShuttingDown);
            }
            if std::time::Instant::now()
                .checked_add(state.config.animation_duration)
                .is_none()
            {
                return Err(DialogEngineError::InvalidDuration);
            }
            if let DialogAnimation::Custom(name) = &state.config.default_theme.animation {
                if !state.animations.contains_key(name) {
                    return Err(DialogEngineError::UnknownAnimation(name.clone()));
                }
            }
            let occupied = state.dialogs.len() + state.retiring.len();
            if occupied >= state.config.max_dialogs {
                return Err(DialogEngineError::LimitReached);
            }
            if self.core.events.len() + state.dialogs.len() + 2 > EVENT_CAPACITY {
                return Err(DialogEngineError::EventQueueFull);
            }
            if usize::from(state.config.base_z_index) + occupied * 2 + 1 > usize::from(u16::MAX) {
                return Err(DialogEngineError::LayerExhausted);
            }
            let id = DialogId(state.next_id.ok_or(DialogEngineError::IdExhausted)?);
            state.next_id = id.0.checked_add(1);
            let on_close = content.connect(Arc::downgrade(&self.core), id);
            let priority = content.priority();
            let (completion, receiver) = async_channel::bounded(1);
            state.dialogs.insert(
                id,
                Entry {
                    content,
                    priority,
                    completion,
                    receiver: Some(receiver),
                    on_close,
                    activity: Activity::default(),
                },
            );
            state.order.push(id);
            state.focus.dialog_opened(id);
            self.core.events.push(DialogEvent::Opened(id));
            state.last_error = None;
            id
        };
        self.core.events.wake();
        self.core.wake();
        Ok(id)
    }
    fn legacy(&mut self, result: Result<DialogId, DialogEngineError>) -> DialogId {
        match result {
            Ok(id) => id,
            Err(error) => {
                self.core.state.lock().unwrap().last_error = Some(error);
                DialogId::INVALID
            }
        }
    }
}

impl DialogId {
    /// Returned by legacy show methods when a dialog cannot be opened.
    pub const INVALID: Self = Self(0);
}

macro_rules! show {
    ($show:ident, $try_show:ident, $options:ty, $variant:ident) => {
        impl DialogEngine {
            /// Open a dialog, or return `DialogId::INVALID` and set `last_error`.
            pub fn $show(&mut self, options: $options) -> DialogId {
                let result = self.$try_show(options);
                self.legacy(result)
            }
            /// Open a dialog without silently accepting a failed request.
            pub fn $try_show(&mut self, options: $options) -> Result<DialogId, DialogEngineError> {
                self.open(Content::$variant(options.into()))
            }
        }
    };
}
show!(
    show_confirmation,
    try_show_confirmation,
    ConfirmationDialogOptions,
    Confirmation
);
show!(show_input, try_show_input, InputDialogOptions, Input);
show!(
    show_autocomplete,
    try_show_autocomplete,
    AutocompleteDialogOptions,
    Autocomplete
);
show!(
    show_progress,
    try_show_progress,
    ProgressDialogOptions,
    Progress
);
show!(show_toast, try_show_toast, ToastOptions, Toast);
show!(show_wizard, try_show_wizard, WizardDialogOptions, Wizard);

impl crate::app::RootComponent for DialogEngine {
    fn render(&self) -> Element {
        DialogEngine::render(self)
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn closing_transition_releases_retained_content_and_its_deadline() {
        let mut engine = DialogEngine::with_config(DialogEngineConfig {
            animation_duration: Duration::from_millis(2),
            ..Default::default()
        });
        let scheduler = Arc::new(Scheduler::new());
        engine.core.state.lock().unwrap().scheduler = Some(Arc::downgrade(&scheduler));
        let id = engine.show_progress(Default::default());
        engine.core.finish(
            id,
            DialogResult::Confirmed(None),
            Some(Arc::downgrade(&engine.core)),
        );
        assert_eq!(engine.active_count(), 0);
        assert_eq!(engine.core.state.lock().unwrap().retiring.len(), 1);
        assert!(scheduler.next_deadline().is_some());
        // A hang guard, not a timing check: generous so a busy machine
        // cannot fail a correct test by running it slowly.
        let end = Instant::now() + Duration::from_secs(30);
        while !engine.core.state.lock().unwrap().retiring.is_empty() && Instant::now() < end {
            scheduler.run_ready_timers();
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(engine.core.state.lock().unwrap().retiring.is_empty());
        assert!(scheduler.next_deadline().is_none());
    }

    #[test]
    fn shutdown_cancels_retired_dialog_deadlines_without_waiting() {
        let mut engine = DialogEngine::with_config(DialogEngineConfig {
            animation_duration: Duration::from_secs(60),
            ..Default::default()
        });
        let scheduler = Arc::new(Scheduler::new());
        engine.core.state.lock().unwrap().scheduler = Some(Arc::downgrade(&scheduler));
        let id = engine.show_progress(Default::default());
        engine.core.finish(
            id,
            DialogResult::Confirmed(None),
            Some(Arc::downgrade(&engine.core)),
        );
        assert!(scheduler.next_deadline().is_some());
        engine.close_all();
        assert!(scheduler.next_deadline().is_none());
        assert!(engine.core.state.lock().unwrap().retiring.is_empty());
    }

    #[test]
    fn exhausted_identifiers_never_wrap_to_zero_or_an_existing_session() {
        let mut engine = DialogEngine::new();
        engine.core.state.lock().unwrap().next_id = Some(u32::MAX);
        assert_eq!(
            engine
                .try_show_progress(Default::default())
                .unwrap()
                .as_u32(),
            u32::MAX
        );
        assert_eq!(
            engine.try_show_progress(Default::default()),
            Err(DialogEngineError::IdExhausted)
        );
        assert_eq!(engine.active_count(), 1);
    }
}
