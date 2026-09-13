Example from src/backend/crossterm/src/style.rs:30

```no_run
use std::io::{self, Write};
use crossterm::execute;
use crossterm::style::{Print, SetForegroundColor, SetBackgroundColor, ResetColor, Color, Attribute};

fn main() -> io::Result<()> {
    execute!(
        io::stdout(),
        // Blue foreground
        SetForegroundColor(Color::Blue),
        // Red background
        SetBackgroundColor(Color::Red),
        // Print text
        Print("Blue text on Red.".to_string()),
        // Reset to default colors
        ResetColor
    )
}
```

Example from src/backend/crossterm/src/style.rs:55

```no_run
use crossterm::style::Stylize;

println!("{}", "Red foreground color & blue background.".red().on_blue());
```

Example from src/backend/crossterm/src/style.rs:69

```no_run
use std::io::{self, Write};

use crossterm::execute;
use crossterm::style::{Attribute, Print, SetAttribute};

fn main() -> io::Result<()> {
    execute!(
        io::stdout(),
        // Set to bold
        SetAttribute(Attribute::Bold),
        Print("Bold text here.".to_string()),
        // Reset all attributes
        SetAttribute(Attribute::Reset)
    )
}
```

Example from src/backend/crossterm/src/style.rs:92

```no_run
use crossterm::style::Stylize;

println!("{}", "Bold".bold());
println!("{}", "Underlined".underlined());
println!("{}", "Negative".negative());
```

Example from src/backend/crossterm/src/style.rs:104

```no_run
use crossterm::style::Attribute;

println!(
    "{} Underlined {} No Underline",
    Attribute::Underlined,
    Attribute::NoUnderline
);
```

Example from src/backend/crossterm/src/style.rs:145

```no_run
use crossterm::style::{style, Stylize, Color};

let styled_content = style("Blue colored text on yellow background")
    .with(Color::Blue)
    .on(Color::Yellow);

println!("{}", styled_content);
```

Example from src/backend/crossterm/src/style.rs:273

```no_run
use std::io::{stdout, Write};

use crossterm::execute;
use crossterm::style::{Color::{Green, Black}, Colors, Print, SetColors};

execute!(
    stdout(),
    SetColors(Colors::new(Green, Black)),
    Print("Hello, world!".to_string()),
).unwrap();
```
