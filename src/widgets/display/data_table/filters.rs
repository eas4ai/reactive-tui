use super::*;

const KINDS: [&str; 5] = ["contains", "equals", "range", "date", "boolean"];

pub(super) fn format(filter: &FilterType) -> (usize, String) {
    match filter {
        FilterType::Contains(value) => (0, value.clone()),
        FilterType::Equals(value) => (1, value.clone()),
        FilterType::Range(min, max) => (2, format!("{min}..{max}")),
        FilterType::DateRange(min, max) => (3, format!("{min}..{max}")),
        FilterType::Boolean(value) => (4, value.to_string()),
    }
}

fn parse(kind: usize, value: &str) -> Result<FilterType, String> {
    match kind {
        0 => Ok(FilterType::Contains(value.into())),
        1 => Ok(FilterType::Equals(value.into())),
        2 => {
            let (min, max) = value
                .split_once("..")
                .ok_or("Use min..max for a numeric range")?;
            let min: f64 = min.trim().parse().map_err(|_| "Invalid minimum number")?;
            let max: f64 = max.trim().parse().map_err(|_| "Invalid maximum number")?;
            if !min.is_finite() || !max.is_finite() || min > max {
                return Err("Range needs finite numbers with min <= max".into());
            }
            Ok(FilterType::Range(min, max))
        }
        3 => {
            let (min, max) = value.split_once("..").ok_or("Use YYYY-MM-DD..YYYY-MM-DD")?;
            if !super::super::valid_date(min) || !super::super::valid_date(max) || min > max {
                return Err("Use valid dates with start <= end".into());
            }
            Ok(FilterType::DateRange(min.into(), max.into()))
        }
        _ => match value.to_ascii_lowercase().as_str() {
            "true" | "yes" | "1" => Ok(FilterType::Boolean(true)),
            "false" | "no" | "0" => Ok(FilterType::Boolean(false)),
            _ => Err("Use true or false".into()),
        },
    }
}

fn change(shared: &Arc<Mutex<Model>>, column: &str, value: String, clear_empty: bool) {
    let result = {
        let mut model = shared.lock().unwrap();
        model.inputs.insert(column.into(), value.clone());
        let kind = *model.kinds.get(column).unwrap_or(&0);
        let filter = if clear_empty && value.is_empty() {
            Ok(None)
        } else {
            parse(kind, &value).map(Some)
        };
        match filter {
            Err(error) => {
                model.errors.insert(column.into(), error);
                None
            }
            Ok(filter) => {
                let previous = model.filters.clone();
                model.errors.remove(column);
                model.filters.retain(|filter| filter.column_key != column);
                if let Some(filter_type) = filter {
                    model.filters.push(ColumnFilter {
                        column_key: column.into(),
                        filter_type,
                        active: true,
                    });
                }
                model.reset_query();
                (previous != model.filters)
                    .then(|| (model.on_filter_change.clone(), model.filters.clone()))
            }
        }
    };
    if let Some((Some(callback), filters)) = result {
        callback(filters);
    }
}

pub(super) fn panel(props: &DataTableProps, model: &Model, shared: &Arc<Mutex<Model>>) -> Element {
    let mut children = Vec::new();
    for column in &props.table_props.columns {
        let kind = *model.kinds.get(&column.key).unwrap_or(&0);
        let key = column.key.clone();
        let state = shared.clone();
        let mode = button(&format!("{}: {}", column.title, KINDS[kind]), move || {
            let value = {
                let mut model = state.lock().unwrap();
                let kind = model.kinds.entry(key.clone()).or_insert(0);
                *kind = (*kind + 1) % KINDS.len();
                model.inputs.get(&key).cloned().unwrap_or_default()
            };
            change(&state, &key, value, true);
        })
        .with_key(format!("kind:{}", column.key));
        let key = column.key.clone();
        let state = shared.clone();
        let editor = input(
            model.inputs.get(&column.key).cloned().unwrap_or_default(),
            format!("Filter {}", column.title),
            move |value| change(&state, &key, value, true),
        )
        .with_key(format!("filter:{}", column.key));
        let active = model
            .filters
            .iter()
            .any(|filter| filter.column_key == column.key && filter.active);
        let key = column.key.clone();
        let state = shared.clone();
        let toggle = button(if active { "On" } else { "Off" }, move || {
            if active {
                let (callback, filters) = {
                    let mut model = state.lock().unwrap();
                    for filter in &mut model.filters {
                        if filter.column_key == key {
                            filter.active = false;
                        }
                    }
                    model.reset_query();
                    (model.on_filter_change.clone(), model.filters.clone())
                };
                if let Some(callback) = callback {
                    callback(filters);
                }
            } else {
                let value = state
                    .lock()
                    .unwrap()
                    .inputs
                    .get(&key)
                    .cloned()
                    .unwrap_or_default();
                change(&state, &key, value, false);
            }
        })
        .with_key(format!("active:{}", column.key));
        children.push(row(vec![mode, toggle]));
        children.push(editor);
        if let Some(error) = model.errors.get(&column.key) {
            children.push(
                Element::text(error)
                    .with_class("text-red-500")
                    .with_key(format!("error:{}", column.key)),
            );
        }
    }
    let state = shared.clone();
    children.push(
        button("Clear filters", move || {
            let callback = {
                let mut model = state.lock().unwrap();
                let changed = !model.filters.is_empty();
                model.filters.clear();
                model.inputs.clear();
                model.errors.clear();
                model.reset_query();
                changed.then(|| model.on_filter_change.clone()).flatten()
            };
            if let Some(callback) = callback {
                callback(Vec::new());
            }
        })
        .with_key("clear-filters"),
    );
    Element::layout(LayoutType::Flex)
        .with_class("flex-col w-full")
        .with_children(children)
}
