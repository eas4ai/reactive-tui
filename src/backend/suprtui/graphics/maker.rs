//! Canvas pictures made ready for the terminal on a thread of their own,
//! named `rtui-picture-` and a number, which the App does not wait for
//! (GFX-009): copied out of the canvas's frame, written to shared memory,
//! or encoded as base64 or Sixel. The backend's worker hands each canvas's
//! newest picture over and goes on; a picture handed over while another of
//! the same canvas still waits replaces it, so at most one waits. A
//! picture sent through shared memory names its object; the worker keeps
//! it once it writes the picture, and an object whose picture is never
//! written is removed (GFX-005).

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
    /// The shared-memory object the picture's Kitty command names.
    pub object: Option<String>,
}

impl<P> Made<P> {
    /// The picture is not written: its shared-memory object goes.
    pub fn drop_unwritten(self) {
        if let Some(name) = self.object {
            shared::unlink(&name);
        }
    }
}

/// What the worker asks of the thread, and the picture it works on.
struct Queue<P> {
    /// At most one picture per canvas, in the order they came.
    waiting: Vec<Job<P>>,
    /// Whether a picture is being made ready.
    working: bool,
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

    /// Whether a picture waits or is being made ready. Asked before the
    /// pictures made so far are taken: a picture made after that was
    /// waiting or being made when this answered.
    pub fn making(&self) -> bool {
        let queue = lock(&self.queue.0);
        queue.working || !queue.waiting.is_empty()
    }

    /// The newest picture of each canvas made ready since the last call, in
    /// the order they were made. A picture that a newer one of its canvas
    /// followed before the worker looked is dropped, never written, so
    /// pictures never queue (GFX-005).
    pub fn made(&self) -> Vec<Made<P>> {
        let mut newest: Vec<Made<P>> = Vec::new();
        for made in self.made.try_iter() {
            if let Some(older) = newest
                .iter()
                .position(|kept| kept.plane.id() == made.plane.id())
            {
                newest.remove(older).drop_unwritten();
            }
            newest.push(made);
        }
        newest
    }

    /// How many pictures wait, not counting the one being made ready.
    #[cfg(test)]
    pub fn waiting(&self) -> usize {
        lock(&self.queue.0).waiting.len()
    }
}

impl<P> Drop for Maker<P> {
    /// Stop the thread once it has made the picture it works on, wait for
    /// it, and remove the shared-memory objects of the pictures made and
    /// never written.
    fn drop(&mut self) {
        let (queue, ready) = &*self.queue;
        lock(queue).stop = true;
        ready.notify_all();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        for made in self.made.try_iter() {
            made.drop_unwritten();
        }
    }
}

