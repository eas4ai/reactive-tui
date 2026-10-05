//! The code review of 2026-10-04's six high-priority findings, each as its
//! requirement's falsifier (docs/spec/roadmap.md, review-high-findings):
//! SIG-002 (an animation's callbacks run with no lock of it held), FFI-001
//! (a C export that uses its caller's pointer is unsafe on the Rust side),
//! THM-004 (a theme whose variables name each other still resolves) and
//! CHT-040 (a chart's work follows its data, not the slot counts it is told).
//! INP-012 is in tests/review_high_windows.rs and CHT-026's subnormal value in
//! tests/charts_goldens.rs.

mod common;

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// The bytes this test process holds, and the most it has held since a mark:
// CHT-040's memory bound reads them.

struct Counting;

static HELD: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

// SAFETY: every call forwards to the system allocator with the same layout;
// the counters only add and subtract the sizes it hands out and takes back.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            let held = HELD.fetch_add(layout.size(), Ordering::Relaxed) + layout.size();
            PEAK.fetch_max(held, Ordering::Relaxed);
        }
        pointer
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) };
        HELD.fetch_sub(layout.size(), Ordering::Relaxed);
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let moved = unsafe { System.realloc(pointer, layout, size) };
        if !moved.is_null() {
            HELD.fetch_sub(layout.size(), Ordering::Relaxed);
            let held = HELD.fetch_add(size, Ordering::Relaxed) + size;
            PEAK.fetch_max(held, Ordering::Relaxed);
        }
        moved
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

/// Run `work` on its own thread and wait at most `limit` for it.
fn within<T: Send + 'static>(
    limit: Duration,
    work: impl FnOnce() -> T + Send + 'static,
) -> Option<T> {
    let (done, finished) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = done.send(work());
    });
    finished.recv_timeout(limit).ok()
}

// ---------------------------------------------------------------------------
// SIG-002: an animation's callbacks run with none of its own locks held.

mod sig_002 {
    use super::*;
    use reactive_tui::animation::{AnimatedProperty, Animation, AnimationManager, AnimationState};

    /// What an `on_complete` callback does to its animation, through its
    /// shared state: the one way a callback holding `&Animation` can.
    #[derive(Clone, Copy, Debug)]
    enum Then {
        Stop,
        Pause,
        Restart,
    }

    impl Then {
        const ALL: [Then; 3] = [Then::Stop, Then::Pause, Then::Restart];

        /// The state the animation is in after its callback ran.
        fn leaves(self) -> AnimationState {
            match self {
                Then::Stop => AnimationState::Stopped,
                Then::Pause => AnimationState::Paused,
                Then::Restart => AnimationState::Playing,
            }
        }
    }

    /// An animation of 100 ms whose `on_update` reads its own progress,
    /// state and values, and whose `on_complete` reads them too and then
    /// stops, pauses or restarts it: what a callback may do.
    fn reentering(calls: Arc<AtomicUsize>, then: Then) -> Animation {
        let completed = calls.clone();
        let mut animation = Animation::builder("reentering")
            .animate_property(AnimatedProperty::Opacity(0.0, 1.0))
            .duration(Duration::from_millis(100))
            .on_update(move |animation, _| {
                let _ = animation.get_progress();
                let _ = animation.get_state();
                let _ = animation.get_current_values();
                calls.fetch_add(1, Ordering::SeqCst);
            })
            .on_complete(move |animation| {
                let _ = animation.get_progress();
                let _ = animation.get_state();
                let _ = animation.get_current_values();
                if let Ok(mut state) = animation.state.write() {
                    state.state = then.leaves();
                    if let Then::Restart = then {
                        state.current_time = Duration::ZERO;
                        state.progress = 0.0;
                    }
                }
                completed.fetch_add(1000, Ordering::SeqCst);
            })
            .build();
        animation.play();
        animation
    }

