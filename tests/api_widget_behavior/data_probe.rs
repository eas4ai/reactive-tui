//! Data controls for genuine Orca interaction and speech checks.
use reactive_tui::{
    component::Element,
    widgets::display::{
        table::{Table, TableColumn, TableProps, TableRow},
        DisplaySize,
    },
};

pub(super) fn table() -> Element {
    let mut locked = TableRow::new("locked").with_cell("name", "Unavailable member");
    locked.selectable = false;
    let mut props = TableProps {
        columns: vec![TableColumn::new("Member", "name").with_width(DisplaySize::Flex(1.0))],
        rows: vec![
            TableRow::new("ada").with_cell("name", "Ada member"),
            locked,
            TableRow::new("bea").with_cell("name", "Bea member"),
        ],
        sortable: true,
        ..Default::default()
    };
    props.border.enabled = false;
    Element::typed::<Table>(props)
        .with_accessibility_label("Project members")
        .auto_focus()
}

pub(super) fn data_table() -> Element {
    use reactive_tui::widgets::display::{DataTable, DataTableProps};
    let mut props = DataTableProps::new(
        vec![TableColumn::new("Customer", "name").with_width(DisplaySize::Flex(1.0))],
        ["Ada customer", "Bea customer", "Cam customer"]
            .iter()
            .enumerate()
            .map(|(i, name)| TableRow::new(&i.to_string()).with_cell("name", name))
            .collect(),
    )
    .with_features(true, false, false)
    .with_pagination(true, 2);
    props.column_visibility_control = false;
    props.table_props.border.enabled = false;
    Element::typed::<DataTable>(props).with_accessibility_label("Customer records")
}

pub(super) fn tree() -> Element {
    use reactive_tui::widgets::display::tree::{Tree, TreeNode, TreeProps};
    let mut guide = TreeNode::new("guide", "User guide");
    guide.checkable = true;
    guide.checked = Some(false);
    let mut locked = TreeNode::new("locked", "Unavailable folder");
    locked.selectable = false;
    locked.expandable = false;
    locked.checkable = false;
    let mut props = TreeProps {
        root: Some(
            TreeNode::new("root", "Project folders")
                .expanded(true)
                .children(vec![
                    TreeNode::new("docs", "Documentation folder").children(vec![guide]),
                    locked,
                    TreeNode::new("tests", "Test folder"),
                ]),
        ),
        show_icons: false,
        show_lines: false,
        ..Default::default()
    };
    props.border.enabled = false;
    Element::typed::<Tree>(props)
        .with_accessibility_label("Workspace tree")
        .auto_focus()
}

pub(super) fn files_fixture() -> std::io::Result<tempfile::TempDir> {
    let directory = tempfile::tempdir()?;
    std::fs::create_dir(directory.path().join("Docs"))?;
    std::fs::write(directory.path().join("Docs/guide.txt"), "Guide contents")?;
    std::fs::write(directory.path().join("alpha.txt"), "Alpha contents")?;
    Ok(directory)
}

pub(super) fn files(path: &std::path::Path) -> Element {
    reactive_tui::builder::file_explorer()
        .root_path(path)
        .show_details(false)
        .build()
        .with_accessibility_label("Project files")
        .auto_focus()
}

pub(super) fn tabs() -> Element {
    use reactive_tui::widgets::layout::{Tab, TabsBuilder};
    TabsBuilder::new()
        .add_tab("Overview", Element::text("Project overview"))
        .tab(Tab::new("Unavailable", Element::text("Forbidden panel")).disabled(true))
        .add_tab(
            "Settings",
            reactive_tui::builder::text_input()
                .value("seed")
                .build()
                .with_accessibility_label("Project setting"),
        )
        .closable(true)
        .render()
        .with_accessibility_label("Project sections")
        .auto_focus()
}
