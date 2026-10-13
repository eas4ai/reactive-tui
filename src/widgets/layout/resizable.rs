//! The resizable split (docs/spec/layout-widgets.md, NAV-007 to NAV-010):
//! panes side by side or stacked, with a divider between each pair. Each
//! divider is a focus stop of its own and a splitter for the screen reader;
//! a drag, its arrow keys, Home and End, or the screen reader's requests
//! move it.

mod share;

use super::look;
use crate::accessibility::{Node, Role};
use crate::component::{Component, Element, FocusProps, LayoutInfo, LayoutType, Props};
use crate::event::router::EventResult;
use crate::event::types::{KeyEvent, MouseButton, MouseEventKind, Position};
use crate::event::{CustomEvent, Event, MouseEvent};
use crate::keymap::{Action, Keymap};
use crate::widgets::display::pieces::separator::{glyph, SeparatorStyle};
use std::any::Any;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// The callback called with every pane's size after a move.
pub(crate) type ResizeCallback = Arc<dyn Fn(Vec<u16>) + Send + Sync>;

/// The default minimum size of a pane, in cells.
const DEFAULT_MIN: u16 = 4;

/// The events a divider takes for the screen reader's requests (NAV-010).
/// Each event's name ends in `:` and the divider's index; a SetValue
/// carries the requested size of the pane before the divider as text.
const SET_VALUE_EVENT: &str = "reactive_tui.resizable.set_value";
const INCREMENT_EVENT: &str = "reactive_tui.resizable.increment";
const DECREMENT_EVENT: &str = "reactive_tui.resizable.decrement";

/// The cells a divider moves on an arrow key with Shift held.
const SHIFT_STEP: i32 = 10;

/// A move longer than any split: Home and End, as far as the divider goes.
const FAR: i32 = u16::MAX as i32;

/// No divider holds the keyboard focus.
const NO_DIVIDER: usize = usize::MAX;

/// How a pane's box is sized along the split.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Along {
    /// Its share in whole cells, once the split knows its length; the box
    /// shows the pane's content.
    Cells(u16),
    /// Before the first layout, a `Cells` pane: this many cells, through
    /// the layout.
    Fixed(u16),
    /// Before the first layout, any other pane: its share of what the
    /// `Cells` panes leave, by this grow factor, through the layout.
    Grow(f32),
}

/// Which way the panes are laid out.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SplitDirection {
    /// Panes side by side, with a vertical divider `│` between them.
    #[default]
    Row,
    /// Panes stacked, with a horizontal divider `─` between them.
    Column,
}

/// How big a pane is asked to be.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PaneSize {
    /// A fixed number of cells.
    Cells(u16),
    /// A share of the cells left after the fixed panes, by weight.
    Weight(f32),
}

/// One pane of a split: its content, its size and the least it can take.
#[derive(Clone, PartialEq)]
pub struct Pane {
    /// The element shown in the pane.
    pub content: Element,
    /// The pane's size; none takes an equal share.
    pub size: Option<PaneSize>,
    /// The fewest cells the pane can be moved down to.
    pub min: u16,
}

impl Pane {
    /// Create a pane of `content` with an equal share and the default
    /// minimum of four cells.
    pub fn new(content: Element) -> Self {
        Self {
            content,
            size: None,
            min: DEFAULT_MIN,
        }
    }

    /// Ask for a fixed number of cells.
    pub fn size(mut self, cells: u16) -> Self {
        self.size = Some(PaneSize::Cells(cells));
        self
    }

    /// Ask for a share of the leftover cells by weight.
    pub fn weight(mut self, weight: f32) -> Self {
        self.size = Some(PaneSize::Weight(weight));
        self
    }

    /// Set the fewest cells the pane can take.
    pub fn min(mut self, cells: u16) -> Self {
        self.min = cells;
        self
    }
}

/// Properties for the resizable split.
#[derive(Clone, Default, PartialEq)]
pub struct ResizableProps {
    /// Which way the panes are laid out.
    pub direction: SplitDirection,
    /// The panes, in order.
    pub panes: Vec<Pane>,
    /// The name the screen reader hears for each divider.
    pub aria_label: Option<String>,
    /// Whether the dividers ignore keys, drags and the screen reader.
    pub disabled: bool,
}

