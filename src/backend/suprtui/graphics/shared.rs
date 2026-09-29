//! Canvas pictures sent to a Kitty terminal through POSIX shared memory
//! (GFX-005): the pixels go into a shared-memory object and the terminal is
//! sent its name, so a picture costs the terminal link a hundred bytes. The
//! terminal removes an object when it has read it. Those it has not read
//! when newer pictures of the same canvas follow are removed here: a
//! picture the terminal is too slow for is dropped, never queued. Each
//! canvas has its own list, so a frame of many canvases never removes a
//! picture it has just made for another of them.

/// The shared-memory objects this process made and may still have to
/// remove, by the canvas they show, oldest first.
#[derive(Default)]
pub(super) struct Pictures {
    #[cfg(unix)]
    names: std::collections::HashMap<u32, std::collections::VecDeque<String>>,
}

/// How many pictures of one canvas may wait for the terminal to read them:
/// the one of the frame being written and the one before it.
#[cfg(unix)]
const WAITING: usize = 2;

#[cfg(unix)]
impl Pictures {
    /// The Kitty command that shows `pixels` from shared memory as image
    /// `id` at the cursor; `None` when the object cannot be made, and the
    /// picture is then sent in the command itself.
    pub fn kitty(&mut self, pixels: &image::RgbaImage, id: u32, z: i32) -> Option<String> {
        use base64::Engine;
        use rustix::fs::Mode;
        use rustix::shm;
        use std::io::Write;
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let names = self.names.entry(id).or_default();
        while names.len() >= WAITING {
            if let Some(name) = names.pop_front() {
                let _ = shm::unlink(name.as_str());
            }
        }
        let name = format!(
            "/rtui-canvas-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        );
        // A leftover from an earlier process with this id is not ours to keep.
        let _ = shm::unlink(name.as_str());
        let flags = shm::OFlags::CREATE | shm::OFlags::EXCL | shm::OFlags::RDWR;
        let fd = shm::open(name.as_str(), flags, Mode::RUSR | Mode::WUSR).ok()?;
        let mut file = std::fs::File::from(fd);
        if file.write_all(pixels.as_raw()).is_err() {
            let _ = shm::unlink(name.as_str());
            return None;
        }
        names.push_back(name.clone());
        let (width, height) = pixels.dimensions();
        let name = base64::engine::general_purpose::STANDARD.encode(name.as_bytes());
        Some(format!(
            "\x1b_Ga=T,f=32,t=s,s={width},v={height},i={id},q=2,C=1,z={z};{name}\x1b\\"
        ))
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

#[cfg(unix)]
impl Drop for Pictures {
    fn drop(&mut self) {
        self.forget();
    }
}

#[cfg(not(unix))]
impl Pictures {
    /// Shared memory is sent on Unix only; elsewhere the picture is sent
    /// in the command itself.
    pub fn kitty(&mut self, _: &image::RgbaImage, _: u32, _: i32) -> Option<String> {
        None
    }

    pub fn forget(&self) {}

    pub fn keep(&mut self, _: &[u32]) {}
}
