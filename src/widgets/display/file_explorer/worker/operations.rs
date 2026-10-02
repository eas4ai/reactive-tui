use super::{check_cancel, Dir, Path, PathBuf, ENTRY_LIMIT};
use cap_std::fs::{DirBuilder, Metadata, OpenOptions};
use std::{
    ffi::OsStr,
    io::{self, Read, Write},
};

#[path = "operations/rename.rs"]
mod rename;
#[cfg(windows)]
#[path = "operations/symlink.rs"]
mod symlink;

#[cfg(test)]
std::thread_local! {
    static IDENTITY_RACE_HOOK: std::cell::RefCell<Option<Box<dyn FnOnce()>>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
pub(super) fn set_identity_race_hook(hook: impl FnOnce() + 'static) {
    IDENTITY_RACE_HOOK.with(|slot| {
        assert!(slot.borrow_mut().replace(Box::new(hook)).is_none());
    });
}

#[cfg(test)]
fn run_identity_race_hook() {
    let hook = IDENTITY_RACE_HOOK.with(|slot| slot.borrow_mut().take());
    if let Some(hook) = hook {
        hook();
    }
}

#[cfg(not(test))]
fn run_identity_race_hook() {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in super::super) enum Operation {
    Copy,
    Move,
    Rename,
    Delete,
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

fn ensure_same_entry(expected: &Metadata, actual: &Metadata) -> io::Result<()> {
    use cap_fs_ext::MetadataExt;
    if expected.dev() == actual.dev() && expected.ino() == actual.ino() {
        Ok(())
    } else {
        Err(io::Error::other("Source entry changed during operation"))
    }
}

/// Return an anchored parent and a single leaf name. Neither rename endpoint may
/// contain a path traversal that bypasses the directory capability.
fn parent(root: &Dir, path: &Path) -> io::Result<(Dir, std::ffi::OsString)> {
    let name = path
        .file_name()
        .ok_or_else(|| invalid("The explorer root cannot be changed"))?;
    if name.as_encoded_bytes().contains(&0) {
        return Err(invalid("File names cannot contain NUL"));
    }
    #[cfg(windows)]
    if name.to_string_lossy().contains(':') {
        return Err(invalid(
            "File operations do not accept alternate data stream names",
        ));
    }
    let directory = path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    Ok((root.open_dir(directory)?, name.to_os_string()))
}

pub(super) fn execute(
    root: &Dir,
    operation: Operation,
    sources: &[PathBuf],
    destination: Option<&Path>,
    cancelled: &impl Fn() -> bool,
) -> io::Result<String> {
    if sources.is_empty() {
        return Err(invalid("Select a file first"));
    }
    if operation == Operation::Rename && sources.len() != 1 {
        return Err(invalid("Rename requires one selected entry"));
    }
    // Avoid deleting or moving a nested selection twice.
    let mut sources = sources.to_vec();
    sources.sort();
    sources.dedup();
    let mut unique = Vec::new();
    for path in sources {
        if !unique
            .iter()
            .any(|parent: &PathBuf| path.starts_with(parent))
        {
            unique.push(path);
        }
    }
    let mut completed = 0;
    for source in &unique {
        let result = (|| {
            check_cancel(cancelled)?;
            let (source_parent, source_name) = parent(root, source)?;
            match operation {
                Operation::Delete => {
                    let mut count = 0;
                    remove(&source_parent, &source_name, cancelled, &mut count, 0)
                }
                Operation::Copy | Operation::Move | Operation::Rename => {
                    let destination =
                        destination.ok_or_else(|| invalid("Enter a destination path"))?;
                    let destination =
                        if operation != Operation::Rename && root.open_dir(destination).is_ok() {
                            destination.join(&source_name)
                        } else {
                            if unique.len() > 1 {
                                return Err(invalid(
                                    "Multiple entries require an existing destination directory",
                                ));
                            }
                            destination.to_path_buf()
                        };
                    let destination_parent_path = destination
                        .parent()
                        .filter(|path| !path.as_os_str().is_empty())
                        .unwrap_or(Path::new("."));
                    let destination_parent_path = root.canonicalize(destination_parent_path)?;
                    if let Ok(source_path) = root.canonicalize(source) {
                        if destination_parent_path.starts_with(&source_path) {
                            return Err(invalid("Destination is inside the source directory"));
                        }
                    }
                    let (destination_parent, destination_name) = parent(root, &destination)?;
                    if operation == Operation::Copy {
                        copy(
                            &source_parent,
                            &source_name,
                            &destination_parent,
                            &destination_name,
                            cancelled,
                        )
                    } else {
                        check_cancel(cancelled)?;
                        rename::exclusive(
                            &source_parent,
                            &source_name,
                            &destination_parent,
                            &destination_name,
                        )
                    }
                }
            }
        })();
        if let Err(error) = result {
            return Err(io::Error::new(
                error.kind(),
                format!(
                    "{operation:?}: {completed} completed; {}: {error}. Earlier changes remain.",
                    source.display()
                ),
            ));
        }
        completed += 1;
    }
    Ok(format!("{operation:?}: {completed} completed"))
}

fn copy(
    source: &Dir,
    name: &OsStr,
    destination: &Dir,
    target: &OsStr,
    cancelled: &impl Fn() -> bool,
) -> io::Result<()> {
    match destination.symlink_metadata(target) {
        Ok(_) => {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Destination already exists",
            ))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => (),
        Err(error) => return Err(error),
    }
    let staging_name = format!(".reactive-tui-copy-{}", uuid::Uuid::new_v4());
    let options = DirBuilder::new();
    #[cfg(unix)]
    let options = {
        use cap_std::fs::DirBuilderExt;
        let mut options = options;
        options.mode(0o700);
        options
    };
    destination.create_dir_with(&staging_name, &options)?;
    let staging = match destination.open_dir(&staging_name) {
        Ok(directory) => directory,
        Err(error) => {
            return match destination.remove_dir(&staging_name) {
                Ok(()) => Err(error),
                Err(cleanup) => Err(io::Error::other(format!(
                    "{error}; staging cleanup failed: {cleanup}"
                ))),
            };
        }
    };
    let result = (|| {
        let mut count = 0;
        copy_entry(
            source,
            name,
            &staging,
            OsStr::new("item"),
            cancelled,
            &mut count,
            0,
        )?;
        check_cancel(cancelled)?;
        rename::exclusive(&staging, OsStr::new("item"), destination, target)
    })();
    // The private staging directory never replaces the user's destination. A
    // failed copy leaves the source intact and removes its partial staged data.
    let cleanup = prepare_cleanup(&staging, 0).and_then(|()| staging.remove_open_dir_all());
    match (result, cleanup) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) => Err(error),
        (result, Err(cleanup)) => Err(io::Error::other(format!(
            "{}; could not remove staging directory {staging_name}: {cleanup}",
            result
                .err()
                .map_or_else(|| "Copy published".into(), |error| error.to_string())
        ))),
    }
}

