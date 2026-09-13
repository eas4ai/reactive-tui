Example from src/backend/crossterm/src/lib.rs:99

```no_run
use std::io::{Write, stdout};
use crossterm::{QueueableCommand, cursor};

let mut stdout = stdout();
stdout.queue(cursor::MoveTo(5,5));

// some other code ...

stdout.flush();
```

Example from src/backend/crossterm/src/lib.rs:116

```no_run
use std::io::{Write, stdout};
use crossterm::{queue, QueueableCommand, cursor};

let mut stdout = stdout();
queue!(stdout,  cursor::MoveTo(5, 5));

// some other code ...

// move operation is performed only if we flush the buffer.
stdout.flush();
```

Example from src/backend/crossterm/src/lib.rs:147

```no_run
use std::io::{Write, stdout};
use crossterm::{ExecutableCommand, cursor};

let mut stdout = stdout();
stdout.execute(cursor::MoveTo(5,5));
```

Example from src/backend/crossterm/src/lib.rs:159

```no_run
use std::io::{stdout, Write};
use crossterm::{execute, ExecutableCommand, cursor};

let mut stdout = stdout();
execute!(stdout, cursor::MoveTo(5, 5));
```

Example from src/backend/crossterm/src/lib.rs:176

```no_run
use std::io::{self, Write};
use crossterm::{
    ExecutableCommand, QueueableCommand,
    terminal, cursor, style::{self, Stylize}
};

fn main() -> io::Result<()> {
  let mut stdout = io::stdout();

  stdout.execute(terminal::Clear(terminal::ClearType::All))?;

  for y in 0..40 {
    for x in 0..150 {
      if (y == 0 || y == 40 - 1) || (x == 0 || x == 150 - 1) {
        // in this loop we are more efficient by not flushing the buffer.
        stdout
          .queue(cursor::MoveTo(x,y))?
          .queue(style::PrintStyledContent( "█".magenta()))?;
      }
    }
  }
  stdout.flush()?;
  Ok(())
}
```

Example from src/backend/crossterm/src/lib.rs:205

```no_run
use std::io::{self, Write};
use crossterm::{
    execute, queue,
    style::{self, Stylize}, cursor, terminal
};

fn main() -> io::Result<()> {
  let mut stdout = io::stdout();

  execute!(stdout, terminal::Clear(terminal::ClearType::All))?;

  for y in 0..40 {
    for x in 0..150 {
      if (y == 0 || y == 40 - 1) || (x == 0 || x == 150 - 1) {
        // in this loop we are more efficient by not flushing the buffer.
        queue!(stdout, cursor::MoveTo(x,y), style::PrintStyledContent( "█".magenta()))?;
      }
    }
  }
  stdout.flush()?;
  Ok(())
}
```
