use std::fs;
use std::path::Path;
use toml::Value;
use syn::{visit::Visit, File};
use syn::{
    ItemEnum, ItemFn, ItemImpl, ItemMod, ItemStruct, ItemTrait,
};
use walkdir::WalkDir;
use quote::{quote,ToTokens};
mod test;


fn main() {
    // Path to the Cargo.toml file
    let cargo_toml_path = "./Cargo.toml";
    let cargo_toml_content = fs::read_to_string(cargo_toml_path).expect("Failed to read Cargo.toml");

    // Parse Cargo.toml
    let cargo_toml: Value = cargo_toml_content.parse().expect("Failed to parse Cargo.toml");
    println!("Parsed Cargo.toml: {:#?}", cargo_toml);

    let mut paths_to_parse = Vec::new();

    // Check if there is a library specified
    if let Some(lib) = cargo_toml.get("lib").and_then(|v| v.as_table()) {
        if let Some(path) = lib.get("path").and_then(|v| v.as_str()) {
            paths_to_parse.push(path.to_string());
        }
    } else {
        // Default library path
        paths_to_parse.push("src/lib.rs".to_string());
    }

    // Check for binaries
    if let Some(bins) = cargo_toml.get("bin").and_then(|v| v.as_array()) {
        for bin in bins {
            if let Some(path) = bin.as_table().and_then(|b| b.get("path").and_then(|p| p.as_str())) {
                paths_to_parse.push(path.to_string());
            }
        }
    } else {
        // Default binary path
        paths_to_parse.push("src/main.rs".to_string());
        // Optionally, scan src/bin/ directory for any binaries
        if Path::new("src/bin").is_dir() {
            for entry in WalkDir::new("src/bin").into_iter().filter_map(|e| e.ok()) {
                if entry.path().extension().map_or(false, |e| e == "rs") {
                    paths_to_parse.push(entry.path().to_str().unwrap().to_string());
                }
            }
        }
    }


    // Process each relevant Rust source file
    for path_str in paths_to_parse {
        let path = Path::new(&path_str);
        if path.exists() {
            let content = fs::read_to_string(path).expect("Failed to read file");
            let ast = syn::parse_file(&content).expect("Failed to parse file to AST");

            println!("Successfully parsed AST for: {}", path.display());
            let mut visitor = MyAstVisitor;
            visitor.visit_file(&ast);
        } else {
            println!("Specified path does not exist: {}", path.display());
        }
    }
}

struct MyAstVisitor;

impl<'ast> Visit<'ast> for MyAstVisitor {
    
        // Function definitions
        fn visit_item_fn(&mut self, i: &'ast ItemFn) {
            println!("Visiting a function: {}", i.sig.ident);

            let block = &i.block;

            // Using the quote macro to turn the syn::Block back into Rust code
            let generated_code = quote! { #i };

            let a : File = syn::parse2(generated_code).unwrap();
            let p = prettyplease::unparse(&a);
            println!("{}",p);

            // Print the generated code
            // println!("{}", generated_code);

            syn::visit::visit_item_fn(self, i); // Continue traversal
        }
    
        // Struct definitions
        fn visit_item_struct(&mut self, i: &'ast ItemStruct) {
            println!("Visiting a struct: {}", i.ident);
            syn::visit::visit_item_struct(self, i);
        }
    
        // Enum definitions
        fn visit_item_enum(&mut self, i: &'ast ItemEnum) {
            println!("Visiting an enum: {}", i.ident);
            syn::visit::visit_item_enum(self, i);
        }
    
        // Modules
        fn visit_item_mod(&mut self, i: &'ast ItemMod) {
            println!("Visiting a module: {}", i.ident);
            syn::visit::visit_item_mod(self, i);
        }
    
        // Traits
        fn visit_item_trait(&mut self, i: &'ast ItemTrait) {
            println!("Visiting a trait: {}", i.ident);
            syn::visit::visit_item_trait(self, i);
        }
    
        // Implementations
        fn visit_item_impl(&mut self, i: &'ast ItemImpl) {
            if let Some((_, path, _)) = &i.trait_ {
                println!("Visiting an implementation of trait: {}", quote::ToTokens::to_token_stream(path));
            } else {
                println!("Visiting an implementation of a struct or enum");
            }
            syn::visit::visit_item_impl(self, i);
        }
    
        // Additional methods can be implemented similarly
    }