//! Specialized widget builders for advanced components
//!
//! This module contains builders for specialized widgets like Tree and Image
//! that have complete implementations in the widgets module.

use crate::component::Element;
use crate::widgets::display::tree::{TreeNode, TreeProps};
use crate::widgets::layout::stack::{StackAlignment, StackDirection, StackJustify, StackPadding};
use crate::widgets::{ImageDisplayMode, ImageFormat, ImageQuality, ImageSource};
use std::path::PathBuf;

/// Create a Tree view builder
///
/// Returns a `TreeBuilder` for creating hierarchical tree view components.
pub fn tree() -> TreeBuilder {
    TreeBuilder::new()
}

/// Create an Image display builder
///
/// Returns an `ImageBuilder` for creating image display components.
pub fn image() -> ImageBuilder {
    ImageBuilder::new()
}

/// Builder for Tree View components
///
/// Provides a fluent API for creating tree view widgets with hierarchical data display.
pub struct TreeBuilder {
    props: TreeProps,
    class: String,
}

impl Default for TreeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeBuilder {
    /// Create a new TreeBuilder with default values
    pub fn new() -> Self {
        Self {
            props: TreeProps::default(),
            class: String::new(),
        }
    }

    /// Set the root node of the tree
    pub fn root(mut self, root: TreeNode) -> Self {
        self.props.root = Some(root);
        self
    }

    /// Set whether nodes can be selected
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.props.selectable = selectable;
        self
    }

    /// Set whether multiple nodes can be selected
    pub fn multi_select(mut self, multi_select: bool) -> Self {
        self.props.multi_select = multi_select;
        self
    }

    /// Set whether to show icons for nodes
    pub fn show_icons(mut self, show_icons: bool) -> Self {
        self.props.show_icons = show_icons;
        self
    }

    /// Set whether to show tree lines
    pub fn show_lines(mut self, show_lines: bool) -> Self {
        self.props.show_lines = show_lines;
        self
    }

    /// Set whether nodes can be checked
    pub fn checkable(mut self, checkable: bool) -> Self {
        self.props.checkable = checkable;
        self
    }

    /// Set CSS classes for styling
    pub fn class(mut self, class: &str) -> Self {
        self.class.push(' ');
        self.class.push_str(class);
        self
    }

    /// Build the Tree element.
    pub fn build(self) -> Element {
        crate::widgets::display::Tree::with_props(self.props).with_class(self.class)
    }
}

impl From<TreeBuilder> for Element {
    fn from(builder: TreeBuilder) -> Self {
        builder.build()
    }
}

/// Builder for Image display components
///
/// Provides a fluent API for creating image display widgets with various formats and rendering modes.
pub struct ImageBuilder {
    source: Option<ImageSource>,
    display_mode: ImageDisplayMode,
    quality: ImageQuality,
    format: Option<ImageFormat>,
    class: String,
}

impl Default for ImageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageBuilder {
    /// Create a new ImageBuilder with default values
    pub fn new() -> Self {
        Self {
            source: None,
            display_mode: ImageDisplayMode::Auto,
            quality: ImageQuality::Balanced,
            format: None,
            class: String::new(),
        }
    }

    /// Set the image source from a file path
    pub fn source_file<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.source = Some(ImageSource::FilePath(path.into()));
        self
    }

    /// Set the image source from raw bytes with format
    pub fn source_raw_bytes(
        mut self,
        data: Vec<u8>,
        width: u32,
        height: u32,
        format: ImageFormat,
    ) -> Self {
        self.source = Some(ImageSource::RawBytes {
            data,
            width,
            height,
            format,
        });
        self
    }

    /// Set the image source from base64 data
    pub fn source_base64(mut self, data: String) -> Self {
        self.source = Some(ImageSource::Base64Data(data));
        self
    }

    /// Set the image source from a URL
    pub fn source_url(mut self, url: String) -> Self {
        self.source = Some(ImageSource::Url(url));
        self
    }

    /// Set the display mode for the image
    pub fn display_mode(mut self, mode: ImageDisplayMode) -> Self {
        self.display_mode = mode;
        self
    }

    /// Set the image quality
    pub fn quality(mut self, quality: ImageQuality) -> Self {
        self.quality = quality;
        self
    }

    /// Set the image format
    pub fn format(mut self, format: ImageFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Set CSS classes for styling
    pub fn class(mut self, class: &str) -> Self {
        self.class.push(' ');
        self.class.push_str(class);
        self
    }

    /// Build a retained image control with its source, format hint and classes.
    pub fn build(self) -> Element {
        let image = crate::widgets::Image {
            source: self
                .source
                .unwrap_or_else(|| ImageSource::FilePath(PathBuf::new())),
            display_mode: self.display_mode,
            quality: self.quality,
            ..Default::default()
        };
        image
            .into_element_with_hint(self.format)
            .with_class(self.class)
    }
}

