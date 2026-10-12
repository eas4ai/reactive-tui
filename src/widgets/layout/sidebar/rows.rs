//! The rows a sidebar shows: its items in order, each open group followed
//! by its children (NAV-008, NAV-009).

use super::SidebarItem;

/// One item the sidebar shows, with its place in the tree.
pub(super) struct Row<'a> {
    /// The item.
    pub item: &'a SidebarItem,
    /// How deep it lies: 0 for a top-level item.
    pub level: usize,
    /// Its place among its siblings, from 1.
    pub position: usize,
    /// The count of its siblings, itself among them.
    pub count: usize,
    /// The row of the group that holds it.
    pub parent: Option<usize>,
    /// Whether it is a group whose children show.
    pub open: bool,
}

impl Row<'_> {
    /// Whether the item is a group: it has children.
    pub fn is_group(&self) -> bool {
        !self.item.children.is_empty()
    }
}

/// The rows of `items`: each item, and after an open group, as `open`
/// says, its children's rows.
pub(super) fn visible<'a>(
    items: &'a [SidebarItem],
    open: &dyn Fn(&SidebarItem) -> bool,
) -> Vec<Row<'a>> {
    let mut rows = Vec::new();
    push(&mut rows, items, 0, None, open);
    rows
}

fn push<'a>(
    rows: &mut Vec<Row<'a>>,
    items: &'a [SidebarItem],
    level: usize,
    parent: Option<usize>,
    open: &dyn Fn(&SidebarItem) -> bool,
) {
    for (index, item) in items.iter().enumerate() {
        let shown = !item.children.is_empty() && open(item);
        rows.push(Row {
            item,
            level,
            position: index + 1,
            count: items.len(),
            parent,
            open: shown,
        });
        if shown {
            let row = rows.len() - 1;
            push(rows, &item.children, level + 1, Some(row), open);
        }
    }
}

/// Whether any of `items` is a group, so any item has children: then
/// every row keeps a column for the expander, so labels line up.
pub(super) fn has_groups(items: &[SidebarItem]) -> bool {
    items.iter().any(|item| !item.children.is_empty())
}
