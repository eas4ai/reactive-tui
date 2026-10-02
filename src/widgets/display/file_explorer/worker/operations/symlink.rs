//! Preserve a Windows symbolic link without resolving its target or destination.
use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt};
use cap_std::fs::{Dir, OpenOptions, OpenOptionsExt};
use std::{ffi::OsStr, io, os::windows::io::AsRawHandle};
use windows_sys::Win32::{
    Storage::FileSystem::{FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT},
    System::{
        Ioctl::{FSCTL_GET_REPARSE_POINT, FSCTL_SET_REPARSE_POINT},
        SystemServices::IO_REPARSE_TAG_SYMLINK,
        IO::DeviceIoControl,
    },
};

pub(super) fn copy(
    source: &Dir,
    name: &OsStr,
    destination: &Dir,
    target: &OsStr,
    directory: bool,
) -> io::Result<()> {
    let mut options = OpenOptions::new();
    options
        .access_mode(0)
        .follow(FollowSymlinks::No)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT);
    let source = source.open_with(name, &options)?;
    // Windows limits a reparse payload to 16 KiB. u64 storage supplies alignment;
    // only the initialized bytes reported by the kernel are used below.
    let mut storage = vec![0u64; 2048];
    let mut length = 0;
    // SAFETY: the live source is opened without following the link; the output
    // allocation and byte-count pointer remain valid for this synchronous call.
    if unsafe {
        DeviceIoControl(
            source.as_raw_handle(),
            FSCTL_GET_REPARSE_POINT,
            std::ptr::null(),
            0,
            storage.as_mut_ptr().cast(),
            16384,
            &mut length,
            std::ptr::null_mut(),
        )
    } == 0
    {
        return Err(context(
            "Read symbolic-link contents",
            io::Error::last_os_error(),
        ));
    }
    if !(20..=16384).contains(&length) {
        return Err(io::Error::other(
            "Invalid symbolic-link reparse payload length",
        ));
    }
    // SAFETY: length is within the allocation initialized above.
    let bytes =
        unsafe { std::slice::from_raw_parts(storage.as_ptr().cast::<u8>(), length as usize) };
    let tag = u32::from_le_bytes(bytes[..4].try_into().unwrap());
    let payload = u16::from_le_bytes(bytes[4..6].try_into().unwrap()) as usize;
    if tag != IO_REPARSE_TAG_SYMLINK || payload + 8 != bytes.len() {
        return Err(io::Error::other(
            "Source is not a valid Windows symbolic link",
        ));
    }
    let mut options = OpenOptions::new();
    options
        .write(true)
        .follow(FollowSymlinks::No)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT);
    if directory {
        destination.create_dir(target)?;
    } else {
        options.create_new(true);
    }
    let target = destination.open_with(target, &options)?;
    let mut returned = 0;
    // SAFETY: target is a newly created entry in the owned staging directory.
    // Both handles, the validated kernel payload and out pointer stay live until
    // this non-overlapped operation completes. No link target is dereferenced.
    if unsafe {
        DeviceIoControl(
            target.as_raw_handle(),
            FSCTL_SET_REPARSE_POINT,
            storage.as_ptr().cast(),
            length,
            std::ptr::null_mut(),
            0,
            &mut returned,
            std::ptr::null_mut(),
        )
    } == 0
    {
        return Err(context(
            "Write symbolic-link contents",
            io::Error::last_os_error(),
        ));
    }
    Ok(())
}

fn context(action: &str, error: io::Error) -> io::Error {
    io::Error::new(error.kind(), format!("{action}: {error}"))
}
