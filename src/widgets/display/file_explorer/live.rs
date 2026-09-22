use super::*;
use crate::{
    component::LayoutInfo,
    event::{
        router::EventResult,
        types::{KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEventKind, WheelDelta},
        CustomEvent, Event, MouseEvent,
    },
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use unicode_segmentation::UnicodeSegmentation;

fn path_key(path: &Path) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD_NO_PAD.encode(path.as_os_str().as_encoded_bytes())
}

mod paint;
#[path = "worker.rs"]
mod worker;

#[derive(Clone)]
struct Row {
    entry: FileEntry,
    depth: usize,
}

#[derive(Clone, PartialEq)]
enum Target {
    Entry(PathBuf),
    Expand(PathBuf),
    Navigate(PathBuf),
    View,
    Sort,
    Order,
    Hidden,
    Search,
    Preview,
    Refresh,
    Operation(worker::Operation),
    Confirm,
    Cancel,
}

struct Prompt {
    operation: worker::Operation,
    sources: Vec<PathBuf>,
    destination: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct LiveProps {
    pub config: FileExplorerProps,
    pub seed: FileExplorerState,
}
impl Props for LiveProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub(super) struct LiveExplorer {
    inner: Mutex<Explorer>,
    seed: FileExplorerState,
}

struct Explorer {
    previous: FileExplorerProps,
    config: FileExplorerProps,
    worker: Option<worker::Worker>,
    pending: Option<u64>,
    pending_preview: bool,
    pending_path: Option<PathBuf>,
    pending_operation: bool,
    refresh_after_error: bool,
    prompt: Option<Prompt>,
    status: Option<String>,
    directories: HashMap<PathBuf, Vec<FileEntry>>,
    expanded: HashSet<PathBuf>,
    rows: Vec<Row>,
    selected: HashSet<PathBuf>,
    cursor: Option<PathBuf>,
    anchor: Option<PathBuf>,
    viewport: Option<LayoutInfo>,
    targets: Arc<Mutex<Vec<(Target, LayoutInfo)>>>,
    scroll: usize,
    error: Option<String>,
    preview: Option<(PathBuf, String)>,
    search_edit: bool,
    last_press: Option<Target>,
    focused: bool,
}

impl Explorer {
    fn apply_seed(&mut self, seed: &FileExplorerState) {
        if seed.initialized || !seed.entries.is_empty() {
            self.directories.clear();
            self.directories
                .insert(self.config.current_path.clone(), seed.entries.clone());
            self.selected = seed
                .entries
                .iter()
                .enumerate()
                .filter(|(index, entry)| seed.selected_indices.contains(index) || entry.selected)
                .map(|(_, entry)| entry.path.clone())
                .collect();
            self.cursor = seed
                .focused_index
                .and_then(|index| {
                    seed.entries
                        .get(seed.filtered_entries.get(index).copied().unwrap_or(index))
                })
                .or_else(|| seed.entries.iter().find(|entry| entry.focused))
                .map(|entry| entry.path.clone());
            if self.config.selection_mode == SelectionMode::None {
                self.selected.clear();
            }
            if self.config.selection_mode == SelectionMode::Single && self.selected.len() > 1 {
                self.selected = seed
                    .entries
                    .iter()
                    .find(|entry| self.selected.contains(&entry.path))
                    .map(|entry| entry.path.clone())
                    .into_iter()
                    .collect();
            }
            self.scroll = seed.scroll_position;
            self.error = seed.error.clone();
            self.status = seed.loading.then(|| "Loading…".into());
            self.rebuild();
        }
    }
    fn request(&mut self, path: PathBuf, preview: bool) {
        let Some(worker) = &self.worker else { return };
        worker.observe();
        self.pending_path = Some(path.clone());
        self.pending = Some(worker.submit(
            self.previous.root_path.clone(),
            if preview {
                worker::Job::Preview(path)
            } else {
                worker::Job::Read(path)
            },
        ));
        self.pending_preview = preview;
        self.error = None;
    }

