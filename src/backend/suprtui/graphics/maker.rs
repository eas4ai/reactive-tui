//! Canvas pictures made ready for the terminal on a thread of their own,
//! named `rtui-picture-` and a number, which the App does not wait for
//! (GFX-009): copied out of the canvas's frame, written to shared memory,
//! or encoded as base64 or Sixel. The backend's worker hands each canvas's
//! newest picture over and goes on; a picture handed over while another of
//! the same canvas still waits replaces it, so at most one waits. The
//! thread owns the shared-memory objects it makes (GFX-005), and removes
//! those the terminal has not read when it stops.

use super::{coverage::Coverage, draw_plane, shared, PixelLayers, RasterPlane};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;

/// The most bytes one picture may take, as a frame's pictures may.
const ROOM: usize = 64 * 1024 * 1024;

/// A canvas's picture to make ready: the plane it is for, its place among
/// the frame's planes, and what the frame shows below it.
pub(super) struct Job<P> {
    pub plane: P,
    pub z: usize,
    pub cell: (u16, u16),
    pub blend_legacy: bool,
    pub below: PixelLayers,
}

/// A picture made ready for the plane it names.
pub(super) struct Made<P> {
    pub plane: P,
    pub covered: Coverage,
    /// What shows the picture, from a cursor move to the plane's corner.
    pub bytes: Vec<u8>,
    /// The name of the thread that made it ready.
    pub thread: String,
    /// Whether no newer picture of its canvas waited when it was ready.
    pub latest: bool,
}

/// What the worker asks of the thread, and the picture it works on.
struct Queue<P> {
    /// At most one picture per canvas, in the order they came.
    waiting: Vec<Job<P>>,
    /// Whether a picture is being made ready.
    working: bool,
    /// The canvases still shown, when that changed: the shared-memory
    /// objects of the others are removed.
    keep: Option<Vec<u32>>,
    /// Remove every shared-memory object the terminal has not read.
    forget: bool,
    stop: bool,
}

pub(super) struct Maker<P> {
    queue: Arc<(Mutex<Queue<P>>, Condvar)>,
    made: mpsc::Receiver<Made<P>>,
    thread: Option<JoinHandle<()>>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

impl<P: RasterPlane> Maker<P> {
    /// Start the thread.
    pub fn start() -> std::io::Result<Self> {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let queue = Arc::new((
            Mutex::new(Queue {
                waiting: Vec::new(),
                working: false,
                keep: None,
                forget: false,
                stop: false,
            }),
            Condvar::new(),
        ));
        let (send, made) = mpsc::channel();
        let theirs = Arc::clone(&queue);
        let thread = std::thread::Builder::new()
            .name(format!(
                "rtui-picture-{}",
                NEXT.fetch_add(1, Ordering::Relaxed)
            ))
            .spawn(move || run(&theirs, &send))?;
        Ok(Self {
            queue,
            made,
            thread: Some(thread),
        })
    }

    /// Hand `job` over. It replaces a picture of the same canvas that still
    /// waits.
    pub fn submit(&self, job: Job<P>) {
        let (queue, ready) = &*self.queue;
        let mut queue = lock(queue);
        match queue
            .waiting
            .iter_mut()
            .find(|waiting| waiting.plane.id() == job.plane.id())
        {
            Some(waiting) => *waiting = job,
            None => queue.waiting.push(job),
        }
        ready.notify_one();
    }

    /// Remove the shared-memory objects of every canvas but those `shown`.
    pub fn keep(&self, shown: Vec<u32>) {
        let (queue, ready) = &*self.queue;
        lock(queue).keep = Some(shown);
        ready.notify_one();
    }

    /// Remove every shared-memory object the terminal has not read.
    pub fn forget(&self) {
        let (queue, ready) = &*self.queue;
        lock(queue).forget = true;
        ready.notify_one();
    }

    /// Whether a picture waits or is being made ready. Asked before the
    /// pictures made so far are taken: a picture made after that was
    /// waiting or being made when this answered.
    pub fn making(&self) -> bool {
        let queue = lock(&self.queue.0);
        queue.working || !queue.waiting.is_empty()
    }

    /// The pictures made ready since the last call, oldest first.
    pub fn made(&self) -> Vec<Made<P>> {
        self.made.try_iter().collect()
    }

