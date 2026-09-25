use super::{
    app_input::{run_when, run_when_all},
    click, key, Control,
};
use reactive_tui::{
    app::RootComponent,
    builder,
    component::Element,
    event::types::{Event, KeyCode, KeyEvent, KeyModifiers},
    widgets::display::SelectionMode,
};
use std::sync::{Arc, Mutex};

type Calls = Arc<Mutex<Vec<(String, Vec<String>)>>>;

struct Notifications {
    element: Element,
    calls: Calls,
}
impl RootComponent for Notifications {
    fn render(&self) -> Element {
        self.element.clone()
    }
    fn wake_driven(&self) -> bool {
        true
    }
    fn handle_event(&self, event: &Event) -> reactive_tui::event::router::EventResult {
        if let Event::Custom(event) = event {
            let payload: serde_json::Value = serde_json::from_slice(&event.data).unwrap();
            self.calls.lock().unwrap().push((
                event.name.clone(),
                serde_json::from_value(payload["paths"].clone()).unwrap(),
            ));
            return reactive_tui::event::router::EventResult::Consumed;
        }
        reactive_tui::event::router::EventResult::Ignored
    }
}

fn modified(code: KeyCode, modifiers: KeyModifiers) -> Option<Event> {
    Some(Event::Key(KeyEvent::new(code).with_modifiers(modifiers)))
}

#[test]
fn explorer_loads_real_directory_and_navigates_with_app_input() {
    let fixture = tempfile::tempdir().unwrap();
    std::fs::create_dir(fixture.path().join("Folder")).unwrap();
    std::fs::write(fixture.path().join("alpha.txt"), "Alpha preview").unwrap();
    std::fs::write(fixture.path().join("Folder/inside.txt"), "Inside").unwrap();
    for size in [(24, 8), (48, 14)] {
        let element = builder::file_explorer()
            .root_path(fixture.path())
            .show_details(false)
            .build()
            .auto_focus();
        let frames = run_when(
            Control(element),
            size,
            vec![
                ("alpha.txt", click(7, 3)),
                ("1 selected", key(KeyCode::Enter)),
                ("inside.txt", key(KeyCode::Backspace)),
                ("alpha.txt", None),
            ],
        );
        assert!(frames.iter().any(|frame| frame.text.contains("inside.txt")));
        assert!(frames.last().unwrap().text.contains("Folder"));
    }
}

#[test]
fn explorer_preview_wakes_an_idle_app() {
    let fixture = tempfile::tempdir().unwrap();
    std::fs::write(fixture.path().join("alpha.txt"), "Alpha preview").unwrap();
    for size in [(24, 10), (48, 14)] {
        let element = builder::file_explorer()
            .root_path(fixture.path())
            .show_details(false)
            .show_preview(true)
            .build();
        let frames = run_when(Control(element), size, vec![("Alpha preview", None)]);
        assert!(frames.last().unwrap().text.contains("alpha.txt"));
    }
}

