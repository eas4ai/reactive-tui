# Menu System for Reactive-TUI

A comprehensive menu system providing menubar, context menus, popup menus, and dialog menus for terminal applications.

## Features

- **MenuBar**: Traditional desktop-style menubar with dropdown submenus
- **ContextMenu**: Right-click context menus with customizable trigger areas
- **PopupMenu**: General-purpose popup menus with flexible placement
- **DialogMenu**: Modal dialog-style menus for complex interactions
- **Rich Menu Items**: Support for actions, checkboxes, radio buttons, separators, and submenus
- **Keyboard Navigation**: Full keyboard support with arrow keys, Enter, Escape, etc.
- **Mouse Support**: Click, hover, and scroll interactions
- **Theming**: Built-in themes (Default, Dark, Light, HighContrast) and custom styling
- **Shortcuts**: Keyboard shortcut display and handling
- **Icons**: Optional icon support for menu items
- **Accessibility**: Focus management and screen reader friendly

## Components

### MenuBar

A horizontal menubar typically placed at the top of the application.

```rust
use reactive_tui::widgets::{MenuBar, MenuBarBuilder, MenuItem, MenuTheme};

let menubar = MenuBarBuilder::new()
    .items(vec![
        MenuItem::submenu("file", "File", vec![
            MenuItem::action("new", "New", || println!("New file")),
            MenuItem::action("open", "Open", || println!("Open file")),
            MenuItem::separator(),
            MenuItem::action("exit", "Exit", || std::process::exit(0)),
        ]),
        MenuItem::submenu("edit", "Edit", vec![
            MenuItem::action("undo", "Undo", || println!("Undo")),
            MenuItem::action("redo", "Redo", || println!("Redo")),
        ]),
    ])
    .theme(MenuTheme::Dark)
    .title("My App")
    .build();
```

### ContextMenu

Right-click context menus that appear at the cursor position.

```rust
use reactive_tui::widgets::{ContextMenu, MenuItem};

let context_menu = ContextMenu::default()
    .with_on_item_selected(|item_id| {
        println!("Selected: {}", item_id);
    });

let props = ContextMenuProps {
    items: vec![
        MenuItem::action("cut", "Cut", || println!("Cut")),
        MenuItem::action("copy", "Copy", || println!("Copy")),
        MenuItem::action("paste", "Paste", || println!("Paste")),
    ],
    show_on_right_click: true,
    ..Default::default()
};
```

### PopupMenu

General-purpose popup menus with flexible placement options.

```rust
use reactive_tui::widgets::{PopupMenu, PopupPlacement, MenuItem};

let popup_props = PopupMenuProps {
    items: vec![
        MenuItem::action("option1", "Option 1", || {}),
        MenuItem::action("option2", "Option 2", || {}),
        MenuItem::action("option3", "Option 3", || {}),
    ],
    placement: PopupPlacement::Below { x: 10, y: 5, width: 20 },
    visible: true,
    auto_close: true,
    ..Default::default()
};
```

### DialogMenu

Modal dialog-style menus for complex interactions.

```rust
use reactive_tui::widgets::{DialogMenu, DialogMenuType, MenuItem};

// Confirmation dialog
let dialog_props = DialogMenuBuilder::confirmation()
    .title("Confirm Action")
    .message("Are you sure you want to delete this file?")
    .items(vec![
        MenuItem::action("yes", "Yes", || println!("Confirmed")),
        MenuItem::action("no", "No", || println!("Cancelled")),
    ])
    .default_button(0)
    .cancel_button(1)
    .build();

// Input dialog
let input_dialog = DialogMenuBuilder::input()
    .title("Enter Name")
    .message("Please enter your name:")
    .build();

// Multi-selection dialog
let multi_dialog = DialogMenuBuilder::multi_selection()
    .title("Select Options")
    .items(vec![
        MenuItem::checkbox("opt1", "Option 1", false, |_| {}),
        MenuItem::checkbox("opt2", "Option 2", true, |_| {}),
        MenuItem::checkbox("opt3", "Option 3", false, |_| {}),
    ])
    .build();
```

## Menu Items

### Basic Items

```rust
// Action item
let action = MenuItem::action("save", "Save File", || {
    println!("File saved!");
});

// Action with shortcut and icon
let action_with_extras = MenuItem::action("save", "Save File", || {})
    .shortcut(MenuShortcut::new("Ctrl+S", vec!["ctrl", "s"]))
    .icon("💾")
    .description("Save the current file");

// Separator
let separator = MenuItem::separator();
```

### Interactive Items

