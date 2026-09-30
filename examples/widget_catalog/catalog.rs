use reactive_tui::builder::specialized::WizardStep;
use reactive_tui::{
    app::{RootComponent, RootUpdate},
    builder::{
        action_item, checkbox, checkbox_item, confirmation_dialog, context_menu, data_table, div,
        file_explorer, image, menu_item, menubar, path_breadcrumb, popover, progress_bar,
        progress_dialog, radio_button, radio_item, scroll_view, select, separator,
        simple_accordion, slider, stack, submenu_item, tabs, text_input, toast, tree, wizard,
    },
    component::Element,
    core::geometry::Rect,
    event::{
        router::EventResult,
        types::{Event, KeyCode, KeyEventKind},
    },
    theme::{self, Theme},
    widgets::{
        dialog::{
            AutocompleteConfig, AutocompleteDialog, AutocompleteDialogOptions, DialogComponent,
            DialogId, DialogTheme, InputDialog, InputDialogOptions,
        },
        display::{
            image::ImageDisplayMode,
            table::{Table, TableColumn, TableProps, TableRow},
            tree::TreeNode,
            AreaChartBuilder, BarChartBuilder, CandlestickChartBuilder, DonutChartBuilder,
            LineChartBuilder, PieChartBuilder, RadarChartBuilder, SankeyChartBuilder, SankeyLink,
            ScatterChartBuilder, SizeClass,
        },
        menu::{DialogMenuBuilder, MenuItem},
        DialogMenu, TerminalProps, TerminalWidget,
    },
};
use std::path::PathBuf;
use std::time::Instant;
#[path = "motion.rs"]
pub mod motion;
use motion::CubeAnimation;
#[cfg(feature = "wgpu-graphics")]
#[path = "scene.rs"]
pub mod scene;
#[cfg(feature = "wgpu-graphics")]
use reactive_tui::graphics::{Canvas, CanvasProps, GraphicsFault, GraphicsOptions, GraphicsWorker};
#[cfg(feature = "wgpu-graphics")]
use std::sync::Arc;

/// The built-in themes, in the order F3 cycles through them.
const PRESETS: [fn() -> Theme; 5] = [
    theme::dark_theme,
    theme::light_theme,
    theme::high_contrast_theme,
    theme::solarized_dark_theme,
    theme::gruvbox_dark_theme,
];

/// What the catalog's command line asks of the canvas, and whether the
/// catalog opens on the Motion page.
#[cfg(feature = "wgpu-graphics")]
pub fn graphics_from(
    args: impl IntoIterator<Item = String>,
) -> Result<(GraphicsOptions, bool), String> {
    let mut options = GraphicsOptions::default();
    let mut start_motion = false;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--motion" => start_motion = true,
            "--cpu" => options.force_cpu = true,
            "--graphics-fault" => {
                options.fault =
                    Some(match args.next().as_deref() {
                        Some("adapter") => GraphicsFault::Adapter,
                        Some("device-loss") => GraphicsFault::DeviceLoss,
                        Some("readback") => GraphicsFault::Readback,
                        Some("software") => GraphicsFault::Software,
                        _ => return Err(
                            "--graphics-fault requires adapter, device-loss, readback or software"
                                .into(),
                        ),
                    });
            }
            _ => return Err(format!("unknown catalog option: {arg}")),
        }
    }
    Ok((options, start_motion))
}

/// The Motion page's canvas: how it draws, the worker that draws it and
/// when the cube began to turn.
#[cfg(feature = "wgpu-graphics")]
struct CanvasStage {
    options: GraphicsOptions,
    worker: Option<Arc<GraphicsWorker>>,
    started: Instant,
}

