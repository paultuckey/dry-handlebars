mod parser;

use crate::parser::block::add_builtins;
use crate::parser::compiler::{Compiler, Options, Usage};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use syn::{LitStr, Token, parse::Parse, parse::ParseStream, parse_macro_input};
use walkdir::WalkDir;

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            for lc in c.to_lowercase() {
                result.push(lc);
            }
        } else {
            result.push(c);
        }
    }
    result
}

fn generate_code_for_content(
    name: &str,
    content: &str,
    path_for_include: Option<&str>,
    mut mappings: HashMap<String, syn::Type>,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let struct_name_str = name.replace("-", "_");
    let struct_name = format_ident!("{}", struct_name_str);

    let mut content = content.to_string();

    let mut block_map = HashMap::new();
    add_builtins(&mut block_map);

    let temp_options = Options {
        root_var_name: None,
        write_var_name: "f",
        variable_types: HashMap::new(),
    };
    let temp_compiler = Compiler::new(temp_options, block_map.clone());
    let usages = temp_compiler.scan(&content).unwrap_or_default();

    for (name, usage) in &usages {
        if !mappings.contains_key(name)
            && let Usage::Boolean = usage
        {
            let bool_ty: syn::Type = syn::parse_quote! { bool };
            mappings.insert(name.clone(), bool_ty);
        }
    }

    // Detect variables used in {{#if var}}
    let re_if = Regex::new(r"\{\{#if\s+([a-zA-Z0-9_]+)\s*\}\}").unwrap();
    let mut if_vars = HashSet::new();
    for cap in re_if.captures_iter(&content) {
        if_vars.insert(cap[1].to_string());
    }

    // Update mappings for if_vars to be Option<T>
    for var in &if_vars {
        if let Some(ty) = mappings.get(var) {
            // Check if already Option
            let ty_str = quote! { #ty }.to_string();
            if !ty_str.contains("Option") && ty_str != "bool" {
                let new_ty: syn::Type = syn::parse_quote! { Option<#ty> };
                mappings.insert(var.clone(), new_ty);
            }
        }
    }

    // Flatten nested variables: {{ obj.title }} -> {{ obj_title }}
    let re_flatten = Regex::new(r"\{\{\s*([a-zA-Z0-9_]+(?:\.[a-zA-Z0-9_]+)+)\s*\}\}").unwrap();
    let mut mapping = HashMap::new();
    content = re_flatten
        .replace_all(&content, |caps: &regex::Captures| {
            let full_match = &caps[0];
            let var_name = &caps[1];

            let parts: Vec<&str> = var_name.split('.').collect();
            let root = parts[0];
            if mappings.contains_key(root) {
                return full_match.to_string();
            }

            let new_var_name = var_name.replace(".", "_");
            mapping.insert(new_var_name.clone(), var_name.to_string());
            full_match.replace(var_name, &new_var_name)
        })
        .to_string();

    // Prepare variable types for Compiler
    let mut variable_types = HashMap::new();
    for (k, v) in &mappings {
        variable_types.insert(k.clone(), quote! { #v }.to_string());
    }

    // Compile template
    let options = Options {
        root_var_name: Some("self"),
        write_var_name: "f",
        variable_types,
    };
    let compiler = Compiler::new(options, block_map);
    let rust_code = compiler
        .compile(&content)
        .expect("Failed to compile template");
    let render_body: proc_macro2::TokenStream = rust_code
        .code
        .parse()
        .expect("Failed to parse generated code");

    // Extract variables
    // Use top_level_vars from compiler
    let mut vars_set = HashSet::new();
    for var in rust_code.top_level_vars {
        let root = var.split('.').next().unwrap();
        vars_set.insert(root.to_string());
    }

    // Also include variables found in {{#if}} that might not be in {{}}
    for var in if_vars {
        vars_set.insert(var);
    }

    let mut sorted_vars = Vec::new();
    let mut seen_roots = HashSet::new();

    // Use usages to determine order
    for (name, _) in &usages {
        let root = name.split('.').next().unwrap().to_string();
        if vars_set.contains(&root) && !seen_roots.contains(&root) {
            sorted_vars.push(root.clone());
            seen_roots.insert(root);
        }
    }

    // Add any remaining vars
    let mut remaining_vars: Vec<_> = vars_set
        .into_iter()
        .filter(|v| !seen_roots.contains(v))
        .collect();
    remaining_vars.sort();
    sorted_vars.extend(remaining_vars);

    let mut type_params = Vec::new();
    let mut field_defs = Vec::new();
    let mut new_args = Vec::new();
    let mut field_inits = Vec::new();
    let mut method_args = Vec::new();
    let mut call_args = Vec::new();

    let mut generic_param_index: usize = 0;

    for v in &sorted_vars {
        let name = format_ident!("{}", v);

        if let Some(mapped_type) = mappings.get(v) {
            field_defs.push(quote! { pub #name: #mapped_type });
            new_args.push(quote! { #name: #mapped_type });
            field_inits.push(quote! { #name });
            method_args.push(quote! { #name: #mapped_type });
            call_args.push(quote! { #name });
        } else {
            let t_param = format_ident!("T{}", generic_param_index);
            generic_param_index += 1;

            type_params.push(t_param.clone());

            field_defs.push(quote! { pub #name: #t_param });
            new_args.push(quote! { #name: #t_param });
            field_inits.push(quote! { #name });
            method_args.push(quote! { #name: #t_param });
            call_args.push(quote! { #name });
        }
    }

    let method_name_str = to_snake_case(&struct_name_str);
    let method_name = format_ident!("{}", method_name_str);

    let function_def = quote! {
        pub fn #method_name<#(#type_params: std::fmt::Display),*>(#(#method_args),*) -> #struct_name<#(#type_params),*> {
            #struct_name::new(#(#call_args),*)
        }
    };

    let include_bytes_stmt = if let Some(path_str) = path_for_include {
        quote! {
            // ensure the compiler is aware the output is linked to the source so that any changes
            // to the hbs file will trigger a recompilation
            const _: &[u8] = include_bytes!(#path_str);
        }
    } else {
        quote! {}
    };

    let struct_def = quote! {
        #include_bytes_stmt

        pub struct #struct_name<#(#type_params),*> {
            #(#field_defs),*
        }

        impl<#(#type_params: std::fmt::Display),*> #struct_name<#(#type_params),*> {
            pub fn new(#(#new_args),*) -> Self {
                Self {
                    #(#field_inits),*
                }
            }

            pub fn render(&self) -> String {
                use std::fmt::Write;
                let mut f = String::new();
                let mut render_inner = || -> std::fmt::Result {
                    #render_body
                    Ok(())
                };
                render_inner().unwrap();
                f
            }
        }
    };

    (struct_def, function_def)
}

fn generate_code_for_file(path: &Path) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let file_stem = path.file_stem().unwrap().to_string_lossy();
    let path_str = path.to_string_lossy();
    let content = fs::read_to_string(path).expect("Failed to read file");
    generate_code_for_content(&file_stem, &content, Some(&path_str), HashMap::new())
}

struct StrInput {
    name: LitStr,
    content: LitStr,
    mappings: Vec<(String, syn::Type)>,
}

impl Parse for StrInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let content: LitStr = input.parse()?;

        let mut mappings = Vec::new();
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            while !input.is_empty() {
                let content;
                syn::parenthesized!(content in input);
                let key: LitStr = content.parse()?;
                content.parse::<Token![,]>()?;
                let ty: syn::Type = content.parse()?;
                mappings.push((key.value(), ty));

                if input.peek(Token![,]) {
                    input.parse::<Token![,]>()?;
                }
            }
        }
        Ok(StrInput {
            name,
            content,
            mappings,
        })
    }
}

#[proc_macro]
pub fn dry_handlebars_directory(input: TokenStream) -> TokenStream {
    let dir_lit = parse_macro_input!(input as LitStr);
    let dir_str = dir_lit.value();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let root_path = Path::new(&manifest_dir).join(&dir_str);

    if !root_path.exists() {
        return syn::Error::new(
            dir_lit.span(),
            format!("Directory not found: {:?}", root_path),
        )
        .to_compile_error()
        .into();
    }

    let mut structs = Vec::new();
    let mut functions = Vec::new();

    for entry in WalkDir::new(&root_path) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "hbs") {
            let (struct_def, function_def) = generate_code_for_file(path);
            structs.push(struct_def);
            functions.push(function_def);
        }
    }

    let expanded = quote! {
        #(#structs)*
        #(#functions)*
    };

    TokenStream::from(expanded)
}

#[proc_macro]
pub fn dry_handlebars_file(input: TokenStream) -> TokenStream {
    let file_lit = parse_macro_input!(input as LitStr);
    let file_str = file_lit.value();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let path = Path::new(&manifest_dir).join(&file_str);

    if !path.exists() {
        return syn::Error::new(file_lit.span(), format!("File not found: {:?}", path))
            .to_compile_error()
            .into();
    }

    let (struct_def, function_def) = generate_code_for_file(&path);

    let expanded = quote! {
        #struct_def
        #function_def
    };

    TokenStream::from(expanded)
}

#[proc_macro]
pub fn dry_handlebars_str(input: TokenStream) -> TokenStream {
    let StrInput {
        name,
        content,
        mappings,
    } = parse_macro_input!(input as StrInput);
    let mappings_map: HashMap<String, syn::Type> = mappings.into_iter().collect();
    let (struct_def, function_def) =
        generate_code_for_content(&name.value(), &content.value(), None, mappings_map);

    let expanded = quote! {
        #struct_def
        #function_def
    };

    TokenStream::from(expanded)
}