impl From<ImageBuilder> for Element {
    fn from(builder: ImageBuilder) -> Self {
        builder.build()
    }
}

/// Builder for Radio Button input components
///
/// Provides a fluent API for creating radio button widgets for single-choice selections.
pub struct RadioButtonBuilder {
    value: String,
    label: Option<String>,
    checked: bool,
    disabled: bool,
    group: Option<String>,
    class: Option<String>,
}

impl Default for RadioButtonBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RadioButtonBuilder {
    /// Create a new RadioButtonBuilder with default values
    pub fn new() -> Self {
        Self {
            value: String::new(),
            label: None,
            checked: false,
            disabled: false,
            group: None,
            class: None,
        }
    }

    /// Set the value of the radio button
    pub fn value(mut self, value: &str) -> Self {
        self.value = value.to_string();
        self
    }

    /// Set the label text for the radio button
    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    /// Set whether the radio button is initially checked
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Set whether the radio button is disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the radio button group name
    pub fn group(mut self, group: &str) -> Self {
        self.group = Some(group.to_string());
        self
    }

    /// Set CSS classes for styling
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_owned());
        self
    }

    /// Build the RadioButton element
    pub fn build(self) -> Element {
        use crate::widgets::input::named_radio::{NamedRadio, NamedRadioProps};
        let mut element = Element::typed::<NamedRadio>(NamedRadioProps {
            value: self.value,
            label: self.label,
            checked: self.checked,
            disabled: self.disabled,
            group: self.group,
        });
        if let Some(class) = self.class {
            element = element.with_class(class);
        }
        element
    }
}

impl From<RadioButtonBuilder> for Element {
    fn from(builder: RadioButtonBuilder) -> Self {
        builder.build()
    }
}

/// Builder for Slider input components
///
/// Provides a fluent API for creating slider widgets for numeric value selection.
#[derive(Clone, PartialEq)]
pub struct SliderBuilder {
    value: f64,
    min: f64,
    max: f64,
    step: f64,
    label: Option<String>,
    disabled: bool,
    class: Option<String>,
}

impl Default for SliderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SliderBuilder {
    /// Create a new SliderBuilder with default values
    pub fn new() -> Self {
        Self {
            value: 0.0,
            min: 0.0,
            max: 100.0,
            step: 1.0,
            label: None,
            disabled: false,
            class: None,
        }
    }

    /// Set the current value of the slider
    pub fn value(mut self, value: f64) -> Self {
        self.value = value;
        self
    }

    /// Set the minimum value
    pub fn min(mut self, min: f64) -> Self {
        self.min = min;
        self
    }

    /// Set the maximum value
    pub fn max(mut self, max: f64) -> Self {
        self.max = max;
        self
    }

    /// Set the step increment
    pub fn step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    /// Set the label text for the slider
    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    /// Set whether the slider is disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set CSS classes for styling
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_owned());
        self
    }

    /// Build the Slider element
    pub fn build(self) -> Element {
        let class = self.class.clone();
        let mut element = Element::typed::<ConfiguredSlider>(self);
        if let Some(class) = class {
            element = element.with_class(class);
        }
        element
    }

    fn widget_props(&self) -> crate::widgets::input::SliderProps {
        crate::widgets::input::SliderProps {
            min: self.min,
            max: self.max,
            value: self.value,
            step: self.step,
            disabled: self.disabled,
            ..Default::default()
        }
    }
}

