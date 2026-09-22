#![cfg(unix)]

use reactive_tui::core::surface::Surface;
use reactive_tui::core::terminal::Terminal;
use reactive_tui::core::window::Window;
use reactive_tui::event::types::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind, Position};
use std::collections::BTreeSet;
use std::fs;
use std::os::unix::{fs::PermissionsExt, process::CommandExt};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use syn::visit::Visit;

const CHILD_ENV: &str = "RTUI_TERMINAL_HELPER_TEST_CHILD";
const PID_FILE_ENV: &str = "RTUI_TERMINAL_HELPER_PID_FILE";

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct HelperCall {
    file: String,
    function: String,
    program: String,
}

#[derive(Default)]
struct HelperInventory {
    file: String,
    function: Option<String>,
    helpers: BTreeSet<HelperCall>,
    bounded: BTreeSet<(String, String)>,
    unbounded: BTreeSet<(String, String, String)>,
}

impl HelperInventory {
    fn enter_function(&mut self, name: String, visit: impl FnOnce(&mut Self)) {
        let previous = self.function.replace(name);
        visit(self);
        self.function = previous;
    }

    fn function_key(&self) -> Option<(String, String)> {
        Some((self.file.clone(), self.function.clone()?))
    }

    fn visit_function_block(&mut self, name: String, block: &syn::Block) {
        self.enter_function(name, |inventory| {
            syn::visit::visit_block(inventory, block);
        });
    }
}

impl<'ast> Visit<'ast> for HelperInventory {
    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.visit_function_block(node.sig.ident.to_string(), &node.block);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.visit_function_block(node.sig.ident.to_string(), &node.block);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(function) = node.func.as_ref() {
            let segments: Vec<_> = function
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect();
            if segments.ends_with(&["Command".into(), "new".into()]) {
                if let Some(syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(program),
                    ..
                })) = node.args.first()
                {
                    if let Some(function) = &self.function {
                        self.helpers.insert(HelperCall {
                            file: self.file.clone(),
                            function: function.clone(),
                            program: program.value(),
                        });
                    }
                }
            }
            if segments.ends_with(&["owned_process".into(), "run".into()]) {
                if let Some(key) = self.function_key() {
                    self.bounded.insert(key);
                }
            }
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let method = node.method.to_string();
        if matches!(method.as_str(), "output" | "status" | "spawn") {
            if let Some((file, function)) = self.function_key() {
                self.unbounded.insert((file, function, method));
            }
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}

fn audit_sources(sources: &[(&str, &str)]) -> Result<(), String> {
    let mut inventory = HelperInventory::default();
    for (file, source) in sources {
        inventory.file = (*file).to_owned();
        let syntax = syn::parse_file(source).map_err(|error| format!("{file}: {error}"))?;
        inventory.visit_file(&syntax);
    }

    let expected = BTreeSet::from([
        HelperCall {
            file: "terminal.rs".into(),
            function: "has_external_tool".into(),
            program: "which".into(),
        },
        HelperCall {
            file: "window.rs".into(),
            function: "get_character_dimensions".into(),
            program: "stty".into(),
        },
    ]);
    if inventory.helpers != expected {
        return Err(format!(
            "terminal helper inventory changed: expected {expected:?}, found {:?}",
            inventory.helpers
        ));
    }
    for helper in &inventory.helpers {
        let key = (helper.file.clone(), helper.function.clone());
        if !inventory.bounded.contains(&key) {
            return Err(format!(
                "{}::{} does not use owned_process::run",
                helper.file, helper.function
            ));
        }
        if let Some((_, _, method)) = inventory
            .unbounded
            .iter()
            .find(|(file, function, _)| (file, function) == (&key.0, &key.1))
        {
            return Err(format!(
                "{}::{} still calls Command::{method}",
                helper.file, helper.function
            ));
        }
    }
    Ok(())
}

#[test]
fn validator_rejects_safe_violating_fixtures() {
    let unbounded = r#"
        impl Terminal {
            fn has_external_tool() { let _ = Command::new("which").output(); }
        }
        impl Window {
            fn get_character_dimensions() {
                crate::core::owned_process::run(Command::new("stty"));
            }
        }
    "#;
    assert!(audit_sources(&[("terminal.rs", unbounded), ("window.rs", "")]).is_err());

    let direct_child_only = r#"
        impl Terminal {
            fn has_external_tool() {
                let mut child = Command::new("which").spawn().unwrap();
                child.kill();
            }
        }
        impl Window {
            fn get_character_dimensions() {
                crate::core::owned_process::run(Command::new("stty"));
            }
        }
    "#;
    assert!(audit_sources(&[("terminal.rs", direct_child_only), ("window.rs", "")]).is_err());

    let skipped_helper = r#"
        impl Window {
            fn get_character_dimensions() {
                crate::core::owned_process::run(Command::new("stty"));
            }
        }
    "#;
    assert!(audit_sources(&[("terminal.rs", skipped_helper), ("window.rs", "")]).is_err());
}

#[test]
fn terminal_helper_inventory_requires_the_owned_runner() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let terminal = fs::read_to_string(root.join("src/core/terminal.rs")).unwrap();
    let window = fs::read_to_string(root.join("src/core/window.rs")).unwrap();
    assert_eq!(
        audit_sources(&[("terminal.rs", &terminal), ("window.rs", &window)]),
        Ok(()),
        "the checked-in terminal helper inventory matches the expected set"
    );
}

