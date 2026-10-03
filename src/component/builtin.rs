//! Factories for the historical named widget construction route.

use super::{AnyComponentInstance, Component, ComponentInstance};
use std::any::Any;

pub(super) fn create(name: &str, props: &dyn Any) -> Option<AnyComponentInstance> {
    use crate::widgets::{Checkbox, Select, Slider, TextInput};
    match name {
        "TextInput" => typed::<TextInput>(props),
        "Checkbox" => typed::<Checkbox>(props),
        "Slider" => typed::<Slider>(props),
        "Tabs" => typed::<crate::widgets::layout::Tabs>(props),
        "Accordion" => typed::<crate::widgets::layout::accordion::Accordion>(props),
        "Breadcrumb" => typed::<crate::widgets::layout::breadcrumb::Breadcrumb>(props),
        "Stack" => typed::<crate::widgets::layout::Stack>(props),
        "ScrollView" => typed::<crate::widgets::layout::ScrollView>(props),
        "Select" => typed::<Select<String>>(props),
        "Tree" => typed::<crate::widgets::display::Tree>(props),
        "Table" => typed::<crate::widgets::display::Table>(props),
        "DataTable" => typed::<crate::widgets::display::DataTable>(props),
        "ProgressBar" => typed::<crate::widgets::display::ProgressBar>(props),
        "Popover" => typed::<crate::widgets::display::Popover>(props),
        "Modal" => typed::<crate::widgets::display::Modal>(props),
        "Chart" | "Charts" => typed::<crate::widgets::display::Chart>(props),
        "Terminal" | "TerminalWidget" => typed::<crate::widgets::TerminalWidget>(props),
        "FileExplorer" => typed::<crate::widgets::display::FileExplorer>(props),
        "MenuBar" => typed::<crate::widgets::menu::MenuBar>(props),
        "ContextMenu" => typed::<crate::widgets::menu::ContextMenu>(props),
        "PopupMenu" => typed::<crate::widgets::menu::PopupMenu>(props),
        "DialogMenu" => typed::<crate::widgets::menu::DialogMenu>(props),
        _ => None,
    }
}

fn typed<C: Component>(props: &dyn Any) -> Option<AnyComponentInstance> {
    props
        .downcast_ref::<C::Props>()
        .map(|props| AnyComponentInstance::new(ComponentInstance::<C>::new(props.clone())))
}