    fn receive(&mut self) -> bool {
        let Some(worker) = &self.worker else {
            return false;
        };
        worker.observe();
        let Some(response) = worker.take() else {
            return false;
        };
        if self.pending != Some(response.id) {
            return false;
        }
        self.pending = None;
        let requested_path = self.pending_path.take();
        match response.result {
            Ok(worker::Output::Entries {
                root,
                path,
                entries,
            }) => {
                let count: usize = self
                    .directories
                    .iter()
                    .filter(|(key, _)| *key != &path)
                    .map(|(_, entries)| entries.len())
                    .sum();
                if count.saturating_add(entries.len()) > 100_000 {
                    self.expanded.remove(&path);
                    self.rebuild();
                    self.error = Some("Cache exceeds 100000 entries; branch was not expanded. Collapse another branch and refresh".into());
                    return true;
                }
                self.config.root_path = root;
                // The first listing establishes the canonical path spelling.
                if self.directories.is_empty() {
                    self.config.current_path = path.clone();
                }
                self.directories.insert(path, entries);
                self.rebuild();
            }
            Ok(worker::Output::Preview { path, text }) => self.preview = Some((path, text)),
            Ok(worker::Output::Mutated(message)) => {
                self.status = Some(message);
                self.pending_operation = false;
                self.preview = None;
                self.directories.clear();
                self.request(self.config.current_path.clone(), false);
            }
            Err(error) => {
                self.refresh_after_error |= self.pending_operation;
                if self.pending_preview {
                    self.preview = requested_path.map(|path| (path, format!("Preview: {error}")));
                } else {
                    if let Some(path) = requested_path {
                        self.expanded.remove(&path);
                    }
                    self.rebuild();
                    self.error = Some(format!("Filesystem: {error}"));
                }
                self.pending_operation = false;
            }
        }
        true
    }

    fn visible_entries(&self, path: &Path) -> Vec<&FileEntry> {
        let query = self
            .config
            .search_query
            .as_deref()
            .unwrap_or("")
            .to_lowercase();
        let mut entries: Vec<_> = self
            .directories
            .get(path)
            .into_iter()
            .flatten()
            .filter(|entry| {
                (self.config.show_hidden || !entry.hidden)
                    && (entry.file_type == FileType::Directory
                        || self.config.file_filters.is_empty()
                        || entry.extension.as_ref().is_some_and(|extension| {
                            self.config.file_filters.iter().any(|filter| {
                                filter
                                    .trim_start_matches('.')
                                    .eq_ignore_ascii_case(extension)
                            })
                        }))
                    && (query.is_empty()
                        || entry.name.to_lowercase().contains(&query)
                        || (self.config.view_mode == ViewMode::Tree
                            && entry.file_type == FileType::Directory))
            })
            .collect();
        entries.sort_by(|a, b| {
            let folders =
                (b.file_type == FileType::Directory).cmp(&(a.file_type == FileType::Directory));
            let value = match self.config.sort_criteria {
                SortCriteria::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortCriteria::Size => a.size.cmp(&b.size),
                SortCriteria::Modified => a.modified.cmp(&b.modified),
                SortCriteria::Type => a.extension.cmp(&b.extension),
            }
            .then_with(|| a.path.cmp(&b.path));
            folders.then(if self.config.sort_order == SortOrder::Descending {
                value.reverse()
            } else {
                value
            })
        });
        entries
    }

    fn rebuild(&mut self) {
        let mut rows = Vec::new();
        let mut pending: Vec<_> = self
            .visible_entries(&self.config.current_path)
            .into_iter()
            .rev()
            .map(|entry| (entry, 0))
            .collect();
        while let Some((entry, depth)) = pending.pop() {
            rows.push(Row {
                entry: entry.clone(),
                depth,
            });
            if self.config.view_mode == ViewMode::Tree && self.expanded.contains(&entry.path) {
                pending.extend(
                    self.visible_entries(&entry.path)
                        .into_iter()
                        .rev()
                        .map(|child| (child, depth + 1)),
                );
            }
        }
        self.rows = rows;
        self.selected.retain(|path| {
            let mut candidate = path.as_path();
            while let Some(parent) = candidate.parent() {
                if let Some(entries) = self.directories.get(parent) {
                    return entries.iter().any(|entry| entry.path == candidate);
                }
                candidate = parent;
            }
            true
        });
        let expanding = self.rows.iter().any(|row| {
            self.expanded.contains(&row.entry.path)
                && !self.directories.contains_key(&row.entry.path)
        });
        if !expanding
            && !self
                .rows
                .iter()
                .any(|row| Some(&row.entry.path) == self.cursor.as_ref())
        {
            self.cursor = self.rows.first().map(|row| row.entry.path.clone());
        }
        self.clamp();
    }

