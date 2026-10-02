//! App-owned snapshots and a bounded, weakly addressed mode request.
use crate::display::{adaptive::AdaptiveFpsManager, monitor::PerformanceMode};
use crate::hooks::{
    fps::{FpsState, FrameTiming},
    perf_context::PerformanceContext,
};
use crate::reactive::{hooks::ThreadSafeSignal, wake::AppWaker};
use std::sync::{Arc, Mutex};
use std::time::Duration;

struct Requests {
    pending: Option<PerformanceMode>,
    closed: bool,
}

pub(super) struct Owner {
    requests: Arc<Mutex<Requests>>,
    context: Arc<PerformanceContext>,
    wake: AppWaker,
}

impl Owner {
    pub(super) fn new(manager: &AdaptiveFpsManager, wake: AppWaker) -> Self {
        let requests = Arc::new(Mutex::new(Requests {
            pending: None,
            closed: false,
        }));
        let weak = Arc::downgrade(&requests);
        let notify = wake.clone();
        let context = Arc::new(PerformanceContext {
            fps_state: ThreadSafeSignal::new(FpsState::default()),
            metrics: ThreadSafeSignal::new(manager.get_performance_metrics()),
            frame_timing: ThreadSafeSignal::new(FrameTiming::default()),
            set_mode: Arc::new(move |mode| {
                let Some(requests) = weak.upgrade() else {
                    return;
                };
                let mut requests = requests.lock().unwrap();
                if requests.closed {
                    return;
                }
                requests.pending = Some(mode);
                drop(requests);
                notify.wake();
            }),
        });
        let owner = Self {
            requests,
            context,
            wake,
        };
        owner.publish(manager, Duration::ZERO);
        owner
    }

    pub(super) fn context(&self) -> Arc<PerformanceContext> {
        self.context.clone()
    }

    pub(super) fn take_request(&self) -> Option<PerformanceMode> {
        self.requests.lock().unwrap().pending.take()
    }

    pub(super) fn close(&self) {
        let mut requests = self.requests.lock().unwrap();
        requests.closed = true;
        requests.pending = None;
    }

    pub(super) fn publish(&self, manager: &AdaptiveFpsManager, last_render: Duration) {
        let metrics = manager.get_performance_metrics();
        self.context.fps_state.set_except(
            FpsState {
                target_fps: manager.get_target_fps(),
                current_fps: metrics.current_fps,
                avg_render_time_ms: metrics.avg_render_time_ms,
                drop_rate_percent: metrics.drop_rate_percent,
                is_stable: metrics.is_stable,
                mode: manager.performance_mode(),
            },
            Some(&self.wake),
        );
        self.context.metrics.set_except(metrics, Some(&self.wake));
        let target = manager.get_frame_duration().as_secs_f32() * 1000.0;
        let last = last_render.as_secs_f32() * 1000.0;
        self.context.frame_timing.set_except(
            FrameTiming {
                last_frame_ms: last,
                target_frame_ms: target,
                budget_remaining_ms: target - last,
            },
            Some(&self.wake),
        );
    }
}