#[test]
fn explorer_copy_rename_move_and_confirmed_delete_change_real_files() {
    for size in [(24, 10), (48, 14)] {
        let fixture = tempfile::tempdir().unwrap();
        std::fs::write(fixture.path().join("alpha.txt"), "original bytes").unwrap();
        std::fs::create_dir(fixture.path().join("dest")).unwrap();
        let element = builder::file_explorer()
            .root_path(fixture.path())
            .show_details(false)
            .build()
            .auto_focus();
        let mut steps = vec![
            ("alpha.txt", key(KeyCode::End)),
            ("1 selected", key(KeyCode::F(7))),
            ("Copy:", key(KeyCode::Enter)),
            (
                "Enter a destination",
                Some(Event::Paste(reactive_tui::event::types::PasteEvent::new(
                    "copy.txt".into(),
                ))),
            ),
        ];
        steps.extend([
            ("copy.txt▏", key(KeyCode::Enter)),
            ("Copy: 1 completed", None),
        ]);
        run_when(Control(element), size, steps);
        assert_eq!(
            std::fs::read(fixture.path().join("copy.txt")).unwrap(),
            b"original bytes"
        );
        assert_eq!(
            std::fs::read(fixture.path().join("alpha.txt")).unwrap(),
            b"original bytes"
        );

        let element = builder::file_explorer()
            .root_path(fixture.path())
            .search("copy.txt")
            .show_details(false)
            .build()
            .auto_focus();
        let mut steps = vec![("1 files", key(KeyCode::F(2)))];
        for c in "renamed.txt".chars() {
            steps.push(("Rename:", key(KeyCode::Char(c))));
        }
        steps.extend([
            ("renamed.txt▏", key(KeyCode::Enter)),
            ("Rename: 1 completed", None),
        ]);
        run_when(Control(element), size, steps);
        assert!(!fixture.path().join("copy.txt").exists());
        assert_eq!(
            std::fs::read(fixture.path().join("renamed.txt")).unwrap(),
            b"original bytes"
        );

        let element = builder::file_explorer()
            .root_path(fixture.path())
            .search("renamed.txt")
            .show_details(false)
            .build()
            .auto_focus();
        let mut steps = vec![("1 files", key(KeyCode::F(6)))];
        for c in "dest".chars() {
            steps.push(("Move:", key(KeyCode::Char(c))));
        }
        steps.extend([("dest▏", key(KeyCode::Enter)), ("Move: 1 completed", None)]);
        run_when(Control(element), size, steps);
        assert!(!fixture.path().join("renamed.txt").exists());
        assert_eq!(
            std::fs::read(fixture.path().join("dest/renamed.txt")).unwrap(),
            b"original bytes"
        );

        let element = builder::file_explorer()
            .root_path(fixture.path())
            .current_path(fixture.path().join("dest"))
            .show_details(false)
            .build()
            .auto_focus();
        run_when(
            Control(element),
            size,
            vec![
                ("renamed.txt", key(KeyCode::Delete)),
                ("Delete 1 entries?", key(KeyCode::Escape)),
                ("1 files", None),
            ],
        );
        assert!(fixture.path().join("dest/renamed.txt").exists());
        let element = builder::file_explorer()
            .root_path(fixture.path())
            .current_path(fixture.path().join("dest"))
            .show_details(false)
            .build()
            .auto_focus();
        run_when(
            Control(element),
            size,
            vec![
                ("renamed.txt", key(KeyCode::Delete)),
                ("Delete 1 entries?", key(KeyCode::Enter)),
                ("Delete: 1 completed", None),
            ],
        );
        assert!(!fixture.path().join("dest/renamed.txt").exists());
    }
}

#[test]
fn explorer_multiple_selection_uses_paths_and_emits_callbacks_once() {
    let fixture = tempfile::tempdir().unwrap();
    for name in ["a.txt", "b.txt", "c.txt"] {
        std::fs::write(fixture.path().join(name), name).unwrap();
    }
    let root = fixture.path().canonicalize().unwrap();
    let path = |name| root.join(name).to_string_lossy().into_owned();
    for size in [(24, 10), (48, 14)] {
        let calls = Arc::default();
        let element = builder::file_explorer()
            .root_path(fixture.path())
            .show_details(false)
            .selection_mode(SelectionMode::Multiple)
            .on_select("selected")
            .on_activate("activated")
            .build()
            .auto_focus();
        run_when(
            Notifications {
                element,
                calls: Arc::clone(&calls),
            },
            size,
            vec![
                ("3 files", click(7, 3)),
                (
                    "1 selected",
                    modified(
                        KeyCode::Down,
                        KeyModifiers {
                            shift: true,
                            ..KeyModifiers::empty()
                        },
                    ),
                ),
                (
                    "2 selected",
                    modified(
                        KeyCode::Char(' '),
                        KeyModifiers {
                            ctrl: true,
                            ..KeyModifiers::empty()
                        },
                    ),
                ),
                ("1 selected", key(KeyCode::End)),
                ("1 selected", key(KeyCode::Enter)),
                ("c.txt", None),
            ],
        );
        assert_eq!(
            *calls.lock().unwrap(),
            vec![
                ("selected".into(), vec![path("a.txt")]),
                ("selected".into(), vec![path("a.txt"), path("b.txt")]),
                ("selected".into(), vec![path("a.txt")]),
                ("selected".into(), vec![path("c.txt")]),
                ("activated".into(), vec![path("c.txt")]),
            ]
        );
    }
}

