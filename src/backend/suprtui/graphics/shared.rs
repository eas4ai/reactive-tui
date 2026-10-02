//! Canvas pictures sent to a Kitty terminal through POSIX shared memory
//! (GFX-005): the pixels go into a shared-memory object and the terminal is
//! sent its name, so a picture costs the terminal link a hundred bytes. The
//! terminal removes an object when it has read it. Those it has not read
//! when newer pictures of the same canvas follow are removed here: a
//! picture the terminal is too slow for is dropped, never queued. Each
//! canvas has its own list, so a frame of many canvases never removes a
//! picture it has just made for another of them.

/// The shared-memory objects of the pictures written, which this process
/// may still have to remove, by the canvas they show, oldest first. The
/// object of a picture made ready apart is the maker's until the worker
/// writes the picture or drops it (GFX-009).
#[derive(Default)]
pub(super) struct Pictures {
    #[cfg(unix)]
    names: std::collections::HashMap<u32, std::collections::VecDeque<String>>,
}

/// How many pictures of one canvas may wait for the terminal to read them:
/// the one of the frame being written and the one before it.
#[cfg(unix)]
const WAITING: usize = 2;

/// A new shared-memory object holding `pixels`, and the Kitty command that
/// shows it as image `id` at the cursor; `None` when the object cannot be
/// made, and the picture is then sent in the command itself. The caller
/// owns the object: [`Pictures::written`] takes it once its picture is
/// written, and [`unlink`] removes it when it is not.
#[cfg(unix)]
pub fn make(pixels: &image::RgbaImage, id: u32, z: i32) -> Option<(String, String)> {
    use base64::Engine;
    use rustix::fs::Mode;
    use rustix::shm;
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let name = format!(
        "/rtui-canvas-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    );
    // A leftover from an earlier process with this id is not ours to keep.
    let _ = shm::unlink(name.as_str());
    let flags = shm::OFlags::CREATE | shm::OFlags::EXCL | shm::OFlags::RDWR;
    let object = shm::open(name.as_str(), flags, Mode::RUSR | Mode::WUSR).ok()?;
    if fill(object, pixels.as_raw()).is_none() {
        let _ = shm::unlink(name.as_str());
        return None;
    }
    let (width, height) = pixels.dimensions();
    let encoded = base64::engine::general_purpose::STANDARD.encode(name.as_bytes());
    Some((
        format!("\x1b_Ga=T,f=32,t=s,s={width},v={height},i={id},q=2,C=1,z={z};{encoded}\x1b\\"),
        name,
    ))
}

/// Remove the shared-memory object `name`, whose picture is not written.
#[cfg(unix)]
pub fn unlink(name: &str) {
    let _ = rustix::shm::unlink(name);
}

#[cfg(unix)]
impl Pictures {
    /// The Kitty command that shows `pixels` from shared memory as image
    /// `id` at the cursor, its picture written now; `None` when the object
    /// cannot be made, and the picture is then sent in the command itself.
    pub fn kitty(&mut self, pixels: &image::RgbaImage, id: u32, z: i32) -> Option<String> {
        let (command, name) = make(pixels, id, z)?;
        self.written(id, name);
        Some(command)
    }

    /// The picture of canvas `id` whose object is `name` is being written.
    /// The terminal may still read the one before it; older objects of the
    /// canvas are removed.
    pub fn written(&mut self, id: u32, name: String) {
        let names = self.names.entry(id).or_default();
        while names.len() >= WAITING {
            if let Some(old) = names.pop_front() {
                unlink(&old);
            }
        }
        names.push_back(name);
    }

    /// Remove every object the terminal has not read.
    pub fn forget(&self) {
        for name in self.names.values().flatten() {
            let _ = rustix::shm::unlink(name.as_str());
        }
    }

    /// Remove the objects of every canvas but those `shown`: a canvas that
    /// is gone gets no newer picture to remove its last ones.
    pub fn keep(&mut self, shown: &[u32]) {
        self.names.retain(|id, names| {
            let kept = shown.contains(id);
            if !kept {
                for name in names.iter() {
                    let _ = rustix::shm::unlink(name.as_str());
                }
            }
            kept
        });
    }
}

/// Put `bytes` into the new shared-memory object. Linux keeps the object
/// in a file system that takes `write` and answers a full one with an
/// error; a mapping of it would end the process with SIGBUS instead.
#[cfg(any(target_os = "linux", target_os = "android"))]
fn fill(object: rustix::fd::OwnedFd, bytes: &[u8]) -> Option<()> {
    use std::io::Write;
    std::fs::File::from(object).write_all(bytes).ok()
}

/// Put `bytes` into the new shared-memory object. macOS takes no `write`
/// on shared memory, so the object is given its size and mapped.
#[cfg(all(unix, not(any(target_os = "linux", target_os = "android"))))]
fn fill(object: rustix::fd::OwnedFd, bytes: &[u8]) -> Option<()> {
    use rustix::mm::{mmap, munmap, MapFlags, ProtFlags};
    if bytes.is_empty() {
        return None;
    }
    rustix::fs::ftruncate(&object, bytes.len() as u64).ok()?;
    // SAFETY: the mapping is new, of as many bytes as the object was just
    // given, and nothing else knows its address. `bytes` is another
    // allocation of that length, so the two do not overlap. The mapping is
    // removed before returning.
    unsafe {
        let start = mmap(
            std::ptr::null_mut(),
            bytes.len(),
            ProtFlags::READ | ProtFlags::WRITE,
            MapFlags::SHARED,
            &object,
            0,
        )
        .ok()?;
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), start.cast::<u8>(), bytes.len());
        munmap(start, bytes.len()).ok()
    }
}

#[cfg(unix)]
impl Drop for Pictures {
    fn drop(&mut self) {
        self.forget();
    }
}

/// Shared memory is sent on Unix only; elsewhere the picture is sent in the
/// command itself.
#[cfg(not(unix))]
pub fn make(_: &image::RgbaImage, _: u32, _: i32) -> Option<(String, String)> {
    None
}

#[cfg(not(unix))]
pub fn unlink(_: &str) {}

#[cfg(not(unix))]
impl Pictures {
    pub fn kitty(&mut self, _: &image::RgbaImage, _: u32, _: i32) -> Option<String> {
        None
    }

    pub fn written(&mut self, _: u32, _: String) {}

    pub fn forget(&self) {}

    pub fn keep(&mut self, _: &[u32]) {}
}
