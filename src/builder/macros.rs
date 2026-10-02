//! Declarative macros for the builder API
//!
//! This module contains all the declarative macros that provide a convenient,
//! HTML-like syntax for creating UI elements with the builder API.

/// Universal element creation - accepts anything that can become an Element
///
/// Usage:
/// ```
/// use reactive_tui::el;
/// use reactive_tui::builder::{div, span};
/// use reactive_tui::vdom::VNode;
///
/// let elements = el![
///     div().text("Web API").build(),
///     VNode::text("VDOM text"),
///     "Just text",
///     span().text("More web API").build(),
/// ];
/// ```
#[macro_export]
macro_rules! el {
    // Single element
    ($child:expr) => {
        $crate::builder::to_element($child)
    };

    // Container with class and children
    ($tag:ident, class: $class:expr, [$($child:expr),* $(,)?]) => {
        $crate::builder::$tag()
            .class($class)
            .children(vec![$($crate::builder::to_element($child)),*])
            .build()
    };

    // Container with just children
    ($tag:ident, [$($child:expr),* $(,)?]) => {
        $crate::builder::$tag()
            .children(vec![$($crate::builder::to_element($child)),*])
            .build()
    };

    // Multiple elements as children
    [$($child:expr),* $(,)?] => {
        vec![$($crate::builder::to_element($child)),*]
    };
}

/// Improved composition macros
///
/// Create a div with optional class and children
///
/// Usage:
/// ```
/// use reactive_tui::{div, span, input};
///
/// fn example() {
///     let simple = div!["Hello"];                           // Simple text child
///     let with_class = div![class: "flex", "Hello", "World"];   // With class and multiple children
///     let nested = div![span!["Name:"], input![]];          // Nested elements
/// }
/// ```
#[macro_export]
macro_rules! div {
    // Just children
    [$($child:expr),* $(,)?] => {
        $crate::builder::div()
            .children(vec![$($crate::el!($child)),*])
            .build()
    };

    // Class and children
    [class: $class:expr, $($child:expr),* $(,)?] => {
        $crate::builder::div()
            .class($class)
            .children(vec![$($crate::el!($child)),*])
            .build()
    };

    // Just class
    [class: $class:expr] => {
        $crate::builder::div().class($class).build()
    };

    // Empty
    [] => {
        $crate::builder::div().build()
    };
}

/// Create a span with optional class and children
#[macro_export]
macro_rules! span {
    [$($child:expr),* $(,)?] => {
        $crate::builder::span()
            .children(vec![$($crate::el!($child)),*])
            .build()
    };

    [class: $class:expr, $($child:expr),* $(,)?] => {
        $crate::builder::span()
            .class($class)
            .children(vec![$($crate::el!($child)),*])
            .build()
    };

    [class: $class:expr] => {
        $crate::builder::span().class($class).build()
    };

    [] => {
        $crate::builder::span().build()
    };
}

/// Create a button with optional properties
#[macro_export]
macro_rules! button {
    [$($child:expr),* $(,)?] => {
        $crate::builder::button()
            .children(vec![$($crate::el!($child)),*])
            .build()
    };

    [class: $class:expr, $($child:expr),* $(,)?] => {
        $crate::builder::button()
            .class($class)
            .children(vec![$($crate::el!($child)),*])
            .build()
    };

    [onclick: $handler:expr, $($child:expr),* $(,)?] => {
        $crate::builder::button()
            .on_click($handler)
            .children(vec![$($crate::el!($child)),*])
            .build()
    };

    [class: $class:expr, onclick: $handler:expr, $($child:expr),* $(,)?] => {
        $crate::builder::button()
            .class($class)
            .on_click($handler)
            .children(vec![$($crate::el!($child)),*])
            .build()
    };
}

/// Create an input with optional properties
#[macro_export]
macro_rules! input {
    [] => {
        $crate::builder::input().build()
    };

    [placeholder: $placeholder:expr] => {
        $crate::builder::input().placeholder($placeholder).build()
    };

    [class: $class:expr] => {
        $crate::builder::input().class($class).build()
    };

    [class: $class:expr, placeholder: $placeholder:expr] => {
        $crate::builder::input()
            .class($class)
            .placeholder($placeholder)
            .build()
    };
}

/// Create a data table with optional configuration
#[macro_export]
macro_rules! data_table {
    // Simple table with columns and rows
    [columns: [$($col_title:expr => $col_key:expr),* $(,)?], rows: [$($row:expr),* $(,)?]] => {
        {
            let mut builder = $crate::builder::data_table();
            $(
                builder = builder.column($col_title, $col_key);
            )*
            $(
                builder = builder.simple_row($row);
            )*
            builder.build()
        }
    };

    // Table with pagination
    [columns: [$($col_title:expr => $col_key:expr),* $(,)?], rows: [$($row:expr),* $(,)?], pagination: $page_size:expr] => {
        {
            let mut builder = $crate::builder::data_table();
            $(
                builder = builder.column($col_title, $col_key);
            )*
            $(
                builder = builder.simple_row($row);
            )*
            builder.pagination(true, $page_size).build()
        }
    };
}

