use std::env;

use log::LevelFilter;

use crate::{
    application_error::{ApplicationResultExt, FrontendError, LoggedApplicationResultExt},
    net::synergia_api::SynergiaApi,
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
async fn send(login: String, password: String) -> Result<String> {
    SynergiaApi::with_authorized()
        .await
        .into_app_result()
        .log_on_err()?
        .login(&login, &password)
        .await
        .into_app_result()
        .log_on_err()?;
    Ok(login)
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
        .invoke_handler(tauri::generate_handler![send])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