struct Fixture {
    directory: PathBuf,
    child: Option<Child>,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if let Ok(entries) = fs::read_to_string(self.directory.join("pids")) {
            for pid in entries.lines().filter_map(parse_pid) {
                unsafe {
                    libc::kill(-pid, libc::SIGKILL);
                    libc::kill(pid, libc::SIGKILL);
                }
            }
        }
        if let Some(child) = &mut self.child {
            unsafe {
                libc::kill(-(child.id() as i32), libc::SIGKILL);
            }
            let _ = child.wait();
        }
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn parse_pid(line: &str) -> Option<i32> {
    line.split_once(':')?.1.trim().parse().ok()
}

fn write_blocking_helper(directory: &Path, helper: &str) {
    let script = "#!/bin/sh\n\
        echo \"direct:$$\" >> \"$RTUI_TERMINAL_HELPER_PID_FILE\"\n\
        /bin/sh -c 'echo \"desc:$$\" >> \"$RTUI_TERMINAL_HELPER_PID_FILE\"; exec /bin/sleep 30' &\n\
        wait\n";
    let path = directory.join(helper);
    fs::write(&path, script).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
}

fn isolated(name: &str, helper: &str) -> bool {
    if std::env::var(CHILD_ENV).as_deref() == Ok(name) {
        return false;
    }
    let directory =
        std::env::temp_dir().join(format!("rtui-terminal-helper-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&directory).unwrap();
    write_blocking_helper(&directory, helper);
    let output_path = directory.join("output");
    let output = fs::File::create(&output_path).unwrap();
    let mut fixture = Fixture {
        directory,
        child: None,
    };
    fixture.child = Some(
        Command::new(std::env::current_exe().unwrap())
            .args(["--exact", name, "--nocapture"])
            .env(CHILD_ENV, name)
            .env(PID_FILE_ENV, fixture.directory.join("pids"))
            .env("PATH", &fixture.directory)
            .env("COLUMNS", "80")
            .env("LINES", "24")
            .stdin(Stdio::null())
            .stdout(output.try_clone().unwrap())
            .stderr(output)
            .process_group(0)
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    let status = loop {
        if let Some(status) = fixture.child.as_mut().unwrap().try_wait().unwrap() {
            break status;
        }
        assert!(
            Instant::now() < deadline,
            "{name}: terminal helper call exceeded outer five-second safety deadline"
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    let captured = fs::read_to_string(&output_path).unwrap();
    assert!(status.success(), "{name}: {captured}");
    assert!(
        captured.contains("1 passed"),
        "fixture did not execute its test: {captured}"
    );
    true
}

fn process_is_running(pid: i32) -> bool {
    if unsafe { libc::kill(pid, 0) } < 0
        && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
    {
        return false;
    }
    let stat = fs::read_to_string(format!("/proc/{pid}/stat"));
    !stat.is_ok_and(|stat| {
        stat.rsplit_once(") ")
            .is_some_and(|(_, rest)| rest.starts_with('Z'))
    })
}

fn assert_helpers_stopped(expected_invocations: usize) {
    let entries = fs::read_to_string(std::env::var(PID_FILE_ENV).unwrap()).unwrap();
    let entries: Vec<_> = entries.lines().collect();
    assert_eq!(entries.len(), expected_invocations * 2, "{entries:?}");
    for entry in entries {
        let (kind, _) = entry.split_once(':').unwrap();
        let pid = parse_pid(entry).unwrap();
        let deadline = Instant::now() + Duration::from_secs(1);
        while process_is_running(pid) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(
            !process_is_running(pid),
            "{kind} helper process {pid} survived"
        );
        if kind == "direct" {
            assert_eq!(
                unsafe { libc::kill(pid, 0) },
                -1,
                "direct helper {pid} was not reaped"
            );
            assert_eq!(
                std::io::Error::last_os_error().raw_os_error(),
                Some(libc::ESRCH)
            );
        }
    }
}

#[test]
fn which_discovery_has_a_deadline_and_reaps_its_process_groups() {
    if isolated(
        "which_discovery_has_a_deadline_and_reaps_its_process_groups",
        "which",
    ) {
        return;
    }
    let started = Instant::now();
    let capabilities = Terminal::detect_image_capabilities();
    assert!(started.elapsed() < Duration::from_secs(2));
    assert!(!capabilities.chafa_available);
    assert!(!capabilities.viu_available);
    assert_helpers_stopped(2);
}

#[test]
fn stty_configuration_has_a_deadline_and_reaps_its_process_group() {
    if isolated(
        "stty_configuration_has_a_deadline_and_reaps_its_process_group",
        "stty",
    ) {
        return;
    }
    let mut surface = Surface::new(4, 4);
    let window = Window::new(&mut surface);
    let event = MouseEvent {
        kind: MouseEventKind::Click,
        button: MouseButton::Left,
        position: Position::Pixel { x: 1, y: 1 },
        modifiers: KeyModifiers::empty(),
        timestamp: Instant::now(),
        wheel: None,
    };
    let started = Instant::now();
    assert!(window.has_mouse(Some(&event)).is_some());
    assert!(started.elapsed() < Duration::from_secs(1));
    assert_helpers_stopped(1);
}
