Example from src/backend/crossterm/src/style/types/attribute.rs:49

```no_run
use crossterm::style::Attribute;

println!(
    "{} Underlined {} No Underline",
    Attribute::Underlined,
    Attribute::NoUnderline
);
```

Example from src/backend/crossterm/src/style/types/attribute.rs:61

```no_run
use crossterm::style::Stylize;

println!("{}", "Bold text".bold());
println!("{}", "Underlined text".underlined());
println!("{}", "Negative text".negative());
```
