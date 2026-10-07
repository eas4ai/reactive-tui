//! Fixtures the tests build fresh for each run.

/// A directory of four fixed names for a file explorer: three files and a
/// directory. A golden that lists it is stable across clones and edits,
/// unlike one that lists a directory of the repository, whose files,
/// sizes and dates change.
#[allow(dead_code)]
pub fn explorer_dir() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("a fixture directory");
    for name in ["alpha.md", "beta.md", "gamma.md"] {
        std::fs::write(dir.path().join(name), name).expect("a fixture file");
    }
    std::fs::create_dir(dir.path().join("notes")).expect("a fixture directory");
    dir
}
