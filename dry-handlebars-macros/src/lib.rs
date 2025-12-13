mod parser;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use regex::Regex;
use std::collections::{HashSet, HashMap};
use std::fs;
use std::path::Path;
use syn::{LitStr, parse_macro_input, parse::Parse, parse::ParseStream, Token};
use walkdir::WalkDir;
use crate::parser::compiler::{Compiler, Options};
use crate::parser::block::add_builtins;

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

fn generate_code_for_content(name: &str, content: &str, path_for_include: Option<&str>) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let struct_name_str = name.replace("-", "_");
    let struct_name = format_ident!("{}", struct_name_str);

    let mut content = content.to_string();

    // Flatten nested variables: {{ obj.title }} -> {{ obj_title }}
    let re_flatten = Regex::new(r"\{\{\s*([a-zA-Z0-9_]+(?:\.[a-zA-Z0-9_]+)+)\s*\}\}").unwrap();
    content = re_flatten.replace_all(&content, |caps: &regex::Captures| {
        let full_match = &caps[0];
        let var_name = &caps[1];
        let new_var_name = var_name.replace(".", "_");
        full_match.replace(var_name, &new_var_name)
    }).to_string();

    // Compile template
    let mut block_map = HashMap::new();
    add_builtins(&mut block_map);
    let options = Options {
        root_var_name: Some("self"),
        write_var_name: "f",
    };
    let compiler = Compiler::new(options, block_map);
    let rust_code = compiler.compile(&content).expect("Failed to compile template");
    let render_body: proc_macro2::TokenStream = rust_code.code.parse().expect("Failed to parse generated code");

    // Extract variables
    let re = Regex::new(r"\{\{\s*([a-zA-Z0-9_]+)(?:[\.\[][a-zA-Z0-9_\.\[\]]*)?\s*\}\}").unwrap();
    let mut vars = HashSet::new();
    for cap in re.captures_iter(&content) {
        vars.insert(cap[1].to_string());
    }
    let mut sorted_vars: Vec<_> = vars.into_iter().collect();
    sorted_vars.sort();

    let type_params: Vec<_> = (0..sorted_vars.len())
        .map(|i| format_ident!("T{}", i))
        .collect();
    let field_defs = sorted_vars.iter().zip(&type_params).map(|(v, t)| {
        let name = format_ident!("{}", v);
        quote! { pub #name: #t }
    });

    let new_args = sorted_vars.iter().zip(&type_params).map(|(v, t)| {
        let name = format_ident!("{}", v);
        quote! { #name: #t }
    });

    let field_inits = sorted_vars.iter().map(|v| {
        let name = format_ident!("{}", v);
        quote! { #name }
    });


    let method_name_str = to_snake_case(&struct_name_str);
    let method_name = format_ident!("{}", method_name_str);

    // Clone args for method signature
    let method_args = sorted_vars.iter().zip(&type_params).map(|(v, t)| {
        let name = format_ident!("{}", v);
        quote! { #name: #t }
    });

    let call_args = sorted_vars.iter().map(|v| {
        let name = format_ident!("{}", v);
        quote! { #name }
    });

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
    generate_code_for_content(&file_stem, &content, Some(&path_str))
}

struct StrInput {
    name: LitStr,
    content: LitStr,
}

impl Parse for StrInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let content: LitStr = input.parse()?;
        Ok(StrInput { name, content })
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
        if path.is_file() && path.extension().map_or(false, |ext| ext == "hbs") {
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
        return syn::Error::new(
            file_lit.span(),
            format!("File not found: {:?}", path),
        )
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
    let StrInput { name, content } = parse_macro_input!(input as StrInput);
    let (struct_def, function_def) = generate_code_for_content(&name.value(), &content.value(), None);

    let expanded = quote! {
        #struct_def
        #function_def
    };

    TokenStream::from(expanded)
}