/// Public widget families represented by the catalog.
pub const WIDGET_FAMILY_INVENTORY: &str = "\
TextInput Checkbox RadioButton Select Slider \
Accordion Breadcrumb ScrollView Stack Tabs \
Chart Table DataTable Tree FileExplorer ProgressBar Modal Popover Image \
MenuBar ContextMenu PopupMenu DialogMenu \
ConfirmationDialog InputDialog AutocompleteDialog ProgressDialog Toast WizardDialog \
TerminalWidget";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationLayout {
    Compact,
    Sidebar,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CatalogPage {
    Overview,
    Input,
    Layout,
    Data,
    Charts,
    MenusDialogs,
    Media,
    Motion,
    System,
}

impl CatalogPage {
    pub const ALL: [Self; 9] = [
        Self::Overview,
        Self::Input,
        Self::Layout,
        Self::Data,
        Self::Charts,
        Self::MenusDialogs,
        Self::Media,
        Self::Motion,
        Self::System,
    ];

    pub const fn title(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Input => "Input widgets",
            Self::Layout => "Layout widgets",
            Self::Data => "Data display",
            Self::Charts => "Charts",
            Self::MenusDialogs => "Menus & dialogs",
            Self::Media => "Media",
            Self::Motion => "Motion",
            Self::System => "System widgets",
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::Overview => 0,
            Self::Input => 1,
            Self::Layout => 2,
            Self::Data => 3,
            Self::Charts => 4,
            Self::MenusDialogs => 5,
            Self::Media => 6,
            Self::Motion => 7,
            Self::System => 8,
        }
    }

    fn offset(self, delta: isize) -> Self {
        let count = Self::ALL.len() as isize;
        let index = (self.index() as isize + delta).rem_euclid(count) as usize;
        Self::ALL[index]
    }
}

pub struct Catalog {
    page: CatalogPage,
    width: u16,
    height: u16,
    motion: CubeAnimation,
    exit_requested: bool,
    demo: usize,
    /// Set when the catalog draws the Motion page on the canvas.
    #[cfg(feature = "wgpu-graphics")]
    canvas: Option<CanvasStage>,
}

impl Default for Catalog {
    fn default() -> Self {
        Self {
            page: CatalogPage::Overview,
            width: 100,
            height: 30,
            motion: CubeAnimation::new(Instant::now()),
            exit_requested: false,
            demo: 0,
            #[cfg(feature = "wgpu-graphics")]
            canvas: None,
        }
    }
}

impl Catalog {
    #[cfg(feature = "wgpu-graphics")]
    pub fn with_graphics(options: GraphicsOptions, start_motion: bool) -> Self {
        // Called before SuprTuiBackend::new: the worker makes its renderer
        // now, so what a graphics driver prints while it starts does not
        // land on the App's screen.
        let worker = GraphicsWorker::spawn(options.clone()).ok().map(Arc::new);
        if let Some(worker) = &worker {
            worker.wait_ready(std::time::Duration::from_secs(10));
        }
        Self {
            canvas: Some(CanvasStage {
                options,
                worker,
                started: Instant::now(),
            }),
            page: if start_motion {
                CatalogPage::Motion
            } else {
                CatalogPage::Overview
            },
            ..Self::default()
        }
    }

    #[cfg(test)]
    pub fn page(&self) -> CatalogPage {
        self.page
    }

    #[cfg(test)]
    pub fn set_page(&mut self, page: CatalogPage) {
        self.page = page;
    }

    #[cfg(test)]
    pub fn demo_element(&self) -> Element {
        self.selected_page()
    }

    pub fn navigation_layout(&self) -> NavigationLayout {
        if self.width < 80 {
            NavigationLayout::Compact
        } else {
            NavigationLayout::Sidebar
        }
    }

