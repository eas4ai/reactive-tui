use reactive_tui::{
    app::{RootComponent, RootUpdate},
    builder::div,
    component::Element,
    event::{
        router::EventResult,
        types::{Event, KeyCode, KeyEventKind},
    },
};

/// Five complex application layouts showcasing Tailwind utility classes.
/// Every panel has its own background color so region boundaries are obvious.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShowcasePage {
    Ide,
    Dashboard,
    Mail,
    Store,
    Monitor,
}

impl ShowcasePage {
    pub const ALL: [Self; 5] = [
        Self::Ide,
        Self::Dashboard,
        Self::Mail,
        Self::Store,
        Self::Monitor,
    ];

    pub const fn title(self) -> &'static str {
        match self {
            Self::Ide => "IDE · editor workspace",
            Self::Dashboard => "Dashboard · stats overview",
            Self::Mail => "Mail · three-pane client",
            Self::Store => "Store · admin with cart",
            Self::Monitor => "Monitor · system status",
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::Ide => 0,
            Self::Dashboard => 1,
            Self::Mail => 2,
            Self::Store => 3,
            Self::Monitor => 4,
        }
    }

    fn offset(self, delta: isize) -> Self {
        let count = Self::ALL.len() as isize;
        let index = (self.index() as isize + delta).rem_euclid(count) as usize;
        Self::ALL[index]
    }
}

pub struct Showcase {
    page: ShowcasePage,
    width: u16,
    height: u16,
    exit_requested: bool,
}

impl Default for Showcase {
    fn default() -> Self {
        Self {
            page: ShowcasePage::Ide,
            width: 100,
            height: 30,
            exit_requested: false,
        }
    }
}

/// One line of panel body text.
fn line(text: &str) -> Element {
    div().class("h-1 shrink-0 text-gray-200").text(text).build()
}

/// A colored panel with a bold title and body rows. The background color
/// marks the panel boundary; adjacent panels always use different colors.
fn panel(title: &str, bg: &str, body: Vec<Element>) -> Element {
    let mut children = vec![div()
        .class("h-1 shrink-0 text-white font-bold")
        .text(title)
        .build()];
    children.extend(body);
    div()
        .class("flex-col flex-1 min-w-0 min-h-0 p-0.25")
        .class(bg)
        .children(children)
        .build()
}

/// A single-height colored bar for menus, status lines, and toolbars.
fn bar(text: &str, bg: &str) -> Element {
    div()
        .class("w-full h-1 shrink-0 px-0.25 text-white")
        .class(bg)
        .text(text)
        .build()
}

impl Showcase {
    #[cfg(test)]
    pub fn page(&self) -> ShowcasePage {
        self.page
    }

    fn compact(&self) -> bool {
        self.width < 80
    }

    fn page_for_number(ch: char) -> Option<ShowcasePage> {
        ch.to_digit(10)
            .and_then(|number| number.checked_sub(1))
            .and_then(|index| ShowcasePage::ALL.get(index as usize).copied())
    }

    fn ide_page(&self) -> Element {
        let files = panel(
            "Files · bg-cyan-900",
            "bg-cyan-900",
            vec![
                line("▾ src/"),
                line("  main.rs"),
                line("  lib.rs"),
                line("  app.rs"),
                line("▾ tests/"),
            ],
        );
        let editor = panel(
            "Editor · flex-1 bg-slate-800",
            "bg-slate-800",
            vec![line("fn main() {"), line("    run()?;"), line("}")],
        );
        let outline = panel(
            "Outline · bg-violet-900",
            "bg-violet-900",
            vec![line("◆ main"), line("◆ render"), line("◆ update")],
        );
        let terminal = panel(
            "Terminal · h-6 bg-emerald-900",
            "bg-emerald-900",
            vec![line("$ cargo test"), line("ok · 21 passed")],
        );
        if self.compact() {
            div()
                .class("flex-col flex-1 min-h-0 min-w-0 gap-0.25")
                .child(bar("▤ File Edit View Run", "bg-indigo-800"))
                .child(editor)
                .child(
                    div()
                        .class("w-full h-4 shrink-0 p-0.25 bg-cyan-900")
                        .child(
                            div()
                                .class("h-1 text-white font-bold")
                                .text("Files · strip")
                                .build(),
                        )
                        .child(line("src/ main.rs lib.rs app.rs"))
                        .build(),
                )
                .child(
                    div()
                        .class("w-full h-5 shrink-0 flex-col p-0.25 bg-emerald-900")
                        .children(vec![
                            div()
                                .class("h-1 text-white font-bold")
                                .text("Terminal")
                                .build(),
                            line("$ cargo test · ok"),
                        ])
                        .build(),
                )
                .build()
        } else {
            div()
                .class("flex-col flex-1 min-h-0 min-w-0 gap-0.25")
                .child(bar("▤ File  Edit  View  Run  Help", "bg-indigo-800"))
                .child(
                    div()
                        .class("flex-row flex-1 min-h-0 min-w-0 gap-0.25")
                        .child(div().class("w-24 shrink-0 min-h-0").child(files).build())
                        .child(editor)
                        .child(div().class("w-24 shrink-0 min-h-0").child(outline).build())
                        .build(),
                )
                .child(
                    div()
                        .class("w-full h-6 shrink-0 min-h-0")
                        .child(terminal)
                        .build(),
                )
                .child(bar("main* · UTF-8 · Tab-friendly", "bg-gray-700"))
                .build()
        }
    }

