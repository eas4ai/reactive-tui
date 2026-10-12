use std::collections::VecDeque;
use std::sync::atomic::Ordering;
use std::time::Duration;

use crossterm_winapi::{Console, Handle, InputRecord, KeyEventRecord};

use crate::event::{
    sys::startup::{Fed, ReplyCollector},
    sys::windows::{
        parse::MouseButtonsPressed,
        poll::WinApiPoll,
        startup::{reply_character, STARTUP_REPLIES_PENDING},
    },
    Event,
};

#[cfg(feature = "event-stream")]
use crate::event::sys::Waker;
use crate::event::{
    source::EventSource,
    sys::windows::parse::{handle_key_event, handle_mouse_event},
    timeout::PollTimeout,
    InternalEvent,
};

pub(crate) struct WindowsEventSource {
    console: Console,
    poll: WinApiPoll,
    surrogate_buffer: Option<u16>,
    mouse_buttons_pressed: MouseButtonsPressed,
    /// Assembles a startup reply from the key records that carry no key
    /// code, during the exchange and after it (INP-013).
    replies: ReplyCollector<KeyEventRecord>,
    /// Events parsed from records the collector gave back, delivered in
    /// their order before any new record is read.
    queued: VecDeque<InternalEvent>,
}

impl WindowsEventSource {
    pub(crate) fn new() -> std::io::Result<WindowsEventSource> {
        let console = Console::from(Handle::current_in_handle()?);
        Ok(WindowsEventSource {
            console,

            #[cfg(not(feature = "event-stream"))]
            poll: WinApiPoll::new(),
            #[cfg(feature = "event-stream")]
            poll: WinApiPoll::new()?,

            surrogate_buffer: None,
            mouse_buttons_pressed: MouseButtonsPressed::default(),
            replies: ReplyCollector::default(),
            queued: VecDeque::new(),
        })
    }

    /// Parses the key records the collector gave back, in their order.
    fn queue_keys(&mut self, records: Vec<KeyEventRecord>) {
        for record in records {
            if let Some(event) = handle_key_event(record, &mut self.surrogate_buffer) {
                self.queued.push_back(InternalEvent::Event(event));
            }
        }
    }

    /// One key record: a key, or part of a reply the collector holds or
    /// completes. A reply completed after the startup exchange ended is
    /// consumed: no one waits for it and it is no key (INP-013).
    fn key_record(&mut self, record: KeyEventRecord) -> Option<InternalEvent> {
        let pending = STARTUP_REPLIES_PENDING.load(Ordering::Acquire);
        let character = reply_character(&record);
        match self.replies.feed(record, character, pending) {
            Fed::Held => None,
            Fed::Reply(reply) if pending => {
                // The device attributes reply ends the exchange.
                if matches!(reply, InternalEvent::PrimaryDeviceAttributes { .. }) {
                    STARTUP_REPLIES_PENDING.store(false, Ordering::Release);
                }
                Some(reply)
            }
            Fed::Reply(_) => None,
            Fed::Keys(records) => {
                self.queue_keys(records);
                self.queued.pop_front()
            }
        }
    }
}

impl EventSource for WindowsEventSource {
    fn try_read(&mut self, timeout: Option<Duration>) -> std::io::Result<Option<InternalEvent>> {
        let poll_timeout = PollTimeout::new(timeout);

        loop {
            if let Some(event) = self.queued.pop_front() {
                return Ok(Some(event));
            }

            if let Some(event_ready) = self.poll.poll(poll_timeout.leftover())? {
                let number = self.console.number_of_console_input_events()?;
                if event_ready && number != 0 {
                    let event = match self.console.read_single_input_event()? {
                        InputRecord::KeyEvent(record) => self.key_record(record),
                        InputRecord::MouseEvent(record) => {
                            let mouse_event =
                                handle_mouse_event(record, &self.mouse_buttons_pressed);
                            self.mouse_buttons_pressed = MouseButtonsPressed {
                                left: record.button_state.left_button(),
                                right: record.button_state.right_button(),
                                middle: record.button_state.middle_button(),
                            };

                            mouse_event.map(InternalEvent::Event)
                        }
                        InputRecord::WindowBufferSizeEvent(record) => {
                            // windows starts counting at 0, unix at 1, add one to replicate unix behaviour.
                            Some(InternalEvent::Event(Event::Resize(
                                (record.size.x as i32 + 1) as u16,
                                (record.size.y as i32 + 1) as u16,
                            )))
                        }
                        InputRecord::FocusEvent(record) => {
                            let event = if record.set_focus {
                                Event::FocusGained
                            } else {
                                Event::FocusLost
                            };
                            Some(InternalEvent::Event(event))
                        }
                        _ => None,
                    };

                    if let Some(event) = event {
                        return Ok(Some(event));
                    }
                }
            }

            if poll_timeout.elapsed() {
                return Ok(None);
            }
        }
    }

    #[cfg(feature = "event-stream")]
    fn waker(&self) -> Waker {
        self.poll.waker()
    }
}
