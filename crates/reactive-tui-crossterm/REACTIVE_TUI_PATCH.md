# Crossterm input readiness repair

Upstream: https://github.com/crossterm-rs/crossterm, crates.io release 0.29.0.
Archive SHA-256: d8b9f2e4c67f833b660cdb0a3523065869fb35570177239812ed4c905aeff87b

Preserve pending Unix Mio readiness across returned events and check input
readiness before reading a retained token. Regression coverage includes
controlled queued input bursts, mixed readiness, zero-timeout exhaustion and
application resize workflows.

The Unix input and terminal code also serves the Kitty keyboard protocol and
the startup queries:

- `KeyboardEnhancementFlags::REPORT_ASSOCIATED_TEXT` is defined; upstream
  left it commented out. The `CSI u` parser reads the text a key types: a
  single character becomes the key's `KeyCode::Char`, and longer text one
  `Char` press per character. Key number 0, text that no known key produced,
  is dropped when it carries no text.
- The keyboard flags reply, `ESC [ ? flags u`, is read as a decimal number
  with all five bits. Upstream read the byte of its first digit.
- `terminal::query_startup` and `StartupReplies` (Unix, `events` feature)
  ask for the keyboard flags, the background color (`OSC 11 ; ?`) and the
  device attributes, whose reply ends the exchange. Keys typed meanwhile stay
  queued for the next read. While the replies are due, a process-wide flag,
  `STARTUP_REPLIES_PENDING`, lets the parser read `ESC ] 11 ;` as the
  background reply (`InternalEvent::BackgroundColor`) instead of Alt+].
- The Unix event sources parse again the bytes left after each event, so a
  key read in the same buffer as a reply stays apart from it.

Windows code remains upstream.

The upstream event-stream-async-std example and its async-std
dev-dependency are removed: async-std is discontinued (RUSTSEC-2025-0052).
The event-stream-tokio example shows the same event stream on tokio.

The original MIT license is in LICENSE.

Reactive TUI publishes this implementation as `reactive-tui-crossterm` and
aliases it to the Rust crate name `crossterm`. Downstream builds therefore use
the repaired implementation without a root-only Cargo patch.
