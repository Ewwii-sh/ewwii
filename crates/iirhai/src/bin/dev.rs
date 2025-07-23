use iirhai::parse_widget_code;

fn main() {
    let input = r#"
        box("vertical", [
            label("CPU Usage"),
            row([ label("Load:"), label("75%") ])
        ])
    "#;

    let result = parse_widget_code(input);
    println!("Result: {:#?}", result);
}
