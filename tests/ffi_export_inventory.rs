use std::path::{Path, PathBuf};

use syn::{Expr, Item, Stmt};

fn is_public_c(function: &syn::ItemFn) -> bool {
    matches!(function.vis, syn::Visibility::Public(_))
        && function
            .sig
            .abi
            .as_ref()
            .and_then(|abi| abi.name.as_ref())
            .is_some_and(|name| name.value() == "C")
}

fn has_export_guard(function: &syn::ItemFn) -> bool {
    function.attrs.iter().any(|attribute| {
        attribute
            .path()
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "ffi_export")
    })
}

fn call_name(expression: &Expr) -> Option<&syn::Ident> {
    let Expr::Call(call) = expression else {
        return None;
    };
    let Expr::Path(path) = call.func.as_ref() else {
        return None;
    };
    path.path.segments.last().map(|segment| &segment.ident)
}

fn has_tail_guard(function: &syn::ItemFn) -> bool {
    let Some(Stmt::Expr(expression, None)) = function.block.stmts.last() else {
        return false;
    };
    call_name(expression).is_some_and(|name| {
        matches!(
            name.to_string().as_str(),
            "catch_panic" | "catch_panic_with_default" | "catch_unwind"
        )
    })
}

fn unguarded_exports(path: &Path, source: &str) -> Vec<String> {
    let syntax = syn::parse_file(source).unwrap_or_else(|error| {
        panic!("failed to parse {}: {error}", path.display());
    });
    syntax
        .items
        .into_iter()
        .filter_map(|item| match item {
            Item::Fn(function)
                if is_public_c(&function)
                    && !has_export_guard(&function)
                    && !has_tail_guard(&function) =>
            {
                Some(format!("{}::{}", path.display(), function.sig.ident))
            }
            _ => None,
        })
        .collect()
}

fn active_ffi_sources(root: &Path) -> Vec<PathBuf> {
    let module_path = root.join("src/ffi/mod.rs");
    let source = std::fs::read_to_string(&module_path).unwrap();
    let syntax = syn::parse_file(&source).unwrap();
    let mut paths = vec![module_path];
    for item in syntax.items {
        if let Item::Mod(module) = item {
            if module.content.is_none() {
                let path = root.join("src/ffi").join(format!("{}.rs", module.ident));
                if path.is_file() {
                    paths.push(path);
                }
            }
        }
    }
    paths
}

#[test]
fn every_active_c_export_has_a_panic_boundary() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let missing: Vec<_> = active_ffi_sources(root)
        .into_iter()
        .flat_map(|path| {
            let source = std::fs::read_to_string(&path).unwrap();
            unguarded_exports(path.strip_prefix(root).unwrap(), &source)
        })
        .collect();
    assert!(
        missing.is_empty(),
        "unguarded C exports:\n{}",
        missing.join("\n")
    );
}

#[test]
fn inventory_rejects_an_export_with_its_boundary_removed() {
    let fixture = r#"
        #[no_mangle]
        pub extern "C" fn violating_fixture() -> u32 { 7 }
    "#;
    assert_eq!(
        unguarded_exports(Path::new("violating.rs"), fixture),
        ["violating.rs::violating_fixture"]
    );
}