#[test]
fn explorer_breadcrumbs_respect_root_and_notify_new_location() {
    let fixture = tempfile::tempdir().unwrap();
    std::fs::create_dir(fixture.path().join("Folder")).unwrap();
    std::fs::write(fixture.path().join("Folder/inside"), "inside").unwrap();
    for size in [(24, 10), (48, 14)] {
        let calls = Arc::default();
        let element = builder::file_explorer()
            .root_path(fixture.path())
            .show_details(false)
            .on_navigate("navigate")
            .build()
            .auto_focus();
        let frames = run_when(
            Notifications {
                element,
                calls: Arc::clone(&calls),
            },
            size,
            vec![
                ("Folder", key(KeyCode::Enter)),
                ("inside", click(1, 0)),
                ("Folder", key(KeyCode::Backspace)),
                ("Folder", None),
            ],
        );
        assert_eq!(
            *calls.lock().unwrap(),
            vec![
                (
                    "navigate".into(),
                    vec![fixture
                        .path()
                        .canonicalize()
                        .unwrap()
                        .join("Folder")
                        .to_string_lossy()
                        .into_owned()]
                ),
                (
                    "navigate".into(),
                    vec![fixture
                        .path()
                        .canonicalize()
                        .unwrap()
                        .to_string_lossy()
                        .into_owned()]
                ),
            ]
        );
        assert!(frames.last().unwrap().text.contains("Root/"));
    }
}

#[test]
fn explorer_views_filter_hidden_files_and_render_a_real_expanded_tree() {
    use reactive_tui::widgets::display::ViewMode;
    let fixture = tempfile::tempdir().unwrap();
    std::fs::create_dir(fixture.path().join("Folder")).unwrap();
    for name in ["a.rs", "b.txt", ".hidden.rs", "Folder/child.rs"] {
        std::fs::write(fixture.path().join(name), name).unwrap();
    }
    for size in [(24, 10), (48, 14)] {
        let element = builder::code_project_explorer(fixture.path())
            .root_path(fixture.path())
            .show_details(false)
            .file_filters(vec!["RS".into()])
            .build()
            .auto_focus();
        let frames = run_when(
            Control(element),
            size,
            vec![
                ("2 files", key(KeyCode::Right)),
                ("child.rs", key(KeyCode::Char('.'))),
                (".hidden.rs", key(KeyCode::Char('/'))),
                ("/", key(KeyCode::Char('a'))),
                ("/a", key(KeyCode::Enter)),
                ("2 files", None),
            ],
        );
        assert!(frames.iter().any(|frame| frame.text.contains("child.rs")));
        assert!(!frames.last().unwrap().text.contains("child.rs"));
        assert!(!frames.iter().any(|frame| frame.text.contains("b.txt")));
        for view in [ViewMode::List, ViewMode::Grid] {
            let element = builder::file_explorer()
                .root_path(fixture.path())
                .show_details(false)
                .view_mode(view.clone())
                .build();
            let frames = run_when_all(
                Control(element),
                size,
                vec![(&["Folder", "a.rs", "b.txt"], None)],
            );
            let last = frames.last().unwrap();
            assert!(last.text.contains("a.rs") && last.text.contains("b.txt"));
            if size.0 == 48 && view == ViewMode::Grid {
                assert!(last
                    .text
                    .lines()
                    .any(|line| line.contains("Folder") && line.contains("a.rs")));
            } else {
                assert!(!last
                    .text
                    .lines()
                    .any(|line| line.contains("Folder") && line.contains("a.rs")));
            }
        }
    }
}