    #[test]
    fn sig_002_callbacks_that_read_and_change_their_animation_return_when_updated_directly() {
        for then in Then::ALL {
            let calls = Arc::new(AtomicUsize::new(0));
            let mut animation = reentering(calls.clone(), then);
            let returned = within(Duration::from_secs(5), move || {
                // A frame inside the animation, then one past its end.
                animation.update(Duration::from_millis(16));
                animation.update(Duration::from_millis(200));
                animation.get_state()
            });
            assert!(
                returned.is_some(),
                "SIG-002: an update whose callbacks read their own animation and {then:?} it did not return within 5 seconds"
            );
            assert!(
                calls.load(Ordering::SeqCst) >= 1001,
                "the update and completion callbacks both ran ({then:?}: {})",
                calls.load(Ordering::SeqCst)
            );
            assert_eq!(returned, Some(then.leaves()), "{then:?}");
        }
    }

    #[test]
    fn sig_002_callbacks_that_read_and_change_their_animation_return_when_a_manager_updates_it() {
        for then in Then::ALL {
            let calls = Arc::new(AtomicUsize::new(0));
            let animation = reentering(calls.clone(), then);
            let returned = within(Duration::from_secs(5), move || {
                let mut manager = AnimationManager::new();
                manager.add_animation(animation);
                manager.update();
                std::thread::sleep(Duration::from_millis(150));
                manager.update();
                manager.active_count()
            });
            assert!(
                returned.is_some(),
                "SIG-002: a manager update whose animation's callbacks read it and {then:?} it did not return within 5 seconds"
            );
            assert!(
                calls.load(Ordering::SeqCst) >= 1001,
                "the update and completion callbacks both ran ({then:?}: {})",
                calls.load(Ordering::SeqCst)
            );
        }
    }
}

// ---------------------------------------------------------------------------
// FFI-001: a C export that uses its caller's pointer beyond a null check is
// unsafe on the Rust side and documents what its caller must guarantee.

mod ffi_001 {
    use std::path::Path;
    use syn::visit::Visit;

    /// The raw-pointer parameter names of a function.
    fn pointer_parameters(signature: &syn::Signature) -> Vec<String> {
        signature
            .inputs
            .iter()
            .filter_map(|input| match input {
                syn::FnArg::Typed(typed) => match (&*typed.pat, &*typed.ty) {
                    (syn::Pat::Ident(name), syn::Type::Ptr(_)) => Some(name.ident.to_string()),
                    _ => None,
                },
                syn::FnArg::Receiver(_) => None,
            })
            .collect()
    }

    /// Every use of `name` in a function body, and those that only compare
    /// it with null: `name.is_null()`, `name == ptr::null()`.
    #[derive(Default)]
    struct Uses {
        name: String,
        all: usize,
        null_checks: usize,
    }

    fn is_name(expression: &syn::Expr, name: &str) -> bool {
        match expression {
            syn::Expr::Path(path) => path.qself.is_none() && path.path.is_ident(name),
            syn::Expr::Paren(inner) => is_name(&inner.expr, name),
            _ => false,
        }
    }

