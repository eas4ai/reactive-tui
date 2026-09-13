Example from src/backend/crossterm/src/style/types/colors.rs:13

```no_run
use crossterm::style::{Color, Colors, Colored};

// An example color, loaded from a config, file in ANSI format.
let config_color = "38;2;23;147;209";

// Default to green text on a black background.
let default_colors = Colors::new(Color::Green, Color::Black);
// Load a colored value from a config and override the default colors
let colors = match Colored::parse_ansi(config_color) {
    Some(colored) => default_colors.then(&colored.into()),
    None => default_colors,
};
```