#[test]
fn explorer_virtual_rows_and_resize_keep_mouse_targets_on_the_painted_entries() {
    use reactive_tui::event::types::ResizeEvent;
    let fixture = tempfile::tempdir().unwrap();
    for index in 0..100 {
        std::fs::write(fixture.path().join(format!("file{index:03}")), "data").unwrap();
    }
    for size in [(24, 8), (48, 14)] {
        let calls = Arc::default();
        let element = builder::file_explorer()
            .root_path(fixture.path())
            .show_details(false)
            .max_visible_items(4)
            .on_select("selected")
            .build()
            .auto_focus();
        let frames = run_when(
            Notifications {
                element,
                calls: Arc::clone(&calls),
            },
            size,
            vec![
                ("file000", key(KeyCode::End)),
                ("file099", Some(Event::Resize(ResizeEvent::new(36, 12)))),
                ("file099", click(7, 3)),
                ("1 selected", None),
            ],
        );
        let calls = calls.lock().unwrap();
        assert_eq!(
            calls[0].1,
            vec![fixture
                .path()
                .canonicalize()
                .unwrap()
                .join("file099")
                .to_string_lossy()
                .into_owned()]
        );
        assert_eq!(
            calls.last().unwrap().1,
            vec![fixture
                .path()
                .canonicalize()
                .unwrap()
                .join("file096")
                .to_string_lossy()
                .into_owned()]
        );
        assert!(frames.iter().all(|frame| frame.geometry.len() < 40));
    }
}

#[test]
fn explorer_disabled_empty_and_no_selection_modes_are_inert_where_requested() {
    for size in [(24, 10), (48, 14)] {
        let fixture = tempfile::tempdir().unwrap();
        let element = builder::file_explorer()
            .root_path(fixture.path())
            .build()
            .auto_focus();
        let frames = run_when(
            Control(element),
            size,
            vec![
                ("No matching files", key(KeyCode::Enter)),
                ("No matching files", None),
            ],
        );
        assert!(frames.last().unwrap().text.contains("0 files"));
        std::fs::write(fixture.path().join("entry"), "data").unwrap();
        for disabled in [false, true] {
            let calls = Arc::default();
            let mut element = builder::file_explorer()
                .root_path(fixture.path())
                .show_details(false)
                .selection_mode(SelectionMode::None)
                .on_select("selected")
                .on_activate("activated")
                .build()
                .auto_focus();
            if disabled {
                element = element.disabled(true);
            }
            run_when(
                Notifications {
                    element,
                    calls: Arc::clone(&calls),
                },
                size,
                vec![
                    ("entry", click(7, 3)),
                    ("0 selected", key(KeyCode::Enter)),
                    ("0 selected", None),
                ],
            );
            assert_eq!(calls.lock().unwrap().len(), usize::from(!disabled));
        }
    }
}

#[test]
fn explorer_all_sort_criteria_and_orders_use_real_metadata() {
    use reactive_tui::widgets::display::{SortCriteria, SortOrder};
    use std::time::{Duration, SystemTime};
    let fixture = tempfile::tempdir().unwrap();
    for (name, bytes, seconds) in [("a.rs", "a", 3), ("b.txt", "bbbb", 1), ("c.md", "ccc", 2)] {
        let path = fixture.path().join(name);
        std::fs::write(&path, bytes).unwrap();
        std::fs::File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(946_684_800 + seconds))
            .unwrap();
    }
    for size in [(24, 10), (48, 14)] {
        for (criteria, expected) in [
            (SortCriteria::Name, ["a.rs", "b.txt", "c.md"]),
            (SortCriteria::Size, ["a.rs", "c.md", "b.txt"]),
            (SortCriteria::Modified, ["b.txt", "c.md", "a.rs"]),
            (SortCriteria::Type, ["c.md", "a.rs", "b.txt"]),
        ] {
            for descending in [false, true] {
                let element = builder::file_explorer()
                    .root_path(fixture.path())
                    .show_details(false)
                    .sort_by(criteria.clone())
                    .sort_order(if descending {
                        SortOrder::Descending
                    } else {
                        SortOrder::Ascending
                    })
                    .build();
                let frames = run_when_all(
                    Control(element),
                    size,
                    vec![(&["a.rs", "b.txt", "c.md"], None)],
                );
                let text = &frames.last().unwrap().text;
                let mut expected = expected;
                if descending {
                    expected.reverse();
                }
                let positions: Vec<_> = expected
                    .iter()
                    .map(|name| text.find(name).unwrap())
                    .collect();
                assert!(
                    positions.windows(2).all(|pair| pair[0] < pair[1]),
                    "{criteria:?} descending={descending}: {text}"
                );
            }
        }
    }
    let element = builder::file_explorer()
        .root_path(fixture.path())
        .show_details(true)
        .build();
    let frames = run_when(Control(element), (60, 10), vec![("3 files", None)]);
    let text = &frames.last().unwrap().text;
    assert!(
        text.contains("1 B") && text.contains("2000-01-01 00:00 UTC"),
        "{text}"
    );
}

