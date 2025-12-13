mod hbs;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use syn::{LitStr, parse_macro_input};
use walkdir::WalkDir;

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
    let re = Regex::new(r"\{\{\s*([a-zA-Z0-9_]+)\s*\}\}").unwrap();

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

            let content = fs::read_to_string(path).expect("Failed to read file");

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

            let template_str = &content;

            structs.push(quote! {
                // ensure the compiler is aware the output is linked to the source so that any changes
                // to the hbs file will trigger a recompilation
                const _: &[u8] = include_bytes!(#path_str);

                #[derive(dry_handlebars::serde::Serialize)]
                #[serde(crate = "dry_handlebars::serde")]
                pub struct #struct_name<#(#type_params),*> {
                    #(#field_defs),*
                }

                impl<#(#type_params: dry_handlebars::serde::Serialize),*> #struct_name<#(#type_params),*> {
                    pub fn new(#(#new_args),*) -> Self {
                        Self {
                            #(#field_inits),*
                        }
                    }

                    pub fn render(&self) -> String {
                        use dry_handlebars::handlebars::Handlebars;
                        let mut reg = Handlebars::new();
                        reg.register_template_string("template", #template_str).expect("Failed to register template");
                        reg.render("template", &self).expect("Failed to render")
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
