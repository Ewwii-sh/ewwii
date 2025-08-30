use iirhai::providers;
use rhai::{Engine, module_resolvers::StaticModuleResolver};
use rhai_autodocs::{export::options, generate::mdbook};
use std::{env, fs, path::Path};

fn generate_docs(
    engine: &Engine,
    path: &str,
    filename: &str,
    include_std: bool,
    pre_description: &str,
) {
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
    let full_docs =
        docs_content.into_iter().map(|(_, doc)| doc).collect::<Vec<String>>().join("\n");

    // combination of all docs and pre description
    let final_docs = format!("{}\n\n{}", pre_description, full_docs);

    // Write documentation to markdown file
    let file_path = Path::new(path).join(format!("{}.md", filename));
    fs::write(&file_path, final_docs).expect("failed to write documentation");
    println!("Documentation generated at: {}", file_path.display());
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = if args.len() > 1 { &args[1] } else { "./docs/src/modules" };

    // engine/resolver
    let engine = Engine::new();
    let mut resolver = StaticModuleResolver::new();

    // Generate global.md (full docs)
    generate_docs(
        &engine,
        path,
        "global",
        true,
        r#"
        # Global Builtin Rhai Functions

        These functions are built-in and available globally, meaning they can be used directly without any import.

        For example, to get the value of PI, you can simply write:
        
        ```js
        let x = PI();
        ```

        This section covers all the core functions provided by Rhai that are ready to use out of the box.
    "#,
    );

    // Generate stdlib.md docs (custom stdlib)
    resolver.clear();
    let mut engine = Engine::new(); // recreate engine to reset state
    providers::register_stdlib(&mut resolver);
    for (module_name, module) in resolver.iter() {
        engine.register_static_module(module_name, module.clone());
    }
    generate_docs(
        &engine,
        path,
        "stdlib",
        false,
        r#"
        # Std Library Module

        These are all the standard modules in ewwii.

        Each library in this module is under `std::<m>`, where `<m>` is the name of the specific module.
        These modules provide essential functionalities that are will be useful for making widgets.
        They cover tasks like string manipulation, environmental variable manipuation, running shell commands, and more.
    "#,
    );

    // Generate apilib.md docs
    resolver.clear();
    let mut engine = Engine::new(); // recreate engine to reset state
    providers::register_apilib(&mut resolver);
    for (module_name, module) in resolver.iter() {
        engine.register_static_module(module_name, module.clone());
    }
    generate_docs(
        &engine,
        path,
        "apilib",
        false,
        r#"
        # API Library Module

        These are all the API modules available in ewwii.

        Each library in this module is under `api::<m>`, where `<m>` is the name of the specific module.
        
        The API library provides system-level functionality, allowing you to interact with external resources and perform advanced operations. Examples include interacting with Wi-Fi, networking, and more.
    "#,
    );

    println!("Docs generation completed.");
}
