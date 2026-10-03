use super::*;
use std::time::{Duration, Instant};

const XIS_003_REPLACEMENT_ROUNDS: usize = 32;

fn request(root: &Path, job: Job) -> Request {
    Request {
        id: 1,
        root: root.to_path_buf(),
        job,
    }
}
fn operation(
    root: &Path,
    operation: Operation,
    source: &str,
    destination: Option<&str>,
) -> Request {
    request(
        root,
        Job::Mutate {
            operation,
            sources: vec![PathBuf::from(source)],
            destination: destination.map(PathBuf::from),
        },
    )
}

fn xis_003_copy_outcome_is_safe<T>(result: &io::Result<T>, copied: Option<&[u8]>) -> bool {
    result.is_err() || copied == Some(b"original")
}

fn xis_003_remove_outcome_is_safe<T>(
    result: &io::Result<T>,
    original_exists: bool,
    replacement_exists: bool,
) -> bool {
    result.is_err() || (!original_exists && replacement_exists)
}

#[test]
fn xis_003_validator_rejects_replaced_entry_outcomes() {
    assert!(!xis_003_copy_outcome_is_safe(
        &Ok::<(), io::Error>(()),
        Some(b"replacement")
    ));
    assert!(!xis_003_remove_outcome_is_safe(
        &Ok::<(), io::Error>(()),
        true,
        false
    ));
    assert!(xis_003_copy_outcome_is_safe(
        &Err::<(), _>(io::Error::other("entry changed")),
        None
    ));
    assert!(xis_003_remove_outcome_is_safe(
        &Err::<(), _>(io::Error::other("entry changed")),
        true,
        true
    ));
}

#[test]
fn xis_003_copy_rejects_an_entry_replaced_after_inspection() {
    for round in 0..XIS_003_REPLACEMENT_ROUNDS {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path();
        std::fs::write(root.join("source"), b"original").unwrap();
        std::fs::write(root.join("replacement"), b"replacement").unwrap();
        let hook_root = root.to_path_buf();
        operations::set_identity_race_hook(move || {
            std::fs::rename(hook_root.join("source"), hook_root.join("original-held")).unwrap();
            std::fs::rename(hook_root.join("replacement"), hook_root.join("source")).unwrap();
        });

        let result = execute(
            &operation(root, Operation::Copy, "source", Some("copied")),
            &|| false,
        );
        let copied = std::fs::read(root.join("copied")).ok();
        assert!(
            xis_003_copy_outcome_is_safe(&result, copied.as_deref()),
            "copy accepted the replacement entry in round {round}"
        );
    }
}

#[test]
fn xis_003_remove_rejects_an_entry_replaced_after_inspection() {
    for round in 0..XIS_003_REPLACEMENT_ROUNDS {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path();
        std::fs::write(root.join("source"), b"original").unwrap();
        std::fs::write(root.join("replacement"), b"replacement").unwrap();
        let hook_root = root.to_path_buf();
        operations::set_identity_race_hook(move || {
            std::fs::rename(hook_root.join("source"), hook_root.join("original-held")).unwrap();
            std::fs::rename(hook_root.join("replacement"), hook_root.join("source")).unwrap();
        });

        let result = execute(&operation(root, Operation::Delete, "source", None), &|| {
            false
        });
        assert!(
            xis_003_remove_outcome_is_safe(
                &result,
                root.join("original-held").exists(),
                root.join("source").exists(),
            ),
            "remove deleted the replacement entry in round {round}"
        );
    }
}

#[test]
fn worker_rejects_parent_traversal_and_absolute_paths_outside_root() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("root");
    std::fs::create_dir(&root).unwrap();
    std::fs::write(fixture.path().join("private.txt"), "outside").unwrap();
    for path in [
        PathBuf::from("../private.txt"),
        fixture.path().join("private.txt"),
    ] {
        assert!(execute(&request(&root, Job::Preview(path)), &|| false).is_err());
    }
    std::fs::write(root.join("inside.txt"), "inside").unwrap();
    assert!(
        matches!(execute(&request(&root, Job::Preview("inside.txt".into())), &|| false).unwrap(), Output::Preview { text, .. } if text == "inside")
    );
}

