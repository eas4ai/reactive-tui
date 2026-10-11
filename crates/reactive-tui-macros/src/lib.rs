//! Procedural macros for reactive-tui: `#[component]` turns a function into
//! a component, `#[derive(Props)]` builds a props type with its defaults and
//! validation, and `#[ffi_export]` exports a C ABI function behind a panic
//! boundary. The main crate re-exports `component` and `Props`; its C ABI
//! uses `ffi_export`.
#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, FnArg, ItemFn, Pat, Type};

/// Export a C ABI function with a panic boundary and a type-specific fallback.
#[proc_macro_attribute]
pub fn ffi_export(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "ffi_export takes no arguments",
        )
        .to_compile_error()
        .into();
    }

    let mut function = parse_macro_input!(input as ItemFn);
    let is_c_abi = function
        .sig
        .abi
        .as_ref()
        .and_then(|abi| abi.name.as_ref())
        .is_some_and(|name| name.value() == "C");
    if !is_c_abi {
        return syn::Error::new_spanned(&function.sig, "ffi_export requires extern \"C\"")
            .to_compile_error()
            .into();
    }

    let body = function.block;
    function.block = Box::new(parse_quote!({
        match ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| #body)) {
            Ok(value) => value,
            Err(_) => crate::ffi::ffi_panic_default(),
        }
    }));
    function.attrs.push(parse_quote!(#[no_mangle]));
    quote!(#function).into()
}

/// Transforms a function into a reactive-tui Component
///
/// # Usage
///
/// ```rust,ignore
/// use reactive_tui::prelude::*;
///
/// #[component]
/// fn Counter(hooks: &Hooks) -> Element {
///     let count = use_signal(hooks, 0);
///
///     Element::text(&format!("Count: {}", count.get()))
/// }
/// ```
///
/// With props:
///
/// ```rust,ignore
/// use reactive_tui::prelude::*;
///
/// #[component]
/// fn Greeting(hooks: &Hooks, name: String, age: Option<u32>) -> Element {
///     let message = if let Some(age) = age {
///         format!("Hello {}, you are {} years old!", name, age)
///     } else {
///         format!("Hello {}!", name)
///     };
///
///     Element::text(&message)
/// }
/// ```
#[proc_macro_attribute]
pub fn component(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_body = &input_fn.block;
    let fn_return_type = &input_fn.sig.output;

    // Parse function parameters
    let mut hooks_param = None;
    let mut prop_params = Vec::new();

    for input in &input_fn.sig.inputs {
        match input {
            FnArg::Typed(pat_type) => {
                if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    let param_name = &pat_ident.ident;
                    let param_type = &*pat_type.ty;

                    // Check if this is the hooks parameter
                    if param_name == "hooks" {
                        hooks_param = Some((param_name, param_type));
                    } else {
                        prop_params.push((param_name, param_type));
                    }
                }
            }
            FnArg::Receiver(_) => {
                return syn::Error::new_spanned(
                    input,
                    "Component functions cannot have self parameters",
                )
                .to_compile_error()
                .into();
            }
        }
    }

    // Generate the component struct and implementation
    let component_struct_name = fn_name;

    if prop_params.is_empty() {
        // No props component
        generate_no_props_component(
            fn_vis,
            component_struct_name,
            fn_body,
            fn_return_type,
            hooks_param.is_some(),
        )
    } else {
        // Component with props
        generate_props_component(
            fn_vis,
            component_struct_name,
            fn_body,
            fn_return_type,
            &prop_params,
            hooks_param.is_some(),
        )
    }
}

fn generate_no_props_component(
    vis: &syn::Visibility,
    name: &syn::Ident,
    body: &syn::Block,
    return_type: &syn::ReturnType,
    _has_hooks: bool,
) -> TokenStream {
    let expanded = quote! {
        #[derive(Clone)]
        #vis struct #name {
            hooks: reactive_tui::reactive::Hooks,
        }

        impl reactive_tui::component::Component for #name {
            type Props = reactive_tui::component::props::EmptyProps;
            type State = ();

            fn new(_props: Self::Props) -> Self {
                Self {
                    hooks: reactive_tui::reactive::Hooks::new(),
                }
            }

            fn update(&mut self, _props: &Self::Props, _state: &mut Self::State) -> bool {
                true
            }

            fn render(&self, _props: &Self::Props, _state: &Self::State) #return_type {
                let _hook_frame = self.hooks.begin_render();
                let hooks = &self.hooks;
                #body
            }
        }

        impl #name {
            /// Create a new element for this component
            pub fn element() -> reactive_tui::component::Element {
                reactive_tui::component::Element::component_with_props(
                    stringify!(#name),
                    reactive_tui::component::props::EmptyProps
                )
            }
        }

        impl Into<reactive_tui::component::Element> for #name {
            fn into(self) -> reactive_tui::component::Element {
                Self::element()
            }
        }
    };

    TokenStream::from(expanded)
}

