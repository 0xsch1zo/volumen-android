use std::env;

use log::LevelFilter;

use crate::{
    application_error::{ApplicationResultExt, FrontendError, LoggedApplicationResultExt},
    net::librus_api::LibrusApi,
};

mod application_error;
mod common;
mod net;

#[cfg(debug_assertions)]
const fn is_debug() -> bool {
    true
}

#[cfg(not(debug_assertions))]
const fn is_debug() -> bool {
    false
}

type Result<T, E = FrontendError> = std::result::Result<T, E>;

#[tauri::command]
async fn greet(name: String) -> Result<String> {
    LibrusApi::with_authorized()
        .await
        .into_app_result()
        .log_on_err()?
        .mobile_login("test", "test")
        .await
        .into_app_result()
        .log_on_err()?;
    Ok(name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut logger = env_logger::builder();
    match env::var("RUST_LOG") {
        Ok(_) => &mut logger,
        Err(env::VarError::NotPresent) => logger.filter_level(LevelFilter::Debug),
        Err(_) => &mut logger, // if it's some other error it's not our responsiblilty to handle it
    }
    .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