impl Props for ResizableProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ResizableProps {
    fn specs(&self) -> Vec<share::Spec> {
        self.panes
            .iter()
            .map(|pane| (pane.size, pane.min))
            .collect()
    }

    fn mins(&self) -> Vec<u16> {
        self.panes.iter().map(|pane| pane.min).collect()
    }
}

/// A resizable split: panes with a divider between each pair. The split
/// fills what its parent allots; its length less one cell per divider is
/// shared among the panes in whole cells (NAV-008).
pub struct Resizable {
    /// Called with every pane's size after each move.
    on_resize: Option<ResizeCallback>,
    /// The split's box from its last layout.
    viewport: Option<LayoutInfo>,
    /// Each pane's size in cells; empty until the first layout.
    sizes: Vec<u16>,
    /// Each pane's share of the length the panes share, which a change of
    /// the split's length keeps (NAV-008).
    shares: Vec<f64>,
    /// The length the sizes share: the split's less one cell per divider.
    length: u16,
    /// What the props asked of the panes when the sizes were made.
    specs: Vec<share::Spec>,
    /// The divider that holds the keyboard focus, set by its focus
    /// callbacks; `NO_DIVIDER` when none does.
    focused: Arc<AtomicUsize>,
    /// The divider under the pointer.
    hovered: Option<usize>,
    /// The divider a press took and its drag moves.
    dragged: Option<usize>,
    /// The size the last move left the pane before its divider at, told to
    /// the screen reader politely, and the count of moves so far, which
    /// keys its node: each move publishes a new node, which the reader
    /// announces even when the number is the one it heard last (NAV-011).
    announced: Option<(u64, u16)>,
    /// How many moves the split has made.
    moves: u64,
}

impl Resizable {
    /// Create a split from its props, with no callback yet.
    pub fn new(_props: ResizableProps) -> Self {
        Self {
            on_resize: None,
            viewport: None,
            sizes: Vec::new(),
            shares: Vec::new(),
            length: 0,
            specs: Vec::new(),
            focused: Arc::new(AtomicUsize::new(NO_DIVIDER)),
            hovered: None,
            dragged: None,
            announced: None,
            moves: 0,
        }
    }

    /// Set the callback called with every pane's size after each move.
    pub fn with_on_resize(mut self, f: impl Fn(Vec<u16>) + Send + Sync + 'static) -> Self {
        self.on_resize = Some(Arc::new(f));
        self
    }

    /// Replace the resize callback, or remove it with `None`: a builder's
    /// split takes the one its latest render carried (CMP-008, CMP-009).
    pub(crate) fn set_on_resize(&mut self, f: Option<ResizeCallback>) {
        self.on_resize = f;
    }

    /// The split's length along its panes and across them, in cells.
    fn extent(&self, direction: SplitDirection) -> (u16, u16) {
        let (width, height) = self
            .viewport
            .map_or((0.0, 0.0), |layout| layout.content_size());
        let (width, height) = (width.max(0.0) as u16, height.max(0.0) as u16);
        match direction {
            SplitDirection::Row => (width, height),
            SplitDirection::Column => (height, width),
        }
    }

    /// Whether the sizes belong to the props' panes.
    fn laid_out(&self, props: &ResizableProps) -> bool {
        !self.sizes.is_empty() && self.sizes.len() == props.panes.len()
    }

    /// Shares the split's length among the panes again: from the props
    /// when their panes changed or none were laid out, else by each pane's
    /// share (NAV-008). Returns whether a size changed.
    fn share(&mut self, props: &ResizableProps) -> bool {
        if self.viewport.is_none() {
            return false;
        }
        let count = props.panes.len();
        let dividers = u16::try_from(count.saturating_sub(1)).unwrap_or(u16::MAX);
        let length = self.extent(props.direction).0.saturating_sub(dividers);
        let specs = props.specs();
        let before = std::mem::take(&mut self.sizes);
        if before.len() != count || self.length == 0 || !share::same(&specs, &self.specs) {
            self.sizes = share::initial(&specs, length);
            self.specs = specs;
            self.length = length;
            self.remember_shares();
            // The last size told describes panes the props no longer ask for.
            self.announced = None;
        } else {
            self.sizes = if length == self.length {
                before.clone()
            } else {
                share::rescale(&self.shares, &props.mins(), length)
            };
            if length > 0 {
                self.length = length;
            }
        }
        before != self.sizes
    }

