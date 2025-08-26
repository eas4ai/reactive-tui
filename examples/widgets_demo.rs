use reactive_tui::component::{Component, Element};
use reactive_tui::widgets::input::{
    TextInput, TextInputProps,
    Select, SelectProps, SelectOption,
    Checkbox, CheckboxProps,
};

fn main() {
    println!("Widget Components Demo");
    println!("======================\n");
    
    // TextInput Demo
    println!("1. TextInput Component:");
    let mut text_input = TextInput::new(TextInputProps {
        value: "Hello World".to_string(),
        placeholder: Some("Enter text...".to_string()),
        max_length: Some(20),
        mask: None,
        disabled: false,
        width: Some(30),
        validator_pattern: Some("alphanumeric".to_string()),
        error_message: Some("Only alphanumeric allowed".to_string()),
    });
    
    let text_props = TextInputProps {
        value: "Hello World".to_string(),
        placeholder: Some("Enter text...".to_string()),
        max_length: Some(20),
        mask: None,
        disabled: false,
        width: Some(30),
        validator_pattern: Some("alphanumeric".to_string()),
        error_message: Some("Only alphanumeric allowed".to_string()),
    };
    
    let mut text_state = Default::default();
    let rendered = text_input.render(&text_props, &text_state);
    println!("Normal: {}", get_text(&rendered));
    
    // Show focused state
    text_state.is_focused = true;
    text_state.cursor_position = 5;
    let rendered = text_input.render(&text_props, &text_state);
    println!("Focused: {}", get_text(&rendered));
    
    // Show password masking
    let pass_props = TextInputProps {
        value: "secret123".to_string(),
        mask: Some('*'),
        ..Default::default()
    };
    let rendered = text_input.render(&pass_props, &text_state);
    println!("Masked: {}", get_text(&rendered));
    
    // Select Demo
    println!("\n2. Select Component:");
    let mut select = Select::<String>::new(SelectProps {
        options: vec![
            SelectOption::new("apple".to_string(), "Apple"),
            SelectOption::new("banana".to_string(), "Banana"),
            SelectOption::new("cherry".to_string(), "Cherry"),
            SelectOption::new("date".to_string(), "Date").disabled(true),
            SelectOption::new("elderberry".to_string(), "Elderberry"),
        ],
        selected: Some("banana".to_string()),
        placeholder: Some("Choose a fruit...".to_string()),
        disabled: false,
        max_visible_items: 3,
        width: Some(20),
    });
    
    let select_props = SelectProps {
        options: vec![
            SelectOption::new("apple".to_string(), "Apple"),
            SelectOption::new("banana".to_string(), "Banana"),
            SelectOption::new("cherry".to_string(), "Cherry"),
            SelectOption::new("date".to_string(), "Date").disabled(true),
            SelectOption::new("elderberry".to_string(), "Elderberry"),
        ],
        selected: Some("banana".to_string()),
        placeholder: Some("Choose a fruit...".to_string()),
        disabled: false,
        max_visible_items: 3,
        width: Some(20),
    };
    
    let mut select_state = Default::default();
    let rendered = select.render(&select_props, &select_state);
    println!("Closed: {}", get_text(&rendered));
    
    // Show open dropdown
    select_state.is_open = true;
    select_state.highlighted_index = 1;
    let rendered = select.render(&select_props, &select_state);
    println!("Open with dropdown:");
    for line in get_text(&rendered).lines() {
        println!("  {}", line);
    }
    
    // Checkbox Demo
    println!("\n3. Checkbox Component:");
    let mut checkbox = Checkbox::new(CheckboxProps::default());
    
    let check_props = CheckboxProps {
        checked: false,
        label: Some("Accept terms".to_string()),
        disabled: false,
        indeterminate: false,
    };
    let mut check_state = Default::default();
    let rendered = checkbox.render(&check_props, &check_state);
    println!("Unchecked: {}", get_text(&rendered));
    
    let check_props = CheckboxProps {
        checked: true,
        label: Some("Accept terms".to_string()),
        disabled: false,
        indeterminate: false,
    };
    let rendered = checkbox.render(&check_props, &check_state);
    println!("Checked: {}", get_text(&rendered));
    
    let check_props = CheckboxProps {
        checked: false,
        label: Some("Select all".to_string()),
        disabled: false,
        indeterminate: true,
    };
    let rendered = checkbox.render(&check_props, &check_state);
    println!("Indeterminate: {}", get_text(&rendered));
    
    let check_props = CheckboxProps {
        checked: true,
        label: Some("Disabled option".to_string()),
        disabled: true,
        indeterminate: false,
    };
    let rendered = checkbox.render(&check_props, &check_state);
    println!("Disabled: {}", get_text(&rendered));
}

fn get_text(element: &Element) -> String {
    match &element.element_type {
        reactive_tui::component::ElementType::Text(text) => text.clone(),
        _ => String::new(),
    }
}