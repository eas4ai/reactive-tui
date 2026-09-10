use super::*;
use crate::{
    builder,
    component::{ElementType, LayoutInfo, LayoutType},
    layout::style::StyleBuilder,
    widgets::{
        input::{TextInput, TextInputProps},
        layout::ScrollViewBuilder,
    },
};
use std::collections::HashSet;
use std::sync::Mutex;

#[path = "filters.rs"]
mod filters;

#[derive(Clone, PartialEq)]
pub(super) struct LiveProps {
    pub config: DataTableProps,
    pub seed: DataTableState,
}
impl Props for LiveProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

type FilterCallback = Arc<dyn Fn(Vec<ColumnFilter>) + Send + Sync>;
type SearchCallback = Arc<dyn Fn(String) + Send + Sync>;

#[derive(Clone, Default)]
pub(super) struct Model {
    filters: Vec<ColumnFilter>,
    search: String,
    page: usize,
    scroll_offset: usize,
    view_revision: u64,
    hidden: Vec<String>,
    sorts: Vec<(String, bool)>,
    selected: Option<String>,
    selections: Vec<String>,
    filter_panel: bool,
    column_panel: bool,
    inputs: HashMap<String, String>,
    kinds: HashMap<String, usize>,
    errors: HashMap<String, String>,
    on_filter_change: Option<FilterCallback>,
    on_search_change: Option<SearchCallback>,
}

impl Model {
    fn new(props: &DataTableProps, seed: &DataTableState) -> Self {
        let selected = props
            .table_props
            .selected_row
            .and_then(|index| props.table_props.rows.get(index))
            .map(|row| row.id.clone());
        let mut model = Self {
            filters: props.filters.clone(),
            search: props
                .search_query
                .clone()
                .unwrap_or_else(|| seed.search_input.clone()),
            page: props.pagination.current_page,
            scroll_offset: if seed.scroll_position != 0 {
                seed.scroll_position
            } else if props.virtual_scroll.enabled {
                props.virtual_scroll.scroll_offset
            } else {
                0
            },
            view_revision: 0,
            hidden: props.hidden_columns.clone(),
            sorts: props
                .table_props
                .sort_column
                .and_then(|i| props.table_props.columns.get(i))
                .map(|col| (col.key.clone(), props.table_props.sort_ascending))
                .into_iter()
                .collect(),
            selections: selected.iter().cloned().collect(),
            selected,
            filter_panel: seed.filter_panel_open,
            column_panel: seed.column_panel_open,
            inputs: seed.filter_inputs.clone(),
            kinds: HashMap::new(),
            errors: HashMap::new(),
            on_filter_change: props.on_filter_change.clone(),
            on_search_change: props.on_search_change.clone(),
        };
        model.load_filters();
        model.inputs.extend(seed.filter_inputs.clone());
        model
    }

    fn reset_scroll(&mut self) {
        self.scroll_offset = 0;
        self.view_revision = self.view_revision.wrapping_add(1);
    }

    fn reset_query(&mut self) {
        self.page = 0;
        self.reset_scroll();
    }

    fn load_filters(&mut self) {
        self.kinds.clear();
        self.inputs.clear();
        self.errors.clear();
        for filter in &self.filters {
            let (kind, value) = filters::format(&filter.filter_type);
            self.kinds.entry(filter.column_key.clone()).or_insert(kind);
            self.inputs
                .entry(filter.column_key.clone())
                .or_insert(value);
        }
    }

    fn query(&self, props: &DataTableProps) -> Vec<usize> {
        let query = self.search.to_lowercase();
        let mut rows: Vec<_> = props
            .table_props
            .rows
            .iter()
            .enumerate()
            .filter_map(|(index, row)| {
                let search = query.is_empty()
                    || row
                        .cells
                        .values()
                        .any(|cell| cell.content.to_lowercase().contains(&query));
                let filters = self
                    .filters
                    .iter()
                    .filter(|filter| filter.active)
                    .all(|filter| {
                        row.cells.get(&filter.column_key).is_some_and(|cell| {
                            DataTable.apply_filter_to_cell(cell, &filter.filter_type)
                        })
                    });
                (search && filters).then_some(index)
            })
            .collect();
        if !self.sorts.is_empty() {
            rows.sort_by(|&a, &b| {
                for (column, ascending) in &self.sorts {
                    let a = props.table_props.rows[a]
                        .cells
                        .get(column)
                        .map_or("", |cell| cell.content.as_str());
                    let b = props.table_props.rows[b]
                        .cells
                        .get(column)
                        .map_or("", |cell| cell.content.as_str());
                    let order = if *ascending { a.cmp(b) } else { b.cmp(a) };
                    if order != std::cmp::Ordering::Equal {
                        return order;
                    }
                }
                std::cmp::Ordering::Equal
            });
        }
        rows
    }
}

