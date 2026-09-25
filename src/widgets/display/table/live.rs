use super::*;
use crate::{
    accessibility::{Node, Role},
    builder::ElementBuilder,
    component::{ElementType, FocusProps, LayoutInfo},
    event::types::{
        FocusEventKind, KeyEventKind, MouseButton, MouseEvent, MouseEventKind, WheelDelta,
    },
    layout::style::{Direction, StyleBuilder},
};
use std::sync::Mutex;

#[derive(Clone)]
pub(super) struct LiveProps {
    pub config: TableProps,
    pub seed: TableState,
    pub sort_request: Option<Arc<dyn Fn(usize, bool) + Send + Sync>>,
    pub cursor_change: Option<Arc<dyn Fn(Option<usize>) + Send + Sync>>,
    pub controlled_selection: bool,
    pub window: Option<(usize, usize, u64)>,
}

impl PartialEq for LiveProps {
    fn eq(&self, other: &Self) -> bool {
        self.config == other.config
            && self.seed.selected_row == other.seed.selected_row
            && self.seed.selected_rows == other.seed.selected_rows
            && self.seed.sort_column == other.seed.sort_column
            && self.seed.sort_ascending == other.seed.sort_ascending
            && self.window == other.window
            && self.controlled_selection == other.controlled_selection
    }
}

impl Props for LiveProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Clone, Copy)]
enum Target {
    Header(usize),
    Cell(usize, usize),
}

pub(super) struct LiveTable {
    viewport: Option<LayoutInfo>,
    previous: TableProps,
    seed: TableState,
    targets: Arc<Mutex<Vec<(Target, LayoutInfo)>>>,
    resized: HashMap<String, u16>,
    drag: Option<(usize, u32, u16)>,
    scroll_y: usize,
    public_scroll_y: u16,
    window: Option<(usize, usize, u64)>,
    error: Option<String>,
}

impl LiveTable {
    fn widths(&self, props: &TableProps) -> Vec<u16> {
        let width = self
            .viewport
            .map_or(0, |layout| layout.content_size().0.max(0.0) as u16);
        Table
            .calculate_column_widths(props, width)
            .into_iter()
            .enumerate()
            .map(|(i, width)| {
                let column = &props.columns[i];
                self.resized
                    .get(&column.key)
                    .copied()
                    .unwrap_or(width)
                    .max(column.min_width)
                    .min(column.max_width.unwrap_or(u16::MAX).max(column.min_width))
            })
            .collect()
    }

    fn clamp(&mut self, props: &TableProps, state: &mut TableState) {
        let (width, height) = self
            .viewport
            .map_or((0.0, 0.0), |layout| layout.content_size());
        state.column_widths = self.widths(props);
        state.scroll_state.viewport_width = width.max(0.0) as u16;
        state.scroll_state.viewport_height =
            (height.max(0.0) as u16).saturating_sub(u16::from(props.show_header));
        state.scroll_state.content_width = state
            .column_widths
            .iter()
            .fold(0u16, |sum, &width| sum.saturating_add(width));
        state.scroll_state.content_height = props.rows.len().min(u16::MAX as usize) as u16;
        let scroll = &mut state.scroll_state;
        scroll.offset_x = if props.scrollable {
            scroll
                .offset_x
                .min(scroll.content_width.saturating_sub(scroll.viewport_width))
        } else {
            0
        };
        if scroll.offset_y != self.public_scroll_y {
            self.scroll_y = scroll.offset_y as usize;
        }
        self.scroll_y = if props.scrollable {
            self.scroll_y.min(
                props
                    .rows
                    .len()
                    .saturating_sub(scroll.viewport_height as usize),
            )
        } else {
            0
        };
        scroll.offset_y = self.scroll_y.min(u16::MAX as usize) as u16;
        self.public_scroll_y = scroll.offset_y;
        let overscan = self.window.map_or(0, |(_, overscan, _)| overscan);
        state.visible_rows = Table
            .sort_rows(props, state)
            .into_iter()
            .skip(self.scroll_y.saturating_sub(overscan))
            .take(
                (state.scroll_state.viewport_height as usize)
                    .saturating_add(overscan)
                    .saturating_add(self.scroll_y.min(overscan)),
            )
            .collect();
    }

