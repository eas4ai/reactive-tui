fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("gnu")
    {
        // Without the FFI feature there are no explicit C exports. GNU ld
        // otherwise exports dependency internals and overflows the PE table.
        // Explicit exports emitted by rustc for the FFI feature are retained.
        println!("cargo:rustc-link-arg-cdylib=-Wl,--exclude-all-symbols");
    }
}
