//! Own a verified Microsoft ConPTY module and its runtime files for one session.

use super::{error, TerminalResult};
use libloading::os::windows::Library;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::Read,
    os::windows::fs::OpenOptionsExt,
    path::{Path, PathBuf},
};
use windows_sys::Win32::{
    Foundation::HANDLE,
    Storage::FileSystem::FILE_SHARE_READ,
    System::Console::{COORD, HPCON},
};

type Create = unsafe extern "system" fn(COORD, HANDLE, HANDLE, u32, *mut HPCON) -> i32;
type Resize = unsafe extern "system" fn(HPCON, COORD) -> i32;
type Close = unsafe extern "system" fn(HPCON);

#[derive(serde::Deserialize)]
struct Manifest {
    version: String,
    files: BTreeMap<String, String>,
}

pub(super) struct Runtime {
    // Field order matters: unload the DLL before releasing the file locks.
    _library: Library,
    _files: Vec<File>,
    pub(super) create: Create,
    pub(super) resize: Resize,
    pub(super) close: Close,
}

impl Runtime {
    pub(super) fn load() -> TerminalResult<Self> {
        let directory = runtime_directory()?;
        let manifest: Manifest = serde_json::from_str(include_str!("runtime/manifest.json"))
            .map_err(|failure| error(format!("Invalid compiled ConPTY manifest: {failure}")))?;
        let architecture = match std::env::consts::ARCH {
            "x86_64" => "x64",
            "x86" => "x86",
            "aarch64" => "arm64",
            other => return Err(error(format!("ConPTY runtime has no shim for {other}"))),
        };
        let mut files = Vec::with_capacity(4);
        let dll = directory.join("conpty.dll");
        files.push(verify(
            &dll,
            &manifest,
            &format!("{architecture}/conpty.dll"),
        )?);
        // The SDK chooses the host matching the native OS architecture. Keep
        // every shipped host pinned, including when the application is emulated.
        for host in ["x86", "x64", "arm64"] {
            let name = format!("{host}/OpenConsole.exe");
            files.push(verify(&directory.join(&name), &manifest, &name)?);
        }
        // SAFETY: the DLL and hosts match the compiled package digests. Open
        // read handles deny modification/deletion until after module unload.
        // 0x1100 is LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_SYSTEM32;
        // the working directory and PATH cannot supply another dependency.
        let library = unsafe { Library::load_with_flags(&dll, 0x1100) }
            .map_err(|failure| error(format!("Load ConPTY {}: {failure}", dll.display())))?;
        // SAFETY: signatures match the pinned package's inc/conpty.h. Function
        // pointers remain private and the library outlives all calls from Session.
        let (create, resize, close) = unsafe {
            (
                *library
                    .get::<Create>(b"ConptyCreatePseudoConsole\0")
                    .map_err(error)?,
                *library
                    .get::<Resize>(b"ConptyResizePseudoConsole\0")
                    .map_err(error)?,
                *library
                    .get::<Close>(b"ConptyClosePseudoConsole\0")
                    .map_err(error)?,
            )
        };
        Ok(Self {
            _library: library,
            _files: files,
            create,
            resize,
            close,
        })
    }
}

fn runtime_directory() -> TerminalResult<PathBuf> {
    if let Some(directory) = std::env::var_os("REACTIVE_TUI_CONPTY_DIR") {
        let directory = PathBuf::from(directory);
        if !directory.is_absolute() {
            return Err(error(
                "REACTIVE_TUI_CONPTY_DIR must be an absolute directory",
            ));
        }
        return Ok(directory);
    }
    let executable = std::env::current_exe().map_err(error)?;
    let directory = executable
        .parent()
        .ok_or_else(|| error("Application directory is unavailable"))?;
    Ok(directory.join("reactive-tui-conpty"))
}

fn verify(path: &Path, manifest: &Manifest, name: &str) -> TerminalResult<File> {
    let expected = manifest
        .files
        .get(name)
        .ok_or_else(|| error(format!("Compiled ConPTY manifest omits {name}")))?;
    let context = |failure: &dyn std::fmt::Display| {
        error(format!(
        "ConPTY runtime {} at {}: {failure}. Install the matched runtime with scripts/install-conpty-runtime.py; see manual/terminal-and-embedded-sessions.md",
        manifest.version, path.display(),
    ))
    };
    let mut file = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .open(path)
        .map_err(|failure| context(&failure))?;
    if file.metadata().map_err(|failure| context(&failure))?.len() > 2 * 1024 * 1024 {
        return Err(context(&"runtime file exceeds its size bound"));
    }
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 16384];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|failure| context(&failure))?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    if format!("{:x}", digest.finalize()) != *expected {
        return Err(context(
            &"runtime file SHA-256 does not match the pinned Microsoft package",
        ));
    }
    Ok(file)
}
