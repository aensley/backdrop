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
        .setup(|app| {
            use tauri::Manager;
            let window = app.get_webview_window("main").expect("no main window");
            let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/icon.png"))?;
            window.set_icon(icon)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::update,
            commands::set_source,
            commands::set_time,
            commands::get_status,
            commands::get_image_meta,
            commands::open_url,
            commands::random_wallpaper,
            commands::enable_timer,
            commands::disable_timer,
            commands::set_config_value,
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