    fn dashboard_page(&self) -> Element {
        let kpi = |title: &str, value: &str, bg: &str| {
            div()
                .class("flex-col min-w-0 p-0.25")
                .class(bg)
                .child(
                    div()
                        .class("h-1 shrink-0 text-gray-200")
                        .text(title)
                        .build(),
                )
                .child(
                    div()
                        .class("h-1 shrink-0 text-white font-bold")
                        .text(value)
                        .build(),
                )
                .build()
        };
        let kpis = div()
            .class(if self.compact() {
                "w-full grid grid-cols-2 gap-0.25"
            } else {
                "w-full grid grid-cols-4 gap-0.25"
            })
            .child(kpi("Revenue", "$48.2k ▲", "bg-blue-800"))
            .child(kpi("Users", "12,409 ▲", "bg-emerald-800"))
            .child(kpi("Errors", "17 ▼", "bg-rose-800"))
            .child(kpi("Uptime", "99.98%", "bg-amber-800"))
            .build();
        let chart = panel(
            "Traffic · flex-1 bg-slate-800",
            "bg-slate-800",
            vec![line("▁▂▃▄▅▆▇█ ▁▂▃▄▅▆▇█"), line("12k ──●──●──●── 9k")],
        );
        let feed = panel(
            "Activity · bg-purple-900",
            "bg-purple-900",
            vec![
                line("● deploy ok"),
                line("● user +412"),
                line("● backup done"),
            ],
        );
        let table = panel(
            "Top pages · h-7 bg-teal-900",
            "bg-teal-900",
            vec![line("/  8.1k"), line("/docs  3.4k"), line("/blog  1.9k")],
        );
        if self.compact() {
            div()
                .class("flex-col flex-1 min-h-0 min-w-0 gap-0.25")
                .child(kpis)
                .child(chart)
                .child(
                    div()
                        .class("w-full h-6 shrink-0 min-h-0")
                        .child(feed)
                        .build(),
                )
                .build()
        } else {
            div()
                .class("flex-col flex-1 min-h-0 min-w-0 gap-0.25")
                .child(kpis)
                .child(
                    div()
                        .class("flex-row flex-1 min-h-0 min-w-0 gap-0.25")
                        .child(chart)
                        .child(div().class("w-28 shrink-0 min-h-0").child(feed).build())
                        .build(),
                )
                .child(
                    div()
                        .class("w-full h-7 shrink-0 min-h-0")
                        .child(table)
                        .build(),
                )
                .build()
        }
    }

    fn mail_page(&self) -> Element {
        let folders = panel(
            "Folders",
            "bg-indigo-900",
            vec![line("▶ Inbox 4"), line("  Sent"), line("  Drafts")],
        );
        let list = panel(
            "Inbox · flex-1 bg-slate-800",
            "bg-slate-800",
            vec![
                line("● deploy passed"),
                line("○ invoice #1042"),
                line("○ weekend plans"),
            ],
        );
        let reader = div()
            .class("flex-col flex-1 min-w-0 min-h-0 gap-0.25")
            .child(panel(
                "Reading · flex-1 bg-gray-800",
                "bg-gray-800",
                vec![line("Subject: deploy passed"), line("All 21 checks green.")],
            ))
            .child(bar("Compose · h-2 · Reply…", "bg-emerald-800"))
            .build();
        if self.compact() {
            div()
                .class("flex-col flex-1 min-h-0 min-w-0 gap-0.25")
                .child(bar("Inbox Sent Drafts", "bg-indigo-900"))
                .child(reader)
                .build()
        } else {
            div()
                .class("flex-row flex-1 min-h-0 min-w-0 gap-0.25")
                .child(div().class("w-16 shrink-0 min-h-0").child(folders).build())
                .child(list)
                .child(reader)
                .build()
        }
    }

