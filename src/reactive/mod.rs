/// Effect system for side effects and cleanup
pub mod effect;
/// React-like hooks for state management
pub mod hooks;
/// Reactive runtime and context management
pub mod runtime;
/// Reactive scheduler for batching updates
pub mod scheduler;
/// Signal system for reactive state
pub mod signal;

pub use effect::{Cleanup, Effect, EffectId};
pub use hooks::{
    provide_context, use_context, use_effect, use_memo, use_previous, use_reducer, use_signal,
    Hooks, ThreadSafeSignal,
};
pub use runtime::{ReactiveRuntime, RuntimeContext};
pub use scheduler::{Scheduler, TimerId};
pub use signal::{ReadSignal, Signal, SignalId, WriteSignal};