    fn columns(&self) -> usize {
        if self.config.view_mode == ViewMode::Grid {
            (self.width() / 18)
                .max(1)
                .min(self.config.max_visible_items.max(1))
        } else {
            1
        }
    }

    fn width(&self) -> usize {
        self.viewport
            .map_or(0, |layout| layout.content_size().0.max(0.0) as usize)
    }

    fn header_rows(&self) -> usize {
        usize::from(self.config.show_breadcrumb)
            + 2
            + usize::from(
                self.search_edit
                    || self
                        .config
                        .search_query
                        .as_ref()
                        .is_some_and(|s| !s.is_empty()),
            )
            + usize::from(self.prompt.is_some()) * 2
    }

    fn visible_rows(&self) -> usize {
        self.viewport
            .map_or(0, |layout| layout.content_size().1.max(0.0) as usize)
            .saturating_sub(self.header_rows() + 1 + usize::from(self.config.show_preview) * 3)
            .min((self.config.max_visible_items / self.columns()).max(1))
    }

    fn clamp(&mut self) {
        let total = self.rows.len().div_ceil(self.columns());
        self.scroll = self
            .scroll
            .min(total.saturating_sub(self.visible_rows().max(1)));
    }

    fn reveal(&mut self) {
        if let Some(index) = self
            .rows
            .iter()
            .position(|row| Some(&row.entry.path) == self.cursor.as_ref())
        {
            let row = index / self.columns();
            if row < self.scroll {
                self.scroll = row;
            }
            if row >= self.scroll + self.visible_rows().max(1) {
                self.scroll = row + 1 - self.visible_rows().max(1);
            }
        }
        self.clamp();
    }

    fn notify(id: &Option<String>, paths: Vec<PathBuf>) {
        if let Some(id) = id {
            #[cfg(windows)]
            let (encoding, native_paths) = {
                use std::os::windows::ffi::OsStrExt;
                (
                    "utf16",
                    paths
                        .iter()
                        .map(|path| path.as_os_str().encode_wide().collect::<Vec<_>>())
                        .collect::<Vec<_>>(),
                )
            };
            #[cfg(not(windows))]
            let (encoding, native_paths) = (
                "bytes",
                paths
                    .iter()
                    .map(|path| path.as_os_str().as_encoded_bytes())
                    .collect::<Vec<_>>(),
            );
            let display: Vec<_> = paths.iter().map(|path| path.to_string_lossy()).collect();
            let payload = serde_json::json!({ "paths": display, "native_paths": native_paths, "encoding": encoding });
            crate::event::notifications::emit(CustomEvent::new(
                id,
                serde_json::to_vec(&payload).unwrap(),
            ));
        }
    }

    fn select(&mut self, path: PathBuf, shift: bool, toggle: bool) {
        let before = self.selected.clone();
        self.cursor = Some(path.clone());
        if self.config.selection_mode == SelectionMode::None {
            self.selected.clear();
        } else if self.config.selection_mode == SelectionMode::Multiple && shift {
            let start = self
                .rows
                .iter()
                .position(|row| Some(&row.entry.path) == self.anchor.as_ref())
                .unwrap_or(0);
            let end = self
                .rows
                .iter()
                .position(|row| row.entry.path == path)
                .unwrap_or(start);
            self.selected = self.rows[start.min(end)..=start.max(end)]
                .iter()
                .map(|row| row.entry.path.clone())
                .collect();
        } else if self.config.selection_mode == SelectionMode::Multiple && toggle {
            if !self.selected.remove(&path) {
                self.selected.insert(path.clone());
            }
            self.anchor = Some(path);
        } else {
            self.selected = HashSet::from([path.clone()]);
            self.anchor = Some(path);
        }
        if self.selected != before {
            let mut paths: Vec<_> = self.selected.iter().cloned().collect();
            paths.sort();
            Self::notify(&self.config.on_select, paths);
        }
        self.reveal();
        self.request_pending_content();
    }