    /// Product columns derive from the measured terminal width (one column
    /// per ~28 cells, capped by the product count), so pixel-wide windows
    /// fill instead of stretching a fixed column tier.
    fn product_grid_class(width: u16) -> &'static str {
        match (width / 28).clamp(1, 8) {
            1 => "w-full grid grid-cols-1 gap-0.25",
            2 => "w-full grid grid-cols-2 gap-0.25",
            3 => "w-full grid grid-cols-3 gap-0.25",
            4 => "w-full grid grid-cols-4 gap-0.25",
            5 => "w-full grid grid-cols-5 gap-0.25",
            6 => "w-full grid grid-cols-6 gap-0.25",
            7 => "w-full grid grid-cols-7 gap-0.25",
            _ => "w-full grid grid-cols-8 gap-0.25",
        }
    }

    fn store_page(&self) -> Element {
        let compact = self.compact();
        let card = move |name: &'static str, price: &'static str, bg: &'static str| {
            let title = if compact {
                format!("{name} · {price}")
            } else {
                name.to_string()
            };
            let mut card = div().class("flex-col min-w-0 p-0.25").class(bg).child(
                div()
                    .class("h-1 shrink-0 text-white font-bold")
                    .text(&title)
                    .build(),
            );
            if !compact {
                card = card.child(line(price));
            }
            card.build()
        };
        let grid = div()
            .class(Self::product_grid_class(self.width))
            .child(card("Keyboard", "$89", "bg-cyan-800"))
            .child(card("Mouse", "$49", "bg-teal-800"))
            .child(card("Monitor", "$299", "bg-sky-800"))
            .child(card("Desk", "$199", "bg-indigo-800"))
            .child(card("Headphones", "$129", "bg-violet-800"))
            .child(card("Webcam", "$79", "bg-blue-800"))
            .child(card("Chair", "$249", "bg-emerald-800"))
            .child(card("Lamp", "$39", "bg-rose-800"))
            .build();
        let cats = panel(
            "Cats · bg-amber-900",
            "bg-amber-900",
            vec![line("● Office"), line("○ Audio"), line("○ Desks")],
        );
        let cart = panel(
            "Cart · bg-emerald-900",
            "bg-emerald-900",
            vec![line("3 items"), line("Total $187")],
        );
        if self.compact() {
            div()
                .class("flex-col flex-1 min-h-0 min-w-0 gap-0.25")
                .child(bar("⌕ Search products…", "bg-slate-900"))
                .child(bar("Office Audio Desks", "bg-amber-900"))
                .child(grid)
                .child(
                    div()
                        .class("w-full h-4 shrink-0 min-h-0")
                        .child(cart)
                        .build(),
                )
                .build()
        } else {
            div()
                .class("flex-col flex-1 min-h-0 min-w-0 gap-0.25")
                .child(bar("⌕ Search products…   [_input_]   [Go]", "bg-slate-900"))
                .child(
                    div()
                        .class("flex-row flex-1 min-h-0 min-w-0 gap-0.25")
                        .child(div().class("w-20 shrink-0 min-h-0").child(cats).build())
                        .child(grid)
                        .child(div().class("w-24 shrink-0 min-h-0").child(cart).build())
                        .build(),
                )
                .build()
        }
    }

    fn monitor_page(&self) -> Element {
        let stat = |title: &str, value: &str, bg: &str| {
            div()
                .class("flex-col flex-1 min-w-0 p-0.25")
                .class(bg)
                .child(
                    div()
                        .class("h-1 shrink-0 text-gray-200")
                        .text(title)
                        .build(),
                )
                .child(
                    div()
                        .class("h-1 shrink-0 text-white font-bold")
                        .text(value)
                        .build(),
                )
                .build()
        };
        let stats = div()
            .class("w-full flex-row gap-0.25")
            .child(stat("CPU", "███░░░ 34%", "bg-blue-800"))
            .child(stat("MEM", "█████░ 71%", "bg-violet-800"))
            .child(stat("DISK", "██░░░░ 22%", "bg-amber-800"))
            .child(stat("NET", "▲12 ▼40", "bg-teal-800"))
            .build();
        let table = panel(
            "Processes · flex-1 bg-slate-800",
            "bg-slate-800",
            vec![
                line("1234 app      12%"),
                line("  87 worker    4%"),
                line("  42 logger    1%"),
            ],
        );
        let log = panel(
            "Log · w-28 bg-gray-900",
            "bg-gray-900",
            vec![line("ok started"), line("ok synced"), line(".. polling")],
        );
        if self.compact() {
            div()
                .class("flex-col flex-1 min-h-0 min-w-0 gap-0.25")
                .child(stats)
                .child(table)
                .child(
                    div()
                        .class("w-full h-6 shrink-0 min-h-0")
                        .child(log)
                        .build(),
                )
                .build()
        } else {
            div()
                .class("flex-col flex-1 min-h-0 min-w-0 gap-0.25")
                .child(stats)
                .child(
                    div()
                        .class("flex-row flex-1 min-h-0 min-w-0 gap-0.25")
                        .child(table)
                        .child(div().class("w-28 shrink-0 min-h-0").child(log).build())
                        .build(),
                )
                .build()
        }
    }

    fn selected_page(&self) -> Element {
        match self.page {
            ShowcasePage::Ide => self.ide_page(),
            ShowcasePage::Dashboard => self.dashboard_page(),
            ShowcasePage::Mail => self.mail_page(),
            ShowcasePage::Store => self.store_page(),
            ShowcasePage::Monitor => self.monitor_page(),
        }
    }

    fn footer_text(&self) -> &'static str {
        if self.width < 80 {
            "Tab next · 1–5 · Ctrl+Q quit"
        } else {
            "Tab next layout · Shift+Tab prev · 1–5 jump · Ctrl+Q quit"
        }
    }
}

