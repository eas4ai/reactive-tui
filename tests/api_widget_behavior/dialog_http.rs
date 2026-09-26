use super::{app_input, Control};
use reactive_tui::{
    component::Element,
    core::geometry::Rect,
    event::types::{Event, KeyCode, PasteEvent},
    reactive::ThreadSafeSignal,
    widgets::dialog::{
        DialogComponent, DialogId, DialogResult, DialogTheme, InputDialog, InputDialogOptions,
        ValidationConfig,
    },
};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

#[derive(Clone)]
struct Reply {
    status: u16,
    body: String,
    delay: Duration,
    hold_until_teardown: bool,
}

impl Reply {
    fn json(body: &str) -> Self {
        Self {
            status: 200,
            body: body.into(),
            delay: Duration::ZERO,
            hold_until_teardown: false,
        }
    }
}

#[derive(Debug)]
struct Request {
    headers: String,
    body: serde_json::Value,
}

struct Server {
    url: String,
    requests: Arc<Mutex<Vec<Request>>>,
    count: ThreadSafeSignal<usize>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

/// A hang guard, not a timing check: generous so a busy machine cannot fail
/// a correct test by running it slowly. Dropping the server ends it at once.
const HANG_GUARD: Duration = Duration::from_secs(30);

impl Server {
    fn new(replies: Vec<Reply>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}/validate", listener.local_addr().unwrap());
        let stop = Arc::new(AtomicBool::new(false));
        let cancelled = stop.clone();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = requests.clone();
        let count = ThreadSafeSignal::new(0);
        let request_count = count.clone();
        let worker = thread::spawn(move || {
            let end = Instant::now() + HANG_GUARD;
            let mut connections = Vec::new();
            for reply in replies {
                let connection = loop {
                    if cancelled.load(Ordering::Acquire) || Instant::now() >= end {
                        break None;
                    }
                    match listener.accept() {
                        Ok((stream, _)) => break Some(stream),
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(1))
                        }
                        Err(error) => panic!("accept loopback request: {error}"),
                    }
                };
                let Some(mut stream) = connection else { break };
                stream.set_nonblocking(false).unwrap();
                let stopped = cancelled.clone();
                let captured = captured.clone();
                let request_count = request_count.clone();
                connections.push(thread::spawn(move || {
                    let Some(request) = read_request(&mut stream, &stopped) else { return };
                    let count = {
                        let mut requests = captured.lock().unwrap();
                        requests.push(request);
                        requests.len()
                    };
                    request_count.set(count);
                    // A slow server: the reply waits its configured delay.
                    let until = Instant::now() + reply.delay;
                    while (reply.hold_until_teardown || Instant::now() < until)
                        && !stopped.load(Ordering::Acquire)
                    {
                        thread::sleep(Duration::from_millis(1));
                    }
                    if stopped.load(Ordering::Acquire) { return }
                    // A hang guard: a client that stops reading closes its end.
                    stream.set_write_timeout(Some(HANG_GUARD)).unwrap();
                    let response = format!("HTTP/1.1 {} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", reply.status, reply.body.len(), reply.body);
                    // The client may have cancelled or rejected an oversized response.
                    let _ = stream.write_all(response.as_bytes());
                }));
            }
            drop(listener);
            for connection in connections {
                connection.join().unwrap();
            }
        });
        Self {
            url,
            requests,
            count,
            stop,
            worker: Some(worker),
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            if let Err(error) = worker.join() {
                if !thread::panicking() {
                    std::panic::resume_unwind(error);
                }
            }
        }
    }
}