    /// Keeps each pane's share of the length, for the next change of it.
    fn remember_shares(&mut self) {
        if self.length == 0 {
            return;
        }
        let length = f64::from(self.length);
        self.shares = self
            .sizes
            .iter()
            .map(|size| f64::from(*size) / length)
            .collect();
    }

    /// The divider that holds the keyboard focus, if it is one of the props'.
    fn focused_divider(&self, props: &ResizableProps) -> Option<usize> {
        let divider = self.focused.load(Ordering::Acquire);
        (divider != NO_DIVIDER && divider + 1 < props.panes.len()).then_some(divider)
    }

    /// The cell along the panes where divider `divider` stands.
    fn position(&self, divider: usize) -> u32 {
        self.sizes[..=divider]
            .iter()
            .map(|size| u32::from(*size))
            .sum::<u32>()
            + divider as u32
    }

    /// The divider that stands in cell `cell` along the panes.
    fn divider_at(&self, cell: u32) -> Option<usize> {
        (0..self.sizes.len().saturating_sub(1)).find(|divider| self.position(*divider) == cell)
    }

    /// The cell along the panes under the pointer, from the split's first
    /// content cell; a pointer before it is at cell 0.
    fn pointer_cell(&self, mouse: &MouseEvent, direction: SplitDirection) -> Option<u32> {
        let Position::Cell { x, y } = mouse.position else {
            return None;
        };
        let insets = self.viewport.map_or([0.0; 4], |layout| layout.insets);
        let cell = match direction {
            SplitDirection::Row => f32::from(x) - insets[0],
            SplitDirection::Column => f32::from(y) - insets[1],
        };
        Some(cell.max(0.0) as u32)
    }

    /// Moves divider `divider` by `delta` cells, right or down when
    /// positive, and reports every pane's size once when a size changed
    /// (NAV-009).
    fn move_divider(&mut self, divider: usize, delta: i32, props: &ResizableProps) -> EventResult {
        if !self.laid_out(props) || divider + 1 >= self.sizes.len() {
            return EventResult::Ignored;
        }
        let sizes = share::moved(&self.sizes, &props.mins(), divider, delta);
        if sizes != self.sizes {
            self.sizes = sizes;
            self.remember_shares();
            // Orca 50.1 speaks a named splitter's role on a change of its
            // value but not the value, so the split also tells the reader
            // the new size of the pane before the divider (NAV-011).
            self.moves += 1;
            self.announced = Some((self.moves, self.sizes[divider]));
            if let Some(callback) = &self.on_resize {
                callback(self.sizes.clone());
            }
        }
        EventResult::Consumed
    }

    /// The arrow keys between the panes move the focused divider a cell,
    /// ten with Shift; Home and End move it as far as it goes (NAV-009).
    fn handle_key(&mut self, key: &KeyEvent, props: &ResizableProps) -> EventResult {
        let Some(divider) = self.focused_divider(props) else {
            return EventResult::Ignored;
        };
        let Some((action, shifted)) = Keymap::active().action_shifted(key) else {
            return EventResult::Ignored;
        };
        let step = if shifted { SHIFT_STEP } else { 1 };
        let (back, forward) = match props.direction {
            SplitDirection::Row => (Action::Left, Action::Right),
            SplitDirection::Column => (Action::Up, Action::Down),
        };
        let delta = match action {
            action if action == back => -step,
            action if action == forward => step,
            Action::Home => -FAR,
            Action::End => FAR,
            _ => return EventResult::Ignored,
        };
        self.move_divider(divider, delta, props)
    }