/// Create a chart with optional configuration
#[macro_export]
macro_rules! chart {
    // Simple bar chart
    [bar: $name:expr => [$($value:expr),* $(,)?]] => {
        $crate::builder::chart()
            .bar_chart()
            .simple_series($name, vec![$($value),*])
            .build()
    };

    // Simple line chart
    [line: $name:expr => [$($value:expr),* $(,)?]] => {
        $crate::builder::chart()
            .line_chart()
            .simple_series($name, vec![$($value),*])
            .build()
    };

    // Simple pie chart
    [pie: $name:expr => [$($value:expr),* $(,)?]] => {
        $crate::builder::chart()
            .pie_chart()
            .simple_series($name, vec![$($value),*])
            .build()
    };

    // Chart with title
    [bar: $name:expr => [$($value:expr),* $(,)?], title: $title:expr] => {
        $crate::builder::chart()
            .bar_chart()
            .simple_series($name, vec![$($value),*])
            .title($title)
            .build()
    };

    [line: $name:expr => [$($value:expr),* $(,)?], title: $title:expr] => {
        $crate::builder::chart()
            .line_chart()
            .simple_series($name, vec![$($value),*])
            .title($title)
            .build()
    };

    [pie: $name:expr => [$($value:expr),* $(,)?], title: $title:expr] => {
        $crate::builder::chart()
            .pie_chart()
            .simple_series($name, vec![$($value),*])
            .title($title)
            .build()
    };
}

/// Create a text input with optional configuration
#[macro_export]
macro_rules! text_input {
    // Simple text input
    [] => {
        $crate::builder::text_input().build()
    };

    // Text input with placeholder
    [placeholder: $placeholder:expr] => {
        $crate::builder::text_input().placeholder($placeholder).build()
    };

    // Text input with value
    [value: $value:expr] => {
        $crate::builder::text_input().value($value).build()
    };

    // Text input with value and placeholder
    [value: $value:expr, placeholder: $placeholder:expr] => {
        $crate::builder::text_input()
            .value($value)
            .placeholder($placeholder)
            .build()
    };
}

/// Create a checkbox with optional configuration
#[macro_export]
macro_rules! checkbox {
    // Simple checkbox
    [] => {
        $crate::builder::checkbox().build()
    };

    // Checkbox with label
    [label: $label:expr] => {
        $crate::builder::checkbox().label($label).build()
    };

    // Checked checkbox with label
    [checked: $checked:expr, label: $label:expr] => {
        $crate::builder::checkbox()
            .checked($checked)
            .label($label)
            .build()
    };
}

/// Create a select dropdown with options
#[macro_export]
macro_rules! select {
    // Select with options
    [options: [$($value:expr => $label:expr),* $(,)?]] => {
        {
            let mut builder = $crate::builder::select();
            $(
                builder = builder.option($value, $label);
            )*
            builder.build()
        }
    };

    // Select with options and selected value
    [options: [$($value:expr => $label:expr),* $(,)?], selected: $selected:expr] => {
        {
            let mut builder = $crate::builder::select();
            $(
                builder = builder.option($value, $label);
            )*
            builder.selected($selected).build()
        }
    };
}

/// Create a progress bar with value
#[macro_export]
macro_rules! progress_bar {
    // Simple progress bar
    [$value:expr] => {
        $crate::builder::progress_bar().value($value).build()
    };

    // Progress bar with label
    [$value:expr, label: $label:expr] => {
        $crate::builder::progress_bar()
            .value($value)
            .label($label)
            .build()
    };

    // Progress bar with custom max value
    [$value:expr, max: $max:expr] => {
        $crate::builder::progress_bar()
            .value($value)
            .max_value($max)
            .build()
    };
}

/// Create a toast notification
#[macro_export]
macro_rules! toast {
    // Success toast
    [success: $message:expr] => {
        $crate::builder::toast().success($message).build()
    };

    // Error toast
    [error: $message:expr] => {
        $crate::builder::toast().error($message).build()
    };

    // Warning toast
    [warning: $message:expr] => {
        $crate::builder::toast().warning($message).build()
    };

    // Info toast
    [info: $message:expr] => {
        $crate::builder::toast().info($message).build()
    };
}

/// Create tabs with content
#[macro_export]
macro_rules! tabs {
    // Tabs with title and content pairs
    [$($title:expr => $content:expr),* $(,)?] => {
        {
            let mut builder = $crate::builder::tabs();
            $(
                builder = builder.tab($title, $content);
            )*
            builder.build()
        }
    };
}