#[test]
fn worker_never_overwrites_and_failed_copy_preserves_source_and_cleans_staging() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    std::fs::write(root.join("source"), vec![7; 256 * 1024]).unwrap();
    std::fs::write(root.join("existing"), "keep").unwrap();
    for kind in [Operation::Copy, Operation::Move, Operation::Rename] {
        assert!(
            execute(&operation(root, kind, "source", Some("existing")), &|| {
                false
            })
            .is_err()
        );
        assert_eq!(std::fs::read(root.join("existing")).unwrap(), b"keep");
        assert_eq!(
            std::fs::metadata(root.join("source")).unwrap().len(),
            256 * 1024
        );
    }
    let checks = std::cell::Cell::new(0);
    let result = execute(
        &operation(root, Operation::Copy, "source", Some("cancelled")),
        &|| {
            checks.set(checks.get() + 1);
            checks.get() > 5
        },
    );
    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::Interrupted);
    assert!(!root.join("cancelled").exists());
    assert_eq!(
        std::fs::read(root.join("source")).unwrap(),
        vec![7; 256 * 1024]
    );
    assert_eq!(
        std::fs::read_dir(root).unwrap().count(),
        2,
        "private staging must be removed"
    );
}

#[test]
fn worker_copies_directory_contents_rejects_cycles_and_preserves_failed_move() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    std::fs::create_dir_all(root.join("source/nested")).unwrap();
    std::fs::write(root.join("source/nested/file"), "bytes").unwrap();
    execute(
        &operation(root, Operation::Copy, "source", Some("copy")),
        &|| false,
    )
    .unwrap();
    assert_eq!(
        std::fs::read(root.join("copy/nested/file")).unwrap(),
        b"bytes"
    );
    for kind in [Operation::Copy, Operation::Move] {
        assert!(execute(
            &operation(root, kind, "source", Some("source/nested/child")),
            &|| false
        )
        .is_err());
        assert!(!root.join("source/nested/child").exists());
        assert!(root.join("source/nested/file").exists());
    }
    execute(&operation(root, Operation::Delete, "copy", None), &|| false).unwrap();
    assert!(!root.join("copy").exists());
    assert!(execute(&operation(root, Operation::Delete, ".", None), &|| false).is_err());
    assert!(root.join("source/nested/file").exists());
}

#[test]
fn worker_preview_bounds_bytes_and_handles_a_split_utf8_tail() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    let mut text = "x".repeat(PREVIEW_BYTES as usize - 1);
    text.push_str("界tail");
    std::fs::write(root.join("text"), text).unwrap();
    let Output::Preview { text, .. } =
        execute(&request(root, Job::Preview("text".into())), &|| false).unwrap()
    else {
        panic!("preview required")
    };
    assert!(text.ends_with("[Preview truncated]"));
    assert!(!text.contains("Binary"));
    assert!(text.len() < PREVIEW_BYTES as usize + 30);
    std::fs::write(root.join("binary"), [0, 1, 2]).unwrap();
    assert!(
        matches!(execute(&request(root, Job::Preview("binary".into())), &|| false).unwrap(), Output::Preview { text, .. } if text.contains("Binary file") && text.contains("3 bytes"))
    );
}

#[test]
fn worker_replaces_pending_reads_and_drop_joins_its_owner() {
    let fixture = tempfile::tempdir().unwrap();
    std::fs::create_dir(fixture.path().join("new")).unwrap();
    std::fs::write(fixture.path().join("new/latest"), "latest").unwrap();
    let worker = Worker::new().unwrap();
    let shared = Arc::downgrade(&worker.shared);
    worker.submit(fixture.path().to_path_buf(), Job::Read("missing".into()));
    let id = worker.submit(fixture.path().to_path_buf(), Job::Read("new".into()));
    // A hang guard, not a timing check: generous so a busy machine cannot
    // fail a correct test by running it slowly.
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if let Some(response) = worker.take() {
            assert_eq!(response.id, id);
            assert!(
                matches!(response.result.unwrap(), Output::Entries { entries, .. } if entries.len() == 1 && entries[0].name == "latest")
            );
            break;
        }
        assert!(Instant::now() < deadline, "worker response timed out");
        thread::sleep(Duration::from_millis(1));
    }
    drop(worker);
    assert!(
        shared.upgrade().is_none(),
        "worker and result slots must be released on removal"
    );
}