#[test]
fn dialog_engine_http_validation_delivers_an_awaitable_result() {
    use reactive_tui::widgets::dialog::{DialogEngine, DialogEvent, InputFieldConfig};
    for size in [(32, 12), (60, 20)] {
        let server = Server::new(vec![Reply::json(r#"{"valid":true}"#)]);
        let mut engine = DialogEngine::new();
        engine.enable_async();
        let id = engine.show_input(InputDialogOptions {
            title: "ENGINE REMOTE".into(),
            input: InputFieldConfig {
                default_value: Some("界e\u{301}🙂".into()),
                ..Default::default()
            },
            validation: Some(ValidationConfig {
                validate_on_change: false,
                validate_on_blur: false,
                async_validation_url: Some(server.url.clone()),
                ..Default::default()
            }),
            ..Default::default()
        });
        let completion = engine.completion(id).unwrap();
        app_input::run_until_hidden(
            engine.clone(),
            size,
            vec![("ENGINE REMOTE", app_input::key(KeyCode::Enter))],
            "ENGINE REMOTE",
        );
        assert!(
            matches!(futures_lite::future::block_on(completion.result()), DialogResult::Confirmed(Some(value)) if value == "界e\u{301}🙂")
        );
        let events: Vec<_> = std::iter::from_fn(|| engine.take_event()).collect();
        assert!(matches!(
            events.as_slice(),
            [
                DialogEvent::Opened(_),
                DialogEvent::Closed(_, DialogResult::Confirmed(Some(_)))
            ]
        ));
        let requests = server.requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].body,
            serde_json::json!({"value":"界e\u{301}🙂"})
        );
    }
}

#[test]
fn dialog_engine_http_close_cancels_a_live_request_and_does_not_submit() {
    use reactive_tui::{
        app::RootComponent,
        event::router::EventResult,
        widgets::dialog::{DialogEngine, DialogId},
    };
    use std::sync::atomic::AtomicUsize;
    struct Root {
        engine: DialogEngine,
        id: DialogId,
        requests: ThreadSafeSignal<usize>,
        closed_in: Arc<Mutex<Option<Duration>>>,
    }
    impl RootComponent for Root {
        fn render(&self) -> Element {
            let status = if self.engine.is_open(self.id) {
                format!("REQUESTS {}", self.requests.get())
            } else {
                "REMOVED".into()
            };
            // The cancellation gate must remain visible with a fully open
            // backdrop, not only during its opening fade.
            let mut status = Element::text(status);
            status.metadata.styles = Some(Arc::new(
                reactive_tui::layout::style::StyleBuilder::new()
                    .size_px(Some(16.0), Some(1.0))
                    .z_index(i32::MAX)
                    .snapshot(),
            ));
            reactive_tui::builder::div()
                .children(vec![status, self.engine.render()])
                .build()
        }
        fn wake_driven(&self) -> bool {
            true
        }
        fn try_handle_event(&mut self, event: &Event) -> reactive_tui::error::Result<EventResult> {
            if matches!(event, Event::Key(key) if key.code == KeyCode::F(2)) {
                let start = Instant::now();
                self.engine.close_dialog(self.id, DialogResult::Cancelled);
                *self.closed_in.lock().unwrap() = Some(start.elapsed());
                Ok(EventResult::Handled)
            } else {
                Ok(EventResult::Ignored)
            }
        }
    }
    let server = Server::new(vec![Reply {
        hold_until_teardown: true,
        ..Reply::json(r#"{"valid":true}"#)
    }]);
    let submissions = Arc::new(AtomicUsize::new(0));
    let submitted = submissions.clone();
    let mut config = reactive_tui::widgets::dialog::DialogEngineConfig::default();
    config.default_theme.animation = reactive_tui::widgets::dialog::DialogAnimation::None;
    let mut engine = DialogEngine::with_config(config);
    engine.enable_async();
    let id = engine.show_input(InputDialogOptions {
        title: "ENGINE PENDING".into(),
        validation: Some(ValidationConfig {
            validate_on_change: false,
            validate_on_blur: false,
            async_validation_url: Some(server.url.clone()),
            ..Default::default()
        }),
        on_submit: Some(Arc::new(move |_| {
            submitted.fetch_add(1, Ordering::SeqCst);
            true
        })),
        ..Default::default()
    });
    let completion = engine.completion(id).unwrap();
    let closed_in = Arc::new(Mutex::new(None));
    let frames = app_input::run_when(
        Root {
            engine: engine.clone(),
            id,
            requests: server.count.clone(),
            closed_in: closed_in.clone(),
        },
        (60, 20),
        vec![
            ("ENGINE PENDING", app_input::key(KeyCode::Enter)),
            ("REQUESTS 1", app_input::key(KeyCode::F(2))),
            ("REMOVED", None),
        ],
    );
    // The behavior under test: closing returns at once although the reply
    // is held until teardown.
    assert!(
        closed_in
            .lock()
            .unwrap()
            .expect("close event was delivered")
            < Duration::from_secs(1)
    );
    assert!(!frames.last().unwrap().text.contains("ENGINE PENDING"));
    assert_eq!(submissions.load(Ordering::SeqCst), 0);
    assert!(matches!(
        completion.try_result(),
        Some(DialogResult::Cancelled)
    ));
    assert_eq!(engine.active_count(), 0);
    assert_eq!(server.requests.lock().unwrap().len(), 1);
}

fn read_request(stream: &mut TcpStream, stopped: &AtomicBool) -> Option<Request> {
    stream
        .set_read_timeout(Some(Duration::from_millis(10)))
        .unwrap();
    let end = Instant::now() + HANG_GUARD;
    let mut bytes = Vec::new();
    let mut buffer = [0; 4096];
    while !stopped.load(Ordering::Acquire) && Instant::now() < end {
        match stream.read(&mut buffer) {
            Ok(0) => return None,
            Ok(count) => bytes.extend_from_slice(&buffer[..count]),
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                continue
            }
            Err(_) => return None,
        }
        assert!(bytes.len() <= 2 * 1024 * 1024, "request is unbounded");
        let Some(header_end) = bytes.windows(4).position(|bytes| bytes == b"\r\n\r\n") else {
            continue;
        };
        let headers = std::str::from_utf8(&bytes[..header_end]).unwrap();
        let length: usize = headers
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse().unwrap())
            })
            .unwrap();
        if bytes.len() >= header_end + 4 + length {
            return Some(Request {
                headers: headers.into(),
                body: serde_json::from_slice(&bytes[header_end + 4..header_end + 4 + length])
                    .unwrap(),
            });
        }
    }
    None
}

