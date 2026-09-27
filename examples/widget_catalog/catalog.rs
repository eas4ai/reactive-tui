use reactive_tui::builder::specialized::WizardStep;
use reactive_tui::{
    app::{RootComponent, RootUpdate},
    builder::{
        action_item, checkbox, confirmation_dialog, context_menu, data_table, div, file_explorer,
        image, menubar, path_breadcrumb, popover, progress_bar, progress_dialog, radio_button,
        scroll_view, select, simple_accordion, slider, stack, tabs, text_input, toast, tree,
        wizard,
    },
    component::Element,
    core::geometry::Rect,
    event::{
        router::EventResult,
        types::{Event, KeyCode, KeyEventKind},
    },
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
        menu::DialogMenuBuilder,
        DialogMenu, TerminalProps, TerminalWidget,
    },
};
use std::path::PathBuf;
use std::time::Instant;
#[path = "motion.rs"]
pub mod motion;
use motion::CubeAnimation;
#[cfg(feature = "wgpu-graphics")]
use reactive_tui::graphics::{GraphicsCanvas, GraphicsOptions, HybridCubeRenderer};

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
    #[cfg(feature = "wgpu-graphics")]
    graphics: Option<GraphicsCanvas>,
    #[cfg(feature = "wgpu-graphics")]
    graphics_options: GraphicsOptions,
    #[cfg(feature = "wgpu-graphics")]
    prepared_graphics: Option<HybridCubeRenderer>,
    #[cfg(feature = "wgpu-graphics")]
    graphics_started: Instant,
    #[cfg(feature = "wgpu-graphics")]
    graphics_error: Option<String>,
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
            graphics: None,
            #[cfg(feature = "wgpu-graphics")]
            graphics_options: GraphicsOptions::default(),
            #[cfg(feature = "wgpu-graphics")]
            prepared_graphics: None,
            #[cfg(feature = "wgpu-graphics")]
            graphics_started: Instant::now(),
            #[cfg(feature = "wgpu-graphics")]
            graphics_error: None,
        }
    }
}

impl Catalog {
    #[cfg(feature = "wgpu-graphics")]
    pub fn with_graphics(options: GraphicsOptions, start_motion: bool) -> Self {
        Self {
            graphics_options: options,
            // Called before SuprTuiBackend::new: driver stderr must not scroll
            // or corrupt a raw alternate-screen session during initialization.
            prepared_graphics: Some(HybridCubeRenderer::new(options)),
            page: if start_motion {
                CatalogPage::Motion
            } else {
                CatalogPage::Overview
            },
            ..Self::default()
        }
    }