#[test]
fn explorer_reports_invalid_paths_limits_and_overwrite_errors() {
    let fixture = tempfile::tempdir().unwrap();
    for size in [(24, 10), (48, 14)] {
        let element = builder::file_explorer()
            .root_path(fixture.path())
            .current_path(fixture.path().join("missing"))
            .build();
        let frames = run_when(Control(element), size, vec![("Filesystem:", None)]);
        assert!(!frames.last().unwrap().text.contains("Loading…"));
        let element = builder::file_explorer()
            .root_path(fixture.path())
            .max_visible_items(0)
            .build()
            .auto_focus();
        let frames = run_when(
            Control(element),
            size,
            vec![
                ("max_visible_items", key(KeyCode::Delete)),
                ("max_visible_items", None),
            ],
        );
        assert!(!frames.last().unwrap().text.contains("Delete 1"));
        std::fs::write(fixture.path().join("a-source"), "source").unwrap();
        std::fs::write(fixture.path().join("z-existing"), "existing").unwrap();
        let element = builder::file_explorer()
            .root_path(fixture.path())
            .show_details(false)
            .build()
            .auto_focus();
        let mut steps = vec![("2 files", click(1, 2))];
        for c in "z-existing".chars() {
            steps.push(("Copy:", key(KeyCode::Char(c))));
        }
        steps.extend([("z-existing▏", click(1, 4)), ("Filesystem:", None)]);
        let frames = run_when(Control(element), size, steps);
        assert!(!frames.last().unwrap().text.contains("Copy: 1 completed"));
        assert_eq!(
            std::fs::read(fixture.path().join("a-source")).unwrap(),
            b"source"
        );
        assert_eq!(
            std::fs::read(fixture.path().join("z-existing")).unwrap(),
            b"existing"
        );
    }
}

#[test]
fn explorer_uses_padded_parent_layout_and_handles_resize() {
    use reactive_tui::{component::LayoutType, event::types::ResizeEvent};
    let fixture = tempfile::tempdir().unwrap();
    std::fs::write(fixture.path().join("wide界.txt"), "bytes").unwrap();
    for size in [(28, 12), (48, 16)] {
        let calls = Arc::default();
        let explorer = builder::file_explorer()
            .root_path(fixture.path())
            .show_details(false)
            .on_select("selected")
            .build()
            .auto_focus();
        let element = Element::layout(LayoutType::Flex)
            .with_class("flex-col pl-1 pt-1")
            .with_children(vec![explorer]);
        run_when(
            Notifications {
                element,
                calls: Arc::clone(&calls),
            },
            size,
            vec![
                ("wide界.txt", click(10, 7)),
                ("1 selected", Some(Event::Resize(ResizeEvent::new(40, 14)))),
                ("wide界.txt", key(KeyCode::Enter)),
                ("1 selected", None),
            ],
        );
        assert_eq!(calls.lock().unwrap().len(), 1);
    }
}