    /// How many pictures wait, not counting the one being made ready.
    #[cfg(test)]
    pub fn waiting(&self) -> usize {
        lock(&self.queue.0).waiting.len()
    }
}

impl<P> Drop for Maker<P> {
    /// Stop the thread once it has made the picture it works on, and wait
    /// for it, so the shared-memory objects it made are removed.
    fn drop(&mut self) {
        let (queue, ready) = &*self.queue;
        lock(queue).stop = true;
        ready.notify_all();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn run<P: RasterPlane>(queue: &(Mutex<Queue<P>>, Condvar), made: &mpsc::Sender<Made<P>>) {
    crate::platform::keep_full_speed();
    let thread = std::thread::current().name().unwrap_or_default().to_owned();
    let mut objects = shared::Pictures::default();
    let (lock_queue, ready) = queue;
    loop {
        let job = {
            let mut queue = lock(lock_queue);
            loop {
                if let Some(shown) = queue.keep.take() {
                    objects.keep(&shown);
                }
                if std::mem::take(&mut queue.forget) {
                    objects.forget();
                }
                if queue.stop {
                    return;
                }
                if !queue.waiting.is_empty() {
                    break;
                }
                queue = ready.wait(queue).unwrap_or_else(PoisonError::into_inner);
            }
            queue.working = true;
            queue.waiting.remove(0)
        };
        let Job {
            plane,
            z,
            cell,
            blend_legacy,
            mut below,
        } = job;
        let mut rastered = 0;
        // A panic while a picture is made ready ends neither the thread nor
        // the App: the picture is refused as any other that cannot be
        // made.
        let drawn = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            draw_plane(
                &plane,
                z,
                false,
                cell,
                blend_legacy,
                &mut below,
                &mut rastered,
                ROOM,
                &mut objects,
            )
        }))
        .unwrap_or_else(|_| {
            Err(crate::error::ReactiveError::resource(
                "the canvas picture could not be made ready",
            ))
        });
        // A picture that cannot be shown is left out and the canvas is told
        // why (GFX-007).
        let drawn = drawn.map_err(|error| plane.refuse(&error.to_string())).ok();
        // The picture goes to the worker before the thread says it is no
        // longer working, so a worker that saw it working takes it.
        let mut queue = lock(lock_queue);
        let gone = match drawn {
            Some((covered, bytes)) => {
                let latest = !queue
                    .waiting
                    .iter()
                    .any(|waiting| waiting.plane.id() == plane.id());
                made.send(Made {
                    plane,
                    covered,
                    bytes,
                    thread: thread.clone(),
                    latest,
                })
                .is_err()
            }
            None => false,
        };
        queue.working = false;
        if gone {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Job, Maker, PixelLayers};
    use crate::{
        backend::suprtui::graphics::RasterPlane, error::Result,
        widgets::display::image::paint::ImageProtocol,
    };
    use std::time::{Duration, Instant};

    /// A Sixel picture of one canvas that takes `slow` to copy out.
    #[derive(Clone, PartialEq)]
    struct Slow {
        id: u32,
        frame: u32,
        slow: Duration,
    }
    impl RasterPlane for Slow {
        fn id(&self) -> u32 {
            self.id
        }
        fn protocol(&self) -> ImageProtocol {
            ImageProtocol::Sixel
        }
        fn quality(&self) -> crate::widgets::ImageQuality {
            crate::widgets::ImageQuality::Fast
        }
        fn position(&self) -> (u32, u32) {
            (0, 0)
        }
        fn raster(&self, _: (u16, u16)) -> Result<image::RgbaImage> {
            std::thread::sleep(self.slow);
            Ok(image::RgbaImage::from_pixel(
                6,
                6,
                image::Rgba([9, 9, 9, 255]),
            ))
        }
        fn background(&self, _: u32, _: u32, _: (u16, u16)) -> image::Rgba<u8> {
            image::Rgba([0, 0, 0, 255])
        }
    }

    fn job(frame: u32, slow: Duration) -> Job<Slow> {
        Job {
            plane: Slow { id: 7, frame, slow },
            z: 0,
            cell: (1, 1),
            blend_legacy: true,
            below: PixelLayers::new((1, 1)),
        }
    }

    #[test]
    fn gfx_009_a_picture_handed_over_while_another_waits_replaces_it() {
        let maker = Maker::start().expect("the picture thread");
        // The first picture keeps the thread busy while three more come.
        maker.submit(job(0, Duration::from_millis(300)));
        let started = Instant::now();
        while !maker.making() || maker.waiting() > 0 {
            assert!(
                started.elapsed() < Duration::from_secs(5),
                "the thread took no picture"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        for frame in 1..4 {
            maker.submit(job(frame, Duration::ZERO));
        }
        let waiting = maker.waiting();
        let mut made = Vec::new();
        let started = Instant::now();
        while made.len() < 2 && started.elapsed() < Duration::from_secs(5) {
            made.extend(
                maker
                    .made()
                    .into_iter()
                    .map(|made| (made.plane.frame, made.latest, made.thread)),
            );
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(
            waiting == 1
                && made.iter().map(|(frame, _, _)| *frame).collect::<Vec<_>>() == [0, 3]
                && made.iter().map(|(_, latest, _)| *latest).collect::<Vec<_>>() == [false, true]
                && made.iter().all(|(_, _, thread)| thread.starts_with("rtui-picture-")),
            "GFX-009: with three pictures handed over while one was made ready, {waiting} waited, and the pictures made (frame, latest, thread) were {made:?}"
        );
    }
}