fn generate_props_component(
    vis: &syn::Visibility,
    name: &syn::Ident,
    body: &syn::Block,
    return_type: &syn::ReturnType,
    prop_params: &[(&syn::Ident, &Type)],
    has_hooks: bool,
) -> TokenStream {
    let props_struct_name = syn::Ident::new(&format!("{}Props", name), name.span());

    // Generate props struct fields
    let prop_fields = prop_params.iter().map(|(name, ty)| {
        quote! { pub #name: #ty }
    });

    // Generate props struct field names for destructuring
    let prop_names: Vec<_> = prop_params.iter().map(|(name, _)| name).collect();
    let prop_names_clone = prop_names.clone();
    let prop_types: Vec<_> = prop_params.iter().map(|(_, ty)| ty).collect();

    // Generate render function call
    let render_call = if has_hooks {
        quote! {
            let hooks = &self.hooks;
            let #props_struct_name { #(#prop_names),* } = props;
            #body
        }
    } else {
        quote! {
            let #props_struct_name { #(#prop_names),* } = props;
            #body
        }
    };

    let expanded = quote! {
        #[derive(Clone, PartialEq, Debug)]
        #vis struct #props_struct_name {
            #(#prop_fields),*
        }

        impl reactive_tui::component::Props for #props_struct_name {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }

        #[derive(Clone)]
        #vis struct #name {
            hooks: reactive_tui::reactive::Hooks,
        }

        impl reactive_tui::component::Component for #name {
            type Props = #props_struct_name;
            type State = ();

            fn new(_props: Self::Props) -> Self {
                Self {
                    hooks: reactive_tui::reactive::Hooks::new(),
                }
            }

            fn update(&mut self, _props: &Self::Props, _state: &mut Self::State) -> bool {
                true
            }

            fn render(&self, props: &Self::Props, _state: &Self::State) #return_type {
                let _hook_frame = self.hooks.begin_render();
                #render_call
            }
        }

        impl #name {
            /// Create a new element for this component with props
            pub fn element(#(#prop_names_clone: #prop_types),*) -> reactive_tui::component::Element {
                let props = #props_struct_name {
                    #(#prop_names_clone),*
                };
                reactive_tui::component::Element::component_with_props(stringify!(#name), props)
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derives the Props trait, defaults, builders and caller-invoked validation.
///
/// # Usage
///
/// ```rust,ignore
/// use reactive_tui::prelude::*;
///
/// fn non_blank(text: &str) -> bool { !text.trim().is_empty() }
///
/// #[derive(Props, Clone, PartialEq)]
/// struct ButtonProps {
///     #[prop(validate = non_blank)]
///     text: String,
///     #[prop(default)]
///     disabled: bool,
///     #[prop(default = "primary")]
///     variant: String,
///     #[prop(optional)]
///     icon: Option<String>,
/// }
/// let props = ButtonProps::new().with_text("Save".into());
/// assert!(props.validate());
/// assert!(!props.with_text(" ".into()).validate());
/// ```
///
/// # Attributes
///
/// - `#[prop(default)]` - Use Default::default() for this field
/// - `#[prop(default = "value")]` - Convert a string literal into the field type.
/// - `#[prop(optional)]` - Default an explicitly written `Option<T>` field to `None`.
/// - `#[prop(validate = rule)]` - Call a named predicate with `&self.field`.
///
/// `validate()` returns true only if every declared rule returns true. Evaluation
/// follows field order and stops at the first false rule. No rules means no extra
/// constraints. Construction, builders and App mounting do not call validation.
/// Defaults may therefore be invalid until the caller supplies a value. Fields
/// without a default annotation use `Default::default()` too. An explicit string
/// default takes precedence over `optional`; `optional, default` remains `None`.
/// Bare `#[prop(validate)]` is an error; name the
/// rule explicitly. Rules must return `bool`; field types are never rewritten.
/// Structs must have named fields and satisfy the `Props` trait bounds.
#[proc_macro_derive(Props, attributes(prop))]
pub fn derive_props(input: TokenStream) -> TokenStream {
    use syn::{parse_macro_input, Data, DeriveInput, Fields};

    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    if let Some(attr) = input.attrs.iter().find(|attr| attr.path().is_ident("prop")) {
        return syn::Error::new_spanned(attr, "prop attributes belong on named fields")
            .to_compile_error()
            .into();
    }

    // Parse the struct fields
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(
                    &input,
                    "Props can only be derived for structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&input, "Props can only be derived for structs")
                .to_compile_error()
                .into();
        }
    };

    // Process each field to extract prop attributes
    let mut builder_methods = Vec::new();
    let mut default_implementations = Vec::new();
    let mut validations = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        let options = match PropOptions::parse(field) {
            Ok(options) => options,
            Err(error) => return error.to_compile_error().into(),
        };
        if let Some(rule) = options.validate {
            validations.push(quote! { #rule(&self.#field_name) });
        }

        // Generate builder method
        let builder_method_name =
            syn::Ident::new(&format!("with_{}", field_name), field_name.span());
        builder_methods.push(quote! {
            pub fn #builder_method_name(mut self, value: #field_type) -> Self {
                self.#field_name = value;
                self
            }
        });

        // Generate default implementation
        if let Some(default_val) = options.default_value {
            default_implementations.push(quote! {
                #field_name: #default_val.into()
            });
        } else if options.optional {
            default_implementations.push(quote! {
                #field_name: ::core::option::Option::None
            });
        } else {
            default_implementations.push(quote! {
                #field_name: ::core::default::Default::default()
            });
        }
    }

    // Generate the complete implementation
    let expanded = quote! {
        impl reactive_tui::component::Props for #name {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }

        impl Default for #name {
            fn default() -> Self {
                Self {
                    #(#default_implementations),*
                }
            }
        }

        impl #name {
            /// Create a new instance with default values
            pub fn new() -> Self {
                Self::default()
            }

            /// Builder methods for fluent API
            #(#builder_methods)*

            /// Evaluate declared field predicates, stopping at the first false rule.
            /// Construction and builders do not invoke validation. No rules returns true.
            pub fn validate(&self) -> bool {
                true #(&& #validations)*
            }
        }


    };

    TokenStream::from(expanded)
}