    fn is_null_call(expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Call(call) => matches!(&*call.func, syn::Expr::Path(path)
            if path.path.segments.last().is_some_and(|segment| {
                segment.ident == "null" || segment.ident == "null_mut"
            })),
            _ => false,
        }
    }

    impl<'ast> Visit<'ast> for Uses {
        fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
            if path.qself.is_none() && path.path.is_ident(&self.name) {
                self.all += 1;
            }
            syn::visit::visit_expr_path(self, path);
        }
        fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
            if call.method == "is_null" && is_name(&call.receiver, &self.name) {
                self.null_checks += 1;
            }
            syn::visit::visit_expr_method_call(self, call);
        }
        fn visit_expr_binary(&mut self, binary: &'ast syn::ExprBinary) {
            if matches!(binary.op, syn::BinOp::Eq(_) | syn::BinOp::Ne(_))
                && ((is_name(&binary.left, &self.name) && is_null_call(&binary.right))
                    || (is_name(&binary.right, &self.name) && is_null_call(&binary.left)))
            {
                self.null_checks += 1;
            }
            syn::visit::visit_expr_binary(self, binary);
        }
        fn visit_macro(&mut self, mac: &'ast syn::Macro) {
            // A macro's arguments are tokens to syn: a pointer named in them
            // counts as a use, whatever the macro does with it.
            let tokens = mac.tokens.to_string();
            if tokens
                .split(|c: char| !(c.is_alphanumeric() || c == '_'))
                .any(|word| word == self.name)
            {
                self.all += 1;
            }
        }
    }

    fn documented_safety(attributes: &[syn::Attribute]) -> bool {
        attributes.iter().any(|attribute| {
            attribute.path().is_ident("doc")
                && matches!(&attribute.meta, syn::Meta::NameValue(value)
                    if matches!(&value.value, syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(text), .. })
                        if text.value().trim_start().starts_with("# Safety")))
        })
    }

    /// The exports in `sources` that break FFI-001, with why.
    #[derive(Default)]
    struct Exports {
        file: String,
        problems: Vec<String>,
        checked: usize,
    }

    impl Exports {
        fn check(
            &mut self,
            attributes: &[syn::Attribute],
            visibility: &syn::Visibility,
            signature: &syn::Signature,
            block: &syn::Block,
        ) {
            let exported = matches!(visibility, syn::Visibility::Public(_))
                && signature
                    .abi
                    .as_ref()
                    .is_some_and(|abi| abi.name.as_ref().is_some_and(|name| name.value() == "C"));
            if !exported {
                return;
            }
            self.checked += 1;
            let pointers = pointer_parameters(signature);
            let name = &signature.ident;
            if signature.unsafety.is_some() {
                if !pointers.is_empty() && !documented_safety(attributes) {
                    self.problems.push(format!(
                        "{}: unsafe export {name} takes a pointer and has no `# Safety` section",
                        self.file
                    ));
                }
                return;
            }
            for pointer in pointers {
                let mut uses = Uses {
                    name: pointer.clone(),
                    ..Uses::default()
                };
                uses.visit_block(block);
                if uses.all > uses.null_checks {
                    self.problems.push(format!(
                        "{}: safe export {name} uses its pointer `{pointer}` for more than a null check",
                        self.file
                    ));
                }
            }
        }
    }

    impl<'ast> Visit<'ast> for Exports {
        fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
            self.check(&item.attrs, &item.vis, &item.sig, &item.block);
            syn::visit::visit_item_fn(self, item);
        }
        fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
            self.check(&item.attrs, &item.vis, &item.sig, &item.block);
            syn::visit::visit_impl_item_fn(self, item);
        }
    }

    fn audit(sources: &[(String, String)]) -> Result<(usize, Vec<String>), String> {
        let mut exports = Exports::default();
        for (file, source) in sources {
            exports.file = file.clone();
            let syntax = syn::parse_file(source).map_err(|error| format!("{file}: {error}"))?;
            exports.visit_file(&syntax);
        }
        Ok((exports.checked, exports.problems))
    }

    fn rust_files(directory: &Path, found: &mut Vec<(String, String)>) {
        for entry in std::fs::read_dir(directory).expect("a source directory") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                rust_files(&path, found);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                let text = std::fs::read_to_string(&path).expect("a source file");
                let root = Path::new(env!("CARGO_MANIFEST_DIR"));
                let name = path
                    .strip_prefix(root)
                    .unwrap_or(&path)
                    .display()
                    .to_string();
                found.push((name, text));
            }
        }
    }

    #[test]
    fn ffi_001_the_audit_finds_a_safe_export_that_uses_its_callers_pointer() {
        let violating = r#"
            #[no_mangle]
            pub extern "C" fn reads(handle: *mut u32) -> u32 {
                if handle.is_null() { return 0; }
                catch(|| unsafe { *handle })
            }
            /// Frees the handle.
            #[no_mangle]
            pub unsafe extern "C" fn undocumented(handle: *mut u32) { drop(handle); }
            #[no_mangle]
            pub extern "C" fn checks_only(handle: *const u8) -> bool { handle.is_null() }
            /// Reads a handle.
            ///
            /// # Safety
            ///
            /// `handle` is a live handle the library returned.
            #[no_mangle]
            pub unsafe extern "C" fn documented(handle: *const u8) -> u8 { unsafe { *handle } }
        "#;
        let (checked, problems) = audit(&[("fixture.rs".into(), violating.into())]).unwrap();
        assert_eq!(checked, 4);
        assert_eq!(
            problems,
            vec![
                "fixture.rs: safe export reads uses its pointer `handle` for more than a null check".to_owned(),
                "fixture.rs: unsafe export undocumented takes a pointer and has no `# Safety` section".to_owned(),
            ],
            "the audit finds the safe export that reads through its pointer and the undocumented unsafe one, and passes the null check and the documented export"
        );
    }

    #[test]
    fn ffi_001_every_c_export_that_uses_its_callers_pointer_is_unsafe_and_documented() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ffi");
        let mut sources = Vec::new();
        rust_files(&root, &mut sources);
        let (checked, problems) = audit(&sources).expect("src/ffi parses");
        for problem in &problems {
            eprintln!("FFI-001 problem: {problem}");
        }
        assert!(checked >= 200, "the audit read the exports ({checked})");
        assert!(
            problems.is_empty(),
            "FFI-001: {} of {checked} C exports break the rule, for example: {}",
            problems.len(),
            problems
                .iter()
                .take(8)
                .cloned()
                .collect::<Vec<_>>()
                .join("; ")
        );
    }
}