    fn request_pending_content(&mut self) {
        if self.pending.is_some() || self.error.is_some() {
            return;
        }
        if self.config.view_mode == ViewMode::Tree {
            if let Some(path) = self
                .rows
                .iter()
                .find(|row| {
                    self.expanded.contains(&row.entry.path)
                        && !self.directories.contains_key(&row.entry.path)
                })
                .map(|row| row.entry.path.clone())
            {
                self.request(path, false);
                return;
            }
        }
        if !self.config.show_preview {
            return;
        }
        if let Some(path) = self.cursor.clone() {
            if self
                .preview
                .as_ref()
                .is_none_or(|(shown, _)| shown != &path)
            {
                self.request(path, true);
            }
        }
    }

    fn navigate(&mut self, path: PathBuf) {
        self.config.current_path = path.clone();
        self.directories.clear();
        self.rows.clear();
        self.selected.clear();
        self.cursor = None;
        self.anchor = None;
        self.scroll = 0;
        self.preview = None;
        Self::notify(&self.config.on_navigate, vec![path.clone()]);
        self.request(path, false);
    }

    fn expand(&mut self, path: PathBuf) {
        if self.expanded.remove(&path) {
            if self
                .cursor
                .as_ref()
                .is_some_and(|cursor| cursor != &path && cursor.starts_with(&path))
            {
                self.cursor = Some(path);
            }
        } else {
            self.expanded.insert(path.clone());
            if !self.directories.contains_key(&path) {
                self.request(path, false);
            }
        }
        self.rebuild();
    }

    fn activate(&mut self, path: PathBuf) {
        let entry = self
            .rows
            .iter()
            .find(|row| row.entry.path == path)
            .map(|row| &row.entry);
        match entry.map(|entry| &entry.file_type) {
            Some(FileType::Directory) => {
                if self.config.view_mode == ViewMode::Tree {
                    self.expand(path);
                } else {
                    self.navigate(path);
                }
            }
            Some(_) => Self::notify(&self.config.on_activate, vec![path]),
            None => (),
        }
    }

