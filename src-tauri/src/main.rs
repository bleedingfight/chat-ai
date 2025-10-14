// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod controller;
mod model;
mod view;
use crate::controller::basic_handlers::*;
use crate::model::model_cache as cache;
use std::env;
fn main() {
    #[cfg(debug_assertions)]
    {
        env::set_var("RUST_LOG", "info");
        env_logger::init();
    }
    // 修复wayland下nvidia显示错误
    std::env::set_var("__GL_THREADED_OPTIMIZATIONS", "0");
    std::env::set_var("__NV_DISABLE_EXPLICIT_SYNC", "1");

    let app = tauri::Builder::default()
        .setup(|_| Ok(()))
        .invoke_handler(tauri::generate_handler![
            chat,
            fetch_models,
            get_cache_directory
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|_app_handle, event| match event {
        tauri::RunEvent::Ready => {}
        tauri::RunEvent::ExitRequested { api, .. } => {
            let cache_dir = cache::get_cache_dir();
            cache::save_frequencies(cache_dir.join("frequency.json"));
            api.prevent_exit();
        }
        _ => {}
    });
}