#[test]
fn explorer_changed_props_replace_location_but_keep_user_view_and_remeasure_input() {
    use std::sync::atomic::{AtomicBool, Ordering};
    struct Switching {
        root: std::path::PathBuf,
        changed: AtomicBool,
        calls: Calls,
    }
    impl RootComponent for Switching {
        fn render(&self) -> Element {
            builder::file_explorer()
                .root_path(&self.root)
                .current_path(self.root.join(if self.changed.load(Ordering::SeqCst) {
                    "second"
                } else {
                    "first"
                }))
                .keyboard_navigation(!self.changed.load(Ordering::SeqCst))
                .show_details(false)
                .on_select("selected")
                .build()
                .auto_focus()
        }
        fn wake_driven(&self) -> bool {
            true
        }
        fn handle_event(&self, event: &Event) -> reactive_tui::event::router::EventResult {
            if matches!(event, Event::Key(key) if key.code == KeyCode::F(8)) {
                self.changed.store(true, Ordering::SeqCst);
                return reactive_tui::event::router::EventResult::Consumed;
            }
            Notifications {
                element: Element::text(""),
                calls: self.calls.clone(),
            }
            .handle_event(event)
        }
    }
    let fixture = tempfile::tempdir().unwrap();
    for (directory, name) in [("first", "Before"), ("second", "After")] {
        std::fs::create_dir(fixture.path().join(directory)).unwrap();
        std::fs::write(fixture.path().join(directory).join(name), "data").unwrap();
    }
    for size in [(24, 10), (48, 14)] {
        let calls = Arc::default();
        let frames = run_when_all(
            Switching {
                root: fixture.path().to_path_buf(),
                changed: AtomicBool::new(false),
                calls: Arc::clone(&calls),
            },
            size,
            vec![
                (&["Before"], key(KeyCode::Char('v'))),
                (&["Grid", "Before"], key(KeyCode::F(8))),
                (&["Grid", "After"], key(KeyCode::Char(' '))),
                (&["0 selected", "After"], click(7, 3)),
                (&["1 selected", "After"], None),
            ],
        );
        assert!(!frames.last().unwrap().text.contains("Before"));
        assert_eq!(calls.lock().unwrap().len(), 1);
        assert!(calls.lock().unwrap()[0].1[0].ends_with("After"));
    }
}

#[test]
fn explorer_native_callbacks_are_lossless_and_rename_uses_the_native_path() {
    #[cfg(target_os = "linux")]
    use std::os::unix::ffi::OsStringExt;
    #[cfg(windows)]
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    struct Native {
        element: Element,
        payloads: Arc<Mutex<Vec<serde_json::Value>>>,
    }
    impl RootComponent for Native {
        fn render(&self) -> Element {
            self.element.clone()
        }
        fn wake_driven(&self) -> bool {
            true
        }
        fn handle_event(&self, event: &Event) -> reactive_tui::event::router::EventResult {
            if let Event::Custom(event) = event {
                self.payloads
                    .lock()
                    .unwrap()
                    .push(serde_json::from_slice(&event.data).unwrap());
                return reactive_tui::event::router::EventResult::Consumed;
            }
            reactive_tui::event::router::EventResult::Ignored
        }
    }
    for size in [(24, 10), (48, 14)] {
        let fixture = tempfile::tempdir().unwrap();
        #[cfg(target_os = "linux")]
        let name = std::ffi::OsString::from_vec(vec![b'n', 0xff]);
        #[cfg(windows)]
        let name = std::ffi::OsString::from_wide(&[b'n' as u16, 0xd800]);
        #[cfg(not(any(target_os = "linux", windows)))]
        let name = std::ffi::OsString::from("n-café-🦀");
        let source = fixture.path().join(name);
        std::fs::write(&source, "native bytes").unwrap();
        // Filesystems may normalize Unicode spelling. Compare the exact native
        // entry identity rooted at the explorer's canonical root.
        let name = std::fs::read_dir(fixture.path())
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .file_name();
        let source = fixture.path().canonicalize().unwrap().join(name);
        let payloads = Arc::default();
        let element = builder::file_explorer()
            .root_path(fixture.path())
            .show_details(false)
            .on_select("selected")
            .build()
            .auto_focus();
        let mut steps = vec![("1 files", click(7, 3)), ("1 selected", key(KeyCode::F(2)))];
        for c in "renamed".chars() {
            steps.push(("Rename:", key(KeyCode::Char(c))));
        }
        steps.extend([
            ("renamed▏", key(KeyCode::Enter)),
            ("Rename: 1 completed", None),
        ]);
        run_when(
            Native {
                element,
                payloads: Arc::clone(&payloads),
            },
            size,
            steps,
        );
        let payloads = payloads.lock().unwrap();
        assert_eq!(payloads.len(), 1);
        #[cfg(not(windows))]
        let (encoding, expected) = (
            "bytes",
            serde_json::json!(source.as_os_str().as_encoded_bytes()),
        );
        #[cfg(windows)]
        let (encoding, expected) = (
            "utf16",
            serde_json::json!(source.as_os_str().encode_wide().collect::<Vec<_>>()),
        );
        assert_eq!(payloads[0]["encoding"], encoding);
        assert_eq!(payloads[0]["native_paths"][0], expected);
        assert!(!source.exists());
        assert_eq!(
            std::fs::read(fixture.path().join("renamed")).unwrap(),
            b"native bytes"
        );
    }
}

