//! The builder for the empty display piece (docs/spec/display-pieces.md), carrying DIS-001, DIS-002, DIS-005.

use std::sync::Arc;

use crate::component::Element;
use crate::widgets::display::pieces::empty::{Empty, EmptyAction, EmptyProps};
use crate::widgets::display::pieces::icon::Icon;

/// Create an empty state: an icon, a title, a description and actions.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::empty;
/// use reactive_tui::widgets::display::pieces::icon::Icon;
///
/// let state = empty()
///     .icon(Icon::Folder)
///     .title("No projects yet")
///     .description("Create a project to start.")
///     .action("New project", || {})
///     .build();
/// ```
pub fn empty() -> EmptyBuilder {
    EmptyBuilder {
        props: EmptyProps {
            icon: Some(Icon::Info),
            ..EmptyProps::default()
        },
    }
}

/// Builder for an empty state. Its icon defaults to the catalog's `Info` mark.
#[derive(Clone)]
pub struct EmptyBuilder {
    props: EmptyProps,
}

impl EmptyBuilder {
    /// Set the icon above the title, from the icon catalog.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.props.icon = Some(icon);
        self
    }

    /// Set the title, painted in `foreground`.
    pub fn title(mut self, title: &str) -> Self {
        self.props.title = title.to_string();
        self
    }

    /// Set the description, painted in `text-muted`.
    pub fn description(mut self, description: &str) -> Self {
        self.props.description = description.to_string();
        self
    }

    /// Add a button. It runs `on_action` on Confirm, Activate or a click.
    /// Actions appear in the order they were added.
    pub fn action<F>(mut self, label: &str, on_action: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.props.actions.push(EmptyAction {
            label: label.to_string(),
            on_action: Arc::new(on_action),
        });
        self
    }

    /// Add classes to the state's element, after its own.
    pub fn class(mut self, class: &str) -> Self {
        if !self.props.class.is_empty() {
            self.props.class.push(' ');
        }
        self.props.class.push_str(class);
        self
    }

    /// Build the state's element.
    pub fn build(self) -> Element {
        Element::typed::<Empty>(self.props)
    }
}
