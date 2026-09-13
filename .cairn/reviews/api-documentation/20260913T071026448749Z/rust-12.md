Example from src/backend/crossterm/src/style/stylize.rs:62

```no_run
use crossterm::style::Stylize;

println!("{}", "Bold text".bold());
println!("{}", "Underlined text".underlined());
println!("{}", "Negative text".negative());
println!("{}", "Red on blue".red().on_blue());
```