pub(super) struct LiveDataTable {
    previous: DataTableProps,
    seed: DataTableState,
    viewport: Option<LayoutInfo>,
}

fn button(label: &str, action: impl Fn() + Send + Sync + 'static) -> Element {
    builder::button()
        .text(label)
        .class("h-1 p-0 shrink-0")
        .on_click(action)
        .build()
}

fn row(children: Vec<Element>) -> Element {
    builder::ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
        .styles(StyleBuilder::new().gap_px(1.0, 0.0))
        .class("flex-row flex-wrap shrink-0 w-full")
        .children(children)
        .build()
}

fn input(value: String, label: String, action: impl Fn(String) + Send + Sync + 'static) -> Element {
    let action = Arc::new(action);
    Element::typed_with::<TextInput>(
        TextInputProps {
            value,
            placeholder: Some(label.clone()),
            ..Default::default()
        },
        move |props| {
            let action = action.clone();
            TextInput::new(props).with_on_change(move |value| action(value))
        },
    )
    .with_class("w-full h-1 p-0 shrink-0")
    .with_accessibility_label(label)
}

impl LiveDataTable {
    fn search(&self, model: &Model, shared: &Arc<Mutex<Model>>) -> Element {
        let shared = shared.clone();
        input(model.search.clone(), "Search table".into(), move |value| {
            let callback = {
                let mut model = shared.lock().unwrap();
                if model.search == value {
                    return;
                }
                model.search = value.clone();
                model.reset_query();
                model.on_search_change.clone()
            };
            if let Some(callback) = callback {
                callback(value);
            }
        })
        .with_key("search")
    }

    fn toolbar(&self, props: &DataTableProps, shared: &Arc<Mutex<Model>>) -> Element {
        let mut buttons = Vec::new();
        if props.show_filters {
            let shared = shared.clone();
            buttons.push(
                button("Filters", move || {
                    let mut model = shared.lock().unwrap();
                    model.filter_panel = !model.filter_panel;
                })
                .with_key("filters"),
            );
        }
        if props.column_visibility_control {
            let shared = shared.clone();
            buttons.push(
                button("Columns", move || {
                    let mut model = shared.lock().unwrap();
                    model.column_panel = !model.column_panel;
                })
                .with_key("columns"),
            );
        }
        if props.exportable {
            for format in ["csv", "json"] {
                let callback = props.on_export.clone();
                buttons.push(
                    button(&format.to_uppercase(), move || {
                        if let Some(callback) = &callback {
                            callback(format);
                        }
                    })
                    .with_key(format),
                );
            }
        }
        row(buttons).with_key("toolbar")
    }

    fn panel(&self, content: Element, key: &str) -> Element {
        let (width, height) = self
            .viewport
            .map_or((0.0, 0.0), |layout| layout.content_size());
        ScrollViewBuilder::new(content)
            .scroll_x(false)
            .viewport_size(
                width.max(0.0) as usize,
                (height.max(0.0) as usize / 2).max(1),
            )
            .render()
            .with_class("w-full shrink-0")
            .with_key(key)
    }

    fn columns(
        &self,
        props: &DataTableProps,
        model: &Model,
        shared: &Arc<Mutex<Model>>,
    ) -> Element {
        let children = props
            .table_props
            .columns
            .iter()
            .map(|column| {
                let hidden = model.hidden.contains(&column.key);
                let label = format!("[{}] {}", if hidden { " " } else { "x" }, column.title);
                let key = column.key.clone();
                let shared = shared.clone();
                let callback = props.on_column_visibility_change.clone();
                button(&label, move || {
                    let hidden = {
                        let mut model = shared.lock().unwrap();
                        if let Some(index) = model.hidden.iter().position(|value| value == &key) {
                            model.hidden.remove(index);
                        } else {
                            model.hidden.push(key.clone());
                        }
                        model.hidden.clone()
                    };
                    if let Some(callback) = &callback {
                        callback(hidden);
                    }
                })
                .with_key(format!("column:{}", column.key))
            })
            .collect();
        self.panel(
            Element::layout(LayoutType::Flex)
                .with_class("flex-col")
                .with_children(children),
            "column-panel",
        )
    }