impl crate::component::Props for SliderBuilder {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

struct ConfiguredSlider(crate::widgets::Slider);

impl crate::component::Component for ConfiguredSlider {
    type Props = SliderBuilder;
    type State = crate::widgets::input::SliderState;
    fn new(props: Self::Props) -> Self {
        let mut inner = crate::widgets::Slider::new(props.widget_props());
        inner.set_label(props.label);
        Self(inner)
    }
    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        self.0.set_label(props.label.clone());
        self.0.update(&props.widget_props(), state)
    }
    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        self.0.render(&props.widget_props(), state)
    }
    fn layout(
        &mut self,
        bounds: crate::component::LayoutInfo,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> bool {
        self.0.layout(bounds, &mut props.widget_props(), state)
    }
    fn handle_event(
        &mut self,
        event: &crate::event::Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> crate::event::router::EventResult {
        let mut inner = props.widget_props();
        let result = self.0.handle_event(event, &mut inner, state);
        props.value = inner.value;
        result
    }
}

impl From<SliderBuilder> for Element {
    fn from(builder: SliderBuilder) -> Self {
        builder.build()
    }
}

/// Builder for Scroll View layout components
///
/// Provides a fluent API for creating scrollable container widgets.
pub struct ScrollViewBuilder {
    content: Vec<Element>,
    class: String,
    horizontal_scroll: bool,
    vertical_scroll: bool,
    show_scrollbars: bool,
}

impl Default for ScrollViewBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ScrollViewBuilder {
    /// Create a new ScrollViewBuilder with default values
    pub fn new() -> Self {
        Self {
            content: Vec::new(),
            class: String::new(),
            horizontal_scroll: false,
            vertical_scroll: true,
            show_scrollbars: true,
        }
    }

    /// Add content to the scroll view
    pub fn content(mut self, element: Element) -> Self {
        self.content.push(element);
        self
    }

    /// Add multiple content elements
    pub fn contents(mut self, elements: Vec<Element>) -> Self {
        self.content.extend(elements);
        self
    }

    /// Set whether horizontal scrolling is enabled
    pub fn horizontal_scroll(mut self, enabled: bool) -> Self {
        self.horizontal_scroll = enabled;
        self
    }

    /// Set whether vertical scrolling is enabled
    pub fn vertical_scroll(mut self, enabled: bool) -> Self {
        self.vertical_scroll = enabled;
        self
    }

    /// Set whether scrollbars are visible
    pub fn show_scrollbars(mut self, show: bool) -> Self {
        self.show_scrollbars = show;
        self
    }

    /// Set CSS classes for styling
    pub fn class(mut self, class: &str) -> Self {
        self.class.push(' ');
        self.class.push_str(class);
        self
    }

    /// Build the ScrollView element
    pub fn build(self) -> Element {
        crate::widgets::layout::ScrollViewBuilder::new(
            crate::widgets::layout::StackBuilder::vertical()
                .children(self.content)
                .render(),
        )
        .scroll_x(self.horizontal_scroll)
        .scroll_y(self.vertical_scroll)
        .show_scrollbars(self.show_scrollbars)
        .render()
        .with_class(self.class)
    }
}

impl From<ScrollViewBuilder> for Element {
    fn from(builder: ScrollViewBuilder) -> Self {
        builder.build()
    }
}

/// Builder for Stack layout components
///
/// Provides a fluent API for creating stack layout containers that arrange children
/// in a single direction (horizontal or vertical).
pub struct StackBuilder {
    children: Vec<Element>,
    class: String,
    direction: StackDirection,
    alignment: StackAlignment,
    justify: StackJustify,
    spacing: f32,
    padding: StackPadding,
}

impl Default for StackBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl StackBuilder {
    /// Create a new StackBuilder with default values
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            class: String::new(),
            direction: StackDirection::Vertical,
            alignment: StackAlignment::Start,
            justify: StackJustify::Start,
            spacing: 0.0,
            padding: StackPadding::default(),
        }
    }

    /// Add a child element to the stack
    pub fn child(mut self, element: Element) -> Self {
        self.children.push(element);
        self
    }

    /// Add multiple child elements
    pub fn children(mut self, elements: Vec<Element>) -> Self {
        self.children.extend(elements);
        self
    }

    /// Set the stack direction
    pub fn direction(mut self, direction: StackDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Set the cross-axis alignment
    pub fn alignment(mut self, alignment: StackAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Set the main-axis justification
    pub fn justify(mut self, justify: StackJustify) -> Self {
        self.justify = justify;
        self
    }

    /// Set the spacing between children
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Set the padding around the stack
    pub fn padding(mut self, padding: StackPadding) -> Self {
        self.padding = padding;
        self
    }

    /// Set CSS classes for styling
    pub fn class(mut self, class: &str) -> Self {
        self.class.push(' ');
        self.class.push_str(class);
        self
    }

    /// Build the Stack element
    pub fn build(self) -> Element {
        let props = crate::widgets::layout::StackProps {
            children: self.children,
            direction: self.direction,
            alignment: self.alignment,
            justify: self.justify,
            padding: self.padding,
            ..Default::default()
        };
        let mut element = crate::widgets::layout::stack::stack_element(&props, self.spacing);
        element
            .class
            .get_or_insert_with(String::new)
            .push_str(&self.class);
        element
    }
}

impl From<StackBuilder> for Element {
    fn from(builder: StackBuilder) -> Self {
        builder.build()
    }
}

/// Builder for Dialog components
///
/// Provides a fluent API for creating modal dialog widgets.
pub struct DialogBuilder {
    title: Option<String>,
    content: Vec<Element>,
    modal: bool,
    closable: bool,
    width: Option<u16>,
    height: Option<u16>,
    class: Option<String>,
}

impl Default for DialogBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl DialogBuilder {
    /// Create a new DialogBuilder with default values
    pub fn new() -> Self {
        Self {
            title: None,
            content: Vec::new(),
            modal: true,
            closable: true,
            width: None,
            height: None,
            class: None,
        }
    }

    /// Set the dialog title
    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    /// Add content to the dialog
    pub fn content(mut self, element: Element) -> Self {
        self.content.push(element);
        self
    }

    /// Add multiple content elements
    pub fn contents(mut self, elements: Vec<Element>) -> Self {
        self.content.extend(elements);
        self
    }

    /// Set whether the dialog is modal
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }

    /// Set whether the dialog can be closed
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// Set the dialog width
    pub fn width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the dialog height
    pub fn height(mut self, height: u16) -> Self {
        self.height = Some(height);
        self
    }

    /// Set CSS classes for styling
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Build the Dialog element
    pub fn build(self) -> Element {
        use crate::widgets::display::modal::{Modal, ModalProps, ModalSize};
        Modal::with_props(ModalProps {
            title: self.title,
            content: Some(
                Element::layout(crate::component::LayoutType::Flex)
                    .class("flex-col")
                    .children(self.content),
            ),
            visible: true,
            closable: self.closable,
            backdrop_clickable: self.closable,
            keyboard_navigation: self.closable,
            focus_trap: self.modal,
            backdrop_style: self.modal.then(|| "bg-black/50".to_string()),
            width: self.width.map_or(ModalSize::Auto, ModalSize::Fixed),
            height: self.height.map_or(ModalSize::Auto, ModalSize::Fixed),
            modal_style: self.class.or_else(|| ModalProps::default().modal_style),
            ..Default::default()
        })
    }
}

