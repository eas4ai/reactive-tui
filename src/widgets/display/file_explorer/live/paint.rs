use super::*;
use crate::{
    accessibility::{Node, Role},
    builder::ElementBuilder,
    component::{ElementType, FocusProps, LayoutType},
    layout::style::StyleBuilder,
};
use unicode_width::UnicodeWidthStr;

fn safe_text(text: &str) -> String {
    text.chars()
        .map(|c| if c.is_control() { '�' } else { c })
        .collect()
}

impl Explorer {
    fn text(
        &self,
        text: String,
        x: usize,
        y: usize,
        width: usize,
        target: Option<Target>,
    ) -> Element {
        let mut element = ElementBuilder::new(ElementType::Text(safe_text(&text)))
            .styles(
                StyleBuilder::new()
                    .position_absolute()
                    .inset_left(x as f32)
                    .inset_top(y as f32)
                    .width_px(width as f32)
                    .height_px(1.0)
                    .overflow_hidden(),
            )
            .class("whitespace-pre truncate")
            .build();
        if let Some(target) = target {
            let targets = self.targets.clone();
            let mut node = Node::new(if matches!(target, Target::Entry(_)) {
                if self.config.view_mode == ViewMode::Tree {
                    Role::TreeItem
                } else {
                    Role::ListBoxOption
                }
            } else {
                Role::Button
            });
            node.set_label(match &target {
                Target::View => format!("Change view: {:?}", self.config.view_mode),
                Target::Sort => format!("Sort files by {:?}", self.config.sort_criteria),
                Target::Order => format!("Sort order: {:?}", self.config.sort_order),
                Target::Hidden => if self.config.show_hidden {
                    "Hide hidden files"
                } else {
                    "Show hidden files"
                }
                .into(),
                Target::Search => "Search files".into(),
                Target::Preview => if self.config.show_preview {
                    "Hide file preview"
                } else {
                    "Show file preview"
                }
                .into(),
                Target::Refresh => "Refresh files".into(),
                _ => safe_text(&text),
            });
            node.set_clickable();
            if let Target::Entry(path) = &target {
                if let Some(row) = self.rows.iter().find(|row| &row.entry.path == path) {
                    node.set_label(safe_text(&row.entry.name));
                }
                node.set_selected(self.selected.contains(path));
                if self.config.view_mode == ViewMode::Tree
                    && self.rows.iter().any(|row| {
                        &row.entry.path == path && row.entry.file_type == FileType::Directory
                    })
                {
                    node.set_expanded(self.expanded.contains(path));
                }
                let options = element
                    .metadata
                    .accessibility_options
                    .get_or_insert_default();
                options.focus = self.focused
                    && self.prompt.is_none()
                    && !self.search_edit
                    && self.cursor.as_ref() == Some(path);
                options.focus_event = Some(CustomEvent::new(
                    "reactive_tui.file_explorer.focus",
                    path_key(path).into_bytes(),
                ));
                element.key = Some(format!("entry:{}", path_key(path)));
            } else if let Target::Expand(path) = &target {
                element.key = Some(format!("expand:{}", path_key(path)));
                node.set_label(if self.expanded.contains(path) {
                    "Collapse directory"
                } else {
                    "Expand directory"
                });
            } else if let Target::Navigate(path) = &target {
                element.key = Some(format!("navigate:{}", path_key(path)));
            } else if let Target::Operation(operation) = &target {
                node.set_label(format!("{operation:?} files"));
            }
            element.metadata.accessibility = Some(node);
            element.metadata.layout.push(Arc::new(move |layout| {
                targets.lock().unwrap().push((target.clone(), layout));
                false
            }));
        }
        element
    }

    fn breadcrumbs(&self, y: usize) -> Vec<Element> {
        let mut segments = vec![("Root".to_string(), self.config.root_path.clone())];
        if let Ok(relative) = self
            .config
            .current_path
            .strip_prefix(&self.config.root_path)
        {
            let mut path = self.config.root_path.clone();
            for component in relative.components() {
                path.push(component);
                segments.push((
                    component.as_os_str().to_string_lossy().into_owned(),
                    path.clone(),
                ));
            }
        }
        let mut x = 0;
        let mut output = Vec::new();
        for (label, path) in segments {
            let width = UnicodeWidthStr::width(label.as_str()) + 2;
            if x >= self.width() {
                break;
            }
            output.push(self.text(
                format!("{label}/ "),
                x,
                y,
                width.min(self.width() - x),
                Some(Target::Navigate(path)),
            ));
            x += width;
        }
        output
    }