    #[cfg(feature = "wgpu-graphics")]
    fn graphics_viewport(&self) -> (u32, u32) {
        let sidebar = self.navigation_layout() == NavigationLayout::Sidebar;
        (
            u32::from(self.width.saturating_sub(if sidebar { 24 } else { 0 })),
            u32::from(self.height.saturating_sub(if sidebar { 6 } else { 9 })),
        )
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
            "w-full grid grid-cols-1 gap-0.25"
        } else if width < 150 {
            "w-full grid grid-cols-2 gap-0.25"
        } else if width < 200 {
            "w-full grid grid-cols-3 gap-0.25"
        } else {
            "w-full grid grid-cols-4 gap-0.25"
        }
    }

    /// Footer hints name only keys that work on the current page: the
    /// F1/F2 demo switcher lives on the Menus & dialogs page.
    fn footer_text(&self) -> &'static str {
        match (self.navigation_layout(), self.page) {
            (NavigationLayout::Compact, CatalogPage::MenusDialogs) => {
                "1–9 page · F2 demo · Ctrl+Q quit"
            }
            (NavigationLayout::Compact, _) => "1–9 page · Ctrl+Q quit",
            (_, CatalogPage::MenusDialogs) => {
                "↑↓/←→ page · 1–9 jump · F1/F2 demo · Tab interact · Ctrl+Q / Ctrl+C / Esc quit"
            }
            (_, _) => "↑↓/←→ page · 1–9 jump · Tab interact · Ctrl+Q / Ctrl+C / Esc quit",
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
                        "bg-cyan-900 text-cyan-200 font-bold"
                    } else {
                        "text-gray-400"
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
                .class("w-24 shrink-0 h-full flex-col border-r border-gray-700 bg-gray-950 p-0.25")
                .children(entries)
                .build(),
            NavigationLayout::Compact => div()
                .class("w-full shrink-0 h-3 flex-row border-b border-gray-700 bg-gray-950 px-0.25")
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
            .class("flex-col min-w-0 shrink-0 border border-gray-700 bg-gray-900 p-0.25 gap-0.25")
            .child(
                div()
                    .class("h-1 shrink-0 text-cyan-300 font-bold")
                    .text(title)
                    .build(),
            )
            .child(body.with_class(body_class))
            .build()
    }

    fn overview_page(&self) -> Element {
        div()
            .class("w-full flex-col gap-0.25")
            .child(
                div()
                    .class("w-full shrink-0 whitespace-normal text-white font-bold")
                    .text("One small app. Every widget family. Built for capture.")
                    .build(),
            )
            .child(
                div()
                    .class("w-full shrink-0 whitespace-normal text-gray-400")
                    .text("Use arrows or the numbered shortcuts to move between focused stages.")
                    .build(),
            )
            .child(Self::card(
                "Coverage",
                div()
                    .class("w-full whitespace-normal text-gray-300")
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
                .class("min-w-0 px-0.25 text-white")
                .class(classes)
                .text(label)
                .build()
        };
        div()
            .class("w-full flex-col gap-0.25")
            .child(Self::card(
                "Column spans · four-column grid",
                div()
                    .class("w-full grid grid-cols-4 gap-0.25")
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
            let mut column = div().class("flex-col gap-0.25");
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

    fn menus_dialogs_page(&self) -> Element {
        let menu_items = vec![
            action_item("new", "New capture", || {}),
            action_item("export", "Export clip", || {}),
        ];
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
            0 => menubar().items(menu_items).title("Catalog").build(),
            1 => context_menu().items(menu_items).build(),
            2 => reactive_tui::builder::popup_menu()
                .items(menu_items)
                .build(),
            3 => Element::typed::<DialogMenu>(
                DialogMenuBuilder::confirmation()
                    .title("DialogMenu")
                    .message("Keep this capture?")
                    .build(),
            ),
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
            .class("flex-col gap-0.25")
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
            .class("w-full flex-col gap-0.25")
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
                    .class("text-gray-400")
                    .text("Tracked local asset · terminal protocol or cell fallback")
                    .build(),
            )
            .build()
    }

    fn motion_page(&self) -> Element {
        div()
            .class("w-full h-full flex-1 flex-col min-h-0")
            .child(
                div()
                    .class("h-1 shrink-0 text-white font-bold")
                    .text("Motion")
                    .build(),
            )
            .child(
                div()
                    .class("h-1 shrink-0 text-cyan-300")
                    .text("Wireframe cube · Braille subpixels · 50 ms")
                    .build(),
            )
            .child(
                div()
                    .class("w-full flex-1 min-h-0 whitespace-pre text-cyan-300")
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
            .class("w-full flex-col gap-0.25")
            .child(Self::card(
                "TerminalWidget",
                Element::typed::<TerminalWidget>(terminal),
            ))
            .child(
                div()
                    .class("text-yellow-300")
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
        #[cfg(feature = "wgpu-graphics")]
        if self.page == CatalogPage::Motion {
            if let Some(graphics) = &self.graphics {
                return div()
                    .class("flex-col flex-1 min-w-0 min-h-0 h-full bg-gray-900")
                    .child(
                        div()
                            .class("h-1 shrink-0 text-white font-bold")
                            .text("Shaded spinning cube")
                            .build(),
                    )
                    .child(
                        div()
                            .class("h-1 shrink-0 text-cyan-300")
                            .text(
                                &self
                                    .graphics_error
                                    .clone()
                                    .unwrap_or_else(|| graphics.mode_label()),
                            )
                            .build(),
                    )
                    .child(
                        graphics
                            .element()
                            .unwrap_or_else(|error| Element::text(error.to_string())),
                    )
                    .build();
            }
        }
        if self.page == CatalogPage::Motion {
            return div()
                .class("flex-col flex-1 min-w-0 min-h-0 h-full bg-gray-900")
                .child(self.motion_page())
                .build();
        }
        let page = self.selected_page();
        let page_class = format!(
            "{} w-full min-w-0 shrink-0 whitespace-normal",
            page.class.as_deref().unwrap_or_default()
        );
        div()
            .class("flex-col flex-1 min-w-0 min-h-0 h-full p-0.25 gap-0.25 bg-black")
            .child(
                div()
                    .class("h-1 shrink-0 text-white font-bold")
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
    #[cfg(feature = "wgpu-graphics")]
    fn attach_waker(&mut self, wake: reactive_tui::app::AppWaker) {
        self.graphics = Some(match self.prepared_graphics.take() {
            Some(renderer) => GraphicsCanvas::with_renderer(wake, renderer),
            None => GraphicsCanvas::new(wake, self.graphics_options),
        });
    }
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
            .class("w-screen h-screen flex-col bg-black text-gray-200")
            .child(
                div()
                    .class(
                        "w-full shrink-0 h-3 flex-row px-0.25 bg-gray-950 border-b border-gray-700",
                    )
                    .child(
                        div()
                            .class("flex-1 text-cyan-300 font-bold")
                            .text("◈ Reactive TUI · Widget Catalog")
                            .build(),
                    )
                    .child(
                        div()
                            .class("text-gray-500")
                            .text(&format!("{}×{}", self.width, self.height))
                            .build(),
                    )
                    .build(),
            )
            .child(content)
            .child(
                div()
                    .class("w-full shrink-0 h-1 px-0.25 bg-gray-950 text-gray-500")
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
        #[cfg(feature = "wgpu-graphics")]
        {
            if self.exit_requested {
                if let Some(graphics) = &mut self.graphics {
                    graphics.shutdown().map_err(|error| {
                        reactive_tui::ReactiveError::invalid_state(error.to_string())
                    })?;
                }
                return Ok(RootUpdate::Exit);
            }
            if self.page == CatalogPage::Motion {
                let (columns, rows) = self.graphics_viewport();
                if let Some(graphics) = &mut self.graphics {
                    return Ok(
                        match graphics.advance(self.graphics_started.elapsed(), columns, rows) {
                            Ok(true) => {
                                self.graphics_error = None;
                                RootUpdate::Redraw
                            }
                            Ok(false) => RootUpdate::Unchanged,
                            Err(error) => {
                                let message = error.to_string();
                                if self.graphics_error.as_ref() == Some(&message) {
                                    RootUpdate::Unchanged
                                } else {
                                    self.graphics_error = Some(message);
                                    RootUpdate::Redraw
                                }
                            }
                        },
                    );
                }
            }
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