    /// A press on a divider takes the pointer (INP-003) and each motion of
    /// the drag moves the divider to the cell under the pointer (NAV-009);
    /// the pointer over a divider hovers it.
    fn handle_mouse(&mut self, mouse: &MouseEvent, props: &ResizableProps) -> EventResult {
        match mouse.kind {
            MouseEventKind::Move | MouseEventKind::Enter => {
                let hovered = self
                    .pointer_cell(mouse, props.direction)
                    .and_then(|cell| self.divider_at(cell));
                if hovered == self.hovered {
                    return EventResult::Ignored;
                }
                self.hovered = hovered;
                EventResult::Handled
            }
            MouseEventKind::Leave => {
                if self.hovered.take().is_some() {
                    EventResult::Handled
                } else {
                    EventResult::Ignored
                }
            }
            MouseEventKind::Down if mouse.button == MouseButton::Left => {
                let pressed = self
                    .pointer_cell(mouse, props.direction)
                    .and_then(|cell| self.divider_at(cell));
                match pressed {
                    Some(divider) => {
                        self.dragged = Some(divider);
                        EventResult::Consumed
                    }
                    None => EventResult::Ignored,
                }
            }
            MouseEventKind::Drag if mouse.button == MouseButton::Left => {
                let (Some(divider), Some(cell)) =
                    (self.dragged, self.pointer_cell(mouse, props.direction))
                else {
                    return EventResult::Ignored;
                };
                if divider + 1 >= self.sizes.len() {
                    return EventResult::Ignored;
                }
                let delta = i64::from(cell) - i64::from(self.position(divider));
                let delta = delta.clamp(i64::from(-FAR), i64::from(FAR)) as i32;
                self.move_divider(divider, delta, props)
            }
            MouseEventKind::Up if self.dragged.is_some() => {
                self.dragged = None;
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }

    /// The screen reader's requests on a divider (NAV-010): Increment and
    /// Decrement move it as Right and Left do; SetValue moves it so the
    /// pane before it takes the requested size, within what it can take.
    fn handle_request(&mut self, event: &CustomEvent, props: &ResizableProps) -> EventResult {
        let Some((name, divider)) = event.name.rsplit_once(':') else {
            return EventResult::Ignored;
        };
        let Ok(divider) = divider.parse::<usize>() else {
            return EventResult::Ignored;
        };
        match name {
            INCREMENT_EVENT => self.move_divider(divider, 1, props),
            DECREMENT_EVENT => self.move_divider(divider, -1, props),
            SET_VALUE_EVENT => {
                let Some(requested) = std::str::from_utf8(&event.data)
                    .ok()
                    .and_then(|text| text.trim().parse::<f64>().ok())
                    .filter(|value| value.is_finite())
                else {
                    return EventResult::Ignored;
                };
                let Some(size) = self.sizes.get(divider) else {
                    return EventResult::Ignored;
                };
                let delta = (requested.round() - f64::from(*size))
                    .clamp(f64::from(-FAR), f64::from(FAR)) as i32;
                self.move_divider(divider, delta, props)
            }
            _ => EventResult::Ignored,
        }
    }

    /// Pane `index`'s box, sized along the split by `size`. Only a box of
    /// whole cells holds the pane's content, so the first frame that shows
    /// a pane's content already has the sizes the screen reader reads from
    /// the dividers.
    fn pane(pane: &Pane, index: usize, size: Along, direction: SplitDirection) -> Element {
        let (along, across) = match direction {
            SplitDirection::Row => ("w", "h-full"),
            SplitDirection::Column => ("h", "w-full"),
        };
        let class = match size {
            Along::Cells(cells) => format!("{along}-{cells} {across}"),
            Along::Fixed(_) | Along::Grow(_) => across.to_owned(),
        };
        let mut element = Element::layout(LayoutType::Flex)
            .class(format!("flex flex-col shrink-0 overflow-hidden {class}"))
            .with_key(format!("pane:{index}"));
        match size {
            Along::Cells(_) => element = element.with_child(pane.content.clone()),
            Along::Fixed(cells) => {
                element.metadata.inline_styles = Some(format!("flex-basis: {cells}"));
            }
            Along::Grow(grow) => {
                element.metadata.inline_styles = Some(format!("flex-basis: 0; flex-grow: {grow}"));
            }
        }
        element
    }

    /// Before the first layout, as `share::initial` will share the length:
    /// how each pane's box is sized, a `Cells` pane by its size, at least
    /// its minimum, and the others by their weights, a pane with no size
    /// counting as a weight of one and weights that are all zero sharing
    /// equally; and the sizes the dividers tell the screen reader until the
    /// layout gives the real ones, a `Cells` pane's and every other pane's
    /// minimum.
    fn provisional(props: &ResizableProps) -> (Vec<Along>, Vec<u16>) {
        let weight = |pane: &Pane| match pane.size {
            Some(PaneSize::Cells(_)) => None,
            Some(PaneSize::Weight(weight)) if weight.is_finite() => Some(weight.max(0.0)),
            Some(PaneSize::Weight(_)) => Some(0.0),
            None => Some(1.0),
        };
        let even = props.panes.iter().filter_map(weight).sum::<f32>() <= 0.0;
        props
            .panes
            .iter()
            .map(|pane| match (pane.size, weight(pane)) {
                (Some(PaneSize::Cells(cells)), _) => {
                    let cells = cells.max(pane.min);
                    (Along::Fixed(cells), cells)
                }
                (_, Some(_)) if even => (Along::Grow(1.0), pane.min),
                (_, weight) => (Along::Grow(weight.unwrap_or(0.0)), pane.min),
            })
            .unzip()
    }

    /// A divider's cells without pixels: the separator piece's `│` down a
    /// column of `cross` cells when `row`, else its `─` along a row of them.
    fn line(row: bool, cross: u16) -> Element {
        let line = glyph(SeparatorStyle::Line, row).to_string();
        let lines: Vec<Element> = if row {
            (0..cross)
                .map(|_| Element::text(line.clone()).class("shrink-0 h-1 w-1"))
                .collect()
        } else {
            vec![Element::text(line.repeat(usize::from(cross))).class("shrink-0 h-1 whitespace-pre")]
        };
        // The line is the divider's look; the splitter's node speaks for it.
        let mut decoration = Node::new(Role::Label);
        decoration.set_hidden();
        Element::layout(LayoutType::Flex)
            .class(if row {
                "flex flex-col w-1 h-full"
            } else {
                "flex flex-row h-1 w-full"
            })
            .with_children(lines)
            .with_accessibility(decoration)
    }

    /// Divider `index`: the separator piece's line across the split, in
    /// `border`, on `hover` under the pointer, and in `selection` while it
    /// holds the focus or is dragged (NAV-007); a focus stop and a splitter
    /// whose value is the size of the pane before it among `sizes`
    /// (NAV-010). With pixels its line is a picture in place of the glyphs
    /// (PIX-009). Before the first layout it is a focus stop that draws no
    /// line, `cross` being unknown, so the App can give it the initial
    /// focus on the first frame; the first divider takes the auto focus
    /// the split's element asks for (NAV-011).
    fn divider(
        &self,
        index: usize,
        props: &ResizableProps,
        sizes: &[u16],
        cross: Option<u16>,
    ) -> Element {
        let row = props.direction == SplitDirection::Row;
        let active = !props.disabled
            && (self.focused_divider(props) == Some(index) || self.dragged == Some(index));
        let hovered = !props.disabled && self.hovered == Some(index);
        let fill = if active {
            look::FOCUSED.to_owned()
        } else if hovered {
            format!("{} {}", look::DIVIDER, look::HOVER)
        } else {
            look::DIVIDER.to_owned()
        };

        let mut node = Node::new(Role::Splitter);
        if let Some(label) = &props.aria_label {
            node.set_label(label.clone());
        }
        // A divider between panes side by side runs down the split.
        node.inner.set_orientation(if row {
            accesskit::Orientation::Vertical
        } else {
            accesskit::Orientation::Horizontal
        });
        let (least, greatest) = share::bounds(sizes, &props.mins(), index);
        node.inner.set_numeric_value(f64::from(sizes[index]));
        node.inner.set_min_numeric_value(f64::from(least));
        node.inner.set_max_numeric_value(f64::from(greatest));
        node.inner.set_numeric_value_step(1.0);
        if props.disabled {
            node.set_disabled();
        }
        let gained = self.focused.clone();
        let lost = self.focused.clone();
        let focus = FocusProps {
            on_focus: Some(Arc::new(move || gained.store(index, Ordering::Release))),
            on_blur: Some(Arc::new(move || {
                let _ =
                    lost.compare_exchange(index, NO_DIVIDER, Ordering::AcqRel, Ordering::Acquire);
            })),
            ..FocusProps::input()
        };
        let mut element = Element::layout(LayoutType::Flex)
            .class(format!(
                "flex shrink-0 {} {fill}",
                if row { "w-1 h-full" } else { "h-1 w-full" }
            ))
            .with_key(format!("divider:{index}"))
            .with_accessibility(node)
            .with_focus(focus)
            .disabled(props.disabled);
        element.metadata.takes_auto_focus = index == 0;
        // With pixels the divider's cells hold no glyph: a picture over them
        // draws its line with a grip, in `ring` while it holds the focus or
        // is dragged, in place of `│` or `─` (PIX-009). Without pixels they
        // hold the separator piece's line.
        if let Some(cross) = cross {
            if crate::widgets::input::look::pixels() {
                #[cfg(feature = "wgpu-graphics")]
                {
                    element.metadata.look = Some(Arc::new(crate::graphics::look::Look::divider(
                        usize::from(cross),
                        row,
                        active,
                    )));
                }
            } else {
                element = element.with_child(Self::line(row, cross));
            }
        }
        if !props.disabled {
            let options = element
                .metadata
                .accessibility_options
                .get_or_insert_default();
            let event = |name: &str| Some(CustomEvent::new(format!("{name}:{index}"), Vec::new()));
            options.set_value_event = event(SET_VALUE_EVENT);
            options.increment_event = event(INCREMENT_EVENT);
            options.decrement_event = event(DECREMENT_EVENT);
        }
        element
    }
}

impl Component for Resizable {
    type Props = ResizableProps;
    type State = ();

    fn new(props: Self::Props) -> Self {
        Self::new(props)
    }

    fn update(&mut self, props: &Self::Props, _state: &mut Self::State) -> bool {
        if props.disabled {
            self.dragged = None;
            self.hovered = None;
        }
        self.share(props);
        true
    }

    fn layout(
        &mut self,
        bounds: LayoutInfo,
        props: &mut Self::Props,
        _state: &mut Self::State,
    ) -> bool {
        let before = self.extent(props.direction);
        self.viewport = Some(bounds);
        let moved = self.share(props);
        moved || before != self.extent(props.direction)
    }

    fn render(&self, props: &Self::Props, _state: &Self::State) -> Element {
        let count = props.panes.len();
        let mut children = Vec::with_capacity(count * 2 + 1);
        // Until the split knows its length its panes share it through the
        // layout and show no content, and its dividers are focus stops that
        // draw nothing: the App can give the first divider the initial
        // focus on the first frame (NAV-011), and the first frame that shows
        // a pane's content already has the sizes the screen reader reads
        // from the dividers.
        let laid_out = self.laid_out(props);
        let (boxes, sizes, cross) = if laid_out {
            (
                self.sizes.iter().map(|size| Along::Cells(*size)).collect(),
                self.sizes.clone(),
                Some(self.extent(props.direction).1),
            )
        } else {
            let (boxes, sizes) = Self::provisional(props);
            (boxes, sizes, None)
        };
        for (index, (pane, size)) in props.panes.iter().zip(boxes).enumerate() {
            children.push(Self::pane(pane, index, size, props.direction));
            if index + 1 < count {
                children.push(self.divider(index, props, &sizes, cross));
            }
        }
        // The size the last move left: screen-reader-only text that takes
        // no cell, in a polite live region beside the splitters, so a reader
        // that speaks a splitter's role but not its value still hears the
        // number (NAV-011). Each move's node is new, so the reader hears it
        // even when the number repeats.
        if let Some((serial, size)) = self.announced {
            children.push(
                Element::text(size.to_string())
                    .with_key(format!("announced:{serial}"))
                    .class("sr-only aria-live-polite"),
            );
        }
        let direction = match props.direction {
            SplitDirection::Row => "flex-row",
            SplitDirection::Column => "flex-col",
        };
        // The split fills what its parent allots unless a class sizes it
        // (NAV-008).
        Element::layout(LayoutType::Flex)
            .class(format!(
                "flex {direction} w-full h-full min-w-0 min-h-0 overflow-hidden"
            ))
            .with_children(children)
    }

    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        _state: &mut Self::State,
    ) -> EventResult {
        if props.disabled {
            return EventResult::Ignored;
        }
        match event {
            Event::Key(key) => self.handle_key(key, props),
            Event::Mouse(mouse) => self.handle_mouse(mouse, props),
            Event::Custom(request) => self.handle_request(request, props),
            _ => EventResult::Ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::types::{KeyCode, KeyModifiers};
    use std::sync::Mutex;

    fn props(sizes: &[Option<u16>]) -> ResizableProps {
        ResizableProps {
            panes: sizes
                .iter()
                .map(|size| {
                    let pane = Pane::new(Element::text("pane"));
                    match size {
                        Some(cells) => pane.size(*cells),
                        None => pane,
                    }
                })
                .collect(),
            ..ResizableProps::default()
        }
    }

    fn laid_out(props: &mut ResizableProps, width: f32) -> (Resizable, Arc<Mutex<Vec<Vec<u16>>>>) {
        let log: Arc<Mutex<Vec<Vec<u16>>>> = Arc::default();
        let sink = log.clone();
        let mut split = Resizable::new(props.clone())
            .with_on_resize(move |sizes| sink.lock().unwrap().push(sizes));
        split.layout(
            LayoutInfo::from_bounds(crate::event::hit::Bounds::new(0.0, 0.0, width, 20.0)),
            props,
            &mut (),
        );
        (split, log)
    }

    fn key(code: KeyCode, shift: bool) -> Event {
        let mut event = KeyEvent::new(code);
        if shift {
            event = event.with_modifiers(KeyModifiers::shift());
        }
        Event::Key(event)
    }

    /// NAV-008 and NAV-009: the focused divider moves a cell on Right, ten
    /// with Shift, to its end on End, and reports each move once.
    #[test]
    #[serial_test::serial(keymap)]
    fn nav_009_the_focused_divider_moves_on_its_keys_and_reports_once() {
        let mut props = props(&[Some(49), None]);
        let (mut split, log) = laid_out(&mut props, 100.0);
        assert_eq!(split.sizes, vec![49, 50]);
        assert_eq!(
            split.handle_event(&key(KeyCode::Right, false), &mut props, &mut ()),
            EventResult::Ignored,
            "no divider holds the focus"
        );
        split.focused.store(0, Ordering::Release);
        for (code, shift) in [
            (KeyCode::Right, false),
            (KeyCode::Right, true),
            (KeyCode::End, false),
            (KeyCode::End, false),
        ] {
            split.handle_event(&key(code, shift), &mut props, &mut ());
        }
        assert_eq!(
            *log.lock().unwrap(),
            vec![vec![50, 49], vec![60, 39], vec![95, 4]],
            "a key that moves nothing reports nothing"
        );
    }

    /// NAV-008: the panes keep their shares when the split widens.
    #[test]
    fn nav_008_a_widened_split_keeps_each_panes_share() {
        let mut props = props(&[Some(30), None]);
        let (mut split, _) = laid_out(&mut props, 100.0);
        assert_eq!(split.sizes, vec![30, 69]);
        split.layout(
            LayoutInfo::from_bounds(crate::event::hit::Bounds::new(0.0, 0.0, 200.0, 20.0)),
            &mut props,
            &mut (),
        );
        assert_eq!(split.sizes.iter().sum::<u16>(), 199);
        assert!(split.sizes[0] == 60 || split.sizes[0] == 61);
    }

    /// NAV-010: SetValue moves the divider so the pane before it takes the
    /// requested size; a request for a divider the split lacks does
    /// nothing.
    #[test]
    fn nav_010_set_value_sizes_the_pane_before_the_divider() {
        let mut props = props(&[Some(49), None]);
        let (mut split, log) = laid_out(&mut props, 100.0);
        let request = |name: &str, divider: usize, data: &str| {
            Event::Custom(CustomEvent::new(
                format!("{name}:{divider}"),
                data.as_bytes().to_vec(),
            ))
        };
        split.handle_event(&request(SET_VALUE_EVENT, 0, "30"), &mut props, &mut ());
        split.handle_event(&request(SET_VALUE_EVENT, 1, "30"), &mut props, &mut ());
        split.handle_event(&request(SET_VALUE_EVENT, 0, "500"), &mut props, &mut ());
        assert_eq!(*log.lock().unwrap(), vec![vec![30, 69], vec![95, 4]]);
    }
}
