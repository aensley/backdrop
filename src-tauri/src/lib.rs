pub mod cli;
pub mod commands;
pub mod config;
pub mod image;
pub mod screen;
pub mod sources;
pub mod timer;
pub mod wallpaper;

pub fn run_gui() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::update,
            commands::set_source,
            commands::set_time,
            commands::get_status,
            commands::random_wallpaper,
            commands::enable_timer,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run tauri application");
}

pub fn run_cli(args: Vec<String>) {
    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    if let Err(e) = rt.block_on(cli::dispatch(args)) {
        eprintln!("backdrop: {e}");
        std::process::exit(1);
    }
}
