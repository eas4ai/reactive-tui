Example from src/backend/crossterm/src/terminal.rs:65

```no_run
use std::io::{self, Write};
use crossterm::{execute, terminal::{ScrollUp, SetSize, size}};

fn main() -> io::Result<()> {
    let (cols, rows) = size()?;
    // Resize terminal and scroll up.
    execute!(
        io::stdout(),
        SetSize(10, 10),
        ScrollUp(5)
    )?;

    // Be a good citizen, cleanup
    execute!(io::stdout(), SetSize(cols, rows))?;
    Ok(())
}
```

Example from src/backend/crossterm/src/terminal.rs:204

```no_run
use std::io::{self, Write};
use crossterm::{execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen}};

fn main() -> io::Result<()> {
    execute!(io::stdout(), EnterAlternateScreen)?;

    // Do anything on the alternate screen

    execute!(io::stdout(), LeaveAlternateScreen)
}
```

Example from src/backend/crossterm/src/terminal.rs:242

```no_run
use std::io::{self, Write};
use crossterm::{execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen}};

fn main() -> io::Result<()> {
    execute!(io::stdout(), EnterAlternateScreen)?;

    // Do anything on the alternate screen

    execute!(io::stdout(), LeaveAlternateScreen)
}
```

Example from src/backend/crossterm/src/terminal.rs:418

```no_run
use std::io::{self, Write};
use crossterm::{execute, terminal::{BeginSynchronizedUpdate, EndSynchronizedUpdate}};

fn main() -> io::Result<()> {
    execute!(io::stdout(), BeginSynchronizedUpdate)?;

    // Anything performed here will not be rendered until EndSynchronizedUpdate is called.

    execute!(io::stdout(), EndSynchronizedUpdate)?;
    Ok(())
}
```

Example from src/backend/crossterm/src/terminal.rs:471

```no_run
use std::io::{self, Write};
use crossterm::{execute, terminal::{BeginSynchronizedUpdate, EndSynchronizedUpdate}};

fn main() -> io::Result<()> {
    execute!(io::stdout(), BeginSynchronizedUpdate)?;

    // Anything performed here will not be rendered until EndSynchronizedUpdate is called.

    execute!(io::stdout(), EndSynchronizedUpdate)?;
    Ok(())
}
```