// ---------------------------------------------------------------------------
// THM-004: resolving a color ends for every theme. A theme whose variables
// name each other overflowed the stack, which ends the process: the
// resolution runs in a child process of this test binary.

mod thm_004 {
    use reactive_tui::theme::{Theme, ThemeVariables};
    use std::time::{Duration, Instant};

    /// Resolves through the cycles, the chains and the extended theme and
    /// asserts what each gives; a stack overflow here ends only this child
    /// process, which the test below reads. Prints `THM004 done` once every
    /// assertion has held.
    #[test]
    #[ignore = "run by thm_004_variables_that_name_each_other_resolve_as_undefined"]
    fn thm_004_child() {
        let theme = Theme::new("cycles").with_variables(
            ThemeVariables::new()
                .set("--color-a", "b")
                .set("--color-b", "a")
                .set("--color-input", "hover")
                .set("--color-hover", "input")
                .set("--color-surface", "#123456"),
        );
        // A cycle that runs through a fallback (selection falls back to
        // primary), and two roles that share one alias, which is no cycle.
        let through = Theme::new("through-fallback").with_variables(
            ThemeVariables::new()
                .set("--color-primary", "selection")
                .set("--color-selection", "primary"),
        );
        let shared = Theme::new("shared-alias").with_variables(
            ThemeVariables::new()
                .set("--color-neutral", "#abcdef")
                .set("--color-surface", "neutral")
                .set("--color-foreground", "neutral"),
        );
        // Chains of 32 and of 33 names, the last naming a color: the first
        // passes no more than 32 names, the second passes 32.
        let chain = |names: usize| {
            let mut variables = ThemeVariables::new();
            for at in 0..names {
                let value = if at + 1 == names {
                    "#0a0b0c".to_owned()
                } else {
                    format!("c{}", at + 1)
                };
                variables = variables.set(format!("--color-c{at}"), value);
            }
            Theme::new("chain").with_variables(variables)
        };
        let (long32, long33) = (chain(32), chain(33));
        // A cycle split between a theme and the theme it extends.
        let base = Theme::new("base").with_variables(
            ThemeVariables::new()
                .set("--color-e", "f")
                .set("--color-input", "hover")
                .set("--color-surface", "#654321"),
        );
        let extending = Theme::new("extending")
            .with_variables(
                ThemeVariables::new()
                    .set("--color-f", "e")
                    .set("--color-hover", "input"),
            )
            .extend(base);
        let started = Instant::now();
        assert_eq!(
            theme.resolve_color("a"),
            None,
            "THM-004: a name in a cycle that is not a role resolves to nothing"
        );
        assert_eq!(
            theme.resolve_color("input"),
            theme.resolve_color("surface"),
            "THM-004: the role `input` in a cycle takes its fallback, its surface"
        );
        assert!(
            through.resolve_color("selection").is_some(),
            "THM-004: a cycle through a fallback still resolves the role"
        );
        assert_eq!(
            shared.resolve_color("hover"),
            shared.resolve_color("neutral"),
            "two roles sharing one alias are no cycle: hover mixes surface and foreground, both the alias"
        );
        assert!(
            long32.resolve_color("c0").is_some(),
            "THM-004: a chain of 32 names resolves to its color"
        );
        assert_eq!(
            long33.resolve_color("c0"),
            None,
            "THM-004: a chain that passes 32 names resolves as undefined"
        );
        assert_eq!(
            extending.resolve_color("e"),
            None,
            "THM-004: a cycle split between a theme and the theme it extends resolves to nothing"
        );
        let extending_surface = extending.resolve_color("surface");
        assert!(extending_surface.is_some(), "the extended theme's surface");
        assert_eq!(
            extending.resolve_color("input"),
            extending_surface,
            "THM-004: the role `input` in a cycle through an extended theme takes its fallback, its surface"
        );
        let elapsed = started.elapsed();
        assert!(
            elapsed < Duration::from_secs(1),
            "THM-004: resolving took {elapsed:?}"
        );
        println!("THM004 done");
    }

