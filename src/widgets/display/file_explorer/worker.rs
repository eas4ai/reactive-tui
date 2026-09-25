//! One cancellable filesystem producer per retained explorer.
use super::super::{FileEntry, FileType};
use crate::reactive::ThreadSafeSignal;
use cap_std::fs::Dir;
use std::{
    io::{self, Read},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Condvar, Mutex,
    },
    thread::{self, JoinHandle},
};

const ENTRY_LIMIT: usize = 100_000;
const PREVIEW_BYTES: u64 = 16 * 1024;
#[path = "worker/operations.rs"]
mod operations;
pub(super) use operations::Operation;

pub(super) enum Job {
    Read(PathBuf),
    Preview(PathBuf),
    Mutate {
        operation: Operation,
        sources: Vec<PathBuf>,
        destination: Option<PathBuf>,
    },
}

#[derive(Debug)]
pub(super) enum Output {
    Entries {
        root: PathBuf,
        path: PathBuf,
        entries: Vec<FileEntry>,
    },
    Preview {
        path: PathBuf,
        text: String,
    },
    Mutated(String),
}

#[cfg(test)]
#[path = "worker/tests.rs"]
mod tests;

pub(super) struct Request {
    pub id: u64,
    pub root: PathBuf,
    pub job: Job,
}

pub(super) struct Response {
    pub id: u64,
    pub result: Result<Output, String>,
}

#[derive(Default)]
struct Slots {
    request: Option<Request>,
    response: Option<Response>,
}

struct Shared {
    slots: Mutex<Slots>,
    ready: Condvar,
    closed: AtomicBool,
    generation: AtomicU64,
    cancelled: AtomicBool,
    changed: ThreadSafeSignal<u64>,
}

pub(super) struct Worker {
    shared: Arc<Shared>,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    pub fn new() -> io::Result<Self> {
        let shared = Arc::new(Shared {
            slots: Mutex::default(),
            ready: Condvar::new(),
            closed: AtomicBool::new(false),
            generation: AtomicU64::new(0),
            cancelled: AtomicBool::new(false),
            changed: ThreadSafeSignal::new(0),
        });
        let owner = shared.clone();
        let thread = thread::Builder::new()
            .name("file-explorer".into())
            .spawn(move || run(owner))?;
        Ok(Self {
            shared,
            thread: Some(thread),
        })
    }

    pub fn observe(&self) {
        self.shared.changed.get();
    }

    pub fn submit(&self, root: PathBuf, job: Job) -> u64 {
        let id = self.shared.generation.fetch_add(1, Ordering::AcqRel) + 1;
        self.shared.cancelled.store(false, Ordering::Release);
        let mut slots = self.shared.slots.lock().unwrap();
        slots.response = None;
        slots.request = Some(Request { id, root, job });
        drop(slots);
        self.shared.ready.notify_one();
        id
    }

    pub fn take(&self) -> Option<Response> {
        self.shared.slots.lock().unwrap().response.take()
    }

    pub fn cancel(&self) {
        self.shared.cancelled.store(true, Ordering::Release);
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.shared.closed.store(true, Ordering::Release);
        self.shared.slots.lock().unwrap().request = None;
        self.shared.ready.notify_one();
        if let Some(thread) = self.thread.take() {
            if thread.join().is_err() {
                log::error!("FileExplorer filesystem worker panicked during shutdown");
            }
        }
    }
}

fn run(shared: Arc<Shared>) {
    let mut root = None;
    loop {
        let request = {
            let mut slots = shared.slots.lock().unwrap();
            while slots.request.is_none() && !shared.closed.load(Ordering::Acquire) {
                slots = shared.ready.wait(slots).unwrap();
            }
            if shared.closed.load(Ordering::Acquire) {
                return;
            }
            slots.request.take().unwrap()
        };
        let stale = || {
            shared.closed.load(Ordering::Acquire)
                || shared.generation.load(Ordering::Acquire) != request.id
        };
        let cancelled = || stale() || shared.cancelled.load(Ordering::Acquire);
        let result =
            execute_cached(&request, &cancelled, &mut root).map_err(|error| error.to_string());
        if stale() {
            continue;
        }
        let mut slots = shared.slots.lock().unwrap();
        if stale() {
            continue;
        }
        slots.response = Some(Response {
            id: request.id,
            result,
        });
        drop(slots);
        shared.changed.set(request.id);
    }
}

fn check_cancel(cancelled: &impl Fn() -> bool) -> io::Result<()> {
    if cancelled() {
        Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "Filesystem operation cancelled",
        ))
    } else {
        Ok(())
    }
}

fn relative(root: &Path, canonical_root: &Path, path: &Path) -> io::Result<PathBuf> {
    if !path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    if let Ok(relative) = path
        .strip_prefix(canonical_root)
        .or_else(|_| path.strip_prefix(root))
    {
        return Ok(relative.to_path_buf());
    }
    #[cfg(windows)]
    {
        // Windows canonical paths use the verbatim prefix and expand 8.3 aliases.
        // Resolve the parent, not the leaf: copy/delete must keep a symlink's
        // identity and rename destinations need not exist. Subsequent access
        // still resolves through the retained directory capability.
        let normalized = match (path.parent(), path.file_name()) {
            (Some(parent), Some(name)) => std::fs::canonicalize(parent)?.join(name),
            _ => std::fs::canonicalize(path)?,
        };
        if let Ok(relative) = normalized.strip_prefix(canonical_root) {
            return Ok(relative.to_path_buf());
        }
    }
    Err(io::Error::new(
        io::ErrorKind::PermissionDenied,
        "Path is outside the explorer root",
    ))
}

