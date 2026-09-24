use std::{collections::VecDeque, io, time::Duration};

use mio::{unix::SourceFd, Events, Interest, Poll, Token};
use signal_hook_mio::v1_0::Signals;

#[cfg(feature = "event-stream")]
use crate::event::sys::Waker;
use crate::event::{
    source::EventSource, sys::unix::parse::parse_event, timeout::PollTimeout, Event, InternalEvent,
};
use crate::terminal::sys::file_descriptor::{tty_fd, FileDesc};

// Tokens to identify file descriptor
const TTY_TOKEN: Token = Token(0);
const SIGNAL_TOKEN: Token = Token(1);
#[cfg(feature = "event-stream")]
const WAKE_TOKEN: Token = Token(2);

// I (@zrzka) wasn't able to read more than 1_022 bytes when testing
// reading on macOS/Linux -> we don't need bigger buffer and 1k of bytes
// is enough.
const TTY_BUFFER_SIZE: usize = 1_024;

pub(crate) struct UnixInternalEventSource {
    poll: Poll,
    events: Events,
    pending: VecDeque<Token>,
    parser: Parser,
    tty_buffer: [u8; TTY_BUFFER_SIZE],
    tty_fd: FileDesc<'static>,
    signals: Signals,
    #[cfg(feature = "event-stream")]
    waker: Waker,
}

impl UnixInternalEventSource {
    pub fn new() -> io::Result<Self> {
        UnixInternalEventSource::from_file_descriptor(tty_fd()?)
    }

    pub(crate) fn from_file_descriptor(input_fd: FileDesc<'static>) -> io::Result<Self> {
        let poll = Poll::new()?;
        let registry = poll.registry();

        let tty_raw_fd = input_fd.raw_fd();
        let mut tty_ev = SourceFd(&tty_raw_fd);
        registry.register(&mut tty_ev, TTY_TOKEN, Interest::READABLE)?;

        let mut signals = Signals::new([signal_hook::consts::SIGWINCH])?;
        registry.register(&mut signals, SIGNAL_TOKEN, Interest::READABLE)?;

        #[cfg(feature = "event-stream")]
        let waker = Waker::new(registry, WAKE_TOKEN)?;

        Ok(UnixInternalEventSource {
            poll,
            events: Events::with_capacity(3),
            pending: VecDeque::with_capacity(3),
            parser: Parser::default(),
            tty_buffer: [0u8; TTY_BUFFER_SIZE],
            tty_fd: input_fd,
            signals,
            #[cfg(feature = "event-stream")]
            waker,
        })
    }
}

impl EventSource for UnixInternalEventSource {
    fn try_read(&mut self, timeout: Option<Duration>) -> io::Result<Option<InternalEvent>> {
        if let Some(event) = self.parser.next() {
            return Ok(Some(event));
        }

        let timeout = PollTimeout::new(timeout);

        loop {
            // Retained TTY work must not hide newly arriving resize/wake
            // edges. Poll without waiting while work remains, merging tokens
            // rather than replacing the batch from the previous call.
            let wait = if self.pending.is_empty() {
                timeout.leftover()
            } else {
                Some(Duration::ZERO)
            };
            if let Err(error) = self.poll.poll(&mut self.events, wait) {
                if error.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(error);
            }
            for token in self.events.iter().map(|event| event.token()) {
                if !self.pending.contains(&token) {
                    self.pending.push_back(token);
                }
            }
            if self.pending.is_empty() {
                return Ok(None);
            }

            while let Some(token) = self.pending.pop_front() {
                match token {
                    TTY_TOKEN => {
                        // Mio readiness is edge-triggered. Keep the token until
                        // drained, but never assume the caller's fd is nonblocking.
                        match tty_readable(&self.tty_fd) {
                            Ok(false) => continue,
                            Ok(true) => {}
                            Err(error) => {
                                if error.kind() == io::ErrorKind::Interrupted {
                                    self.pending.push_front(TTY_TOKEN);
                                }
                                return Err(error);
                            }
                        }
                        match self.tty_fd.read(&mut self.tty_buffer) {
                            Ok(0) => {
                                return Err(io::Error::new(
                                    io::ErrorKind::UnexpectedEof,
                                    "terminal input closed",
                                ))
                            }
                            Ok(count) => {
                                self.pending.push_back(TTY_TOKEN);
                                self.parser
                                    .advance(&self.tty_buffer[..count], count == TTY_BUFFER_SIZE);
                                if let Some(event) = self.parser.next() {
                                    return Ok(Some(event));
                                }
                            }
                            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                            Err(error) if error.kind() == io::ErrorKind::Interrupted => {
                                self.pending.push_front(TTY_TOKEN);
                            }
                            Err(error) => return Err(error),
                        }
                    }
                    SIGNAL_TOKEN => {
                        if self.signals.pending().next() == Some(signal_hook::consts::SIGWINCH) {
                            let new_size = crate::terminal::size()?;
                            return Ok(Some(InternalEvent::Event(Event::Resize(
                                new_size.0, new_size.1,
                            ))));
                        }
                    }
                    #[cfg(feature = "event-stream")]
                    WAKE_TOKEN => {
                        return Err(io::Error::new(
                            io::ErrorKind::Interrupted,
                            "Poll operation was woken up by `Waker::wake`",
                        ));
                    }
                    _ => unreachable!("Synchronize Evented handle registration & token handling"),
                }
                if timeout.elapsed() {
                    return Ok(None);
                }
            }

            // Processing above can take some time, check if timeout expired
            if timeout.elapsed() {
                return Ok(None);
            }
        }
    }

