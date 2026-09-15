#![cfg(feature = "tokio")]

use reactive_tui::platform::{EventLoop, TokioEventLoop};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::Duration;

async fn wait_until(event_loop: &TokioEventLoop, running: bool) {
    tokio::time::timeout(Duration::from_secs(2), async {
        while event_loop.is_running() != running {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("TokioEventLoop did not reach the expected running state");
}

async fn exercise_lifecycle(runtime: &str) {
    let mut sync_started = TokioEventLoop::new();
    let start = catch_unwind(AssertUnwindSafe(|| EventLoop::start(&mut sync_started)))
        .expect("sync start panicked inside an active Tokio runtime");
    if start.is_ok() {
        wait_until(&sync_started, true).await;
        tokio::time::timeout(Duration::from_secs(2), sync_started.stop_async())
            .await
            .expect("async cleanup after sync start timed out")
            .expect("async cleanup after sync start failed");
    }

    let mut sync_stopped = TokioEventLoop::new();
    sync_stopped
        .start_async()
        .await
        .expect("async start failed");
    wait_until(&sync_stopped, true).await;
    catch_unwind(AssertUnwindSafe(|| EventLoop::stop(&mut sync_stopped)))
        .expect("sync stop panicked inside an active Tokio runtime")
        .expect("sync stop failed");
    wait_until(&sync_stopped, false).await;

    let mut async_lifecycle = TokioEventLoop::new();
    async_lifecycle
        .start_async()
        .await
        .expect("async start failed");
    wait_until(&async_lifecycle, true).await;
    tokio::time::timeout(Duration::from_secs(2), async_lifecycle.stop_async())
        .await
        .expect("async stop timed out")
        .expect("async stop failed");
    assert!(!async_lifecycle.is_running());

    println!("RTR003 PASS {runtime}");
}

#[tokio::test(flavor = "current_thread")]
async fn lifecycle_routes_are_safe_in_current_thread_runtime() {
    exercise_lifecycle("current-thread").await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn lifecycle_routes_are_safe_in_multi_thread_runtime() {
    exercise_lifecycle("multi-thread").await;
}
