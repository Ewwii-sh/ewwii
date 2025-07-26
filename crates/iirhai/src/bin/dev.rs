use iirhai::{
    parser::ParseConfig,
    ipc_manager::IpcManager,
    daemon::IIRhaiDaemon
};
use std::{
    fs,
    path::PathBuf
};

fn main() {
    let input = fs::read_to_string("examples/eww-bar/ewwii.rhai") // run from root of ewwii/
        .expect("Should have been able to read the file");

    let mut config = ParseConfig::new();
    let result = config.parse_widget_code(&input);

    println!("Raw result: {:#?}", result);
    let manager = IpcManager::new(result.expect("Failed to pass result to IpcManager"));
    println!("JSON result: {:#?}", manager.transpile_to_json());

    start_daemon();
}

#[tokio::main]
async fn start_daemon() -> Result<(), anyhow::Error> {
    let socket_path = PathBuf::from("/tmp/iirhai.sock");
    let config_path = PathBuf::from("examples/eww-bar/ewwii.rhai");
    let daemon = IIRhaiDaemon::new(socket_path, config_path);

    daemon.run_server().await.expect("Failed to run the iirhai daemon.");
    Ok(())
}