    #[cfg(feature = "event-stream")]
    fn waker(&self) -> Waker {
        self.waker.clone()
    }
}

// Poll is unsuitable for /dev/tty on Apple platforms; select supports it.
#[cfg(target_vendor = "apple")]
fn tty_readable(fd: &FileDesc<'_>) -> io::Result<bool> {
    use rustix::event::{fd_set_insert, fd_set_num_elements, select, FdSetElement, Timespec};
    let raw = fd.raw_fd();
    let mut read = vec![FdSetElement::default(); fd_set_num_elements(1, raw + 1)];
    fd_set_insert(&mut read, raw);
    // SAFETY: the only selected descriptor is borrowed from the live FileDesc;
    // the set has the capacity required for raw + 1 descriptors.
    unsafe {
        select(
            raw + 1,
            Some(&mut read),
            None,
            None,
            Some(&Timespec::default()),
        )
    }
    .map(|count| count > 0)
    .map_err(Into::into)
}

#[cfg(not(target_vendor = "apple"))]
fn tty_readable(fd: &FileDesc<'_>) -> io::Result<bool> {
    use rustix::event::{poll, PollFd, PollFlags, Timespec};
    use std::os::fd::BorrowedFd;
    // SAFETY: this borrow cannot outlive the FileDesc passed to this function.
    let borrowed = unsafe { BorrowedFd::borrow_raw(fd.raw_fd()) };
    let mut fds = [PollFd::new(&borrowed, PollFlags::IN)];
    poll(&mut fds, Some(&Timespec::default()))?;
    if fds[0].revents().contains(PollFlags::NVAL) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "terminal input fd is invalid",
        ));
    }
    Ok(fds[0]
        .revents()
        .intersects(PollFlags::IN | PollFlags::HUP | PollFlags::ERR))
}

//
// Following `Parser` structure exists for two reasons:
//
//  * mimic anes Parser interface
//  * move the advancing, parsing, ... stuff out of the `try_read` method
//
#[derive(Debug)]
struct Parser {
    buffer: Vec<u8>,
    internal_events: VecDeque<InternalEvent>,
}

impl Default for Parser {
    fn default() -> Self {
        Parser {
            // This buffer is used for -> 1 <- ANSI escape sequence. Are we
            // aware of any ANSI escape sequence that is bigger? Can we make
            // it smaller?
            //
            // Probably not worth spending more time on this as "there's a plan"
            // to use the anes crate parser.
            buffer: Vec::with_capacity(256),
            // TTY_BUFFER_SIZE is 1_024 bytes. How many ANSI escape sequences can
            // fit? What is an average sequence length? Let's guess here
            // and say that the average ANSI escape sequence length is 8 bytes. Thus
            // the buffer size should be 1024/8=128 to avoid additional allocations
            // when processing large amounts of data.
            //
            // There's no need to make it bigger, because when you look at the `try_read`
            // method implementation, all events are consumed before the next TTY_BUFFER
            // is processed -> events pushed.
            internal_events: VecDeque::with_capacity(128),
        }
    }
}