#[test]
fn explorer_convenience_builders_produce_browsable_controls() {
    let fixture = tempfile::tempdir().unwrap();
    for name in ["code.rs", "document.txt", "photo.png"] {
        std::fs::write(fixture.path().join(name), name).unwrap();
    }
    for size in [(24, 10), (48, 14)] {
        for (element, expected) in [
            (builder::simple_file_browser(fixture.path()), "code.rs"),
            (
                builder::code_project_explorer(fixture.path()).build(),
                "code.rs",
            ),
            (
                builder::media_gallery_explorer(fixture.path()).build(),
                "photo.png",
            ),
            (
                builder::document_browser(fixture.path()).build(),
                "document.txt",
            ),
            (
                builder::compact_file_picker(fixture.path()).build(),
                "code.rs",
            ),
            (
                builder::system_file_manager(fixture.path()).build(),
                "code.rs",
            ),
            (
                builder::filtered_file_explorer(fixture.path(), vec!["rs"]),
                "code.rs",
            ),
        ] {
            let frames = run_when(
                Control(element.auto_focus()),
                size,
                vec![(expected, key(KeyCode::Home)), ("1 selected", None)],
            );
            assert!(frames.iter().any(|frame| frame.text.contains(expected)));
        }
        // These routes resolve the real environment and only read names. Their
        // App workflow must still load, rather than paint a component description.
        for element in [
            builder::current_directory_explorer(),
            builder::home_directory_explorer(),
        ] {
            let frames = run_when(Control(element), size, vec![("files ·", None)]);
            assert!(!frames.last().unwrap().text.contains("FileExplorer ("));
        }
    }
}

