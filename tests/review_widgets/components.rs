use std::sync::{Arc, Mutex};

use crate::common::app_input::{self, click, key};
use reactive_tui::{
    app::RootComponent,
    component::{ComponentInstance, Element},
    event::{
        router::EventResult,
        types::{Event, KeyCode},
    },
    widgets::display::{
        table::{Table, TableColumn, TableProps, TableRow},
        DataTable, DataTableProps, Tree, TreeNode, TreeProps,
    },
};

/// The tags of the callbacks that ran, in order.
type Log = Arc<Mutex<Vec<&'static str>>>;

/// A root that renders a widget from props it keeps, and on F2 or F3 swaps
/// one callback of those props, to a new one or to none, changing nothing
/// else; the widget's instance stays mounted under the same identity.
struct Swap<P: Clone + Send + Sync + 'static> {
    props: Mutex<P>,
    swap: fn(&mut P, Option<&'static str>),
    render: fn(P) -> Element,
}

impl<P: Clone + Send + Sync + 'static> RootComponent for Swap<P> {
    fn render(&self) -> Element {
        (self.render)(self.props.lock().unwrap().clone())
    }
    fn handle_event(&self, event: &Event) -> EventResult {
        let Event::Key(key) = event else {
            return EventResult::Ignored;
        };
        let tag = match key.code {
            KeyCode::F(2) => Some("B"),
            KeyCode::F(3) => None,
            _ => return EventResult::Ignored,
        };
        (self.swap)(&mut self.props.lock().unwrap(), tag);
        EventResult::Consumed
    }
    fn wake_driven(&self) -> bool {
        true
    }
}

fn logging<T: 'static>(log: &Log, tag: &'static str) -> Arc<dyn Fn(T) + Send + Sync> {
    let log = log.clone();
    Arc::new(move |_| log.lock().unwrap().push(tag))
}

fn logging_str(log: &Log, tag: &'static str) -> Arc<dyn Fn(&str) + Send + Sync> {
    let log = log.clone();
    Arc::new(move |_: &str| log.lock().unwrap().push(tag))
}

fn table_props(log: &Log) -> TableProps {
    TableProps {
        columns: vec![TableColumn::new("Name", "name")],
        rows: vec![
            TableRow::new("a").with_cell("name", "alpha"),
            TableRow::new("b").with_cell("name", "beta"),
            TableRow::new("c").with_cell("name", "gamma"),
        ],
        selectable: true,
        on_select: Some(logging(log, "A")),
        ..Default::default()
    }
}

fn tree_props(log: &Log) -> TreeProps {
    TreeProps {
        root: Some(
            TreeNode::new("root", "root")
                .with_children(vec![TreeNode::new("x", "x"), TreeNode::new("y", "y")]),
        ),
        selectable: true,
        on_select: Some(logging(log, "A")),
        ..Default::default()
    }
}

fn data_table_props(log: &Log) -> DataTableProps {
    DataTableProps {
        table_props: TableProps {
            on_select: None,
            ..table_props(log)
        },
        exportable: true,
        on_export: Some(logging_str(log, "A")),
        ..Default::default()
    }
}

/// CMP-008: a table's `on_select` swapped by a rerender that changes
/// nothing else is the one the next selection calls; swapped to none, the
/// next selection calls nothing.
#[test]
fn cmp_008_a_tables_replaced_callback_is_called_from_the_next_selection() {
    let log: Log = Arc::default();
    let swapped = log.clone();
    let root = Swap {
        props: Mutex::new(table_props(&log)),
        swap: |props: &mut TableProps, tag| {
            props.on_select = tag.map(|tag| {
                let log = SWAP_LOG.with(|l| l.borrow().clone().unwrap());
                let _ = tag;
                logging::<Option<usize>>(&log, "B")
            });
        },
        render: |props| Table::with_props(props).auto_focus(),
    };
    SWAP_LOG.with(|l| *l.borrow_mut() = Some(swapped));
    app_input::run(
        root,
        (30, 8),
        vec![
            (2, key(KeyCode::F(2))),
            (3, key(KeyCode::Down)),
            (4, key(KeyCode::Enter)),
            (5, key(KeyCode::F(3))),
            (6, key(KeyCode::Down)),
            (7, key(KeyCode::Enter)),
            (8, None),
        ],
    );
    assert_eq!(*log.lock().unwrap(), vec!["B"]);
}

