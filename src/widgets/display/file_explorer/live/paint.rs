use super::*;
use crate::{
    accessibility::{Node, Role},
    builder::ElementBuilder,
    component::{ElementType, FocusProps, LayoutType},
    layout::style::{Direction, StyleBuilder},
    widgets::display::look,
};
use unicode_width::UnicodeWidthStr;

fn safe_text(text: &str) -> String {
    text.chars()
        .map(|c| if c.is_control() { '�' } else { c })
        .collect()
}

impl Explorer {
    /// A line of text in `class`, at a cell of the explorer; a target makes
    /// it a button the pointer can press.
    fn styled(
        &self,
        text: String,
        x: usize,
        y: usize,
        width: usize,
        target: Option<Target>,
        class: &str,
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
            .class(&format!("whitespace-pre truncate {class}"))
            .build();
        if let Some(target) = target {
            let targets = self.targets.clone();
            let mut node = Node::new(Role::Button);
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
            if let Target::Expand(path) = &target {
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

    /// One entry's row: its icon and name, then its size and date in
    /// `text-muted` when details are on. The row is `selection` while it
    /// holds the cursor and the list the focus, `accent` when selected,
    /// `hover` under the pointer (DAT-001); it tells the screen reader its
    /// name, its position and the count of entries (DAT-004).
    fn entry(
        &self,
        row: &Row,
        x: usize,
        y: usize,
        width: usize,
        position: usize,
        count: usize,
    ) -> Element {
        let path = &row.entry.path;
        let listing = self.prompt.is_none() && !self.search_edit;
        let cursor = self.focused && listing && self.cursor.as_ref() == Some(path);
        let selected = self.selected.contains(path);
        let hovered = self.hover.as_ref() == Some(path);
        // Each piece has its measured width, so its spaces are kept; the
        // details give way before the name when the row is narrow.
        let label = format!("{} {}", row.entry.icon, safe_text(&row.entry.name));
        let mut pieces = vec![ElementBuilder::new(ElementType::Text(label.clone()))
            .styles(
                StyleBuilder::new()
                    .width_px(UnicodeWidthStr::width(label.as_str()) as f32)
                    .height_px(1.0)
                    .flex_shrink(0.0),
            )
            .class("whitespace-pre")
            .build()];
        if self.config.show_details && self.config.view_mode != ViewMode::Grid {
            let details = format!(
                "  {}  {}",
                row.entry.format_size(),
                row.entry.format_modified()
            );
            pieces.push(
                ElementBuilder::new(ElementType::Text(details.clone()))
                    .styles(
                        StyleBuilder::new()
                            .width_px(UnicodeWidthStr::width(details.as_str()) as f32)
                            .height_px(1.0)
                            .flex_shrink(1.0)
                            .min_width_px(0.0)
                            .overflow_hidden(),
                    )
                    .class(&format!("whitespace-pre {}", look::MUTED))
                    .build(),
            );
        }
        let tree = self.config.view_mode == ViewMode::Tree;
        let mut node = Node::new(if tree {
            Role::TreeItem
        } else {
            Role::ListBoxOption
        });
        node.set_label(safe_text(&row.entry.name));
        node.set_selected(selected);
        node.set_clickable();
        node.inner.set_position_in_set(position);
        node.inner.set_size_of_set(count);
        if tree && row.entry.file_type == FileType::Directory {
            node.set_expanded(self.expanded.contains(path));
        }
        let mut element = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(
                StyleBuilder::new()
                    .display_flex()
                    .direction(Direction::Row)
                    .position_absolute()
                    .inset_left(x as f32)
                    .inset_top(y as f32)
                    .width_px(width as f32)
                    .height_px(1.0)
                    .overflow_hidden(),
            )
            .class(look::row(cursor, selected, hovered, false))
            .children(pieces)
            .build()
            .with_accessibility(node);
        element.key = Some(format!("entry:{}", path_key(path)));
        let options = element
            .metadata
            .accessibility_options
            .get_or_insert_default();
        options.focus = cursor;
        options.focus_event = Some(CustomEvent::new(
            "reactive_tui.file_explorer.focus",
            path_key(path).into_bytes(),
        ));
        let targets = self.targets.clone();
        let target = Target::Entry(path.clone());
        element.metadata.layout.push(Arc::new(move |layout| {
            targets.lock().unwrap().push((target.clone(), layout));
            false
        }));
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
        let last = segments.len().saturating_sub(1);
        for (index, (label, path)) in segments.into_iter().enumerate() {
            let width = UnicodeWidthStr::width(label.as_str()) + 2;
            if x >= self.width() {
                break;
            }
            // Earlier directories are hints; the current one is text.
            let class = if index == last {
                look::TEXT
            } else {
                look::MUTED
            };
            output.push(self.styled(
                format!("{label}/ "),
                x,
                y,
                width.min(self.width() - x),
                Some(Target::Navigate(path)),
                class,
            ));
            x += width;
        }
        output
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
            children.push(self.styled(label, x, toolbar, size, Some(target), look::MUTED));
            x += size;
        }
        let mut x = 0;
        for (label, operation) in [
            ("Copy", worker::Operation::Copy),
            ("Move", worker::Operation::Move),
            ("Ren", worker::Operation::Rename),
            ("Del", worker::Operation::Delete),
        ] {
            children.push(self.styled(
                label.into(),
                x,
                toolbar + 1,
                label.len() + 1,
                Some(Target::Operation(operation)),
                look::MUTED,
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
            let mut search = self.styled(
                format!(
                    "/{}{}",
                    self.config.search_query.as_deref().unwrap_or(""),
                    if self.search_edit { "▏" } else { "" }
                ),
                0,
                toolbar + 2,
                width,
                Some(Target::Search),
                look::TEXT,
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
            let mut input = self.styled(label.clone(), 0, y, width, None, look::TEXT);
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
            children.push(self.styled(
                "Confirm".into(),
                0,
                y + 1,
                8,
                Some(Target::Confirm),
                look::TEXT,
            ));
            children.push(self.styled(
                "Cancel".into(),
                8,
                y + 1,
                7,
                Some(Target::Cancel),
                look::TEXT,
            ));
        }
        let columns = self.columns();
        let cell_width = width / columns;
        let start = self.scroll.saturating_mul(columns);
        let count = self
            .visible_rows()
            .saturating_mul(columns)
            .min(self.config.max_visible_items.max(1));
        let total = self.rows.len();
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
                    self.styled(
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
                        look::MUTED,
                    ),
                );
            }
            children.push(self.entry(
                row,
                x + indent + marker,
                y,
                cell_width.saturating_sub(indent + marker),
                index + 1,
                total,
            ));
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
                    .class(&format!("whitespace-normal break-words {}", look::ERROR))
                    .build(),
            );
        }
        if self.rows.is_empty() && self.error.is_none() && self.config.max_visible_items > 0 {
            children.push(
                self.styled(
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
                    look::MUTED,
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
                children.push(self.styled(text.into(), 0, y + line, width, None, look::TEXT));
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
        // The status line is a hint, unless it carries the error (DAT-001).
        let status_class = if self.error.is_some() {
            look::ERROR
        } else {
            look::MUTED
        };
        children.push(self.styled(
            status,
            0,
            height.saturating_sub(1),
            width.saturating_sub(usize::from(self.pending_operation) * 7),
            None,
            status_class,
        ));
        if self.pending_operation {
            children.push(self.styled(
                "Cancel".into(),
                width.saturating_sub(7),
                height.saturating_sub(1),
                7,
                Some(Target::Cancel),
                look::TEXT,
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
        let natural_height = self.header_rows()
            + 1
            + usize::from(self.config.show_preview) * 3
            + self
                .rows
                .len()
                .max(1)
                .div_ceil(columns)
                .min((self.config.max_visible_items / columns).max(1));
        // The rows are absolute, so this child gives an auto-sized parent the
        // explorer's natural size.
        let intrinsic = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(
                StyleBuilder::new()
                    .width_px(24.0)
                    .height_px(natural_height.min(u16::MAX as usize) as f32),
            )
            .build();
        let mut node = Node::new(if self.config.view_mode == ViewMode::Tree {
            Role::Tree
        } else {
            Role::ListBox
        });
        // Named by `aria_label` alone; without one it has no fixed English
        // name (DAT-004).
        if let Some(label) = &self.config.aria_label {
            node.set_label(label.clone());
        }
        // The explorer fills the width and the height its parent allots,
        // and takes its natural size in a parent that allots none, unless
        // the props set a size (DAT-002).
        let mut style = StyleBuilder::new()
            .max_height_percent(100.0)
            .min_height_px(0.0)
            .overflow_hidden();
        style = match self.config.height {
            Some(height) => style.height_px(height as f32),
            None => style.height_percent(100.0),
        };
        style = match self.config.width {
            Some(width) => style.width_px(width as f32),
            None => style.width_percent(100.0),
        };
        ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(style)
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
