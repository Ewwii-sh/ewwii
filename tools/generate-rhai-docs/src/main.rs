use rhai::{Engine, module_resolvers::StaticModuleResolver};
use std::{env, fs, path::Path};
use iirhai::providers;
use rhai_autodocs::{export::options, generate::mdbook};

fn generate_docs(engine: &Engine, path: &str, filename: &str, include_std: bool) {
    let docs = options()
        .include_standard_packages(include_std)
        .export(engine)
        .expect("failed to generate documentation");

    // Generate markdown documentation content
    let docs_content = mdbook().generate(&docs).unwrap();

    if docs_content.is_empty() {
        eprintln!("No documentation generated for {}.", filename);
        return;
    }

    // Combine all module docs into one
    let full_docs = docs_content.into_iter().map(|(_, doc)| doc).collect::<Vec<String>>().join("\n");

    // Write documentation to markdown file
    let file_path = Path::new(path).join(format!("{}.md", filename));
    fs::write(&file_path, full_docs).expect("failed to write documentation");
    println!("Documentation generated at: {}", file_path.display());
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = if args.len() > 1 {
        &args[1]
    } else {
        "./docs/src/modules"
    };

    // engine/resolver
    let engine = Engine::new();
    let mut resolver = StaticModuleResolver::new();

    // Generate global.md (full docs)
    generate_docs(&engine, path, "global", true);

    // Generate stdlib.md docs (custom stdlib)
    resolver.clear();
    let mut engine = Engine::new(); // recreate engine to reset state
    providers::register_stdlib(&mut resolver);
    for (module_name, module) in resolver.iter() {
        engine.register_static_module(module_name, module.clone());
    }
    generate_docs(&engine, path, "stdlib", false);

    // Generate apilib.md docs
    resolver.clear();
    let mut engine = Engine::new(); // recreate engine to reset state
    providers::register_apilib(&mut resolver);
    for (module_name, module) in resolver.iter() {
        engine.register_static_module(module_name, module.clone());
    }
    generate_docs(&engine, path, "apilib", false);

    println!("Docs generation completed.");
}
