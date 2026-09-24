use reactive_tui::{
    app::{App, RootComponent, RootUpdate},
    backend::SuprTuiBackend,
    component::Element,
    error::{ReactiveError, Result},
    ui::Updater,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

struct Root {
    value: Arc<AtomicUsize>,
    observed: Arc<Mutex<Vec<usize>>>,
}
impl RootComponent for Root {
    fn render(&self) -> Element {
        let value = self.value.load(Ordering::SeqCst);
        self.observed.lock().unwrap().push(value);
        Element::text(format!("updated {value}"))
    }
    fn update(&mut self) -> Result<RootUpdate> {
        Ok(RootUpdate::Exit)
    }
    fn accepts_input(&self) -> bool {
        false
    }
}
struct Update {
    value: Arc<AtomicUsize>,
    dropped: Arc<AtomicUsize>,
    failure: u8,
}
impl Updater for Update {
    fn update(&mut self) -> Result<()> {
        match self.failure {
            1 => Err(ReactiveError::invalid_state("controlled updater error")),
            2 => panic!("controlled updater panic"),
            _ => {
                self.value.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }
    }
}
impl Drop for Update {
    fn drop(&mut self) {
        self.dropped.fetch_add(1, Ordering::SeqCst);
    }
}
fn fixture() -> (App, Arc<AtomicUsize>, Arc<Mutex<Vec<usize>>>) {
    let value = Arc::new(AtomicUsize::new(0));
    let observed = Arc::new(Mutex::new(Vec::new()));
    let app = App::builder()
        .backend(SuprTuiBackend::with_writer(20, 4, std::io::sink()).unwrap())
        .root(Root {
            value: value.clone(),
            observed: observed.clone(),
        })
        .build()
        .unwrap();
    (app, value, observed)
}
#[test]
fn requests_update_only_the_registered_app_before_render_and_close_after_exit() {
    let (mut first, value, observed) = fixture();
    let (second, other, _) = fixture();
    let dropped = Arc::new(AtomicUsize::new(0));
    let token = first.register_updater(Update {
        value: value.clone(),
        dropped: dropped.clone(),
        failure: 0,
    });
    let handle = token.handle();
    std::thread::spawn({
        let handle = handle.clone();
        move || {
            for _ in 0..1000 {
                assert!(handle.request());
            }
        }
    })
    .join()
    .unwrap();
    first.run().unwrap();
    assert_eq!(*observed.lock().unwrap(), vec![1]);
    assert_eq!(value.load(Ordering::SeqCst), 1);
    assert_eq!(other.load(Ordering::SeqCst), 0);
    assert_eq!(dropped.load(Ordering::SeqCst), 1);
    assert!(!handle.request());
    second.run().unwrap();
}
#[test]
fn errors_unwinds_and_unrun_drop_close_requests_and_release_updaters() {
    for failure in 0..3 {
        let (mut app, value, _) = fixture();
        let dropped = Arc::new(AtomicUsize::new(0));
        let token = app.register_updater(Update {
            value,
            dropped: dropped.clone(),
            failure,
        });
        let handle = token.handle();
        handle.request();
        if failure == 0 {
            drop(app);
        } else {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| app.run()));
            if failure == 1 {
                assert!(result
                    .unwrap()
                    .unwrap_err()
                    .to_string()
                    .contains("controlled updater error"));
            } else {
                assert!(result.is_err());
            }
        }
        assert!(!handle.request());
        assert_eq!(dropped.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn worker_request_wakes_idle_app_and_repaints_state() {
    struct LiveRoot {
        value: Arc<AtomicUsize>,
        painted: Arc<AtomicUsize>,
    }
    impl RootComponent for LiveRoot {
        fn render(&self) -> Element {
            let value = self.value.load(Ordering::SeqCst);
            self.painted.store(value + 1, Ordering::SeqCst);
            Element::text(format!("live {value}"))
        }
        fn update(&mut self) -> Result<RootUpdate> {
            Ok(if self.painted.load(Ordering::SeqCst) == 2 {
                RootUpdate::Exit
            } else {
                RootUpdate::Unchanged
            })
        }
        fn wake_driven(&self) -> bool {
            true
        }
        fn accepts_input(&self) -> bool {
            false
        }
    }
    let value = Arc::new(AtomicUsize::new(0));
    let painted = Arc::new(AtomicUsize::new(0));
    let mut app = App::builder()
        .backend(SuprTuiBackend::with_writer(20, 4, std::io::sink()).unwrap())
        .root(LiveRoot {
            value: value.clone(),
            painted: painted.clone(),
        })
        .build()
        .unwrap();
    let token = app.register_updater(Update {
        value,
        dropped: Arc::new(AtomicUsize::new(0)),
        failure: 0,
    });
    let handle = token.handle();
    let wake = app.waker();
    let (done, wait) = std::sync::mpsc::channel();
    let observer = painted.clone();
    let worker = std::thread::spawn(move || {
        // Both waits are a hang guard, not a timing check: generous so a busy machine cannot fail a correct test.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while observer.load(Ordering::SeqCst) == 0 && std::time::Instant::now() < deadline {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        handle.request();
        if wait
            .recv_timeout(std::time::Duration::from_secs(30))
            .is_err()
        {
            wake.request_stop();
        }
    });
    let result = app.run();
    let _ = done.send(());
    worker.join().unwrap();
    result.unwrap();
    assert_eq!(
        painted.load(Ordering::SeqCst),
        2,
        "worker request did not repaint the updated state"
    );
}