#[derive(Default)]
struct PropOptions {
    default: bool,
    default_value: Option<syn::LitStr>,
    optional: bool,
    validate: Option<syn::Path>,
}

impl PropOptions {
    fn parse(field: &syn::Field) -> syn::Result<Self> {
        let mut options = Self::default();
        for attr in field
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("prop"))
        {
            let mut entries = 0;
            attr.parse_nested_meta(|meta| {
                entries += 1;
                if meta.path.is_ident("default") {
                    if options.default {
                        return Err(meta.error("duplicate prop default"));
                    }
                    options.default = true;
                    if meta.input.peek(syn::Token![=]) {
                        options.default_value = Some(meta.value()?.parse()?);
                    }
                } else if meta.path.is_ident("optional") {
                    if options.optional {
                        return Err(meta.error("duplicate prop optional"));
                    }
                    options.optional = true;
                } else if meta.path.is_ident("validate") {
                    if options.validate.is_some() {
                        return Err(meta.error("duplicate prop validate"));
                    }
                    if !meta.input.peek(syn::Token![=]) {
                        return Err(meta.error("name a validation rule: #[prop(validate = rule)]"));
                    }
                    options.validate = Some(meta.value()?.parse()?);
                } else {
                    return Err(meta.error(
                        "unsupported prop option; use default, optional or validate = rule",
                    ));
                }
                Ok(())
            })?;
            if entries == 0 {
                return Err(syn::Error::new_spanned(attr, "prop requires an option"));
            }
        }
        if options.optional {
            let is_option = match &field.ty {
                syn::Type::Path(ty) if ty.qself.is_none() => {
                    ty.path.segments.last().is_some_and(|segment| {
                        segment.ident == "Option"
                            && matches!(&segment.arguments, syn::PathArguments::AngleBracketed(args)
                                if args.args.len() == 1 && matches!(args.args.first(), Some(syn::GenericArgument::Type(_))))
                    })
                }
                _ => false,
            };
            if !is_option {
                return Err(syn::Error::new_spanned(
                    &field.ty,
                    "#[prop(optional)] requires an explicit Option<T> field",
                ));
            }
        }
        Ok(options)
    }
}

