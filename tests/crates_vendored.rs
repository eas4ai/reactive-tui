//! Guards the vendoring veto: companion crates stay path-only workspace
//! members under `crates/` and must never publish.
//!
//! This mirrors `scripts/check-crates-release.py` in the Rust harness so a
//! removed `publish = false` or a stale workspace member fails `cargo test`
//! directly.

use std::path::PathBuf;

const COMPANION_MANIFESTS: &[&str] = &[
    "crates/reactive-tui-macros/Cargo.toml",
    "crates/reactive-tui-crossterm/Cargo.toml",
    "crates/reactive-tui-suprtui/Cargo.toml",
    "crates/libghostty-vt/Cargo.toml",
    "crates/libghostty-vt-sys/Cargo.toml",
];

const WORKSPACE_MEMBERS: &[&str] = &[
    "crates/reactive-tui-macros",
    "crates/reactive-tui-crossterm",
    "crates/reactive-tui-suprtui",
    "crates/libghostty-vt",
    "crates/libghostty-vt-sys",
];

fn read_tree(rel: &str) -> String {
    let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), rel].iter().collect();
    std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("readable manifest: {rel}"))
}

#[test]
fn companions_are_unpublishable() {
    for manifest in COMPANION_MANIFESTS {
        let text = read_tree(manifest);
        assert!(
            text.lines().any(|line| line.trim() == "publish = false"),
            "{manifest} must set publish = false (vendored, never publish)"
        );
    }
}

#[test]
fn workspace_members_are_the_vendored_set() {
    let root = read_tree("Cargo.toml");
    let workspace = root
        .split("[workspace]")
        .nth(1)
        .expect("root [workspace] section");
    // End at the next section header; the members array's own `[` stays.
    let workspace = workspace.split("\n[").next().unwrap_or(workspace);
    for member in WORKSPACE_MEMBERS {
        assert!(
            workspace.contains(&format!("\"{member}\"")),
            "workspace must list member {member}"
        );
    }
    assert!(
        !workspace.contains("src/backend"),
        "workspace must not reference the pre-vendor src/backend layout"
    );
}
