// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;

mod models;
mod chat;
mod cache;
mod handlers;

fn main() {
    #[cfg(debug_assertions)]
    {
        env::set_var("RUST_LOG", "info");
        env_logger::init();
    }

    let app = tauri::Builder::default()
        .setup(|_| Ok(()))
        .invoke_handler(tauri::generate_handler![handlers::chat, handlers::fetch_models])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|_app_handle, event| match event {
        tauri::RunEvent::Ready => {}
        tauri::RunEvent::ExitRequested { api, .. } => {
            cache::save_frequencies();
            api.prevent_exit();
        }
        _ => {}
    });
}