    fn action(&mut self, target: Target) {
        if self.pending_operation {
            if target == Target::Cancel {
                if let Some(worker) = &self.worker {
                    worker.cancel();
                }
                self.status = Some("Cancellation requested…".into());
            }
            return;
        }
        if self.prompt.is_some() && !matches!(target, Target::Confirm | Target::Cancel) {
            return;
        }
        if self.error.is_some()
            && self.prompt.is_none()
            && !matches!(
                target,
                Target::Refresh | Target::Navigate(_) | Target::Cancel
            )
        {
            return;
        }
        match target {
            Target::Entry(path) => self.activate(path),
            Target::Expand(path) => self.expand(path),
            Target::Navigate(path) => self.navigate(path),
            Target::View => {
                self.config.view_mode = match self.config.view_mode {
                    ViewMode::List => ViewMode::Grid,
                    ViewMode::Grid => ViewMode::Tree,
                    ViewMode::Tree => ViewMode::List,
                }
            }
            Target::Sort => {
                self.config.sort_criteria = match self.config.sort_criteria {
                    SortCriteria::Name => SortCriteria::Size,
                    SortCriteria::Size => SortCriteria::Modified,
                    SortCriteria::Modified => SortCriteria::Type,
                    SortCriteria::Type => SortCriteria::Name,
                }
            }
            Target::Order => {
                self.config.sort_order = if self.config.sort_order == SortOrder::Ascending {
                    SortOrder::Descending
                } else {
                    SortOrder::Ascending
                }
            }
            Target::Hidden => self.config.show_hidden = !self.config.show_hidden,
            Target::Search => self.search_edit = true,
            Target::Preview => {
                self.config.show_preview = !self.config.show_preview;
                self.request_pending_content();
            }
            Target::Refresh => {
                self.directories
                    .retain(|path, _| path == &self.config.current_path);
                self.preview = None;
                self.request(self.config.current_path.clone(), false);
            }
            Target::Operation(operation) => {
                let mut sources: Vec<_> = self.selected.iter().cloned().collect();
                if sources.is_empty() {
                    sources.extend(self.cursor.clone());
                }
                sources.sort();
                if sources.is_empty() {
                    self.error = Some("Select a file first".into());
                } else if operation == worker::Operation::Rename && sources.len() > 1 {
                    self.error = Some("Rename requires one selected entry".into());
                } else {
                    self.prompt = Some(Prompt {
                        operation,
                        sources,
                        destination: String::new(),
                    });
                }
            }
            Target::Confirm => {
                if let Some(prompt) = self.prompt.take() {
                    if prompt.operation != worker::Operation::Delete
                        && prompt.destination.is_empty()
                    {
                        self.error = Some("Enter a destination path".into());
                        self.prompt = Some(prompt);
                    } else if let Some(worker) = &self.worker {
                        let destination = if prompt.operation == worker::Operation::Delete {
                            None
                        } else if prompt.operation == worker::Operation::Rename {
                            let name = Path::new(&prompt.destination);
                            if name.components().count() != 1
                                || !matches!(
                                    name.components().next(),
                                    Some(std::path::Component::Normal(_))
                                )
                            {
                                self.error =
                                    Some("Rename needs a file name; use Move for a path".into());
                                self.prompt = Some(prompt);
                                return;
                            }
                            Some(
                                prompt.sources[0]
                                    .parent()
                                    .unwrap_or(&self.config.current_path)
                                    .join(name),
                            )
                        } else {
                            Some(self.config.current_path.join(prompt.destination))
                        };
                        self.pending = Some(worker.submit(
                            self.previous.root_path.clone(),
                            worker::Job::Mutate {
                                operation: prompt.operation,
                                sources: prompt.sources,
                                destination,
                            },
                        ));
                        self.pending_preview = false;
                        self.pending_path = None;
                        self.pending_operation = true;
                        self.error = None;
                        self.status = Some("Working… Esc cancels".into());
                    }
                }
            }
            Target::Cancel => {
                self.prompt = None;
                self.error = None;
                if std::mem::take(&mut self.refresh_after_error) {
                    self.directories.clear();
                    self.request(self.config.current_path.clone(), false);
                }
            }
        }
        self.rebuild();
        self.reveal();
    }

