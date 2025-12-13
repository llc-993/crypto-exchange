use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Meta};



/// Derive macro for automatic config field mapping
/// 
/// Example:
/// ```
/// #[derive(ConfigMapping)]
/// pub struct BaseConfig {
///     #[config(code = "UPLOAD_SERVICE_NAME", default = "loadUpload")]
///     pub upload_service: Option<String>,
/// }
/// ```
#[proc_macro_derive(ConfigMapping, attributes(config))]
pub fn derive_config_mapping(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    let fields = match &input.data {
        Data::Struct(data) => {
            match &data.fields {
                Fields::Named(fields) => &fields.named,
                _ => panic!("ConfigMapping only supports structs with named fields"),
            }
        }
        _ => panic!("ConfigMapping only supports structs"),
    };

    let mut field_mappings = Vec::new();
    let mut from_map_assignments = Vec::new();
    let mut field_names = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        field_names.push(field_name.clone());
        
        // Parse #[config(...)] attribute
        let mut config_code = field_name_str.clone();
        let mut has_default = false;
        let mut default_str = String::new();
        
        for attr in &field.attrs {
            if attr.path().is_ident("config") {
                if let Meta::List(meta_list) = &attr.meta {
                    let nested = meta_list.tokens.to_string();
                    // Simple parsing for code = "..." and default = "..."
                    for pair in nested.split(',') {
                        let pair = pair.trim();
                        if let Some(code_val) = pair.strip_prefix("code = ") {
                            config_code = code_val.trim_matches('"').to_string();
                        } else if let Some(default_val) = pair.strip_prefix("default = ") {
                            default_str = default_val.trim_matches('"').to_string();
                            has_default = true;
                        }
                    }
                }
            }
        }

        let config_code_literal = config_code.clone();
        
        let default_value_opt = if has_default {
            quote! { Some(#default_str.to_string()) }
        } else {
            quote! { None }
        };
        
        field_mappings.push(quote! {
            common::services::config_mapping::FieldMapping {
                field_name: #field_name_str.to_string(),
                config_code: #config_code_literal.to_string(),
                default_value: #default_value_opt,
            }
        });

        // Generate assignment logic based on field type
        let ty = &field.ty;
        
        // Check if it's Option<String>, Option<i32>, etc.
        let assignment = if is_option_string(ty) {
            if has_default {
                quote! {
                    obj.#field_name = map.get(#config_code_literal)
                        .map(|s| s.clone())
                        .or_else(|| Some(#default_str.to_string()));
                }
            } else {
                quote! {
                    obj.#field_name = map.get(#config_code_literal)
                        .map(|s| s.clone());
                }
            }
        } else if is_option_i32(ty) {
            if has_default {
                quote! {
                    obj.#field_name = map.get(#config_code_literal)
                        .and_then(|s| s.parse::<i32>().ok())
                        .or_else(|| #default_str.parse::<i32>().ok());
                }
            } else {
                quote! {
                    obj.#field_name = map.get(#config_code_literal)
                        .and_then(|s| s.parse::<i32>().ok());
                }
            }
        } else if is_option_bool(ty) {
            if has_default {
                quote! {
                    obj.#field_name = map.get(#config_code_literal)
                        .and_then(|s| s.parse::<bool>().ok())
                        .or_else(|| #default_str.parse::<bool>().ok());
                }
            } else {
                quote! {
                    obj.#field_name = map.get(#config_code_literal)
                        .and_then(|s| s.parse::<bool>().ok());
                }
            }
        } else {
            // Default: assume String
            if has_default {
                quote! {
                    obj.#field_name = map.get(#config_code_literal)
                        .map(|s| s.clone())
                        .or_else(|| Some(#default_str.to_string()));
                }
            } else {
                quote! {
                    obj.#field_name = map.get(#config_code_literal)
                        .map(|s| s.clone());
                }
            }
        };
        
        from_map_assignments.push(assignment);
    }

    let expanded = quote! {
        impl common::services::config_mapping::ConfigMapping for #name {
            fn field_mappings() -> Vec<common::services::config_mapping::FieldMapping> {
                vec![
                    #(#field_mappings),*
                ]
            }

            fn from_config_map(map: std::collections::HashMap<String, String>) -> Self {
                let mut obj = Self::default();
                #(#from_map_assignments)*
                obj
            }

            fn cache_key() -> String {
                format!("app_config:{}", stringify!(#name))
            }
        }

        impl Default for #name {
            fn default() -> Self {
                Self {
                    #(#field_names : None),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

// Helper functions to check field types
fn is_option_string(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(syn::Type::Path(inner_path))) = args.args.first() {
                        if let Some(inner_segment) = inner_path.path.segments.last() {
                            return inner_segment.ident == "String";
                        }
                    }
                }
            }
        }
    }
    false
}

fn is_option_i32(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(syn::Type::Path(inner_path))) = args.args.first() {
                        if let Some(inner_segment) = inner_path.path.segments.last() {
                            return inner_segment.ident == "i32";
                        }
                    }
                }
            }
        }
    }
    false
}

fn is_option_bool(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(syn::Type::Path(inner_path))) = args.args.first() {
                        if let Some(inner_segment) = inner_path.path.segments.last() {
                            return inner_segment.ident == "bool";
                        }
                    }
                }
            }
        }
    }
    false
}