#[cfg(any(unix, windows))]
#[test]
fn worker_never_follows_an_external_symlink() {
    #[cfg(unix)]
    use std::os::unix::fs::symlink;
    #[cfg(windows)]
    use std::os::windows::fs::symlink_file as symlink;
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("root");
    std::fs::create_dir(&root).unwrap();
    let outside = fixture.path().join("outside");
    std::fs::write(&outside, "outside bytes").unwrap();
    symlink(&outside, root.join("link")).unwrap();
    let stored_target = std::fs::read_link(root.join("link")).unwrap();
    let Output::Entries { entries, .. } =
        execute(&request(&root, Job::Read(".".into())), &|| false).unwrap()
    else {
        panic!("listing required")
    };
    assert!(entries
        .iter()
        .any(|entry| entry.name == "link" && entry.file_type == FileType::Symlink));
    assert!(execute(&request(&root, Job::Preview("link".into())), &|| false).is_err());
    execute(
        &operation(&root, Operation::Copy, "link", Some("copied-link")),
        &|| false,
    )
    .unwrap();
    assert_eq!(
        std::fs::read_link(root.join("copied-link")).unwrap(),
        stored_target
    );
    execute(&operation(&root, Operation::Delete, "link", None), &|| {
        false
    })
    .unwrap();
    assert_eq!(std::fs::read(outside).unwrap(), b"outside bytes");
}

#[cfg(any(unix, windows))]
#[test]
fn worker_copies_directory_symlinks_without_traversing_their_targets() {
    #[cfg(unix)]
    use std::os::unix::fs::symlink;
    #[cfg(windows)]
    use std::os::windows::fs::symlink_dir as symlink;
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("root");
    let outside = fixture.path().join("outside");
    std::fs::create_dir(&root).unwrap();
    std::fs::create_dir(&outside).unwrap();
    std::fs::write(outside.join("secret"), b"outside bytes").unwrap();
    symlink(&outside, root.join("link")).unwrap();
    let stored_target = std::fs::read_link(root.join("link")).unwrap();
    assert!(execute(&request(&root, Job::Read("link".into())), &|| false).is_err());
    execute(
        &operation(&root, Operation::Copy, "link", Some("copied-link")),
        &|| false,
    )
    .unwrap();
    assert_eq!(
        std::fs::read_link(root.join("copied-link")).unwrap(),
        stored_target
    );
    assert!(execute(
        &operation(&root, Operation::Copy, "link", Some("copied-link")),
        &|| false
    )
    .is_err());
    execute(&operation(&root, Operation::Delete, "link", None), &|| {
        false
    })
    .unwrap();
    execute(
        &operation(&root, Operation::Delete, "copied-link", None),
        &|| false,
    )
    .unwrap();
    assert_eq!(
        std::fs::read(outside.join("secret")).unwrap(),
        b"outside bytes"
    );
    assert!(std::fs::read_dir(&root).unwrap().next().is_none());
}

#[cfg(windows)]
#[test]
fn worker_resolves_nonverbatim_paths_inside_a_verbatim_root() {
    let fixture = tempfile::tempdir().unwrap();
    std::fs::create_dir(fixture.path().join("folder")).unwrap();
    std::fs::write(fixture.path().join("folder/entry"), b"data").unwrap();
    let canonical = fixture.path().canonicalize().unwrap();
    let Output::Entries { entries, .. } = execute(
        &request(&canonical, Job::Read(fixture.path().join("folder"))),
        &|| false,
    )
    .unwrap() else {
        panic!("listing required")
    };
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].path, canonical.join("folder/entry"));
    execute(
        &operation(
            &canonical,
            Operation::Copy,
            fixture.path().join("folder/entry").to_str().unwrap(),
            Some(fixture.path().join("copy").to_str().unwrap()),
        ),
        &|| false,
    )
    .unwrap();
    assert_eq!(std::fs::read(fixture.path().join("copy")).unwrap(), b"data");
}