    fn key(&mut self, key: &KeyEvent) -> EventResult {
        if key.kind == KeyEventKind::Release {
            return EventResult::Ignored;
        }
        if self.config.max_visible_items == 0 {
            return EventResult::Ignored;
        }
        if self.pending_operation {
            if key.code == KeyCode::Escape {
                self.action(Target::Cancel);
            }
            return EventResult::Consumed;
        }
        if self.error.is_some() && self.prompt.is_none() {
            match key.code {
                KeyCode::Escape => self.action(Target::Cancel),
                KeyCode::F(5) if self.config.keyboard_navigation => self.action(Target::Refresh),
                _ => return EventResult::Ignored,
            }
            return EventResult::Consumed;
        }
        if self.prompt.is_some() {
            match key.code {
                KeyCode::Escape => self.action(Target::Cancel),
                KeyCode::Enter => self.action(Target::Confirm),
                KeyCode::Backspace => {
                    let text = &mut self.prompt.as_mut().unwrap().destination;
                    if let Some((index, _)) = text.grapheme_indices(true).next_back() {
                        text.truncate(index);
                    }
                }
                KeyCode::Char('u') if key.modifiers.ctrl => {
                    self.prompt.as_mut().unwrap().destination.clear()
                }
                KeyCode::Char(c) if !key.modifiers.ctrl && !key.modifiers.alt => {
                    self.prompt.as_mut().unwrap().destination.push(c)
                }
                _ => return EventResult::Ignored,
            }
            return EventResult::Consumed;
        }
        if self.search_edit {
            match &key.code {
                KeyCode::Escape | KeyCode::Enter => self.search_edit = false,
                KeyCode::Backspace => {
                    let query = self.config.search_query.get_or_insert_default();
                    if let Some((index, _)) = query.grapheme_indices(true).next_back() {
                        query.truncate(index);
                    }
                }
                KeyCode::Char(c) if !key.modifiers.ctrl && !key.modifiers.alt => {
                    self.config.search_query.get_or_insert_default().push(*c)
                }
                _ => return EventResult::Ignored,
            }
            self.scroll = 0;
            self.rebuild();
            return EventResult::Consumed;
        }
        if !self.config.keyboard_navigation {
            return EventResult::Ignored;
        }
        let action = match key.code {
            KeyCode::Char('/') => Some(Target::Search),
            KeyCode::Char('v') if !key.modifiers.ctrl => Some(Target::View),
            KeyCode::Char('s') if !key.modifiers.ctrl => Some(Target::Sort),
            KeyCode::Char('o') if !key.modifiers.ctrl => Some(Target::Order),
            KeyCode::Char('.') => Some(Target::Hidden),
            KeyCode::Char('p') if !key.modifiers.ctrl => Some(Target::Preview),
            KeyCode::F(5) => Some(Target::Refresh),
            KeyCode::F(2) => Some(Target::Operation(worker::Operation::Rename)),
            KeyCode::F(6) => Some(Target::Operation(worker::Operation::Move)),
            KeyCode::F(7) => Some(Target::Operation(worker::Operation::Copy)),
            KeyCode::Delete => Some(Target::Operation(worker::Operation::Delete)),
            _ => None,
        };
        if let Some(action) = action {
            self.action(action);
            return EventResult::Consumed;
        }
        if key.code == KeyCode::Backspace {
            let path = &self.config.current_path;
            if path != &self.config.root_path {
                if let Some(parent) = path
                    .parent()
                    .filter(|parent| parent.starts_with(&self.config.root_path))
                {
                    self.navigate(parent.to_path_buf());
                }
            }
            return EventResult::Consumed;
        }
        if self.rows.is_empty() {
            return EventResult::Ignored;
        }
        let index = self
            .rows
            .iter()
            .position(|row| Some(&row.entry.path) == self.cursor.as_ref())
            .unwrap_or(0);
        let columns = self.columns();
        let target = match key.code {
            KeyCode::Up => Some(index.saturating_sub(columns)),
            KeyCode::Down => Some(index.saturating_add(columns).min(self.rows.len() - 1)),
            KeyCode::Home => Some(0),
            KeyCode::End => Some(self.rows.len() - 1),
            KeyCode::PageUp => Some(index.saturating_sub(self.visible_rows().max(1) * columns)),
            KeyCode::PageDown => Some(
                index
                    .saturating_add(self.visible_rows().max(1) * columns)
                    .min(self.rows.len() - 1),
            ),
            KeyCode::Left if columns > 1 => Some(index.saturating_sub(1)),
            KeyCode::Right if columns > 1 => Some((index + 1).min(self.rows.len() - 1)),
            _ => None,
        };
        if let Some(index) = target {
            self.select(
                self.rows[index].entry.path.clone(),
                key.modifiers.shift,
                false,
            );
            return EventResult::Consumed;
        }
        let path = self.rows[index].entry.path.clone();
        match key.code {
            KeyCode::Enter => self.activate(path),
            KeyCode::Char(' ') => self.select(path, key.modifiers.shift, true),
            KeyCode::Right if self.config.view_mode == ViewMode::Tree => {
                if !self.expanded.contains(&path)
                    && self.rows[index].entry.file_type == FileType::Directory
                {
                    self.expand(path);
                }
            }
            KeyCode::Left if self.config.view_mode == ViewMode::Tree => {
                self.expanded.remove(&path);
                self.rebuild();
            }
            KeyCode::Char('a')
                if key.modifiers.ctrl && self.config.selection_mode == SelectionMode::Multiple =>
            {
                let before = self.selected.clone();
                self.selected = self.rows.iter().map(|row| row.entry.path.clone()).collect();
                if self.selected != before {
                    Self::notify(
                        &self.config.on_select,
                        self.rows.iter().map(|row| row.entry.path.clone()).collect(),
                    );
                }
            }
            _ => return EventResult::Ignored,
        }
        EventResult::Consumed
    }