impl Parser {
    fn advance(&mut self, buffer: &[u8], more: bool) {
        for (idx, byte) in buffer.iter().enumerate() {
            let more = idx + 1 < buffer.len() || more;

            self.buffer.push(*byte);

            match parse_event(&self.buffer, more) {
                Ok(Some(ie)) => {
                    self.internal_events.push_back(ie);
                    self.buffer.clear();
                }
                Ok(None) => {
                    // Event can't be parsed, because we don't have enough bytes for
                    // the current sequence. Keep the buffer and process next bytes.
                }
                Err(_) => {
                    // Event can't be parsed (not enough parameters, parameter is not a number, ...).
                    // Clear the buffer and continue with another sequence.
                    self.buffer.clear();
                }
            }
        }
    }
}

impl Iterator for Parser {
    type Item = InternalEvent;

    fn next(&mut self) -> Option<Self::Item> {
        self.internal_events.pop_front()
    }
}

#[cfg(test)]
mod readiness_tests {
    use super::*;
    use std::io::Write;
    use std::os::unix::net::UnixStream;

    fn source() -> (UnixStream, UnixInternalEventSource) {
        let (writer, reader) = UnixStream::pair().unwrap();
        // Deliberately keep the descriptor blocking: stale readiness must not
        // make a subsequent zero-timeout poll block after the final byte.
        #[cfg(not(feature = "libc"))]
        let fd = FileDesc::Owned(reader.into());
        #[cfg(feature = "libc")]
        let fd = {
            use std::os::fd::IntoRawFd;
            FileDesc::new(reader.into_raw_fd(), true)
        };
        (
            writer,
            UnixInternalEventSource::from_file_descriptor(fd).unwrap(),
        )
    }

    #[test]
    fn one_readiness_edge_retains_a_burst_larger_than_the_read_buffer() {
        let (mut writer, mut source) = source();
        let count = TTY_BUFFER_SIZE * 3 + 7;
        writer.write_all(&vec![b'e'; count]).unwrap();
        // Consume the actual readiness batch once. Further input writes are
        // forbidden: the reader must retain work across every early return.
        source
            .poll
            .poll(&mut source.events, Some(Duration::ZERO))
            .unwrap();
        assert!(source.events.iter().any(|event| event.token() == TTY_TOKEN));
        source
            .pending
            .extend(source.events.iter().map(|event| event.token()));
        for _ in 0..count {
            assert!(matches!(source.try_read(Some(Duration::ZERO)).unwrap(),
                Some(InternalEvent::Event(Event::Key(key))) if key.code == crate::event::KeyCode::Char('e')));
        }
        assert!(source.try_read(Some(Duration::ZERO)).unwrap().is_none());
    }

    #[cfg(feature = "event-stream")]
    #[test]
    fn new_wake_is_harvested_while_a_tty_burst_remains_readable() {
        let (mut writer, mut source) = source();
        writer.write_all(&vec![b'e'; TTY_BUFFER_SIZE * 4]).unwrap();
        for _ in 0..TTY_BUFFER_SIZE {
            assert!(source.try_read(Some(Duration::ZERO)).unwrap().is_some());
        }
        assert!(source.pending.contains(&TTY_TOKEN));
        source.waker.wake().unwrap();
        // At most one already retained TTY buffer may precede the new wake.
        // No additional input edge is supplied to rescue the poller.
        // The wake reads as an interrupted poll.
        let woke = (0..=TTY_BUFFER_SIZE).any(|_| match source.try_read(Some(Duration::ZERO)) {
            Err(error) if error.kind() == io::ErrorKind::Interrupted => true,
            Ok(Some(InternalEvent::Event(Event::Key(_)))) => false,
            other => panic!("unexpected result before wake: {other:?}"),
        });
        assert!(woke, "new wake starved behind retained TTY readiness");
    }

    #[cfg(feature = "event-stream")]
    #[test]
    fn wake_return_preserves_later_readiness_without_another_input_edge() {
        let (mut writer, mut source) = source();
        writer.write_all(b"e").unwrap();
        source
            .poll
            .poll(&mut source.events, Some(Duration::ZERO))
            .unwrap();
        assert!(source.events.iter().any(|event| event.token() == TTY_TOKEN));
        // Fix the batch order directly without process-global signal delivery.
        source.pending.extend([WAKE_TOKEN, TTY_TOKEN]);
        assert_eq!(
            source.try_read(Some(Duration::ZERO)).unwrap_err().kind(),
            io::ErrorKind::Interrupted
        );
        assert!(matches!(source.try_read(Some(Duration::ZERO)).unwrap(),
            Some(InternalEvent::Event(Event::Key(key))) if key.code == crate::event::KeyCode::Char('e')));
        assert!(source.try_read(Some(Duration::ZERO)).unwrap().is_none());
    }
}
