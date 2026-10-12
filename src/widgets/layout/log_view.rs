//! The log view (docs/spec/layout-widgets.md, NAV-010): a scrolling list of
//! log rows that follows the newest row while rows arrive, with a `↓ N new`
//! bar when the user has scrolled away.
//!
//! This is the skeleton of the widget: the rows and their handle are real,
//! and the view renders an empty element until the rendering is written.

use crate::component::{Component, Element, Props};
use std::any::Any;
use std::sync::{Arc, Mutex, MutexGuard};

/// How a log row is colored.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LogKind {
    /// Plain text in `foreground`.
    #[default]
    Plain,
    /// Information, in the info role's color.
    Info,
    /// A success, in the success role's color.
    Success,
    /// A warning, in the warning role's color.
    Warning,
    /// An error, in the error role's color.
    Error,
}

/// One row of a log view: its text and how it is colored.
#[derive(Clone, Debug, PartialEq)]
pub struct LogRow {
    /// The text of the row.
    pub text: String,
    /// How the row is colored.
    pub kind: LogKind,
}

impl LogRow {
    /// Create a plain row with the given text.
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            kind: LogKind::Plain,
        }
    }

    /// Set how the row is colored.
    pub fn kind(mut self, kind: LogKind) -> Self {
        self.kind = kind;
        self
    }
}

/// The shared rows of a log view. A clone holds the same rows, so a clone
/// may append from any thread and the view shows the change.
#[derive(Clone, Debug)]
pub struct LogHandle {
    rows: Arc<Mutex<Vec<LogRow>>>,
}

impl Default for LogHandle {
    fn default() -> Self {
        Self::new()
    }
}

/// Two handles are equal when they hold the same rows.
impl PartialEq for LogHandle {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.rows, &other.rows)
    }
}

impl LogHandle {
    /// Create a handle with no rows.
    pub fn new() -> Self {
        Self {
            rows: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add a row at the end.
    pub fn append(&self, row: LogRow) {
        // Skeleton: waking the App that shows the view is left to the
        // widget's job.
        self.lock().push(row);
    }

    /// Add rows at the end, in order.
    pub fn append_all(&self, rows: Vec<LogRow>) {
        self.lock().extend(rows);
    }

    /// Add rows above the existing rows, keeping their order.
    pub fn prepend(&self, rows: Vec<LogRow>) {
        let mut held = self.lock();
        let older = std::mem::take(&mut *held);
        *held = rows;
        held.extend(older);
    }

    /// Remove every row.
    pub fn clear(&self) {
        self.lock().clear();
    }

    /// The number of rows.
    pub fn len(&self) -> usize {
        self.lock().len()
    }

    /// Whether the handle holds no rows.
    pub fn is_empty(&self) -> bool {
        self.lock().is_empty()
    }

    /// The rows, locked. A poisoned lock is read anyway: the rows are plain
    /// data, so a panic elsewhere cannot leave them half-written.
    fn lock(&self) -> MutexGuard<'_, Vec<LogRow>> {
        self.rows
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// Properties for the log view.
#[derive(Clone, PartialEq)]
pub struct LogViewProps {
    /// The rows the view shows.
    pub handle: LogHandle,
    /// The word in the `↓ N new` bar; none is `new`.
    pub new_word: Option<String>,
    /// The name the screen reader hears for the view.
    pub aria_label: Option<String>,
    /// The view's width in cells; none takes what the parent allots.
    pub width: Option<u16>,
    /// The view's height in cells; none takes what the parent allots.
    pub height: Option<u16>,
}

impl Props for LogViewProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// A log view: the rows of a handle, following the newest one.
pub struct LogView;

impl LogView {
    /// Create a log view from its props.
    pub fn new(_props: LogViewProps) -> Self {
        Self
    }
}

impl Component for LogView {
    type Props = LogViewProps;
    type State = ();

    fn new(props: Self::Props) -> Self {
        Self::new(props)
    }

    fn render(&self, _props: &Self::Props, _state: &Self::State) -> Element {
        // Skeleton: the view's rendering is written by the widget's job.
        crate::builder::div().build()
    }
}