    fn target(&self, mouse: &MouseEvent) -> Option<Target> {
        let [a, b, c, d, tx, ty] = self.viewport?.transform;
        let (x, y) = (mouse.position.x() as f32, mouse.position.y() as f32);
        let (x, y) = (a * x + c * y + tx, b * x + d * y + ty);
        self.targets
            .lock()
            .unwrap()
            .iter()
            .find_map(|(target, layout)| {
                let clip = layout.clip;
                (x >= clip.x
                    && y >= clip.y
                    && x < clip.x + clip.width
                    && y < clip.y + clip.height
                    && layout.local_cell(x, y).is_some())
                .then(|| target.clone())
            })
    }
}

impl Explorer {
    fn new(props: FileExplorerProps) -> Self {
        let mut config = props.clone();
        if config.current_path == Path::new("/") && config.root_path != Path::new("/") {
            config.current_path = config.root_path.clone();
        }
        let (worker, error) = match worker::Worker::new() {
            Ok(worker) => (Some(worker), None),
            Err(error) => (
                None,
                Some(format!("Cannot start filesystem worker: {error}")),
            ),
        };
        Self {
            previous: props,
            config,
            worker,
            pending: None,
            pending_preview: false,
            pending_path: None,
            pending_operation: false,
            refresh_after_error: false,
            prompt: None,
            status: None,
            directories: HashMap::new(),
            expanded: HashSet::new(),
            rows: Vec::new(),
            selected: HashSet::new(),
            cursor: None,
            anchor: None,
            viewport: None,
            targets: Arc::default(),
            scroll: 0,
            error,
            preview: None,
            search_edit: false,
            last_press: None,
            focused: false,
        }
    }
    fn update(&mut self, props: &FileExplorerProps) -> bool {
        self.receive();
        if props != &self.previous {
            let changed_path = props.current_path != self.previous.current_path
                || props.root_path != self.previous.root_path;
            macro_rules! sync { ($($field:ident),* $(,)?) => { $(if props.$field != self.previous.$field { self.config.$field = props.$field.clone(); })* }; }
            sync!(
                root_path,
                current_path,
                show_hidden,
                file_filters,
                selection_mode,
                view_mode,
                sort_criteria,
                sort_order,
                show_preview,
                show_breadcrumb,
                show_details,
                search_query,
                class,
                keyboard_navigation,
                on_select,
                on_activate,
                on_navigate,
                max_visible_items
            );
            self.previous = props.clone();
            if changed_path {
                if self.pending_operation {
                    self.pending_operation = false;
                    self.status = Some("Path changed; earlier changes may remain".into());
                }
                self.prompt = None;
                self.directories.clear();
                self.expanded.clear();
                self.rows.clear();
                self.selected.clear();
                self.cursor = None;
                self.preview = None;
                self.scroll = 0;
                self.request(self.config.current_path.clone(), false);
            }
            if self.config.selection_mode == SelectionMode::None {
                self.selected.clear();
            }
            if self.config.selection_mode == SelectionMode::Single && self.selected.len() > 1 {
                self.selected = self.cursor.iter().cloned().collect();
            }
            self.rebuild();
        }
        self.request_pending_content();
        true
    }
    fn layout(&mut self, layout: LayoutInfo) -> bool {
        let changed = self.viewport != Some(layout);
        self.viewport = Some(layout);
        let received = self.receive();
        self.request_pending_content();
        if self
            .cursor
            .as_ref()
            .is_some_and(|path| self.selected.contains(path))
        {
            self.reveal();
        } else {
            self.clamp();
        }
        changed || received
    }
    fn handle_event(&mut self, event: &Event) -> EventResult {
        if self.config.max_visible_items == 0 {
            return EventResult::Ignored;
        }
        match event {
            Event::Focus(focus) => {
                self.focused = focus.kind == crate::event::types::FocusEventKind::Gained;
                self.last_press = None;
                EventResult::Consumed
            }
            Event::Custom(event) if event.name == "reactive_tui.file_explorer.focus" => {
                let path = self
                    .rows
                    .iter()
                    .find(|row| path_key(&row.entry.path).as_bytes() == event.data)
                    .map(|row| row.entry.path.clone());
                if let Some(path) = path {
                    self.cursor = Some(path);
                    self.focused = true;
                    self.reveal();
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            }
            Event::Paste(paste) if !self.pending_operation => {
                if let Some(prompt) = &mut self.prompt {
                    if prompt.operation != worker::Operation::Delete {
                        prompt.destination.push_str(&paste.content);
                    }
                    EventResult::Consumed
                } else if self.search_edit {
                    self.config
                        .search_query
                        .get_or_insert_default()
                        .push_str(&paste.content);
                    self.rebuild();
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            }
            Event::Key(key) => {
                self.last_press = None;
                self.key(key)
            }
            Event::Mouse(mouse) => {
                if mouse.kind == MouseEventKind::Wheel {
                    if let Some(wheel) = &mouse.wheel {
                        let (WheelDelta::Lines { y, .. } | WheelDelta::Pixels { y, .. }) =
                            wheel.delta;
                        self.scroll = (self.scroll as f64 + y as f64)
                            .max(0.0)
                            .min(usize::MAX as f64) as usize;
                        self.clamp();
                        return EventResult::Consumed;
                    }
                }
                if mouse.button != MouseButton::Left {
                    return EventResult::Ignored;
                }
                let Some(target) = self.target(mouse) else {
                    return EventResult::Ignored;
                };
                match mouse.kind {
                    MouseEventKind::Down => {
                        self.last_press = Some(target.clone());
                        if let Target::Entry(path) = target {
                            if self.prompt.is_none()
                                && self.error.is_none()
                                && !self.pending_operation
                            {
                                self.select(path, mouse.modifiers.shift, mouse.modifiers.ctrl);
                            }
                        } else {
                            self.action(target);
                        }
                    }
                    MouseEventKind::Click => {
                        if self.last_press.take().as_ref() == Some(&target) {
                            return EventResult::Consumed;
                        }
                        if let Target::Entry(path) = target {
                            if self.prompt.is_none()
                                && self.error.is_none()
                                && !self.pending_operation
                            {
                                self.select(path, mouse.modifiers.shift, mouse.modifiers.ctrl);
                            }
                        } else {
                            self.action(target);
                        }
                    }
                    MouseEventKind::DoubleClick => {
                        if let Target::Entry(path) = target {
                            if self.prompt.is_none()
                                && self.error.is_none()
                                && !self.pending_operation
                            {
                                self.activate(path);
                            }
                        }
                    }
                    _ => return EventResult::Ignored,
                }
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
}

impl Component for LiveExplorer {
    type Props = LiveProps;
    type State = ();
    fn new(props: Self::Props) -> Self {
        let mut inner = Explorer::new(props.config);
        inner.apply_seed(&props.seed);
        Self {
            inner: Mutex::new(inner),
            seed: props.seed,
        }
    }
    fn initial_state(&mut self, _props: &Self::Props) {
        let inner = self.inner.get_mut().unwrap();
        if !self.seed.initialized && self.seed.entries.is_empty() {
            inner.request(inner.config.current_path.clone(), false);
        }
    }
    fn update(&mut self, props: &Self::Props, _state: &mut ()) -> bool {
        let inner = self.inner.get_mut().unwrap();
        inner.update(&props.config);
        if self.seed != props.seed {
            self.seed = props.seed.clone();
            inner.apply_seed(&self.seed);
        }
        true
    }
    fn render(&self, _props: &Self::Props, _state: &()) -> Element {
        let mut inner = self.inner.lock().unwrap();
        inner.receive();
        inner.request_pending_content();
        inner.paint()
    }
    fn layout(&mut self, layout: LayoutInfo, _props: &mut Self::Props, _state: &mut ()) -> bool {
        self.inner.get_mut().unwrap().layout(layout)
    }
    fn handle_event(
        &mut self,
        event: &Event,
        _props: &mut Self::Props,
        _state: &mut (),
    ) -> EventResult {
        self.inner.get_mut().unwrap().handle_event(event)
    }
}