// Only private copies pass through this cleanup path. A normal Delete must not
// change source permissions to circumvent the user's filesystem protections.
fn prepare_cleanup(directory: &Dir, depth: usize) -> io::Result<()> {
    if depth > 128 {
        return Err(invalid(
            "Staging directory nesting exceeds the cleanup limit",
        ));
    }
    for entry in directory.entries()? {
        let entry = entry?;
        let name = entry.file_name();
        let metadata = directory.symlink_metadata(&name)?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            #[cfg(unix)]
            {
                use cap_std::fs::PermissionsExt;
                let permissions =
                    cap_std::fs::Permissions::from_mode(metadata.permissions().mode() | 0o700);
                directory.set_permissions(&name, permissions)?;
            }
            #[cfg(windows)]
            {
                let mut permissions = metadata.permissions();
                permissions.set_readonly(false);
                directory.set_permissions(&name, permissions)?;
            }
            prepare_cleanup(&directory.open_dir(name)?, depth + 1)?;
        } else {
            #[cfg(windows)]
            {
                let mut permissions = metadata.permissions();
                permissions.set_readonly(false);
                directory.set_permissions(&name, permissions)?;
            }
        }
    }
    Ok(())
}

fn visit(count: &mut usize, depth: usize, cancelled: &impl Fn() -> bool) -> io::Result<()> {
    check_cancel(cancelled)?;
    *count += 1;
    if *count > ENTRY_LIMIT || depth > 128 {
        return Err(invalid(
            "Operation exceeds 100000 entries or 128 directory levels",
        ));
    }
    Ok(())
}