#[test]
fn autocomplete_http_sends_query_headers_and_renders_suggestion_objects() {
    use reactive_tui::widgets::dialog::{
        AutocompleteConfig, AutocompleteDialog, AutocompleteDialogOptions,
    };
    for (size, descriptions) in [
        ((32, 12), true),
        ((60, 20), true),
        ((32, 12), false),
        ((60, 20), false),
    ] {
        let server = Server::new(vec![Reply::json(
            r#"[{"value":"saved-id","display":"Remote choice","description":"EXPLANATION","icon":"*","metadata":{"source":"fixture"}}]"#,
        )]);
        let results = Arc::new(Mutex::new(Vec::new()));
        let closed = results.clone();
        let element = AutocompleteDialog::new(
            DialogId::from_u32(40),
            AutocompleteDialogOptions {
                title: "SUGGESTIONS".into(),
                prompt: "QUERY".into(),
                autocomplete: AutocompleteConfig {
                    default_value: Some("remote".into()),
                    suggestions_url: Some(server.url.clone()),
                    headers: Some([("X-Fixture".into(), "autocomplete".into())].into()),
                    debounce_delay: Duration::from_millis(10),
                    show_descriptions: descriptions,
                    ..Default::default()
                },
                on_close: Some(Arc::new(move |result| closed.lock().unwrap().push(result))),
                ..Default::default()
            },
        )
        .render(Rect::default(), &DialogTheme::default());
        let frames = app_input::run_until_hidden(
            Control(element),
            size,
            vec![(
                if descriptions {
                    "EXPLANATION"
                } else {
                    "Remote choice"
                },
                app_input::key(KeyCode::Enter),
            )],
            "SUGGESTIONS",
        );
        assert!(frames
            .iter()
            .any(|frame| frame.text.contains("* Remote choice")));
        assert_eq!(
            frames
                .iter()
                .any(|frame| frame.text.contains("EXPLANATION")),
            descriptions
        );
        assert!(
            matches!(results.lock().unwrap().as_slice(), [DialogResult::Selected(value)] if value == "saved-id")
        );
        let requests = server.requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].body, serde_json::json!({"query":"remote"}));
        assert!(requests[0].headers.contains("X-Fixture: autocomplete"));
    }
}