    #[test]
    fn thm_004_variables_that_name_each_other_resolve_as_undefined() {
        let mut child =
            std::process::Command::new(std::env::current_exe().expect("the test binary"))
                .args([
                    "--exact",
                    "thm_004::thm_004_child",
                    "--ignored",
                    "--nocapture",
                ])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .expect("the test binary runs");
        let deadline = Instant::now() + Duration::from_secs(10);
        while child.try_wait().expect("the child's status").is_none() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
        }
        if child.try_wait().expect("the child's status").is_none() {
            let _ = child.kill();
            panic!(
                "THM-004: resolving variables that name each other did not return within 10 seconds"
            );
        }
        let output = child.wait_with_output().expect("the child's output");
        let said = String::from_utf8_lossy(&output.stdout).into_owned()
            + &String::from_utf8_lossy(&output.stderr);
        assert!(
            output.status.success(),
            "THM-004: resolving variables that name each other failed in the child ({}): {}",
            output.status,
            said.lines()
                .filter(|line| line.contains("overflow")
                    || line.contains("THM-004")
                    || line.contains("panicked")
                    || line.contains("left:")
                    || line.contains("right:"))
                .collect::<Vec<_>>()
                .join(" / ")
        );
        assert!(
            said.lines().any(|line| line == "THM004 done"),
            "the child ran its test to the end: {said}"
        );
    }
}

// ---------------------------------------------------------------------------
// CHT-040: a chart's work and memory follow its data and its plot's cells.

mod cht_040 {
    use super::*;
    use crate::common::app_input;
    use reactive_tui::app::RootComponent;
    use reactive_tui::component::Element;
    use reactive_tui::widgets::display::{Chart, ChartProps, ChartsBuilder, DataPoint, DataSeries};

    struct Root(Element);
    impl RootComponent for Root {
        fn render(&self) -> Element {
            self.0.clone()
        }
    }

    const SIZE: (u16, u16) = (80, 24);

    /// A chart, built where the test measures it: `build()` runs a tick
    /// format over the ticks it lays out.
    type Build = Box<dyn FnOnce() -> ChartProps + Send>;