```rust
// Checkbox
let checkbox = MenuItem::checkbox("show_toolbar", "Show Toolbar", true, |checked| {
    println!("Toolbar visibility: {}", checked);
});

// Radio button
let radio = MenuItem::radio("theme_dark", "Dark Theme", false, "theme_group", || {
    println!("Dark theme selected");
});

// Submenu
let submenu = MenuItem::submenu("recent", "Recent Files", vec![
    MenuItem::action("file1", "document.txt", || {}),
    MenuItem::action("file2", "readme.md", || {}),
]);
```

### Item States

```rust
let item = MenuItem::action("test", "Test", || {})
    .enabled(false)        // Disable the item
    .visible(true)         // Control visibility
    .separator_type(MenuSeparator::Line); // Add separator after item
```

## Styling and Theming

### Built-in Themes

```rust
use reactive_tui::widgets::{MenuTheme, MenuStyle};

// Use built-in themes
let dark_style = MenuTheme::Dark.to_style();
let light_style = MenuTheme::Light.to_style();
let high_contrast_style = MenuTheme::HighContrast.to_style();
```

### Custom Styling

```rust
use reactive_tui::widgets::MenuStyle;

// Create custom styling using CSS utility classes
let custom_style = MenuStyle::new()
    .base_classes("bg-gray-900 text-white p-2")
    .selected_classes("bg-blue-600 text-white font-bold")
    .focused_classes("bg-cyan-500 text-black font-bold")
    .disabled_classes("text-gray-500")
    .show_icons(true)
    .show_shortcuts(true)
    .min_width(15)
    .padding(1);

// Or use theme variables for consistent styling
let themed_style = MenuStyle::new()
    .base_classes("bg-primary text-on-primary p-2")
    .selected_classes("bg-accent text-on-accent font-bold")
    .focused_classes("bg-secondary text-on-secondary font-bold");
```

## Event Handling

### Callbacks

```rust
let menubar = MenuBar::default()
    .with_on_item_selected(|item_id| {
        match item_id {
            "new" => create_new_file(),
            "open" => open_file_dialog(),
            "save" => save_current_file(),
            _ => {}
        }
    })
    .with_on_dropdown_opened(|menu_index| {
        println!("Opened menu {}", menu_index);
    })
    .with_on_dropdown_closed(|| {
        println!("Closed dropdown");
    });
```

### Keyboard Navigation

- **Arrow Keys**: Navigate between items
- **Enter**: Activate selected item
- **Escape**: Close menu/dialog
- **Tab**: Navigate between elements (in dialogs)
- **Space**: Toggle checkboxes
- **Home/End**: Jump to first/last item
- **Page Up/Down**: Scroll through long menus

### Mouse Interaction

- **Left Click**: Select and activate items
- **Right Click**: Show context menu (if enabled)
- **Hover**: Highlight items
- **Scroll**: Navigate through long menus
- **Outside Click**: Close popup menus (if enabled)

## Integration with Component System

The menu system follows the reactive-tui component architecture:

```rust
use reactive_tui::component::{Component, Element};

// Create a component that uses a menubar
struct MyApp {
    menubar: MenuBar,
}

impl Component for MyApp {
    type Props = MyAppProps;
    type State = MyAppState;

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        Element::container()
            .child(
                Element::component::<MenuBar>()
                    .with_props(props.menubar_props.clone())
            )
            .child(
                // ... other UI elements
            )
    }

    fn handle_event(
        &mut self,
        event: &Event,
        props: &mut Self::Props,
        state: &mut Self::State,
    ) -> EventResult {
        // Forward events to menubar
        self.menubar.handle_event(event, &mut props.menubar_props, &mut state.menubar_state)
    }
}
```

## Best Practices

1. **Keep menu hierarchies shallow** - Avoid deeply nested submenus
2. **Use consistent naming** - Follow platform conventions for menu item names
3. **Group related items** - Use separators to group related functionality
4. **Provide shortcuts** - Add keyboard shortcuts for frequently used actions
5. **Handle disabled states** - Clearly indicate when items are not available
6. **Test accessibility** - Ensure keyboard navigation works properly
7. **Use appropriate menu types** - Choose the right menu type for the use case

## Examples

See the [menus manual chapter](../../../manual/menus.md) for complete examples of all menu types and features.

## Future Enhancements

- **Cascading menus**: Support for multi-level dropdown menus
- **Menu templates**: Pre-built menu structures for common use cases
- **Animation support**: Smooth transitions for menu open/close
- **Touch support**: Better support for touch interfaces
- **Internationalization**: Support for RTL languages and localization
- **Custom renderers**: Allow custom rendering for special menu items