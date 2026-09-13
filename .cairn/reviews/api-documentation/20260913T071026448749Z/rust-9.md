Example from src/backend/crossterm/src/cursor.rs:15

```no_run
use std::io::{self, Write};

use crossterm::{
    ExecutableCommand, execute,
    cursor::{DisableBlinking, EnableBlinking, MoveTo, RestorePosition, SavePosition}
};

fn main() -> io::Result<()> {
    // with macro
    execute!(
        io::stdout(),
        SavePosition,
        MoveTo(10, 10),
        EnableBlinking,
        DisableBlinking,
        RestorePosition
    );

  // with function
  io::stdout()
    .execute(MoveTo(11,11))?
    .execute(RestorePosition);

 Ok(())
}
```
