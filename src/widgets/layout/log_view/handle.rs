//! The shared rows of a log view (NAV-009): a handle any thread may change,
//! which wakes the App whose latest frame showed a view of it.

use super::LogRow;
use crate::reactive::wake::Subscriptions;
use std::sync::{Arc, Mutex, MutexGuard};

/// The rows a handle holds. Each row has an id that never changes while it
/// is held: an appended row takes the id after the last, a prepended one
/// the id before the first, so a view keeps its place by id.
#[derive(Debug, Default)]
pub(super) struct Rows {
    /// The rows, oldest first.
    pub(super) rows: Vec<LogRow>,
    /// The id of the first row.
    pub(super) first: i64,
    /// How many times the rows were cleared: an id read before a clear
    /// names no row after it.
    pub(super) epoch: u64,
}

impl Rows {
    /// The id of the last row; one before the first when there is none.
    pub(super) fn last(&self) -> i64 {
        self.first + self.rows.len() as i64 - 1
    }
}

/// What a handle shares between its clones.
#[derive(Default)]
struct Shared {
    rows: Mutex<Rows>,
    /// The Apps whose latest frame showed a view of these rows.
    readers: Subscriptions,
}

/// The shared rows of a log view. A clone holds the same rows, so a clone
/// may change them from any thread; each change wakes the App whose latest
/// frame showed a view of the handle, which redraws it. A handle no view
/// shows keeps its rows and wakes nothing.
#[derive(Clone, Default)]
pub struct LogHandle {
    shared: Arc<Shared>,
}

impl std::fmt::Debug for LogHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LogHandle")
            .field("rows", &self.len())
            .finish()
    }
}

/// Two handles are equal when they hold the same rows.
impl PartialEq for LogHandle {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.shared, &other.shared)
    }
}

impl LogHandle {
    /// Create a handle with no rows.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a row at the end.
    pub fn append(&self, row: LogRow) {
        self.change(|rows| {
            rows.rows.push(row);
            true
        });
    }

    /// Add rows at the end, in order.
    pub fn append_all(&self, rows: Vec<LogRow>) {
        self.change(|held| {
            let any = !rows.is_empty();
            held.rows.extend(rows);
            any
        });
    }

    /// Add rows above the existing rows, keeping their order.
    pub fn prepend(&self, rows: Vec<LogRow>) {
        self.change(|held| {
            if rows.is_empty() {
                return false;
            }
            held.first -= rows.len() as i64;
            let newer = std::mem::replace(&mut held.rows, rows);
            held.rows.extend(newer);
            true
        });
    }

    /// Remove every row.
    pub fn clear(&self) {
        self.change(|rows| {
            if rows.rows.is_empty() {
                return false;
            }
            rows.first += rows.rows.len() as i64;
            rows.rows.clear();
            rows.epoch += 1;
            true
        });
    }

    /// The number of rows.
    pub fn len(&self) -> usize {
        self.lock().rows.len()
    }

    /// Whether the handle holds no rows.
    pub fn is_empty(&self) -> bool {
        self.lock().rows.is_empty()
    }

    /// Changes the rows with `change`, which says whether it changed them,
    /// then wakes the Apps that show them, with the rows' lock released.
    fn change(&self, change: impl FnOnce(&mut Rows) -> bool) {
        let changed = change(&mut self.lock());
        if changed {
            self.shared.readers.notify();
        }
    }

    /// Reads the rows for the App rendering now, which a later change wakes.
    pub(super) fn read<R>(&self, read: impl FnOnce(&Rows) -> R) -> R {
        let rows = self.lock();
        self.shared.readers.track();
        read(&rows)
    }

    /// The rows, locked. A poisoned lock is read anyway: the rows are plain
    /// data, so a panic elsewhere cannot leave them half-written.
    fn lock(&self) -> MutexGuard<'_, Rows> {
        self.shared
            .rows
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