fn copy_entry(
    source: &Dir,
    name: &OsStr,
    destination: &Dir,
    target: &OsStr,
    cancelled: &impl Fn() -> bool,
    count: &mut usize,
    depth: usize,
) -> io::Result<()> {
    visit(count, depth, cancelled)?;
    let metadata = source.symlink_metadata(name)?;
    run_identity_race_hook();
    if metadata.file_type().is_symlink() {
        #[cfg(unix)]
        {
            let link = source.read_link_contents(name)?;
            ensure_same_entry(&metadata, &source.symlink_metadata(name)?)?;
            destination.symlink_contents(link, target)?;
        }
        #[cfg(windows)]
        {
            use cap_std::fs::MetadataExt;
            symlink::copy(
                source,
                name,
                destination,
                target,
                metadata.file_attributes()
                    & windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_DIRECTORY
                    != 0,
            )?;
            ensure_same_entry(&metadata, &source.symlink_metadata(name)?)?;
        }
        return Ok(());
    }
    if metadata.is_dir() {
        let source = source.open_dir(name)?;
        let opened_metadata = source.dir_metadata()?;
        ensure_same_entry(&metadata, &opened_metadata)?;
        destination.create_dir(target)?;
        let destination = destination.open_dir(target)?;
        for entry in source.entries()? {
            let name = entry?.file_name();
            copy_entry(
                &source,
                &name,
                &destination,
                &name,
                cancelled,
                count,
                depth + 1,
            )?;
        }
        destination
            .set_permissions(".", opened_metadata.permissions())
            .map_err(|error| {
                io::Error::new(
                    error.kind(),
                    format!("Set copied directory permissions: {error}"),
                )
            })?;
        return Ok(());
    }
    if !metadata.is_file() {
        return Err(invalid(
            "Only regular files, directories and symbolic links can be copied",
        ));
    }
    use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt, OpenOptionsSyncExt};
    let mut options = OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No).nonblock(true);
    let mut input = source.open_with(name, &options)?;
    let opened_metadata = input.metadata()?;
    ensure_same_entry(&metadata, &opened_metadata)?;
    if !opened_metadata.is_file() {
        return Err(invalid("Source is no longer a regular file"));
    }
    let mut output =
        destination.open_with(target, OpenOptions::new().write(true).create_new(true))?;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        check_cancel(cancelled)?;
        let size = input.read(&mut buffer)?;
        if size == 0 {
            break;
        }
        output.write_all(&buffer[..size])?;
    }
    output
        .set_permissions(opened_metadata.permissions())
        .map_err(|error| {
            io::Error::new(
                error.kind(),
                format!("Set copied file permissions: {error}"),
            )
        })?;
    output
        .sync_all()
        .map_err(|error| io::Error::new(error.kind(), format!("Sync copied file: {error}")))
}

fn remove(
    parent: &Dir,
    name: &OsStr,
    cancelled: &impl Fn() -> bool,
    count: &mut usize,
    depth: usize,
) -> io::Result<()> {
    visit(count, depth, cancelled)?;
    let metadata = parent.symlink_metadata(name)?;
    run_identity_race_hook();
    #[cfg(windows)]
    {
        use cap_std::fs::MetadataExt;
        if metadata.file_type().is_symlink()
            && metadata.file_attributes()
                & windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_DIRECTORY
                != 0
        {
            return parent.remove_dir(name);
        }
    }
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        let directory = parent.open_dir(name)?;
        ensure_same_entry(&metadata, &directory.dir_metadata()?)?;
        for entry in directory.entries()? {
            remove(&directory, &entry?.file_name(), cancelled, count, depth + 1)?;
        }
        check_cancel(cancelled)?;
        directory.remove_open_dir()
    } else {
        if metadata.is_file() {
            use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt, OpenOptionsSyncExt};
            let mut options = OpenOptions::new();
            options.read(true).follow(FollowSymlinks::No).nonblock(true);
            let source = parent.open_with(name, &options)?;
            ensure_same_entry(&metadata, &source.metadata()?)?;
        }
        ensure_same_entry(&metadata, &parent.symlink_metadata(name)?)?;
        parent.remove_file(name)
    }
}