impl From<DialogBuilder> for Element {
    fn from(builder: DialogBuilder) -> Self {
        builder.build()
    }
}

/// Builder for Confirmation Dialog components
///
/// Provides a fluent API for creating confirmation dialog widgets.
pub struct ConfirmationDialogBuilder {
    title: Option<String>,
    message: String,
    confirm_text: String,
    cancel_text: String,
    danger: bool,
    class: Option<String>,
}

impl Default for ConfirmationDialogBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfirmationDialogBuilder {
    /// Create a new ConfirmationDialogBuilder with default values
    pub fn new() -> Self {
        Self {
            title: None,
            message: String::new(),
            confirm_text: "OK".to_string(),
            cancel_text: "Cancel".to_string(),
            danger: false,
            class: None,
        }
    }

    /// Set the dialog title
    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    /// Set the confirmation message
    pub fn message(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self
    }

    /// Set the confirm button text
    pub fn confirm_text(mut self, text: &str) -> Self {
        self.confirm_text = text.to_string();
        self
    }

    /// Set the cancel button text
    pub fn cancel_text(mut self, text: &str) -> Self {
        self.cancel_text = text.to_string();
        self
    }

    /// Set whether this is a dangerous action
    pub fn danger(mut self, danger: bool) -> Self {
        self.danger = danger;
        self
    }

    /// Set CSS classes for styling
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Build the ConfirmationDialog element
    pub fn build(self) -> Element {
        use crate::widgets::dialog::{
            ButtonVariant, ConfirmationButton, ConfirmationButtons, ConfirmationDialog,
            ConfirmationDialogOptions, DialogComponent, DialogId, DialogTheme,
        };
        let mut confirm = ConfirmationButton::ok();
        confirm.text = self.confirm_text;
        if self.danger {
            confirm.variant = ButtonVariant::Danger;
        }
        let mut cancel = ConfirmationButton::cancel();
        cancel.text = self.cancel_text;
        let mut css_classes = std::collections::HashMap::new();
        if let Some(class) = self.class {
            css_classes.insert("dialog".to_string(), class);
        }
        ConfirmationDialog::new(
            DialogId::from_u32(0),
            ConfirmationDialogOptions {
                title: self.title.unwrap_or_else(|| "Confirm".to_string()),
                message: self.message,
                buttons: ConfirmationButtons::Custom(vec![confirm, cancel]),
                css_classes,
                ..Default::default()
            },
        )
        .render(
            crate::core::geometry::Rect::default(),
            &DialogTheme::default(),
        )
    }
}

impl From<ConfirmationDialogBuilder> for Element {
    fn from(builder: ConfirmationDialogBuilder) -> Self {
        builder.build()
    }
}

// Re-export additional dialog builders
pub use super::dialog_builders::{ProgressDialogBuilder, WizardBuilder, WizardStep};