fn run<P: RasterPlane>(queue: &(Mutex<Queue<P>>, Condvar), made: &mpsc::Sender<Made<P>>) {
    crate::platform::keep_full_speed();
    let thread = std::thread::current().name().unwrap_or_default().to_owned();
    let (lock_queue, ready) = queue;
    loop {
        let job = {
            let mut queue = lock(lock_queue);
            loop {
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
        let mut object = None;
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
                &mut super::Objects::Made(&mut object),
            )
        }))
        .unwrap_or_else(|_| {
            Err(crate::error::ReactiveError::resource(
                "the canvas picture could not be made ready",
            ))
        });
        // A picture that cannot be shown is left out and the canvas is told
        // why (GFX-007).
        let drawn = drawn
            .map_err(|error| {
                if let Some(name) = object.take() {
                    shared::unlink(&name);
                }
                plane.refuse(&error.to_string());
            })
            .ok();
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
                    object: object.take(),
                })
                .map_err(|unsent| unsent.0.drop_unwritten())
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
        while !made.iter().any(|(frame, _, _)| *frame == 3)
            && started.elapsed() < Duration::from_secs(5)
        {
            made.extend(
                maker
                    .made()
                    .into_iter()
                    .map(|made| (made.plane.frame, made.latest, made.thread)),
            );
            std::thread::sleep(Duration::from_millis(5));
        }
        // The first picture comes back on its own, or is left out for the
        // last when both were ready as the test looked; the two replaced
        // pictures are never made.
        assert!(
            waiting == 1
                && made.iter().all(|(frame, _, _)| *frame == 0 || *frame == 3)
                && made
                    .last()
                    .is_some_and(|(frame, latest, _)| *frame == 3 && *latest)
                && made.iter().all(|(_, _, thread)| thread.starts_with("rtui-picture-")),
            "GFX-009: with three pictures handed over while one was made ready, {waiting} waited, and the pictures made (frame, latest, thread) were {made:?}"
        );
    }

    #[test]
    fn gfx_005_a_picture_made_ready_after_another_of_its_canvas_leaves_that_one_unwritten() {
        let maker = Maker::start().expect("the picture thread");
        // The first picture is made while the second waits; both are ready
        // before the worker looks, as when it is busy writing a frame.
        maker.submit(job(0, Duration::from_millis(200)));
        let started = Instant::now();
        while !maker.making() || maker.waiting() > 0 {
            assert!(
                started.elapsed() < Duration::from_secs(5),
                "the thread took no picture"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        maker.submit(job(1, Duration::ZERO));
        while maker.making() {
            assert!(
                started.elapsed() < Duration::from_secs(5),
                "the pictures were not made"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        let made: Vec<u32> = maker
            .made()
            .into_iter()
            .map(|made| made.plane.frame)
            .collect();
        assert!(
            made == [1],
            "GFX-005: of two pictures of one canvas both ready when the worker looked, these were handed back to be written: {made:?}"
        );
    }

    /// A canvas's Sixel picture that tells why it was refused.
    #[derive(Clone)]
    struct Told {
        frame: u32,
        refused: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    }
    impl PartialEq for Told {
        fn eq(&self, other: &Self) -> bool {
            self.frame == other.frame
        }
    }
    impl RasterPlane for Told {
        fn id(&self) -> u32 {
            9
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
            Ok(image::RgbaImage::from_pixel(
                6,
                6,
                image::Rgba([9, 9, 9, 255]),
            ))
        }
        fn background(&self, _: u32, _: u32, _: (u16, u16)) -> image::Rgba<u8> {
            image::Rgba([0, 0, 0, 255])
        }
        fn cells(&self) -> (u32, u32) {
            (6, 6)
        }
        fn canvas(&self) -> Option<crate::widgets::display::image::paint::CanvasPicture> {
            Some(crate::widgets::display::image::paint::CanvasPicture {
                shared_memory: false,
            })
        }
        fn refuse(&self, reason: &str) {
            self.refused.lock().unwrap().push(reason.to_owned());
        }
    }

    #[test]
    fn gfx_009_a_picture_thread_that_cannot_start_refuses_the_picture_and_is_tried_again() {
        use crate::backend::suprtui::graphics::Graphics;
        let refused = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let picture = |frame| Told {
            frame,
            refused: refused.clone(),
        };
        let mut graphics = Graphics::<Told>::default();
        graphics.make_pictures_apart();
        graphics.fail_start = true;
        let (_, first) = graphics
            .prepare(&[picture(0)], (1, 1), false)
            .expect("a frame")
            .expect("its graphics");
        graphics.acknowledge(vec![picture(0)]);
        let told = refused.lock().unwrap().clone();
        // The thread can start again: the next picture is made on it.
        graphics.fail_start = false;
        graphics
            .prepare(&[picture(1)], (1, 1), false)
            .expect("a frame");
        graphics.acknowledge(vec![picture(1)]);
        let started = Instant::now();
        let mut written = Vec::new();
        while written.is_empty() && started.elapsed() < Duration::from_secs(5) {
            written = graphics.take_made((1, 1)).0;
            std::thread::sleep(Duration::from_millis(2));
        }
        let made_on = graphics.take_made_on();
        assert!(
            told.len() == 1
                && told[0].contains("cannot be made ready")
                && !String::from_utf8_lossy(&first).contains("\x1bP")
                && written.len() == 1
                && made_on.iter().all(|thread| thread.starts_with("rtui-picture-")),
            "GFX-009: with the picture thread unable to start the canvas was told {told:?} and the frame sent {:?}; once it could start, {} pictures were written, made on {made_on:?}",
            String::from_utf8_lossy(&first),
            written.len()
        );
    }

    /// A canvas's Kitty picture sent through shared memory, which takes
    /// `slow` to copy out, the next picture of the one before it.
    #[cfg(unix)]
    #[derive(Clone, PartialEq)]
    struct Shared {
        frame: u32,
        slow: Duration,
    }
    #[cfg(unix)]
    impl RasterPlane for Shared {
        fn id(&self) -> u32 {
            11
        }
        fn protocol(&self) -> ImageProtocol {
            ImageProtocol::Kitty
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
                4,
                4,
                image::Rgba([7, 7, 7, 255]),
            ))
        }
        fn background(&self, _: u32, _: u32, _: (u16, u16)) -> image::Rgba<u8> {
            image::Rgba([0, 0, 0, 255])
        }
        fn cells(&self) -> (u32, u32) {
            (4, 4)
        }
        fn canvas(&self) -> Option<crate::widgets::display::image::paint::CanvasPicture> {
            Some(crate::widgets::display::image::paint::CanvasPicture {
                shared_memory: true,
            })
        }
        fn replaces(&self, _: &Self) -> bool {
            true
        }
    }

    /// The shared-memory object each Kitty picture in `bytes` names.
    #[cfg(unix)]
    fn objects(bytes: &[u8]) -> Vec<String> {
        use base64::Engine;
        String::from_utf8_lossy(bytes)
            .split("\x1b_G")
            .filter(|command| command.contains("t=s"))
            .filter_map(|command| {
                let name = command.split_once(';')?.1.split_once('\x1b')?.0;
                let name = base64::engine::general_purpose::STANDARD
                    .decode(name)
                    .ok()?;
                String::from_utf8(name).ok()
            })
            .collect()
    }

    #[cfg(unix)]
    #[test]
    fn gfx_005_a_picture_being_written_keeps_its_shared_memory_while_later_ones_are_made() {
        use crate::backend::suprtui::graphics::Graphics;
        let wait = |graphics: &Graphics<Shared>| {
            let started = Instant::now();
            while graphics.making() {
                assert!(
                    started.elapsed() < Duration::from_secs(5),
                    "the pictures were not made"
                );
                std::thread::sleep(Duration::from_millis(1));
            }
        };
        let mut graphics = Graphics::<Shared>::default();
        graphics.make_pictures_apart();
        let picture = |frame, slow| Shared { frame, slow };
        graphics
            .prepare(&[picture(0, Duration::ZERO)], (1, 1), false)
            .expect("a frame");
        graphics.acknowledge(vec![picture(0, Duration::ZERO)]);
        wait(&graphics);
        // The worker takes the picture and starts to write it; the terminal
        // reads its object only once the write is through.
        let (written, _) = graphics.take_made((1, 1));
        let names: Vec<String> = written.iter().flat_map(|bytes| objects(bytes)).collect();
        // Meanwhile a slow picture is made and a quick one waits behind it.
        graphics
            .prepare(&[picture(1, Duration::from_millis(100))], (1, 1), false)
            .expect("a frame");
        graphics.acknowledge(vec![picture(1, Duration::from_millis(100))]);
        let started = Instant::now();
        let maker = |graphics: &Graphics<Shared>| graphics.maker.as_ref().map_or(1, Maker::waiting);
        while !graphics.making() || maker(&graphics) > 0 {
            assert!(
                started.elapsed() < Duration::from_secs(5),
                "the thread took no picture"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        graphics
            .prepare(&[picture(2, Duration::ZERO)], (1, 1), false)
            .expect("a frame");
        graphics.acknowledge(vec![picture(2, Duration::ZERO)]);
        wait(&graphics);
        let there: Vec<bool> = names
            .iter()
            .map(|name| {
                rustix::shm::open(
                    name.as_str(),
                    rustix::shm::OFlags::RDONLY,
                    rustix::fs::Mode::empty(),
                )
                .is_ok()
            })
            .collect();
        // What the test made is not left behind.
        drop(graphics);
        assert!(
            names.len() == 1 && there == [true],
            "GFX-005: the picture being written named the objects {names:?}; after two more pictures were made, they were there: {there:?}"
        );
    }
}
