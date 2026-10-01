use super::*;
use crate::{
    accessibility::{Node, Role},
    builder::ElementBuilder,
    component::{ElementType, FocusProps, LayoutInfo},
    event::types::{
        FocusEventKind, KeyEventKind, MouseButton, MouseEvent, MouseEventKind, WheelDelta,
    },
    layout::style::{Direction, StyleBuilder},
    widgets::display::look,
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
                let floor = Table::column_floor(props, column);
                self.resized
                    .get(&column.key)
                    .copied()
                    .unwrap_or(width)
                    .max(floor)
                    .min(column.max_width.unwrap_or(u16::MAX).max(floor))
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
        if let Some(position) = state.cursor_row.or(state.selected_row).and_then(|row| {
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

    /// A header cell: the column's title and, when the column is sorted,
    /// its mark in `primary` (DAT-001).
    #[allow(clippy::too_many_arguments)]
    fn header(
        &self,
        title: &str,
        mark: Option<&str>,
        width: u16,
        column: usize,
        alignment: &Alignment,
        style: Option<&str>,
        node: Node,
    ) -> Element {
        // The title and the mark sit where a cell's text of the same width
        // would, so a centered or right-aligned header lines up with its
        // column's cells.
        let width = width as usize;
        let mark_width = mark.map_or(0, UnicodeWidthStr::width);
        let title_width = UnicodeWidthStr::width(title).min(width.saturating_sub(mark_width));
        let start = match alignment {
            Alignment::Center => width.saturating_sub(title_width + mark_width) / 2,
            Alignment::End => width.saturating_sub(title_width + mark_width),
            _ => 0,
        };
        let piece = |text: &str, x: usize, piece_width: usize, class: &str| {
            ElementBuilder::new(ElementType::Text(text.to_string()))
                .styles(
                    StyleBuilder::new()
                        .position_absolute()
                        .inset_left(x as f32)
                        .inset_top(0.0)
                        .width_px(piece_width as f32)
                        .height_px(1.0)
                        .overflow_hidden(),
                )
                .class(&format!("whitespace-pre {class}"))
                .build()
        };
        let mut pieces = vec![piece(
            title,
            start,
            title_width,
            style.unwrap_or(look::HEADER),
        )];
        if let Some(mark) = mark {
            pieces.push(piece(mark, start + title_width, mark_width, look::MARK));
        }
        let mut cell = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(
                StyleBuilder::new()
                    .width_px(width as f32)
                    .height_px(1.0)
                    .flex_shrink(0.0)
                    .overflow_hidden(),
            )
            .children(pieces)
            .build()
            .with_accessibility(node);
        let targets = self.targets.clone();
        cell.metadata.layout.push(Arc::new(move |layout| {
            targets
                .lock()
                .unwrap()
                .push((Target::Header(column), layout));
            false
        }));
        cell
    }

    fn row(
        &self,
        cells: Vec<Element>,
        y: isize,
        offset: u16,
        width: usize,
        class: &str,
    ) -> Element {
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
            .class(class)
            .children(cells)
            .build()
    }

    /// Scrolls sideways so that `column` is whole in view (DAT-004).
    fn reveal_column(&self, state: &mut TableState, column: usize) {
        let widths = &state.column_widths;
        let start = widths
            .iter()
            .take(column)
            .fold(0u16, |sum, &width| sum.saturating_add(width));
        let end = start.saturating_add(widths.get(column).copied().unwrap_or(0));
        let scroll = &mut state.scroll_state;
        if start < scroll.offset_x {
            scroll.offset_x = start;
        } else if end > scroll.offset_x.saturating_add(scroll.viewport_width) {
            scroll.offset_x = end.saturating_sub(scroll.viewport_width);
        }
    }

    /// Sorts by `column` as a click on its header does: through the data
    /// table's request when it owns the sort, else here.
    fn sort_by(
        &self,
        props: &LiveProps,
        state: &mut TableState,
        column: usize,
        extend: bool,
    ) -> EventResult {
        let config = &props.config;
        if !config.sortable || !config.columns.get(column).is_some_and(|col| col.sortable) {
            return EventResult::Ignored;
        }
        if let Some(request) = &props.sort_request {
            request(column, extend);
            EventResult::Consumed
        } else {
            self.sort(config, state, column)
        }
    }

    /// The first row the keys can reach, in the sorted order.
    fn first_row(props: &TableProps, state: &TableState) -> Option<usize> {
        Table
            .sort_rows(props, state)
            .into_iter()
            .find(|&index| props.selectable && props.rows[index].selectable)
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
        state.cursor_row = state.cursor_row.and_then(remap);
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
            state.cursor_row = None;
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
            return Element::text(error).with_class(format!(
                "{} whitespace-normal break-words w-full",
                look::ERROR
            ));
        }
        let props = &props.config;
        self.targets.lock().unwrap().clear();
        let widths = self.widths(props);
        let total: usize = widths.iter().map(|&width| width as usize).sum();
        // A row's fill reaches the box's last cell when the columns are
        // narrower than the box.
        let row_width = total.max(state.scroll_state.viewport_width as usize);
        let mut children = Vec::new();
        if props.show_header {
            let cells = props
                .columns
                .iter()
                .enumerate()
                .map(|(i, column)| {
                    let sorted = state.sort_column == Some(i);
                    let mark = sorted.then_some(if state.sort_ascending { " ↑" } else { " ↓" });
                    let mut node = Node::new(Role::ColumnHeader);
                    node.set_label(column.title.clone());
                    node.inner.set_column_index(i);
                    if sorted {
                        node.inner.set_sort_direction(if state.sort_ascending {
                            accesskit::SortDirection::Ascending
                        } else {
                            accesskit::SortDirection::Descending
                        });
                    }
                    self.header(
                        &column.title,
                        mark,
                        widths[i],
                        i,
                        &column.alignment,
                        props.header_style.as_deref(),
                        node,
                    )
                    .with_key(format!("header:{}", column.key))
                })
                .collect();
            children.push(
                self.row(cells, 0, state.scroll_state.offset_x, row_width, "")
                    .with_key("header"),
            );
        }
        // The cursor row shows while the table holds the focus (DAT-001).
        let cursor = if state.focused {
            state.cursor_row.or(state.selected_row)
        } else {
            None
        };
        let buffered = self
            .window
            .map_or(0, |(_, overscan, _)| self.scroll_y.min(overscan));
        for (visible, &index) in state.visible_rows.iter().enumerate() {
            let row = &props.rows[index];
            let selected = state.selected_rows.contains(&index);
            // The application's style for a selected row takes the place of
            // `accent`; the cursor and the hover keep their roles.
            let state_class = look::row(
                cursor == Some(index),
                selected && props.selected_style.is_none(),
                state.hover_row == Some(index),
                !row.selectable,
            );
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
                    semantic.inner.set_row_index(index);
                    semantic.inner.set_column_index(column);
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
                        options.focus =
                            cursor == Some(index) && state.selected_column.unwrap_or(0) == column;
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
            node.inner.set_row_index(index);
            if !row.selectable {
                node.set_disabled();
            }
            children.push(
                self.row(
                    cells,
                    visible as isize - buffered as isize,
                    state.scroll_state.offset_x,
                    row_width,
                    state_class,
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
        let bordered = super::border::enabled(&props.border);
        let natural_height = props
            .rows
            .len()
            .saturating_add(usize::from(props.show_header))
            .saturating_add(usize::from(bordered) * 2)
            .min(u16::MAX as usize);
        // The rows are absolute, so this child gives the table its natural
        // size inside an auto-sized parent: the columns' width and a row
        // for each row of data.
        let intrinsic = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(
                StyleBuilder::new()
                    .width_px(total as f32)
                    .height_px(natural_height as f32),
            )
            .build()
            .with_key("intrinsic-size");
        let mut children = vec![content, intrinsic];
        if let Some(viewport) = self.viewport {
            let (width, height) = viewport.size;
            children.extend(super::border::elements(
                &props.border,
                width.max(0.0) as usize,
                height.max(0.0) as usize,
            ));
        }
        // The box is its parent's background with no fill of its own
        // (DAT-001). It fills the width and the height its parent allots,
        // and takes its natural size in a parent that allots none, unless
        // the props set a size (DAT-002).
        let mut style = StyleBuilder::new()
            .display_flex()
            .direction(Direction::Column)
            .padding_all_px(if bordered { 1.0 } else { 0.0 })
            .overflow_hidden();
        style = match (props.height, props.max_height) {
            (Some(height), _) => style.height_px(height as f32),
            (None, Some(max)) => style
                .height_px(natural_height.min(max as usize) as f32)
                .max_height_percent(100.0),
            (None, None) => style
                .height_percent(100.0)
                .min_height_px(0.0)
                .max_height_percent(100.0),
        };
        style = match props.width {
            Some(width) => style.width_px(width as f32),
            None => style.width_percent(100.0),
        };
        let mut node = Node::new(Role::Table);
        if let Some(label) = &props.aria_label {
            node.set_label(label.clone());
        }
        node.inner.set_row_count(props.rows.len());
        node.inner.set_column_count(props.columns.len());
        ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(style)
            .class("min-w-0")
            .children(children)
            .build()
            .with_focus(FocusProps::input())
            .with_accessibility(node)
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
                    state.cursor_row = Some(row);
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
                } else if state.cursor_row.is_none() {
                    // The cursor starts on the selected row, else on the first
                    // row, without selecting it.
                    state.cursor_row = state
                        .selected_row
                        .or_else(|| Self::first_row(config, state));
                }
                EventResult::Consumed
            }
            Event::Key(key) if state.focused && key.kind != KeyEventKind::Release => {
                match key.code {
                    // Left and Right move the column cursor and bring its
                    // column into view; `s` sorts by it (DAT-004).
                    KeyCode::Left | KeyCode::Right if !config.columns.is_empty() => {
                        let last = config.columns.len() - 1;
                        let current = state.selected_column.unwrap_or(0).min(last);
                        let column = if key.code == KeyCode::Left {
                            current.saturating_sub(1)
                        } else {
                            (current + 1).min(last)
                        };
                        state.selected_column = Some(column);
                        self.reveal_column(state, column);
                        EventResult::Consumed
                    }
                    KeyCode::Char('s') if !key.modifiers.ctrl && !key.modifiers.alt => {
                        let column = state.selected_column.unwrap_or(0);
                        self.sort_by(props, state, column, key.modifiers.shift)
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
                // The limits `clamp` applies at the next layout.
                let scroll = &state.scroll_state;
                let last_x = scroll.content_width.saturating_sub(scroll.viewport_width);
                let last_y = config
                    .rows
                    .len()
                    .saturating_sub(scroll.viewport_height as usize);
                let offset_x = (scroll.offset_x as f32 + x).clamp(0.0, f32::from(last_x)) as u16;
                let scroll_y = (self.scroll_y as f64 + y as f64).clamp(0.0, last_y as f64) as usize;
                // At an edge the wheel passes to an enclosing view (INP-005).
                if (offset_x, scroll_y) == (scroll.offset_x, self.scroll_y) {
                    return EventResult::Ignored;
                }
                state.scroll_state.offset_x = offset_x;
                self.scroll_y = scroll_y;
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
                if matches!(mouse.kind, MouseEventKind::Move | MouseEventKind::Enter)
                    && self.drag.is_none() =>
            {
                state.hover_row = match self.target_at(mouse) {
                    Some((Target::Cell(row, _), _)) => Some(row),
                    _ => None,
                };
                EventResult::Ignored
            }
            Event::Mouse(mouse) if mouse.kind == MouseEventKind::Leave => {
                state.hover_row = None;
                EventResult::Ignored
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
                if mouse.kind == MouseEventKind::Down && mouse.button == MouseButton::Left =>
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
                        } else {
                            self.sort_by(props, state, column, mouse.modifiers.shift)
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

    /// DAT-004: the table's node tells its row and column counts and is
    /// named by `aria_label` alone.
    #[test]
    fn dat_004_a_table_tells_its_row_and_column_counts() {
        let config = TableProps {
            columns: vec![
                TableColumn::new("Widget", "widget"),
                TableColumn::new("State", "state"),
            ],
            rows: vec![
                TableRow::new("input").with_cell("widget", "Input"),
                TableRow::new("layout").with_cell("widget", "Layout"),
                TableRow::new("data").with_cell("widget", "Data"),
            ],
            ..Default::default()
        };
        let props = LiveProps {
            config,
            seed: TableState::default(),
            sort_request: None,
            cursor_change: None,
            controlled_selection: false,
            window: None,
        };
        let live = LiveTable::new(props.clone());
        let element = live.render(&props, &TableState::default());
        let node = &element
            .metadata
            .accessibility
            .as_ref()
            .expect("the table's node")
            .inner;
        assert_eq!(node.row_count(), Some(3));
        assert_eq!(node.column_count(), Some(2));
        assert_eq!(
            node.label(),
            None,
            "the table has no name when the props set none"
        );
    }

    /// DAT-004: the sorted header tells its direction, each row its index
    /// and each cell its column index; the table is named by `aria_label`.
    #[test]
    fn dat_004_a_sorted_header_tells_its_direction_and_each_cell_its_place() {
        let config = TableProps {
            columns: vec![
                TableColumn::new("Name", "name"),
                TableColumn::new("Count", "count"),
            ],
            rows: vec![
                TableRow::new("a")
                    .with_cell("name", "alpha")
                    .with_cell("count", "9"),
                TableRow::new("b")
                    .with_cell("name", "beta")
                    .with_cell("count", "10"),
            ],
            sortable: true,
            sort_column: Some(1),
            sort_ascending: false,
            aria_label: Some("Project members".into()),
            ..Default::default()
        };
        let mut props = LiveProps {
            config,
            seed: TableState {
                sort_column: Some(1),
                sort_ascending: false,
                ..Default::default()
            },
            sort_request: None,
            cursor_change: None,
            controlled_selection: false,
            window: None,
        };
        let mut live = LiveTable::new(props.clone());
        let mut state = live.initial_state(&props);
        live.layout(
            LayoutInfo::from_bounds(crate::event::hit::Bounds {
                x: 0.0,
                y: 0.0,
                width: 40.0,
                height: 7.0,
            }),
            &mut props,
            &mut state,
        );
        let element = live.render(&props, &state);
        let node = |element: &Element| {
            element
                .metadata
                .accessibility
                .as_ref()
                .expect("a node")
                .inner
                .clone()
        };
        assert_eq!(node(&element).label(), Some("Project members"));
        let content = &element.children[0];
        let header = &content.children[0];
        assert_eq!(
            node(&header.children[1]).sort_direction(),
            Some(accesskit::SortDirection::Descending),
            "the sorted column's header tells its direction"
        );
        assert_eq!(node(&header.children[0]).sort_direction(), None);
        assert_eq!(node(&header.children[1]).column_index(), Some(1));
        // Sorted by the numbers, descending: beta (10) comes first.
        let rows = &content.children[1];
        assert_eq!(node(&rows.children[0]).row_index(), Some(1));
        assert_eq!(node(&rows.children[1]).row_index(), Some(0));
        assert_eq!(node(&rows.children[0].children[1]).column_index(), Some(1));
        assert_eq!(node(&rows.children[0].children[1]).row_index(), Some(1));
    }

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