struct Root {
    authored: PathBuf,
    canonical: PathBuf,
    directory: Dir,
}

#[cfg(test)]
fn execute(request: &Request, cancelled: &impl Fn() -> bool) -> io::Result<Output> {
    execute_cached(request, cancelled, &mut None)
}

fn execute_cached(
    request: &Request,
    cancelled: &impl Fn() -> bool,
    cache: &mut Option<Root>,
) -> io::Result<Output> {
    check_cancel(cancelled)?;
    if cache
        .as_ref()
        .is_none_or(|root| root.authored != request.root)
    {
        let canonical = std::fs::canonicalize(&request.root)?;
        let directory = Dir::open_ambient_dir(&canonical, cap_std::ambient_authority())?;
        *cache = Some(Root {
            authored: request.root.clone(),
            canonical,
            directory,
        });
    }
    let cached = cache.as_ref().unwrap();
    let canonical_root = &cached.canonical;
    let root = &cached.directory;
    if let Job::Mutate {
        operation,
        sources,
        destination,
    } = &request.job
    {
        let sources = sources
            .iter()
            .map(|path| relative(&request.root, canonical_root, path))
            .collect::<io::Result<Vec<_>>>()?;
        let destination = destination
            .as_ref()
            .map(|path| relative(&request.root, canonical_root, path))
            .transpose()?;
        return operations::execute(
            root,
            *operation,
            &sources,
            destination.as_deref(),
            cancelled,
        )
        .map(Output::Mutated);
    }
    let path = match &request.job {
        Job::Read(path) | Job::Preview(path) => path,
        Job::Mutate { .. } => unreachable!(),
    };
    let relative = relative(&request.root, canonical_root, path)?;
    let relative = if relative.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        relative
    };
    // Resolve through the directory capability, including symlinks. A string-prefix
    // check alone cannot confine filesystem access when another process changes paths.
    let resolved = root.canonicalize(&relative)?;
    let absolute = if resolved == Path::new(".") {
        canonical_root.clone()
    } else {
        canonical_root.join(&resolved)
    };
    check_cancel(cancelled)?;
    match &request.job {
        Job::Read(_) => {
            let directory = root.open_dir(&resolved)?;
            let mut entries = Vec::new();
            for entry in directory.entries()? {
                check_cancel(cancelled)?;
                let entry = entry?;
                if entries.len() == ENTRY_LIMIT {
                    return Err(io::Error::other(format!(
                        "Directory exceeds {ENTRY_LIMIT} entries"
                    )));
                }
                let name = entry.file_name();
                let metadata = directory.symlink_metadata(&name)?;
                let file_type = if metadata.file_type().is_symlink() {
                    FileType::Symlink
                } else if metadata.is_dir() {
                    FileType::Directory
                } else if metadata.is_file() {
                    FileType::File
                } else {
                    FileType::Unknown
                };
                let path = absolute.join(&name);
                let name = name.to_string_lossy().into_owned();
                let extension = path
                    .extension()
                    .map(|value| value.to_string_lossy().to_lowercase());
                let icon = FileEntry::get_icon_for_file(&name, &file_type, &extension);
                let hidden = name.starts_with('.');
                #[cfg(windows)]
                let hidden = {
                    use cap_std::fs::MetadataExt;
                    hidden
                        || metadata.file_attributes()
                            & windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_HIDDEN
                            != 0
                };
                entries.push(FileEntry {
                    hidden,
                    name,
                    path,
                    file_type,
                    extension,
                    icon,
                    size: metadata.is_file().then_some(metadata.len()),
                    modified: metadata.modified().ok().map(|value| value.into_std()),
                    selected: false,
                    focused: false,
                });
            }
            Ok(Output::Entries {
                root: canonical_root.clone(),
                path: absolute,
                entries,
            })
        }
        Job::Preview(_) => {
            let metadata = root.metadata(&resolved)?;
            if !metadata.is_file() {
                return Ok(Output::Preview {
                    path: path.clone(),
                    text: "No file preview".into(),
                });
            }
            use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt, OpenOptionsSyncExt};
            let mut options = cap_std::fs::OpenOptions::new();
            options.read(true).follow(FollowSymlinks::No).nonblock(true);
            let file = root.open_with(&resolved, &options)?;
            if !file.metadata()?.is_file() {
                return Err(io::Error::other(
                    "Preview source is no longer a regular file",
                ));
            }
            let mut bytes = Vec::new();
            file.take(PREVIEW_BYTES + 1).read_to_end(&mut bytes)?;
            check_cancel(cancelled)?;
            let truncated = bytes.len() as u64 > PREVIEW_BYTES;
            bytes.truncate(PREVIEW_BYTES as usize);
            if truncated {
                if let Err(error) = std::str::from_utf8(&bytes) {
                    if error.error_len().is_none() {
                        bytes.truncate(error.valid_up_to());
                    }
                }
            }
            let text = if bytes.contains(&0) {
                format!("Binary file · {} bytes", metadata.len())
            } else {
                match String::from_utf8(bytes) {
                    Ok(mut text) => {
                        if truncated {
                            text.push_str("\n[Preview truncated]");
                        }
                        text
                    }
                    Err(_) => format!("Binary or non-UTF8 file · {} bytes", metadata.len()),
                }
            };
            Ok(Output::Preview {
                path: path.clone(),
                text,
            })
        }
        Job::Mutate { .. } => unreachable!(),
    }
}
