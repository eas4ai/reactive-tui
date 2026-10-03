use super::{platform::unix::Adapter, Snapshot};
use crate::{
    app::AppWaker,
    error::{ReactiveError, Result},
};
use accesskit::{ActionHandler, ActionRequest, ActivationHandler, DeactivationHandler, TreeUpdate};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver, SyncSender, TrySendError},
    Arc, Mutex,
};

const ACTION_CAPACITY: usize = 64;

struct Activation(Arc<Mutex<TreeUpdate>>);
impl ActivationHandler for Activation {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        Some(self.0.lock().unwrap().clone())
    }
}

struct Actions {
    sender: SyncSender<ActionRequest>,
    overflow: Arc<AtomicBool>,
    wake: AppWaker,
}
impl ActionHandler for Actions {
    fn do_action(&mut self, request: ActionRequest) {
        match self.sender.try_send(request) {
            Ok(()) => self.wake.request_redraw(),
            Err(TrySendError::Full(_)) => {
                self.overflow.store(true, Ordering::Release);
                self.wake.request_redraw();
            }
            Err(TrySendError::Disconnected(_)) => {}
        }
    }
}

struct Deactivation;
impl DeactivationHandler for Deactivation {
    fn deactivate_accessibility(&mut self) {}
}

pub(crate) struct Connection {
    adapter: Adapter,
    latest: Arc<Mutex<TreeUpdate>>,
    receiver: Mutex<Receiver<ActionRequest>>,
    overflow: Arc<AtomicBool>,
}

impl Connection {
    pub(crate) fn new(name: &str, wake: AppWaker) -> Result<Self> {
        let latest = Arc::new(Mutex::new(Snapshot::empty(name).update));
        let (sender, receiver) = mpsc::sync_channel(ACTION_CAPACITY);
        let overflow = Arc::new(AtomicBool::new(false));
        let adapter = Adapter::new(
            Activation(latest.clone()),
            Actions {
                sender,
                overflow: overflow.clone(),
                wake: wake.clone(),
            },
            Deactivation,
            wake,
        )?;
        Ok(Self {
            adapter,
            latest,
            receiver: Mutex::new(receiver),
            overflow,
        })
    }

    pub(crate) fn publish(&mut self, snapshot: &Snapshot) -> Result<()> {
        self.adapter.check().map_err(ReactiveError::invalid_state)?;
        *self.latest.lock().unwrap() = snapshot.update.clone();
        self.adapter.update_if_active(|| snapshot.update.clone());
        self.adapter.check().map_err(ReactiveError::invalid_state)
    }

    pub(crate) fn focus(&mut self, focused: bool) -> Result<()> {
        self.adapter.update_window_focus_state(focused);
        self.adapter.check().map_err(ReactiveError::invalid_state)
    }

    pub(crate) fn close(&mut self) -> Result<()> {
        self.adapter.close().map_err(ReactiveError::invalid_state)
    }

    pub(crate) fn actions(&self) -> Result<Vec<ActionRequest>> {
        self.adapter.check().map_err(ReactiveError::invalid_state)?;
        if self.overflow.swap(false, Ordering::AcqRel) {
            return Err(ReactiveError::invalid_state(
                "screen-reader action queue exceeded its 64-request bound",
            ));
        }
        Ok(self
            .receiver
            .lock()
            .unwrap()
            .try_iter()
            .take(ACTION_CAPACITY)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn assistive_action_overload_is_bounded_and_wakes_app() {
        let (sender, receiver) = mpsc::sync_channel(ACTION_CAPACITY);
        let overflow = Arc::new(AtomicBool::new(false));
        let wake = AppWaker::new();
        let mut actions = Actions {
            sender,
            overflow: overflow.clone(),
            wake: wake.clone(),
        };
        for id in 0..=ACTION_CAPACITY {
            actions.do_action(ActionRequest {
                action: accesskit::Action::Focus,
                target_tree: accesskit::TreeId::ROOT,
                target_node: accesskit::NodeId(id as u64),
                data: None,
            });
        }
        assert!(wake.is_pending());
        assert!(overflow.load(Ordering::Acquire));
        assert_eq!(receiver.try_iter().count(), ACTION_CAPACITY);
    }
}