    fn reveal_selection(&mut self, props: &TableProps, state: &TableState) {
        if !props.scrollable {
            return;
        }
        if let Some(position) = state.selected_row.and_then(|row| {
            Table
                .sort_rows(props, state)
                .iter()
                .position(|&index| index == row)
        }) {
            let height = state.scroll_state.viewport_height.max(1) as usize;
            if position < self.scroll_y {
                self.scroll_y = position;
            } else if position >= self.scroll_y.saturating_add(height) {
                self.scroll_y = position.saturating_add(1).saturating_sub(height);
            }
            // The public compatibility state has a u16 offset; keep the full
            // source-row offset privately for tables with more than 65535 rows.
            self.public_scroll_y = state.scroll_state.offset_y;
        }
    }

    fn target_at(&self, mouse: &MouseEvent) -> Option<(Target, LayoutInfo)> {
        let root = self.viewport?;
        let [a, b, c, d, tx, ty] = root.transform;
        let (x, y) = (mouse.position.x() as f32, mouse.position.y() as f32);
        let (x, y) = (a * x + c * y + tx, b * x + d * y + ty);
        self.targets
            .lock()
            .unwrap()
            .iter()
            .copied()
            .find(|(_, layout)| {
                let clip = layout.clip;
                x >= clip.x
                    && y >= clip.y
                    && x < clip.x + clip.width
                    && y < clip.y + clip.height
                    && layout.local_cell(x, y).is_some()
            })
    }

    fn cell(
        &self,
        text: String,
        class: &str,
        width: u16,
        target: Target,
        alignment: &Alignment,
        node: Node,
    ) -> Element {
        let align = match alignment {
            Alignment::Center => "text-center",
            Alignment::End => "text-right",
            _ => "text-left",
        };
        let mut cell = ElementBuilder::new(ElementType::Text(text))
            .styles(
                StyleBuilder::new()
                    .width_px(width as f32)
                    .height_px(1.0)
                    .flex_shrink(0.0)
                    .overflow_hidden(),
            )
            .class(&format!("whitespace-pre truncate {align} {class}"))
            .build()
            .with_accessibility(node);
        let targets = self.targets.clone();
        cell.metadata.layout.push(Arc::new(move |layout| {
            targets.lock().unwrap().push((target, layout));
            false
        }));
        cell
    }

    fn row(&self, cells: Vec<Element>, y: isize, offset: u16, width: usize) -> Element {
        ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(
                StyleBuilder::new()
                    .display_flex()
                    .direction(Direction::Row)
                    .position_absolute()
                    .inset_left(-(offset as f32))
                    .inset_top(y as f32)
                    .height_px(1.0)
                    .width_px(width as f32),
            )
            .children(cells)
            .build()
    }

    fn sort(&self, props: &TableProps, state: &mut TableState, column: usize) -> EventResult {
        if !props.sortable || !props.columns.get(column).is_some_and(|col| col.sortable) {
            return EventResult::Ignored;
        }
        state.sort_ascending = state.sort_column != Some(column) || !state.sort_ascending;
        state.sort_column = Some(column);
        state.scroll_state.offset_y = 0;
        if let Some(callback) = &props.on_sort {
            callback(column, state.sort_ascending);
        }
        EventResult::Consumed
    }
}

impl Component for LiveTable {
    type Props = LiveProps;
    type State = TableState;

    fn new(props: Self::Props) -> Self {
        Self {
            error: validation_error(&props.config),
            scroll_y: props.window.map_or(
                props.seed.scroll_state.offset_y as usize,
                |(offset, _, _)| offset,
            ),
            public_scroll_y: props.seed.scroll_state.offset_y,
            window: props.window,
            viewport: None,
            previous: props.config,
            seed: props.seed,
            targets: Arc::default(),
            resized: HashMap::new(),
            drag: None,
        }
    }