    fn pages(
        &self,
        props: &DataTableProps,
        model: &Model,
        shared: &Arc<Mutex<Model>>,
        count: usize,
    ) -> Element {
        let total = count.div_ceil(props.pagination.page_size.max(1)).max(1);
        let current = model.page.min(total - 1);
        let mut children = Vec::new();
        for (label, page, enabled) in [
            ("Prev", current.saturating_sub(1), current > 0),
            (
                "Next",
                current.saturating_add(1).min(total - 1),
                current + 1 < total,
            ),
        ] {
            let shared = shared.clone();
            let callback = props.on_page_change.clone();
            children.push(
                button(label, move || {
                    let changed = {
                        let mut model = shared.lock().unwrap();
                        let changed = model.page != page;
                        model.page = page;
                        if changed {
                            model.reset_scroll();
                        }
                        changed
                    };
                    if changed {
                        if let Some(callback) = &callback {
                            callback(page);
                        }
                    }
                })
                .disabled(!enabled)
                .with_key(label),
            );
        }
        children.push(
            Element::text(format!("{}/{} ({count})", current + 1, total)).with_class("shrink-0"),
        );
        row(children).with_key("pagination")
    }

    fn table(
        &self,
        props: &DataTableProps,
        model: &Model,
        shared: &Arc<Mutex<Model>>,
        indices: &[usize],
    ) -> Element {
        let mut table = props.table_props.clone();
        table.rows = indices
            .iter()
            .map(|&index| props.table_props.rows[index].clone())
            .collect();
        let columns: Vec<_> = props
            .table_props
            .columns
            .iter()
            .enumerate()
            .filter(|(_, col)| !model.hidden.contains(&col.key))
            .map(|(index, _)| index)
            .collect();
        table.columns = columns
            .iter()
            .map(|&i| props.table_props.columns[i].clone())
            .collect();
        for column in &mut table.columns {
            if let Some((priority, (_, ascending))) = model
                .sorts
                .iter()
                .enumerate()
                .find(|(_, (key, _))| key == &column.key)
            {
                column.title.push_str(if *ascending { " ↑" } else { " ↓" });
                if model.sorts.len() > 1 {
                    column.title.push_str(&(priority + 1).to_string());
                }
            }
        }
        table.sort_column = None;
        table.selected_row = model
            .selected
            .as_ref()
            .and_then(|id| table.rows.iter().position(|row| &row.id == id));
        let selected_ids: HashSet<_> = model.selections.iter().collect();
        let seed = super::super::table::TableState {
            selected_row: table.selected_row,
            selected_rows: table
                .rows
                .iter()
                .enumerate()
                .filter_map(|(index, row)| selected_ids.contains(&row.id).then_some(index))
                .collect(),
            ..Default::default()
        };
        let indices = Arc::new(indices.to_vec());
        let ids: Arc<Vec<_>> = Arc::new(table.rows.iter().map(|row| row.id.clone()).collect());
        let state = shared.clone();
        let source = indices.clone();
        let row_ids = ids.clone();
        let callback = props.table_props.on_select.clone();
        table.on_select = Some(Arc::new(move |index| {
            let selected = index.and_then(|index| row_ids.get(index)).cloned();
            let changed = {
                let mut model = state.lock().unwrap();
                let changed = model.selected != selected;
                model.selections = selected.iter().cloned().collect();
                model.selected = selected;
                changed
            };
            if changed {
                if let Some(callback) = &callback {
                    callback(index.and_then(|index| source.get(index)).copied());
                }
            }
        }));
        let state = shared.clone();
        let row_ids = ids.clone();
        let callback = props.table_props.on_multi_select.clone();
        let all_ids: Arc<Vec<_>> = Arc::new(
            props
                .table_props
                .rows
                .iter()
                .map(|row| row.id.clone())
                .collect(),
        );
        table.on_multi_select = Some(Arc::new(move |selected| {
            let selected_sources = {
                let mut model = state.lock().unwrap();
                let visible: HashSet<_> = row_ids.iter().collect();
                model.selections.retain(|id| !visible.contains(id));
                model.selections.extend(
                    selected
                        .iter()
                        .filter_map(|&index| row_ids.get(index))
                        .cloned(),
                );
                let selected: HashSet<_> = model.selections.iter().collect();
                all_ids
                    .iter()
                    .enumerate()
                    .filter_map(|(index, id)| selected.contains(id).then_some(index))
                    .collect()
            };
            if let Some(callback) = &callback {
                callback(selected_sources);
            }
        }));
        let callback = props.table_props.on_row_action.clone();
        let source = indices.clone();
        table.on_row_action = Some(Arc::new(move |index, action| {
            if let (Some(callback), Some(&index)) = (&callback, source.get(index)) {
                callback(index, action);
            }
        }));
        let state = shared.clone();
        let callback = props.table_props.on_sort.clone();
        let column_keys: Vec<_> = props
            .table_props
            .columns
            .iter()
            .map(|column| column.key.clone())
            .collect();
        let sort = Arc::new(move |index: usize, extend: bool| {
            let Some(&source) = columns.get(index) else {
                return;
            };
            let key = &column_keys[source];
            let ascending = {
                let mut model = state.lock().unwrap();
                let ascending = model
                    .sorts
                    .iter()
                    .find(|(column, _)| column == key)
                    .is_none_or(|(_, ascending)| !ascending);
                if !extend {
                    model.sorts.clear();
                }
                if let Some((_, value)) = model.sorts.iter_mut().find(|(column, _)| column == key) {
                    *value = ascending;
                } else {
                    model.sorts.push((key.clone(), ascending));
                }
                model.reset_query();
                ascending
            };
            if let Some(callback) = &callback {
                callback(source, ascending);
            }
        });
        let window = Some((
            model.scroll_offset,
            if props.virtual_scroll.enabled {
                props.virtual_scroll.overscan
            } else {
                0
            },
            model.view_revision,
        ));
        if props.virtual_scroll.enabled {
            let rows = props
                .virtual_scroll
                .viewport_height
                .div_ceil(props.virtual_scroll.row_height.max(1));
            let maximum = rows
                .saturating_add(u16::from(table.show_header))
                .saturating_add(
                    if table.border.enabled && table.border.style != super::super::BorderStyle::None
                    {
                        2
                    } else {
                        0
                    },
                );
            table.max_height = Some(
                table
                    .max_height
                    .map_or(maximum, |height| height.min(maximum)),
            );
        }
        let state = shared.clone();
        let cursor = Arc::new(move |index: Option<usize>| {
            state.lock().unwrap().selected = index.and_then(|index| ids.get(index)).cloned();
        });
        super::super::table::data_view(table, seed, sort, cursor, window).with_key("table")
    }
}