    fn row_text(&self, row: &Row) -> String {
        let selected = if self.selected.contains(&row.entry.path) {
            "*"
        } else {
            " "
        };
        let cursor = if self.cursor.as_ref() == Some(&row.entry.path) {
            ">"
        } else {
            " "
        };
        let details = if self.config.show_details && self.config.view_mode != ViewMode::Grid {
            format!(
                "  {}  {}",
                row.entry.format_size(),
                row.entry.format_modified()
            )
        } else {
            String::new()
        };
        format!(
            "{cursor}{selected}{} {}{details}",
            row.entry.icon, row.entry.name
        )
    }

    pub(super) fn paint(&self) -> Element {
        if let Some(worker) = &self.worker {
            worker.observe();
        }
        self.targets.lock().unwrap().clear();
        let width = self.width();
        let height = self
            .viewport
            .map_or(0, |layout| layout.content_size().1.max(0.0) as usize);
        let mut children = Vec::new();
        let toolbar = usize::from(self.config.show_breadcrumb);
        if self.config.show_breadcrumb {
            children.extend(self.breadcrumbs(0));
        }
        let controls = [
            (format!("{:?}", self.config.view_mode), Target::View),
            (format!("{:?}", self.config.sort_criteria), Target::Sort),
            (
                (if self.config.sort_order == SortOrder::Ascending {
                    "↑"
                } else {
                    "↓"
                })
                .into(),
                Target::Order,
            ),
            (
                (if self.config.show_hidden { "All" } else { "." }).into(),
                Target::Hidden,
            ),
            ("/".into(), Target::Search),
            ("P".into(), Target::Preview),
            ("⟳".into(), Target::Refresh),
        ];
        let mut x = 0;
        for (label, target) in controls {
            let size = UnicodeWidthStr::width(label.as_str()) + 1;
            children.push(self.text(label, x, toolbar, size, Some(target)));
            x += size;
        }
        let mut x = 0;
        for (label, operation) in [
            ("Copy", worker::Operation::Copy),
            ("Move", worker::Operation::Move),
            ("Ren", worker::Operation::Rename),
            ("Del", worker::Operation::Delete),
        ] {
            children.push(self.text(
                label.into(),
                x,
                toolbar + 1,
                label.len() + 1,
                Some(Target::Operation(operation)),
            ));
            x += label.len() + 1;
        }
        if self.search_edit
            || self
                .config
                .search_query
                .as_ref()
                .is_some_and(|s| !s.is_empty())
        {
            let mut search = self.text(
                format!(
                    "/{}{}",
                    self.config.search_query.as_deref().unwrap_or(""),
                    if self.search_edit { "▏" } else { "" }
                ),
                0,
                toolbar + 2,
                width,
                Some(Target::Search),
            );
            let mut node = Node::new(Role::TextInput);
            node.set_label("Filter files");
            node.set_value(self.config.search_query.clone().unwrap_or_default());
            search.metadata.accessibility = Some(node);
            search
                .metadata
                .accessibility_options
                .get_or_insert_default()
                .focus = self.focused && self.search_edit;
            search.key = Some("search-input".into());
            children.push(search);
        }
        if let Some(prompt) = &self.prompt {
            let label = if prompt.operation == worker::Operation::Delete {
                format!("Delete {} entries?", prompt.sources.len())
            } else {
                format!("{:?}: {}▏", prompt.operation, prompt.destination)
            };
            let y = self.header_rows().saturating_sub(2);
            let mut input = self.text(label.clone(), 0, y, width, None);
            let mut node = Node::new(if prompt.operation == worker::Operation::Delete {
                Role::AlertDialog
            } else {
                Role::TextInput
            });
            node.set_label(if prompt.operation == worker::Operation::Delete {
                label
            } else {
                format!("{:?} destination", prompt.operation)
            });
            if prompt.operation != worker::Operation::Delete {
                node.set_value(prompt.destination.clone());
            }
            input.metadata.accessibility = Some(node);
            input
                .metadata
                .accessibility_options
                .get_or_insert_default()
                .focus = self.focused;
            input.key = Some("operation-input".into());
            children.push(input);
            children.push(self.text("Confirm".into(), 0, y + 1, 8, Some(Target::Confirm)));
            children.push(self.text("Cancel".into(), 8, y + 1, 7, Some(Target::Cancel)));
        }
        let columns = self.columns();
        let cell_width = width / columns;
        let start = self.scroll.saturating_mul(columns);
        let count = self
            .visible_rows()
            .saturating_mul(columns)
            .min(self.config.max_visible_items.max(1));
        for (index, row) in self.rows.iter().enumerate().skip(start).take(
            if self.error.is_none() && self.config.max_visible_items > 0 {
                count
            } else {
                0
            },
        ) {
            let y = index / columns - self.scroll + self.header_rows();
            let x = index % columns * cell_width;
            let indent = if self.config.view_mode == ViewMode::Tree {
                row.depth.saturating_mul(2).min(cell_width)
            } else {
                0
            };
            let marker = usize::from(self.config.view_mode == ViewMode::Tree) * 2;
            if marker > 0 && row.entry.file_type == FileType::Directory {
                children.push(
                    self.text(
                        (if self.expanded.contains(&row.entry.path) {
                            "▼ "
                        } else {
                            "▶ "
                        })
                        .into(),
                        x + indent,
                        y,
                        marker,
                        Some(Target::Expand(row.entry.path.clone())),
                    ),
                );
            }
            let mut entry = self.text(
                self.row_text(row),
                x + indent + marker,
                y,
                cell_width.saturating_sub(indent + marker),
                Some(Target::Entry(row.entry.path.clone())),
            );
            if self.selected.contains(&row.entry.path) {
                entry.class = Some("whitespace-pre truncate bg-blue-600 text-white".into());
            }
            children.push(entry);
        }
        let validation = (self.config.max_visible_items == 0)
            .then_some("max_visible_items must be greater than zero");
        if let Some(error) = validation.or(self.error.as_deref()) {
            children.push(
                ElementBuilder::new(ElementType::Text(safe_text(error)))
                    .styles(
                        StyleBuilder::new()
                            .position_absolute()
                            .inset_left(0.0)
                            .inset_top(self.header_rows() as f32)
                            .width_px(width as f32)
                            .height_px(self.visible_rows().max(1) as f32)
                            .overflow_hidden(),
                    )
                    .class("whitespace-normal break-words bg-black text-red-400")
                    .build(),
            );
        }
        if self.rows.is_empty() && self.error.is_none() && self.config.max_visible_items > 0 {
            children.push(
                self.text(
                    (if self.pending.is_some() {
                        "Loading…"
                    } else {
                        "No matching files"
                    })
                    .into(),
                    0,
                    self.header_rows(),
                    width,
                    None,
                ),
            );
        }
        if self.config.show_preview {
            let y = height.saturating_sub(4);
            let preview = self
                .preview
                .as_ref()
                .filter(|(path, _)| Some(path) == self.cursor.as_ref())
                .map_or(
                    if self.pending_preview && self.pending.is_some() {
                        "Loading preview…"
                    } else {
                        "Preview"
                    },
                    |(_, text)| text.as_str(),
                );
            for (line, text) in preview.lines().take(3).enumerate() {
                children.push(self.text(text.into(), 0, y + line, width, None));
            }
        }
        let status = self.error.clone().unwrap_or_else(|| {
            if self.pending_operation {
                self.status.clone().unwrap_or_else(|| "Working…".into())
            } else if self.pending.is_some() {
                "Loading…".into()
            } else if let Some(status) = &self.status {
                status.clone()
            } else {
                format!(
                    "{} files · {} selected",
                    self.rows.len(),
                    self.selected.len()
                )
            }
        });
        children.push(self.text(
            status,
            0,
            height.saturating_sub(1),
            width.saturating_sub(usize::from(self.pending_operation) * 7),
            None,
        ));
        if self.pending_operation {
            children.push(self.text(
                "Cancel".into(),
                width.saturating_sub(7),
                height.saturating_sub(1),
                7,
                Some(Target::Cancel),
            ));
        }
        let insets = self.viewport.map_or([0.0; 4], |layout| layout.insets);
        let content = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(
                StyleBuilder::new()
                    .position_absolute()
                    .inset_left(insets[0])
                    .inset_top(insets[1])
                    .width_px(width as f32)
                    .height_px(height as f32)
                    .overflow_hidden(),
            )
            .children(children)
            .build();
        let intrinsic = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(StyleBuilder::new().width_px(24.0).height_px(0.0))
            .build();
        let mut node = Node::new(if self.config.view_mode == ViewMode::Tree {
            Role::Tree
        } else {
            Role::ListBox
        });
        node.set_label("File explorer");
        let natural_height = self.header_rows()
            + 1
            + usize::from(self.config.show_preview) * 3
            + self
                .rows
                .len()
                .max(1)
                .div_ceil(columns)
                .min((self.config.max_visible_items / columns).max(1));
        ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(
                StyleBuilder::new()
                    .width_percent(100.0)
                    .height_px(natural_height.min(u16::MAX as usize) as f32)
                    .max_height_percent(100.0)
                    .min_height_px(1.0)
                    .overflow_hidden(),
            )
            .class(self.config.class.as_deref().unwrap_or(""))
            .children(vec![content, intrinsic])
            .build()
            .with_focus(FocusProps {
                focusable: true,
                ..Default::default()
            })
            .with_accessibility(node)
    }
}
