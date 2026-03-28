use std::env;

use log::LevelFilter;
use tauri::{async_runtime::Mutex, Manager};

use crate::{error::FrontendError, state::AppStatesInner};

mod cache;
mod commands;
mod common;
mod error;
mod net;
mod repositories;
mod state;
mod sync;

// TODO: ENABLE COMPRESSION ON RELEASE

#[cfg(debug_assertions)]
const fn is_debug() -> bool {
    true
}

#[cfg(not(debug_assertions))]
const fn is_debug() -> bool {
    false
}

type Result<T, E = FrontendError> = std::result::Result<T, E>;

// TODO: handle logs better for release
#[cfg(target_os = "android")]
fn init_logging() {
    use android_logger::Config;

    android_logger::init_once(Config::default().with_max_level(LevelFilter::Trace));
}

#[cfg(not(target_os = "android"))]
fn init_logging() {
    let mut logger = env_logger::builder();
    match env::var("RUST_LOG") {
        Err(env::VarError::NotPresent) => logger.filter_level(LevelFilter::Debug),
        _ => &mut logger,
    }
    .init();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logging();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_m3::init())
        .setup(|app| {
            let state = Mutex::new(AppStatesInner::try_new()?);
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::login,
            commands::accounts,
            commands::select_account,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
