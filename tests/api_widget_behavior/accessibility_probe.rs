//! Real terminal fixture for the isolated Orca acceptance workflow.
use reactive_tui::{
    app::{App, RootComponent, RootUpdate},
    backend::SuprTuiBackend,
    builder,
    component::{Element, LayoutType},
    error::{ReactiveError, Result},
    event::{router::EventResult, types::KeyCode, Event},
    widgets::layout::{
        accordion::{Accordion, AccordionProps, AccordionSection},
        breadcrumb::{Breadcrumb, BreadcrumbProps, BreadcrumbSegment, OverflowStrategy},
    },
};
use std::{
    io::Write,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

#[path = "data_probe.rs"]
mod data_probe;
#[path = "dialog_probe.rs"]
mod dialog_probe;
#[path = "display_probe.rs"]
mod display_probe;
#[path = "menu_probe.rs"]
mod menu_probe;
#[path = "overlay_probe.rs"]
mod overlay_probe;

struct Probe {
    started: Instant,
    changes: Arc<Mutex<Vec<serde_json::Value>>>,
    navigation: Arc<Mutex<Vec<serde_json::Value>>>,
    removed: AtomicBool,
    breadcrumbs_removed: AtomicBool,
    exit: AtomicBool,
    semantic_labels: bool,
    semantic_styles: bool,
    css: AtomicBool,
    controls: AtomicUsize,
    menu_stage: AtomicUsize,
    menu_calls: Arc<Mutex<Vec<String>>>,
    files: tempfile::TempDir,
    terminal_files: tempfile::TempDir,
    css_calls: Arc<AtomicUsize>,
    overlay_open: Arc<AtomicBool>,
    progress_value: Arc<AtomicUsize>,
}

struct Background {
    stop: Arc<AtomicBool>,
    calls: Arc<AtomicUsize>,
}

impl RootComponent for Background {
    fn render(&self) -> Element {
        css_controls(&self.calls, true)
    }
    fn accepts_input(&self) -> bool {
        false
    }
    fn update(&mut self) -> Result<RootUpdate> {
        Ok(if self.stop.load(Ordering::SeqCst) {
            RootUpdate::Exit
        } else {
            RootUpdate::Unchanged
        })
    }
}

fn css_controls(calls: &Arc<AtomicUsize>, semantic_styles: bool) -> Element {
    use reactive_tui::layout::{
        css::focus::{apply_aria_attribute, apply_role},
        style::StyleBuilder,
    };
    let calls = calls.clone();
    let count = calls.load(Ordering::SeqCst);
    let style = apply_aria_attribute(StyleBuilder::new(), "labelledby", "background-label");
    let style = apply_aria_attribute(style, "describedby", "background-description");
    let style = apply_aria_attribute(style, "pressed", if count > 0 { "true" } else { "false" });
    let mut action = builder::button()
        .text("Background action")
        .class(if semantic_styles {
            "p-0 h-1 w-full aria-labelledby aria-describedby tabindex-0"
        } else {
            "p-0 h-1 w-full"
        })
        .on_click(move || {
            calls.fetch_add(1, Ordering::SeqCst);
        })
        .build();
    if semantic_styles {
        action.metadata.styles = Some(Arc::new(apply_role(style, "button").snapshot()));
    }
    Element::layout(LayoutType::Flex)
        .with_class("flex flex-col")
        .with_child(action)
        .with_child(Element::text(format!("Background calls {count}")).class("aria-live-polite"))
        .with_child(
            Element::text("Independent App action")
                .with_accessibility_id("background-label")
                .class("sr-only aria-hidden"),
        )
        .with_child(
            Element::text("CSS description delivered")
                .with_accessibility_id("background-description")
                .class("sr-only aria-hidden"),
        )
        .with_child(Element::text("Must stay hidden from reader").class("aria-hidden"))
        .with_child(Element::text("CSS expanded control").class("role-button aria-expanded"))
        .with_child(
            builder::span()
                .text("Painted CSS heading")
                .styles(apply_aria_attribute(
                    StyleBuilder::new(),
                    "label",
                    "CSS direct label",
                ))
                .class("role-heading aria-label")
                .build(),
        )
        .with_child(
            Element::text("CSS selected option").class("role-option aria-selected aria-checked"),
        )
        .with_child(
            Element::text("CSS restored reader text").class("aria-hidden aria-hidden-false"),
        )
        .with_child(
            Element::fragment()
                .class("sr-only")
                .with_child(Element::text("Reader-only nested detail")),
        )
}

impl RootComponent for Probe {
    fn render(&self) -> Element {
        let stage = self.menu_stage.load(Ordering::SeqCst);
        if stage > 0 {
            return menu_probe::render(stage, &self.menu_calls);
        }
        let control = self.controls.load(Ordering::SeqCst);
        if (18..=24).contains(&control) {
            return dialog_probe::render(control, &self.menu_calls);
        }
        if self.controls.load(Ordering::SeqCst) == 17 {
            return display_probe::terminal(self.terminal_files.path());
        }
        if self.controls.load(Ordering::SeqCst) == 16 {
            return display_probe::image(&self.progress_value);
        }
        if self.controls.load(Ordering::SeqCst) == 15 {
            return display_probe::scroll();
        }
        if self.controls.load(Ordering::SeqCst) == 14 {
            return display_probe::chart();
        }
        if self.controls.load(Ordering::SeqCst) == 13 {
            return display_probe::progress(&self.progress_value);
        }
        if self.controls.load(Ordering::SeqCst) == 12 {
            return overlay_probe::popover();
        }
        if self.controls.load(Ordering::SeqCst) == 11 {
            return overlay_probe::modal(&self.overlay_open);
        }
        if self.controls.load(Ordering::SeqCst) == 10 {
            return data_probe::tabs();
        }
        if self.controls.load(Ordering::SeqCst) == 9 {
            return data_probe::files(self.files.path());
        }
        if self.controls.load(Ordering::SeqCst) == 8 {
            return data_probe::tree();
        }
        if self.controls.load(Ordering::SeqCst) == 7 {
            return data_probe::data_table();
        }
        if self.controls.load(Ordering::SeqCst) == 6 {
            return data_probe::table();
        }
        if self.controls.load(Ordering::SeqCst) == 5 {
            return builder::select()
                .option("email", "Email notices")
                .option("phone", "Phone notices")
                .multiple(true)
                .build()
                .with_accessibility_label("Notice channels")
                .auto_focus();
        }
        if self.controls.load(Ordering::SeqCst) == 4 {
            use reactive_tui::widgets::input::{SelectBuilder, SelectOption};
            return SelectBuilder::new()
                .options(vec![
                    SelectOption::new(1, "Standard shipping"),
                    SelectOption::new(2, "Unavailable shipping").disabled(true),
                    SelectOption::new(3, "Express shipping"),
                ])
                .selected(1)
                .render()
                .with_accessibility_label("Shipping speed")
                .auto_focus();
        }
        if self.controls.load(Ordering::SeqCst) == 3 {
            return Element::layout(LayoutType::Flex)
                .class("flex flex-col")
                .with_child(
                    builder::radio_button()
                        .value("alpha")
                        .label("Named alpha")
                        .group("reader-choice")
                        .checked(true)
                        .build()
                        .with_key("alpha")
                        .auto_focus(),
                )
                .with_child(
                    builder::radio_button()
                        .value("beta")
                        .label("Named beta")
                        .group("reader-choice")
                        .build()
                        .with_key("beta"),
                );
        }
        if self.controls.load(Ordering::SeqCst) == 2 {
            use reactive_tui::widgets::input::RadioButtonBuilder;
            return RadioButtonBuilder::new()
                .option(1, "Standard delivery")
                .disabled_option(2, "Unavailable delivery")
                .option(3, "Express delivery")
                .selected(1)
                .render()
                .with_accessibility_label("Delivery method")
                .auto_focus();
        }
        if self.controls.load(Ordering::SeqCst) == 1 {
            use reactive_tui::widgets::input::{Checkbox, CheckboxProps, Slider, SliderProps};
            return Element::layout(LayoutType::Flex)
                .class("flex flex-col")
                .with_child(
                    Element::typed::<Checkbox>(CheckboxProps {
                        label: Some("Receive updates".into()),
                        indeterminate: true,
                        ..Default::default()
                    })
                    .with_accessibility_label("Notification preference")
                    .auto_focus(),
                )
                .with_child(Element::typed::<Checkbox>(CheckboxProps {
                    label: Some("Locked preference".into()),
                    checked: true,
                    disabled: true,
                    ..Default::default()
                }))
                .with_child(
                    Element::typed::<Slider>(SliderProps {
                        min: 0.0,
                        max: 100.0,
                        value: 25.0,
                        step: 5.0,
                        ..Default::default()
                    })
                    .with_accessibility_label("Playback volume"),
                );
        }
        if self.css.load(Ordering::SeqCst) {
            return css_controls(&self.css_calls, self.semantic_styles);
        }
        if self.removed.load(Ordering::SeqCst) {
            if self.breadcrumbs_removed.load(Ordering::SeqCst) {
                return Element::text("Controls removed");
            }
            return Element::typed::<Breadcrumb>(BreadcrumbProps {
                segments: vec![
                    BreadcrumbSegment::new("home", "Home", "/")
                        .aria_label("Breadcrumb home accessible label"),
                    BreadcrumbSegment::new("locked", "Locked", "/locked")
                        .clickable(false)
                        .aria_label("Breadcrumb unavailable accessible label"),
                    BreadcrumbSegment::new("docs", "Docs", "/docs")
                        .aria_label("Breadcrumb docs accessible label"),
                    BreadcrumbSegment::new("page", "Page", "/docs/page")
                        .current(true)
                        .aria_label("Breadcrumb current accessible label"),
                ],
                overflow_strategy: OverflowStrategy::Wrap,
                compact: true,
                show_home_icon: false,
                on_click: Some("probe-navigation".into()),
                ..Default::default()
            })
            .auto_focus();
        }
        let labels = [
            "Account preferences accessible label",
            "Unavailable preferences accessible label",
            "Privacy preferences accessible label",
        ];
        let mut sections = vec![
            AccordionSection::new("account", "Painted account").content(
                Element::layout(LayoutType::Flex)
                    .with_class("flex flex-col")
                    .with_child(Element::text("Account body visible"))
                    .with_child(
                        builder::text_input()
                            .value("seed")
                            .build()
                            .with_accessibility_label("Nested account entry"),
                    ),
            ),
            AccordionSection::new("locked", "Painted locked").disabled(true),
            AccordionSection::new("privacy", "Painted privacy")
                .content(Element::text("Privacy body visible")),
        ];
        if self.semantic_labels {
            for (section, label) in sections.iter_mut().zip(labels) {
                section.aria_label = Some(label.into());
            }
        }
        Element::typed::<Accordion>(AccordionProps {
            reduced_motion: true,
            on_change: Some("probe-change".into()),
            sections,
            ..Default::default()
        })
        .auto_focus()
    }

    fn handle_event(&self, event: &Event) -> EventResult {
        match event {
            Event::Key(key) if key.code == KeyCode::F(12) => {
                let current = self.controls.load(Ordering::SeqCst);
                self.controls.store(
                    if (18..24).contains(&current)
                        || current == 11
                        || (13..17).contains(&current)
                        || (6..10).contains(&current)
                    {
                        current + 1
                    } else {
                        6
                    },
                    Ordering::SeqCst,
                );
                EventResult::Consumed
            }
            Event::Key(key) if key.code == KeyCode::F(7) => {
                let current = self.controls.load(Ordering::SeqCst);
                self.controls
                    .store(if current == 4 { 5 } else { 4 }, Ordering::SeqCst);
                EventResult::Consumed
            }
            Event::Key(key) if key.code == KeyCode::F(1) => {
                self.controls.store(3, Ordering::SeqCst);
                EventResult::Consumed
            }
            Event::Key(key) if key.code == KeyCode::F(2) => {
                self.controls.store(2, Ordering::SeqCst);
                EventResult::Consumed
            }
            Event::Key(key) if key.code == KeyCode::F(3) => {
                self.controls.store(1, Ordering::SeqCst);
                EventResult::Consumed
            }
            Event::Key(key) if key.code == KeyCode::F(4) => {
                self.controls.store(0, Ordering::SeqCst);
                self.menu_stage.fetch_add(1, Ordering::SeqCst);
                EventResult::Consumed
            }
            Event::Key(key) if key.code == KeyCode::F(5) => {
                self.menu_stage.store(0, Ordering::SeqCst);
                self.css.store(true, Ordering::SeqCst);
                EventResult::Consumed
            }
            Event::Custom(event) if event.name == "probe-navigation" => {
                let mut navigation = self.navigation.lock().unwrap();
                assert!(navigation.len() < 16, "unexpected repeated navigation");
                navigation.push(serde_json::from_slice(&event.data).unwrap());
                EventResult::Consumed
            }
            Event::Custom(event) if event.name == "probe-change" => {
                let mut changes = self.changes.lock().unwrap();
                assert!(changes.len() < 16, "unexpected repeated callbacks");
                changes.push(serde_json::from_slice(&event.data).unwrap());
                EventResult::Consumed
            }
            Event::Key(key) if key.code == KeyCode::F(8) => {
                self.removed.store(true, Ordering::SeqCst);
                EventResult::Consumed
            }
            Event::Key(key) if key.code == KeyCode::F(6) => {
                self.breadcrumbs_removed.store(true, Ordering::SeqCst);
                EventResult::Consumed
            }
            Event::Key(key) if key.code == KeyCode::F(9) => {
                self.exit.store(true, Ordering::SeqCst);
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }

    fn update(&mut self) -> Result<RootUpdate> {
        if self.exit.load(Ordering::SeqCst) {
            Ok(RootUpdate::Exit)
        } else if self.started.elapsed() >= Duration::from_secs(30) {
            Err(ReactiveError::invalid_state(
                "accessibility fixture watchdog expired",
            ))
        } else {
            Ok(RootUpdate::Unchanged)
        }
    }
}

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let mut report = std::fs::File::create(args.next().expect("callback report path"))?;
    let mode = args.next();
    let semantic_labels =
        mode.as_deref() != Some(std::ffi::OsStr::new("--without-semantic-labels"));
    let explicit_reader = mode.as_deref() != Some(std::ffi::OsStr::new("--automatic"));
    let semantic_styles =
        mode.as_deref() != Some(std::ffi::OsStr::new("--without-semantic-styles"));
    let stop = Arc::new(AtomicBool::new(false));
    let calls = Arc::new(AtomicUsize::new(0));
    let background_root = Background {
        stop: stop.clone(),
        calls: calls.clone(),
    };
    let background = std::thread::spawn(move || {
        App::builder()
            .backend(SuprTuiBackend::with_writer(32, 10, std::io::sink())?)
            .root(background_root)
            .screen_reader(explicit_reader)
            .accessibility_name("Independent background App")
            .build()?
            .run()
    });
    let foreground = run_foreground(
        &mut report,
        semantic_labels,
        semantic_styles,
        explicit_reader,
        match mode.as_deref().and_then(|s| s.to_str()) {
            Some("--tabs") => 10,
            Some("--overlays") => 11,
            Some("--display") => 13,
            Some("--dialogs") => 18,
            _ => 0,
        },
    );
    stop.store(true, Ordering::SeqCst);
    let background = background
        .join()
        .map_err(|_| ReactiveError::invalid_state("background App panicked"))?;
    foreground?;
    background?;
    writeln!(
        report,
        "{}",
        serde_json::json!({"background_calls": calls.load(Ordering::SeqCst)})
    )?;
    Ok(())
}

fn run_foreground(
    report: &mut std::fs::File,
    semantic_labels: bool,
    semantic_styles: bool,
    explicit_reader: bool,
    start_control: usize,
) -> Result<()> {
    for round in 1..=2 {
        let changes = Arc::new(Mutex::new(Vec::new()));
        let navigation = Arc::new(Mutex::new(Vec::new()));
        let css_calls = Arc::new(AtomicUsize::new(0));
        let menu_calls = Arc::new(Mutex::new(Vec::new()));
        let app = App::builder()
            .backend(SuprTuiBackend::new()?)
            .root(Probe {
                started: Instant::now(),
                changes: changes.clone(),
                navigation: navigation.clone(),
                removed: AtomicBool::new(false),
                breadcrumbs_removed: AtomicBool::new(false),
                exit: AtomicBool::new(false),
                semantic_labels,
                semantic_styles,
                css: AtomicBool::new(false),
                controls: AtomicUsize::new(start_control),
                menu_stage: AtomicUsize::new(0),
                menu_calls: menu_calls.clone(),
                files: data_probe::files_fixture()?,
                terminal_files: display_probe::terminal_fixture()?,
                css_calls: css_calls.clone(),
                overlay_open: Arc::new(AtomicBool::new(false)),
                progress_value: Arc::new(AtomicUsize::new(50)),
            })
            .accessibility_name(format!("Reactive TUI App accessibility round {round}"));
        let app = if explicit_reader {
            app.screen_reader(true)
        } else {
            app
        };
        app.build()?.run()?;
        writeln!(
            report,
            "{}",
            serde_json::json!({"round": round, "changes": *changes.lock().unwrap(),
                "navigation": *navigation.lock().unwrap(), "menu_calls": *menu_calls.lock().unwrap(), "css_calls": css_calls.load(Ordering::SeqCst)})
        )?;
        report.flush()?;
    }
    Ok(())
}