    /// Card-grid columns track the measured terminal width so wide
    /// viewports fill instead of stretching two narrow columns.
    fn card_grid_class(width: u16) -> &'static str {
        if width < 80 {
            "w-full grid grid-cols-1 gap-1"
        } else if width < 150 {
            "w-full grid grid-cols-2 gap-1"
        } else if width < 200 {
            "w-full grid grid-cols-3 gap-1"
        } else {
            "w-full grid grid-cols-4 gap-1"
        }
    }

    /// Footer hints name only keys that work on the current page: the
    /// F1/F2 demo switcher lives on the Menus & dialogs page.
    fn footer_text(&self) -> &'static str {
        match (self.navigation_layout(), self.page) {
            (NavigationLayout::Compact, CatalogPage::MenusDialogs) => {
                "1–9 page · F2 demo · F3 theme · Ctrl+Q quit"
            }
            (NavigationLayout::Compact, _) => "1–9 page · F3 theme · Ctrl+Q quit",
            (_, CatalogPage::MenusDialogs) => {
                "↑↓/←→ page · 1–9 jump · F1/F2 demo · F3 theme · Tab interact · Ctrl+Q / Ctrl+C / Esc quit"
            }
            (_, _) => "↑↓/←→ page · 1–9 jump · F3 theme · Tab interact · Ctrl+Q / Ctrl+C / Esc quit",
        }
    }

    fn page_for_number(ch: char) -> Option<CatalogPage> {
        ch.to_digit(10)
            .and_then(|number| number.checked_sub(1))
            .and_then(|index| CatalogPage::ALL.get(index as usize).copied())
    }

    fn navigation(&self) -> Element {
        let compact = self.navigation_layout() == NavigationLayout::Compact;
        let entries = CatalogPage::ALL
            .iter()
            .enumerate()
            .map(|(index, page)| {
                let marker = if *page == self.page { "▶" } else { " " };
                div()
                    .class(if compact {
                        "flex-1 min-w-0 h-1"
                    } else {
                        "w-full shrink-0 h-1"
                    })
                    .class(if *page == self.page {
                        "bg-selection text-selection-foreground font-bold"
                    } else {
                        "text-muted"
                    })
                    .text(&if compact {
                        format!("{marker}[{}]", index + 1)
                    } else {
                        format!("{marker}[{}] {}", index + 1, page.title())
                    })
                    .build()
            })
            .collect::<Vec<_>>();

        match self.navigation_layout() {
            NavigationLayout::Sidebar => div()
                .class("w-24 shrink-0 h-full flex-col bg-surface p-0")
                .children(entries)
                .build(),
            NavigationLayout::Compact => div()
                .class("w-full shrink-0 h-3 flex-row bg-surface px-0")
                .children(entries)
                .build(),
        }
    }

    fn card(title: &str, body: Element) -> Element {
        let body_class = format!(
            "{} min-w-0 whitespace-normal",
            body.class.as_deref().unwrap_or_default()
        );
        div()
            .class("flex-col min-w-0 shrink-0 bg-surface p-1 gap-1")
            .child(
                div()
                    .class("h-1 shrink-0 text-accent font-bold")
                    .text(title)
                    .build(),
            )
            .child(body.with_class(body_class))
            .build()
    }

    fn overview_page(&self) -> Element {
        div()
            .class("w-full flex-col gap-1")
            .child(
                div()
                    .class("w-full shrink-0 whitespace-normal text-foreground font-bold")
                    .text("One small app. Every widget family. Built for capture.")
                    .build(),
            )
            .child(
                div()
                    .class("w-full shrink-0 whitespace-normal text-muted")
                    .text("Use arrows or the numbered shortcuts to move between focused stages.")
                    .build(),
            )
            .child(Self::card(
                "Coverage",
                div()
                    .class("w-full whitespace-normal text-muted")
                    .text(WIDGET_FAMILY_INVENTORY)
                    .build(),
            ))
            .build()
    }

    fn input_page(&self) -> Element {
        div()
            .class(Self::card_grid_class(self.width))
            .child(Self::card(
                "TextInput",
                text_input()
                    .value("Reactive TUI")
                    .placeholder("Type here")
                    .class("w-full")
                    .build(),
            ))
            .child(Self::card(
                "Checkbox",
                checkbox().label("Capture-ready").checked(true).build(),
            ))
            .child(Self::card(
                "RadioButton",
                div()
                    .class("flex-col")
                    .child(
                        radio_button()
                            .group("quality")
                            .value("balanced")
                            .label("Balanced")
                            .checked(true)
                            .build(),
                    )
                    .child(
                        radio_button()
                            .group("quality")
                            .value("high")
                            .label("High detail")
                            .build(),
                    )
                    .build(),
            ))
            .child(Self::card(
                "Select",
                select()
                    .option("cyan", "Cyan")
                    .option("violet", "Violet")
                    .selected("cyan")
                    .build(),
            ))
            .child(Self::card(
                "Slider",
                slider()
                    .label("Intensity")
                    .min(0.0)
                    .max(100.0)
                    .step(5.0)
                    .value(65.0)
                    .build(),
            ))
            .build()
    }

    fn layout_page(&self) -> Element {
        let scroll_content = (1..=10)
            .map(|index| Element::text(format!("Scrollable row {index}")))
            .collect();
        let examples = div()
            .class(Self::card_grid_class(self.width))
            .child(Self::card(
                "Breadcrumb",
                path_breadcrumb("/catalog/layout/widgets"),
            ))
            .child(Self::card(
                "Accordion",
                simple_accordion(vec![
                    ("one", "Focused demo", "One primary state per page"),
                    ("two", "Variants", "Useful alternatives stay nearby"),
                ]),
            ))
            .child(Self::card(
                "Tabs",
                tabs()
                    .tab("Preview", Element::text("Live preview"))
                    .tab("Source", Element::text("Public builder API"))
                    .build(),
            ))
            .child(Self::card(
                "ScrollView",
                scroll_view()
                    .contents(scroll_content)
                    .vertical_scroll(true)
                    .show_scrollbars(true)
                    .class("h-7")
                    .build(),
            ))
            .child(Self::card(
                "Stack",
                stack()
                    .spacing(1.0)
                    .child(Element::text("Layer one"))
                    .child(Element::text("Layer two"))
                    .build(),
            ))
            .build();
        let span_cell = |label: &str, classes: &str| {
            div()
                .class(if self.width < 80 { "h-2" } else { "h-3" })
                .class("min-w-0 px-0 text-foreground")
                .class(classes)
                .text(label)
                .build()
        };
        div()
            .class("w-full flex-col gap-1")
            .child(Self::card(
                "Column spans · four-column grid",
                div()
                    .class("w-full grid grid-cols-4 gap-1")
                    .child(span_cell("span 4", "col-span-4 bg-cyan-700"))
                    .child(span_cell("span 2", "col-span-2 bg-violet-700"))
                    .child(span_cell("span 1", "bg-amber-700"))
                    .child(span_cell("span 1", "bg-emerald-700"))
                    .child(span_cell("span 3", "col-span-3 bg-blue-700"))
                    .child(span_cell("span 1", "bg-rose-700"))
                    .build(),
            ))
            .child(examples)
            .build()
    }

    /// Every chart type at its three size classes: a mini sparkline, a
    /// medium panel and a large layout forced through the builder.
    fn charts_page(&self) -> Element {
        struct Sample {
            label: &'static str,
            value: f64,
            open: f64,
            close: f64,
        }
        fn samples() -> Vec<Sample> {
            let labels = ["mon", "tue", "wed", "thu", "fri", "sat"];
            let values = [2.0, 8.0, 5.0, 9.0, 3.0, 7.0];
            labels
                .iter()
                .zip(values)
                .enumerate()
                .map(|(i, (label, value))| Sample {
                    label,
                    value,
                    open: if i == 0 { value } else { values[i - 1] },
                    close: value,
                })
                .collect()
        }
        let classes = [
            (SizeClass::Mini, 20u16, 5u16),
            (SizeClass::Medium, 60, 12),
            (SizeClass::Large, 60, 14),
        ];
        let stack = |charts: Vec<Element>| {
            let mut column = div().class("flex-col gap-1");
            for chart in charts {
                column = column.child(chart);
            }
            column.build()
        };
        let line = stack(
            classes
                .iter()
                .map(|(class, w, h)| {
                    LineChartBuilder::new(samples())
                        .x(|s| s.label)
                        .y(|s| s.value)
                        .name("value")
                        .natural()
                        .dot()
                        .size(*w, *h)
                        .size_class(*class)
                        .render()
                })
                .collect(),
        );
        let area = stack(
            classes
                .iter()
                .map(|(class, w, h)| {
                    AreaChartBuilder::new(samples())
                        .x(|s| s.label)
                        .y(|s| s.value)
                        .name("value")
                        .fill("chart-2")
                        .step_after()
                        .size(*w, *h)
                        .size_class(*class)
                        .render()
                })
                .collect(),
        );
        let scatter = stack(
            classes
                .iter()
                .map(|(class, w, h)| {
                    ScatterChartBuilder::new(samples())
                        .x(|s| s.label)
                        .y(|s| s.value)
                        .name("value")
                        .size(*w, *h)
                        .size_class(*class)
                        .render()
                })
                .collect(),
        );
        let bar = stack(
            classes
                .iter()
                .map(|(class, w, h)| {
                    BarChartBuilder::new(samples())
                        .band(|s| s.label)
                        .value(|s| s.value)
                        .name("value")
                        .fill("chart-3")
                        .size(*w, *h)
                        .size_class(*class)
                        .render()
                })
                .collect(),
        );
        let candlestick = stack(
            classes
                .iter()
                .map(|(class, w, h)| {
                    CandlestickChartBuilder::new(samples())
                        .x(|s| s.label)
                        .open(|s| s.open)
                        .close(|s| s.close)
                        .high(|s| s.open.max(s.close) + 1.0)
                        .low(|s| s.open.min(s.close) - 1.0)
                        .size(*w, *h)
                        .size_class(*class)
                        .render()
                })
                .collect(),
        );
        let pie = stack(
            classes
                .iter()
                .map(|(class, w, h)| {
                    PieChartBuilder::new(samples())
                        .value(|s| s.value)
                        .label(|s| s.label)
                        .name("value")
                        .size(*w, *h)
                        .size_class(*class)
                        .render()
                })
                .collect(),
        );
        let donut = stack(
            classes
                .iter()
                .map(|(class, w, h)| {
                    DonutChartBuilder::new(samples())
                        .value(|s| s.value)
                        .label(|s| s.label)
                        .name("value")
                        .inner_radius(0.55)
                        .pad_angle(0.06)
                        .size(*w, *h)
                        .size_class(*class)
                        .render()
                })
                .collect(),
        );
        let radar = stack(
            classes
                .iter()
                .map(|(class, w, h)| {
                    RadarChartBuilder::new(samples())
                        .label(|s| s.label)
                        .value(|s| s.open)
                        .name("open")
                        .stroke("chart-1")
                        .value(|s| s.close)
                        .name("close")
                        .stroke("chart-2")
                        .fill("none")
                        .dot()
                        .grid_levels(4)
                        .max_value(10.0)
                        .size(*w, *h)
                        .size_class(*class)
                        .render()
                })
                .collect(),
        );
        // Energy flows: three sources, a power station and three uses, with
        // one flow that skips the station.
        let flows = ["coal", "gas", "solar", "power", "industry", "homes", "loss"];
        let links = [
            (0, 3, 4.0),
            (1, 3, 3.0),
            (2, 3, 2.0),
            (1, 4, 2.0),
            (3, 4, 3.0),
            (3, 5, 5.0),
            (3, 6, 1.0),
        ]
        .map(|(source, target, value)| SankeyLink::new(source, target, value));
        let sankey = stack(
            classes
                .iter()
                .map(|(class, w, h)| {
                    SankeyChartBuilder::new(flows, links)
                        .node_label(|n: &&str| *n)
                        .size(*w, *h)
                        .size_class(*class)
                        .render()
                })
                .collect(),
        );
        div()
            .class(Self::card_grid_class(self.width))
            .child(Self::card("Line chart: mini, medium, large", line))
            .child(Self::card("Area chart: mini, medium, large", area))
            .child(Self::card("Scatter chart: mini, medium, large", scatter))
            .child(Self::card("Bar chart: mini, medium, large", bar))
            .child(Self::card(
                "Candlestick chart: mini, medium, large",
                candlestick,
            ))
            .child(Self::card("Pie chart: mini, medium, large", pie))
            .child(Self::card("Donut chart: mini, medium, large", donut))
            .child(Self::card("Radar chart: mini, medium, large", radar))
            .child(Self::card("Sankey chart: mini, medium, large", sankey))
            .build()
    }

    fn data_page(&self) -> Element {
        let root = TreeNode::new("root", "reactive-tui")
            .expanded(true)
            .add_child(TreeNode::new("src", "src"))
            .add_child(TreeNode::new("manual", "manual"));
        let manual = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("manual");
        div()
            .class(Self::card_grid_class(self.width))
            .child(Self::card(
                "Chart",
                reactive_tui::builder::chart()
                    .line_chart()
                    .title("Frame time")
                    .simple_series("ms", vec![12.0, 9.0, 11.0, 8.0, 10.0])
                    .size(32, 7)
                    .build(),
            ))
            .child(Self::card(
                "Table",
                Table::with_props(TableProps {
                    columns: vec![
                        TableColumn::new("Widget", "widget"),
                        TableColumn::new("State", "state"),
                    ],
                    rows: vec![
                        TableRow::new("input")
                            .with_cell("widget", "Input")
                            .with_cell("state", "Ready"),
                        TableRow::new("layout")
                            .with_cell("widget", "Layout")
                            .with_cell("state", "Ready"),
                    ],
                    sortable: true,
                    ..Default::default()
                }),
            ))
            .child(Self::card(
                "DataTable",
                data_table()
                    .column("Widget", "widget")
                    .column("State", "state")
                    .simple_row(vec![("widget", "Input"), ("state", "Ready")])
                    .simple_row(vec![("widget", "Layout"), ("state", "Ready")])
                    .build(),
            ))
            .child(Self::card(
                "Tree",
                tree().root(root).show_lines(true).show_icons(true).build(),
            ))
            .child(Self::card(
                "FileExplorer",
                file_explorer()
                    .root_path(&manual)
                    .current_path(&manual)
                    .max_visible_items(5)
                    .show_preview(false)
                    .build(),
            ))
            .child(Self::card(
                "ProgressBar",
                progress_bar()
                    .label("Catalog coverage")
                    .value(82.0)
                    .show_percentage(true)
                    .width(28)
                    .build(),
            ))
            .build()
    }

    /// The rows every menu demo shows: an action with a shortcut, a
    /// separator, a checkbox, a radio pair, a disabled row and a submenu.
    fn menu_rows() -> Vec<reactive_tui::builder::widgets::menu::MenuItem> {
        use reactive_tui::builder::widgets::menu::MenuShortcut;
        vec![
            menu_item("new", "New capture")
                .shortcut(MenuShortcut::new("Ctrl+N", vec!["ctrl+n"]))
                .action(|| {})
                .build(),
            action_item("export", "Export clip", || {}),
            separator(),
            checkbox_item("wrap", "Wrap lines", true, |_| {}),
            radio_item("small", "Small", false, "size", || {}),
            radio_item("large", "Large", true, "size", || {}),
            menu_item("locked", "Locked").enabled(false).build(),
            submenu_item(
                "recent",
                "Recent",
                vec![
                    action_item("first", "First capture", || {}),
                    action_item("second", "Second capture", || {}),
                ],
            ),
        ]
    }

    fn menus_dialogs_page(&self) -> Element {
        let menu_items = Self::menu_rows();
        let titles = [
            "MenuBar",
            "ContextMenu",
            "PopupMenu",
            "DialogMenu",
            "Modal",
            "Popover",
            "ConfirmationDialog",
            "InputDialog",
            "AutocompleteDialog",
            "ProgressDialog",
            "Toast",
            "WizardDialog",
        ];
        let index = self.demo % titles.len();
        let body = match index {
            0 => menubar()
                .item(submenu_item("file", "File", menu_items))
                .item(submenu_item(
                    "edit",
                    "Edit",
                    vec![
                        action_item("undo", "Undo", || {}),
                        action_item("redo", "Redo", || {}),
                    ],
                ))
                .item(menu_item("help", "Help").enabled(false).build())
                .title("Catalog")
                .build()
                .auto_focus(),
            // The context menu serves its card: a right click in it, or
            // Shift+F10 while it holds the focus, opens the menu there.
            1 => div()
                .class("relative w-full h-6")
                .child(
                    Element::text("Right click here, or press Shift+F10").with_class("text-muted"),
                )
                .child(
                    context_menu()
                        .items(menu_items)
                        .class("absolute left-0 top-0 w-full h-full")
                        .build()
                        .auto_focus(),
                )
                .build(),
            2 => reactive_tui::builder::popup_menu()
                .items(menu_items)
                .build()
                .auto_focus(),
            3 => {
                let mut dialog = DialogMenuBuilder::confirmation()
                    .title("DialogMenu")
                    .message("Keep this capture?")
                    .items(vec![
                        MenuItem::action("keep", "Keep", || {}),
                        MenuItem::action("discard", "Discard", || {}),
                    ])
                    .default_button(0)
                    .cancel_button(1)
                    .build();
                dialog.visible = true;
                Element::typed::<DialogMenu>(dialog)
            }
            4 => reactive_tui::builder::modal()
                .title("Modal")
                .content(Element::text("Focused overlay · Escape closes"))
                .visible(true)
                .build(),
            5 => popover()
                .trigger(reactive_tui::builder::button().text("Open popover").build())
                .content(Element::text("Popover content"))
                .build(),
            6 => confirmation_dialog()
                .title("ConfirmationDialog")
                .message("Ready to record?")
                .build(),
            7 => InputDialog::new(
                DialogId::from_u32(7),
                InputDialogOptions {
                    title: "InputDialog".into(),
                    prompt: "Capture name".into(),
                    ..Default::default()
                },
            )
            .render(Rect::default(), &DialogTheme::default()),
            8 => AutocompleteDialog::new(
                DialogId::from_u32(8),
                AutocompleteDialogOptions {
                    title: "AutocompleteDialog".into(),
                    prompt: "Find a widget".into(),
                    autocomplete: AutocompleteConfig {
                        min_chars: 0,
                        static_suggestions: vec![
                            "Accordion".into(),
                            "Checkbox".into(),
                            "Slider".into(),
                            "Tabs".into(),
                        ],
                        debounce_delay: std::time::Duration::ZERO,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .render(Rect::default(), &DialogTheme::default()),
            9 => progress_dialog()
                .title("ProgressDialog")
                .message("Rendering")
                .progress(0.64)
                .build(),
            10 => toast()
                .success("Toast: capture saved")
                .duration(4_000)
                .build(),
            _ => wizard()
                .title("WizardDialog")
                .step(WizardStep::new("Compose").content(Element::text("Choose widgets")))
                .step(WizardStep::new("Capture").content(Element::text("Record clip")))
                .build(),
        }
        .with_key(format!("overlay-{}", self.demo));
        div()
            .class("flex-col gap-1")
            .child(Element::text(format!(
                "F1/F2 previous/next demo · {}/{} · {}",
                index + 1,
                titles.len(),
                titles[index]
            )))
            .child(Self::card(titles[index], body))
            .build()
    }

    fn media_page(&self) -> Element {
        let logo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("manual/assets/logo.jpg");
        div()
            .class("w-full flex-col gap-1")
            .child(Self::card(
                "Image · project logo",
                image()
                    .source_file(logo)
                    .display_mode(ImageDisplayMode::Auto)
                    .class("w-full h-16")
                    .build(),
            ))
            .child(
                div()
                    .class("text-muted")
                    .text("Tracked local asset · terminal protocol or cell fallback")
                    .build(),
            )
            .build()
    }

    /// The Motion page: with the canvas, the cube as a scene the canvas
    /// fits to the stage, under the name of the renderer that draws it;
    /// without it, the wireframe cube in braille.
    fn motion_page(&self) -> Element {
        #[cfg(feature = "wgpu-graphics")]
        if let Some(stage) = &self.canvas {
            // The renderer is named once it has drawn, so the line and the
            // first picture appear together.
            let renderer = stage
                .worker
                .as_ref()
                .filter(|worker| worker.stats().rendered > 0)
                .and_then(|worker| worker.mode())
                .map_or_else(|| "Starting graphics…".to_owned(), |mode| mode.label());
            let mut props = CanvasProps::new(Arc::new(scene::cube_scene(stage.started.elapsed())))
                .options(stage.options.clone())
                .view(scene::VIEW.0, scene::VIEW.1)
                .label("Shaded spinning cube");
            if let Some(worker) = &stage.worker {
                props = props.worker(worker.clone());
            }
            return div()
                .class("w-full h-full flex-1 flex-col min-h-0")
                .child(
                    div()
                        .class("h-1 shrink-0 text-foreground font-bold")
                        .text("Shaded spinning cube")
                        .build(),
                )
                .child(
                    div()
                        .class("h-1 shrink-0 text-accent")
                        .text(&renderer)
                        .build(),
                )
                .child(
                    div()
                        .class("w-full flex-1 min-h-0")
                        .child(Element::typed::<Canvas>(props))
                        .build(),
                )
                .build();
        }
        div()
            .class("w-full h-full flex-1 flex-col min-h-0")
            .child(
                div()
                    .class("h-1 shrink-0 text-foreground font-bold")
                    .text("Motion")
                    .build(),
            )
            .child(
                div()
                    .class("h-1 shrink-0 text-accent")
                    .text("Wireframe cube · Braille subpixels · 50 ms")
                    .build(),
            )
            .child(
                div()
                    .class("w-full flex-1 min-h-0 whitespace-pre text-accent")
                    .text(self.motion.frame())
                    .build(),
            )
            .build()
    }

    fn system_page(&self) -> Element {
        let terminal = TerminalProps {
            shell_command: Some("printf 'Reactive TUI terminal widget\\n'; exit".into()),
            auto_focus: false,
            show_scrollbar: false,
            title: "TerminalWidget".into(),
            ..Default::default()
        };
        div()
            .class("w-full flex-col gap-1")
            .child(Self::card(
                "TerminalWidget",
                Element::typed::<TerminalWidget>(terminal),
            ))
            .child(
                div()
                    .class("text-warning")
                    .text("Bounded command only; the Kitty shell crash remains tracked separately.")
                    .build(),
            )
            .build()
    }

    fn selected_page(&self) -> Element {
        match self.page {
            CatalogPage::Overview => self.overview_page(),
            CatalogPage::Input => self.input_page(),
            CatalogPage::Layout => self.layout_page(),
            CatalogPage::Data => self.data_page(),
            CatalogPage::Charts => self.charts_page(),
            CatalogPage::MenusDialogs => self.menus_dialogs_page(),
            CatalogPage::Media => self.media_page(),
            CatalogPage::Motion => self.motion_page(),
            CatalogPage::System => self.system_page(),
        }
    }

    fn stage(&self) -> Element {
        if self.page == CatalogPage::Motion {
            return div()
                .class("flex-col flex-1 min-w-0 min-h-0 h-full bg-surface")
                .child(self.motion_page())
                .build();
        }
        let page = self.selected_page();
        let page_class = format!(
            "{} w-full min-w-0 shrink-0 whitespace-normal",
            page.class.as_deref().unwrap_or_default()
        );
        div()
            .class("flex-col flex-1 min-w-0 min-h-0 h-full p-0 gap-1 bg-background")
            .child(
                div()
                    .class("h-1 shrink-0 text-foreground font-bold")
                    .text(self.page.title())
                    .build(),
            )
            .child(
                reactive_tui::widgets::layout::ScrollViewBuilder::new(page.with_class(page_class))
                    .viewport_size(
                        usize::from(self.width.saturating_sub(if self.width >= 80 {
                            26
                        } else {
                            2
                        })),
                        usize::from(self.height.saturating_sub(if self.width >= 80 {
                            8
                        } else {
                            11
                        })),
                    )
                    .scroll_x(false)
                    .scroll_y(true)
                    .show_scrollbars(true)
                    .render()
                    .with_class("w-full flex-1 min-h-0"),
            )
            .build()
    }
}

impl RootComponent for Catalog {
    fn render(&self) -> Element {
        let content = match self.navigation_layout() {
            NavigationLayout::Sidebar => div()
                .class("flex-row flex-1 min-h-0")
                .child(self.navigation())
                .child(self.stage())
                .build(),
            NavigationLayout::Compact => div()
                .class("flex-col flex-1 min-h-0")
                .child(self.navigation())
                .child(self.stage())
                .build(),
        };
        div()
            .class("w-screen h-screen flex-col bg-background text-foreground")
            .child(
                div()
                    .class("w-full shrink-0 h-3 flex-row px-0 bg-surface")
                    .child(
                        div()
                            .class("flex-1 text-accent font-bold")
                            .text("◈ Reactive TUI · Widget Catalog")
                            .build(),
                    )
                    .child(
                        div()
                            .class("text-muted")
                            .text(&format!(
                                "{}×{} · theme {}",
                                self.width,
                                self.height,
                                Theme::active().name
                            ))
                            .build(),
                    )
                    .build(),
            )
            .child(content)
            .child(
                div()
                    .class("w-full shrink-0 h-1 px-0 bg-surface text-muted")
                    .text(self.footer_text())
                    .build(),
            )
            .build()
    }

    fn resize(&mut self, width: u16, height: u16) -> reactive_tui::Result<()> {
        self.width = width;
        self.height = height;
        self.motion.set_viewport(
            usize::from(width.saturating_sub(if width >= 80 { 24 } else { 0 })),
            usize::from(height.saturating_sub(if width >= 80 { 6 } else { 9 })),
        );
        Ok(())
    }

    fn try_handle_event(&mut self, event: &Event) -> reactive_tui::Result<EventResult> {
        let Event::Key(key) = event else {
            return Ok(EventResult::Ignored);
        };
        if key.kind == KeyEventKind::Release {
            return Ok(EventResult::Ignored);
        }
        if key.code == KeyCode::F(3) {
            // The next built-in theme after the active one; an application
            // theme of another name gives way to the first.
            let active = Theme::active();
            let next = PRESETS
                .iter()
                .position(|preset| preset().name == active.name)
                .map_or(0, |index| (index + 1) % PRESETS.len());
            Theme::set_active(PRESETS[next]());
            return Ok(EventResult::Handled);
        }
        if self.page == CatalogPage::MenusDialogs && matches!(key.code, KeyCode::F(1 | 2)) {
            self.demo = if key.code == KeyCode::F(1) {
                (self.demo + 11) % 12
            } else {
                (self.demo + 1) % 12
            };
            return Ok(EventResult::Handled);
        }
        if key.code == KeyCode::Escape
            || (key.modifiers.ctrl && matches!(key.code, KeyCode::Char('q' | 'c')))
        {
            self.exit_requested = true;
            return Ok(EventResult::Handled);
        }
        let next = match key.code {
            KeyCode::Up | KeyCode::Left => Some(self.page.offset(-1)),
            KeyCode::Down | KeyCode::Right => Some(self.page.offset(1)),
            KeyCode::Char(ch) => Self::page_for_number(ch),
            _ => None,
        };
        if let Some(page) = next {
            self.page = page;
            self.demo = 0;
            Ok(EventResult::Handled)
        } else {
            Ok(EventResult::Ignored)
        }
    }

    fn update(&mut self) -> reactive_tui::Result<RootUpdate> {
        // The cube turns with time: each frame of the Motion page is a new
        // scene, which the canvas's worker draws.
        #[cfg(feature = "wgpu-graphics")]
        if !self.exit_requested && self.page == CatalogPage::Motion && self.canvas.is_some() {
            return Ok(RootUpdate::Redraw);
        }
        Ok(if self.exit_requested {
            RootUpdate::Exit
        } else if self.page == CatalogPage::Motion && self.motion.advance(Instant::now()) {
            RootUpdate::Redraw
        } else {
            RootUpdate::Unchanged
        })
    }
}
