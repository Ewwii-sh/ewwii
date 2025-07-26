use iirhai::parser::ParseConfig;
use iirhai::ipc_manager::IpcManager;
use std::fs;

fn main() {
    let input = fs::read_to_string("examples/eww-bar/ewwii.rhai") // run from root of ewwii/
        .expect("Should have been able to read the file");

    let mut config = ParseConfig::new();
    let result = config.parse_widget_code(&input);

    println!("Raw result: {:#?}", result);
    let manager = IpcManager::new(result.expect("Failed to pass result to IpcManager"));
    println!("JSON result: {:#?}", manager.transpile_to_json());

    /* transpiler stuff */
    // match parse_widget_code(&input) {
    //     Ok(output) => {
    //         println!("{}", output); // optional
    //         fs::write("./eww/eww.yuck", output)
    //             .expect("Should be able to write the file");
    //     }
    //     Err(e) => eprintln!("Error: {e}"),
    // }
}