    fn initial_state(&mut self, props: &Self::Props) -> Self::State {
        props.seed.clone()
    }

    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        let config = &props.config;
        self.error = validation_error(config);
        if self.window != props.window {
            if let Some((offset, _, _)) = props.window {
                self.scroll_y = offset;
            }
            self.window = props.window;
        }
        let reveal =
            self.previous.rows != config.rows || self.previous.selected_row != config.selected_row;
        let selectable: HashMap<_, _> = config
            .rows
            .iter()
            .enumerate()
            .filter(|(_, row)| row.selectable)
            .map(|(index, row)| (row.id.as_str(), index))
            .collect();
        let remap = |index: usize| {
            self.previous
                .rows
                .get(index)
                .and_then(|old| selectable.get(old.id.as_str()))
                .copied()
        };
        state.selected_rows = state
            .selected_rows
            .iter()
            .filter_map(|&index| remap(index))
            .collect();
        state.selected_row = state.selected_row.and_then(remap);
        if props.controlled_selection
            || self.seed.selected_row != props.seed.selected_row
            || self.seed.selected_rows != props.seed.selected_rows
        {
            state.selected_row = props.seed.selected_row;
            state.selected_rows = props.seed.selected_rows.clone();
        }
        if self.seed.sort_column != props.seed.sort_column
            || self.seed.sort_ascending != props.seed.sort_ascending
        {
            state.sort_column = props.seed.sort_column;
            state.sort_ascending = props.seed.sort_ascending;
        }
        if !props.controlled_selection && self.previous.selected_row != config.selected_row {
            state.selected_row = config
                .selected_row
                .filter(|&i| config.rows.get(i).is_some_and(|row| row.selectable));
            state.selected_rows = state.selected_row.into_iter().collect();
        }
        if self.previous.sort_column != config.sort_column
            || self.previous.sort_ascending != config.sort_ascending
        {
            state.sort_column = config.sort_column;
            state.sort_ascending = config.sort_ascending;
        } else if let Some(column) = state.sort_column.and_then(|i| self.previous.columns.get(i)) {
            state.sort_column = config.columns.iter().position(|col| col.key == column.key);
        }
        if !config.selectable {
            state.selected_rows.clear();
            state.selected_row = None;
        }
        self.resized
            .retain(|key, _| config.columns.iter().any(|col| &col.key == key));
        if self.previous.columns != config.columns || !config.resizable_columns {
            self.drag = None;
            state.resizing_column = None;
        }
        self.previous = config.clone();
        self.seed = props.seed.clone();
        self.clamp(config, state);
        if reveal {
            self.reveal_selection(config, state);
            self.clamp(config, state);
        }
        true
    }

    fn layout(
        &mut self,
        layout: LayoutInfo,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> bool {
        let initial = self.viewport.is_none();
        let changed = self.viewport != Some(layout);
        let before = (
            state.column_widths.clone(),
            state.visible_rows.clone(),
            state.scroll_state.offset_x,
            state.scroll_state.offset_y,
        );
        self.viewport = Some(layout);
        self.clamp(&props.config, state);
        if initial {
            self.reveal_selection(&props.config, state);
            self.clamp(&props.config, state);
        }
        changed
            || before
                != (
                    state.column_widths.clone(),
                    state.visible_rows.clone(),
                    state.scroll_state.offset_x,
                    state.scroll_state.offset_y,
                )
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        if let Some(error) = &self.error {
            return Element::text(error)
                .with_class("text-red-500 whitespace-normal break-words w-full");
        }
        let props = &props.config;
        self.targets.lock().unwrap().clear();
        let widths = self.widths(props);
        let total = widths.iter().map(|&width| width as usize).sum();
        let mut children = Vec::new();
        if props.show_header {
            let cells = props
                .columns
                .iter()
                .enumerate()
                .map(|(i, column)| {
                    let mut title = column.title.clone();
                    if state.sort_column == Some(i) {
                        title.push_str(if state.sort_ascending { " ↑" } else { " ↓" });
                    }
                    self.cell(
                        title,
                        props.header_style.as_deref().unwrap_or("font-bold"),
                        widths[i],
                        Target::Header(i),
                        &column.alignment,
                        Node::new(Role::ColumnHeader),
                    )
                    .with_key(format!("header:{}", column.key))
                })
                .collect();
            children.push(
                self.row(cells, 0, state.scroll_state.offset_x, total)
                    .with_key("header"),
            );
        }
        let buffered = self
            .window
            .map_or(0, |(_, overscan, _)| self.scroll_y.min(overscan));
        for (visible, &index) in state.visible_rows.iter().enumerate() {
            let row = &props.rows[index];
            let selected = state.selected_rows.contains(&index);
            let mut class = props.row_style.clone().unwrap_or_default();
            if props.zebra_striping && (self.scroll_y.saturating_sub(buffered) + visible) % 2 == 1 {
                class.push(' ');
                class.push_str(props.alternate_row_style.as_deref().unwrap_or_default());
            }
            if let Some(style) = &row.style {
                class.push(' ');
                class.push_str(style);
            }
            if selected {
                class.push(' ');
                class.push_str(props.selected_style.as_deref().unwrap_or_default());
            }
            let cells = props
                .columns
                .iter()
                .enumerate()
                .map(|(column, config)| {
                    let cell = row.cells.get(&config.key);
                    let class = format!(
                        "{class} {}",
                        cell.and_then(|cell| cell.style.as_deref())
                            .unwrap_or_default()
                    );
                    let mut semantic = Node::new(Role::Cell);
                    semantic.set_selected(selected);
                    if !row.selectable {
                        semantic.set_disabled();
                    }
                    let mut element = self
                        .cell(
                            cell.map_or_else(String::new, |cell| cell.content.clone()),
                            &class,
                            widths[column],
                            Target::Cell(index, column),
                            cell.and_then(|cell| cell.alignment.as_ref())
                                .unwrap_or(&config.alignment),
                            semantic,
                        )
                        .with_key(format!("cell:{}", config.key));
                    if props.selectable && row.selectable {
                        let options = element
                            .metadata
                            .accessibility_options
                            .get_or_insert_default();
                        options.focus = state.focused
                            && state.selected_row == Some(index)
                            && state.selected_column.unwrap_or(0) == column;
                        options.focus_event = Some(crate::event::CustomEvent::new(
                            "reactive_tui.table.focus",
                            format!("{index}:{column}").into_bytes(),
                        ));
                    }
                    element
                })
                .collect();
            let mut node = Node::new(Role::Row);
            node.set_selected(selected);
            if !row.selectable {
                node.set_disabled();
            }
            children.push(
                self.row(
                    cells,
                    visible as isize - buffered as isize,
                    state.scroll_state.offset_x,
                    total,
                )
                .with_key(format!("row:{}", row.id))
                .disabled(!row.selectable)
                .with_accessibility(node),
            );
        }
        let insets = self.viewport.map_or([0.0; 4], |layout| layout.insets);
        let (content_width, content_height) = self
            .viewport
            .map_or((0.0, 0.0), |layout| layout.content_size());
        let header = usize::from(props.show_header);
        let rows = children.split_off(header);
        children.push(
            ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
                .styles(
                    StyleBuilder::new()
                        .position_absolute()
                        .inset_left(0.0)
                        .inset_top(header as f32)
                        .width_px(content_width)
                        .height_px((content_height - header as f32).max(0.0))
                        .overflow_hidden(),
                )
                .children(rows)
                .build()
                .with_key("rows"),
        );
        let content = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(
                StyleBuilder::new()
                    .position_absolute()
                    .inset_left(insets[0])
                    .inset_top(insets[1])
                    .width_px(content_width)
                    .height_px(content_height)
                    .overflow_hidden(),
            )
            .children(children)
            .build()
            .with_key("viewport");
        // Absolute rows need an intrinsic width contribution when the table
        // is inside an auto-sized parent.
        let intrinsic = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(StyleBuilder::new().width_px(total as f32).height_px(0.0))
            .build()
            .with_key("intrinsic-width");
        let mut children = vec![content, intrinsic];
        if let Some(viewport) = self.viewport {
            let (width, height) = viewport.size;
            children.extend(super::border::elements(
                &props.border,
                width.max(0.0) as usize,
                height.max(0.0) as usize,
            ));
        }
        let border = usize::from(super::border::enabled(&props.border)) * 2;
        let natural_height = props
            .rows
            .len()
            .saturating_add(usize::from(props.show_header))
            .saturating_add(border)
            .min(u16::MAX as usize);
        let height = props
            .max_height
            .map_or(natural_height, |max| natural_height.min(max as usize));
        ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(
                StyleBuilder::new()
                    .display_flex()
                    .direction(Direction::Column)
                    .height_px(height as f32)
                    .max_height_percent(100.0)
                    .padding_all_px(if super::border::enabled(&props.border) {
                        1.0
                    } else {
                        0.0
                    })
                    .overflow_hidden(),
            )
            .class(if super::border::enabled(&props.border) {
                "w-full min-w-0 min-h-0 border"
            } else {
                "w-full min-w-0 min-h-0"
            })
            .children(children)
            .build()
            .with_focus(FocusProps::input())
            .with_accessibility(Node::new(Role::Table))
    }

    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        if self.error.is_some() {
            return EventResult::Ignored;
        }
        let previous_cursor = state.selected_row;
        let config = &props.config;
        let result = match event {
            Event::Custom(event) if event.name == "reactive_tui.table.focus" => {
                let target = std::str::from_utf8(&event.data).ok().and_then(|value| {
                    let (row, column) = value.split_once(':')?;
                    Some((row.parse::<usize>().ok()?, column.parse::<usize>().ok()?))
                });
                if let Some((row, column)) = target.filter(|&(row, column)| {
                    config.selectable
                        && column < config.columns.len()
                        && config.rows.get(row).is_some_and(|row| row.selectable)
                }) {
                    state.selected_row = Some(row);
                    state.selected_column = Some(column);
                    state.focused = true;
                    self.reveal_selection(config, state);
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            }
            Event::Focus(focus) => {
                state.focused = focus.kind == FocusEventKind::Gained;
                if !state.focused {
                    self.drag = None;
                    state.resizing_column = None;
                }
                EventResult::Consumed
            }
            Event::Key(key) if state.focused && key.kind != KeyEventKind::Release => {
                match key.code {
                    KeyCode::Left if config.scrollable => {
                        state.scroll_state.offset_x = state.scroll_state.offset_x.saturating_sub(1);
                        EventResult::Consumed
                    }
                    KeyCode::Right if config.scrollable => {
                        state.scroll_state.offset_x = state.scroll_state.offset_x.saturating_add(1);
                        EventResult::Consumed
                    }
                    _ => {
                        Table.handle_key_navigation(key.code.clone(), key.modifiers, config, state)
                    }
                }
            }
            Event::Mouse(mouse) if mouse.kind == MouseEventKind::Wheel && config.scrollable => {
                let (mut x, mut y) = match mouse.wheel.as_ref().map(|wheel| &wheel.delta) {
                    Some(WheelDelta::Lines { x, y } | WheelDelta::Pixels { x, y }) => (*x, *y),
                    None => return EventResult::Ignored,
                };
                if mouse.modifiers.shift && x == 0.0 {
                    x = y;
                    y = 0.0;
                }
                if !x.is_finite() || !y.is_finite() {
                    return EventResult::Ignored;
                }
                state.scroll_state.offset_x =
                    (state.scroll_state.offset_x as f32 + x).clamp(0.0, u16::MAX as f32) as u16;
                self.scroll_y =
                    (self.scroll_y as f64 + y as f64).clamp(0.0, usize::MAX as f64) as usize;
                self.public_scroll_y = state.scroll_state.offset_y;
                EventResult::Consumed
            }
            Event::Mouse(mouse) if mouse.kind == MouseEventKind::Up => {
                if self.drag.take().is_none() {
                    return EventResult::Ignored;
                }
                state.resizing_column = None;
                EventResult::Consumed
            }
            Event::Mouse(mouse)
                if matches!(mouse.kind, MouseEventKind::Move | MouseEventKind::Drag)
                    && self.drag.is_some() =>
            {
                let (column, start, width) = self.drag.unwrap();
                let config = &config.columns[column];
                let width = (width as i64 + mouse.position.x() as i64 - start as i64).clamp(
                    config.min_width as i64,
                    config.max_width.unwrap_or(u16::MAX).max(config.min_width) as i64,
                ) as u16;
                self.resized.insert(config.key.clone(), width);
                EventResult::Consumed
            }
            Event::Mouse(mouse)
                if matches!(mouse.kind, MouseEventKind::Down | MouseEventKind::Click)
                    && mouse.button == MouseButton::Left =>
            {
                let Some((target, layout)) = self.target_at(mouse) else {
                    return EventResult::Ignored;
                };
                match target {
                    Target::Header(column) => {
                        let root = self.viewport.unwrap();
                        let [a, b, c, d, tx, ty] = root.transform;
                        let (x, y) = (mouse.position.x() as f32, mouse.position.y() as f32);
                        let local = layout.local_cell(a * x + c * y + tx, b * x + d * y + ty);
                        if mouse.kind == MouseEventKind::Down
                            && config.resizable_columns
                            && config.columns[column].resizable
                            && local.is_some_and(|(x, _)| {
                                x.saturating_add(1) >= state.column_widths[column]
                            })
                        {
                            self.drag =
                                Some((column, mouse.position.x(), state.column_widths[column]));
                            state.resizing_column = Some(column);
                            state.resize_start_x = mouse.position.x().min(u16::MAX as u32) as u16;
                            EventResult::Consumed
                        } else if let Some(request) = &props.sort_request {
                            if !config.sortable || !config.columns[column].sortable {
                                return EventResult::Ignored;
                            }
                            request(column, mouse.modifiers.shift);
                            EventResult::Consumed
                        } else {
                            self.sort(config, state, column)
                        }
                    }
                    Target::Cell(row, column) => {
                        if !config.rows[row].selectable {
                            return EventResult::Ignored;
                        }
                        state.selected_column = Some(column);
                        if mouse.modifiers.ctrl && config.multi_select {
                            Table.toggle_row_selection(config, state, row);
                        } else {
                            Table.select_row(config, state, row, mouse.modifiers.shift);
                        }
                        if let Some(cell) = config.rows[row]
                            .cells
                            .get(&config.columns[column].key)
                            .filter(|cell| cell.clickable)
                        {
                            if let (Some(action), Some(callback)) =
                                (&cell.action, &config.on_row_action)
                            {
                                callback(row, action);
                            }
                        }
                        EventResult::Consumed
                    }
                }
            }
            _ => EventResult::Ignored,
        };
        if matches!(event,Event::Key(key) if matches!(key.code,KeyCode::Up | KeyCode::Down | KeyCode::Home | KeyCode::End | KeyCode::PageUp | KeyCode::PageDown))
        {
            self.reveal_selection(config, state);
        }
        if previous_cursor != state.selected_row {
            if let Some(callback) = &props.cursor_change {
                callback(state.selected_row);
            }
        }
        self.clamp(config, state);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn large_table_navigation_retains_full_row_offset() {
        let config = TableProps {
            columns: vec![TableColumn::new("Name", "name")],
            rows: (0..70_000).map(|i| TableRow::new(&i.to_string())).collect(),
            ..Default::default()
        };
        let mut props = LiveProps {
            config,
            seed: TableState::default(),
            sort_request: None,
            cursor_change: None,
            controlled_selection: false,
            window: None,
        };
        let mut live = LiveTable::new(props.clone());
        let mut state = TableState {
            focused: true,
            ..Default::default()
        };
        live.layout(
            LayoutInfo::from_bounds(crate::event::hit::Bounds {
                x: 0.0,
                y: 0.0,
                width: 24.0,
                height: 7.0,
            }),
            &mut props,
            &mut state,
        );
        live.handle_event(
            &Event::Key(crate::event::types::KeyEvent::new(KeyCode::End)),
            &mut props,
            &mut state,
        );
        assert_eq!(state.selected_row, Some(69_999));
        assert_eq!(state.visible_rows.last(), Some(&69_999));
        live.handle_event(
            &Event::Key(crate::event::types::KeyEvent::new(KeyCode::Up)),
            &mut props,
            &mut state,
        );
        assert_eq!(state.selected_row, Some(69_998));
        assert!(state.visible_rows.contains(&69_998));
    }
}