#[test]
fn autocomplete_http_rejects_malformed_suggestions_and_can_cancel() {
    use reactive_tui::widgets::dialog::{
        AutocompleteConfig, AutocompleteDialog, AutocompleteDialogOptions,
    };
    for size in [(32, 12), (60, 20)] {
        let server = Server::new(vec![Reply::json(
            r#"[{"display":"missing required value"}]"#,
        )]);
        let results = Arc::new(Mutex::new(Vec::new()));
        let closed = results.clone();
        let element = AutocompleteDialog::new(
            DialogId::from_u32(41),
            AutocompleteDialogOptions {
                title: "BAD RESPONSE".into(),
                autocomplete: AutocompleteConfig {
                    default_value: Some("query".into()),
                    suggestions_url: Some(server.url.clone()),
                    debounce_delay: Duration::ZERO,
                    ..Default::default()
                },
                on_close: Some(Arc::new(move |result| closed.lock().unwrap().push(result))),
                ..Default::default()
            },
        )
        .render(Rect::default(), &DialogTheme::default());
        app_input::run_until_hidden(
            Control(element),
            size,
            vec![("Invalid suggestion", app_input::key(KeyCode::Escape))],
            "BAD RESPONSE",
        );
        assert!(matches!(
            results.lock().unwrap().as_slice(),
            [DialogResult::Cancelled]
        ));
    }
}

fn input(url: &str, result: Arc<Mutex<Option<String>>>) -> Element {
    input_with(url, result, |_| {})
}

fn input_with(
    url: &str,
    result: Arc<Mutex<Option<String>>>,
    configure: impl FnOnce(&mut InputDialogOptions),
) -> Element {
    let mut options = InputDialogOptions {
        title: "REMOTE".into(),
        prompt: "VALUE".into(),
        validation: Some(ValidationConfig {
            validate_on_change: false,
            validate_on_blur: false,
            async_validation_url: Some(url.into()),
            ..Default::default()
        }),
        on_close: Some(Arc::new(move |value| {
            if let DialogResult::Confirmed(Some(value)) = value {
                assert!(result.lock().unwrap().replace(value).is_none());
            }
        })),
        ..Default::default()
    };
    configure(&mut options);
    InputDialog::new(DialogId::from_u32(21), options)
        .render(Rect::default(), &DialogTheme::default())
}