impl Component for LiveDataTable {
    type Props = LiveProps;
    type State = Arc<Mutex<Model>>;

    fn new(props: Self::Props) -> Self {
        Self {
            previous: props.config,
            seed: props.seed,
            viewport: None,
        }
    }
    fn initial_state(&mut self, props: &Self::Props) -> Self::State {
        Arc::new(Mutex::new(Model::new(&props.config, &props.seed)))
    }
    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        let config = &props.config;
        let mut model = state.lock().unwrap();
        if self.previous.filters != config.filters {
            model.filters = config.filters.clone();
            model.load_filters();
            model.reset_query();
        }
        if self.previous.search_query != config.search_query {
            model.search = config.search_query.clone().unwrap_or_default();
            model.reset_query();
        }
        if self.previous.pagination != config.pagination {
            model.page = config.pagination.current_page;
            model.reset_scroll();
        }
        if self.previous.virtual_scroll.scroll_offset != config.virtual_scroll.scroll_offset {
            model.scroll_offset = config.virtual_scroll.scroll_offset;
            model.view_revision = model.view_revision.wrapping_add(1);
        }
        if self.seed.scroll_position != props.seed.scroll_position {
            model.scroll_offset = props.seed.scroll_position;
            model.view_revision = model.view_revision.wrapping_add(1);
        }
        if self.previous.hidden_columns != config.hidden_columns {
            model.hidden = config.hidden_columns.clone();
        }
        if self.previous.table_props.sort_column != config.table_props.sort_column
            || self.previous.table_props.sort_ascending != config.table_props.sort_ascending
        {
            model.sorts = config
                .table_props
                .sort_column
                .and_then(|index| config.table_props.columns.get(index))
                .map(|col| (col.key.clone(), config.table_props.sort_ascending))
                .into_iter()
                .collect();
            model.reset_query();
        }
        if self.previous.table_props.selected_row != config.table_props.selected_row {
            model.selected = config
                .table_props
                .selected_row
                .and_then(|index| config.table_props.rows.get(index))
                .map(|row| row.id.clone());
            model.selections = model.selected.iter().cloned().collect();
        }
        if self.seed.filter_panel_open != props.seed.filter_panel_open {
            model.filter_panel = props.seed.filter_panel_open;
        }
        if self.seed.column_panel_open != props.seed.column_panel_open {
            model.column_panel = props.seed.column_panel_open;
        }
        if self.seed.search_input != props.seed.search_input {
            model.search = props.seed.search_input.clone();
            model.reset_query();
        }
        model.on_filter_change = config.on_filter_change.clone();
        model.on_search_change = config.on_search_change.clone();
        let selectable: HashSet<_> = config
            .table_props
            .rows
            .iter()
            .filter(|row| row.selectable)
            .map(|row| &row.id)
            .collect();
        model.selections.retain(|id| selectable.contains(id));
        model.selected = model.selected.take().filter(|id| selectable.contains(id));
        self.previous = config.clone();
        self.seed = props.seed.clone();
        true
    }
    fn layout(
        &mut self,
        layout: LayoutInfo,
        _props: &mut Self::Props,
        _state: &mut Self::State,
    ) -> bool {
        let changed = self.viewport != Some(layout);
        self.viewport = Some(layout);
        changed
    }
    fn render(&self, props: &Self::Props, shared: &Self::State) -> Element {
        let props_config = &props.config;
        let mut model = shared.lock().unwrap().clone();
        let indices = model.query(props_config);
        if props_config.pagination.enabled {
            model.page = model.page.min(
                indices
                    .len()
                    .div_ceil(props_config.pagination.page_size.max(1))
                    .saturating_sub(1),
            );
            shared.lock().unwrap().page = model.page;
        }
        let mut children = Vec::new();
        if props_config.searchable {
            children.push(self.search(&model, shared));
        }
        if props_config.show_filters
            || props_config.column_visibility_control
            || props_config.exportable
        {
            children.push(self.toolbar(props_config, shared));
        }
        if props_config.show_filters && model.filter_panel {
            children.push(self.panel(filters::panel(props_config, &model, shared), "filter-panel"));
        }
        if props_config.column_visibility_control && model.column_panel {
            children.push(self.columns(props_config, &model, shared));
        }
        if props.seed.loading {
            children.push(Element::text("Loading…").with_key("loading"));
        }
        if let Some(error) = &props.seed.error_message {
            children.push(
                Element::text(error)
                    .with_class("text-red-500")
                    .with_key("error"),
            );
        }
        let table_error = super::super::table::validation_error(&props_config.table_props);
        let error = if props_config.pagination.enabled && props_config.pagination.page_size == 0 {
            Some("Table page size must be greater than zero")
        } else if props_config.virtual_scroll.enabled && props_config.virtual_scroll.row_height == 0
        {
            Some("Virtual row height must be greater than zero")
        } else {
            table_error.as_deref()
        };
        if let Some(error) = error {
            children.push(
                Element::text(error)
                    .with_class("text-red-500")
                    .with_key("configuration-error"),
            );
        } else {
            let range = if props_config.pagination.enabled {
                let page = PaginationConfig {
                    current_page: model.page,
                    total_rows: indices.len(),
                    ..props_config.pagination.clone()
                };
                page.start_index()..page.end_index()
            } else {
                0..indices.len()
            };
            children.push(self.table(props_config, &model, shared, &indices[range]));
        }
        if props_config.pagination.enabled && props_config.show_pagination {
            children.push(self.pages(props_config, &model, shared, indices.len()));
        }
        Element::layout(LayoutType::Flex)
            .with_class("flex-col w-full h-full min-w-0 min-h-0 overflow-hidden")
            .with_children(children)
    }
}
