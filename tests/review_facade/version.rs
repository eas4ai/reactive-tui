//! Part of tests/review_facade.rs: FFI-007, one version everywhere.

use reactive_tui::ffi::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// FFI-007: the native version is the crate's, with the ABI version apart.
#[test]
fn ffi_007_the_native_version_is_the_crates() {
    let version = rtui_version();
    let reported = format!("{}.{}.{}", version.major, version.minor, version.patch);
    assert_eq!(
        reported, VERSION,
        "FFI-007: rtui_version reports {reported} where Cargo.toml says {VERSION}"
    );
    assert_eq!(
        version.abi_version, 1,
        "the ABI version is its own constant"
    );
}

/// FFI-007: README, the umbrella header and the TypeScript package name the
/// crate's version, and README names the highlighter in use.
#[test]
fn ffi_007_readme_header_and_package_name_the_crates_version() {
    let readme = include_str!("../../README.md");
    let header = include_str!("../../include/reactive_tui.h");
    let package = include_str!("../../bindings/typescript/package.json");
    let mut wrong = Vec::new();
    if !readme.contains(VERSION) {
        wrong.push(format!("README.md does not name {VERSION}"));
    }
    if readme.contains("Syntect") || !readme.contains("Lumis") {
        wrong.push("README.md names Syntect, not Lumis, as the highlighter".to_string());
    }
    if !header.contains(&format!("@version {VERSION}")) {
        wrong.push(format!(
            "include/reactive_tui.h does not say @version {VERSION}"
        ));
    }
    if !package.contains(&format!("\"version\": \"{VERSION}\"")) {
        wrong.push(format!(
            "bindings/typescript/package.json is not at {VERSION}"
        ));
    }
    assert!(wrong.is_empty(), "FFI-007: {}", wrong.join("; "));
}