#[test]
#[cfg(unix)]
fn xis_001_disabled_dialog_configuration_never_spawns_curl() {
    const CHILD: &str = "RTUI_XIS_DISABLED_CHILD";
    const MARKER: &str = "RTUI_XIS_DISABLED_MARKER";
    if std::env::var_os(CHILD).is_some() {
        let marker = std::path::PathBuf::from(std::env::var_os(MARKER).unwrap());
        let result = Arc::new(Mutex::new(None));
        let closed = result.clone();
        let input = InputDialog::new(
            DialogId::from_u32(210),
            InputDialogOptions {
                title: "LOCAL INPUT".into(),
                prompt: "VALUE".into(),
                on_close: Some(Arc::new(move |value| {
                    if let DialogResult::Confirmed(Some(value)) = value {
                        *closed.lock().unwrap() = Some(value);
                    }
                })),
                ..Default::default()
            },
        )
        .render(Rect::default(), &DialogTheme::default());
        app_input::run_until_hidden(
            Control(input),
            (32, 12),
            vec![("VALUE", super::key(KeyCode::Enter))],
            "LOCAL INPUT",
        );
        assert_eq!(result.lock().unwrap().as_deref(), Some(""));

        use reactive_tui::widgets::dialog::{
            AutocompleteConfig, AutocompleteDialog, AutocompleteDialogOptions,
        };
        let autocomplete = AutocompleteDialog::new(
            DialogId::from_u32(211),
            AutocompleteDialogOptions {
                title: "LOCAL SEARCH".into(),
                prompt: "QUERY".into(),
                autocomplete: AutocompleteConfig {
                    static_suggestions: vec!["local".into()],
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .render(Rect::default(), &DialogTheme::default());
        app_input::run_until_hidden(
            Control(autocomplete),
            (32, 12),
            vec![("LOCAL SEARCH", super::key(KeyCode::Escape))],
            "LOCAL SEARCH",
        );
        assert!(
            !marker.exists(),
            "disabled dialog configuration started curl"
        );
        return;
    }

    use std::os::unix::fs::PermissionsExt;
    let directory = tempfile::tempdir().unwrap();
    let marker = directory.path().join("started");
    let executable = directory.path().join("curl");
    std::fs::write(
        &executable,
        format!("#!/bin/sh\n: > '{}'\nexit 99\n", marker.display()),
    )
    .unwrap();
    let mut permissions = std::fs::metadata(&executable).unwrap().permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(&executable, permissions).unwrap();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "dialog_http_acceptance::xis_001_disabled_dialog_configuration_never_spawns_curl",
            "--nocapture",
        ])
        .env_clear()
        .env(CHILD, "1")
        .env(MARKER, &marker)
        .env("PATH", directory.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
    assert!(
        !marker.exists(),
        "disabled dialog configuration started curl"
    );
}

#[test]
fn input_dialog_remote_validation_posts_json_and_completes_pending_submission() {
    for size in [(32, 12), (60, 20)] {
        let server = Server::new(vec![Reply::json(r#"{"valid":true}"#)]);
        let result = Arc::new(Mutex::new(None));
        app_input::run_until_hidden(
            Control(input(&server.url, result.clone())),
            size,
            vec![
                (
                    "VALUE",
                    Some(Event::Paste(PasteEvent::new("界e\u{301}🙂".into()))),
                ),
                ("界e\u{301}🙂", super::key(KeyCode::Enter)),
            ],
            "REMOTE",
        );
        assert_eq!(result.lock().unwrap().as_deref(), Some("界e\u{301}🙂"));
        let requests = server.requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].body,
            serde_json::json!({"value":"界e\u{301}🙂"})
        );
        assert!(requests[0]
            .headers
            .starts_with("POST /validate HTTP/1.1\r\n"));
        assert!(requests[0]
            .headers
            .to_ascii_lowercase()
            .contains("content-type: application/json"));
    }
}

#[test]
fn input_dialog_remote_rejection_can_be_corrected_and_resubmitted() {
    for size in [(32, 12), (60, 20)] {
        let server = Server::new(vec![
            Reply::json(r#"{"valid":false,"message":"Name already taken"}"#),
            Reply::json(r#"{"valid":true}"#),
        ]);
        let result = Arc::new(Mutex::new(None));
        app_input::run_until_hidden(
            Control(input(&server.url, result.clone())),
            size,
            vec![
                ("VALUE", super::key(KeyCode::Char('a'))),
                ("VALUE", super::key(KeyCode::Enter)),
                ("Name already taken", super::key(KeyCode::Char('b'))),
                ("ab", super::key(KeyCode::Enter)),
            ],
            "REMOTE",
        );
        assert_eq!(result.lock().unwrap().as_deref(), Some("ab"));
        let requests = server.requests.lock().unwrap();
        assert_eq!(
            requests
                .iter()
                .map(|request| request.body.clone())
                .collect::<Vec<_>>(),
            [
                serde_json::json!({"value":"a"}),
                serde_json::json!({"value":"ab"})
            ]
        );
    }
}

#[test]
fn input_dialog_remote_http_and_json_failures_remain_open() {
    for (reply, error) in [
        (
            Reply {
                status: 503,
                ..Reply::json(r#"{"valid":true}"#)
            },
            "503",
        ),
        (Reply::json("not JSON"), "JSON"),
        (Reply::json(r#"{"valid":false,"message":""}"#), "rejected"),
        (Reply::json(r#"{"message":"missing decision"}"#), "JSON"),
    ] {
        for size in [(32, 12), (60, 20)] {
            let server = Server::new(vec![reply.clone()]);
            let result = Arc::new(Mutex::new(None));
            let frames = app_input::run_until_hidden(
                Control(input(&server.url, result.clone())),
                size,
                vec![
                    ("VALUE", super::key(KeyCode::Enter)),
                    (error, super::key(KeyCode::Escape)),
                ],
                "REMOTE",
            );
            assert!(frames.iter().any(|frame| frame.text.contains(error)));
            assert!(result.lock().unwrap().is_none());
        }
    }
}

struct ObservedInput {
    element: Element,
    requests: ThreadSafeSignal<usize>,
    visible: ThreadSafeSignal<bool>,
    /// When F(2) removed the element: removal's timing starts there.
    removed_at: Arc<Mutex<Option<Instant>>>,
}

fn autocomplete_remote(
    url: &str,
    seed: Option<&str>,
    delay: Duration,
    results: Arc<Mutex<Vec<DialogResult>>>,
) -> Element {
    use reactive_tui::widgets::dialog::{
        AutocompleteConfig, AutocompleteDialog, AutocompleteDialogOptions,
    };
    AutocompleteDialog::new(
        DialogId::from_u32(44),
        AutocompleteDialogOptions {
            title: "REMOTE SEARCH".into(),
            prompt: "QUERY".into(),
            autocomplete: AutocompleteConfig {
                default_value: seed.map(str::to_owned),
                suggestions_url: Some(url.into()),
                debounce_delay: delay,
                ..Default::default()
            },
            on_close: Some(Arc::new(move |result| results.lock().unwrap().push(result))),
            ..Default::default()
        },
    )
    .render(Rect::default(), &DialogTheme::default())
}

#[test]
fn autocomplete_http_debounces_edits_into_one_current_query() {
    for size in [(32, 12), (60, 20)] {
        let server = Server::new(vec![Reply::json(r#"["Fresh result"]"#)]);
        let results = Arc::new(Mutex::new(Vec::new()));
        let element = autocomplete_remote(
            &server.url,
            None,
            // Long enough that the second key always lands inside it.
            Duration::from_secs(2),
            results.clone(),
        );
        app_input::run_until_hidden(
            Control(element),
            size,
            vec![
                ("QUERY", super::key(KeyCode::Char('a'))),
                ("QUERY", super::key(KeyCode::Char('b'))),
                ("Fresh result", super::key(KeyCode::Enter)),
            ],
            "REMOTE SEARCH",
        );
        let requests = server.requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].body, serde_json::json!({"query":"ab"}));
        assert!(
            matches!(results.lock().unwrap().as_slice(), [DialogResult::Selected(value)] if value == "Fresh result")
        );
    }
}

#[test]
fn autocomplete_http_replacement_discards_the_previous_query() {
    for size in [(32, 12), (60, 20)] {
        let server = Server::new(vec![
            Reply {
                // Still pending when the second key replaces the query.
                delay: Duration::from_secs(2),
                ..Reply::json(r#"["Old result"]"#)
            },
            Reply::json(r#"["New result"]"#),
        ]);
        let results = Arc::new(Mutex::new(Vec::new()));
        let root = ObservedInput {
            element: autocomplete_remote(&server.url, None, Duration::ZERO, results.clone()),
            requests: server.count.clone(),
            visible: ThreadSafeSignal::new(true),
            removed_at: Default::default(),
        };
        let frames = app_input::run_until_hidden(
            root,
            size,
            vec![
                ("QUERY", super::key(KeyCode::Char('a'))),
                ("REQUESTS 1", super::key(KeyCode::Char('b'))),
                ("New result", super::key(KeyCode::Enter)),
            ],
            "REMOTE SEARCH",
        );
        assert!(frames
            .iter()
            .all(|frame| !frame.text.contains("Old result")));
        let requests = server.requests.lock().unwrap();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].body, serde_json::json!({"query":"a"}));
        assert_eq!(requests[1].body, serde_json::json!({"query":"ab"}));
        assert!(
            matches!(results.lock().unwrap().as_slice(), [DialogResult::Selected(value)] if value == "New result")
        );
    }
}

#[test]
fn autocomplete_removal_cancels_a_live_http_request() {
    for size in [(32, 12), (60, 20)] {
        // The reply waits 30 s; finishing well inside that shows removal did
        // not wait for it, with room for a busy machine. Dropping the server
        // ends the wait at once.
        let server = Server::new(vec![Reply {
            delay: Duration::from_secs(30),
            ..Reply::json(r#"["Late result"]"#)
        }]);
        let results = Arc::new(Mutex::new(Vec::new()));
        let root = ObservedInput {
            element: autocomplete_remote(&server.url, Some("a"), Duration::ZERO, results.clone()),
            requests: server.count.clone(),
            visible: ThreadSafeSignal::new(true),
            removed_at: Default::default(),
        };
        let removed_at = root.removed_at.clone();
        app_input::run_when(
            root,
            size,
            vec![("REQUESTS 1", super::key(KeyCode::F(2))), ("REMOVED", None)],
        );
        // The behavior under test, timed from the removal: the App finished
        // well inside the reply's 30 s.
        let removed_at = removed_at.lock().unwrap().expect("F(2) removed the dialog");
        assert!(removed_at.elapsed() < Duration::from_secs(10));
        assert!(results.lock().unwrap().is_empty());
        assert_eq!(server.requests.lock().unwrap().len(), 1);
    }
}

impl reactive_tui::app::RootComponent for ObservedInput {
    fn render(&self) -> Element {
        // Keep test synchronization visible after the modal backdrop finishes
        // fading in. A marker underneath it only works during the animation.
        let mut requests = Element::text(format!("REQUESTS {}", self.requests.get()));
        requests.metadata.styles = Some(Arc::new(
            reactive_tui::layout::style::StyleBuilder::new()
                .size_px(Some(16.0), Some(1.0))
                .z_index(i32::MAX)
                .snapshot(),
        ));
        reactive_tui::builder::div()
            .class("flex-col")
            .children(vec![
                requests,
                if self.visible.get() {
                    self.element.clone()
                } else {
                    Element::text("REMOVED")
                },
            ])
            .build()
    }
    fn handle_event(&self, event: &Event) -> reactive_tui::event::router::EventResult {
        if matches!(event, Event::Key(key) if key.code == KeyCode::F(2)) {
            *self.removed_at.lock().unwrap() = Some(Instant::now());
            self.visible.set(false);
            reactive_tui::event::router::EventResult::Handled
        } else {
            reactive_tui::event::router::EventResult::Ignored
        }
    }
}

#[test]
fn input_dialog_edit_cancels_pending_remote_submission() {
    let server = Server::new(vec![
        Reply {
            // Still pending when the edit cancels it.
            delay: Duration::from_secs(2),
            ..Reply::json(r#"{"valid":true}"#)
        },
        Reply::json(r#"{"valid":true}"#),
    ]);
    let result = Arc::new(Mutex::new(None));
    let root = ObservedInput {
        element: input(&server.url, result.clone()),
        requests: server.count.clone(),
        visible: ThreadSafeSignal::new(true),
        removed_at: Default::default(),
    };
    app_input::run_until_hidden(
        root,
        (60, 20),
        vec![
            ("VALUE", super::key(KeyCode::Char('a'))),
            ("VALUE", super::key(KeyCode::Enter)),
            ("REQUESTS 1", super::key(KeyCode::Char('b'))),
            ("ab", super::key(KeyCode::Enter)),
        ],
        "REMOTE",
    );
    assert_eq!(result.lock().unwrap().as_deref(), Some("ab"));
    assert_eq!(server.requests.lock().unwrap().len(), 2);
}

#[test]
fn input_dialog_removal_cancels_a_live_request_without_waiting_for_the_server() {
    // The reply waits 30 s; finishing well inside that shows removal did not
    // wait for it, with room for a busy machine. Dropping the server ends
    // the wait at once.
    let server = Server::new(vec![Reply {
        delay: Duration::from_secs(30),
        ..Reply::json(r#"{"valid":true}"#)
    }]);
    let result = Arc::new(Mutex::new(None));
    let root = ObservedInput {
        element: input(&server.url, result.clone()),
        requests: server.count.clone(),
        visible: ThreadSafeSignal::new(true),
        removed_at: Default::default(),
    };
    let removed_at = root.removed_at.clone();
    let frames = app_input::run_when(
        root,
        (60, 20),
        vec![
            ("VALUE", super::key(KeyCode::Enter)),
            ("REQUESTS 1", super::key(KeyCode::F(2))),
            ("REMOVED", None),
        ],
    );
    // The behavior under test, timed from the removal: the App finished well
    // inside the reply's 30 s.
    let removed_at = removed_at.lock().unwrap().expect("F(2) removed the dialog");
    assert!(removed_at.elapsed() < Duration::from_secs(10));
    assert!(result.lock().unwrap().is_none());
    assert!(!frames.last().unwrap().text.contains("REMOTE"));
    assert_eq!(server.requests.lock().unwrap().len(), 1);
}

#[test]
fn input_dialog_remote_completion_preserves_the_submit_veto_and_revalidates_on_retry() {
    let server = Server::new(vec![
        Reply {
            // Complete after the backdrop animation, so synchronization cannot
            // accidentally depend on seeing text through a fading backdrop.
            delay: Duration::from_millis(600),
            ..Reply::json(r#"{"valid":true}"#)
        },
        Reply::json(r#"{"valid":true}"#),
    ]);
    let result = Arc::new(Mutex::new(None));
    let submitted = ThreadSafeSignal::new(0);
    let calls = submitted.clone();
    let dialog = input_with(&server.url, result.clone(), move |options| {
        options.on_submit = Some(Arc::new(move |_| {
            let previous = calls.get();
            calls.set(previous + 1);
            previous > 0
        }));
    });
    let root = ObservedInput {
        element: dialog,
        requests: submitted,
        visible: ThreadSafeSignal::new(true),
        removed_at: Default::default(),
    };
    app_input::run_until_hidden(
        root,
        (60, 20),
        vec![
            ("VALUE", super::key(KeyCode::Enter)),
            ("REQUESTS 1", super::key(KeyCode::Enter)),
        ],
        "REMOTE",
    );
    assert_eq!(result.lock().unwrap().as_deref(), Some(""));
    assert_eq!(server.requests.lock().unwrap().len(), 2);
}

struct InlineValidatorInput {
    url: String,
    result: Arc<Mutex<Option<String>>>,
}

impl reactive_tui::app::RootComponent for InlineValidatorInput {
    fn render(&self) -> Element {
        use reactive_tui::widgets::dialog::{InputFieldConfig, ValidationResult};
        let closed = self.result.clone();
        InputDialog::new(
            DialogId::from_u32(23),
            InputDialogOptions {
                title: "REMOTE".into(),
                prompt: "VALUE".into(),
                input: InputFieldConfig {
                    default_value: Some("ok".into()),
                    ..Default::default()
                },
                validation: Some(ValidationConfig {
                    validate_on_change: false,
                    validate_on_blur: false,
                    async_validation_url: Some(self.url.clone()),
                    custom_validator: Some(Arc::new(|_| ValidationResult {
                        valid: true,
                        message: None,
                        warnings: Vec::new(),
                    })),
                    ..Default::default()
                }),
                on_validate: Some(Arc::new(|_| ValidationResult {
                    valid: true,
                    message: None,
                    warnings: Vec::new(),
                })),
                on_close: Some(Arc::new(move |value| {
                    let DialogResult::Confirmed(Some(value)) = value else {
                        panic!("unexpected cancellation")
                    };
                    assert!(closed.lock().unwrap().replace(value).is_none());
                })),
                ..Default::default()
            },
        )
        .render(Rect::default(), &DialogTheme::default())
    }
}

#[test]
fn input_dialog_inline_validators_do_not_cancel_a_matching_http_request_on_redraw() {
    let server = Server::new(vec![Reply {
        delay: Duration::from_millis(50),
        ..Reply::json(r#"{"valid":true}"#)
    }]);
    let result = Arc::new(Mutex::new(None));
    app_input::run_until_hidden(
        InlineValidatorInput {
            url: server.url.clone(),
            result: result.clone(),
        },
        (60, 20),
        vec![("VALUE", super::key(KeyCode::Enter))],
        "REMOTE",
    );
    assert_eq!(result.lock().unwrap().as_deref(), Some("ok"));
    assert_eq!(server.requests.lock().unwrap().len(), 1);
}

#[test]
fn input_dialog_remote_completion_rechecks_current_local_validation() {
    let server = Server::new(vec![Reply::json(r#"{"valid":true}"#)]);
    let result = Arc::new(Mutex::new(None));
    let requests = server.count.clone();
    let dialog = input_with(&server.url, result.clone(), move |options| {
        options.on_validate = Some(Arc::new(move |_| {
            reactive_tui::widgets::dialog::ValidationResult {
                valid: requests.get() == 0,
                message: Some("Local rule rejected".into()),
                warnings: Vec::new(),
            }
        }));
    });
    app_input::run_until_hidden(
        Control(dialog),
        (60, 20),
        vec![
            ("VALUE", super::key(KeyCode::Enter)),
            ("Local rule rejected", super::key(KeyCode::Escape)),
        ],
        "REMOTE",
    );
    assert!(result.lock().unwrap().is_none());
    assert_eq!(server.requests.lock().unwrap().len(), 1);
}

#[test]
fn input_dialog_remote_change_validation_keeps_warnings_and_reuses_the_matching_result() {
    for size in [(32, 12), (60, 20)] {
        let server = Server::new(vec![Reply::json(
            r#"{"valid":true,"warnings":["Review name"]}"#,
        )]);
        let result = Arc::new(Mutex::new(None));
        let dialog = input_with(&server.url, result.clone(), |options| {
            let validation = options.validation.as_mut().unwrap();
            validation.validate_on_change = true;
            validation.debounce_delay = Duration::from_millis(1);
        });
        app_input::run_until_hidden(
            Control(dialog),
            size,
            vec![
                ("VALUE", super::key(KeyCode::Char('a'))),
                ("Review name", super::key(KeyCode::Enter)),
            ],
            "REMOTE",
        );
        assert_eq!(result.lock().unwrap().as_deref(), Some("a"));
        assert_eq!(server.requests.lock().unwrap().len(), 1);
    }
}
