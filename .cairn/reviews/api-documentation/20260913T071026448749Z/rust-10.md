Example from src/backend/crossterm/src/event.rs:30

```no_run
#![cfg(feature = "bracketed-paste")]
use crossterm::{
    event::{
        read, DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste,
        EnableFocusChange, EnableMouseCapture, Event,
    },
    execute,
};

fn print_events() -> std::io::Result<()> {
    execute!(
         std::io::stdout(),
         EnableBracketedPaste,
         EnableFocusChange,
         EnableMouseCapture
    )?;
    loop {
        // `read()` blocks until an `Event` is available
        match read()? {
            Event::FocusGained => println!("FocusGained"),
            Event::FocusLost => println!("FocusLost"),
            Event::Key(event) => println!("{:?}", event),
            Event::Mouse(event) => println!("{:?}", event),
            #[cfg(feature = "bracketed-paste")]
            Event::Paste(data) => println!("{:?}", data),
            Event::Resize(width, height) => println!("New size {}x{}", width, height),
        }
    }
    execute!(
        std::io::stdout(),
        DisableBracketedPaste,
        DisableFocusChange,
        DisableMouseCapture
    )?;
    Ok(())
}
```

Example from src/backend/crossterm/src/event.rs:71

```no_run
#![cfg(feature = "bracketed-paste")]
use std::{time::Duration, io};

use crossterm::{
    event::{
        poll, read, DisableBracketedPaste, DisableFocusChange, DisableMouseCapture,
        EnableBracketedPaste, EnableFocusChange, EnableMouseCapture, Event,
    },
    execute,
};

fn print_events() -> io::Result<()> {
    execute!(
         std::io::stdout(),
         EnableBracketedPaste,
         EnableFocusChange,
         EnableMouseCapture
    )?;
    loop {
        // `poll()` waits for an `Event` for a given time period
        if poll(Duration::from_millis(500))? {
            // It's guaranteed that the `read()` won't block when the `poll()`
            // function returns `true`
            match read()? {
                Event::FocusGained => println!("FocusGained"),
                Event::FocusLost => println!("FocusLost"),
                Event::Key(event) => println!("{:?}", event),
                Event::Mouse(event) => println!("{:?}", event),
                #[cfg(feature = "bracketed-paste")]
                Event::Paste(data) => println!("Pasted {:?}", data),
                Event::Resize(width, height) => println!("New size {}x{}", width, height),
            }
        } else {
            // Timeout expired and no `Event` is available
        }
    }
    execute!(
        std::io::stdout(),
        DisableBracketedPaste,
        DisableFocusChange,
        DisableMouseCapture
    )?;
    Ok(())
}
```

Example from src/backend/crossterm/src/event.rs:180

```no_run
use std::{time::Duration, io};
use crossterm::{event::poll};

fn is_event_available() -> io::Result<bool> {
    // Zero duration says that the `poll` function must return immediately
    // with an `Event` availability information
    poll(Duration::from_secs(0))
}
```

Example from src/backend/crossterm/src/event.rs:193

```no_run
use std::{time::Duration, io};

use crossterm::event::poll;

fn is_event_available() -> io::Result<bool> {
    // Wait for an `Event` availability for 100ms. It returns immediately
    // if an `Event` is/becomes available.
    poll(Duration::from_millis(100))
}
```

Example from src/backend/crossterm/src/event.rs:217

```no_run
use crossterm::event::read;
use std::io;

fn print_events() -> io::Result<bool> {
    loop {
        // Blocks until an `Event` is available
        println!("{:?}", read()?);
    }
}
```

Example from src/backend/crossterm/src/event.rs:231

```no_run
use std::time::Duration;
use std::io;

use crossterm::event::{read, poll};

fn print_events() -> io::Result<bool> {
    loop {
        if poll(Duration::from_millis(100))? {
            // It's guaranteed that `read` won't block, because `poll` returned
            // `Ok(true)`.
            println!("{:?}", read()?);
        } else {
            // Timeout expired, no `Event` is available
        }
    }
}
```

Example from src/backend/crossterm/src/event.rs:460

```no_run
use std::io::{Write, stdout};
use crossterm::execute;
use crossterm::event::{
    KeyboardEnhancementFlags,
    PushKeyboardEnhancementFlags,
    PopKeyboardEnhancementFlags
};

let mut stdout = stdout();

execute!(
    stdout,
    PushKeyboardEnhancementFlags(
        KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
    )
);

// ...

execute!(stdout, PopKeyboardEnhancementFlags);
```

Example from src/backend/crossterm/src/event.rs:579

```no_run
use crossterm::event;

while !event::read()?.is_key_press() {
    // ...
}
# Ok::<(), std::io::Error>(())
```

Example from src/backend/crossterm/src/event.rs:630

```no_run
use crossterm::event;

while let Some(key_event) = event::read()?.as_key_event() {
    // ...
}
# std::io::Result::Ok(())
```

Example from src/backend/crossterm/src/event.rs:698

```no_run
use crossterm::event;

while let Some(mouse_event) = event::read()?.as_mouse_event() {
    // ...
}
# std::io::Result::Ok(())
```

Example from src/backend/crossterm/src/event.rs:720

```no_run
use crossterm::event;

while let Some(paste) = event::read()?.as_paste_event() {
    // ...
}
# std::io::Result::Ok(())
```

Example from src/backend/crossterm/src/event.rs:743

```no_run
use crossterm::event;

while let Some((columns, rows)) = event::read()?.as_resize_event() {
    // ...
}
# std::io::Result::Ok(())
```
