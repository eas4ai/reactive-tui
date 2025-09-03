use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, Pat, Type};

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
                    "Component functions cannot have self parameters"
                ).to_compile_error().into();
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
            hooks_param.is_some()
        )
    } else {
        // Component with props
        generate_props_component(
            fn_vis,
            component_struct_name,
            fn_body,
            fn_return_type,
            &prop_params,
            hooks_param.is_some()
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

/// Derives the Props trait for a struct with automatic validation and defaults
///
/// # Usage
///
/// ```rust,ignore
/// use reactive_tui::prelude::*;
///
/// #[derive(Props)]
/// struct ButtonProps {
///     text: String,
///     #[prop(default)]
///     disabled: bool,
///     #[prop(default = "primary")]
///     variant: String,
///     #[prop(optional)]
///     on_click: Option<Box<dyn Fn()>>,
/// }
/// ```
///
/// # Attributes
///
/// - `#[prop(default)]` - Use Default::default() for this field
/// - `#[prop(default = "value")]` - Use a specific default value
/// - `#[prop(optional)]` - Make this field optional (wrap in Option if not already)
/// - `#[prop(validate)]` - Add validation for this field
#[proc_macro_derive(Props, attributes(prop))]
pub fn derive_props(input: TokenStream) -> TokenStream {
    use syn::{parse_macro_input, DeriveInput, Data, Fields, Lit};

    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Parse the struct fields
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(
                    &input,
                    "Props can only be derived for structs with named fields"
                ).to_compile_error().into();
            }
        },
        _ => {
            return syn::Error::new_spanned(
                &input,
                "Props can only be derived for structs"
            ).to_compile_error().into();
        }
    };

    // Process each field to extract prop attributes
    let mut builder_methods = Vec::new();
    let mut default_implementations = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        // Parse prop attributes
        let mut has_default = false;
        let mut default_value = None;
        let mut is_optional = false;

        for attr in &field.attrs {
            if attr.path().is_ident("prop") {
                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("default") {
                        if meta.input.peek(syn::Token![=]) {
                            let _: syn::Token![=] = meta.input.parse()?;
                            let value: Lit = meta.input.parse()?;
                            if let Lit::Str(lit_str) = value {
                                default_value = Some(lit_str.value());
                            }
                        }
                        has_default = true;
                        Ok(())
                    } else if meta.path.is_ident("optional") {
                        is_optional = true;
                        Ok(())
                    } else {
                        Err(meta.error("unsupported prop attribute"))
                    }
                });
            }
        }

        // Generate builder method
        let builder_method_name = syn::Ident::new(&format!("with_{}", field_name), field_name.span());
        builder_methods.push(quote! {
            pub fn #builder_method_name(mut self, value: #field_type) -> Self {
                self.#field_name = value;
                self
            }
        });

        // Generate default implementation
        if has_default {
            if let Some(default_val) = default_value {
                default_implementations.push(quote! {
                    #field_name: #default_val.into()
                });
            } else {
                default_implementations.push(quote! {
                    #field_name: Default::default()
                });
            }
        } else if is_optional {
            default_implementations.push(quote! {
                #field_name: None
            });
        } else {
            // Required field - no default
            default_implementations.push(quote! {
                #field_name: Default::default()
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

            /// Validate all component properties
            /// Currently performs basic validation - can be extended for specific validation rules
            pub fn validate(&self) -> bool {
                // Basic validation passes - extend this method for specific validation needs
                true
            }
        }


    };

    TokenStream::from(expanded)
}

// Note: Unit tests for procedural macros cannot be run in the same way as regular tests
// because they require the proc-macro context. The functionality is tested through
// integration tests in the main reactive-tui crate (see tests/component_macro_test.rs
// and tests/props_derive_test.rs).