#[cfg(any(target_os = "linux", windows))]
#[test]
fn worker_keeps_non_unicode_filename_identities() {
    // APFS rejects invalid UTF-8 before the explorer can read it. Exercise
    // native non-Unicode filenames on filesystems that can create them.
    #[cfg(target_os = "linux")]
    let native = {
        use std::os::unix::ffi::OsStringExt;
        std::ffi::OsString::from_vec(vec![b'n', 0xff])
    };
    #[cfg(windows)]
    let native = {
        use std::os::windows::ffi::OsStringExt;
        std::ffi::OsString::from_wide(&[u16::from(b'n'), 0xd800])
    };
    let fixture = tempfile::tempdir().unwrap();
    std::fs::write(fixture.path().join(&native), "native").unwrap();
    let Output::Entries { entries, .. } =
        execute(&request(fixture.path(), Job::Read(".".into())), &|| false).unwrap()
    else {
        panic!("listing required")
    };
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].path.file_name(), Some(native.as_os_str()));
}

#[test]
fn worker_preserves_the_filesystems_unicode_filename_identity() {
    let fixture = tempfile::tempdir().unwrap();
    std::fs::write(fixture.path().join("naïve-界.txt"), "native").unwrap();
    // Compare with the filesystem's own spelling, including normalization.
    let native = std::fs::read_dir(fixture.path())
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .file_name();
    let Output::Entries { entries, .. } =
        execute(&request(fixture.path(), Job::Read(".".into())), &|| false).unwrap()
    else {
        panic!("listing required")
    };
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].path.file_name(), Some(native.as_os_str()));
}

#[cfg(unix)]
#[test]
fn worker_directory_copy_preserves_private_mode_bits() {
    use std::os::unix::fs::PermissionsExt;
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    std::fs::create_dir(root.join("private")).unwrap();
    std::fs::set_permissions(root.join("private"), std::fs::Permissions::from_mode(0o700)).unwrap();
    std::fs::write(root.join("private/content"), "private content").unwrap();
    std::fs::set_permissions(
        root.join("private/content"),
        std::fs::Permissions::from_mode(0o644),
    )
    .unwrap();
    execute(
        &operation(root, Operation::Copy, "private", Some("copied")),
        &|| false,
    )
    .unwrap();
    assert_eq!(
        std::fs::metadata(root.join("copied"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
    assert_eq!(
        std::fs::metadata(root.join("copied/content"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o644
    );
}

#[cfg(unix)]
#[test]
fn worker_late_destination_collision_cleans_readonly_staging_without_changing_source() {
    use std::os::unix::fs::PermissionsExt;
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    std::fs::create_dir(root.join("readonly")).unwrap();
    std::fs::write(root.join("readonly/content"), "source").unwrap();
    std::fs::set_permissions(
        root.join("readonly"),
        std::fs::Permissions::from_mode(0o555),
    )
    .unwrap();
    let collision = std::cell::Cell::new(false);
    let result = execute(
        &operation(root, Operation::Copy, "readonly", Some("target")),
        &|| {
            if !collision.get() {
                for entry in std::fs::read_dir(root).unwrap() {
                    let entry = entry.unwrap();
                    if entry
                        .file_name()
                        .to_string_lossy()
                        .starts_with(".reactive-tui-copy-")
                        && std::fs::metadata(entry.path().join("item"))
                            .is_ok_and(|metadata| metadata.permissions().mode() & 0o777 == 0o555)
                    {
                        std::fs::write(root.join("target"), "concurrent destination").unwrap();
                        collision.set(true);
                    }
                }
            }
            false
        },
    );
    assert!(result.is_err());
    assert!(
        collision.get(),
        "the collision must occur after staging, before publication"
    );
    assert_eq!(
        std::fs::read(root.join("target")).unwrap(),
        b"concurrent destination"
    );
    assert_eq!(
        std::fs::read(root.join("readonly/content")).unwrap(),
        b"source"
    );
    assert_eq!(
        std::fs::metadata(root.join("readonly"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o555
    );
    assert_eq!(
        std::fs::read_dir(root).unwrap().count(),
        2,
        "read-only staging must be removed"
    );
    std::fs::set_permissions(
        root.join("readonly"),
        std::fs::Permissions::from_mode(0o700),
    )
    .unwrap();
}