#[cfg(test)]
mod props_options_tests {
    use super::PropOptions;
    use syn::parse::Parser;

    fn parse(source: &str) -> syn::Result<PropOptions> {
        let field = syn::Field::parse_named.parse_str(source)?;
        PropOptions::parse(&field)
    }

    #[test]
    fn accepts_named_rules_and_explicit_option_paths() {
        for source in [
            "#[prop(validate = rules::valid)] value: String",
            "#[prop(default = \"guest\", validate = valid)] value: String",
            "#[prop(optional)] value: Option<String>",
            "#[prop(optional, validate = valid)] value: ::std::option::Option<String>",
            "#[prop(optional)] value: core::option::Option<u32>",
            "#[prop(optional, default)] value: Option<u32>",
            "#[prop(optional, default = \"guest\")] value: Option<&'static str>",
        ] {
            assert!(parse(source).is_ok(), "{source}");
        }
    }

    #[test]
    fn rejects_ignored_or_ambiguous_attributes_with_diagnostics() {
        for (source, diagnostic) in [
            ("#[prop(validate)] value: String", "name a validation rule"),
            ("#[prop(unknown)] value: String", "unsupported prop option"),
            ("#[prop()] value: String", "prop requires an option"),
            (
                "#[prop(default = 42)] value: u32",
                "expected string literal",
            ),
            (
                "#[prop(optional)] value: String",
                "requires an explicit Option<T>",
            ),
            (
                "#[prop(optional)] value: Option",
                "requires an explicit Option<T>",
            ),
            (
                "#[prop(default, default)] value: String",
                "duplicate prop default",
            ),
            (
                "#[prop(optional)] #[prop(optional)] value: Option<u32>",
                "duplicate prop optional",
            ),
            (
                "#[prop(validate = first, validate = second)] value: String",
                "duplicate prop validate",
            ),
            (
                "#[prop(validate = \"rule\")] value: String",
                "expected identifier",
            ),
            (
                "#[prop(validate =)] value: String",
                "unexpected end of input",
            ),
            (
                "#[prop(optional = true)] value: Option<u32>",
                "expected `,`",
            ),
        ] {
            let error = parse(source)
                .err()
                .unwrap_or_else(|| panic!("accepted {source}"));
            assert!(error.to_string().contains(diagnostic), "{source}: {error}");
        }
    }
}

// Note: Unit tests for procedural macros cannot be run in the same way as regular tests
// because they require the proc-macro context. The functionality is tested through
// integration tests in the main reactive-tui crate (see tests/component_macro_test.rs
// and tests/props_derive_test.rs).