impl RootComponent for Showcase {
    fn render(&self) -> Element {
        div()
            .class("w-screen h-screen flex-col bg-black text-gray-200")
            .child(
                div()
                    .class(
                        "w-full shrink-0 h-1 flex-row px-0.25 bg-gray-950 border-b border-gray-700",
                    )
                    .child(
                        div()
                            .class("flex-1 text-cyan-300 font-bold")
                            .text(&format!("◈ Layout Showcase · {}", self.page.title()))
                            .build(),
                    )
                    .child(
                        div()
                            .class("text-gray-500")
                            .text(&format!(
                                "{}/{} {}×{}",
                                self.page.index() + 1,
                                ShowcasePage::ALL.len(),
                                self.width,
                                self.height
                            ))
                            .build(),
                    )
                    .build(),
            )
            .child(
                div()
                    .class("w-full flex-1 min-h-0 min-w-0 p-0.25")
                    .child(self.selected_page())
                    .build(),
            )
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
        Ok(())
    }

    fn try_handle_event(&mut self, event: &Event) -> reactive_tui::Result<EventResult> {
        let Event::Key(key) = event else {
            return Ok(EventResult::Ignored);
        };
        if key.kind == KeyEventKind::Release {
            return Ok(EventResult::Ignored);
        }
        if key.code == KeyCode::Escape
            || (key.modifiers.ctrl && matches!(key.code, KeyCode::Char('q' | 'c')))
        {
            self.exit_requested = true;
            return Ok(EventResult::Handled);
        }
        let next = match key.code {
            KeyCode::Tab if key.modifiers.shift => Some(self.page.offset(-1)),
            KeyCode::Tab | KeyCode::Right | KeyCode::Down => Some(self.page.offset(1)),
            KeyCode::BackTab | KeyCode::Left | KeyCode::Up => Some(self.page.offset(-1)),
            KeyCode::Char(ch) => Self::page_for_number(ch),
            _ => None,
        };
        if let Some(page) = next {
            self.page = page;
            Ok(EventResult::Handled)
        } else {
            Ok(EventResult::Ignored)
        }
    }

    fn update(&mut self) -> reactive_tui::Result<RootUpdate> {
        Ok(if self.exit_requested {
            RootUpdate::Exit
        } else {
            RootUpdate::Unchanged
        })
    }
}