    fn one_value(builder: ChartsBuilder) -> Build {
        Box::new(move || {
            builder
                .size(SIZE.0, SIZE.1)
                .series(DataSeries::new("only", vec![DataPoint::new(5.0)]))
                .build()
        })
    }

    /// The charts CHT-040 names, each asking for `count` slots or columns,
    /// and the charts asking for `count` ticks, which its rule covers too:
    /// the value ticks with a tick format, which `build()` runs per tick.
    fn charts(count: usize) -> Vec<(&'static str, Build)> {
        vec![
            (
                "a bar chart's band_count",
                one_value(ChartsBuilder::bar().band_count(count)),
            ),
            (
                "a line chart's point_count",
                one_value(ChartsBuilder::line().point_count(count)),
            ),
            (
                "a bar chart's grid_columns",
                one_value(ChartsBuilder::bar().grid_columns(count)),
            ),
            (
                "a bar chart's value_tick_count with a tick format",
                one_value(
                    ChartsBuilder::bar()
                        .value_tick_count(count)
                        .value_tick_format(|value| format!("{value:.1}")),
                ),
            ),
            (
                "a line chart's y_tick_count with a tick format",
                one_value(
                    ChartsBuilder::line()
                        .y_tick_count(count)
                        .y_tick_format(|value| format!("{value:.1}")),
                ),
            ),
            (
                "a line chart's x_tick_count",
                one_value(ChartsBuilder::line().x_tick_count(count)),
            ),
            (
                "a bar chart's band_tick_count",
                one_value(ChartsBuilder::bar().band_tick_count(count)),
            ),
        ]
    }

    /// Build the chart and draw it until it has painted settled frames: the
    /// last frame's text and how long both took, or why they did not finish.
    fn draw(build: Build) -> Result<(String, Duration), String> {
        let started = Instant::now();
        let frames = within(Duration::from_secs(60), move || {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let props = build();
                app_input::run(Root(Element::typed::<Chart>(props)), SIZE, vec![(3, None)])
            }))
            .map_err(|_| {
                "the chart did not build, or the App did not paint three settled frames".to_owned()
            })
        })
        .ok_or_else(|| "the chart did not finish within 60 seconds".to_owned())??;
        let last = frames
            .last()
            .map(|frame| frame.text.clone())
            .unwrap_or_default();
        Ok((last, started.elapsed()))
    }

    /// Whether the plot shows a mark: a braille dot or a block glyph.
    fn shows_a_mark(text: &str) -> bool {
        text.chars().any(|glyph| {
            ('\u{2801}'..='\u{28ff}').contains(&glyph) || ('\u{2580}'..='\u{259f}').contains(&glyph)
        })
    }

    #[test]
    fn cht_040_ten_million_slots_columns_or_ticks_cost_what_one_value_costs() {
        for (what, build) in charts(10_000_000) {
            let mark = HELD.load(Ordering::SeqCst);
            PEAK.store(mark, Ordering::SeqCst);
            let drawn = draw(build);
            let grew = PEAK.load(Ordering::SeqCst).saturating_sub(mark);
            let (_, took) =
                drawn.unwrap_or_else(|why| panic!("CHT-040: {what} of 10,000,000: {why}"));
            assert!(
                grew <= 64 * 1024 * 1024,
                "CHT-040: {what} of 10,000,000 with one value took {} MiB",
                grew / (1024 * 1024)
            );
            assert!(
                took <= Duration::from_secs(2),
                "CHT-040: {what} of 10,000,000 with one value took {took:?} to draw"
            );
        }
    }

    #[test]
    fn cht_040_a_count_of_usize_max_draws_its_one_value() {
        for (what, build) in charts(usize::MAX) {
            let (text, _) =
                draw(build).unwrap_or_else(|why| panic!("CHT-040: {what} of usize::MAX: {why}"));
            assert!(
                shows_a_mark(&text),
                "CHT-040: {what} of usize::MAX left the plot without its one value:\n{text}"
            );
        }
    }
}