thread_local! {
    /// The log a swap closure logs to: a `fn` pointer cannot capture it.
    static SWAP_LOG: std::cell::RefCell<Option<Log>> = const { std::cell::RefCell::new(None) };
}

/// CMP-008 for a tree's `on_select`.
#[test]
fn cmp_008_a_trees_replaced_callback_is_called_from_the_next_selection() {
    let log: Log = Arc::default();
    SWAP_LOG.with(|l| *l.borrow_mut() = Some(log.clone()));
    let root = Swap {
        props: Mutex::new(tree_props(&log)),
        swap: |props: &mut TreeProps, tag| {
            props.on_select = tag.map(|_| {
                let log = SWAP_LOG.with(|l| l.borrow().clone().unwrap());
                logging::<Option<String>>(&log, "B")
            });
        },
        render: |props| Tree::with_props(props).auto_focus(),
    };
    app_input::run(
        root,
        (30, 8),
        vec![
            (2, key(KeyCode::F(2))),
            (3, key(KeyCode::Down)),
            (4, key(KeyCode::Enter)),
            (5, key(KeyCode::F(3))),
            (6, key(KeyCode::Down)),
            (7, key(KeyCode::Enter)),
            (8, None),
        ],
    );
    assert_eq!(*log.lock().unwrap(), vec!["B"]);
}

/// CMP-008 for a data table's `on_export`, pressed on its CSV button.
#[test]
fn cmp_008_a_data_tables_replaced_export_callback_is_the_one_pressed() {
    let log: Log = Arc::default();
    SWAP_LOG.with(|l| *l.borrow_mut() = Some(log.clone()));
    let root = Swap {
        props: Mutex::new(data_table_props(&log)),
        swap: |props: &mut DataTableProps, tag| {
            props.on_export = tag.map(|_| {
                let log = SWAP_LOG.with(|l| l.borrow().clone().unwrap());
                logging_str(&log, "B")
            });
        },
        render: |props| Element::typed::<DataTable>(props),
    };
    // The CSV button's cell, from a painted frame.
    let frames = app_input::run(
        Swap {
            props: Mutex::new(data_table_props(&log)),
            swap: |_, _| {},
            render: |props| Element::typed::<DataTable>(props),
        },
        (40, 10),
        vec![(2, None)],
    );
    let (column, row) = frames
        .last()
        .unwrap()
        .text
        .lines()
        .enumerate()
        .find_map(|(row, line)| line.find("CSV").map(|column| (column as u16, row as u16)))
        .expect("the CSV button is painted");
    app_input::run(
        root,
        (40, 10),
        vec![
            (2, key(KeyCode::F(2))),
            (3, click(column, row)),
            (4, key(KeyCode::F(3))),
            (5, click(column, row)),
            (6, None),
        ],
    );
    assert_eq!(*log.lock().unwrap(), vec!["B"]);
}

/// CMP-008: at the instance, equal props with a new callback are adopted
/// without a re-render being asked for, and a removed callback is gone.
#[test]
fn cmp_008_an_instance_adopts_a_callback_without_asking_to_re_render() {
    let log: Log = Arc::default();
    let mut instance = ComponentInstance::<Table>::new(table_props(&log));
    let mut replaced = table_props(&log);
    replaced.on_select = Some(logging(&log, "B"));
    assert!(
        !instance.update_props(replaced),
        "equal props with a new callback ask for no re-render"
    );
    instance.props().on_select.clone().unwrap()(Some(0));
    assert_eq!(*log.lock().unwrap(), vec!["B"]);
    let mut removed = table_props(&log);
    removed.on_select = None;
    assert!(!instance.update_props(removed));
    assert!(instance.props().on_select.is_none());
}
