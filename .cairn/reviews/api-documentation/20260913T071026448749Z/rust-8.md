Example from src/backend/crossterm/src/clipboard.rs:108

```no_run
use crossterm::execute;
use crossterm::clipboard::CopyToClipboard;
// Copy foo to clipboard
execute!(std::io::stdout(), CopyToClipboard::to_clipboard_from("foo"));
// Copy bar to primary
execute!(std::io::stdout(), CopyToClipboard::to_primary_from("bar"));
```

Example from src/backend/crossterm/src/clipboard.rs:179

```no_run
use crossterm::{execute, Command};
use crossterm::clipboard::CopyToClipboard;
execute!(std::io::stdout(), CopyToClipboard::to_clipboard_from("foo"));
```

Example from src/backend/crossterm/src/clipboard.rs:196

```no_run
use crossterm::execute;
use crossterm::clipboard::CopyToClipboard;
execute!(std::io::stdout(), CopyToClipboard::to_primary_from("foo"));
```