#[test]
fn explorer_signed_wheels_and_release_events_preserve_selection_until_activation() {
    use reactive_tui::event::types::{
        KeyEventKind, MouseButton, MouseEvent, MouseEventKind, Position, WheelDelta, WheelEvent,
        WheelPhase,
    };
    let fixture = tempfile::tempdir().unwrap();
    for index in 0..20 {
        std::fs::write(fixture.path().join(format!("file{index:03}")), "data").unwrap();
    }
    let wheel = |delta| {
        let mut event = MouseEvent::new(MouseEventKind::Wheel, Position::cell(7, 3));
        event.wheel = Some(WheelEvent {
            delta: WheelDelta::Lines { x: 0.0, y: delta },
            phase: WheelPhase::Changed,
        });
        Some(Event::Mouse(event))
    };
    for size in [(24, 10), (48, 14)] {
        let calls = Arc::default();
        let element = builder::file_explorer()
            .root_path(fixture.path())
            .show_details(false)
            .max_visible_items(3)
            .on_select("selected")
            .build()
            .auto_focus();
        let mut release = KeyEvent::new(KeyCode::End);
        release.kind = KeyEventKind::Release;
        let frames = run_when_all(
            Notifications {
                element,
                calls: Arc::clone(&calls),
            },
            size,
            vec![
                (&["file000", "file002"], wheel(2.0)),
                (&["file002", "file004"], wheel(-2.0)),
                (&["file000", "file002"], Some(Event::Key(release))),
                (
                    &["file000", "0 selected"],
                    Some(Event::Mouse(
                        MouseEvent::new(MouseEventKind::Up, Position::cell(7, 3))
                            .with_button(MouseButton::Left),
                    )),
                ),
                (&["file000", "0 selected"], click(7, 3)),
                (
                    &["1 selected"],
                    Some(Event::Mouse(
                        MouseEvent::new(MouseEventKind::Click, Position::cell(7, 3))
                            .with_button(MouseButton::Left),
                    )),
                ),
                (&["1 selected"], None),
            ],
        );
        assert_eq!(calls.lock().unwrap().len(), 1);
        assert!(calls.lock().unwrap()[0].1[0].ends_with("file000"));
        assert!(!frames.iter().any(|frame| frame.text.contains("file019")));
    }
}

#[test]
fn explorer_typed_construction_and_public_seed_state_render_through_app() {
    use reactive_tui::{
        component::Component,
        widgets::display::{FileEntry, FileExplorer, FileExplorerProps, FileExplorerState},
    };
    let fixture = tempfile::tempdir().unwrap();
    std::fs::write(fixture.path().join("alpha"), "a").unwrap();
    std::fs::write(fixture.path().join("beta"), "b").unwrap();
    let props = FileExplorerProps {
        root_path: fixture.path().to_path_buf(),
        current_path: fixture.path().to_path_buf(),
        show_details: false,
        ..Default::default()
    };
    let seed = FileExplorerState {
        entries: vec![
            FileEntry::from_path(&fixture.path().join("alpha")).unwrap(),
            FileEntry::from_path(&fixture.path().join("beta")).unwrap(),
        ],
        selected_indices: [1].into_iter().collect(),
        focused_index: Some(1),
        initialized: true,
        ..Default::default()
    };
    for size in [(24, 10), (48, 14)] {
        let frames = run_when_all(
            Control(Element::typed::<FileExplorer>(props.clone())),
            size,
            vec![(&["alpha", "beta"], None)],
        );
        assert!(frames.last().unwrap().text.contains("2 files"));
        let frames = run_when_all(
            Control(FileExplorer.render(&props, &seed).auto_focus()),
            size,
            vec![(&["alpha", "beta", "1 selected"], None)],
        );
        assert!(frames
            .last()
            .unwrap()
            .text
            .lines()
            .any(|line| line.contains("beta") && line.contains('*')));
    }
}

#[test]
fn explorer_grid_limits_keep_the_last_item_reachable_after_column_changes() {
    use reactive_tui::{event::types::ResizeEvent, widgets::display::ViewMode};
    let fixture = tempfile::tempdir().unwrap();
    for index in 0..20 {
        std::fs::write(fixture.path().join(format!("file{index:03}")), "data").unwrap();
    }
    for size in [(24, 10), (48, 14)] {
        for limit in [1, 3] {
            let element = builder::file_explorer()
                .root_path(fixture.path())
                .show_details(false)
                .view_mode(ViewMode::Grid)
                .max_visible_items(limit)
                .build()
                .auto_focus();
            let frames = run_when(
                Control(element),
                size,
                vec![
                    ("file000", key(KeyCode::End)),
                    ("file019", Some(Event::Resize(ResizeEvent::new(30, 12)))),
                    ("file019", None),
                ],
            );
            assert!(frames.last().unwrap().text.contains("file019"));
        }
    }
}
