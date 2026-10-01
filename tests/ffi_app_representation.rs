use std::path::Path;

use reactive_tui::ffi::{RTuiEffectCleanupCallback, RTuiElement, RTuiRootComponentCallback};
use syn::visit::{self, Visit};
use syn::{Expr, GenericArgument, Item, PathArguments, Type};

fn type_argument<'a>(kind: &str, value: &'a Type) -> Option<&'a Type> {
    let Type::Path(path) = value else {
        return None;
    };
    let segment = path.path.segments.last()?;
    if segment.ident != kind {
        return None;
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };
    arguments.args.iter().find_map(|argument| match argument {
        GenericArgument::Type(value) => Some(value),
        _ => None,
    })
}

fn is_named_type(expected: &str, value: &Type) -> bool {
    matches!(value, Type::Path(path) if path.path.segments.last().is_some_and(|segment| segment.ident == expected))
}

fn is_retained_app_slot(value: &Type) -> bool {
    type_argument("Mutex", value)
        .and_then(|value| type_argument("Option", value))
        .is_some_and(|value| is_named_type("App", value))
}

#[derive(Default)]
struct FromRawCalls {
    app_run_uses_box_from_raw: bool,
}

impl<'ast> Visit<'ast> for FromRawCalls {
    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if let Expr::Path(path) = call.func.as_ref() {
            let mut segments = path.path.segments.iter().rev();
            if segments
                .next()
                .is_some_and(|segment| segment.ident == "from_raw")
                && segments
                    .next()
                    .is_some_and(|segment| segment.ident == "Box")
            {
                self.app_run_uses_box_from_raw = true;
            }
        }
        visit::visit_expr_call(self, call);
    }
}

fn representation_errors<'a>(
    sources: impl IntoIterator<Item = (&'a Path, &'a str)>,
) -> Vec<String> {
    let mut app_slot = None;
    let mut app_run = None;

    for (path, source) in sources {
        let syntax = syn::parse_file(source).unwrap_or_else(|error| {
            panic!("failed to parse {}: {error}", path.display());
        });
        for item in &syntax.items {
            match item {
                Item::Struct(owner) if owner.ident == "NativeApp" => {
                    app_slot = owner
                        .fields
                        .iter()
                        .find(|field| field.ident.as_ref().is_some_and(|name| name == "app"))
                        .map(|field| is_retained_app_slot(&field.ty));
                }
                Item::Fn(function) if function.sig.ident == "rtui_app_run" => {
                    let mut calls = FromRawCalls::default();
                    calls.visit_block(&function.block);
                    app_run = Some(calls);
                }
                _ => {}
            }
        }
    }

    let mut errors = Vec::new();
    if app_slot != Some(true) {
        errors.push("NativeApp.app must retain Mutex<Option<App>> storage".to_owned());
    }
    if app_run
        .as_ref()
        .is_none_or(|calls| calls.app_run_uses_box_from_raw)
    {
        errors.push("rtui_app_run must not recover App ownership with Box::from_raw".to_owned());
    }
    errors
}

type RootCallback = extern "C" fn(*mut std::ffi::c_void) -> *mut RTuiElement;
type CleanupCallback = extern "C" fn(*mut std::ffi::c_void);

fn root_callback_is_nullable(callback: RTuiRootComponentCallback) -> Option<RootCallback> {
    callback
}

fn cleanup_callback_is_nullable(callback: RTuiEffectCleanupCallback) -> Option<CleanupCallback> {
    callback
}

#[test]
fn nullable_callback_aliases_have_exact_function_pointer_types() {
    assert!(root_callback_is_nullable(None).is_none());
    assert!(cleanup_callback_is_nullable(None).is_none());
}

#[test]
fn app_handle_retains_ownership_until_run() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sources = ["src/ffi/app.rs", "src/ffi/reactive.rs"]
        .into_iter()
        .map(|name| {
            let path = root.join(name);
            let source = std::fs::read_to_string(&path).unwrap();
            (path, source)
        })
        .collect::<Vec<_>>();
    let errors = representation_errors(
        sources
            .iter()
            .map(|(path, source)| (path.as_path(), source.as_str())),
    );
    assert!(errors.is_empty(), "{}", errors.join("\n"));
}

#[test]
fn inventory_rejects_moved_app_ownership() {
    let fixture = r#"
        struct App;
        struct RTuiApp;
        struct NativeApp { app: App }
        pub extern "C" fn rtui_app_run(app: *mut RTuiApp) {
            let owned = unsafe { Box::from_raw(app.cast::<App>()) };
            drop(owned);
        }
    "#;
    let errors = representation_errors([(Path::new("violating.rs"), fixture)]);
    assert_eq!(errors.len(), 2, "unexpected errors: {errors:?}");
    assert!(errors
        .iter()
        .any(|error| error.contains("Mutex<Option<App>>")));
    assert!(errors.iter().any(|error| error.contains("Box::from_raw")));
}
