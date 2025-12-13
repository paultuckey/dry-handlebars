mod hbs;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use regex::Regex;
use std::collections::{HashSet, HashMap};
use std::fs;
use std::path::Path;
use syn::{LitStr, parse_macro_input};
use walkdir::WalkDir;
use crate::hbs::compiler::{Compiler, Options};
use crate::hbs::block::add_builtins;

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
    let re = Regex::new(r"\{\{\s*([a-zA-Z0-9_]+)(?:[\.\[][a-zA-Z0-9_\.\[\]]*)?\s*\}\}").unwrap();

    for entry in WalkDir::new(&root_path) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "hbs") {
            let file_stem = path.file_stem().unwrap().to_string_lossy();
            let struct_name_str = file_stem.replace("-", "_");
            let struct_name = format_ident!("{}", struct_name_str);

            let path_str = path.to_string_lossy();

            let mut content = fs::read_to_string(path).expect("Failed to read file");

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


            structs.push(quote! {
                // ensure the compiler is aware the output is linked to the source so that any changes
                // to the hbs file will trigger a recompilation
                const _: &[u8] = include_bytes!(#path_str);

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
            });
        }
    }

    let expanded = quote! {
        #(#structs)*
    };

    TokenStream::from(expanded)
}
