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
    MenusDialogs,
    Media,
    Motion,
    System,
}

impl CatalogPage {
    pub const ALL: [Self; 8] = [
        Self::Overview,
        Self::Input,
        Self::Layout,
        Self::Data,
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
            Self::MenusDialogs => 4,
            Self::Media => 5,
            Self::Motion => 6,
            Self::System => 7,
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
        }
    }
}

impl Catalog {
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
                    .class(if *page == self.page {
                        "flex-1 min-w-0 h-1 bg-cyan-900 text-cyan-200 font-bold"
                    } else {
                        "flex-1 min-w-0 h-1 text-gray-400"
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
        div()
            .class("flex-col min-w-0 border border-gray-700 bg-gray-900 p-0.25 gap-0.25")
            .child(
                div()
                    .class("h-1 text-cyan-300 font-bold")
                    .text(title)
                    .build(),
            )
            .child(body)
            .build()
    }

    fn overview_page(&self) -> Element {
        div()
            .class("flex-col gap-1")
            .child(
                div()
                    .class("text-white font-bold")
                    .text("One small app. Every widget family. Built for capture.")
                    .build(),
            )
            .child(
                div()
                    .class("text-gray-400")
                    .text("Use arrows or the numbered shortcuts to move between focused stages.")
                    .build(),
            )
            .child(Self::card(
                "Coverage",
                div()
                    .class("text-gray-300")
                    .text(WIDGET_FAMILY_INVENTORY)
                    .build(),
            ))
            .build()
    }

    fn input_page(&self) -> Element {
        div()
            .class("grid grid-cols-2 gap-1")
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
        div()
            .class("grid grid-cols-2 gap-1")
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
            .build()
    }

    fn data_page(&self) -> Element {
        let root = TreeNode::new("root", "reactive-tui")
            .expanded(true)
            .add_child(TreeNode::new("src", "src"))
            .add_child(TreeNode::new("manual", "manual"));
        let manual = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("manual");
        div()
            .class("grid grid-cols-2 gap-1")
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
            .class("flex-col gap-1")
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
        Self::card(
            "Wireframe cube",
            div()
                .class("w-full h-16 text-cyan-300")
                .text(self.motion.frame())
                .build(),
        )
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
            .class("flex-col gap-1")
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
            CatalogPage::MenusDialogs => self.menus_dialogs_page(),
            CatalogPage::Media => self.media_page(),
            CatalogPage::Motion => self.motion_page(),
            CatalogPage::System => self.system_page(),
        }
    }

    fn stage(&self) -> Element {
        div()
            .class("flex-col flex-1 min-w-0 min-h-0 h-full p-0.25 gap-0.25 bg-black")
            .child(
                div()
                    .class("h-1 text-white font-bold")
                    .text(self.page.title())
                    .build(),
            )
            .child(
                scroll_view()
                    .contents(vec![self.selected_page()])
                    .vertical_scroll(true)
                    .show_scrollbars(true)
                    .class("flex-1 min-h-0")
                    .build(),
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
            .class("w-screen h-screen flex-col bg-black text-gray-200")
            .child(
                div()
                    .class("w-full shrink-0 h-3 flex-row px-0.25 bg-gray-950 border-b border-gray-700")
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
                    .text(if self.width < 80 { "1–8 page · F2 demo · Ctrl+Q quit" } else { "↑↓/←→ page · 1–8 jump · F1/F2 demo · Tab interact · Ctrl+Q / Ctrl+C / Esc quit" })
                    .build(),
            )
            .build()
    }

    fn resize(&mut self, width: u16, height: u16) -> reactive_tui::Result<()> {
        self.width = width;
        self.height = height;
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
        Ok(if self.exit_requested {
            RootUpdate::Exit
        } else if self.page == CatalogPage::Motion && self.motion.advance(Instant::now()) {
            RootUpdate::Redraw
        } else {
            RootUpdate::Unchanged
        })
    }
}
