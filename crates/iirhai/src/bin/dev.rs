use iirhai::parser::parse_widget_code;
use std::fs;

fn main() {
    let input = fs::read_to_string("examples/eww-bar/eww.rhai") // run from root of ewwii/
        .expect("Should have been able to read the file");

    let result = parse_widget_code(&input);

    println!("{:#?}", result);

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
