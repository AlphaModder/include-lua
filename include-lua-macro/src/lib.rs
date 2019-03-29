extern crate proc_macro;

use std::{env, path::{self, PathBuf}};
use proc_macro_hack::proc_macro_hack;
use quote::quote;
use syn::{
    parse_macro_input, Result, LitStr, Token,
    parse::{Parse, ParseStream}, 
    export::{Span, TokenStream2 as TokenStream}
};
use walkdir::WalkDir;


#[proc_macro_hack]
pub fn include_lua(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_macro_input!(input as IncludeLua).expand().into()
}

struct IncludeLua(LitStr, LitStr);

impl IncludeLua {
    fn expand(self) -> TokenStream {
        let manifest_dir: PathBuf = env::var("CARGO_MANIFEST_DIR").expect("Could not locate active Cargo.toml!").into();
        let lua_dir = manifest_dir.join("src").join(self.0.value());
        let modules = WalkDir::new(&lua_dir).into_iter().filter_map(|entry| {
            match entry {
                Ok(ref entry) if entry.file_type().is_file() => {
                    let path = entry.path().strip_prefix(&lua_dir).expect("Reached file outside of lua directory???");
                    if path.extension() == Some("lua".as_ref()) {
                        let module = if path.parent().is_some() && path.file_stem().expect("Missing file name!") == &"init".as_ref() {
                            path.parent().unwrap().to_str().map(|s| s.replace(path::MAIN_SEPARATOR, "."))
                        } 
                        else {
                            // Do paths with a different separator show up? If so, fix this.
                            let mut s = path.to_str().map(|s| s.replace(path::MAIN_SEPARATOR, "."));
                            s.as_mut().map(|s| s.truncate(s.len() - 4));
                            s
                        };
                        return module.map(|module| (module, path.to_owned()))
                    }
                    None
                }
                Err(e) => panic!("An error occured while searching for lua modules: {}", e),
                _ => None,
            }
        });

        let add_files = modules.map(|(module, path)| {
            let module = LitStr::new(&module, Span::call_site());
            let path = LitStr::new(&PathBuf::from(self.0.value()).join(path).to_string_lossy(), Span::call_site());
            quote! { files.insert(#module.to_string(), include_str!(#path).to_string()) }
        });

        let name = &self.1;
        quote! { {
            #[allow(unknown_lints)]
            #[cfg_attr(feature = "cargo-clippy", allow(useless_attribute))]
            #[allow(rust_2018_idioms)]
            extern crate include_lua as _include_lua;

            let mut files = ::std::collections::HashMap::<String, String>::new();
            #(#add_files;)*
            _include_lua::LuaModules::__new(files, #name)
        } }
    }
}

impl Parse for IncludeLua {
    fn parse(input: ParseStream) -> Result<Self> {
        let (path_str, name) = {
            let s1: LitStr = input.parse()?;
            (s1.clone(), if let Err(_) = input.parse::<Token![:]>() { s1 } else { input.parse()? })
        };
        if !input.is_empty() { return Err(input.error("Unknown token in include_lua invocation!")) }
        Ok(IncludeLua(path_str, name))
    }
}
