//! An in-process screen reader: the App publishes its accessibility tree
//! to a [`ReaderChannel`] and takes the channel's requests as it takes the
//! AT-SPI connection's.

use crate::app::AppWaker;
use accesskit::{ActionRequest, TreeUpdate};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

/// The most requests that wait in a channel, as on the AT-SPI connection.
const CAPACITY: usize = 64;

#[derive(Default)]
struct Shared {
    tree: Option<TreeUpdate>,
    requests: VecDeque<ActionRequest>,
    wake: Option<AppWaker>,
}

/// An in-process stand-in for the screen reader. An App built with
/// [`crate::app::AppBuilder::screen_reader_channel`] publishes each frame's
/// accessibility tree to it, as it publishes the tree to AT-SPI, and takes
/// the requests queued on it on its next turn, as it takes the screen
/// reader's (CTL-005). Tests use it to read what a screen reader hears and
/// to act as one acts. Clones share one tree and one queue.
#[derive(Clone, Default)]
pub struct ReaderChannel(Arc<Mutex<Shared>>);

impl ReaderChannel {
    /// A channel that no App has published to yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// The tree the App published last, every node with its role, label,
    /// value and the actions it advertises; `None` before its first frame.
    pub fn tree(&self) -> Option<TreeUpdate> {
        self.0.lock().unwrap().tree.clone()
    }

    /// Queue `request` for the App, as a screen reader's action on the node
    /// it names, and wake the App to take it. Returns `false`, dropping the
    /// request, when 64 requests already wait.
    pub fn request(&self, request: ActionRequest) -> bool {
        let mut shared = self.0.lock().unwrap();
        if shared.requests.len() >= CAPACITY {
            return false;
        }
        shared.requests.push_back(request);
        if let Some(wake) = &shared.wake {
            wake.request_redraw();
        }
        true
    }

    /// Wake `wake`'s App when a request arrives.
    pub(crate) fn attach(&self, wake: AppWaker) {
        self.0.lock().unwrap().wake = Some(wake);
    }

    /// Keep `tree` as the tree the App published last.
    pub(crate) fn publish(&self, tree: &TreeUpdate) {
        self.0.lock().unwrap().tree = Some(tree.clone());
    }

    /// The requests queued since the last take, in order.
    pub(crate) fn take(&self) -> Vec<ActionRequest> {
        self.0.lock().unwrap().requests.drain(..).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_wait_in_order_up_to_the_bound_and_wake_the_app() {
        let channel = ReaderChannel::new();
        let wake = AppWaker::new();
        channel.attach(wake.clone());
        let request = |id| ActionRequest {
            action: accesskit::Action::Focus,
            target_tree: accesskit::TreeId::ROOT,
            target_node: accesskit::NodeId(id),
            data: None,
        };
        for id in 0..CAPACITY as u64 {
            assert!(channel.request(request(id)));
        }
        assert!(!channel.request(request(99)), "the 65th request is dropped");
        assert!(wake.is_pending());
        let taken = channel.take();
        assert_eq!(taken.len(), CAPACITY);
        assert_eq!(taken[3].target_node, accesskit::NodeId(3));
        assert!(channel.take().is_empty(), "a take empties the queue");
    }
}
