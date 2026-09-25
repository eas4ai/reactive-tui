use cap_std::fs::Dir;
use std::{ffi::OsStr, io};

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_vendor = "apple",
    target_os = "redox"
))]
pub(super) fn exclusive(
    source: &Dir,
    name: &OsStr,
    destination: &Dir,
    target: &OsStr,
) -> io::Result<()> {
    rustix::fs::renameat_with(
        source,
        name,
        destination,
        target,
        rustix::fs::RenameFlags::NOREPLACE,
    )
    .map_err(Into::into)
}

#[cfg(windows)]
pub(super) fn exclusive(
    source: &Dir,
    name: &OsStr,
    destination: &Dir,
    target: &OsStr,
) -> io::Result<()> {
    use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt};
    use cap_std::fs::{OpenOptions, OpenOptionsExt};
    use std::{
        mem::{offset_of, size_of, zeroed},
        os::windows::{ffi::OsStrExt, io::AsRawHandle},
    };
    use windows_sys::{
        Wdk::Storage::FileSystem::{
            FileRenameInformation, NtSetInformationFile, FILE_RENAME_INFORMATION,
        },
        Win32::{
            Foundation::RtlNtStatusToDosError,
            Storage::FileSystem::{
                DELETE, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
            },
            System::IO::IO_STATUS_BLOCK,
        },
    };

    let mut options = OpenOptions::new();
    options
        .access_mode(DELETE)
        .follow(FollowSymlinks::No)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT);
    let file = source.open_with(name, &options).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("Open rename source {name:?}: {error}"),
        )
    })?;
    let name: Vec<u16> = target.encode_wide().collect();
    let offset = offset_of!(FILE_RENAME_INFORMATION, FileName);
    let bytes = size_of::<FILE_RENAME_INFORMATION>()
        .checked_add(
            name.len()
                .checked_mul(2)
                .ok_or_else(|| io::Error::other("Rename path too long"))?,
        )
        .ok_or_else(|| io::Error::other("Rename path too long"))?;
    let length = u32::try_from(bytes).map_err(|_| io::Error::other("Rename path too long"))?;
    // usize storage provides pointer alignment for FILE_RENAME_INFORMATION and enough
    // trailing space for its variable-length UTF-16 filename. Both handles and
    // the buffer remain alive through the synchronous Windows call.
    let mut storage = vec![0usize; bytes.div_ceil(size_of::<usize>())];
    let info = storage.as_mut_ptr().cast::<FILE_RENAME_INFORMATION>();
    // SAFETY: both directory handles and the non-overlapped source handle remain
    // alive. The aligned zeroed allocation fits the complete structure and name.
    // cap-std opens the source synchronously, so the status block cannot outlive
    // this call. Only the already validated leaf name is resolved at destination.
    unsafe {
        (*info).RootDirectory = destination.as_raw_handle();
        (*info).FileNameLength = (name.len() * 2) as u32;
        // The zeroed union leaves ReplaceIfExists false.
        std::ptr::copy_nonoverlapping(
            name.as_ptr(),
            storage.as_mut_ptr().cast::<u8>().add(offset).cast::<u16>(),
            name.len(),
        );
        let mut status_block: IO_STATUS_BLOCK = zeroed();
        let status = NtSetInformationFile(
            file.as_raw_handle(),
            &mut status_block,
            info.cast(),
            length,
            FileRenameInformation,
        );
        if status < 0 {
            let error = io::Error::from_raw_os_error(RtlNtStatusToDosError(status) as i32);
            return Err(io::Error::new(
                error.kind(),
                format!("Rename to {target:?} with {length}-byte FILE_RENAME_INFORMATION (NTSTATUS {status:#x}): {error}"),
            ));
        }
    }
    Ok(())
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn exclusive_rename_handles_short_names_and_never_replaces_files_or_directories() {
        for directory in [false, true] {
            for target in ["a", "ab", "abc", "copy", "longer-destination"] {
                let fixture = tempfile::tempdir().unwrap();
                let root =
                    Dir::open_ambient_dir(fixture.path(), cap_std::ambient_authority()).unwrap();
                if directory {
                    root.create_dir("source").unwrap();
                    root.write("source/child", b"keep").unwrap();
                } else {
                    root.write("source", b"keep").unwrap();
                }
                exclusive(&root, OsStr::new("source"), &root, OsStr::new(target)).unwrap_or_else(
                    |error| panic!("directory={directory}, target={target}: {error}"),
                );
                assert!(!root.exists("source"));
                let contents = if directory {
                    format!("{target}/child")
                } else {
                    target.into()
                };
                assert_eq!(root.read(contents).unwrap(), b"keep");
                root.write("existing", b"untouched").unwrap();
                assert!(
                    exclusive(&root, OsStr::new(target), &root, OsStr::new("existing")).is_err()
                );
                assert_eq!(root.read("existing").unwrap(), b"untouched");
                assert!(root.exists(target));
            }
        }
    }
}

#[cfg(not(any(
    windows,
    target_os = "linux",
    target_os = "android",
    target_vendor = "apple",
    target_os = "redox"
)))]
pub(super) fn exclusive(
    _source: &Dir,
    _name: &OsStr,
    _destination: &Dir,
    _target: &OsStr,
) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Atomic rename without replacement is unavailable on this platform",
    ))
}
